#!/usr/bin/env bash
# rhwp autofix bot — 오라클 기반 탐지기 (크래시가 없어도 결함을 찾는다)
#
# 핵심 발상: 판정 근거를 모델의 의견이 아니라 '저장소가 스스로 정의한 계약'에 둔다.
# rhwp CLI 는 이미 계약 위반을 종료코드로 알려준다:
#
#   ir-diff  --json                  차이 발견 시 exit 3
#   convert  --verify                저장 후 재파싱 IR 차이 시 exit 3
#   convert  --verify-pages          저장 전/후 페이지 수 불일치 시 exit 4
#   export-hwpx --verify[-pages]     위와 동일
#
# 이 종료코드가 곧 '버그의 정의'다. 모델이 개입할 여지가 없으므로 오탐이 0이다.
#
#   B. 계약 위반 스윕 — 조용히 틀린 결과 (크래시 없는 결함의 본체)
#   C. 오버플로 스윕 — release 에서 꺼져 있는 검사를 켜고 전수 파싱
#   A. 크래시 퍼징   — 패닉/UB
#   D. 전부 비면 AI 탐색으로 위임
set -uo pipefail

ROOT="${1:?repo root 인자 필요}"
ROTATION="${2:-0}"          # github.run_number — 매 사이클 다른 표본을 본다
cd "$ROOT"

PROFILE="release-test"
BIN="./target/release-test/rhwp"
OUT="$ROOT/FINDING.md"
WORK=$(mktemp -d)
rm -f "$OUT"
STEP() { echo "::group::[detect] $*"; }
END()  { echo "::endgroup::"; }

emit() {  # emit <kind> <title> <evidence-file>
  {
    echo "---"
    echo "kind: $1"
    echo "title: $2"
    echo "---"
    echo
    echo "## 기계적 판정 근거"
    echo
    echo "저장소 CLI 가 계약 위반을 종료코드로 보고했다. 모델 판단이 개입하지 않았다."
    echo
    echo '```'
    head -60 "$3"
    echo '```'
  } > "$OUT"
  echo "발견: [$1] $2"
  echo "DETECT_KIND=$1"
  exit 0
}

rotate() { # rotate <목록파일> <개수> — 매 사이클 다른 구간을 보게 한다
  local total; total=$(wc -l < "$1" 2>/dev/null || echo 0)
  [ "$total" -eq 0 ] && return
  awk -v s=$(( (ROTATION * $2) % total )) -v n="$2" 'NR>s && NR<=s+n' "$1"
}

echo "회전 인덱스: $ROTATION"
cargo build --profile "$PROFILE" --bin rhwp 2>&1 | tail -3
[ -x "$BIN" ] || { echo "빌드 실패 — 탐지 중단"; echo "DETECT_KIND=NONE"; exit 0; }

