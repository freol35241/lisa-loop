#!/usr/bin/env bash
set -euo pipefail

# Lisa Loop v2 — Spiral development loop for engineering and scientific software
#
# Architecture: Outer spiral (convergence-driven, human-gated) with five phases
# per pass: Refine → DDV Red → Build (Ralph loop) → Execute → Validate
#
# Pass 0 (scoping) runs once with an iterative human refinement loop.
#
# Usage:
#   ./loop.sh scope                  # Run Pass 0 (scoping) only
#   ./loop.sh run [--max-passes N]   # Full spiral (scope if needed, then iterate)
#   ./loop.sh resume                 # Resume from current state
#   ./loop.sh status                 # Print current state and exit

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# --- Configuration -----------------------------------------------------------

# Source project config if it exists
[[ -f "$SCRIPT_DIR/lisa.conf" ]] && source "$SCRIPT_DIR/lisa.conf"

# Claude Code model selection per phase (defaults)
CLAUDE_MODEL_SCOPE="${CLAUDE_MODEL_SCOPE:-opus}"
CLAUDE_MODEL_REFINE="${CLAUDE_MODEL_REFINE:-opus}"
CLAUDE_MODEL_DDV="${CLAUDE_MODEL_DDV:-opus}"
CLAUDE_MODEL_BUILD="${CLAUDE_MODEL_BUILD:-sonnet}"
CLAUDE_MODEL_EXECUTE="${CLAUDE_MODEL_EXECUTE:-opus}"
CLAUDE_MODEL_VALIDATE="${CLAUDE_MODEL_VALIDATE:-opus}"

# Loop limits
MAX_SPIRAL_PASSES="${MAX_SPIRAL_PASSES:-5}"
MAX_RALPH_ITERATIONS="${MAX_RALPH_ITERATIONS:-50}"

# Human review
NO_PAUSE="${NO_PAUSE:-false}"

# Git
NO_PUSH="${NO_PUSH:-false}"

# Output collapsing
COLLAPSE_OUTPUT="${COLLAPSE_OUTPUT:-true}"
# Disable collapsing when stdout is not a terminal (e.g. redirected to a file)
[[ -t 1 ]] || COLLAPSE_OUTPUT=false

# --- Colors -------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# --- Helpers ------------------------------------------------------------------

_ts()         { date '+%H:%M:%S'; }
log_info()    { echo -e "${BLUE}[lisa $(_ts)]${NC} $*"; }
log_success() { echo -e "${GREEN}[lisa $(_ts)]${NC} $*"; }
log_warn()    { echo -e "${YELLOW}[lisa $(_ts)]${NC} $*"; }
log_error()   { echo -e "${RED}[lisa $(_ts)]${NC} $*"; }
log_phase()   { echo -e "${CYAN}[lisa $(_ts)]${NC} ━━━ $* ━━━"; }

_init_agent_stats() {
    AGENT_STATS_FILE=$(mktemp)
    AGENT_RESULT_FILE=$(mktemp)
    echo "0 0 0" > "$AGENT_STATS_FILE"  # tool_count file_writes test_runs
    > "$AGENT_RESULT_FILE"              # empty
}

_update_agent_stat() {
    # Usage: _update_agent_stat <field_index> [increment]
    # field_index: 1=tools, 2=file_writes, 3=test_runs
    local idx="$1"
    local inc="${2:-1}"
    if [[ -f "$AGENT_STATS_FILE" ]]; then
        local s1 s2 s3
        read -r s1 s2 s3 < "$AGENT_STATS_FILE"
        case "$idx" in
            1) s1=$((s1 + inc)) ;;
            2) s2=$((s2 + inc)) ;;
            3) s3=$((s3 + inc)) ;;
        esac
        echo "$s1 $s2 $s3" > "$AGENT_STATS_FILE"
    fi
}

_read_agent_stats() {
    # Sets STAT_TOOLS, STAT_WRITES, STAT_TESTS
    if [[ -f "$AGENT_STATS_FILE" ]]; then
        read -r STAT_TOOLS STAT_WRITES STAT_TESTS < "$AGENT_STATS_FILE"
    else
        STAT_TOOLS=0 STAT_WRITES=0 STAT_TESTS=0
    fi
}

_cleanup_agent_stats() {
    [[ -f "${AGENT_STATS_FILE:-}" ]] && rm -f "$AGENT_STATS_FILE"
    [[ -f "${AGENT_RESULT_FILE:-}" ]] && rm -f "$AGENT_RESULT_FILE"
}

_agent_cleanup() {
    # Restore terminal state if interrupted during agent run
    tput rc 2>/dev/null || true
    tput ed 2>/dev/null || true
    _cleanup_agent_stats
    echo ""
    log_warn "Agent interrupted."
}

