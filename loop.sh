#!/usr/bin/env bash
set -euo pipefail

# Lisa Loop v2 — Spiral-V development loop for engineering and scientific software
#
# Architecture: Outer spiral (convergence-driven, human-gated) with inner
# Ralph loop (autonomous task execution) at each pass's build phase.
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
CLAUDE_MODEL_DESCEND="${CLAUDE_MODEL_DESCEND:-opus}"
CLAUDE_MODEL_BUILD="${CLAUDE_MODEL_BUILD:-sonnet}"
CLAUDE_MODEL_ASCEND="${CLAUDE_MODEL_ASCEND:-opus}"

# Loop limits
MAX_SPIRAL_PASSES="${MAX_SPIRAL_PASSES:-5}"
MAX_RALPH_ITERATIONS="${MAX_RALPH_ITERATIONS:-50}"
MAX_RALPH_BLOCKED_RETRIES="${MAX_RALPH_BLOCKED_RETRIES:-1}"

# Human review
REVIEW_DESCEND="${REVIEW_DESCEND:-false}"
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
    # and emit human-readable progress lines showing agent tool calls, plus the
    # final result text.
    #
    # Falls back to raw passthrough if jq is not available.
    if ! command -v jq &>/dev/null; then
        cat
        return
    fi
    jq --unbuffered -r '
      if .type == "assistant" then
        [.message.content[]? | select(.type == "tool_use") |
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
        ] | .[] | select(length > 0)
      elif .type == "result" then
        "RESULT_B64: " + ((.result // "") | @base64)
      else empty end
    ' 2>/dev/null | while IFS= read -r line; do
        if [[ "$line" == TOOL:\ * ]]; then
            echo -e "${MAGENTA}  [agent $(_ts)]${NC} ${line#TOOL: }"
        elif [[ "$line" == RESULT_B64:\ * ]]; then
            local result_b64="${line#RESULT_B64: }"
            local result_text
            result_text="$(echo "$result_b64" | base64 -d 2>/dev/null)" || result_text=""
            if [[ -n "$result_text" ]]; then
                echo ""
                echo -e "${MAGENTA}  [agent $(_ts)]${NC} ── Agent response ──"
                echo "$result_text"
                echo -e "${MAGENTA}  [agent $(_ts)]${NC} ── End agent response ──"
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

# --- Ralph Loop Detection ----------------------------------------------------

count_tasks_for_pass() {
    # Count tasks for a given pass with a given status
    # Usage: count_tasks_for_pass <pass_number> <status>
    local pass="$1"
    local status="$2"

    if [[ ! -f "IMPLEMENTATION_PLAN.md" ]]; then
        echo 0
        return
    fi

    # Use awk to parse task blocks: find tasks matching the pass and status.
    # A task block starts with "### Task" and ends at the next "### Task" or EOF.
    # Within a block, check for "**Spiral pass:** N" and "**Status:** STATUS".
    awk -v pass="$pass" -v status="$status" '
        /^### Task/ { in_task=1; found_pass=0; found_status=0; next }
        in_task && /\*\*Spiral pass:\*\*/ {
            # Extract pass number — match digits after the tag
            match($0, /\*\*Spiral pass:\*\*[[:space:]]*([0-9]+)/, arr)
            if (arr[1] == pass) found_pass=1
        }
        in_task && /\*\*Status:\*\*/ {
            if (index($0, status)) found_status=1
        }
        in_task && found_pass && found_status { count++; in_task=0 }
        END { print count+0 }
    ' IMPLEMENTATION_PLAN.md
}

all_pass_tasks_done() {
    # Returns 0 (true) if no TODO or IN_PROGRESS tasks remain for the given pass
    local pass="$1"
    local todo_count
    local inprog_count
    todo_count=$(count_tasks_for_pass "$pass" "TODO")
    inprog_count=$(count_tasks_for_pass "$pass" "IN_PROGRESS")
    [[ "$todo_count" -eq 0 && "$inprog_count" -eq 0 ]]
}

has_blocked_tasks() {
    # Returns 0 (true) if any tasks for the given pass are BLOCKED
    local pass="$1"
    local blocked_count
    blocked_count=$(count_tasks_for_pass "$pass" "BLOCKED")
    [[ "$blocked_count" -gt 0 ]]
}

# --- Human Interaction Gates --------------------------------------------------

review_gate() {
    # Mandatory review after ascend phase
    local pass="$1"
    if [[ "$NO_PAUSE" == "true" || "$NO_PAUSE" == "1" ]]; then
        log_warn "Review gate skipped (NO_PAUSE=$NO_PAUSE) — defaulting to CONTINUE"
        return 0  # continue
    fi
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  SPIRAL PASS $pass COMPLETE — REVIEW REQUIRED${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""
    echo "  Review package: spiral/pass-$pass/review-package.md"
    echo "  Plots:          plots/REVIEW.md"
    echo ""
    echo "  [A] ACCEPT — Answer has converged. Produce final report."
    echo "  [C] CONTINUE — Proceed to Pass $((pass + 1))."
    echo "  [R] REDIRECT — Provide guidance for Pass $((pass + 1))."
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    while true; do
        read -rp "  Your choice [A/C/R]: " choice
        case "${choice^^}" in
            A)
                log_success "ACCEPTED — producing final output."
                return 1  # accept
                ;;
            C)
                log_info "CONTINUE — proceeding to next pass."
                return 0  # continue
                ;;
            R)
                echo ""
                echo "  Enter your guidance for the next pass (end with an empty line):"
                local redirect_text=""
                while IFS= read -r line; do
                    [[ -z "$line" ]] && break
                    redirect_text+="$line"$'\n'
                done
                mkdir -p "spiral/pass-$pass"
                cat > "spiral/pass-$pass/human-redirect.md" <<EOF
# Human Redirect — Pass $pass

$redirect_text
EOF
                log_info "REDIRECT — guidance saved to spiral/pass-$pass/human-redirect.md"
                return 0  # continue with redirect
                ;;
            *)
                echo "  Please enter A, C, or R."
                ;;
        esac
    done
}

