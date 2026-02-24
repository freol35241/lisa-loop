#!/usr/bin/env bash
set -euo pipefail

# Lisa Loop v2 — Spiral-V development loop for engineering and scientific software
#
# Architecture: Outer spiral (convergence-driven, human-gated) with inner
# Ralph loop (autonomous task execution) per subsystem at each pass.
#
# Each spiral pass iterates over subsystems in dependency order:
#   For each subsystem: refine methodology → build (Ralph loop) → next subsystem
#   Then: system-level validation + convergence check → human review gate
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
CLAUDE_MODEL_BUILD="${CLAUDE_MODEL_BUILD:-sonnet}"
CLAUDE_MODEL_VALIDATE="${CLAUDE_MODEL_VALIDATE:-opus}"

# Loop limits
MAX_SPIRAL_PASSES="${MAX_SPIRAL_PASSES:-5}"
MAX_RALPH_ITERATIONS="${MAX_RALPH_ITERATIONS:-50}"

# Human review
NO_PAUSE="${NO_PAUSE:-false}"

# Git
NO_PUSH="${NO_PUSH:-false}"

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

_filter_agent_stream() {
    # Process NDJSON stream from `claude -p --output-format stream-json --verbose`
    # and emit human-readable progress lines showing agent tool calls, thinking,
    # and the final result text.
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
            echo -e "${DIM}  [thinking $(_ts)] ${thought}${NC}"
        elif [[ "$line" == TOOL:\ * ]]; then
            echo -e "${MAGENTA}  [agent $(_ts)]${NC} ${line#TOOL: }"
        elif [[ "$line" == RESULT_B64:\ * ]]; then
            local result_b64="${line#RESULT_B64: }"
            local result_text
            result_text="$(echo "$result_b64" | base64 -d 2>/dev/null)" || result_text=""
            if [[ -n "$result_text" ]]; then
                echo ""
                echo -e "${MAGENTA}  [agent $(_ts)]${NC} ── Result ──"
                echo "$result_text"
                echo ""
            fi
        fi
    done
    return 0
}

