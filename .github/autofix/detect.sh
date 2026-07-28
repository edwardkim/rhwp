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

# 이 시그니처가 이미 보고/탈락 목록에 있는가
sig_reported() {
  [ -n "${DETECT_EXCLUDE_FILE:-}" ] && [ -s "${DETECT_EXCLUDE_FILE:-}" ] \
    && grep -qF -- "sig:$1" "$DETECT_EXCLUDE_FILE"
}

# 근본 원인 시그니처 — 한 증거에 여러 원인이 섞여 있으면 원인마다 후보를 만들고,
# 그중 '아직 보고되지 않은 첫 번째'를 고른다. 그래서 한 문서에서 필드 세 개가
# 깨져 있어도 사이클마다 원인 하나씩만 나가고, 필드 조합이 다르다는 이유로
# 같은 버그가 여러 번 올라가는 일이 없다. 선택된 원인의 라벨은 SIG_LABEL 로 돌려준다.
# 결과는 SIG_OUT/SIG_LABEL 전역에 담는다 — 명령치환으로 부르면 서브셸이라
# 라벨이 호출부로 돌아오지 못한다.
signature_of() { # signature_of <kind> <evidence-file>  → SIG_OUT, SIG_LABEL 설정
  local K="$1" EV="$2" c h LAST="" LAST_L=""
  local CANDS=() LABELS=()
  # 탐지 '경로'가 아니라 '원인'으로 묶는다. 같은 char_shapes 손실을 HWP 저장·
  # HWPX 내보내기·포맷 비교 세 경로가 각각 잡아내는데, 그건 한 버그다.
  # 경로별로 이슈를 내면 같은 걸 세 번 올리게 된다.
  local FAM="$K"
  case "$K" in
    *ir-loss|ir-mismatch) FAM="ir-loss" ;;
    *page-shift)          FAM="page-shift" ;;
  esac
  case "$K" in
    *ir-loss|ir-mismatch)
      # 어긋난 IR 필드 하나하나가 서로 다른 직렬화 결함이다 (camelCase 포함)
      while read -r f; do
        [ -n "$f" ] && { CANDS+=("field=$f"); LABELS+=("$f"); }
      done < <(grep -oE '[A-Za-z_][A-Za-z0-9_]*: expected=' "$EV" 2>/dev/null \
                 | sed 's/: expected=//' | sort -u)
      # 직렬화 실패 메시지는 그 자체로 독립된 원인
      while read -r r; do
        [ -n "$r" ] && { CANDS+=("ser=$r"); LABELS+=("직렬화 실패"); }
      done < <(grep -oE '직렬화 실패: [^(]{0,40}' "$EV" 2>/dev/null | sort -u | head -3)
      ;;
    arith-overflow|fuzz-crash)
      # 패닉 위치가 곧 버그 위치 — 위치가 다르면 다른 버그다
      c=$(grep -oE "[a-zA-Z0-9_/.-]+\.rs:[0-9]+" "$EV" 2>/dev/null | head -1)
      [ -n "$c" ] && { CANDS+=("at=$c"); LABELS+=("$c"); }
      ;;
    nondeterministic-render)
      # 어떤 SVG 요소가 흔들리는지가 비결정성의 출처를 가른다
      c=$(grep -oE '<[a-zA-Z:]+' "$EV" 2>/dev/null | sort -u | head -4 | tr -d '\n')
      [ -n "$c" ] && { CANDS+=("elem=$c"); LABELS+=("렌더 비결정성"); }
      ;;
  esac
  if [ "${#CANDS[@]}" -eq 0 ]; then
    CANDS=("kind=$K"); LABELS=("")
  fi
  local i=0
  for c in "${CANDS[@]}"; do
    h=$(printf '%s' "${FAM}:${c}" | sha1sum | cut -c1-12)
    LAST="$h"; LAST_L="${LABELS[$i]}"
    if ! sig_reported "$h"; then
      SIG_LABEL="${LABELS[$i]}"; SIG_OUT="$h"; return 0
    fi
    i=$(( i + 1 ))
  done
  SIG_LABEL="$LAST_L"; SIG_OUT="$LAST"   # 후보가 전부 보고됨 — 호출부가 걸러낸다
}