block_gate() {
    # Shown when all remaining tasks are blocked during build
    local pass="$1"
    local completed="$2"
    local total="$3"
    local blocked="$4"

    if [[ "$NO_PAUSE" == "true" || "$NO_PAUSE" == "1" ]]; then
        log_warn "Block gate skipped (NO_PAUSE=$NO_PAUSE) — defaulting to SKIP"
        return 1  # skip
    fi
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  BUILD PHASE BLOCKED — HUMAN INPUT NEEDED${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""
    echo "  Completed: $completed/$total tasks"
    echo "  Blocked:   $blocked tasks"
    echo ""
    echo "  See IMPLEMENTATION_PLAN.md for blocked items and details."
    echo ""
    echo "  [F] FIX — Resolve the blocks, then resume build."
    echo "  [S] SKIP — Skip blocked items, proceed to Ascend."
    echo "  [X] ABORT — Stop this spiral pass."
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    while true; do
        read -rp "  Your choice [F/S/X]: " choice
        case "${choice^^}" in
            F)
                log_info "FIX — resolve blocks in IMPLEMENTATION_PLAN.md, then the build loop will resume."
                return 0  # fix — resume build loop
                ;;
            S)
                log_info "SKIP — proceeding to ascend phase."
                return 1  # skip — exit build loop
                ;;
            X)
                log_error "ABORT — stopping spiral pass."
                return 2  # abort
                ;;
            *)
                echo "  Please enter F, S, or X."
                ;;
        esac
    done
}

