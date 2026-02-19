#!/usr/bin/env bash
set -euo pipefail

# Lisa Loop — Methodology-rigorous engineering software development loop
#
# Usage:
#   ./loop.sh methodology [max_iterations]   # Phase 1: Develop methodology
#   ./loop.sh plan [max_iterations]           # Phase 2: Plan implementation
#   ./loop.sh build [max_iterations]          # Phase 3: Build with methodology adherence
#   ./loop.sh review                          # One-shot methodology compliance audit
#
# Environment variables:
#   AGENT_CMD   — Agent CLI command (default: claude)
#   AGENT_ARGS  — Additional agent arguments (default: see below)
#   NO_PUSH     — Set to 1 to skip git push after build iterations
#   NO_PAUSE    — Set to 1 to skip human review pauses

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# --- Configuration -----------------------------------------------------------

AGENT_CMD="${AGENT_CMD:-claude}"
AGENT_ARGS="${AGENT_ARGS:--p --dangerously-skip-permissions --model opus --verbose}"
NO_PUSH="${NO_PUSH:-0}"
NO_PAUSE="${NO_PAUSE:-0}"

# --- Colors -------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# --- Helpers ------------------------------------------------------------------

_ts()         { date '+%H:%M:%S'; }
log_info()    { echo -e "${BLUE}[lisa $(_ts)]${NC} $*"; }
log_success() { echo -e "${GREEN}[lisa $(_ts)]${NC} $*"; }
log_warn()    { echo -e "${YELLOW}[lisa $(_ts)]${NC} $*"; }
log_error()   { echo -e "${RED}[lisa $(_ts)]${NC} $*"; }
log_phase()   { echo -e "${CYAN}[lisa $(_ts)]${NC} ━━━ $* ━━━"; }

run_agent() {
    local prompt_file="$1"
    if [[ ! -f "$prompt_file" ]]; then
        log_error "Prompt file not found: $prompt_file"
        exit 1
    fi
    log_info "Calling agent with prompt: $prompt_file"
    log_info "Agent command: $AGENT_CMD $AGENT_ARGS"
    local start_seconds=$SECONDS
    # shellcheck disable=SC2086
    cat "$prompt_file" | $AGENT_CMD $AGENT_ARGS
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
    if [[ "$NO_PUSH" == "1" ]]; then
        log_info "Skipping push (NO_PUSH=1)"
        return
    fi
    local branch
    branch="$(git rev-parse --abbrev-ref HEAD)"
    log_info "Pushing to origin/$branch..."
    git push -u origin "$branch"
}

pause_for_review() {
    local reason="$1"
    if [[ "$NO_PAUSE" == "1" ]]; then
        log_warn "Review pause skipped (NO_PAUSE=1): $reason"
        return
    fi
    echo ""
    log_warn "═══════════════════════════════════════════════════════════"
    log_warn "  HUMAN REVIEW REQUIRED: $reason"
    log_warn "═══════════════════════════════════════════════════════════"
    echo ""
    read -rp "  Press ENTER to continue, or Ctrl+C to stop... "
    echo ""
}

check_plots_changed() {
    # Check if any files in plots/ were modified in the last commit
    if git diff --name-only HEAD~1 HEAD 2>/dev/null | grep -q '^plots/'; then
        return 0
    fi
    return 1
}

check_reconsiderations() {
    # Check if any reconsideration documents exist that haven't been addressed
    local recon_dir="methodology/reconsiderations"
    if [[ -d "$recon_dir" ]]; then
        local count
        count=$(find "$recon_dir" -name '*.md' 2>/dev/null | wc -l)
        if [[ "$count" -gt 0 ]]; then
            return 0
        fi
    fi
    return 1
}

# --- Phase: Methodology -------------------------------------------------------

