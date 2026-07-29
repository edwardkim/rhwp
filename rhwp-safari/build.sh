#!/bin/bash
# rhwp-safari 빌드 스크립트
# 1. rhwp-chrome의 Vite 빌드 결과(뷰어)를 기반으로 Safari 전용 dist 생성
# 2. Safari 전용 소스(src/)로 background, content-script, manifest 교체
# 3. safari-web-extension-converter로 Xcode 프로젝트 재생성
# 4. xcodebuild로 macOS 빌드

set -e -o pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST="$SCRIPT_DIR/dist"
SRC="$SCRIPT_DIR/src"
CHROME_DIST="$ROOT/rhwp-chrome/dist"

echo "=== rhwp-safari 빌드 시작 ==="

# 1. Chrome 확장 빌드 (뷰어 + 리소스)
echo "[1/5] Chrome 확장 빌드..."
cd "$ROOT/rhwp-chrome" && npm run build

# 2. Chrome dist를 Safari dist로 복사
echo "[2/5] Safari dist 생성..."
rm -rf "$DIST"
cp -R "$CHROME_DIST" "$DIST"

# 3. Safari 전용 소스 문법 검사 + 복사
echo "[3/6] JS 문법 검사..."
for jsfile in "$ROOT"/rhwp-shared/security/file-signature.js "$SRC"/background.js "$SRC"/content-script.js "$SRC"/options.js; do
  if ! node --check "$jsfile" 2>&1; then
    echo "오류: $jsfile 문법 오류 발견. 빌드 중단."
    exit 1
  fi
  echo "  ✓ $(basename "$jsfile")"
done

echo "[4/6] Safari 전용 소스 적용..."
cp "$SRC/background.js" "$DIST/background.js"
cp "$ROOT/rhwp-shared/security/file-signature.js" "$DIST/file-signature.js"
cp "$SRC/content-script.js" "$DIST/content-script.js"
cp "$SRC/manifest.json" "$DIST/manifest.json"
cp "$SRC/options.html" "$DIST/options.html"
cp "$SRC/options.js" "$DIST/options.js"
# Chrome 전용 파일 제거
rm -rf "$DIST/sw"
rm -f "$DIST/dev-tools-inject.js"
# viewer.html에서 dev-tools-inject.js 참조 제거
sed -i '' '/<script src="\/dev-tools-inject.js"><\/script>/d' "$DIST/viewer.html"
# Safari 호환: dev-tools-inject.js 참조만 제거
# viewer.html의 type="module", crossorigin, 절대 경로는 원본 유지

# 5. Xcode 프로젝트 (최초 생성 시에만, 서명 설정 보존)
if [ ! -d "$SCRIPT_DIR/HWP Viewer/HWP Viewer.xcodeproj" ]; then
  echo "[5/6] Xcode 프로젝트 생성 (최초)..."
  xcrun safari-web-extension-converter "$DIST" \
    --project-location "$SCRIPT_DIR" \
    --app-name "HWP Viewer" \
    --bundle-identifier com.edwardkim.rhwp-safari \
    --no-open --no-prompt
else
  echo "[5/6] Xcode 프로젝트 존재 — 서명 설정 보존, 건너뜀"
fi

# 6. macOS 빌드
echo "[6/6] macOS 빌드..."
cd "$SCRIPT_DIR/HWP Viewer"
xcodebuild -scheme "HWP Viewer (macOS)" -configuration Debug build | tail -3

echo ""
echo "=== 빌드 완료 ==="
echo "앱 실행: open \"\$(find ~/dev/xbuild -name 'HWP Viewer.app' -path '*/Debug/*' | head -1)\""