run_agent() {
    # Usage: run_agent <prompt_file> <model> [context_string]
    local prompt_file="$1"
    local model="$2"
    local context="${3:-}"

    if [[ ! -f "$prompt_file" ]]; then
        log_error "Prompt file not found: $prompt_file"
        exit 1
    fi
    log_info "Calling agent with prompt: $prompt_file (model: $model)"
    local start_seconds=$SECONDS

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
    log_info "Agent finished (${elapsed}s elapsed)"
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

# --- Subsystem Parsing -------------------------------------------------------

parse_subsystems() {
    # Read the ordered subsystem list from SUBSYSTEMS.md
    # Look for the numbered list under "## Iteration Order"
    # Return subsystem names, one per line
    # Uses POSIX awk only (no gawk extensions)
    if [[ ! -f "SUBSYSTEMS.md" ]]; then
        log_error "SUBSYSTEMS.md not found. Run scope first."
        return 1
    fi
    awk '
        /^## Iteration Order/ { in_section=1; next }
        /^## / && in_section { exit }
        in_section && /^[0-9]+\./ {
            line = $0
            sub(/^[0-9]+\.[[:space:]]*/, "", line)
            # Strip markdown formatting like [] or backticks
            gsub(/[\[\]`]/, "", line)
            # Trim whitespace
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", line)
            if (line != "") print line
        }
    ' SUBSYSTEMS.md
}

# --- State Management --------------------------------------------------------

STATE_FILE="spiral/current-state.md"

write_state() {
    local pass="$1"
    local phase="$2"
    local status="$3"
    local subsystem="${4:-}"
    local ralph_iter="${5:-0}"
    mkdir -p spiral
    cat > "$STATE_FILE" <<EOF
# Spiral State
pass: $pass
phase: $phase
status: $status
subsystem: $subsystem
ralph_iteration: $ralph_iter
EOF
}

read_state() {
    # Sets global variables: STATE_PASS, STATE_PHASE, STATE_STATUS, STATE_SUBSYSTEM, STATE_RALPH_ITER
    if [[ ! -f "$STATE_FILE" ]]; then
        STATE_PASS=0
        STATE_PHASE="not_started"
        STATE_STATUS="pending"
        STATE_SUBSYSTEM=""
        STATE_RALPH_ITER=0
        return
    fi
    STATE_PASS=$(grep '^pass:' "$STATE_FILE" | awk '{print $2}')
    STATE_PHASE=$(grep '^phase:' "$STATE_FILE" | awk '{print $2}')
    STATE_STATUS=$(grep '^status:' "$STATE_FILE" | awk '{print $2}')
    STATE_SUBSYSTEM=$(grep '^subsystem:' "$STATE_FILE" | awk '{$1=""; sub(/^[[:space:]]+/, ""); print}')
    STATE_RALPH_ITER=$(grep '^ralph_iteration:' "$STATE_FILE" | awk '{print $2}')
    STATE_PASS="${STATE_PASS:-0}"
    STATE_PHASE="${STATE_PHASE:-not_started}"
    STATE_STATUS="${STATE_STATUS:-pending}"
    STATE_SUBSYSTEM="${STATE_SUBSYSTEM:-}"
    STATE_RALPH_ITER="${STATE_RALPH_ITER:-0}"
}

# --- Per-Subsystem Task Detection --------------------------------------------

count_tasks_for_subsystem_pass() {
    # Count tasks for a given subsystem, pass, and status
    # Usage: count_tasks_for_subsystem_pass <subsystem> <pass> <status>
    local subsystem="$1"
    local pass="$2"
    local status="$3"
    local plan_file="subsystems/$subsystem/plan.md"

    if [[ ! -f "$plan_file" ]]; then
        echo 0
        return
    fi

    # Use POSIX-compatible awk to parse task blocks.
    # A task block starts with "### Task" and ends at the next "### Task" or EOF.
    # Within a block, check for "**Pass:** N" or "**Spiral pass:** N" and "**Status:** STATUS".
    awk -v pass="$pass" -v status="$status" '
        /^### Task/ {
            if (in_task && found_pass && found_status) count++
            in_task=1; found_pass=0; found_status=0
            next
        }
        in_task && /\*\*(Spiral pass|Pass):\*\*/ {
            line = $0
            sub(/.*\*\*(Spiral pass|Pass):\*\*[[:space:]]*/, "", line)
            sub(/[^0-9].*/, "", line)
            if (line == pass) found_pass=1
        }
        in_task && /\*\*Status:\*\*/ {
            if (index($0, status)) found_status=1
        }
        END {
            if (in_task && found_pass && found_status) count++
            print count+0
        }
    ' "$plan_file"
}

count_uncompleted_tasks_up_to_pass() {
    # Count TODO or IN_PROGRESS tasks for a subsystem where spiral pass <= given pass
    local subsystem="$1"
    local max_pass="$2"
    local plan_file="subsystems/$subsystem/plan.md"

    if [[ ! -f "$plan_file" ]]; then
        echo 0
        return
    fi

    awk -v max_pass="$max_pass" '
        /^### Task/ {
            if (in_task && task_pass <= max_pass && (found_todo || found_inprog)) count++
            in_task=1; task_pass=9999; found_todo=0; found_inprog=0
            next
        }
        in_task && /\*\*(Spiral pass|Pass):\*\*/ {
            line = $0
            sub(/.*\*\*(Spiral pass|Pass):\*\*[[:space:]]*/, "", line)
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

count_blocked_tasks_up_to_pass() {
    # Count BLOCKED tasks for a subsystem where spiral pass <= given pass
    local subsystem="$1"
    local max_pass="$2"
    local plan_file="subsystems/$subsystem/plan.md"

    if [[ ! -f "$plan_file" ]]; then
        echo 0
        return
    fi

    awk -v max_pass="$max_pass" '
        /^### Task/ {
            if (in_task && task_pass <= max_pass && found_blocked) count++
            in_task=1; task_pass=9999; found_blocked=0
            next
        }
        in_task && /\*\*(Spiral pass|Pass):\*\*/ {
            line = $0
            sub(/.*\*\*(Spiral pass|Pass):\*\*[[:space:]]*/, "", line)
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

all_subsystem_pass_tasks_done() {
    # Returns 0 (true) if no TODO or IN_PROGRESS tasks remain for the given
    # subsystem at or below the given pass (catches leftover tasks from earlier passes)
    local subsystem="$1"
    local pass="$2"
    local remaining
    remaining=$(count_uncompleted_tasks_up_to_pass "$subsystem" "$pass")
    [[ "$remaining" -eq 0 ]]
}

has_subsystem_blocked_tasks() {
    # Returns 0 (true) if any tasks for the given subsystem at or below the
    # given pass are BLOCKED
    local subsystem="$1"
    local pass="$2"
    local blocked_count
    blocked_count=$(count_blocked_tasks_up_to_pass "$subsystem" "$pass")
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
        convergence=$(grep -i 'overall assessment\|CONVERGED\|NOT YET\|DIVERGING' "$review_file" | head -1 | sed 's/.*: *//')
        if [[ -n "$convergence" ]]; then
            local conv_color="$YELLOW"
            [[ "$convergence" == *CONVERGED* && "$convergence" != *NOT* ]] && conv_color="$GREEN"
            [[ "$convergence" == *DIVERGING* ]] && conv_color="$RED"
            echo -e "  ${BOLD}Convergence:${NC} ${conv_color}${convergence}${NC}"
        fi

        # Test summary
        local tests
        tests=$(grep -E '^L[0-3]:' "$review_file" | head -1)
        if [[ -n "$tests" ]]; then
            echo -e "  ${BOLD}Tests:${NC} $tests"
        fi

        # Sanity checks
        local sanity
        sanity=$(grep -i 'sanity checks' "$review_file" | head -1 | sed 's/.*: *//')
        if [[ -n "$sanity" ]]; then
            echo -e "  ${BOLD}Sanity:${NC} $sanity"
        fi

        # Failures (if any)
        local failures
        failures=$(grep -i 'failure\|FAIL' "$review_file" | grep -v '^#' | head -3)
        if [[ -n "$failures" ]]; then
            echo ""
            echo -e "  ${RED}Failures:${NC}"
            while IFS= read -r f; do
                echo -e "    ${RED}•${NC} $f"
            done <<< "$failures"
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
    echo "    Review:  $review_file"
    echo "    Plots:   plots/REVIEW.md"
    echo ""
    echo -e "  ${BOLD}[A]${NC} ACCEPT — converged, produce final report"
    echo -e "  ${BOLD}[C]${NC} CONTINUE — next spiral pass"
    echo -e "  ${BOLD}[R]${NC} REDIRECT — provide guidance (opens \$EDITOR)"
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
    local subsystem="$2"
    local completed="$3"
    local total="$4"
    local blocked="$5"

    if [[ "$NO_PAUSE" == "true" || "$NO_PAUSE" == "1" ]]; then
        log_warn "Block gate skipped (NO_PAUSE=$NO_PAUSE) — defaulting to SKIP"
        return 1
    fi

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${RED}${BOLD}  BUILD BLOCKED: $subsystem${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  Completed: ${GREEN}$completed${NC} / $total tasks"
    echo -e "  Blocked:   ${RED}$blocked${NC} tasks"
    echo ""

    # Show blocked task names and reasons
    local plan_file="subsystems/$subsystem/plan.md"
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

    echo -e "  ${BOLD}[F]${NC} FIX — resolve blocks, then resume build"
    echo -e "  ${BOLD}[S]${NC} SKIP — continue to next subsystem"
    echo -e "  ${BOLD}[X]${NC} ABORT — stop this spiral pass"
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    while true; do
        read -rp "  Your choice [F/S/X]: " choice
        case "${choice^^}" in
            F)
                log_info "FIX — resolve blocks in subsystems/$subsystem/plan.md, then build resumes."
                return 0
                ;;
            S)
                log_info "SKIP — continuing to next subsystem."
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
                local verify_failed=false

                # Extract tool check commands from the env file and re-run them
                # Simple approach: re-run common version checks and see if env file
                # should be cleared
                local recheck_context="Re-verify the local environment. Read spiral/pass-0/environment-resolution.md to see what was missing. Run the version/availability checks again for those specific tools. If everything is now available, clear the file (write an empty file to spiral/pass-0/environment-resolution.md). If tools are still missing, update the file with what remains missing."
                run_agent "prompts/PROMPT_scope.md" "$CLAUDE_MODEL_SCOPE" "$recheck_context"

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
        return
    fi

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  PASS 0 (SCOPING) COMPLETE — REVIEW REQUIRED${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    # Show discovered subsystems
    if [[ -f "SUBSYSTEMS.md" ]]; then
        echo -e "  ${BOLD}Subsystems discovered:${NC}"
        parse_subsystems | while IFS= read -r s; do
            echo -e "    ${CYAN}•${NC} $s"
        done
        echo ""
    fi

    # Show technology stack
    if grep -q 'Language & Runtime' AGENTS.md 2>/dev/null; then
        local stack
        stack=$(awk '/^### Language & Runtime/{found=1; next} found && /\S/ && !/^#/{print; found=0}' AGENTS.md)
        if [[ -n "$stack" ]]; then
            echo -e "  ${BOLD}Stack:${NC} $stack"
            echo ""
        fi
    fi

    echo -e "  ${CYAN}Review these artifacts:${NC}"
    echo "    SUBSYSTEMS.md                          subsystem decomposition"
    echo "    AGENTS.md                              resolved technology stack"
    echo "    subsystems/*/methodology.md            per-subsystem methods"
    echo "    subsystems/*/plan.md                   per-subsystem plans"
    echo "    spiral/pass-0/acceptance-criteria.md   success criteria"
    echo "    spiral/pass-0/spiral-plan.md           anticipated progression"
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""
    read -rp "  Press ENTER to approve, or Ctrl+C to stop and edit... "
    echo ""
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

    log_info "Running scope agent..."
    run_agent "prompts/PROMPT_scope.md" "$CLAUDE_MODEL_SCOPE"

    log_info "Committing scope artifacts..."
    git_commit_all "scope: pass 0 — scoping complete" || true

    environment_gate

    scope_review_gate

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

# --- Phase: Subsystem Refine -------------------------------------------------

run_subsystem_refine() {
    local pass="$1"
    local subsystem="$2"
    log_phase "PASS $pass — REFINE: $subsystem"

    write_state "$pass" "subsystem_refine" "in_progress" "$subsystem"
    mkdir -p "spiral/pass-$pass/subsystems/$subsystem"

    # Build context string for the agent
    local prev_pass=$((pass - 1))
    local context="Current spiral pass: $pass"
    context+=$'\n'"Subsystem: $subsystem"
    context+=$'\n'"Subsystem directory: subsystems/$subsystem/"
    context+=$'\n'"Previous pass results: spiral/pass-$prev_pass/"
    if [[ -f "spiral/pass-$prev_pass/human-redirect.md" ]]; then
        context+=$'\n'"Human redirect file: spiral/pass-$prev_pass/human-redirect.md"
    fi

    log_info "Running refine agent for subsystem: $subsystem..."
    run_agent "prompts/PROMPT_subsystem_refine.md" "$CLAUDE_MODEL_REFINE" "$context"

    write_state "$pass" "subsystem_refine" "complete" "$subsystem"
}

# --- Phase: Subsystem Build (Ralph Loop) -------------------------------------

run_subsystem_build() {
    local pass="$1"
    local subsystem="$2"
    local start_iter="${3:-1}"
    log_phase "PASS $pass — BUILD: $subsystem (Ralph loop)"

    write_state "$pass" "subsystem_build" "in_progress" "$subsystem" 0

    local context="Current spiral pass: $pass"
    context+=$'\n'"Subsystem: $subsystem"
    context+=$'\n'"Subsystem directory: subsystems/$subsystem/"

    # Stall detection: track remaining task count across iterations
    # Uses cross-pass counting to detect progress on leftover tasks from earlier passes
    local prev_remaining stall_count
    prev_remaining=$(count_uncompleted_tasks_up_to_pass "$subsystem" "$pass")
    stall_count=0

    local build_complete=false
    for ((iter = start_iter; iter <= MAX_RALPH_ITERATIONS; iter++)); do
        echo ""
        log_phase "Build — Pass $pass, $subsystem, Iteration $iter / $MAX_RALPH_ITERATIONS"

        # Show task progress
        local total_tasks done_tasks todo_tasks blocked_tasks
        total_tasks=$(grep -c '^### Task' "subsystems/$subsystem/plan.md" 2>/dev/null || echo 0)
        done_tasks=$(grep -c '\*\*Status:\*\* DONE' "subsystems/$subsystem/plan.md" 2>/dev/null || echo 0)
        blocked_tasks=$(grep -c '\*\*Status:\*\* BLOCKED' "subsystems/$subsystem/plan.md" 2>/dev/null || echo 0)
        todo_tasks=$(( total_tasks - done_tasks - blocked_tasks ))
        echo -e "  ${CYAN}Progress:${NC} ${GREEN}${done_tasks} done${NC} / ${YELLOW}${todo_tasks} remaining${NC} / ${RED}${blocked_tasks} blocked${NC} (of ${total_tasks} total)"

        write_state "$pass" "subsystem_build" "in_progress" "$subsystem" "$iter"

        log_info "Running build agent (subsystem: $subsystem, iteration $iter)..."
        run_agent "prompts/PROMPT_subsystem_build.md" "$CLAUDE_MODEL_BUILD" "$context"

        log_info "Committing build work..."
        git_commit_all "build: pass $pass $subsystem iteration $iter" || true

        # Check if all tasks for this subsystem/pass are done
        if all_subsystem_pass_tasks_done "$subsystem" "$pass"; then
            if has_subsystem_blocked_tasks "$subsystem" "$pass"; then
                log_warn "All non-blocked tasks complete for $subsystem. Some tasks are BLOCKED."
                local done_count blocked_count total_count
                done_count=$(count_tasks_for_subsystem_pass "$subsystem" "$pass" "DONE")
                blocked_count=$(count_tasks_for_subsystem_pass "$subsystem" "$pass" "BLOCKED")
                total_count=$((done_count + blocked_count))

                block_gate "$pass" "$subsystem" "$done_count" "$total_count" "$blocked_count"
                local gate_result=$?
                if [[ $gate_result -eq 0 ]]; then
                    # Fix — continue build loop (human resolved blocks)
                    stall_count=0
                    continue
                elif [[ $gate_result -eq 2 ]]; then
                    # Abort
                    log_error "Build aborted by user."
                    return 1
                fi
                # Skip — fall through to exit build loop for this subsystem
            fi
            log_success "All tasks for $subsystem pass $pass complete."
            build_complete=true
            break
        fi

        # Stall detection: check if remaining task count decreased
        local cur_remaining
        cur_remaining=$(count_uncompleted_tasks_up_to_pass "$subsystem" "$pass")

        if [[ "$cur_remaining" -eq "$prev_remaining" ]]; then
            stall_count=$((stall_count + 1))
            log_warn "No task progress for $subsystem (stall count: $stall_count/2, remaining: $cur_remaining)."
        else
            stall_count=0
            prev_remaining=$cur_remaining
        fi

        if [[ $stall_count -ge 2 ]]; then
            log_warn "Build stalled for $subsystem — no progress for 2 consecutive iterations."
            if has_subsystem_blocked_tasks "$subsystem" "$pass"; then
                local done_count blocked_count total_count
                done_count=$(count_tasks_for_subsystem_pass "$subsystem" "$pass" "DONE")
                blocked_count=$(count_tasks_for_subsystem_pass "$subsystem" "$pass" "BLOCKED")
                total_count=$((done_count + blocked_count + cur_remaining))

                block_gate "$pass" "$subsystem" "$done_count" "$total_count" "$blocked_count"
                local gate_result=$?
                if [[ $gate_result -eq 0 ]]; then
                    stall_count=0
                    continue
                elif [[ $gate_result -eq 2 ]]; then
                    log_error "Build aborted by user."
                    return 1
                fi
                # Skip — fall through to exit build loop
            else
                log_warn "No blocked tasks found for $subsystem — nothing left to do."
            fi
            break
        fi

        log_info "Tasks remain for $subsystem — continuing Ralph loop."
    done

    if [[ "$build_complete" != "true" ]] && [[ $iter -gt $MAX_RALPH_ITERATIONS ]]; then
        log_warn "Reached max Ralph iterations ($MAX_RALPH_ITERATIONS) for $subsystem. Some tasks may remain."
    fi

    write_state "$pass" "subsystem_build" "complete" "$subsystem"
    return 0
}

# --- Phase: System Validation -------------------------------------------------

run_system_validate() {
    local pass="$1"
    log_phase "PASS $pass — SYSTEM VALIDATION (V&V + convergence)"

    write_state "$pass" "system_validate" "in_progress"

    # Build context string
    local prev_pass=$((pass - 1))

    # Phase A: Run tests and collect results
    log_info "System validation phase A: running tests and collecting results..."
    local context_a="Current spiral pass: $pass"
    context_a+=$'\n'"Previous pass results: spiral/pass-$prev_pass/"
    context_a+=$'\n'"VALIDATION PHASE A: Run all L2 and L3 tests, execute sanity checks, limiting cases, and reference data comparisons. Collect raw results into spiral/pass-$pass/test-results.md. Do NOT produce the review package or convergence assessment yet — that happens in Phase B."

    run_agent "prompts/PROMPT_system_validate.md" "$CLAUDE_MODEL_VALIDATE" "$context_a"
    git_commit_all "validate: pass $pass phase A — test results collected" || true

    # Phase B: Audit, analyze, and produce reports
    log_info "System validation phase B: analysis, convergence, and reporting..."
    local context_b="Current spiral pass: $pass"
    context_b+=$'\n'"Previous pass results: spiral/pass-$prev_pass/"
    context_b+=$'\n'"VALIDATION PHASE B: Test results have already been collected in spiral/pass-$pass/test-results.md. Now perform the methodology compliance spot-check, derivation completeness check, assumptions register check, traceability check, convergence assessment, and produce all report artifacts (system-validation.md, convergence.md, review-package.md, PASS_COMPLETE.md). Update validation/convergence-log.md and plots/REVIEW.md."

    run_agent "prompts/PROMPT_system_validate.md" "$CLAUDE_MODEL_VALIDATE" "$context_b"

    write_state "$pass" "system_validate" "complete"
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
        context+=$'\n'"Read SUBSYSTEMS.md for the subsystem decomposition."
        context+=$'\n'"Follow the report format specified in the PROMPT_system_validate.md instructions."
        run_agent "prompts/PROMPT_system_validate.md" "$CLAUDE_MODEL_VALIDATE" "$context"
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

    # Parse subsystem list
    local subsystems=()
    while IFS= read -r s; do
        [[ -n "$s" ]] && subsystems+=("$s")
    done < <(parse_subsystems)

    if [[ ${#subsystems[@]} -eq 0 ]]; then
        log_error "No subsystems found in SUBSYSTEMS.md. Check the Iteration Order section."
        exit 1
    fi

    log_info "Subsystems (${#subsystems[@]}): ${subsystems[*]}"

    # Spiral passes
    for ((pass = 1; pass <= max_passes; pass++)); do
        echo ""
        log_phase "═══ SPIRAL PASS $pass / $max_passes ═══"

        # Skip completed passes (for resume support)
        if [[ -f "spiral/pass-$pass/PASS_COMPLETE.md" ]]; then
            log_info "Pass $pass already complete — skipping."
            continue
        fi

        # Phase 1: Iterate subsystems (refine + build each)
        for subsystem in "${subsystems[@]}"; do
            echo ""
            log_phase "── Subsystem: $subsystem ──"

            # 1a. Refine
            run_subsystem_refine "$pass" "$subsystem"
            git_commit_all "refine: pass $pass subsystem $subsystem" || true

            # 1b. Build (Ralph loop)
            if ! run_subsystem_build "$pass" "$subsystem"; then
                log_error "Build phase aborted for $subsystem at pass $pass."
                return 1
            fi
        done

        # Phase 2: System validation
        run_system_validate "$pass"
        git_commit_all "validate: pass $pass system validation" || true
        git_push

        # Phase 3: Human review gate
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

    log_info "Current state: pass=$STATE_PASS phase=$STATE_PHASE status=$STATE_STATUS subsystem=$STATE_SUBSYSTEM ralph_iter=$STATE_RALPH_ITER"

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

    # Parse subsystem list
    local subsystems=()
    while IFS= read -r s; do
        [[ -n "$s" ]] && subsystems+=("$s")
    done < <(parse_subsystems)

    if [[ ${#subsystems[@]} -eq 0 ]]; then
        log_error "No subsystems found in SUBSYSTEMS.md."
        exit 1
    fi

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

            for subsystem in "${subsystems[@]}"; do
                echo ""
                log_phase "── Subsystem: $subsystem ──"
                run_subsystem_refine "$p" "$subsystem"
                git_commit_all "refine: pass $p subsystem $subsystem" || true
                if ! run_subsystem_build "$p" "$subsystem"; then
                    log_error "Build phase aborted for $subsystem at pass $p."
                    return 1
                fi
            done

            run_system_validate "$p"
            git_commit_all "validate: pass $p system validation" || true
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
        subsystem_refine)
            # Find which subsystem we're at and resume from there
            if [[ "$STATE_STATUS" != "complete" ]]; then
                log_info "Resuming: refine phase for $STATE_SUBSYSTEM at pass $pass."
                run_subsystem_refine "$pass" "$STATE_SUBSYSTEM"
                git_commit_all "refine: pass $pass subsystem $STATE_SUBSYSTEM" || true
            fi
            # Build the current subsystem
            if ! run_subsystem_build "$pass" "$STATE_SUBSYSTEM"; then
                log_error "Build aborted."
                return 1
            fi
            # Continue with remaining subsystems
            local found=false
            for subsystem in "${subsystems[@]}"; do
                if [[ "$found" == "true" ]]; then
                    echo ""
                    log_phase "── Subsystem: $subsystem ──"
                    run_subsystem_refine "$pass" "$subsystem"
                    git_commit_all "refine: pass $pass subsystem $subsystem" || true
                    if ! run_subsystem_build "$pass" "$subsystem"; then
                        log_error "Build aborted."
                        return 1
                    fi
                fi
                if [[ "$subsystem" == "$STATE_SUBSYSTEM" ]]; then
                    found=true
                fi
            done
            # System validation + review + remaining passes
            run_system_validate "$pass"
            git_commit_all "validate: pass $pass system validation" || true
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
        subsystem_build)
            log_info "Resuming: build phase for $STATE_SUBSYSTEM at pass $pass (iteration $STATE_RALPH_ITER)."
            if ! run_subsystem_build "$pass" "$STATE_SUBSYSTEM" "$STATE_RALPH_ITER"; then
                log_error "Build aborted."
                return 1
            fi
            # Continue with remaining subsystems
            local found=false
            for subsystem in "${subsystems[@]}"; do
                if [[ "$found" == "true" ]]; then
                    echo ""
                    log_phase "── Subsystem: $subsystem ──"
                    run_subsystem_refine "$pass" "$subsystem"
                    git_commit_all "refine: pass $pass subsystem $subsystem" || true
                    if ! run_subsystem_build "$pass" "$subsystem"; then
                        log_error "Build aborted."
                        return 1
                    fi
                fi
                if [[ "$subsystem" == "$STATE_SUBSYSTEM" ]]; then
                    found=true
                fi
            done
            # System validation + review + remaining passes
            run_system_validate "$pass"
            git_commit_all "validate: pass $pass system validation" || true
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
        system_validate)
            if [[ "$STATE_STATUS" != "complete" ]]; then
                log_info "Resuming: system validation phase of pass $pass."
                run_system_validate "$pass"
                git_commit_all "validate: pass $pass system validation" || true
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
        if [[ -n "$STATE_SUBSYSTEM" ]]; then
            echo "  Subsystem:       $STATE_SUBSYSTEM"
        fi
        if [[ "$STATE_PHASE" == "subsystem_build" ]]; then
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

        # Show per-subsystem task status if subsystems exist
        if [[ -f "SUBSYSTEMS.md" ]]; then
            local subs=()
            while IFS= read -r s; do
                [[ -n "$s" ]] && subs+=("$s")
            done < <(parse_subsystems 2>/dev/null)
            if [[ ${#subs[@]} -gt 0 ]]; then
                echo ""
                echo "  Subsystem task status:"
                for sub in "${subs[@]}"; do
                    if [[ -f "subsystems/$sub/plan.md" ]]; then
                        local todo done blocked inprog
                        todo=$(grep -c '\*\*Status:\*\* TODO' "subsystems/$sub/plan.md" 2>/dev/null || echo 0)
                        done=$(grep -c '\*\*Status:\*\* DONE' "subsystems/$sub/plan.md" 2>/dev/null || echo 0)
                        blocked=$(grep -c '\*\*Status:\*\* BLOCKED' "subsystems/$sub/plan.md" 2>/dev/null || echo 0)
                        inprog=$(grep -c '\*\*Status:\*\* IN_PROGRESS' "subsystems/$sub/plan.md" 2>/dev/null || echo 0)
                        echo "    $sub: TODO=$todo IN_PROGRESS=$inprog DONE=$done BLOCKED=$blocked"
                    fi
                done
            fi
        fi
    fi
    echo ""
}

# --- Main ---------------------------------------------------------------------

usage() {
    echo "Lisa Loop v2 — Spiral-V development loop"
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