run_methodology() {
    local max="${1:-50}"
    log_phase "METHODOLOGY PHASE (max $max iterations)"

    for ((i = 1; i <= max; i++)); do
        echo ""
        log_phase "Methodology — Iteration $i / $max"

        # Check if methodology is already marked complete
        log_info "Checking for METHODOLOGY_COMPLETE.md..."
        if [[ -f "METHODOLOGY_COMPLETE.md" ]]; then
            log_success "METHODOLOGY_COMPLETE.md found."
            pause_for_review "Review methodology before proceeding"
            if [[ -f "METHODOLOGY_COMPLETE.md" ]]; then
                log_success "Methodology phase complete. Proceed to: ./loop.sh plan"
                return 0
            else
                log_info "METHODOLOGY_COMPLETE.md removed — continuing iterations."
            fi
        fi

        # Call agent
        log_info "Step 1/3: Running agent to identify and address a methodology gap..."
        run_agent PROMPT_methodology.md

        # Commit the iteration's work
        log_info "Step 2/3: Committing iteration work..."
        git_commit_all "methodology: iteration $i" || true

        # Check if the agent marked methodology complete
        log_info "Step 3/3: Checking if agent marked methodology complete..."
        if [[ -f "METHODOLOGY_COMPLETE.md" ]]; then
            log_success "Agent marked methodology complete after iteration $i."
            pause_for_review "Review completed methodology"
            if [[ -f "METHODOLOGY_COMPLETE.md" ]]; then
                log_success "Methodology approved. Proceed to: ./loop.sh plan"
                return 0
            else
                log_info "METHODOLOGY_COMPLETE.md removed — continuing iterations."
            fi
        else
            log_info "Methodology not yet complete — continuing to next iteration."
        fi
    done

    log_warn "Reached max iterations ($max) without methodology completion."
    return 1
}

# --- Phase: Planning ----------------------------------------------------------

run_plan() {
    local max="${1:-20}"
    log_phase "PLANNING PHASE (max $max iterations)"

    if [[ ! -f "METHODOLOGY_COMPLETE.md" ]]; then
        log_error "No METHODOLOGY_COMPLETE.md found. Run methodology phase first."
        exit 1
    fi

    for ((i = 1; i <= max; i++)); do
        echo ""
        log_phase "Planning — Iteration $i / $max"

        # Call agent
        log_info "Step 1/3: Running agent to develop implementation plan..."
        run_agent PROMPT_plan.md

        # Commit
        log_info "Step 2/3: Committing iteration work..."
        git_commit_all "plan: iteration $i" || true

        # Check if IMPLEMENTATION_PLAN.md exists and is marked ready
        log_info "Step 3/3: Checking if plan is marked complete..."
        if [[ -f "IMPLEMENTATION_PLAN.md" ]] && grep -q '^\[PLAN_COMPLETE\]' "IMPLEMENTATION_PLAN.md"; then
            log_success "Implementation plan marked complete after iteration $i."
            pause_for_review "Review implementation plan"
            log_success "Planning complete. Proceed to: ./loop.sh build"
            return 0
        else
            log_info "Plan not yet complete — continuing to next iteration."
        fi
    done

    log_warn "Reached max iterations ($max) without plan completion."
    return 1
}

# --- Phase: Building ----------------------------------------------------------

