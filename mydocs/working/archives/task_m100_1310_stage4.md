# Task #1310 Stage 4 - 가설 B-D 판정 및 채택 후보

## 1. 현재 채택 후보

현재 증거 기준으로 채택 후보는 다음이다.

```text
가설 A 확장안:
수식-only TAC 흐름에서 연속 TAC 수식을 폭 기준으로 packing하고,
자동 줄넘김으로 생성된 virtual row에는 문단의 후속 줄 들여쓰기/내어쓰기 규칙을 적용한다.
```

Stage 2의 단순 가설 A는 자동 줄넘김만 처리해 후속 row의 x 기준이 부족했다.
Stage 3에서 후속 row effective margin 재계산까지 포함하면서 작업지시자 피드백을 반영했다.

## 2. 가설별 판정

### 가설 A - layout 단계 TAC-only packing

판정: 채택 후보.

근거:

- 연속 TAC 수식 3개 중 폭을 넘는 세 번째 수식이 다음 visual row로 넘어갔다.
- wrapped row가 첫 row x=402.5가 아니라 후속 줄 내어쓰기 x=442.9에서 시작한다.
- `typeset`/`height_measurer`도 같은 helper를 사용해 extra row 높이를 반영한다.
- `issue_1139_inline_picture_duplicate` 전체 68개가 통과하여 미주 페이지네이션 회귀가 확인되지 않았다.

### 가설 B - composer/IR line model 재정의 필요

판정: 현 단계 보류.

근거:

- 커서/강제 줄넘김/문단 경계 이동을 검증하는 `issue_1308_forced_break_hanging_indent` 8개가 통과했다.
- #1310 대상은 `runs.is_empty()`인 수식-only TAC 흐름에 한정되어 있고, 일반 inline text line model은 변경하지 않았다.
- 현재 문제는 renderer-only 임시 좌표가 아니라 높이 측정과 render tree가 같은 helper 결과를 공유하므로, 당장 `ComposedLine` 구조 자체를 재정의할 증거는 없다.

주의:

- 사용자가 웹에서 이 자동 wrap row의 hit-test/caret 이동을 별도로 실패 판정하면, B를 다시 열어야 한다.

### 가설 C - TAC 폭/단위 계산 문제

판정: 현 단계 보류.

근거:

- 기존 TAC 폭 값으로 packing했을 때 오른쪽 단 right=759.7을 넘지 않는다.
- 문제 TAC의 right는 625.7로 안정적으로 단 안에 들어온다.
- 폭/단위 계산을 전면 수정하지 않아도 목표 overflow와 후속 row x 기준을 동시에 해결했다.

주의:

- 한컴 기준과 비교해 줄넘김 위치 자체가 다르면 수식 bbox 폭 또는 control width 산정 문제를 다시 조사해야 한다.

### 가설 D - text/TAC/fixed-tab 혼합 순서 문제

판정: 이번 구현 범위 밖으로 유지.

근거:

- helper는 모든 `ComposedLine.runs`가 실제로 비어 있는 경우에만 동작한다.
- `eq-002`의 TAC/쉼표/고정탭/일반 글자 순서와 커서 이동 테스트가 통과했다.
- 혼합 문단은 #1308에서 처리한 순서 보존 경로를 유지한다.

주의:

- 수식-only가 아닌 혼합 줄에서도 자동 줄넘김이 필요하다는 한컴 기준 샘플이 나오면 별도 이슈로 확장해야 한다.

## 3. 검증 명령

통과:

```bash
cargo check
cargo check --target wasm32-unknown-unknown --lib
cargo fmt --all -- --check
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1256_2022_sep_page10_question12_keeps_between_notes_gap -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
docker compose --env-file .env.docker run --rm wasm
```

WASM 빌드:

```text
[INFO]: :-) Done in 3m 00s
[INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.
```

## 4. 시각 판정 요청

최종 채택 전 시각 판정 대상:

- `output/poc/task1310/stage3_indent_fixed/3-09월_교육_통합_2022_010.svg`

판정 포인트:

- 문12 하단의 수식-only 블록에서 연속 TAC 수식 3개 중 세 번째가 다음 visual row로 내려갔는가?
- 내려간 row가 문단 후속 줄 내어쓰기 기준으로 시작하는가?
- 문13 시작 전 흐름이 과도하게 벌어지거나 겹치지 않는가?

## 5. 남은 판단

시각 판정이 통과하면 가설 A 확장안을 최종 채택하고 완료 보고서로 넘어간다.
시각 판정이 실패하면 실패 항목에 따라 다음 분기 중 하나를 선택한다.

- 줄넘김 위치 실패: 가설 C 재검토
- 커서/hit-test 실패: 가설 B 재검토
- 혼합 문단 순서 실패: 가설 D 별도 확장
