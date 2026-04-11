#!/usr/bin/env bash
# Create a new feature branch and spec directory. Run from repo root.
# Usage: .claude/skills/speckit/scripts/bash/create-new-feature.sh [--json] [--short-name NAME] [--number N] "Feature description"

set -e

SCRIPT_DIR="$(CDPATH="" cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"
SPECKIT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

JSON_MODE=false
SHORT_NAME=""
BRANCH_NUMBER=""
ARGS=()
i=1
while [ $i -le $# ]; do
    arg="${!i}"
    case "$arg" in
        --json) JSON_MODE=true ;;
        --short-name)
            if [ $((i + 1)) -gt $# ]; then echo 'Error: --short-name requires a value' >&2; exit 1; fi
            i=$((i + 1)); next_arg="${!i}"
            [[ "$next_arg" == --* ]] && { echo 'Error: --short-name requires a value' >&2; exit 1; }
            SHORT_NAME="$next_arg"
            ;;
        --number)
            if [ $((i + 1)) -gt $# ]; then echo 'Error: --number requires a value' >&2; exit 1; fi
            i=$((i + 1)); next_arg="${!i}"
            [[ "$next_arg" == --* ]] && { echo 'Error: --number requires a value' >&2; exit 1; }
            BRANCH_NUMBER="$next_arg"
            ;;
        --help|-h)
            echo "Usage: $0 [--json] [--short-name <name>] [--number N] <feature_description>"
            exit 0
            ;;
        *) ARGS+=("$arg") ;;
    esac
    i=$((i + 1))
done

FEATURE_DESCRIPTION="${ARGS[*]}"
FEATURE_DESCRIPTION=$(echo "$FEATURE_DESCRIPTION" | xargs)
if [ -z "$FEATURE_DESCRIPTION" ]; then
    echo "Error: Feature description cannot be empty" >&2
    exit 1
fi

find_repo_root() {
    local dir="$1"
    while [ "$dir" != "/" ]; do
        if [ -d "$dir/.git" ] || [ -d "$dir/.claude" ]; then
            echo "$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    return 1
}