_filter_agent_stream() {
    # Process NDJSON stream from `claude -p --output-format stream-json --verbose`
    # and emit human-readable progress lines showing agent tool calls, thinking,
    # and the final result text.
    #
    # When COLLAPSE_OUTPUT=true, stats are counted via temp file and the result
    # text is written to AGENT_RESULT_FILE instead of stdout. The caller
    # (run_agent) handles collapsing and printing the summary + result.
    #
    # Falls back to raw passthrough if jq is not available.
    if ! command -v jq &>/dev/null; then
        cat
        return
    fi
    jq --unbuffered -r '
      if .type == "assistant" then
        [.message.content[]? |
          if .type == "thinking" then
            "THINKING: " + (.thinking // "")
          elif .type == "tool_use" then
            "TOOL: " +
            "\(.name)" +
            (if .name == "Read" then " \(.input.file_path // "")"
             elif .name == "Edit" then " \(.input.file_path // "")"
             elif .name == "Write" then " \(.input.file_path // "")"
             elif .name == "Bash" then " $ \((.input.command // "") | split("\n")[0] | .[0:80])"
             elif .name == "Glob" then " \(.input.pattern // "")"
             elif .name == "Grep" then " \(.input.pattern // "")"
             elif .name == "Task" then " \(.input.description // "")"
             elif .name == "TodoWrite" then ""
             else "" end)
          else empty end
        ] | .[] | select(length > 0)
      elif .type == "result" then
        "RESULT_B64: " + ((.result // "") | @base64)
      else empty end
    ' 2>/dev/null | while IFS= read -r line; do
        if [[ "$line" == THINKING:\ * ]]; then
            local thought="${line#THINKING: }"
            # Truncate long thinking to first 200 chars for terminal readability
            if [[ ${#thought} -gt 200 ]]; then
                thought="${thought:0:200}..."
            fi
            echo -e "    ${DIM}[💭 $(_ts)] ${thought}${NC}"
        elif [[ "$line" == TOOL:\ * ]]; then
            local tool_detail="${line#TOOL: }"
            echo -e "    ${MAGENTA}[🔧 $(_ts)]${NC} ${tool_detail}"
            # Count stats via temp file (works from subshell)
            _update_agent_stat 1
            # Count file writes (Write or Edit tools)
            if [[ "$tool_detail" == Write\ * ]] || [[ "$tool_detail" == Edit\ * ]]; then
                _update_agent_stat 2
            fi
            # Count test runs (Bash with pytest or test)
            if [[ "$tool_detail" == Bash\ *pytest* ]] || [[ "$tool_detail" == Bash\ *test* ]]; then
                _update_agent_stat 3
            fi
        elif [[ "$line" == RESULT_B64:\ * ]]; then
            local result_b64="${line#RESULT_B64: }"
            local result_text
            result_text="$(echo "$result_b64" | base64 -d 2>/dev/null)" || result_text=""
            if [[ -n "$result_text" ]]; then
                if [[ "$COLLAPSE_OUTPUT" == "true" ]]; then
                    # Write result to temp file; run_agent prints it after collapsing
                    echo "$result_text" > "$AGENT_RESULT_FILE"
                else
                    echo ""
                    echo -e "    ${MAGENTA}── Result ──${NC}"
                    sed 's/^/    /' <<< "$result_text"
                    echo -e "    ${MAGENTA}── End ──${NC}"
                    echo ""
                fi
            fi
        fi
    done
    return 0
}

run_agent() {
    # Usage: run_agent <prompt_file> <model> [context_string] [label]
    local prompt_file="$1"
    local model="$2"
    local context="${3:-}"
    local label="${4:-agent}"

    if [[ ! -f "$prompt_file" ]]; then
        log_error "Prompt file not found: $prompt_file"
        exit 1
    fi

    local start_seconds=$SECONDS
    _init_agent_stats

    if [[ "$COLLAPSE_OUTPUT" == "true" ]]; then
        # Print a "running" indicator and save cursor position
        echo -ne "  ${CYAN}▸${NC} ${label} ..."
        echo ""
        tput sc  # save cursor position (start of streaming output area)

        trap _agent_cleanup INT TERM
    else
        log_info "Calling agent with prompt: $prompt_file (model: $model)"
    fi

    # Run the agent, streaming through the filter
    {
        if [[ -n "$context" ]]; then
            echo "$context"
            echo ""
        fi
        cat "$prompt_file"
    } | claude -p --dangerously-skip-permissions --verbose \
               --model "$model" --output-format stream-json \
        | _filter_agent_stream

    local elapsed=$(( SECONDS - start_seconds ))
    _read_agent_stats

    if [[ "$COLLAPSE_OUTPUT" == "true" ]]; then
        trap - INT TERM

        # Collapse streaming output: restore cursor and clear everything below
        tput rc  # restore cursor to saved position
        tput ed  # clear from cursor to end of screen

        # Move up one line to overwrite the "▸ label ..." line
        tput cuu1

        # Build summary
        local summary="${STAT_TOOLS} tools"
        [[ "$STAT_WRITES" -gt 0 ]] && summary+=", ${STAT_WRITES} files written"
        [[ "$STAT_TESTS" -gt 0 ]] && summary+=", ${STAT_TESTS} test runs"

        local icon="✓" icon_color="$GREEN"

        # Print collapsed summary line
        echo -e "  ${icon_color}${icon}${NC} ${label} (${elapsed}s, ${summary})"

        # Print the agent's final result text (this stays visible)
        if [[ -s "$AGENT_RESULT_FILE" ]]; then
            echo -e "    ${MAGENTA}── Result ──${NC}"
            sed 's/^/    /' "$AGENT_RESULT_FILE"
            echo -e "    ${MAGENTA}── End ──${NC}"
        fi
    else
        log_info "Agent finished (${elapsed}s elapsed)"
    fi

    _cleanup_agent_stats
}

git_commit_all() {
    local msg="$1"
    log_info "Staging all changes..."
    git add -A
    if git diff --cached --quiet; then
        log_info "No changes to commit."
        return 1
    fi
    log_info "Committing: $msg"
    git commit -m "$msg"
    log_success "Commit created."
    return 0
}

git_push() {
    if [[ "$NO_PUSH" == "true" || "$NO_PUSH" == "1" ]]; then
        log_info "Skipping push (NO_PUSH=$NO_PUSH)"
        return
    fi
    local branch
    branch="$(git rev-parse --abbrev-ref HEAD)"
    log_info "Pushing to origin/$branch..."
    git push -u origin "$branch"
}

# --- State Management --------------------------------------------------------

STATE_FILE="spiral/current-state.md"

write_state() {
    local pass="$1"
    local phase="$2"
    local status="$3"
    local ralph_iter="${4:-0}"
    mkdir -p spiral
    cat > "$STATE_FILE" <<EOF
# Spiral State
pass: $pass
phase: $phase
status: $status
ralph_iteration: $ralph_iter
EOF
}

read_state() {
    # Sets global variables: STATE_PASS, STATE_PHASE, STATE_STATUS, STATE_RALPH_ITER
    if [[ ! -f "$STATE_FILE" ]]; then
        STATE_PASS=0
        STATE_PHASE="not_started"
        STATE_STATUS="pending"
        STATE_RALPH_ITER=0
        return
    fi
    STATE_PASS=$(grep '^pass:' "$STATE_FILE" | awk '{print $2}')
    STATE_PHASE=$(grep '^phase:' "$STATE_FILE" | awk '{print $2}')
    STATE_STATUS=$(grep '^status:' "$STATE_FILE" | awk '{print $2}')
    STATE_RALPH_ITER=$(grep '^ralph_iteration:' "$STATE_FILE" | awk '{print $2}')
    STATE_PASS="${STATE_PASS:-0}"
    STATE_PHASE="${STATE_PHASE:-not_started}"
    STATE_STATUS="${STATE_STATUS:-pending}"
    STATE_RALPH_ITER="${STATE_RALPH_ITER:-0}"
}

# --- Task Detection -----------------------------------------------------------

_count_uncompleted_tasks() {
    # Count TODO or IN_PROGRESS tasks where pass <= given pass
    local max_pass="$1"
    local plan_file="methodology/plan.md"
    [[ ! -f "$plan_file" ]] && { echo 0; return; }

    awk -v max_pass="$max_pass" '
        /^### Task/ {
            if (in_task && task_pass <= max_pass && (found_todo || found_inprog)) count++
            in_task=1; task_pass=9999; found_todo=0; found_inprog=0
            next
        }
        in_task && /\*\*Pass:\*\*/ {
            line = $0
            sub(/.*\*\*Pass:\*\*[[:space:]]*/, "", line)
            sub(/[^0-9].*/, "", line)
            task_pass = line + 0
        }
        in_task && /\*\*Status:\*\*/ {
            if (index($0, "TODO")) found_todo=1
            if (index($0, "IN_PROGRESS")) found_inprog=1
        }
        END {
            if (in_task && task_pass <= max_pass && (found_todo || found_inprog)) count++
            print count+0
        }
    ' "$plan_file"
}

_count_blocked_tasks() {
    # Count BLOCKED tasks where pass <= given pass
    local max_pass="$1"
    local plan_file="methodology/plan.md"
    [[ ! -f "$plan_file" ]] && { echo 0; return; }

    awk -v max_pass="$max_pass" '
        /^### Task/ {
            if (in_task && task_pass <= max_pass && found_blocked) count++
            in_task=1; task_pass=9999; found_blocked=0
            next
        }
        in_task && /\*\*Pass:\*\*/ {
            line = $0
            sub(/.*\*\*Pass:\*\*[[:space:]]*/, "", line)
            sub(/[^0-9].*/, "", line)
            task_pass = line + 0
        }
        in_task && /\*\*Status:\*\*/ {
            if (index($0, "BLOCKED")) found_blocked=1
        }
        END {
            if (in_task && task_pass <= max_pass && found_blocked) count++
            print count+0
        }
    ' "$plan_file"
}

_all_tasks_done() {
    # Returns 0 (true) if no TODO or IN_PROGRESS tasks remain at or below given pass
    local pass="$1"
    local remaining
    remaining=$(_count_uncompleted_tasks "$pass")
    [[ "$remaining" -eq 0 ]]
}

_has_blocked_tasks() {
    # Returns 0 (true) if any BLOCKED tasks exist at or below given pass
    local pass="$1"
    local blocked_count
    blocked_count=$(_count_blocked_tasks "$pass")
    [[ "$blocked_count" -gt 0 ]]
}

# --- Human Interaction Gates --------------------------------------------------

review_gate() {
    local pass="$1"
    local review_file="spiral/pass-$pass/review-package.md"

    if [[ "$NO_PAUSE" == "true" || "$NO_PAUSE" == "1" ]]; then
        log_warn "Review gate skipped (NO_PAUSE=$NO_PAUSE) — defaulting to CONTINUE"
        return 0
    fi

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  SPIRAL PASS $pass COMPLETE — REVIEW REQUIRED${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    # Extract and display key info from review package if it exists
    if [[ -f "$review_file" ]]; then
        # Current answer (first non-empty line after ## Current Answer)
        local answer
        answer=$(awk '/^## Current Answer/{found=1; next} found && /\S/{print; found=0}' "$review_file")
        if [[ -n "$answer" ]]; then
            echo -e "  ${BOLD}Answer:${NC} $answer"
        fi

        # Convergence status
        local convergence
        convergence=$(grep -E '## Convergence:' "$review_file" | head -1 | sed 's/.*Convergence:[[:space:]]*//')
        if [[ -n "$convergence" ]]; then
            local conv_color="$YELLOW"
            [[ "$convergence" == *CONVERGED* && "$convergence" != *NOT* ]] && conv_color="$GREEN"
            [[ "$convergence" == *DIVERGING* ]] && conv_color="$RED"
            echo -e "  ${BOLD}Convergence:${NC} ${conv_color}${convergence}${NC}"
        fi

        # Test summary
        local tests
        tests=$(grep -E '^DDV:' "$review_file" | head -1)
        if [[ -n "$tests" ]]; then
            echo -e "  ${BOLD}Tests:${NC} $tests"
        fi

        # Sanity checks
        local sanity
        sanity=$(grep -i 'Sanity Checks:' "$review_file" | head -1 | sed 's/.*: *//')
        if [[ -n "$sanity" ]]; then
            echo -e "  ${BOLD}Sanity:${NC} $sanity"
        fi

        # Failures (if any)
        local failures
        failures=$(grep -i 'failure\|FAIL' "$review_file" | grep -v '^#' | grep -v 'None' | head -3)
        if [[ -n "$failures" ]]; then
            echo ""
            echo -e "  ${RED}Failures:${NC}"
            while IFS= read -r f; do
                echo -e "    ${RED}•${NC} $f"
            done <<< "$failures"
        fi

        # Engineering judgment issues from execution
        local ej_issues
        ej_issues=$(awk '/^## Engineering Judgment Issues/{found=1; next} found && /^##/{found=0} found && /\S/ && !/None/' "$review_file" | head -3)
        if [[ -n "$ej_issues" ]]; then
            echo ""
            echo -e "  ${YELLOW}Engineering Judgment Issues:${NC}"
            while IFS= read -r issue; do
                echo -e "    ${YELLOW}•${NC} $issue"
            done <<< "$ej_issues"
        fi

        # Agent recommendation
        local recommendation
        recommendation=$(awk '/^## Recommendation/{found=1; next} found && /\S/{print; found=0}' "$review_file")
        if [[ -n "$recommendation" ]]; then
            echo ""
            echo -e "  ${BOLD}Agent recommends:${NC} $recommendation"
        fi
    else
        echo -e "  ${YELLOW}Review package not found at $review_file${NC}"
    fi

    echo ""
    echo -e "  ${CYAN}Files:${NC}"
    echo "    Review:     spiral/pass-$pass/review-package.md"
    echo "    Execution:  spiral/pass-$pass/execution-report.md"
    echo "    Plots:      plots/REVIEW.md"
    echo ""
    echo -e "  ${GREEN}[A]${NC} ACCEPT — converged, produce final report"
    echo -e "  ${YELLOW}[C]${NC} CONTINUE — next spiral pass"
    echo -e "  ${CYAN}[R]${NC} REDIRECT — provide guidance (opens \$EDITOR)"
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    while true; do
        read -rp "  Your choice [A/C/R]: " choice
        case "${choice^^}" in
            A)
                log_success "ACCEPTED — producing final output."
                return 1
                ;;
            C)
                log_info "CONTINUE — proceeding to next pass."
                return 0
                ;;
            R)
                local redirect_file="spiral/pass-$pass/human-redirect.md"
                mkdir -p "spiral/pass-$pass"
                # Seed with template
                cat > "$redirect_file" <<REDIRECT_EOF
# Human Redirect — Pass $pass

<!-- Write your guidance for the next pass below. Save and close when done. -->
<!-- Delete this comment block. -->

REDIRECT_EOF
                # Open in editor
                local editor="${EDITOR:-${VISUAL:-vi}}"
                "$editor" "$redirect_file"
                if [[ -s "$redirect_file" ]]; then
                    log_info "REDIRECT — guidance saved to $redirect_file"
                else
                    log_warn "Redirect file is empty. Treating as CONTINUE."
                fi
                return 0
                ;;
            *)
                echo "  Please enter A, C, or R."
                ;;
        esac
    done
}

block_gate() {
    local pass="$1"
    local plan_file="methodology/plan.md"

    if [[ "$NO_PAUSE" == "true" || "$NO_PAUSE" == "1" ]]; then
        log_warn "Block gate skipped (NO_PAUSE=$NO_PAUSE) — defaulting to SKIP"
        return 1
    fi

    # Gather task counts
    local total_tasks done_tasks blocked_tasks
    total_tasks=$(grep -c '^### Task' "$plan_file" 2>/dev/null || echo 0)
    done_tasks=$(grep -c '\*\*Status:\*\* DONE' "$plan_file" 2>/dev/null || echo 0)
    blocked_tasks=$(grep -c '\*\*Status:\*\* BLOCKED' "$plan_file" 2>/dev/null || echo 0)

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${RED}${BOLD}  BUILD BLOCKED${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  Completed: ${GREEN}$done_tasks${NC} / $total_tasks tasks"
    echo -e "  Blocked:   ${RED}$blocked_tasks${NC} tasks"
    echo ""

    # Show blocked task names and reasons
    if [[ -f "$plan_file" ]]; then
        echo -e "  ${BOLD}Blocked tasks:${NC}"
        awk '
            /^### Task/ { task=$0; sub(/^### /, "", task); in_task=1; is_blocked=0; reason="" }
            in_task && /\*\*Status:\*\* BLOCKED/ { is_blocked=1 }
            in_task && /\*\*BLOCKED:\*\*/ { reason=$0; sub(/.*\*\*BLOCKED:\*\*[[:space:]]*/, "", reason) }
            /^### Task/ && prev_blocked { printf "    • %s\n", prev_task; if (prev_reason != "") printf "      Reason: %s\n", prev_reason }
            { prev_task=task; prev_blocked=is_blocked; prev_reason=reason }
            END { if (is_blocked) { printf "    • %s\n", task; if (reason != "") printf "      Reason: %s\n", reason } }
        ' "$plan_file"
        echo ""
    fi

    echo -e "  ${GREEN}[F]${NC} FIX — resolve blocks, then resume build"
    echo -e "  ${YELLOW}[S]${NC} SKIP — continue to next phase"
    echo -e "  ${RED}[X]${NC} ABORT — stop this spiral pass"
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    while true; do
        read -rp "  Your choice [F/S/X]: " choice
        case "${choice^^}" in
            F)
                log_info "FIX — resolve blocks in methodology/plan.md, then build resumes."
                return 0
                ;;
            S)
                log_info "SKIP — continuing to next phase."
                return 1
                ;;
            X)
                log_error "ABORT — stopping spiral pass."
                return 2
                ;;
            *)
                echo "  Please enter F, S, or X."
                ;;
        esac
    done
}

