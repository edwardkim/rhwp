#!/usr/bin/env bash
# rhwp autofix bot — 발견 루프
#
# 한 run 안에서 [오라클 탐지 → (확정 수정 | AI 탐색) → 기계식 게이트 → 반박 심사]
# 사이클을, 전부 통과하는 발견이 나올 때까지 반복한다. 시도마다 오라클 회전 창과
# 탐색 표적 모듈을 바꾸고, 탈락한 후보는 제외 목록에 쌓아 같은 곳을 다시 파지 않는다.
#
# 모든 시도가 실패해도, 오라클이 기계적으로 확정한 결함이 있으면 status=ORACLE-ONLY 로
# 보고한다 — 워크플로가 이슈로 등록한다(수정 없이). 발견이 사라지는 사이클은 없다.
#
# '무조건 PR'은 기준 완화로 달성하지 않는다 — 게이트(red→green+회귀0+CI 3종)와
# 반박 심사는 그대로 두고, 재시도 횟수와 폴백으로 달성한다.
#
# 사용: cycle-loop.sh <bot_dir> <work_dir> <run_number>
# 필요 env: MODEL OPENAI_API_KEY OPENAI_API_BASE NVIDIA_API_KEY GITHUB_OUTPUT
#           HUNT_MODE HUNT_RANGE HUNT_TARGET HUNT_EXCLUDE
# 선택 env: MAX_ATTEMPTS(기본 5) BUDGET_SECS(기본 3300 = 55분)
# 출력(GITHUB_OUTPUT): status=FOUND|ORACLE-ONLY|NO-FINDING, slug, title, attempts
set -uo pipefail

BOT="${1:?bot dir 인자 필요}"
WORK="${2:?work dir 인자 필요}"
RUN_NO="${3:-0}"
MAX_ATTEMPTS="${MAX_ATTEMPTS:-5}"
BUDGET_SECS="${BUDGET_SECS:-3300}"
START=$(date +%s)
TMP="${RUNNER_TEMP:-/tmp}"
ATTEMPTED="$TMP/attempted-titles.txt"
: > "$ATTEMPTED"
# 오라클 제외 목록 — upstream 에 이미 보고된 제목 + 이번 run 탈락 제목.
# detect.sh 의 emit 이 이 파일을 보고 같은 발견을 건너뛰며 다음 표본/다음 층으로
# 파고든다. '같은 발견 반복 → 탐색 정체'를 끊는 장치.
DETECT_EXCLUDE_FILE="$TMP/detect-exclude.txt"
export DETECT_EXCLUDE_FILE
cd "$WORK"

# repo-map 을 크게 준다 — 11,000+ 파일 저장소에서 1024 토큰 지도는 관련 경로조차
# 못 보여줘 모델이 '디렉터리를 추가해달라'며 헛돌았다(run #12 사고).
# NIM 크레딧은 요청 단위로 닳으므로 컨텍스트를 키우는 것은 공짜다.
AIDER_FLAGS=(--yes-always --no-auto-commits --no-analytics --no-gitignore
             --no-show-model-warnings --map-tokens 8192 --edit-format diff)

out() { echo "$1" >> "$GITHUB_OUTPUT"; }
elapsed() { echo $(( $(date +%s) - START )); }

# target/ 을 지우면 시도마다 전체 재빌드가 나므로 반드시 남긴다.
reset_tree() {
  git reset --hard --quiet
  git clean -fdq -e target/
}

run_aider() { # run_aider <logfile> <추가 인자...>
  local LOG="$1"; shift
  aider --model "openai/${MODEL}" "${AIDER_FLAGS[@]}" "$@" 2>&1 | tee -a "$LOG" || true
}

