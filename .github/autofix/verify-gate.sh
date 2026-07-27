#!/usr/bin/env bash
# rhwp autofix bot — 기계식 오탐 차단 게이트
#
# 이 스크립트는 Claude 의 판단을 신뢰하지 않는다. 작업 트리에 남은 변경만 보고
# "이것이 실제 버그 수정인가"를 명령 종료코드로만 판정한다.
# 하나라도 실패하면 PR 은 생성되지 않는다.
#
# 사용: verify-gate.sh <repo_root>
set -uo pipefail

ROOT="${1:?repo root 인자 필요}"
cd "$ROOT"

TEST_PROFILE="release-test"
FAIL() { echo "::error::GATE FAIL — $*"; exit 1; }
STEP() { echo "::group::[gate] $*"; }
END()  { echo "::endgroup::"; }

# ─────────────────────────────────────────────────────────────
# 0. 변경 형태 검사 — 회귀 테스트 + 소스 수정이 모두 있어야 버그 수정이다
# ─────────────────────────────────────────────────────────────
STEP "0. 변경 형태 검사"
git add -A -N .   # 신규 파일도 git diff 에 보이게 (스테이징은 하지 않음)

NEW_TESTS=$(git diff --name-only --diff-filter=A -- 'tests/issue_TBD_*.rs' || true)
SRC_CHANGES=$(git diff --name-only -- src/ || true)
STRAY=$(git diff --name-only \
  | grep -vE '^(src/|tests/issue_TBD_.*\.rs$|mydocs/|AUTOFIX_RESULT\.md$)' || true)
# src/ 신규 파일은 stash 로 원복이 깔끔하지 않아 red 증명이 흔들린다 —
# 버그 수정은 기존 코드 '수정'이어야 한다.
NEW_SRC=$(git diff --name-only --diff-filter=A -- src/ || true)

echo "신규 테스트: ${NEW_TESTS:-<없음>}"
echo "소스 변경  : ${SRC_CHANGES:-<없음>}"
echo "기타 변경  : ${STRAY:-<없음>}"

[ -n "$NEW_TESTS" ]   || FAIL "회귀 테스트(tests/issue_TBD_*.rs)가 없다. 재현 테스트 없는 수정은 오탐으로 간주한다."
[ -n "$SRC_CHANGES" ] || FAIL "src/ 변경이 없다. 테스트만 있는 변경은 버그 수정이 아니다."
[ -z "$STRAY" ]       || FAIL "허용 범위(src/, tests/issue_TBD_*, mydocs/) 밖 변경이 있다: $STRAY"
[ -z "$NEW_SRC" ]     || FAIL "src/ 에 신규 파일을 추가했다. 기존 코드 수정만 허용한다: $NEW_SRC"

TEST_COUNT=$(echo "$NEW_TESTS" | wc -l)
[ "$TEST_COUNT" -eq 1 ] || FAIL "신규 테스트가 ${TEST_COUNT}개다. PR 1건 = 버그 1건 = 테스트 1개."

TEST_FILE="$NEW_TESTS"
TEST_TARGET="$(basename "$TEST_FILE" .rs)"
echo "검증 대상 테스트: $TEST_TARGET"
END

# ─────────────────────────────────────────────────────────────
# 1. RED — 수정을 원복하면 신규 테스트가 '실패'해야 한다  ★오탐 차단의 핵심★
#    통과해 버리면 애초에 버그가 없었다는 뜻이므로 즉시 폐기한다.
# ─────────────────────────────────────────────────────────────
STEP "1. RED 증명 — 수정 원복 상태에서 테스트 실패 확인"
git stash push --quiet -- src/ || FAIL "src/ 변경 stash 실패"
RED_LOG=$(mktemp)
cargo test --profile "$TEST_PROFILE" --test "$TEST_TARGET" >"$RED_LOG" 2>&1
RED_RC=$?
git stash pop --quiet || FAIL "stash 복원 실패 — 작업 트리가 오염되었다"

tail -40 "$RED_LOG"

if [ "$RED_RC" -eq 0 ]; then
  FAIL "수정을 원복해도 테스트가 통과한다 → 존재하지 않는 버그(오탐). 폐기한다."
fi
if grep -qE '^error\[E[0-9]+\]|^error: could not compile|^error: cannot find' "$RED_LOG"; then
  FAIL "원복 상태에서 컴파일 에러가 났다(테스트 실패가 아님). 버그 재현이 증명되지 않았으므로 폐기한다."
fi
grep -qE 'test result: FAILED|panicked at|assertion' "$RED_LOG" \
  || FAIL "실패 원인이 테스트 단정(assertion)이 아니다. 재현 증명으로 인정하지 않는다."
echo "RED 확인 — 수정 없이는 테스트가 단정 실패한다."
END

# ─────────────────────────────────────────────────────────────
# 2. GREEN — 수정 적용 상태에서 같은 테스트가 통과해야 한다
# ─────────────────────────────────────────────────────────────
STEP "2. GREEN 증명 — 수정 적용 상태에서 테스트 통과 확인"
cargo test --profile "$TEST_PROFILE" --test "$TEST_TARGET" 2>&1 | tail -40
[ "${PIPESTATUS[0]}" -eq 0 ] || FAIL "수정을 적용해도 테스트가 통과하지 않는다."
echo "GREEN 확인 — red→green 성립."
END

# ─────────────────────────────────────────────────────────────
# 3. CONTRIBUTING.md 의 PR 전 체크리스트 3종
# ─────────────────────────────────────────────────────────────
STEP "3-1. cargo fmt --all -- --check"
cargo fmt --all -- --check || FAIL "포맷 위반. CI 가 막는다."
END

STEP "3-2. cargo clippy -- -D warnings"
cargo clippy -- -D warnings 2>&1 | tail -60
[ "${PIPESTATUS[0]}" -eq 0 ] || FAIL "clippy 경고 존재. CI 가 막는다."
END

STEP "3-3. cargo test --profile release-test --tests (전체 회귀)"
FULL_LOG=$(mktemp)
cargo test --profile "$TEST_PROFILE" --tests --no-fail-fast >"$FULL_LOG" 2>&1
FULL_RC=$?
grep -E 'test result:' "$FULL_LOG" | tail -20
if [ "$FULL_RC" -ne 0 ]; then
  echo "── 실패 목록 ──"
  grep -E '^(failures:|    [a-z_0-9:]+$)' "$FULL_LOG" | head -40
  FAIL "전체 회귀 테스트 실패. 인접 기능을 깨뜨렸다."
fi
echo "전체 회귀 0건 확인."
END

echo "GATE PASS — red→green 성립 + 회귀 0 + CI 3종 통과."
