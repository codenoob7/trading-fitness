#!/usr/bin/env bash
# Generate type definitions from JSON Schema
# Usage: ./scripts/generate-types.sh [python|typescript|rust|all]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
SCHEMAS_DIR="$ROOT_DIR/packages/shared-types/schemas"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

generate_python() {
    log_info "Generating Python models from JSON Schema..."

    if ! command -v datamodel-codegen &> /dev/null; then
        log_warn "datamodel-codegen not found. Install with: uv pip install datamodel-code-generator"
        return 1
    fi

    local output_dir="$ROOT_DIR/packages/ith-python/src/ith_python/generated"
    mkdir -p "$output_dir"

    datamodel-codegen \
        --input "$SCHEMAS_DIR" \
        --output "$output_dir/models.py" \
        --input-file-type jsonschema \
        --output-model-type pydantic_v2.BaseModel \
        --target-python-version 3.12 \
        --use-standard-collections \
        --use-union-operator

    log_info "Python models generated at $output_dir/models.py"
}

generate_typescript() {
    log_info "Generating TypeScript types from JSON Schema..."

    local output_dir="$ROOT_DIR/packages/core-bun/src/generated"
    mkdir -p "$output_dir"

    # Check if json-schema-to-zod is available
    if ! bunx json-schema-to-zod --help &> /dev/null 2>&1; then
        log_warn "json-schema-to-zod not found. Install with: bun add -D json-schema-to-zod"
        return 1
    fi

    # Generate from each schema
    for schema in "$SCHEMAS_DIR"/*.json; do
        local name
        name=$(basename "$schema" .json | tr '-' '_')
        bunx json-schema-to-zod -s "$schema" -o "$output_dir/${name}.ts" 2>/dev/null || {
            log_warn "Failed to generate from $schema"
        }
    done

    log_info "TypeScript types generated at $output_dir/"
}

generate_rust() {
    log_info "Generating Rust types from JSON Schema..."

    if ! command -v typify &> /dev/null; then
        log_warn "typify not found. Install with: cargo install typify-cli"
        return 1
    fi

    local output_dir="$ROOT_DIR/packages/core-rust/src/generated"
    mkdir -p "$output_dir"

    for schema in "$SCHEMAS_DIR"/*.json; do
        local name
        name=$(basename "$schema" .json | tr '-' '_')
        typify "$schema" > "$output_dir/${name}.rs" 2>/dev/null || {
            log_warn "Failed to generate from $schema"
        }
    done

    log_info "Rust types generated at $output_dir/"
}

show_usage() {
    echo "Usage: $0 [python|typescript|rust|all]"
    echo ""
    echo "Generate type definitions from JSON Schema files."
    echo ""
    echo "Commands:"
    echo "  python      Generate Python pydantic models"
    echo "  typescript  Generate TypeScript zod schemas"
    echo "  rust        Generate Rust serde structs"
    echo "  all         Generate all targets"
    echo ""
    echo "Prerequisites:"
    echo "  - Python: uv pip install datamodel-code-generator"
    echo "  - TypeScript: bun add -D json-schema-to-zod"
    echo "  - Rust: cargo install typify-cli"
}

main() {
    local target="${1:-}"

    case "$target" in
        python)
            generate_python
            ;;
        typescript)
            generate_typescript
            ;;
        rust)
            generate_rust
            ;;
        all)
            generate_python || true
            generate_typescript || true
            generate_rust || true
            ;;
        -h|--help|"")
            show_usage
            ;;
        *)
            log_error "Unknown target: $target"
            show_usage
            exit 1
            ;;
    esac
}

main "$@"
