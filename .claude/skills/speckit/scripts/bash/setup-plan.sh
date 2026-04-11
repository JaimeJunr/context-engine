#!/usr/bin/env bash
# Setup plan.md for current feature. Run from repo root.
# Usage: .claude/skills/speckit/scripts/bash/setup-plan.sh [--json]

set -e

SCRIPT_DIR="$(CDPATH="" cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"
SPECKIT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

JSON_MODE=false
for arg in "$@"; do
    case "$arg" in
        --json) JSON_MODE=true ;;
        --help|-h) echo "Usage: $0 [--json]"; exit 0 ;;
        *) ;;
    esac
done

_paths_output=$(get_feature_paths) || { echo "ERROR: Failed to resolve feature paths" >&2; exit 1; }
eval "$_paths_output"
unset _paths_output
check_feature_branch "$CURRENT_BRANCH" "$HAS_GIT" || exit 1
mkdir -p "$FEATURE_DIR"

TEMPLATE=$(resolve_template "plan-template" "$REPO_ROOT" "$SPECKIT_ROOT")
if [[ -n "$TEMPLATE" ]] && [[ -f "$TEMPLATE" ]]; then
    cp "$TEMPLATE" "$IMPL_PLAN"
    echo "Copied plan template to $IMPL_PLAN"
else
    touch "$IMPL_PLAN"
    echo "Warning: Plan template not found"
fi

if $JSON_MODE; then
    if has_jq; then
        jq -cn \
            --arg feature_spec "$FEATURE_SPEC" \
            --arg impl_plan "$IMPL_PLAN" \
            --arg specs_dir "$FEATURE_DIR" \
            --arg branch "$CURRENT_BRANCH" \
            --arg has_git "$HAS_GIT" \
            '{FEATURE_SPEC:$feature_spec,IMPL_PLAN:$impl_plan,SPECS_DIR:$specs_dir,BRANCH:$branch,HAS_GIT:$has_git}'
    else
        printf '{"FEATURE_SPEC":"%s","IMPL_PLAN":"%s","SPECS_DIR":"%s","BRANCH":"%s","HAS_GIT":"%s"}\n' \
            "$(json_escape "$FEATURE_SPEC")" "$(json_escape "$IMPL_PLAN")" "$(json_escape "$FEATURE_DIR")" "$(json_escape "$CURRENT_BRANCH")" "$(json_escape "$HAS_GIT")"
    fi
else
    echo "FEATURE_SPEC: $FEATURE_SPEC"
    echo "IMPL_PLAN: $IMPL_PLAN"
    echo "SPECS_DIR: $FEATURE_DIR"
    echo "BRANCH: $CURRENT_BRANCH"
    echo "HAS_GIT: $HAS_GIT"
fi
