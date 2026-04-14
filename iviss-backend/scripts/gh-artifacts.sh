#!/bin/bash
# Script to manage GitHub Actions artifacts via CLI
# Requires: gh CLI (github.com/cli/cli) + token with `repo` scope

set -e

REPO="${1:-skyengpro/iviss}"
COMMAND="${2:-list}"
DAYS="${3:-7}"

function list_artifacts() {
    echo "=== Artifacts in $REPO ==="
    gh api "repos/$REPO/actions/artifacts" --jq '
        .artifacts[] |
        "\(.id) | \(.name) | \(.size_in_bytes / 1024 / 1024 | round)MB | \(.created_at) | \(.expired)"
    ' | column -t -s '|' -N "ID,Name,Size,Created,Expired"
    echo ""
    echo "Total: $(gh api "repos/$REPO/actions/artifacts" --jq '.artifacts | length') artifacts"
    echo "Total size: $(gh api "repos/$REPO/actions/artifacts" --jq '[.artifacts[].size_in_bytes] | add / 1024 / 1024 | round') MB"
}

function delete_old_artifacts() {
    local cutoff_date=$(date -d "$DAYS days ago" -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -v-${DAYS}d -u +%Y-%m-%dT%H:%M:%SZ)
    echo "Deleting artifacts created before: $cutoff_date"
    
    local artifacts=$(gh api "repos/$REPO/actions/artifacts?per_page=100" --jq "
        .artifacts[] | select(.created_at < \"$cutoff_date\" and .expired == false) | .id
    ")
    
    if [ -z "$artifacts" ]; then
        echo "No artifacts to delete"
        return 0
    fi
    
    echo "$artifacts" | while read -r artifact_id; do
        echo "Deleting artifact #$artifact_id..."
        gh api "repos/$REPO/actions/artifacts/$artifact_id" -X DELETE --silent
        echo "✓ Artifact #$artifact_id deleted"
    done
    
    echo "Cleanup completed"
}

function delete_all_artifacts() {
    echo "⚠️  WARNING: Deleting ALL artifacts!"
    read -p "Continue? (yes): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Cancelled"
        exit 0
    fi
    
    local artifacts=$(gh api "repos/$REPO/actions/artifacts?per_page=100" --jq '.artifacts[].id')
    
    if [ -z "$artifacts" ]; then
        echo "No artifacts to delete"
        return 0
    fi
    
    echo "$artifacts" | while read -r artifact_id; do
        gh api "repos/$REPO/actions/artifacts/$artifact_id" -X DELETE --silent
        echo "✓ #$artifact_id deleted"
    done
}

case "$COMMAND" in
    list|ls)
        list_artifacts
        ;;
    delete-old|clean)
        delete_old_artifacts
        ;;
    delete-all|wipe)
        delete_all_artifacts
        ;;
    *)
        echo "Usage: $0 [REPO] [COMMAND] [DAYS]"
        echo ""
        echo "Commands:"
        echo "  list                 - List all artifacts"
        echo "  delete-old [DAYS]    - Delete artifacts older than N days (default: 7)"
        echo "  delete-all           - Delete ALL artifacts (⚠️)"
        echo ""
        echo "Examples:"
        echo "  $0 skyengpro/iviss list"
        echo "  $0 skyengpro/iviss delete-old 3"
        echo "  $0 skyengpro/iviss delete-all"
        exit 1
        ;;
esac
