#!/usr/bin/env bash
# rhwp-exam-ingest helper: PDF → 페이지별 PNG
#
# 사용법:
#   pdf_to_pngs.sh <input.pdf> <out_dir> [<dpi>]
#
# 출력: <out_dir>/page_001.png, page_002.png, ...

set -euo pipefail

INPUT="${1:?입력 PDF 경로 누락}"
OUTDIR="${2:?출력 디렉토리 누락}"
DPI="${3:-300}"

if [ ! -f "$INPUT" ]; then
    echo "오류: 입력 PDF가 존재하지 않습니다: $INPUT" >&2
    exit 1
fi

mkdir -p "$OUTDIR"

# pdftoppm 우선 (poppler-utils, 가벼움), 없으면 magick fallback
if command -v pdftoppm >/dev/null 2>&1; then
    pdftoppm -r "$DPI" -png "$INPUT" "$OUTDIR/page" -f 1
    # pdftoppm은 page-1.png, page-2.png 형식 — page_001 형식으로 rename
    cd "$OUTDIR"
    for f in page-*.png; do
        if [ -f "$f" ]; then
            n=$(echo "$f" | sed 's/page-\([0-9]*\)\.png/\1/')
            # 10진수 강제 — "08", "09" 등 leading-zero 입력이 8진수로 해석되는 것 방지
            printf -v new "page_%03d.png" "$((10#$n))"
            mv "$f" "$new"
        fi
    done
    cd - >/dev/null
elif command -v magick >/dev/null 2>&1; then
    magick -density "$DPI" "$INPUT" "$OUTDIR/page_%03d.png"
elif command -v convert >/dev/null 2>&1; then
    convert -density "$DPI" "$INPUT" "$OUTDIR/page_%03d.png"
else
    echo "오류: pdftoppm / magick / convert 중 하나가 필요합니다" >&2
    exit 2
fi

# 텍스트 layer가 있는 PDF면 텍스트도 추출 (보조 자료)
if command -v pdftotext >/dev/null 2>&1; then
    pdftotext -layout "$INPUT" "$OUTDIR/text.txt" 2>/dev/null || true
fi

count=$(find "$OUTDIR" -name "page_*.png" | wc -l | tr -d ' ')
echo "완료: $count 페이지 → $OUTDIR/page_NNN.png"
[ -f "$OUTDIR/text.txt" ] && echo "텍스트 layer: $OUTDIR/text.txt"
