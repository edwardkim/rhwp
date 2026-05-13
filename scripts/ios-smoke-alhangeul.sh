#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_PATH="$ROOT_DIR/rhwp-ios/AlHangeul.xcodeproj"
DERIVED_DATA="$ROOT_DIR/rhwp-ios/DerivedData"
APP_PATH="$DERIVED_DATA/Build/Products/Debug-iphonesimulator/알한글.app"
BUNDLE_ID="org.rhwp.alhangeul"
SCREENSHOT_PATH="$ROOT_DIR/output/ios-smoke/alhangeul-sample.png"

find_simulator_id() {
  if [[ -n "${RHWP_IOS_SIMULATOR_ID:-}" ]]; then
    printf '%s\n' "$RHWP_IOS_SIMULATOR_ID"
    return
  fi

  local booted
  booted="$(xcrun simctl list devices available \
    | sed -nE 's/^[[:space:]]*iPhone 17 \(([0-9A-F-]+)\) \(Booted\)[[:space:]]*$/\1/p' \
    | head -n 1)"
  if [[ -n "$booted" ]]; then
    printf '%s\n' "$booted"
    return
  fi

  xcrun simctl list devices available \
    | sed -nE 's/^[[:space:]]*iPhone 17 \(([0-9A-F-]+)\).*$/\1/p' \
    | head -n 1
}

SIMULATOR_ID="$(find_simulator_id)"
if [[ -z "$SIMULATOR_ID" ]]; then
  echo "ERROR: no available iPhone 17 simulator found. Set RHWP_IOS_SIMULATOR_ID." >&2
  exit 1
fi

cd "$ROOT_DIR"

echo "==> Building Rust static library for iOS simulator"
cargo build --target aarch64-apple-ios-sim --release

echo "==> Building AlHangeul for simulator $SIMULATOR_ID"
xcodebuild \
  -project "$PROJECT_PATH" \
  -scheme AlHangeul \
  -configuration Debug \
  -derivedDataPath "$DERIVED_DATA" \
  -destination "id=$SIMULATOR_ID" \
  build

if [[ ! -f "$APP_PATH/sample.hwpx" ]]; then
  echo "ERROR: sample.hwpx was not copied into $APP_PATH" >&2
  exit 1
fi

echo "==> Booting, installing, and launching AlHangeul"
xcrun simctl boot "$SIMULATOR_ID" >/dev/null 2>&1 || true
xcrun simctl install "$SIMULATOR_ID" "$APP_PATH"
xcrun simctl launch "$SIMULATOR_ID" "$BUNDLE_ID"

mkdir -p "$(dirname "$SCREENSHOT_PATH")"
sleep "${RHWP_IOS_SMOKE_SCREENSHOT_DELAY:-2}"
xcrun simctl io "$SIMULATOR_ID" screenshot "$SCREENSHOT_PATH" >/dev/null

echo "OK: AlHangeul launched with bundled sample.hwpx"
echo "Screenshot: $SCREENSHOT_PATH"