environment_gate() {
    # Check if the scope agent flagged missing runtimes/toolchains.
    # Only fires for system-level tooling — package-level deps are handled by scope.
    local env_file="spiral/pass-0/environment-resolution.md"

    # Skip if the file doesn't exist or is empty
    if [[ ! -s "$env_file" ]]; then
        return 0
    fi

    if [[ "$NO_PAUSE" == "true" || "$NO_PAUSE" == "1" ]]; then
        log_warn "Environment gate skipped (NO_PAUSE=$NO_PAUSE) — proceeding with possible missing tooling"
        return 0
    fi

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  ENVIRONMENT RESOLUTION REQUIRED${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""
    echo "  The scope agent detected missing runtimes or toolchains."
    echo "  Details: $env_file"
    echo ""
    echo -e "${YELLOW}$(cat "$env_file")${NC}"
    echo ""
    echo "  [F] FIX — I'll install the missing runtimes/tooling. Press Enter when ready."
    echo "  [S] SKIP — Proceed anyway. I accept the risk of build failures."
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    while true; do
        read -rp "  Your choice [F/S]: " choice
        case "${choice^^}" in
            F)
                log_info "FIX — install the missing tooling, then press Enter."
                read -rp "  Press ENTER when you've installed the missing tooling... "
                echo ""

                # Re-verify: run version checks for items listed in the env file
                log_info "Re-verifying environment..."

                local recheck_context="Re-verify the local environment. Read spiral/pass-0/environment-resolution.md to see what was missing. Run the version/availability checks again for those specific tools. If everything is now available, clear the file (write an empty file to spiral/pass-0/environment-resolution.md). If tools are still missing, update the file with what remains missing."
                run_agent "prompts/PROMPT_scope.md" "$CLAUDE_MODEL_SCOPE" "$recheck_context" "Re-verify: environment"

                if [[ -s "$env_file" ]]; then
                    log_warn "Some tooling is still missing. Returning to environment gate."
                    continue
                else
                    log_success "Environment verified — all required tooling is now available."
                    return 0
                fi
                ;;
            S)
                log_warn "SKIP — proceeding with possible missing tooling."
                return 0
                ;;
            *)
                echo "  Please enter F or S."
                ;;
        esac
    done
}