descend_gate() {
    # Optional review after descend phase (if REVIEW_DESCEND=true)
    local pass="$1"
    if [[ "$REVIEW_DESCEND" != "true" ]]; then
        return 0  # proceed without review
    fi
    if [[ "$NO_PAUSE" == "true" || "$NO_PAUSE" == "1" ]]; then
        log_warn "Descend review skipped (NO_PAUSE=$NO_PAUSE)"
        return 0
    fi
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  DESCEND COMPLETE — METHODOLOGY REVIEW${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""
    echo "  Methodology: spiral/pass-$pass/descend-summary.md"
    echo "  Updated plan: IMPLEMENTATION_PLAN.md"
    echo ""
    echo "  [P] PROCEED — Start building."
    echo "  [R] REDIRECT — Adjust before building."
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    while true; do
        read -rp "  Your choice [P/R]: " choice
        case "${choice^^}" in
            P)
                log_info "PROCEED — starting build phase."
                return 0
                ;;
            R)
                echo ""
                echo "  Enter your guidance (end with an empty line):"
                local redirect_text=""
                while IFS= read -r line; do
                    [[ -z "$line" ]] && break
                    redirect_text+="$line"$'\n'
                done
                mkdir -p "spiral/pass-$pass"
                cat > "spiral/pass-$pass/human-redirect.md" <<EOF
# Human Redirect — Pass $pass (Descend Review)

$redirect_text
EOF
                log_info "Guidance saved. Re-running descend phase."
                return 1  # redirect — re-run descend
                ;;
            *)
                echo "  Please enter P or R."
                ;;
        esac
    done
}