get_highest_from_specs() {
    local specs_dir="$1"
    local highest=0
    [ -d "$specs_dir" ] || { echo 0; return; }
    for dir in "$specs_dir"/*; do
        [ -d "$dir" ] || continue
        dirname=$(basename "$dir")
        number=$(echo "$dirname" | grep -oE '^[0-9]+' || echo "0")
        number=$((10#$number))
        [ "$number" -gt "$highest" ] && highest=$number
    done
    echo "$highest"
}

get_highest_from_branches() {
    local highest=0
    local branches=$(git branch -a 2>/dev/null || echo "")
    [ -z "$branches" ] && { echo 0; return; }
    while IFS= read -r branch; do
        clean_branch=$(echo "$branch" | sed 's/^[* ]*//; s|^remotes/[^/]*/||')
        if echo "$clean_branch" | grep -qE '^[0-9]{3}-'; then
            number=$(echo "$clean_branch" | grep -oE '^[0-9]{3}' || echo "0")
            number=$((10#$number))
            [ "$number" -gt "$highest" ] && highest=$number
        fi
    done <<< "$branches"
    echo "$highest"
}

check_existing_branches() {
    local specs_dir="$1"
    git fetch --all --prune 2>/dev/null || true
    local highest_branch=$(get_highest_from_branches)
    local highest_spec=$(get_highest_from_specs "$specs_dir")
    local max_num=$highest_branch
    [ "$highest_spec" -gt "$max_num" ] && max_num=$highest_spec
    echo $((max_num + 1))
}

clean_branch_name() {
    echo "$1" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/-\+/-/g' | sed 's/^-//' | sed 's/-$//'
}

generate_branch_name() {
    local description="$1"
    local stop_words="^(i|a|an|the|to|for|of|in|on|at|by|with|from|is|are|was|were|be|been|being|have|has|had|do|does|did|will|would|should|could|can|may|might|must|shall|this|that|these|those|my|your|our|their|want|need|add|get|set)$"
    local clean_name=$(echo "$description" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/ /g')
    local meaningful_words=()
    for word in $clean_name; do
        [ -z "$word" ] && continue
        if ! echo "$word" | grep -qiE "$stop_words"; then
            [ ${#word} -ge 3 ] && meaningful_words+=("$word")
        fi
    done
    if [ ${#meaningful_words[@]} -gt 0 ]; then
        local max_words=3
        [ ${#meaningful_words[@]} -eq 4 ] && max_words=4
        local result="" count=0
        for word in "${meaningful_words[@]}"; do
            [ $count -ge $max_words ] && break
            [ -n "$result" ] && result="$result-"
            result="$result$word"
            count=$((count + 1))
        done
        echo "$result"
    else
        clean_branch_name "$description" | tr '-' '\n' | grep -v '^$' | head -3 | tr '\n' '-' | sed 's/-$//'
    fi
}

extract_cap_number() {
    # Extract CAP numeric key from arbitrary text.
    # Supports: CAP-1234, CAP 1234, cap1234, cap-1234, etc.
    local text="$1"
    local match
    match="$(echo "$text" | grep -oiE '[cC][aA][pP][-_. ]*[0-9]+' | head -1 || true)"
    if [ -z "$match" ]; then
        echo ""
        return 0
    fi
    echo "$match" | sed -E 's/[^0-9]//g'
}

strip_cap_from_slug() {
    # Remove CAP token(s) from an already-cleaned slug.
    # Examples of cleaned tokens:
    # - cap8395
    # - cap-8395  -> tokens: cap + 8395
    local slug="$1"
    local cap_number="$2"

    [ -z "$slug" ] && { echo ""; return 0; }
    [ -z "$cap_number" ] && { echo "$slug"; return 0; }

    local cap_token="cap${cap_number}"
    local IFS='-'
    read -ra tokens <<< "$slug"

    local out=()
    local i=0
    while [ $i -lt ${#tokens[@]} ]; do
        if [ "${tokens[$i]}" = "$cap_token" ]; then
            i=$((i + 1))
            continue
        fi

        if [ "${tokens[$i]}" = "cap" ] && [ $((i + 1)) -lt ${#tokens[@]} ] && [ "${tokens[$((i + 1))]}" = "$cap_number" ]; then
            i=$((i + 2))
            continue
        fi

        out+=("${tokens[$i]}")
        i=$((i + 1))
    done

    local result
    result="$(IFS='-'; echo "${out[*]}")"
    result="$(echo "$result" | sed 's/^-//; s/-$//; s/--*/-/g')"
    echo "$result"
}

if git rev-parse --show-toplevel >/dev/null 2>&1; then
    REPO_ROOT=$(git rev-parse --show-toplevel)
    HAS_GIT=true
else
    REPO_ROOT="$(find_repo_root "$SCRIPT_DIR")"
    [ -z "$REPO_ROOT" ] && { echo "Error: Could not determine repository root." >&2; exit 1; }
    HAS_GIT=false
fi

cd "$REPO_ROOT"
SPECS_DIR="$REPO_ROOT/specs"
mkdir -p "$SPECS_DIR"

if [ -n "$SHORT_NAME" ]; then
    BRANCH_SUFFIX=$(clean_branch_name "$SHORT_NAME")
else
    BRANCH_SUFFIX=$(generate_branch_name "$FEATURE_DESCRIPTION")
fi

USE_CAP_STYLE=false
CAP_NUMBER=""
BRANCH_PREFIX=""
BRANCH_SUFFIX_USED="$BRANCH_SUFFIX"
FEATURE_NUM=""

CAP_NUMBER="$(extract_cap_number "$FEATURE_DESCRIPTION")"
if [ -z "$CAP_NUMBER" ] && [ -n "$SHORT_NAME" ]; then
    CAP_NUMBER="$(extract_cap_number "$SHORT_NAME")"
fi

if [ -n "$CAP_NUMBER" ]; then
    USE_CAP_STYLE=true
    BRANCH_SUFFIX_USED="$(strip_cap_from_slug "$BRANCH_SUFFIX_USED" "$CAP_NUMBER")"
    if [ -n "$BRANCH_SUFFIX_USED" ]; then
        BRANCH_PREFIX="CAP-${CAP_NUMBER}-"
        BRANCH_NAME="${BRANCH_PREFIX}${BRANCH_SUFFIX_USED}"
    else
        BRANCH_PREFIX="CAP-${CAP_NUMBER}"
        BRANCH_NAME="${BRANCH_PREFIX}"
    fi
else
    if [ -z "$BRANCH_NUMBER" ]; then
        if [ "$HAS_GIT" = true ]; then
            BRANCH_NUMBER=$(check_existing_branches "$SPECS_DIR")
        else
            BRANCH_NUMBER=$(($(get_highest_from_specs "$SPECS_DIR") + 1))
        fi
    fi

    FEATURE_NUM=$(printf "%03d" "$((10#$BRANCH_NUMBER))")
    BRANCH_PREFIX="${FEATURE_NUM}-"
    BRANCH_NAME="${BRANCH_PREFIX}${BRANCH_SUFFIX_USED}"
fi

MAX_BRANCH_LENGTH=244
if [ ${#BRANCH_NAME} -gt $MAX_BRANCH_LENGTH ]; then
    if [ "$USE_CAP_STYLE" = true ] || [ -n "$FEATURE_NUM" ]; then
        # Truncate only the suffix portion, keeping the prefix stable.
        max_suffix_length=$((MAX_BRANCH_LENGTH - ${#BRANCH_PREFIX}))
        if [ $max_suffix_length -gt 0 ] && [ -n "$BRANCH_SUFFIX_USED" ]; then
            truncated_suffix="$(echo "$BRANCH_SUFFIX_USED" | cut -c1-$max_suffix_length | sed 's/-$//')"
            BRANCH_NAME="${BRANCH_PREFIX}${truncated_suffix}"
        else
            BRANCH_NAME="$(echo "$BRANCH_NAME" | cut -c1-$MAX_BRANCH_LENGTH | sed 's/-$//')"
        fi
    else
        BRANCH_NAME="$(echo "$BRANCH_NAME" | cut -c1-$MAX_BRANCH_LENGTH | sed 's/-$//')"
    fi
    echo "[speckit] Warning: Branch name truncated to $MAX_BRANCH_LENGTH bytes" >&2
fi

if [ "$HAS_GIT" = true ]; then
    if ! git checkout -b "$BRANCH_NAME" 2>/dev/null; then
        if git branch --list "$BRANCH_NAME" | grep -q .; then
            echo "Error: Branch '$BRANCH_NAME' already exists." >&2
            exit 1
        fi
        echo "Error: Failed to create git branch '$BRANCH_NAME'." >&2
        exit 1
    fi
else
    echo "[speckit] Warning: Git not detected; skipped branch creation for $BRANCH_NAME" >&2
fi

FEATURE_DIR="$SPECS_DIR/$BRANCH_NAME"
mkdir -p "$FEATURE_DIR"

TEMPLATE=$(resolve_template "spec-template" "$REPO_ROOT" "$SPECKIT_ROOT")
SPEC_FILE="$FEATURE_DIR/spec.md"
if [ -n "$TEMPLATE" ] && [ -f "$TEMPLATE" ]; then
    cp "$TEMPLATE" "$SPEC_FILE"
else
    touch "$SPEC_FILE"
fi

printf '# To persist: export SPECIFY_FEATURE=%q\n' "$BRANCH_NAME" >&2

if $JSON_MODE; then
    if command -v jq >/dev/null 2>&1; then
        jq -cn \
            --arg branch_name "$BRANCH_NAME" \
            --arg spec_file "$SPEC_FILE" \
            --arg feature_num "$FEATURE_NUM" \
            '{BRANCH_NAME:$branch_name,SPEC_FILE:$spec_file,FEATURE_NUM:$feature_num}'
    else
        printf '{"BRANCH_NAME":"%s","SPEC_FILE":"%s","FEATURE_NUM":"%s"}\n' "$(json_escape "$BRANCH_NAME")" "$(json_escape "$SPEC_FILE")" "$(json_escape "$FEATURE_NUM")"
    fi
else
    echo "BRANCH_NAME: $BRANCH_NAME"
    echo "SPEC_FILE: $SPEC_FILE"
    echo "FEATURE_NUM: $FEATURE_NUM"
    printf '# To persist: export SPECIFY_FEATURE=%q\n' "$BRANCH_NAME"
fi
