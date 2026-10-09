#!/usr/bin/env bash
# Keep rustdoc's hidden setup in README.src.md and publish clean examples.
set -euo pipefail

case "${1-}" in
    ""|--check) ;;
    *) echo "Usage: $0 [--check]" >&2; exit 2 ;;
esac
if (( $# > 1 )); then
    echo "Usage: $0 [--check]" >&2
    exit 2
fi

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_path="$repo_dir/rs/eros/README.src.md"
readme_path="$repo_dir/rs/eros/README.md"
generated_path="$(mktemp)"
trap 'rm -f -- "$generated_path"' EXIT

{
    echo '<!-- Generated from rs/eros/README.src.md by scripts/update-readme.sh. Edit the source, then run the script. -->'
    echo
    awk '
        /^```/ {
            if (in_fence) {
                in_fence = 0
                in_rust = 0
            } else {
                in_fence = 1
                in_rust = /^```rust([,[:space:]]|$)/
            }
            print
            next
        }
        in_rust && /^[[:space:]]*#( |$)/ { next }
        in_rust && /^[[:space:]]*##/ { sub(/##/, "#") }
        { print }
    ' "$source_path"
} > "$generated_path"

if [[ "${1-}" == --check ]]; then
    if ! cmp -s -- "$generated_path" "$readme_path"; then
        echo 'README.md is out of date. Run scripts/update-readme.sh.' >&2
        exit 1
    fi
else
    cat -- "$generated_path" > "$readme_path"
fi
