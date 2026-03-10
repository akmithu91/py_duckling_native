#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Step 1: Build with uv ==="
uv build --wheel --out-dir dist/

echo ""
echo "=== Step 2: Locate Duckling .so files ==="

# uv build uses an isolated env, so the marker file won't be here.
# Find ext_lib from the cargo registry cache directly.
DUCKLING_LIB_DIR=$(find "$HOME/.cargo/registry/src/cicd-572493397591.d.codeartifact.us-west-2.amazonaws.com-1708697c256814cc" -type d -path "duckling_rust-*/ext_lib" 2>/dev/null | sort -V | tail -1)

if [[ -z "$DUCKLING_LIB_DIR" || ! -d "$DUCKLING_LIB_DIR" ]]; then
    echo "ERROR: Could not find rust_duckling_host ext_lib in ~/.cargo/registry/src"
    echo "Make sure rust_duckling_host is fetched: cargo fetch"
    exit 1
fi

echo "Duckling libs at: $DUCKLING_LIB_DIR"

SO_CHECK=$(ls "$DUCKLING_LIB_DIR"/*.so* 2>/dev/null | head -1)
if [[ -z "$SO_CHECK" ]]; then
    echo "ERROR: No .so files found in $DUCKLING_LIB_DIR"
    exit 1
fi

echo ""
echo "=== Step 3: Inject .so files into wheel ==="

WHEEL=$(ls -1t dist/*.whl | head -1)
echo "Wheel: $WHEEL"

TMPDIR=$(mktemp -d)
unzip -q "$WHEEL" -d "$TMPDIR"

LIBS_DEST="$TMPDIR/py_duckling_native/libs"
mkdir -p "$LIBS_DEST"

SO_COUNT=0
for so_file in "$DUCKLING_LIB_DIR"/*.so*; do
    if [[ -f "$so_file" ]]; then
        cp "$so_file" "$LIBS_DEST/"
        SO_COUNT=$((SO_COUNT + 1))
    fi
done
echo "Copied $SO_COUNT shared libraries into wheel"

# Update RECORD
RECORD_FILE=$(find "$TMPDIR" -name "RECORD" | head -1)
if [[ -n "$RECORD_FILE" ]]; then
    for f in "$LIBS_DEST"/*; do
        BASENAME=$(basename "$f")
        echo "py_duckling_native/libs/$BASENAME,," >> "$RECORD_FILE"
    done
fi

# Repack
WHEEL_NAME=$(basename "$WHEEL")
rm "$WHEEL"
cd "$TMPDIR"
zip -q -r "$SCRIPT_DIR/dist/$WHEEL_NAME" .
cd "$SCRIPT_DIR"
rm -rf "$TMPDIR"

echo ""
echo "=== Done ==="
echo "Wheel: dist/$WHEEL_NAME"
echo ""
echo "Install with:"
echo "  uv pip install dist/$WHEEL_NAME"