scope_review_gate() {
    # Mandatory review after Pass 0 scoping
    if [[ "$NO_PAUSE" == "true" || "$NO_PAUSE" == "1" ]]; then
        log_warn "Scope review skipped (NO_PAUSE=$NO_PAUSE)"
        return
    fi
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  PASS 0 (SCOPING) COMPLETE — REVIEW REQUIRED${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""
    echo "  Review the following artifacts:"
    echo "    spiral/pass-0/acceptance-criteria.md"
    echo "    spiral/pass-0/validation-strategy.md"
    echo "    spiral/pass-0/sanity-checks.md"
    echo "    spiral/pass-0/literature-survey.md"
    echo "    spiral/pass-0/spiral-plan.md"
    echo "    IMPLEMENTATION_PLAN.md"
    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""
    read -rp "  Press ENTER to approve and continue, or Ctrl+C to stop and edit... "
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

    scope_review_gate

    write_state 0 "scope" "complete"
    log_success "Pass 0 (scoping) complete."
}

# --- Phase: Descend -----------------------------------------------------------

run_descend() {
    local pass="$1"
    log_phase "PASS $pass — DESCEND (methodology refinement)"

    write_state "$pass" "descend" "in_progress"
    mkdir -p "spiral/pass-$pass"

    # Build context string for the agent
    local prev_pass=$((pass - 1))
    local context="Current spiral pass: $pass"
    context+=$'\n'"Previous pass results: spiral/pass-$prev_pass/"
    if [[ -f "spiral/pass-$prev_pass/human-redirect.md" ]]; then
        context+=$'\n'"Human redirect file: spiral/pass-$prev_pass/human-redirect.md"
    fi

    log_info "Running descend agent..."
    run_agent "prompts/PROMPT_descend.md" "$CLAUDE_MODEL_DESCEND" "$context"

    log_info "Committing descend artifacts..."
    git_commit_all "descend: pass $pass — methodology refinement" || true

    # Optional descend review
    descend_gate "$pass"
    local gate_result=$?
    if [[ $gate_result -eq 1 ]]; then
        # Redirect — re-run descend
        log_info "Re-running descend with redirect guidance..."
        run_agent "prompts/PROMPT_descend.md" "$CLAUDE_MODEL_DESCEND" "$context"
        git_commit_all "descend: pass $pass — post-redirect refinement" || true
    fi

    write_state "$pass" "descend" "complete"
}

# --- Phase: Build (Ralph Loop) -----------------------------------------------

run_build() {
    local pass="$1"
    log_phase "PASS $pass — BUILD (Ralph loop)"

    write_state "$pass" "build" "in_progress" 0

    local context="Current spiral pass: $pass"

    for ((iter = 1; iter <= MAX_RALPH_ITERATIONS; iter++)); do
        echo ""
        log_phase "Build — Pass $pass, Iteration $iter / $MAX_RALPH_ITERATIONS"

        write_state "$pass" "build" "in_progress" "$iter"

        log_info "Running build agent (iteration $iter)..."
        run_agent "prompts/PROMPT_build.md" "$CLAUDE_MODEL_BUILD" "$context"

        log_info "Committing build work..."
        git_commit_all "build: pass $pass iteration $iter" || true

        # Check if all tasks for this pass are done
        if all_pass_tasks_done "$pass"; then
            if has_blocked_tasks "$pass"; then
                log_warn "All non-blocked tasks complete. Some tasks are BLOCKED."
                local done_count blocked_count total_count
                done_count=$(count_tasks_for_pass "$pass" "DONE")
                blocked_count=$(count_tasks_for_pass "$pass" "BLOCKED")
                total_count=$((done_count + blocked_count))

                block_gate "$pass" "$done_count" "$total_count" "$blocked_count"
                local gate_result=$?
                if [[ $gate_result -eq 0 ]]; then
                    # Fix — continue build loop (human resolved blocks)
                    continue
                elif [[ $gate_result -eq 2 ]]; then
                    # Abort
                    log_error "Build aborted by user."
                    return 1
                fi
                # Skip — fall through to exit build loop
            fi
            log_success "All tasks for pass $pass complete."
            break
        fi

        log_info "Tasks remain — continuing Ralph loop."
    done

    if [[ $iter -gt $MAX_RALPH_ITERATIONS ]]; then
        log_warn "Reached max Ralph iterations ($MAX_RALPH_ITERATIONS). Some tasks may remain."
    fi

    write_state "$pass" "build" "complete"
    return 0
}

# --- Phase: Ascend ------------------------------------------------------------

run_ascend() {
    local pass="$1"
    log_phase "PASS $pass — ASCEND (verification, validation, convergence)"

    write_state "$pass" "ascend" "in_progress"

    # Build context string
    local prev_pass=$((pass - 1))
    local context="Current spiral pass: $pass"
    context+=$'\n'"Previous pass results: spiral/pass-$prev_pass/"

    log_info "Running ascend agent..."
    run_agent "prompts/PROMPT_ascend.md" "$CLAUDE_MODEL_ASCEND" "$context"

    log_info "Committing ascend artifacts..."
    git_commit_all "ascend: pass $pass — V&V and convergence" || true

    log_info "Pushing to remote..."
    git_push

    write_state "$pass" "ascend" "complete"
}

# --- Finalize -----------------------------------------------------------------

finalize_output() {
    log_phase "FINALIZING — Producing deliverables"

    # Create SPIRAL_COMPLETE.md
    cat > "spiral/SPIRAL_COMPLETE.md" <<EOF
# Spiral Complete

The spiral has converged and the human has accepted the results.

Completed: $(date -Iseconds)
Final pass: $1
EOF

    # If output files don't exist yet, note that ascend should have drafted them
    if [[ ! -f "output/answer.md" ]]; then
        log_warn "output/answer.md not found — check spiral/pass-$1/review-package.md for the answer."
    fi
    if [[ ! -f "output/report.md" ]]; then
        log_warn "output/report.md not found — check spiral artifacts for report content."
    fi

    git_commit_all "final: spiral complete — answer accepted at pass $1" || true
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
    if [[ ! -f "spiral/pass-0/PASS_COMPLETE.md" ]]; then
        log_info "Pass 0 (scoping) not complete. Running scope first."
        run_scope
    else
        log_info "Pass 0 already complete. Starting spiral passes."
    fi

    # Spiral passes
    for ((pass = 1; pass <= max_passes; pass++)); do
        echo ""
        log_phase "═══ SPIRAL PASS $pass / $max_passes ═══"

        # Skip completed passes (for resume support)
        if [[ -f "spiral/pass-$pass/PASS_COMPLETE.md" ]]; then
            log_info "Pass $pass already complete — skipping."
            continue
        fi

        # 1. Descend
        run_descend "$pass"

        # 2. Build (Ralph loop)
        if ! run_build "$pass"; then
            log_error "Build phase aborted at pass $pass."
            return 1
        fi

        # 3. Ascend
        run_ascend "$pass"

        # 4. Human review gate
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

    case "$STATE_PHASE" in
        descend)
            if [[ "$STATE_STATUS" != "complete" ]]; then
                log_info "Resuming: descend phase of pass $pass."
                run_descend "$pass"
            fi
            # Continue with build, ascend, review
            run_build "$pass" || return 1
            run_ascend "$pass"
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            # Continue with remaining passes
            for ((p = pass + 1; p <= MAX_SPIRAL_PASSES; p++)); do
                run_descend "$p"
                run_build "$p" || return 1
                run_ascend "$p"
                review_gate "$p"
                gate_result=$?
                if [[ $gate_result -eq 1 ]]; then
                    finalize_output "$p"
                    return 0
                fi
            done
            ;;
        build)
            log_info "Resuming: build phase of pass $pass (from iteration $STATE_RALPH_ITER)."
            run_build "$pass" || return 1
            run_ascend "$pass"
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            for ((p = pass + 1; p <= MAX_SPIRAL_PASSES; p++)); do
                run_descend "$p"
                run_build "$p" || return 1
                run_ascend "$p"
                review_gate "$p"
                gate_result=$?
                if [[ $gate_result -eq 1 ]]; then
                    finalize_output "$p"
                    return 0
                fi
            done
            ;;
        ascend)
            if [[ "$STATE_STATUS" != "complete" ]]; then
                log_info "Resuming: ascend phase of pass $pass."
                run_ascend "$pass"
            fi
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            for ((p = pass + 1; p <= MAX_SPIRAL_PASSES; p++)); do
                run_descend "$p"
                run_build "$p" || return 1
                run_ascend "$p"
                review_gate "$p"
                gate_result=$?
                if [[ $gate_result -eq 1 ]]; then
                    finalize_output "$p"
                    return 0
                fi
            done
            ;;
        review)
            log_info "Resuming: review gate of pass $pass."
            review_gate "$pass"
            local gate_result=$?
            if [[ $gate_result -eq 1 ]]; then
                finalize_output "$pass"
                return 0
            fi
            for ((p = pass + 1; p <= MAX_SPIRAL_PASSES; p++)); do
                run_descend "$p"
                run_build "$p" || return 1
                run_ascend "$p"
                review_gate "$p"
                gate_result=$?
                if [[ $gate_result -eq 1 ]]; then
                    finalize_output "$p"
                    return 0
                fi
            done
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

        if [[ -f "IMPLEMENTATION_PLAN.md" ]]; then
            echo ""
            echo "  Implementation plan tasks:"
            local todo done blocked inprog
            todo=$(grep -c '\*\*Status:\*\* TODO' IMPLEMENTATION_PLAN.md 2>/dev/null || echo 0)
            done=$(grep -c '\*\*Status:\*\* DONE' IMPLEMENTATION_PLAN.md 2>/dev/null || echo 0)
            blocked=$(grep -c '\*\*Status:\*\* BLOCKED' IMPLEMENTATION_PLAN.md 2>/dev/null || echo 0)
            inprog=$(grep -c '\*\*Status:\*\* IN_PROGRESS' IMPLEMENTATION_PLAN.md 2>/dev/null || echo 0)
            echo "    TODO:        $todo"
            echo "    IN_PROGRESS: $inprog"
            echo "    DONE:        $done"
            echo "    BLOCKED:     $blocked"
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
