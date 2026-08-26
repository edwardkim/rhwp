#!/usr/bin/env bash
# rhwp-exam-ingest helper: PDF → 페이지별 PNG
#
# 사용법:
#   pdf_to_pngs.sh <input.pdf> <out_dir> [<dpi>]
#   pdf_to_pngs.sh --dry-run <input.pdf> <out_dir> [<dpi>]
#   pdf_to_pngs.sh --json --dry-run <input.pdf> <out_dir> [<dpi>]
#
# 출력: <out_dir>/page_001.png, page_002.png, ...
# DPI 기본 300. 허용 범위 72–600.
#
# 종료 코드:
#   0  성공 (또는 dry-run 검증 통과)
#   1  입력 파일 없음 / 인자 누락
#   2  pdftoppm / magick / convert 모두 없음
#   4  DPI 계약 위반

set -euo pipefail

JSON=0
DRY=0
while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1; shift ;;
        --dry-run) DRY=1; shift ;;
        -h|--help)
            echo "사용법: pdf_to_pngs.sh [--json] [--dry-run] <input.pdf> <out_dir> [<dpi>]" >&2
            exit 0
            ;;
        --*) echo "오류: 알 수 없는 플래그 $1" >&2; exit 1 ;;
        *) break ;;
    esac
done

INPUT="${1:-}"
OUTDIR="${2:-}"
DPI="${3:-300}"

emit() {
    local code="$1" msg="$2"
    if [ "$JSON" = "1" ]; then
        printf '{"schemaVersion":"1.0","helper":"pdf_to_pngs.sh","ok":false,"code":"%s","message":"%s","input":"%s","outDir":"%s","dpi":"%s","dryRun":%s}\n' \
            "$code" "$msg" "${INPUT}" "${OUTDIR}" "${DPI}" \
            "$([ "$DRY" = "1" ] && echo true || echo false)"
    else
        echo "$msg" >&2
    fi
}

if [ -z "$INPUT" ] || [ -z "$OUTDIR" ]; then
    emit "PDF_ARGS" "사용법: pdf_to_pngs.sh [--json] [--dry-run] <input.pdf> <out_dir> [<dpi>]"
    exit 1
fi

if ! [[ "$DPI" =~ ^[0-9]+$ ]] || [ "$DPI" -lt 72 ] || [ "$DPI" -gt 600 ]; then
    emit "PDF_DPI_RANGE" "오류: DPI 는 72–600 정수여야 합니다 (got $DPI)"
    exit 4
fi

if [ ! -f "$INPUT" ]; then
    emit "PDF_SRC_MISSING" "오류: 입력 PDF가 존재하지 않습니다: $INPUT"
    exit 1
fi

ENGINE=""
PLANNED=""
if command -v pdftoppm >/dev/null 2>&1; then
    ENGINE="pdftoppm"
    PLANNED="pdftoppm -r ${DPI} -png ${INPUT} ${OUTDIR}/page -f 1"
elif command -v magick >/dev/null 2>&1; then
    ENGINE="magick"
    PLANNED="magick -density ${DPI} ${INPUT} ${OUTDIR}/page_%03d.png"
elif command -v convert >/dev/null 2>&1; then
    ENGINE="convert"
    PLANNED="convert -density ${DPI} ${INPUT} ${OUTDIR}/page_%03d.png"
else
    emit "PDF_MISS_TOOLS" "오류: pdftoppm / magick / convert 중 하나가 필요합니다 (poppler-utils 또는 ImageMagick)"
    exit 2
fi

if [ "$DRY" = "1" ]; then
    if [ "$JSON" = "1" ]; then
        printf '{"schemaVersion":"1.0","helper":"pdf_to_pngs.sh","ok":true,"code":"PDF_OK","dryRun":true,"engine":"%s","planned":"%s","input":"%s","outDir":"%s","dpi":%s,"pagePattern":"page_%%03d.png"}\n' \
            "$ENGINE" "$PLANNED" "$INPUT" "$OUTDIR" "$DPI"
    else
        echo "dry-run: $PLANNED"
        echo "페이지 이름: ${OUTDIR}/page_NNN.png"
    fi
    exit 0
fi

mkdir -p "$OUTDIR"

# pdftoppm 우선 (poppler-utils, 가벼움), 없으면 magick fallback
if [ "$ENGINE" = "pdftoppm" ]; then
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
elif [ "$ENGINE" = "magick" ]; then
    magick -density "$DPI" "$INPUT" "$OUTDIR/page_%03d.png"
else
    convert -density "$DPI" "$INPUT" "$OUTDIR/page_%03d.png"
fi

# 텍스트 layer가 있는 PDF면 텍스트도 추출 (보조 자료)
TEXT_LAYER=false
if command -v pdftotext >/dev/null 2>&1; then
    pdftotext -layout "$INPUT" "$OUTDIR/text.txt" 2>/dev/null || true
    [ -f "$OUTDIR/text.txt" ] && TEXT_LAYER=true
fi

count=$(find "$OUTDIR" -name "page_*.png" | wc -l | tr -d ' ')
if [ "$JSON" = "1" ]; then
    printf '{"schemaVersion":"1.0","helper":"pdf_to_pngs.sh","ok":true,"code":"PDF_OK","pages":%s,"outDir":"%s","engine":"%s","dpi":%s,"textLayer":%s}\n' \
        "$count" "$OUTDIR" "$ENGINE" "$DPI" "$TEXT_LAYER"
else
    echo "완료: $count 페이지 → $OUTDIR/page_NNN.png"
    [ -f "$OUTDIR/text.txt" ] && echo "텍스트 layer: $OUTDIR/text.txt"
fi