emit() {  # emit <kind> <title> <evidence-file>
  local KIND="$1" TITLE="$2" EV="$3"
  SIG_LABEL=""; SIG_OUT=""
  signature_of "$KIND" "$EV"
  local SIG="$SIG_OUT"

  # 제외 판정 — 스윕 모드는 시그니처(근본 원인)로, 단발 모드는 제목 계열로 본다.
  # 단발(autofix 루프)은 '수정 실패한 계열'을 통째로 넘겨 다음 층으로 전진해야 하고,
  # 스윕은 원인이 다르면 계속 잡아내야 하므로 기준이 다르다.
  if [ -n "${DETECT_EXCLUDE_FILE:-}" ] && [ -s "${DETECT_EXCLUDE_FILE:-}" ]; then
    if [ -n "${SWEEP_OUT:-}" ]; then
      if sig_reported "$SIG"; then
        echo "  건너뜀(이미 보고된 원인): $TITLE"
        return 1
      fi
    else
      local T_BASE="${TITLE% (*}"
      if grep -qF -- "$TITLE" "$DETECT_EXCLUDE_FILE" || grep -qF -- "$T_BASE" "$DETECT_EXCLUDE_FILE"; then
        echo "제외된 발견 건너뜀(이미 보고/탈락한 계열): $TITLE"
        return 1
      fi
    fi
  fi

  # 제목에 원인을 드러낸다 — 메인테이너가 제목만 보고 무엇이 깨졌는지 알 수 있고,
  # 중복 여부도 눈으로 판별된다.
  if [ -n "${SIG_LABEL:-}" ]; then
    local HEAD="${TITLE%% (*}" TAIL=""
    case "$TITLE" in *\ \(*) TAIL=" (${TITLE#*(}" ;; esac
    TITLE="${HEAD} — ${SIG_LABEL}${TAIL}"
  fi

  # 스윕 모드는 한 건에서 멈추지 않는다 — 시그니처별로 모아 두고 계속 훑는다.
  local DEST="$OUT"
  if [ -n "${SWEEP_OUT:-}" ]; then
    DEST="${SWEEP_OUT}/${SIG}.md"
    [ -f "$DEST" ] && return 1
  fi

  {
    echo "---"
    echo "kind: $KIND"
    echo "title: $TITLE"
    echo "sig: $SIG"
    echo "---"
    echo
    echo "## 기계적 판정 근거"
    echo
    echo "저장소 CLI 가 계약 위반을 종료코드로 보고했다. 모델 판단이 개입하지 않았다."
    echo
    echo '```'
    head -60 "$EV"
    echo '```'
  } > "$DEST"

  if [ -n "${SWEEP_OUT:-}" ]; then
    echo "  발견 수집: [$KIND] sig:${SIG} — $TITLE"
    return 1   # 멈추지 않고 계속 훑는다
  fi
  echo "발견: [$KIND] $TITLE"
  echo "DETECT_KIND=$KIND"
  exit 0
}