# ─────────────────────────────────────────────────────────────
# B1. 포맷 간 IR 불일치 — `ir-diff` exit 3
#     CLAUDE.md 가 "모든 포맷 파서는 공통 Document IR 을 반환한다" 고 선언한다.
#     같은 문서의 HWP 판과 HWPX 판이 다른 IR 을 내면 그 선언이 깨진 것이다.
# ─────────────────────────────────────────────────────────────
STEP "B1. 포맷 간 IR 일치 (ir-diff, exit 3 = 불일치)"
for f in samples/*.hwpx; do
  b="samples/$(basename "$f" .hwpx)"
  [ -f "$b.hwp" ] && echo "$f $b.hwp"
done > "$WORK/pairs.txt"
echo "대응 쌍 $(wc -l < "$WORK/pairs.txt") 개 — 이번 사이클 12개 검사"

while read -r hwpx hwp; do
  [ -z "${hwpx:-}" ] && continue
  L="$WORK/irdiff.log"
  "$BIN" ir-diff "$hwpx" "$hwp" --json >"$L" 2>&1
  if [ $? -eq 3 ]; then
    echo "샘플 쌍: $hwpx ↔ $hwp" >> "$L"
    emit "ir-mismatch" "HWP/HWPX 파서가 같은 문서에서 서로 다른 IR 을 만든다 ($(basename "$hwpx"))" "$L"
  fi
done < <(rotate "$WORK/pairs.txt" 12)
echo "IR 불일치 없음"
END

# ─────────────────────────────────────────────────────────────
# B2. 라운드트립 계약 — `convert --verify --verify-pages` exit 3 / exit 4
#     저장했다 다시 읽으면 같아야 한다. 다르면 직렬화 손실이다.
# ─────────────────────────────────────────────────────────────
STEP "B2. 라운드트립 무손실 (convert --verify, exit 3=IR차이 / 4=페이지수)"
ls samples/*.hwp > "$WORK/hwps.txt" 2>/dev/null || true
while read -r s; do
  [ -z "${s:-}" ] && continue
  L="$WORK/conv.log"
  "$BIN" convert "$s" "$WORK/rt.hwp" --verify --verify-pages >"$L" 2>&1
  RC=$?
  echo "샘플: $s (exit $RC)" >> "$L"
  case "$RC" in
    3) emit "roundtrip-ir-loss"    "저장 후 재파싱하면 IR 이 달라진다 — 직렬화 손실 ($(basename "$s"))" "$L" ;;
    4) emit "roundtrip-page-shift" "저장 전후로 렌더 페이지 수가 달라진다 ($(basename "$s"))" "$L" ;;
  esac
done < <(rotate "$WORK/hwps.txt" 10)
echo "라운드트립 손실 없음"
END

# ─────────────────────────────────────────────────────────────
# B3. HWPX 경유 라운드트립 — 다른 직렬화 경로의 같은 계약
# ─────────────────────────────────────────────────────────────
STEP "B3. HWPX 라운드트립 (export-hwpx --verify)"
while read -r s; do
  [ -z "${s:-}" ] && continue
  L="$WORK/exp.log"
  "$BIN" export-hwpx "$s" "$WORK/rt.hwpx" --verify --verify-pages >"$L" 2>&1
  RC=$?
  echo "샘플: $s (exit $RC)" >> "$L"
  case "$RC" in
    3) emit "hwpx-roundtrip-ir-loss"    "HWPX 로 내보냈다 읽으면 IR 이 달라진다 ($(basename "$s"))" "$L" ;;
    4) emit "hwpx-roundtrip-page-shift" "HWPX 내보내기 전후로 페이지 수가 달라진다 ($(basename "$s"))" "$L" ;;
  esac
done < <(rotate "$WORK/hwps.txt" 8)
echo "HWPX 라운드트립 손실 없음"
END

# ─────────────────────────────────────────────────────────────
# B4. 렌더 결정성 — 같은 입력을 두 번 렌더하면 바이트까지 같아야 한다.
#     다르면 해시맵 순회 순서 등 비결정성이 출력에 새는 것이다.
# ─────────────────────────────────────────────────────────────
STEP "B4. 렌더 결정성 (export-svg 2회 바이트 비교)"
while read -r s; do
  [ -z "${s:-}" ] && continue
  "$BIN" export-svg "$s" -o "$WORK/a.svg" >/dev/null 2>&1 || continue
  "$BIN" export-svg "$s" -o "$WORK/b.svg" >/dev/null 2>&1 || continue
  if ! cmp -s "$WORK/a.svg" "$WORK/b.svg"; then
    L="$WORK/det.log"
    { echo "샘플: $s"
      echo "1회차와 2회차 export-svg 결과가 바이트 단위로 다르다 (렌더가 비결정적)."
      diff <(head -c 4000 "$WORK/a.svg") <(head -c 4000 "$WORK/b.svg") | head -30
    } > "$L"
    emit "nondeterministic-render" "같은 입력을 두 번 렌더하면 결과가 달라진다 ($(basename "$s"))" "$L"
  fi
done < <(rotate "$WORK/hwps.txt" 8)
echo "렌더 비결정성 없음"
END

# ─────────────────────────────────────────────────────────────
# C. 정수 오버플로 스윕
#     release-test 는 release 를 상속하므로 overflow-checks 가 꺼져 있다.
#     즉 현재 CI 는 오버플로를 조용히 감싸고 통과시킨다 — 원리적으로 볼 수 없는 결함이다.
#     검사를 켜고 전수 파싱하면 그 사각지대가 드러난다.
# ─────────────────────────────────────────────────────────────
STEP "C. 정수 오버플로 스윕 (overflow-checks=on — CI 사각지대)"
export CARGO_PROFILE_RELEASE_TEST_OVERFLOW_CHECKS=true
export CARGO_PROFILE_RELEASE_TEST_DEBUG_ASSERTIONS=true
cargo build --profile "$PROFILE" --bin rhwp 2>&1 | tail -3
ls samples/*.hwp samples/*.hwpx > "$WORK/all.txt" 2>/dev/null || true
while read -r s; do
  [ -z "${s:-}" ] && continue
  L="$WORK/ovf.log"
  "$BIN" export-svg "$s" -o /dev/null >"$L" 2>&1
  if grep -qE 'attempt to (add|subtract|multiply|negate|shift).*overflow|index out of bounds|attempt to divide by zero' "$L"; then
    echo "샘플: $s" >> "$L"
    emit "arith-overflow" "오버플로 검사를 켜면 파싱/렌더 중 산술 오버플로로 패닉한다 ($(basename "$s"))" "$L"
  fi
done < <(rotate "$WORK/all.txt" 20)
unset CARGO_PROFILE_RELEASE_TEST_OVERFLOW_CHECKS CARGO_PROFILE_RELEASE_TEST_DEBUG_ASSERTIONS
echo "산술 오버플로 없음"
END

# ─────────────────────────────────────────────────────────────
# A. 크래시 퍼징 (nightly 가 준비된 경우에만)
# ─────────────────────────────────────────────────────────────
STEP "A. 크래시 퍼징"
if command -v cargo-fuzz >/dev/null 2>&1 && rustup toolchain list 2>/dev/null | grep -q nightly; then
  TARGETS=(parse_hwp parse_hwpx parse_hwp3 parse_hml parse_wmf parse_ooxml_chart)
  T="${TARGETS[$(( ROTATION % ${#TARGETS[@]} ))]}"
  echo "표적: $T"
  L="$WORK/fuzz.log"
  if ! cargo +nightly fuzz run "$T" -- -max_total_time=180 -timeout=10 >"$L" 2>&1; then
    grep -qE 'panicked at|ERROR: libFuzzer|SUMMARY:' "$L" \
      && emit "fuzz-crash" "퍼징 중 ${T} 에서 패닉이 발생한다" "$L"
  fi
  echo "크래시 없음"
else
  echo "nightly/cargo-fuzz 미설치 — 이 층 건너뜀"
fi
END

echo "오라클 층 전부 통과 — 기계적으로 잡히는 결함 없음. AI 탐색으로 위임한다."
echo "DETECT_KIND=NONE"
