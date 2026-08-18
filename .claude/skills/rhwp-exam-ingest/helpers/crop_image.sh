#!/usr/bin/env bash
# rhwp-exam-ingest helper: bbox 기반 이미지 자르기
#
# 사용법:
#   crop_image.sh <source.png> <x> <y> <w> <h> <out.png>
#   crop_image.sh --dry-run <source.png> <x> <y> <w> <h> <out.png>
#   crop_image.sh --json --dry-run <source.png> <x> <y> <w> <h> <out.png>
#
# 좌표는 source.png의 픽셀 단위. (x, y)는 좌상단.
# bbox 계약: x>=0, y>=0, w>=1, h>=1, 전부 10진 정수. 소수·음수·빈 값 거부.
#
# 종료 코드:
#   0  성공 (또는 dry-run 검증 통과)
#   1  입력 파일 없음 / 인자 누락
#   2  ImageMagick 없음
#   3  자르기 실패 (출력 미생성)
#   4  bbox 계약 위반

set -euo pipefail

JSON=0
DRY=0
while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1; shift ;;
        --dry-run) DRY=1; shift ;;
        -h|--help)
            echo "사용법: crop_image.sh [--json] [--dry-run] <source.png> <x> <y> <w> <h> <out.png>" >&2
            exit 0
            ;;
        --*) echo "오류: 알 수 없는 플래그 $1" >&2; exit 1 ;;
        *) break ;;
    esac
done

SRC="${1:-}"
X="${2:-}"
Y="${3:-}"
W="${4:-}"
H="${5:-}"
OUT="${6:-}"

emit() {
    local code="$1" msg="$2"
    if [ "$JSON" = "1" ]; then
        printf '{"schemaVersion":"1.0","helper":"crop_image.sh","ok":%s,"code":"%s","message":"%s","src":"%s","bbox":{"x":"%s","y":"%s","w":"%s","h":"%s"},"out":"%s","dryRun":%s}\n' \
            "$([ "$code" = "CROP_OK" ] && echo true || echo false)" \
            "$code" "$msg" "${SRC}" "${X}" "${Y}" "${W}" "${H}" "${OUT}" \
            "$([ "$DRY" = "1" ] && echo true || echo false)"
    else
        echo "$msg" >&2
    fi
}

if [ -z "$SRC" ] || [ -z "$X" ] || [ -z "$Y" ] || [ -z "$W" ] || [ -z "$H" ] || [ -z "$OUT" ]; then
    emit "CROP_ARGS" "사용법: crop_image.sh [--json] [--dry-run] <source.png> <x> <y> <w> <h> <out.png>"
    exit 1
fi

# bbox 계약 — 10진 정수, 음수 금지, 폭/높이 최소 1.
is_uint() { [[ "$1" =~ ^[0-9]+$ ]]; }
if ! is_uint "$X" || ! is_uint "$Y" || ! is_uint "$W" || ! is_uint "$H"; then
    emit "CROP_BBOX_NOT_UINT" "오류: bbox 는 10진 정수여야 합니다 (x=$X y=$Y w=$W h=$H)"
    exit 4
fi
if [ "$W" -lt 1 ] || [ "$H" -lt 1 ]; then
    emit "CROP_BBOX_EMPTY" "오류: bbox 폭/높이는 1 이상이어야 합니다 (w=$W h=$H)"
    exit 4
fi

if [ ! -f "$SRC" ]; then
    emit "CROP_SRC_MISSING" "오류: source 이미지가 없습니다: $SRC"
    exit 1
fi

MAGICK_BIN=""
if command -v magick >/dev/null 2>&1; then
    MAGICK_BIN="magick"
elif command -v convert >/dev/null 2>&1; then
    MAGICK_BIN="convert"
else
    emit "CROP_MISS_IMAGEMAGICK" "오류: ImageMagick (magick 또는 convert)이 필요합니다"
    exit 2
fi

PLANNED="${MAGICK_BIN} ${SRC} -crop ${W}x${H}+${X}+${Y} +repage ${OUT}"

if [ "$DRY" = "1" ]; then
    if [ "$JSON" = "1" ]; then
        printf '{"schemaVersion":"1.0","helper":"crop_image.sh","ok":true,"code":"CROP_OK","dryRun":true,"planned":"%s","src":"%s","bbox":{"x":%s,"y":%s,"w":%s,"h":%s},"out":"%s","engine":"%s"}\n' \
            "$PLANNED" "$SRC" "$X" "$Y" "$W" "$H" "$OUT" "$MAGICK_BIN"
    else
        echo "dry-run: $PLANNED"
    fi
    exit 0
fi

mkdir -p "$(dirname "$OUT")"

# magick (ImageMagick 7) 우선, convert (ImageMagick 6) fallback
if [ "$MAGICK_BIN" = "magick" ]; then
    magick "$SRC" -crop "${W}x${H}+${X}+${Y}" +repage "$OUT"
else
    convert "$SRC" -crop "${W}x${H}+${X}+${Y}" +repage "$OUT"
fi

# 결과 검증
if [ -f "$OUT" ]; then
    size=$(wc -c < "$OUT")
    if [ "$JSON" = "1" ]; then
        printf '{"schemaVersion":"1.0","helper":"crop_image.sh","ok":true,"code":"CROP_OK","out":"%s","bytes":%s,"bbox":{"x":%s,"y":%s,"w":%s,"h":%s}}\n' \
            "$OUT" "$(echo "$size" | tr -d ' ')" "$X" "$Y" "$W" "$H"
    else
        echo "완료: $OUT ($size bytes, ${W}x${H}px from ${X},${Y})"
    fi
else
    emit "CROP_NO_OUTPUT" "오류: 자르기 실패 — 출력 파일이 생성되지 않음"
    exit 3
fi
