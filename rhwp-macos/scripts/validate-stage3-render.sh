#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUT_DIR="${1:-$ROOT/output/stage3-render}"

if [ "$#" -gt 0 ]; then
  shift
fi

SAMPLES=("$@")
if [ "${#SAMPLES[@]}" -eq 0 ]; then
  SAMPLES=(
    "$ROOT/samples/basic/KTX.hwp"
    "$ROOT/samples/basic/request.hwp"
    "$ROOT/samples/exam_kor.hwp"
  )
fi

LIB="$ROOT/rhwp-macos/Frameworks/universal/librhwp.a"
if [ ! -f "$LIB" ]; then
  echo "ERROR: missing $LIB" >&2
  echo "Run: $ROOT/rhwp-macos/scripts/build-rust-macos.sh" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
BIN="$OUT_DIR/stage3_render_check"
SWIFT_MODULE_CACHE="$OUT_DIR/swift-module-cache"
CLANG_MODULE_CACHE="$OUT_DIR/clang-module-cache"
mkdir -p "$SWIFT_MODULE_CACHE" "$CLANG_MODULE_CACHE"

swiftc -parse-as-library \
  -module-cache-path "$SWIFT_MODULE_CACHE" \
  -Xcc -fmodules-cache-path="$CLANG_MODULE_CACHE" \
  -import-objc-header "$ROOT/rhwp-ios/Sources/rhwp-Bridging-Header.h" \
  "$ROOT/rhwp-ios/Sources/RhwpDocument.swift" \
  "$ROOT/rhwp-ios/Sources/RenderTree.swift" \
  "$ROOT/rhwp-ios/Sources/FontFallback.swift" \
  "$ROOT/rhwp-ios/Sources/CGTreeRenderer.swift" \
  "$ROOT/rhwp-macos/scripts/stage3_render_check.swift" \
  "$LIB" \
  -framework CoreGraphics \
  -framework CoreText \
  -framework ImageIO \
  -framework Security \
  -framework CoreFoundation \
  -o "$BIN"

"$BIN" "$OUT_DIR" "${SAMPLES[@]}"