scope_review_gate() {
    if [[ "$NO_PAUSE" == "true" || "$NO_PAUSE" == "1" ]]; then
        log_warn "Scope review skipped (NO_PAUSE=$NO_PAUSE)"
        return 0
    fi

    while true; do
        echo ""
        echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
        echo -e "${BOLD}  PASS 0 (SCOPING) COMPLETE — REVIEW REQUIRED${NC}"
        echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
        echo ""
        echo -e "  Methodology:       ${CYAN}methodology/methodology.md${NC}"
        echo -e "  Plan:              ${CYAN}methodology/plan.md${NC}"
        echo -e "  Acceptance:        ${CYAN}spiral/pass-0/acceptance-criteria.md${NC}"
        echo -e "  Scope progression: ${CYAN}spiral/pass-0/spiral-plan.md${NC}"
        echo -e "  Validation:        ${CYAN}spiral/pass-0/validation-strategy.md${NC}"

        # Show technology stack if AGENTS.md has been updated
        if grep -q 'Technology Stack\|Language & Runtime' AGENTS.md 2>/dev/null; then
            local stack
            stack=$(awk '/^### Language & Runtime/{found=1; next} found && /\S/ && !/^#/{print; found=0}' AGENTS.md)
            if [[ -n "$stack" && "$stack" != *"To be resolved"* ]]; then
                echo ""
                echo -e "  ${CYAN}Stack:${NC} $stack"
            fi
        fi

        # Show scope progression summary
        if [[ -f "spiral/pass-0/spiral-plan.md" ]]; then
            echo ""
            echo -e "  ${CYAN}Scope progression:${NC}"
            grep -E '^\|.*Pass [0-9]|^\| [0-9]' spiral/pass-0/spiral-plan.md 2>/dev/null | head -5 | while read -r line; do
                echo -e "    $line"
            done
        fi

        # Show methodology structure (section headings)
        if [[ -f "methodology/methodology.md" ]]; then
            echo ""
            echo -e "  ${CYAN}Methodology sections:${NC}"
            grep -E '^## ' methodology/methodology.md 2>/dev/null | grep -v '^## Phenomenon' | head -8 | while read -r line; do
                echo -e "    ${line}"
            done
        fi

        echo ""
        echo -e "  ${GREEN}[A]${NC} APPROVE  — proceed to Pass 1"
        echo -e "  ${YELLOW}[R]${NC} REFINE   — provide feedback, re-run scope agent"
        echo -e "  ${CYAN}[E]${NC} EDIT     — I'll edit the files directly, then approve"
        echo -e "  ${RED}[Q]${NC} QUIT     — stop here"
        echo ""
        echo -n "  Choice: "
        read -r choice

        case "${choice,,}" in
            a)
                log_success "Scope approved. Proceeding to Pass 1."
                return 0
                ;;
            r)
                # Open editor with feedback template
                local feedback_file="spiral/pass-0/scope-feedback.md"
                if [[ ! -f "$feedback_file" ]]; then
                    cat > "$feedback_file" << 'FEEDBACK_TEMPLATE'
