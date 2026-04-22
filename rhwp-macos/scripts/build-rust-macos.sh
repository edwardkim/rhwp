#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUT="$SCRIPT_DIR/../Frameworks"
HEADER="$ROOT/rhwp-ios/Sources/rhwp.h"
GENERATED_H="$OUT/generated_rhwp.h"
MODMAP_DIR="$OUT/modulemap"
HEADER_SYMBOLS="$OUT/rhwp_header_symbols.txt"
GENERATED_SYMBOLS="$OUT/generated_rhwp_symbols.txt"

require_tool() {
  local tool="$1"
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "ERROR: required tool not found: $tool" >&2
    exit 1
  fi
}

require_rust_target() {
  local target="$1"
  if ! rustup target list --installed | grep -qx "$target"; then
    echo "ERROR: Rust target not installed: $target" >&2
    echo "Run: rustup target add $target" >&2
    exit 1
  fi
}

require_tool cargo
require_tool rustup
require_tool cbindgen
require_tool xcodebuild
require_tool xcrun
require_rust_target aarch64-apple-darwin
require_rust_target x86_64-apple-darwin

export MACOSX_DEPLOYMENT_TARGET=12.0

cd "$ROOT"
mkdir -p "$OUT"

echo "[1/4] Rust staticlib (arm64 + x86_64)..."
cargo build --release --lib --target aarch64-apple-darwin
cargo build --release --lib --target x86_64-apple-darwin

echo "[2/4] Universal binary..."
mkdir -p "$OUT/universal"
xcrun lipo -create \
  "$ROOT/target/aarch64-apple-darwin/release/librhwp.a" \
  "$ROOT/target/x86_64-apple-darwin/release/librhwp.a" \
  -output "$OUT/universal/librhwp.a"
xcrun lipo -info "$OUT/universal/librhwp.a"

echo "[3/4] cbindgen header check..."
cbindgen --quiet --config "$ROOT/cbindgen.toml" --crate rhwp \
  --output "$GENERATED_H" "$ROOT"
grep -oE '\brhwp_[a-z_]+' "$HEADER" | sort -u > "$HEADER_SYMBOLS"
grep -oE '\brhwp_[a-z_]+' "$GENERATED_H" | sort -u > "$GENERATED_SYMBOLS"
if ! diff -u "$HEADER_SYMBOLS" "$GENERATED_SYMBOLS"; then
  echo "ERROR: generated FFI symbol set differs from $HEADER" >&2
  echo "Generated header: $GENERATED_H" >&2
  exit 1
fi
echo "FFI symbols:"
cat "$HEADER_SYMBOLS"

for field in width_pt height_pt; do
  if ! grep -q "\b$field\b" "$GENERATED_H"; then
    echo "ERROR: generated header is missing RhwpPageSize.$field" >&2
    echo "Generated header: $GENERATED_H" >&2
    exit 1
  fi
done

echo "[4/4] XCFramework..."
rm -rf "$OUT/Rhwp.xcframework" "$MODMAP_DIR"
mkdir -p "$MODMAP_DIR"
cp "$HEADER" "$MODMAP_DIR/rhwp.h"
cat > "$MODMAP_DIR/module.modulemap" <<'EOF'
module Rhwp {
  header "rhwp.h"
  export *
}
EOF

xcodebuild -create-xcframework \
  -library "$OUT/universal/librhwp.a" -headers "$MODMAP_DIR" \
  -output "$OUT/Rhwp.xcframework"

echo "Done: $OUT/Rhwp.xcframework"
du -sh "$OUT/universal/librhwp.a" "$OUT/Rhwp.xcframework"
