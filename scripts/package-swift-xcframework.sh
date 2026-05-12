#!/bin/bash
# Package the RHWP native C ABI as an Apple XCFramework for Swift callers.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="$(awk -F '"' '/^version = / { print $2; exit }' "$ROOT/Cargo.toml")"
NATIVE_MANIFEST="$ROOT/bindings/Native/Cargo.toml"
SWIFT_DIR="$ROOT/bindings/swift"
HEADER="$SWIFT_DIR/Sources/CRhwpNative/rhwp_native_ffi.h"
DIST_DIR="$ROOT/dist/swift"
BUILD_DIR="$DIST_DIR/build"
HEADERS_DIR="$BUILD_DIR/Headers"
XCFRAMEWORK_NAME="RhwpNative.xcframework"
XCFRAMEWORK="$BUILD_DIR/$XCFRAMEWORK_NAME"
ARCHIVE_NAME="rhwp-native-v${VERSION}-apple-xcframework.zip"
ARCHIVE_PATH="$DIST_DIR/$ARCHIVE_NAME"

DEVICE_TARGET="aarch64-apple-ios"
SIM_TARGETS=("aarch64-apple-ios-sim")
MACOS_TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: XCFramework packaging requires macOS."
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required."
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "error: rustup is required."
  exit 1
fi

if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "error: xcodebuild is required."
  exit 1
fi

if [[ "${INCLUDE_IOS_SIM_X86_64:-0}" == "1" ]]; then
  SIM_TARGETS+=("x86_64-apple-ios")
fi

echo "=== RHWP Swift XCFramework packaging ==="
echo "version: $VERSION"
echo "output: $ARCHIVE_PATH"

rm -rf "$DIST_DIR"
mkdir -p "$HEADERS_DIR"
cp "$HEADER" "$HEADERS_DIR/"

rustup target add "$DEVICE_TARGET" "${SIM_TARGETS[@]}" "${MACOS_TARGETS[@]}"

build_target() {
  local target="$1"
  echo "[build] $target"
  cargo build \
    --release \
    --manifest-path "$NATIVE_MANIFEST" \
    --target "$target"
}

build_target "$DEVICE_TARGET"
for target in "${SIM_TARGETS[@]}"; do
  build_target "$target"
done
for target in "${MACOS_TARGETS[@]}"; do
  build_target "$target"
done

staticlib_for() {
  local target="$1"
  echo "$ROOT/bindings/Native/target/$target/release/librhwp_native_ffi.a"
}

DEVICE_LIB="$(staticlib_for "$DEVICE_TARGET")"
SIM_LIB="$BUILD_DIR/librhwp_native_ffi-ios-simulator.a"
MACOS_LIB="$BUILD_DIR/librhwp_native_ffi-macos.a"

SIM_LIBS=()
for target in "${SIM_TARGETS[@]}"; do
  SIM_LIBS+=("$(staticlib_for "$target")")
done

MACOS_LIBS=()
for target in "${MACOS_TARGETS[@]}"; do
  MACOS_LIBS+=("$(staticlib_for "$target")")
done

echo "[package] universal simulator static library"
lipo -create "${SIM_LIBS[@]}" -output "$SIM_LIB"

echo "[package] universal macOS static library"
lipo -create "${MACOS_LIBS[@]}" -output "$MACOS_LIB"

rm -rf "$XCFRAMEWORK"
xcodebuild -create-xcframework \
  -library "$DEVICE_LIB" -headers "$HEADERS_DIR" \
  -library "$SIM_LIB" -headers "$HEADERS_DIR" \
  -library "$MACOS_LIB" -headers "$HEADERS_DIR" \
  -output "$XCFRAMEWORK"

cp "$ROOT/LICENSE" "$BUILD_DIR/LICENSE"
cp "$SWIFT_DIR/README.md" "$BUILD_DIR/README.md"

echo "[package] archive"
(
  cd "$BUILD_DIR"
  zip -qry "$ARCHIVE_PATH" "$XCFRAMEWORK_NAME" LICENSE README.md
)

(
  cd "$DIST_DIR"
  shasum -a 256 "$ARCHIVE_NAME" > SHA256SUMS.txt
)

echo "=== done ==="
echo "$ARCHIVE_PATH"
echo "$DIST_DIR/SHA256SUMS.txt"
