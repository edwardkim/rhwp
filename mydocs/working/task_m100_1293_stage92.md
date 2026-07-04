# task 1293 stage92 - visible separator 20mm tail overflow 후보 분류

## 목적

stage91에서 p19 `문28`의 question marker drift와 p20 cascade를 피하면서 TAC 뒤 한 줄 tail만 이월했다.
stage92에서는 남은 `2024-09-between20` p18/p19의 render-tree frame tail overflow 후보를 실제 pagination
문제와 검출기 과검출로 나눈다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `d419831b task 1293: visible separator TAC 뒤 tail 이월 보정`
- stage91 targeted sweep v3:
  - `2024-09-between20`: `flagged=3/24`
  - 남은 tail 후보: p18, p19
  - question marker drift: 없음
  - `2024-11-practice-shape987`: `flagged=1/21`
  - `2024-11-practice-above0-between0-below0`: `flagged=0/21`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`

## 처리 방향

- p19에서 stage91이 이동한 p981은 유지한다.
- p18/p19의 tail 후보가 PDF에서도 같은 frame 하단 bleed를 갖는지, 또는 render-tree bbox가 lineSeg/그림 baseline을 과대 계상한 것인지 분리한다.
- 실제 pagination 문제로 확인되는 경우에만 typeset 조건을 좁게 추가한다.
- p11 visual order overlap은 이번 stage에서 코드 수정 대상으로 섞지 않는다.

## 구현 기록

### p19 `문27` equation 뒤 한 줄 tail

- stage91 기준 p19 `pi=961` `그러므로`는 직전 `pi=960` 수식-only 문단 뒤에 이어지는 한 줄 visible text이다.
- typeset의 cursor fit 값은 column 하단 안에 들어간다고 보지만, `predict_current_column_para_y` 기준 render y는 frame 하단을 넘는다.
- visible separator + `미주 사이 20mm` + 현재 column이 마지막 column이 아닌 경우에 한해, 직전 문단이 visible text 없는 equation control이고 현재 문단이 한 줄 visible text이면 render y 기준으로 다음 column advance를 건다.
- 최종 `dump-pages` 확인:
  - p19 단0은 `pi=960`까지 유지한다.
  - p19 단1은 `pi=961` `그러므로`로 시작한다.

### p19 `문28` TAC 뒤 한 줄 tail 유지

- 첫 구현은 `pi=961`을 오른쪽 단으로 보내는 데는 성공했지만, 이어서 `pi=980` TAC 문단까지 다음 페이지로 밀어 p20 이후 question drift를 다시 만들었다.
- 실패 sweep:
  - `output/task1293_stage92_targeted`
  - `2024-09-between20`: `flagged=5/24`, `tail=[18,20,21]`, `question=[20,21,22]`
- 최종 구현에서는 마지막 column의 큰 TAC 문단 뒤에 한 줄 visible tail이 이어지는 경우, TAC 하단이 기존 bottom bleed tolerance 안이면 TAC 문단은 현재 column에 남긴다.
- stage91 규칙이 이어서 `pi=981` 한 줄 tail을 다음 페이지로 넘긴다.
- 최종 `dump-pages` 확인:
  - p19 단1 끝에 `pi=980` TAC 그림 문단이 남는다.
  - p20 단0은 `pi=981` `한편, 점 에서...`로 시작한다.

### p18 남은 후보

- p18 남은 후보는 `pi=922` `문29`의 `[EQ]` tail overflow이다.
- 최종 sweep 기준 `overflow_px=24.5`, `frame_overflow_tolerated_bleed=true`, PDF outside frame pixel은 0이다.
- 이번 stage에서는 p19의 실제 pagination drift만 수정하고, p18 수식 bbox/tail 후보는 다음 stage에서 render-tree bbox 또는 equation baseline 계상 문제로 별도 분리한다.

## 검증 계획

```bash
cargo fmt --check
cargo build --bin rhwp
cargo test --test issue_1139_inline_picture_duplicate issue_1293_equation_control_is_not_always_treat_as_char -- --nocapture
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage92_targeted
```

## 작업지시자 승인

2026-06-14 작업지시자가 stage91 커밋 후 "승인 시작"으로 다음 stage 시작을 승인했다.

## 검증 결과

```bash
cargo fmt --check
cargo build --bin rhwp
cargo test --test issue_1139_inline_picture_duplicate issue_1293_equation_control_is_not_always_treat_as_char -- --nocapture
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage92_targeted_v2
```

- `cargo fmt --check`: 통과
- `cargo build --bin rhwp`: 통과
- 집중 테스트: 통과 (`issue_1293_equation_control_is_not_always_treat_as_char`)
- targeted sweep v2:
  - `2024-09-between20`: `flagged=2/24`, `line=[11]`, `column=[11]`, `tail=[18]`, `question=[]`, `large=[11,18]`
  - `2024-11-practice-shape987`: `flagged=1/21`, `column=[12]`, `tail=[12]`
  - `2024-11-practice-above0-between0-below0`: `flagged=0/21`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`
