#!/usr/bin/env bash
# scripts/svg_regression_diff.sh — Layout 변경 회귀 검증 (Phase 1 #517)
#
# 두 commit 또는 두 디렉토리의 SVG 출력을 페이지별로 byte 비교한다.
# Layout 본질 변경 (Phase 2~4) 시 광범위 회귀 검증에 사용.
#
# 사용법:
#   scripts/svg_regression_diff.sh build <BEFORE_REF> <AFTER_REF> [SAMPLES...]
#     — BEFORE_REF/AFTER_REF (commit/branch) 에서 빌드 → /tmp/svg_diff_{before,after}/ 에 출력 → 비교
#   scripts/svg_regression_diff.sh diff <BEFORE_DIR> <AFTER_DIR>
#     — 이미 존재하는 두 디렉토리만 비교
#
# 기본 SAMPLES (지정 안 하면 사용):
#   exam_kor exam_eng exam_science exam_math synam-001 aift 2010-01-06
#
# 출력:
#   {sample}: total={N} same={M} diff={D}
#   diff 발생 시 페이지 목록 + diff 파일 경로

set -euo pipefail

DEFAULT_SAMPLES=(exam_kor exam_eng exam_science exam_math synam-001 aift 2010-01-06)
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

usage() {
    sed -n '2,/^$/p' "$0" | sed 's/^# //;s/^#//'
    exit 1
}

build_at_ref() {
    local ref="$1"
    local out_dir="$2"
    shift 2
    local samples=("$@")

    pushd "$ROOT" >/dev/null

    # 빌드 (현재 작업 트리 보존)
    if ! git diff --quiet || ! git diff --cached --quiet; then
        echo "  [build] working tree dirty, stashing..." >&2
        git stash push -m "svg_regression_diff_$ref" >&2
        local stashed=1
    else
        local stashed=0
    fi

    git checkout "$ref" -- src/ >&2
    cargo build --release >&2

    # SVG 추출
    mkdir -p "$out_dir"
    for s in "${samples[@]}"; do
        local hwp="samples/${s}.hwp"
        if [ ! -f "$hwp" ]; then
            echo "  [build] skip $s (no $hwp)" >&2
            continue
        fi
        mkdir -p "$out_dir/$s"
        ./target/release/rhwp export-svg "$hwp" -o "$out_dir/$s/" >/dev/null 2>&1 || true
        local n=$(ls "$out_dir/$s/" 2>/dev/null | wc -l | tr -d ' ')
        echo "  [build:$ref] $s: $n pages" >&2
    done

    # 작업 트리 복구
    git checkout HEAD -- src/ >&2
    if [ "$stashed" = "1" ]; then
        git stash pop >&2 || true
    fi

    popd >/dev/null
}

diff_dirs() {
    local before="$1"
    local after="$2"

    local total_pages=0
    local total_same=0
    local total_diff=0

    for d in "$before"/*/; do
        local s=$(basename "$d")
        local b="$before/$s"
        local a="$after/$s"
        [ -d "$a" ] || { echo "$s: SKIP (no $a)"; continue; }

        local total=0 same=0 diff=0
        local diff_pages=()
        for f in "$b"/*.svg; do
            [ -f "$f" ] || continue
            local base=$(basename "$f")
            total=$((total+1))
            if [ -f "$a/$base" ] && cmp -s "$f" "$a/$base"; then
                same=$((same+1))
            else
                diff=$((diff+1))
                diff_pages+=("$base")
            fi
        done

        printf '%s: total=%d same=%d diff=%d' "$s" "$total" "$same" "$diff"
        if [ "$diff" -gt 0 ]; then
            printf '  diff_pages=[%s]' "${diff_pages[*]}"
        fi
        echo

        total_pages=$((total_pages + total))
        total_same=$((total_same + same))
        total_diff=$((total_diff + diff))
    done

    echo "---"
    echo "TOTAL: pages=$total_pages same=$total_same diff=$total_diff"
}

main() {
    [ $# -lt 1 ] && usage

    local cmd="$1"
    shift

    case "$cmd" in
        build)
            [ $# -lt 2 ] && usage
            local before_ref="$1"
            local after_ref="$2"
            shift 2
            local samples=("$@")
            [ ${#samples[@]} -eq 0 ] && samples=("${DEFAULT_SAMPLES[@]}")

            local before_dir="/tmp/svg_diff_before"
            local after_dir="/tmp/svg_diff_after"
            rm -rf "$before_dir" "$after_dir"

            echo "=== Building $before_ref → $before_dir ==="
            build_at_ref "$before_ref" "$before_dir" "${samples[@]}"
            echo
            echo "=== Building $after_ref → $after_dir ==="
            build_at_ref "$after_ref" "$after_dir" "${samples[@]}"
            echo
            echo "=== Comparing ==="
            diff_dirs "$before_dir" "$after_dir"
            ;;
        diff)
            [ $# -lt 2 ] && usage
            diff_dirs "$1" "$2"
            ;;
        *)
            usage
            ;;
    esac
}

main "$@"