# 산출물(AUTOFIX_RESULT.md)이 나올 때까지 같은 대화를 최대 3턴 이어간다.
aider_until_result() { # aider_until_result <logfile> <msgfile> [파일·플래그...]
  local LOG="$1" MSG="$2"; shift 2
  run_aider "$LOG" "$@" --message "$(cat "$MSG")"
  local TURN
  for TURN in 2 3; do
    [ -f AUTOFIX_RESULT.md ] && break
    echo "--- ${TURN}턴: AUTOFIX_RESULT.md 미생성 — 대화 복원 후 계속 ---" | tee -a "$LOG"
    run_aider "$LOG" "$@" --restore-chat-history \
      --message "아직 루트에 AUTOFIX_RESULT.md 가 없다. 지시서의 남은 단계(red 회귀 테스트 작성 → src/ 최소 수정 → AUTOFIX_RESULT.md 작성)를 지금 끝내라. 파일이 더 필요하면 디렉터리가 아니라 정확한 파일 경로를 한 줄에 하나씩 지정해라. 포기하는 경우에도 status: NO-FINDING 으로 AUTOFIX_RESULT.md 는 반드시 남겨라."
  done
}

result_field() { # result_field <key> — AUTOFIX_RESULT.md 프론트매터에서 값 추출
  awk -v k="$1" 'BEGIN{p="^"k": "} $0 ~ p {sub(p, ""); print; exit}' AUTOFIX_RESULT.md | tr -d '\r'
}

# FINDING.md 의 식별자(에러 문자열·필드명)를 git grep 으로 역추적해 관련 소스를 찾는다.
# 5개 이하 파일에만 나타나는 토큰 = 판별력 있는 토큰. 결과는 CTX_FILES 배열.
collect_fix_context() {
  CTX_FILES=()
  local tok
  while read -r tok; do
    [ -z "$tok" ] && continue
    local HITS=()
    mapfile -t HITS < <(git grep -lF -e "$tok" -- src 2>/dev/null | head -6)
    if [ "${#HITS[@]}" -ge 1 ] && [ "${#HITS[@]}" -le 5 ]; then
      CTX_FILES+=("${HITS[@]}")
    fi
  done < <(grep -oE '[A-Za-z_][A-Za-z0-9_]{5,}' FINDING.md | sort -u | head -40)
  mapfile -t CTX_FILES < <(printf '%s\n' "${CTX_FILES[@]}" | grep -v '^$' | sort -u | head -10)
}

