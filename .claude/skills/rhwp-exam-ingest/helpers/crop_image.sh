#!/usr/bin/env bash
# rhwp-exam-ingest helper: bbox 기반 이미지 자르기
#
# 사용법:
#   crop_image.sh <source.png> <x> <y> <w> <h> <out.png>
#
# 좌표는 source.png의 픽셀 단위. (x, y)는 좌상단.

set -euo pipefail

SRC="${1:?source PNG 경로 누락}"
X="${2:?x 좌표 누락}"
Y="${3:?y 좌표 누락}"
W="${4:?폭 누락}"
H="${5:?높이 누락}"
OUT="${6:?출력 경로 누락}"

if [ ! -f "$SRC" ]; then
    echo "오류: source 이미지가 없습니다: $SRC" >&2
    exit 1
fi

mkdir -p "$(dirname "$OUT")"

# magick (ImageMagick 7) 우선, convert (ImageMagick 6) fallback
if command -v magick >/dev/null 2>&1; then
    magick "$SRC" -crop "${W}x${H}+${X}+${Y}" +repage "$OUT"
elif command -v convert >/dev/null 2>&1; then
    convert "$SRC" -crop "${W}x${H}+${X}+${Y}" +repage "$OUT"
else
    echo "오류: ImageMagick (magick 또는 convert)이 필요합니다" >&2
    exit 2
fi

# 결과 검증
if [ -f "$OUT" ]; then
    size=$(wc -c < "$OUT")
    echo "완료: $OUT ($size bytes, ${W}x${H}px from ${X},${Y})"
else
    echo "오류: 자르기 실패 — 출력 파일이 생성되지 않음" >&2
    exit 3
fi