run_build() {
    local max="${1:-100}"
    log_phase "BUILDING PHASE (max $max iterations)"

    if [[ ! -f "IMPLEMENTATION_PLAN.md" ]]; then
        log_error "No IMPLEMENTATION_PLAN.md found. Run planning phase first."
        exit 1
    fi

    for ((i = 1; i <= max; i++)); do
        echo ""
        log_phase "Building — Iteration $i / $max"

        # Check for unresolved reconsiderations before starting
        log_info "Step 1/6: Checking for unresolved reconsiderations..."
        if check_reconsiderations; then
            log_warn "Unresolved methodology reconsiderations found."
            echo ""
            log_info "Reconsiderations in methodology/reconsiderations/:"
            ls -1 methodology/reconsiderations/*.md 2>/dev/null || true
            echo ""
            pause_for_review "Resolve methodology reconsiderations before continuing"
        else
            log_info "No unresolved reconsiderations."
        fi

        # Call agent
        log_info "Step 2/6: Running agent to implement next task..."
        run_agent PROMPT_build.md

        # Commit the iteration's work
        log_info "Step 3/6: Committing iteration work..."
        if git_commit_all "build: iteration $i"; then
            log_info "Step 4/6: Pushing to remote..."
            git_push

            # Check if plots changed — pause for visual review
            log_info "Step 5/6: Checking if plots were updated..."
            if check_plots_changed; then
                log_info "Plots updated in this iteration."
                if [[ -f "plots/REVIEW.md" ]]; then
                    echo ""
                    log_info "--- plots/REVIEW.md ---"
                    cat plots/REVIEW.md
                    echo ""
                    log_info "--- end ---"
                fi
                pause_for_review "Visual review of updated plots"
            else
                log_info "No plot changes in this iteration."
            fi
        else
            log_info "Step 4/6: Skipping push (no changes committed)."
            log_info "Step 5/6: Skipping plot check (no changes committed)."
        fi

        # Check for new reconsiderations created this iteration
        log_info "Step 6/6: Checking for new reconsiderations..."
        if check_reconsiderations; then
            log_warn "New methodology reconsideration raised."
            echo ""
            log_info "Reconsiderations:"
            ls -1 methodology/reconsiderations/*.md 2>/dev/null || true
            echo ""
            pause_for_review "Review methodology reconsideration"
        else
            log_info "No new reconsiderations."
        fi

        # Check if all tasks are done
        log_info "Checking if build is complete..."
        if [[ -f "IMPLEMENTATION_PLAN.md" ]] && grep -q '^\[BUILD_COMPLETE\]' "IMPLEMENTATION_PLAN.md"; then
            log_success "All implementation tasks complete after iteration $i."
            log_success "Run ./loop.sh review for a final compliance audit."
            return 0
        else
            log_info "Build not yet complete — continuing to next iteration."
        fi
    done

    log_warn "Reached max iterations ($max). Some tasks may remain."
    return 1
}

# --- Phase: Review ------------------------------------------------------------

run_review() {
    log_phase "REVIEW — One-shot methodology compliance audit"

    log_info "Step 1/2: Running agent to perform compliance audit..."
    run_agent PROMPT_review.md

    log_info "Step 2/2: Committing audit results..."
    git_commit_all "review: compliance audit" || true

    log_success "Review complete. Check REVIEW_REPORT.md for results."
}

# --- Main ---------------------------------------------------------------------

usage() {
    echo "Usage: ./loop.sh <mode> [max_iterations]"
    echo ""
    echo "Modes:"
    echo "  methodology [max]   Phase 1: Develop methodology (default max: 50)"
    echo "  plan [max]          Phase 2: Plan implementation (default max: 20)"
    echo "  build [max]         Phase 3: Build with verification (default max: 100)"
    echo "  review              One-shot compliance audit"
    echo ""
    echo "Environment variables:"
    echo "  AGENT_CMD   Agent CLI command (default: claude)"
    echo "  AGENT_ARGS  Agent CLI arguments"
    echo "  NO_PUSH     Set to 1 to skip git push after build iterations"
    echo "  NO_PAUSE    Set to 1 to skip human review pauses"
}

if [[ $# -lt 1 ]]; then
    usage
    exit 1
fi

MODE="$1"
MAX="${2:-}"

case "$MODE" in
    methodology)
        run_methodology "${MAX:-50}"
        ;;
    plan)
        run_plan "${MAX:-20}"
        ;;
    build)
        run_build "${MAX:-100}"
        ;;
    review)
        run_review
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        log_error "Unknown mode: $MODE"
        usage
        exit 1
        ;;
esac