rotate() { # rotate <목록파일> <개수> — 매 사이클 다른 구간을 보게 한다
  local total; total=$(wc -l < "$1" 2>/dev/null || echo 0)
  [ "$total" -eq 0 ] && return
  # 스윕 모드는 한 번에 여러 원인을 모으는 것이 목적이므로 창을 넓게 본다.
  local n=$(( $2 * ${SWEEP_FACTOR:-1} ))
  [ "$n" -gt "$total" ] && n="$total"
  awk -v s=$(( (ROTATION * n) % total )) -v n="$n" 'NR>s && NR<=s+n' "$1"
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
TOTAL=$(wc -l < "$WORK/pairs.txt")
echo "대응 쌍 ${TOTAL} 개 — 전수 조사 후 이상치만 취한다"

# ⚠ 자기 보정: 불일치가 '대부분'이면 그건 개별 버그가 아니라 알려진 구조적 격차다.
#   그걸 매 사이클 새 버그처럼 올리면 오탐 공장이 된다. 먼저 전수 측정해서
#   불일치가 소수(이상치)일 때만 결함으로 취급한다.
: > "$WORK/diverged.txt"
while read -r hwpx hwp; do
  [ -z "${hwpx:-}" ] && continue
  "$BIN" ir-diff "$hwpx" "$hwp" --json >"$WORK/ir_$(basename "$hwpx").log" 2>&1
  [ $? -eq 3 ] && echo "$hwpx $hwp" >> "$WORK/diverged.txt"
done < "$WORK/pairs.txt"

DIV=$(wc -l < "$WORK/diverged.txt")
PCT=$(( TOTAL > 0 ? DIV * 100 / TOTAL : 0 ))
echo "불일치 ${DIV}/${TOTAL} 쌍 (${PCT}%)"

if [ "$DIV" -eq 0 ]; then
  echo "IR 불일치 없음"
elif [ "$PCT" -gt 20 ]; then
  # 전반적으로 어긋나 있다 = 개별 결함이 아니라 미구현 영역. 봇이 손댈 대상이 아니다.
  echo "::notice::불일치가 ${PCT}% 로 광범위하다 — 개별 버그가 아니라 알려진 구조적 격차로 판단하고 이 층을 건너뛴다."
else
  read -r hwpx hwp < <(rotate "$WORK/diverged.txt" 1)
  [ -z "${hwpx:-}" ] && read -r hwpx hwp < "$WORK/diverged.txt"
  L="$WORK/ir_$(basename "$hwpx").log"
  { echo "샘플 쌍: $hwpx ↔ $hwp"
    echo "전체 ${TOTAL} 쌍 중 ${DIV} 쌍만 불일치 — 구조적 격차가 아니라 이 문서 고유의 이상치다."
  } >> "$L"
  emit "ir-mismatch" "HWP/HWPX 파서가 같은 문서에서 서로 다른 IR 을 만든다 ($(basename "$hwpx"))" "$L"
fi
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
# A. 크래시 퍼징 — 무한 탐색원. corpus 는 작업 트리 밖에 두고(게이트 STRAY 방지)
#    실행 간 캐시로 누적한다 — 돌수록 깊은 입력 공간을 탐색한다.
# ─────────────────────────────────────────────────────────────
STEP "A. 크래시 퍼징 (corpus 누적 탐색)"
if command -v cargo-fuzz >/dev/null 2>&1 && rustup toolchain list 2>/dev/null | grep -q nightly; then
  TARGETS=(parse_hwp parse_hwpx parse_hwp3 parse_hml parse_wmf parse_ooxml_chart)
  T="${TARGETS[$(( ROTATION % ${#TARGETS[@]} ))]}"
  echo "표적: $T"
  CORPUS_ROOT="${FUZZ_CORPUS_DIR:-$WORK/fuzz-corpus}"
  CORPUS="$CORPUS_ROOT/$T"
  mkdir -p "$CORPUS"
  # 시드: 비어 있으면 저장소 corpus + 실전 샘플로 채운다
  if [ -z "$(ls -A "$CORPUS" 2>/dev/null)" ]; then
    cp fuzz/corpus/"$T"/* "$CORPUS"/ 2>/dev/null || true
    case "$T" in
      parse_hwp|parse_hwp3) cp samples/*.hwp  "$CORPUS"/ 2>/dev/null || true ;;
      parse_hwpx)           cp samples/*.hwpx "$CORPUS"/ 2>/dev/null || true ;;
      parse_hml)            cp samples/*.hml  "$CORPUS"/ 2>/dev/null || true ;;
    esac
    echo "시드 투입: $(ls "$CORPUS" | wc -l)개"
  else
    echo "누적 corpus: $(ls "$CORPUS" | wc -l)개"
  fi
  L="$WORK/fuzz.log"
  rm -rf "fuzz/artifacts/$T"
  cargo +nightly fuzz run "$T" "$CORPUS" -- \
    -max_total_time="${FUZZ_SECS:-240}" -timeout=10 -max_len=65536 >"$L" 2>&1
  RC=$?
  if [ "$RC" -ne 0 ] && grep -qE 'panicked at|ERROR: libFuzzer|SUMMARY:' "$L"; then
    CRASH=$(ls "fuzz/artifacts/$T/" 2>/dev/null | head -1)
    EV="$WORK/fuzz-ev.log"
    {
      echo "표적: fuzz/fuzz_targets/${T}.rs"
      grep -E 'panicked at|SUMMARY:|ERROR:' "$L" | head -8
      if [ -n "$CRASH" ]; then
        SZ=$(stat -c%s "fuzz/artifacts/$T/$CRASH" 2>/dev/null || echo '?')
        echo
        echo "크래시 입력 (${SZ}B) — 회귀 테스트에 바이트 리터럴로 박아 재현할 것:"
        od -Ax -tx1 "fuzz/artifacts/$T/$CRASH" | head -40
        cp "fuzz/artifacts/$T/$CRASH" "${RUNNER_TEMP:-/tmp}/fuzz-crash-${T}.bin" 2>/dev/null || true
      fi
    } > "$EV"
    # 크래시 파일로 작업 트리를 오염시키지 않는다 — 입력은 위 hex 로 FINDING 에 보존된다
    rm -rf fuzz/artifacts
    emit "fuzz-crash" "퍼징 중 ${T} 파서가 패닉한다 (${CRASH:-crash})" "$EV"
  fi
  rm -rf fuzz/artifacts
  echo "크래시 없음 (${T}, ${FUZZ_SECS:-240}s)"
else
  echo "nightly/cargo-fuzz 미설치 — 이 층 건너뜀"
fi
END

echo "오라클 층 전부 통과 — 기계적으로 잡히는 결함 없음. AI 탐색으로 위임한다."
echo "DETECT_KIND=NONE"