# Scope Feedback

## Acceptance Criteria Issues
-

## Methodology Issues
-

## Scope Progression Issues
-

## Validation Issues
-

## Other
-
FEEDBACK_TEMPLATE
                fi
                ${EDITOR:-vim} "$feedback_file"

                # Check if feedback is non-empty (beyond template)
                local content
                content=$(grep -v '^#' "$feedback_file" | grep -v '^-[[:space:]]*$' | grep -v '^$' | wc -l)
                if [[ "$content" -eq 0 ]]; then
                    log_warn "Feedback file is empty. Returning to review."
                    continue
                fi

                log_info "Re-running scope agent with feedback..."
                local context="SCOPE REFINEMENT: The human has reviewed your scope artifacts and provided feedback."
                context+=$'\n'"Read spiral/pass-0/scope-feedback.md carefully and update all affected artifacts."
                context+=$'\n'"Do not discard previous work — refine it based on the feedback."

                run_agent "prompts/PROMPT_scope.md" "$CLAUDE_MODEL_SCOPE" "$context" "Scope: refinement"
                git_commit_all "scope: refined after human feedback" || true

                # Loop back to review
                log_info "Scope refined. Reviewing again..."
                ;;
            e)
                log_info "Edit scope files directly, then press Enter to approve."
                echo -n "  Press Enter when done editing..."
                read -r
                log_success "Scope approved (manually edited). Proceeding to Pass 1."
                return 0
                ;;
            q)
                log_warn "Stopping after scope."
                return 1
                ;;
            *)
                echo "  Invalid choice. Enter A, R, E, or Q."
                ;;
        esac
    done
}

# --- Phase: Scope (Pass 0) ---------------------------------------------------

