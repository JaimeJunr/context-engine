#!/usr/bin/env bash
# Common functions and variables for speckit scripts.
# Scripts must set SCRIPT_DIR and (when under speckit) SPECKIT_ROOT after sourcing.

# Get repository root, with fallback for non-git repositories
get_repo_root() {
    if git rev-parse --show-toplevel >/dev/null 2>&1; then
        git rev-parse --show-toplevel
    else
        local script_dir="$(CDPATH="" cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        (cd "$script_dir/../../.." && pwd)
    fi
}

# Get current branch, with fallback for non-git repositories
get_current_branch() {
    if [[ -n "${SPECIFY_FEATURE:-}" ]]; then
        echo "$SPECIFY_FEATURE"
        return
    fi
    if git rev-parse --abbrev-ref HEAD >/dev/null 2>&1; then
        git rev-parse --abbrev-ref HEAD
        return
    fi
    local repo_root=$(get_repo_root)
    local specs_dir="$repo_root/specs"
    if [[ -d "$specs_dir" ]]; then
        local latest_feature=""
        local latest_dir_by_mtime=""
        local latest_dir_mtime=0
        local highest=0
        for dir in "$specs_dir"/*; do
            if [[ -d "$dir" ]]; then
                local dirname=$(basename "$dir")
                local mtime
                mtime=$(stat -c %Y "$dir" 2>/dev/null || echo 0)
                if [[ "$mtime" -gt "$latest_dir_mtime" ]]; then
                    latest_dir_mtime="$mtime"
                    latest_dir_by_mtime="$dirname"
                fi
                if [[ "$dirname" =~ ^([0-9]{3})- ]]; then
                    local number=${BASH_REMATCH[1]}
                    number=$((10#$number))
                    if [[ "$number" -gt "$highest" ]]; then
                        highest=$number
                        latest_feature=$dirname
                    fi
                fi
            fi
        done
        if [[ -n "$latest_feature" ]]; then
            echo "$latest_feature"
            return
        elif [[ -n "$latest_dir_by_mtime" ]]; then
            echo "$latest_dir_by_mtime"
            return
        fi
    fi
    echo "main"
}

has_git() {
    git rev-parse --show-toplevel >/dev/null 2>&1
}

check_feature_branch() {
    local branch="$1"
    local has_git_repo="$2"
    if [[ "$has_git_repo" != "true" ]]; then
        echo "[speckit] Warning: Git repository not detected; skipped branch validation" >&2
        return 0
    fi
    # Speckit historicamente usava branches no formato 001-feature-name,
    # mas alguns projetos adotam CAP-<id>-... (ou prefixos do tipo fix-CAP-...).
    if [[ ! "$branch" =~ ^([0-9]{3}-|CAP-[0-9]+($|-)|[a-zA-Z]+-CAP-[0-9]+-) ]]; then
        echo "ERROR: Not on a feature branch. Current branch: $branch" >&2
        echo "Feature branches should be named like: 001-feature-name or CAP-<id>-feature-name" >&2
        return 1
    fi
    return 0
}

get_feature_dir() { echo "$1/specs/$2"; }

find_feature_dir_by_prefix() {
    local repo_root="$1"
    local branch_name="$2"
    local specs_dir="$repo_root/specs"
    if [[ ! "$branch_name" =~ ^([0-9]{3})- ]]; then
        echo "$specs_dir/$branch_name"
        return
    fi
    local prefix="${BASH_REMATCH[1]}"
    local matches=()
    if [[ -d "$specs_dir" ]]; then
        for dir in "$specs_dir"/"$prefix"-*; do
            if [[ -d "$dir" ]]; then
                matches+=("$(basename "$dir")")
            fi
        done
    fi
    if [[ ${#matches[@]} -eq 0 ]]; then
        echo "$specs_dir/$branch_name"
    elif [[ ${#matches[@]} -eq 1 ]]; then
        echo "$specs_dir/${matches[0]}"
    else
        echo "ERROR: Multiple spec directories found with prefix '$prefix': ${matches[*]}" >&2
        return 1
    fi
}

get_feature_paths() {
    local repo_root=$(get_repo_root)
    local current_branch=$(get_current_branch)
    local has_git_repo="false"
    if has_git; then has_git_repo="true"; fi
    local feature_dir
    if ! feature_dir=$(find_feature_dir_by_prefix "$repo_root" "$current_branch"); then
        echo "ERROR: Failed to resolve feature directory" >&2
        return 1
    fi
    printf 'REPO_ROOT=%q\n' "$repo_root"
    printf 'CURRENT_BRANCH=%q\n' "$current_branch"
    printf 'HAS_GIT=%q\n' "$has_git_repo"
    printf 'FEATURE_DIR=%q\n' "$feature_dir"
    printf 'FEATURE_SPEC=%q\n' "$feature_dir/spec.md"
    printf 'IMPL_PLAN=%q\n' "$feature_dir/plan.md"
    printf 'TASKS=%q\n' "$feature_dir/tasks.md"
    printf 'RESEARCH=%q\n' "$feature_dir/research.md"
    printf 'DATA_MODEL=%q\n' "$feature_dir/data-model.md"
    printf 'QUICKSTART=%q\n' "$feature_dir/quickstart.md"
    printf 'CONTRACTS_DIR=%q\n' "$feature_dir/contracts"
}

has_jq() {
    command -v jq >/dev/null 2>&1
}

json_escape() {
    local s="$1"
    s="${s//\\/\\\\}"
    s="${s//\"/\\\"}"
    s="${s//$'\n'/\\n}"
    s="${s//$'\t'/\\t}"
    s="${s//$'\r'/\\r}"
    printf '%s' "$s"
}

check_file() { [[ -f "$1" ]] && echo "  ✓ $2" || echo "  ✗ $2"; }
check_dir() { [[ -d "$1" && -n $(ls -A "$1" 2>/dev/null) ]] && echo "  ✓ $2" || echo "  ✗ $2"; }

# Resolve template path. Usage: resolve_template <template_name> <repo_root> [speckit_root]
# Order: repo_root/.claude/speckit-overrides/templates/<name>.md, then speckit_root/templates/<name>.md, then repo_root/.specify/templates (fallback).
resolve_template() {
    local template_name="$1"
    local repo_root="$2"
    local speckit_root="${3:-}"

    local override="$repo_root/.claude/speckit-overrides/templates/${template_name}.md"
    [[ -f "$override" ]] && echo "$override" && return 0

    if [[ -n "$speckit_root" ]] && [[ -f "$speckit_root/templates/${template_name}.md" ]]; then
        echo "$speckit_root/templates/${template_name}.md"
        return 0
    fi

    local legacy="$repo_root/.specify/templates/${template_name}.md"
    [[ -f "$legacy" ]] && echo "$legacy" && return 0

    return 0
}
