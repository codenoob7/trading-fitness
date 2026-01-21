#!/usr/bin/env bash
# affected.sh - Detect affected packages based on git changes
# PROCESS-STORM-OK: Simple grep-based detection, no recursive spawning
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  list    List affected packages"
    echo "  test    Run tests for affected packages"
    echo "  lint    Run linting for affected packages"
    exit 1
}

get_changed_files() {
    git diff --name-only HEAD~1 HEAD 2>/dev/null || git diff --name-only HEAD
}

main() {
    local cmd="${1:-}"
    local changed
    changed=$(get_changed_files)

    case "$cmd" in
        list)
            echo "Affected packages:"
            echo "$changed" | grep -q "^packages/ith-python/" && echo "  - ith-python" || true
            echo "$changed" | grep -q "^packages/core-rust/" && echo "  - core-rust" || true
            echo "$changed" | grep -q "^packages/core-bun/" && echo "  - core-bun" || true
            echo "$changed" | grep -q "^packages/shared-types/" && echo "  - shared-types" || true
            echo "$changed" | grep -q "^packages/metrics-rust/" && echo "  - metrics-rust" || true
            ;;
        test)
            echo "$changed" | grep -q "^packages/ith-python/" && (cd "$REPO_ROOT/packages/ith-python" && uv run pytest) || true
            echo "$changed" | grep -q "^packages/core-rust/" && (cd "$REPO_ROOT/packages/core-rust" && cargo test) || true
            echo "$changed" | grep -q "^packages/core-bun/" && (cd "$REPO_ROOT/packages/core-bun" && bun test) || true
            echo "$changed" | grep -q "^packages/metrics-rust/" && (cd "$REPO_ROOT" && cargo nextest run -p trading-fitness-metrics) || true
            ;;
        lint)
            echo "$changed" | grep -q "^packages/ith-python/" && (cd "$REPO_ROOT/packages/ith-python" && uv run ruff check --fix) || true
            echo "$changed" | grep -q "^packages/core-rust/" && (cd "$REPO_ROOT/packages/core-rust" && cargo clippy) || true
            echo "$changed" | grep -q "^packages/core-bun/" && (cd "$REPO_ROOT/packages/core-bun" && bun run lint) || true
            echo "$changed" | grep -q "^packages/metrics-rust/" && (cd "$REPO_ROOT" && cargo clippy -p trading-fitness-metrics) || true
            ;;
        *)
            usage
            ;;
    esac
}

main "$@"