run_scope() {
    log_phase "PASS 0 — SCOPING"

    # Check if already complete
    if [[ -f "spiral/pass-0/PASS_COMPLETE.md" ]]; then
        log_success "Pass 0 already complete."
        return 0
    fi

    write_state 0 "scope" "in_progress"
    mkdir -p spiral/pass-0

    # Detect if this is a resume with existing feedback (scope refinement was interrupted)
    local scope_context=""
    if [[ -f "spiral/pass-0/scope-feedback.md" ]]; then
        local content
        content=$(grep -v '^#' "spiral/pass-0/scope-feedback.md" | grep -v '^-[[:space:]]*$' | grep -v '^$' | wc -l)
        if [[ "$content" -gt 0 ]]; then
            scope_context="SCOPE REFINEMENT: The human has reviewed your scope artifacts and provided feedback."
            scope_context+=$'\n'"Read spiral/pass-0/scope-feedback.md carefully and update all affected artifacts."
            scope_context+=$'\n'"Do not discard previous work — refine it based on the feedback."
            log_info "Detected existing scope feedback — running as refinement."
        fi
    fi

    run_agent "prompts/PROMPT_scope.md" "$CLAUDE_MODEL_SCOPE" "$scope_context" "Scope"

    log_info "Committing scope artifacts..."
    git_commit_all "scope: pass 0 — scoping complete" || true

    environment_gate

    scope_review_gate
    local gate_result=$?
    if [[ $gate_result -ne 0 ]]; then
        log_warn "Scope not approved. Stopping."
        return 1
    fi

    write_state 0 "scope" "complete"
    log_success "Pass 0 (scoping) complete."
}

ensure_scope_complete() {
    if [[ ! -f "spiral/pass-0/PASS_COMPLETE.md" ]]; then
        log_info "Pass 0 (scoping) not complete. Running scope first."
        run_scope
    else
        log_info "Pass 0 already complete."
    fi
}

# --- Phase: Refine ------------------------------------------------------------

run_refine() {
    local pass="$1"
    log_phase "PASS $pass — REFINE"
    write_state "$pass" "refine" "in_progress"

    mkdir -p "spiral/pass-$pass"

    local prev_pass=$((pass - 1))
    local context="Current spiral pass: $pass"
    context+=$'\n'"Previous pass results: spiral/pass-$prev_pass/"
    if [[ -f "spiral/pass-$prev_pass/human-redirect.md" ]]; then
        context+=$'\n'"Human redirect file: spiral/pass-$prev_pass/human-redirect.md"
    fi

    run_agent "prompts/PROMPT_refine.md" "$CLAUDE_MODEL_REFINE" "$context" "Refine: pass $pass"
    git_commit_all "refine: pass $pass" || true

    write_state "$pass" "refine" "complete"
}

# --- Phase: DDV Red -----------------------------------------------------------

run_ddv_red() {
    local pass="$1"
    log_phase "PASS $pass — DDV RED (domain verification tests)"
    write_state "$pass" "ddv_red" "in_progress"
    mkdir -p "spiral/pass-$pass"

    local context="Current spiral pass: $pass"
    run_agent "prompts/PROMPT_ddv_red.md" "$CLAUDE_MODEL_DDV" "$context" "DDV Red: pass $pass"
    git_commit_all "ddv-red: pass $pass — domain verification tests written" || true

    write_state "$pass" "ddv_red" "complete"
}

# --- Phase: Build (Ralph Loop) -----------------------------------------------

run_build() {
    local pass="$1"
    local start_iter="${2:-1}"
    log_phase "PASS $pass — BUILD (Ralph loop)"
    write_state "$pass" "build" "in_progress" 0

    local context="Current spiral pass: $pass"

    local prev_remaining stall_count
    prev_remaining=$(_count_uncompleted_tasks "$pass")
    stall_count=0

    local build_complete=false
    for ((iter = start_iter; iter <= MAX_RALPH_ITERATIONS; iter++)); do
        echo ""
        log_phase "Build iteration $iter / $MAX_RALPH_ITERATIONS"

        # Display progress
        local total_tasks done_tasks todo_tasks blocked_tasks
        total_tasks=$(grep -c '^### Task' "methodology/plan.md" 2>/dev/null || echo 0)
        done_tasks=$(grep -c '\*\*Status:\*\* DONE' "methodology/plan.md" 2>/dev/null || echo 0)
        blocked_tasks=$(grep -c '\*\*Status:\*\* BLOCKED' "methodology/plan.md" 2>/dev/null || echo 0)
        todo_tasks=$(( total_tasks - done_tasks - blocked_tasks ))
        echo -e "  ${CYAN}Progress:${NC} ${GREEN}${done_tasks} done${NC} / ${YELLOW}${todo_tasks} remaining${NC} / ${RED}${blocked_tasks} blocked${NC} (of ${total_tasks} total)"

        write_state "$pass" "build" "in_progress" "$iter"

        run_agent "prompts/PROMPT_build.md" "$CLAUDE_MODEL_BUILD" "$context" "Build: iter $iter"
        git_commit_all "build: pass $pass iteration $iter" || true

        # Check completion
        if _all_tasks_done "$pass"; then
            if _has_blocked_tasks "$pass"; then
                log_warn "All non-blocked tasks complete. Some tasks are BLOCKED."
                block_gate "$pass"
                local gate_result=$?
                if [[ $gate_result -eq 0 ]]; then
                    stall_count=0
                    continue
                elif [[ $gate_result -eq 2 ]]; then
                    return 1
                fi
                # Skip — fall through to exit build loop
            fi
            log_success "All tasks for pass $pass complete."
            build_complete=true
            break
        fi

        # Stall detection
        local cur_remaining
        cur_remaining=$(_count_uncompleted_tasks "$pass")

        if [[ "$cur_remaining" -eq "$prev_remaining" ]]; then
            stall_count=$((stall_count + 1))
            log_warn "No task progress (stall count: $stall_count/2, remaining: $cur_remaining)."
        else
            stall_count=0
            prev_remaining=$cur_remaining
        fi

        if [[ $stall_count -ge 2 ]]; then
            log_warn "Build stalled — no progress for 2 consecutive iterations."
            if _has_blocked_tasks "$pass"; then
                block_gate "$pass"
                local gate_result=$?
                if [[ $gate_result -eq 0 ]]; then
                    stall_count=0
                    continue
                elif [[ $gate_result -eq 2 ]]; then
                    return 1
                fi
                # Skip — fall through to exit build loop
            else
                log_warn "No blocked tasks found — nothing left to do."
            fi
            break
        fi

        log_info "Tasks remain — continuing Ralph loop."
    done

    if [[ "$build_complete" != "true" ]] && [[ $iter -gt $MAX_RALPH_ITERATIONS ]]; then
        log_warn "Reached max Ralph iterations ($MAX_RALPH_ITERATIONS). Some tasks may remain."
    fi

    write_state "$pass" "build" "complete"
    return 0
}