for (( A=1; A<=MAX_ATTEMPTS; A++ )); do
  E=$(elapsed)
  if [ "$E" -ge "$BUDGET_SECS" ]; then
    echo "시간 예산(${BUDGET_SECS}s) 소진 — 루프 종료 (경과 ${E}s)"
    break
  fi
  echo "::group::[loop] 시도 ${A}/${MAX_ATTEMPTS} — 경과 $(( E / 60 ))분"
  reset_tree

  # ── 1) 오라클 탐지 — 시도마다 다른 회전 창을 보고, 이미 처리된 발견은 건너뛴다
  { printf '%s\n' "${HUNT_EXCLUDE:-}"; cat "$ATTEMPTED"; } > "$DETECT_EXCLUDE_FILE"
  bash "$BOT/.github/autofix/detect.sh" "$PWD" "$(( RUN_NO * 7 + A - 1 ))" 2>&1 | tee -a ../detect.log
  KIND=$(grep -oE 'DETECT_KIND=[A-Za-z-]+' ../detect.log | tail -1 | cut -d= -f2)
  [ "$KIND" = "NONE" ] && KIND=""

  F_TITLE=""
  if [ -n "$KIND" ] && [ -f FINDING.md ]; then
    F_TITLE=$(awk '/^title:/{sub(/^title: /, ""); print; exit}' FINDING.md | tr -d '\r')
    # 폴백용으로 이번 run 의 첫 오라클 발견을 보존한다 — 단 upstream 에 같은 제목의
    # 이슈/PR 이 이미 있으면(열림·닫힘 불문) 중복 보고이므로 제외.
    if [ -n "$F_TITLE" ] && [ ! -f "$TMP/oracle-finding.md" ]; then
      if ! printf '%s\n' "${HUNT_EXCLUDE:-}" | grep -qF -- "$F_TITLE"; then
        cp FINDING.md "$TMP/oracle-finding.md"
      fi
    fi
    # 이번 run 에서 이미 탈락한 발견을 오라클이 또 내면 → AI 탐색으로 전환
    if [ -n "$F_TITLE" ] && grep -qxF -- "$F_TITLE" "$ATTEMPTED"; then
      echo "오라클이 이번 run 에서 이미 탈락한 발견을 다시 냈다 — AI 탐색으로 전환: $F_TITLE"
      KIND=""
      rm -f FINDING.md
    fi
  fi

  # ── 2) 확정 수정 또는 AI 탐색 (읽을 것·고칠 것을 전부 대화에 직접 올린다)
  READS=()
  [ -f AGENTS.md ] && READS+=(--read AGENTS.md)
  [ -f CONTRIBUTING.md ] && READS+=(--read CONTRIBUTING.md)

  if [ -n "$KIND" ]; then
    collect_fix_context
    echo "미리 올리는 관련 소스 (${#CTX_FILES[@]}개):"
    printf '  %s\n' "${CTX_FILES[@]}"
    {
      cat "$BOT/.github/autofix/fix-prompt.md"
      printf '\n---\n\n## FINDING.md 전문 (파일을 다시 읽을 필요 없다 — 아래가 전부다)\n\n'
      cat FINDING.md
      printf '\n---\n\n## 이미 대화에 올라와 있는 관련 소스\n\n'
      printf -- '- %s\n' "${CTX_FILES[@]}"
      printf '\n디렉터리는 추가할 수 없다. 다른 파일이 필요하면 정확한 파일 경로를 한 줄에 하나씩 지정해라.\n'
    } > "$TMP/msg.md"
    aider_until_result ../fix.log "$TMP/msg.md" --read FINDING.md "${READS[@]}" "${CTX_FILES[@]}"
  else
    # 첫 시도는 guard 가 정한 모드, 이후는 모듈 회전으로 표적을 바꾼다 (guard 와 같은 목록)
    MODULES=(src/parser/hwp src/parser/hwpx src/parser/hwp3 src/renderer src/layout src/editor src/document)
    if [ "$A" -eq 1 ] && [ "${HUNT_MODE:-module}" = "diff" ]; then
      MODE_EFF="diff"; TARGET_EFF=""
      mapfile -t CTX_FILES < <(git log ${HUNT_RANGE:-} --name-only --pretty=format: -- src/ 2>/dev/null | grep '\.rs$' | sort -u | head -8)
    else
      MODE_EFF="module"
      TARGET_EFF="${MODULES[$(( (RUN_NO + A) % ${#MODULES[@]} ))]}"
      mapfile -t CTX_FILES < <(git ls-files -- "$TARGET_EFF" 2>/dev/null | grep '\.rs$' | head -8)
    fi
    mapfile -t CTX_FILES < <(printf '%s\n' "${CTX_FILES[@]}" | grep -v '^$' | head -8)
    echo "미리 올리는 표적 소스 (${#CTX_FILES[@]}개, 모드=${MODE_EFF}):"
    printf '  %s\n' "${CTX_FILES[@]}"
    {
      cat "$BOT/.github/autofix/hunt-prompt.md"
      printf '\n---\n\n## 이번 사이클 파라미터 (환경변수를 조회할 필요 없다 — 아래가 그 값이다)\n\n'
      printf -- '- HUNT_MODE: %s\n- HUNT_RANGE: %s\n- HUNT_TARGET: %s\n\n' \
        "$MODE_EFF" "${HUNT_RANGE:-}" "$TARGET_EFF"
      printf '### 제외 목록 — 이미 올렸거나 이번 run 에서 탈락한 것 (겹치면 즉시 폐기)\n\n%s\n' \
        "${HUNT_EXCLUDE:-<없음>}"
      if [ -s "$ATTEMPTED" ]; then
        echo
        sed 's/^/- [이번 run 탈락] /' "$ATTEMPTED"
      fi
      printf '\n---\n\n## 이미 대화에 올라와 있는 표적 소스\n\n'
      printf -- '- %s\n' "${CTX_FILES[@]}"
      printf '\n디렉터리는 추가할 수 없다. 다른 파일이 필요하면 정확한 파일 경로를 한 줄에 하나씩 지정해라.\n'
    } > "$TMP/msg.md"
    aider_until_result ../hunt.log "$TMP/msg.md" "${READS[@]}" "${CTX_FILES[@]}"
  fi

  # aider 부산물은 게이트의 STRAY 검사에 걸린다 — 대화가 끝난 즉시 지운다.
  rm -rf .aider*

  # ── 3) 산출물 판정
  if [ ! -f AUTOFIX_RESULT.md ]; then
    echo "산출물 없음 — 다음 시도"
    [ -n "$F_TITLE" ] && echo "$F_TITLE" >> "$ATTEMPTED"
    echo "::endgroup::"; continue
  fi
  STATUS=$(result_field status | tr -d ' ')
  if [ "$STATUS" != "FOUND" ]; then
    echo "결과: ${STATUS:-<없음>} — 다음 시도"
    [ -n "$F_TITLE" ] && echo "$F_TITLE" >> "$ATTEMPTED"
    echo "::endgroup::"; continue
  fi
  TITLE=$(result_field title)
  SLUG=$(result_field slug | tr -d ' ')

  # ── 4) 기계식 게이트 (red→green + 회귀 0 + CI 3종)
  if ! bash "$BOT/.github/autofix/verify-gate.sh" "$PWD" 2>&1 | tee -a ../gate.log; then
    echo "게이트 탈락 — 다음 시도: $TITLE"
    echo "$TITLE" >> "$ATTEMPTED"
    [ -n "$F_TITLE" ] && [ "$F_TITLE" != "$TITLE" ] && echo "$F_TITLE" >> "$ATTEMPTED"
    echo "::endgroup::"; continue
  fi

  # ── 5) 반박 심사 — 판정이므로 편집 에이전트 없이 요청 1회
  jq -n --arg m "$MODEL" \
        --arg sys "$(cat "$BOT/.github/autofix/refute-prompt.md")" \
        --arg usr "$(printf '## AUTOFIX_RESULT.md\n%s\n\n## git diff\n%s\n' \
                      "$(cat AUTOFIX_RESULT.md)" "$(git diff | head -c 60000)")" \
    '{model:$m, temperature:0, max_tokens:1200,
      messages:[{role:"system",content:$sys},{role:"user",content:$usr}]}' > "$TMP/refute.json"
  curl -sS https://integrate.api.nvidia.com/v1/chat/completions \
    -H "Authorization: Bearer ${NVIDIA_API_KEY}" \
    -H "Content-Type: application/json" \
    -d @"$TMP/refute.json" \
    | jq -r '.choices[0].message.content // "판정 없음"' > "$TMP/refute-now.log" || true
  tee -a ../refute.log < "$TMP/refute-now.log"
  VERDICT=$(grep -oE 'VERDICT: (REAL|REFUTED)' "$TMP/refute-now.log" | tail -1 || true)
  if [ "$VERDICT" != "VERDICT: REAL" ]; then
    echo "반박 심사 탈락(${VERDICT:-판정 없음}) — 다음 시도: $TITLE"
    echo "$TITLE" >> "$ATTEMPTED"
    echo "::endgroup::"; continue
  fi

  # ── 성공: 작업 트리를 수정 적용 상태로 남기고 종료 (제출은 워크플로 몫)
  echo "::endgroup::"
  echo "발견 확정 (시도 ${A}/${MAX_ATTEMPTS}) — $TITLE"
  out "status=FOUND"
  out "slug=${SLUG}"
  { echo "title<<AUTOFIX_EOF"; echo "$TITLE"; echo "AUTOFIX_EOF"; } >> "$GITHUB_OUTPUT"
  out "attempts=${A}"
  exit 0
done

# ── 폴백: 수정은 못 했어도 오라클이 기계적으로 확정한 결함은 반드시 보고한다
if [ -f "$TMP/oracle-finding.md" ]; then
  reset_tree
  cp "$TMP/oracle-finding.md" FINDING.md
  T=$(awk '/^title:/{sub(/^title: /, ""); print; exit}' FINDING.md | tr -d '\r')
  echo "자동 수정은 게이트 기준을 못 넘겼지만, 오라클이 확정한 결함이 있다 — 이슈 폴백: $T"
  out "status=ORACLE-ONLY"
  { echo "title<<AUTOFIX_EOF"; echo "$T"; echo "AUTOFIX_EOF"; } >> "$GITHUB_OUTPUT"
  out "attempts=${MAX_ATTEMPTS}"
  exit 0
fi

echo "모든 시도 소진 — 오라클 확정 결함도, 게이트를 넘는 수정도 없었다. 기준은 낮추지 않는다."
out "status=NO-FINDING"
out "attempts=${MAX_ATTEMPTS}"
exit 0
