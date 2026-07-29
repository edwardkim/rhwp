#!/usr/bin/env bash
# 대량 PR 분류 보조 — 통합 처리 전 판정 근거만 모은다. 판정은 사람이 한다.
#
# 한 기여자가 5일간 300건 가까이 제출해 개별 조회로는 감당이 어려워졌다.
# 복잡한 단일 jq 는 조용히 행을 탈락시켜(실측: 111 → 75건) 신뢰할 수 없었다.
# 여기서는 **필드별로 따로 조회해 셸에서 합친다.** 느리지만 누락이 없다.
#
# 사용:
#   scripts/pr_triage.sh <author>          # 축별 분포 + 충돌 목록
#   scripts/pr_triage.sh <author> --list   # PR 별 한 줄 표
set -uo pipefail

REPO="${RHWP_REPO:-edwardkim/rhwp}"
AUTHOR="${1:-}"
[ -z "$AUTHOR" ] && { echo "usage: $0 <author> [--list]" >&2; exit 2; }
MODE="${2:-}"

# gh 기본 limit 은 30 이다. 명시하지 않으면 대량 작업이 조용히 잘린다.
LIM="${RHWP_PR_LIMIT:-500}"
Q=(gh pr list --repo "$REPO" --author "$AUTHOR" --state open --limit "$LIM")

axis_of() {
  case "$1" in
    src/serializer/hwpx/*|src/parser/hwpx/*)   echo hwpx ;;
    src/parser/hwp3/*)                         echo hwp3 ;;
    src/document_core/*)                       echo doccore ;;
    src/model/*)                               echo model ;;
    src/wasm_api*)                             echo wasm ;;
    src/renderer/*)                            echo render ;;
    src/serializer/*|src/parser/*)             echo core ;;
    rhwp-studio/*)                             echo studio ;;
    npm/*)                                     echo npm ;;
    rhwp-chrome/*|rhwp-firefox/*|rhwp-vscode/*) echo ext ;;
    tests/*)                                   echo test ;;
    *)                                         echo misc ;;
  esac
}

total=$("${Q[@]}" --json number -q 'length')
echo "열린 PR: ${total}건  (author=${AUTHOR})"
echo

echo "== 병합 가능 여부 =="
"${Q[@]}" --json mergeable -q '.[].mergeable' | sort | uniq -c | sed 's/^/  /'
echo

echo "== 리베이스 필요 (CONFLICTING) =="
conflicts=$("${Q[@]}" --json number,mergeable -q '.[] | select(.mergeable=="CONFLICTING") | .number')
if [ -n "$conflicts" ]; then
  echo "$conflicts" | tr '\n' ' ' | fold -sw 88 | sed 's/^/  /'
else
  echo "  없음"
fi
echo

# 축은 코드 파일로 판정한다. files[0] 은 알파벳 순이라 mydocs/ 가 먼저 잡힌다.
echo "== 축별 분포 (통합 그룹 후보) =="
tmp=$(mktemp); trap 'rm -f "$tmp"' EXIT
"${Q[@]}" --json number,files \
  -q '.[] | .number as $n | (.files[].path | select(test("^mydocs/|\\.md$") | not)) as $p | "\($n)\t\($p)"' \
  2>/dev/null | while IFS=$'\t' read -r n p; do
    printf '%s\t%s\n' "$n" "$(axis_of "$p")"
  done | sort -u > "$tmp"
# 한 PR 이 여러 축에 걸치면 첫 축을 대표로 삼는다.
sort -k1,1 -u "$tmp" | cut -f2 | sort | uniq -c | sort -rn | sed 's/^/  /'
echo

if [ "$MODE" = "--list" ]; then
  echo "== PR 목록 =="
  printf '  %-6s %-9s %-8s %s\n' PR M AXIS TITLE
  "${Q[@]}" --json number,mergeable,title -q '.[] | "\(.number)\t\(.mergeable)\t\(.title)"' \
  | while IFS=$'\t' read -r n m t; do
      a=$(awk -F'\t' -v k="$n" '$1==k{print $2; exit}' "$tmp"); a="${a:-docs}"
      case "$m" in MERGEABLE) m=OK ;; CONFLICTING) m=CONFLICT ;; *) m='?' ;; esac
      printf '  %-6s %-9s %-8s %s\n' "$n" "$m" "$a" "${t:0:56}"
    done
fi