# --- Phase: Execute -----------------------------------------------------------

run_execute() {
    local pass="$1"
    log_phase "PASS $pass — EXECUTE"
    write_state "$pass" "execute" "in_progress"
    mkdir -p "spiral/pass-$pass"

    local context="Current spiral pass: $pass"
    run_agent "prompts/PROMPT_execute.md" "$CLAUDE_MODEL_EXECUTE" "$context" "Execute: pass $pass"
    git_commit_all "execute: pass $pass" || true

    write_state "$pass" "execute" "complete"
}

# --- Phase: Validate ----------------------------------------------------------

run_validate() {
    local pass="$1"
    log_phase "PASS $pass — VALIDATE"
    write_state "$pass" "validate" "in_progress"
    mkdir -p "spiral/pass-$pass"

    local context="Current spiral pass: $pass"
    run_agent "prompts/PROMPT_validate.md" "$CLAUDE_MODEL_VALIDATE" "$context" "Validate: pass $pass"
    git_commit_all "validate: pass $pass" || true

    write_state "$pass" "validate" "complete"
}

# --- Finalize -----------------------------------------------------------------

finalize_output() {
    local pass="$1"
    log_phase "FINALIZING — Producing deliverables"

    # If output files weren't drafted during validation, run a finalization agent call
    if [[ ! -f "output/answer.md" ]] || [[ ! -f "output/report.md" ]]; then
        log_info "Output files not yet drafted. Running finalization agent..."
        mkdir -p output
        local context="Current spiral pass: $pass"
        context+=$'\n'"FINALIZATION MODE: The human has ACCEPTED the results."
        context+=$'\n'"You MUST produce output/answer.md and output/report.md now."
        context+=$'\n'"Read the review package at spiral/pass-$pass/review-package.md for the current answer."
        context+=$'\n'"Read all spiral/pass-*/convergence.md files for the convergence history."
        context+=$'\n'"Read methodology/methodology.md for the methodology."
        context+=$'\n'"Follow the report format specified in the PROMPT_validate.md instructions."
        run_agent "prompts/PROMPT_validate.md" "$CLAUDE_MODEL_VALIDATE" "$context" "Finalize: output"
        git_commit_all "final: generate output deliverables" || true
    fi

    # Create SPIRAL_COMPLETE.md
    cat > "spiral/SPIRAL_COMPLETE.md" <<EOF
# Spiral Complete

The spiral has converged and the human has accepted the results.

Completed: $(date -Iseconds)
Final pass: $pass
EOF

    git_commit_all "final: spiral complete — answer accepted at pass $pass" || true
    git_push

    log_success "Done. Final deliverables in output/."
    if [[ -f "output/answer.md" ]]; then
        echo ""
        log_info "--- output/answer.md ---"
        cat output/answer.md
        echo ""
        log_info "--- end ---"
    fi
}

# --- Commands -----------------------------------------------------------------

cmd_scope() {
    run_scope
}

cmd_run() {
    local max_passes="$MAX_SPIRAL_PASSES"

    # Parse --max-passes flag
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --max-passes)
                max_passes="$2"
                shift 2
                ;;
            --max-passes=*)
                max_passes="${1#*=}"
                shift
                ;;
            *)
                log_error "Unknown flag: $1"
                exit 1
                ;;
        esac
    done

    log_phase "LISA LOOP v2 — SPIRAL RUN (max $max_passes passes)"

    # Ensure scoping is done first
    ensure_scope_complete

    # Spiral passes
    for ((pass = 1; pass <= max_passes; pass++)); do
        echo ""
        log_phase "═══ SPIRAL PASS $pass / $max_passes ═══"

        # Skip completed passes (for resume support)
        if [[ -f "spiral/pass-$pass/PASS_COMPLETE.md" ]]; then
            log_info "Pass $pass already complete — skipping."
            continue
        fi

        # Phase 1: Refine
        run_refine "$pass"

        # Phase 2: DDV Red
        run_ddv_red "$pass"

        # Phase 3: Build (Ralph loop)
        if ! run_build "$pass"; then
            log_error "Build aborted at pass $pass."
            return 1
        fi

        # Phase 4: Execute
        run_execute "$pass"

        # Phase 5: Validate
        run_validate "$pass"
        git_push

        # Phase 6: Human review
        write_state "$pass" "review" "in_progress"
        review_gate "$pass"
        local gate_result=$?
        if [[ $gate_result -eq 1 ]]; then
            # Accepted
            finalize_output "$pass"
            return 0
        fi
        # Otherwise: continue or redirect (redirect saved during gate)
    done

    log_warn "Reached max spiral passes ($max_passes) without convergence."
    log_info "Review the latest pass results and decide whether to accept or continue."
}

cmd_resume() {
    log_phase "RESUMING FROM SAVED STATE"

    read_state

    log_info "Current state: pass=$STATE_PASS phase=$STATE_PHASE status=$STATE_STATUS ralph_iter=$STATE_RALPH_ITER"

    if [[ "$STATE_PHASE" == "not_started" ]]; then
        log_info "No previous run found. Starting fresh."
        cmd_run "$@"
        return
    fi

    if [[ "$STATE_PASS" -eq 0 ]]; then
        if [[ "$STATE_STATUS" != "complete" ]]; then
            log_info "Resuming: Pass 0 (scope) was incomplete."
            run_scope
        fi
        # Continue with full run
        cmd_run "$@"
        return
    fi

    local pass="$STATE_PASS"

    # Helper: run remaining passes after completing the current one
    run_remaining_passes() {
        local start_pass="$1"
        for ((p = start_pass; p <= MAX_SPIRAL_PASSES; p++)); do
            echo ""
            log_phase "═══ SPIRAL PASS $p / $MAX_SPIRAL_PASSES ═══"

            if [[ -f "spiral/pass-$p/PASS_COMPLETE.md" ]]; then
                log_info "Pass $p already complete — skipping."
                continue
            fi

            run_refine "$p"
            run_ddv_red "$p"
            if ! run_build "$p"; then
                log_error "Build aborted at pass $p."
                return 1
            fi
            run_execute "$p"
            run_validate "$p"
            git_push

            write_state "$p" "review" "in_progress"
            review_gate "$p"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$p"
                return 0
            fi
        done
        log_warn "Reached max spiral passes ($MAX_SPIRAL_PASSES) without convergence."
    }

    case "$STATE_PHASE" in
        refine)
            if [[ "$STATE_STATUS" != "complete" ]]; then
                log_info "Resuming: refine phase at pass $pass."
                run_refine "$pass"
            fi
            run_ddv_red "$pass"
            if ! run_build "$pass"; then
                log_error "Build aborted."
                return 1
            fi
            run_execute "$pass"
            run_validate "$pass"
            git_push
            write_state "$pass" "review" "in_progress"
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            run_remaining_passes "$((pass + 1))"
            ;;
        ddv_red)
            if [[ "$STATE_STATUS" != "complete" ]]; then
                log_info "Resuming: DDV Red phase at pass $pass."
                run_ddv_red "$pass"
            fi
            if ! run_build "$pass"; then
                log_error "Build aborted."
                return 1
            fi
            run_execute "$pass"
            run_validate "$pass"
            git_push
            write_state "$pass" "review" "in_progress"
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            run_remaining_passes "$((pass + 1))"
            ;;
        build)
            log_info "Resuming: build phase at pass $pass (iteration $STATE_RALPH_ITER)."
            if ! run_build "$pass" "$STATE_RALPH_ITER"; then
                log_error "Build aborted."
                return 1
            fi
            run_execute "$pass"
            run_validate "$pass"
            git_push
            write_state "$pass" "review" "in_progress"
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            run_remaining_passes "$((pass + 1))"
            ;;
        execute)
            if [[ "$STATE_STATUS" != "complete" ]]; then
                log_info "Resuming: execute phase at pass $pass."
                run_execute "$pass"
            fi
            run_validate "$pass"
            git_push
            write_state "$pass" "review" "in_progress"
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            run_remaining_passes "$((pass + 1))"
            ;;
        validate)
            if [[ "$STATE_STATUS" != "complete" ]]; then
                log_info "Resuming: validate phase at pass $pass."
                run_validate "$pass"
                git_push
            fi
            write_state "$pass" "review" "in_progress"
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            run_remaining_passes "$((pass + 1))"
            ;;
        review)
            log_info "Resuming: review gate of pass $pass."
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            run_remaining_passes "$((pass + 1))"
            ;;
        *)
            log_warn "Unknown phase: $STATE_PHASE. Starting full run."
            cmd_run "$@"
            ;;
    esac
}

cmd_status() {
    read_state

    echo ""
    echo -e "${BOLD}Lisa Loop v2 — Current Status${NC}"
    echo ""

    if [[ "$STATE_PHASE" == "not_started" ]]; then
        echo "  State: Not started"
        echo "  Next:  ./loop.sh scope   (or ./loop.sh run)"
    else
        echo "  Spiral pass:     $STATE_PASS"
        echo "  Phase:           $STATE_PHASE"
        echo "  Status:          $STATE_STATUS"
        if [[ "$STATE_PHASE" == "build" ]]; then
            echo "  Ralph iteration: $STATE_RALPH_ITER"
        fi

        if [[ -f "spiral/SPIRAL_COMPLETE.md" ]]; then
            echo ""
            echo -e "  ${GREEN}Spiral COMPLETE — answer accepted.${NC}"
        fi

        echo ""
        echo "  Pass artifacts:"
        for d in spiral/pass-*/; do
            [[ -d "$d" ]] || continue
            local pnum="${d#spiral/pass-}"
            pnum="${pnum%/}"
            local status_marker=""
            if [[ -f "${d}PASS_COMPLETE.md" ]]; then
                status_marker=" ✓"
            fi
            echo "    pass-$pnum$status_marker"
        done

        # Show task status from methodology/plan.md
        if [[ -f "methodology/plan.md" ]]; then
            local todo done blocked inprog
            todo=$(grep -c '\*\*Status:\*\* TODO' "methodology/plan.md" 2>/dev/null || echo 0)
            done=$(grep -c '\*\*Status:\*\* DONE' "methodology/plan.md" 2>/dev/null || echo 0)
            blocked=$(grep -c '\*\*Status:\*\* BLOCKED' "methodology/plan.md" 2>/dev/null || echo 0)
            inprog=$(grep -c '\*\*Status:\*\* IN_PROGRESS' "methodology/plan.md" 2>/dev/null || echo 0)
            echo ""
            echo "  Task status: TODO=$todo IN_PROGRESS=$inprog DONE=$done BLOCKED=$blocked"
        fi
    fi
    echo ""
}

# --- Main ---------------------------------------------------------------------

usage() {
    echo "Lisa Loop v2 — Spiral development loop"
    echo ""
    echo "Usage: ./loop.sh <command> [options]"
    echo ""
    echo "Commands:"
    echo "  scope                  Run Pass 0 (scoping) only"
    echo "  run [--max-passes N]   Full spiral (scope if needed, then iterate)"
    echo "  resume                 Resume from current state"
    echo "  status                 Print current state and exit"
    echo ""
    echo "Configuration: lisa.conf (see file for all options)"
}

if [[ $# -lt 1 ]]; then
    usage
    exit 1
fi

COMMAND="$1"
shift

case "$COMMAND" in
    scope)
        cmd_scope
        ;;
    run)
        cmd_run "$@"
        ;;
    resume)
        cmd_resume "$@"
        ;;
    status)
        cmd_status
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        log_error "Unknown command: $COMMAND"
        usage
        exit 1
        ;;
esac
