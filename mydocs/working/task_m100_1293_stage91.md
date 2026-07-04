# task 1293 stage91 - visible separator 20mm 미주 tail overflow 분리

## 목적

stage90에서 `2024-09-between20` p19의 `문28` question marker drift는 해소했다. stage91에서는
남은 p19 tail overflow와 p18/p11 후보를 marker anchor 보정과 분리해 다룬다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `5d153ea7 task 1293: visible separator head anchor 보정`
- stage90 기준 targeted sweep:
  - `2024-09-between20`: `flagged=3/24`
  - 남은 페이지: p11, p18, p19
  - `2024-11-practice-shape987`: `flagged=1/21`
  - `2024-11-practice-above0-between0-below0`: `flagged=0/21`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`

## 남은 문제

1. p19는 `문28` marker y는 맞았지만, `문28` tail의 p981 overflow와 p27 tail overflow가 남았다.
2. p18은 equation-only tail의 frame bottom 후보가 실제 overflow인지 sweep 과검출인지 분리해야 한다.
3. p11은 visual tail 저장 vpos와 실제 render-tree bbox의 역전/overlap 후보가 남아 있다.

## 처리 방향

- p19 marker anchor 보정은 유지한다.
- 새 stage에서는 tail overflow를 question marker y와 별도로 분류한다.
- p19 `문28` p981 overflow가 실제 한컴/PDF tail 위치 차이인지, render-tree frame overflow 과검출인지 먼저 확인한다.
- p18/p11은 p19와 같은 delayed TAC head 문제가 아니므로 같은 조건에 묶지 않는다.

## 구현 결과

- p19 `문28`의 큰 TAC 그림 p980 자체를 다음 쪽으로 넘기는 실험은 p20~p22 cascade를 다시 만들었다.
  - `2024-09-between20`: `flagged=6/24`
  - 회귀 페이지: p20, p21, p22
  - 판단: stage90에서 폐기한 boundary height 추가와 같은 부작용이므로 버린다.
- 최종 구현은 p980은 현재 쪽에 유지하고, 보이는 구분선 + 큰 `미주 사이` + 마지막 단 하단에서 큰 TAC 그림 바로 뒤의 한 줄 visible text tail만 다음 쪽으로 넘긴다.
  - 적용 조건: visible separator, non-default large between-notes, last column, `ep_idx > 1`, current height 96% 이상, current paragraph one visible-text line, 직전 emitted paragraph has TAC picture/shape height 80px 이상.
  - `2024-09-between20` p19에서 p980은 유지되고 p981만 p20 왼쪽 단 첫 항목으로 이동했다.
  - p20 오른쪽 단 `문29` cascade는 발생하지 않았다.

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
  --out output/task1293_stage91_targeted_v3
```

- `cargo fmt --check`: 통과
- `cargo build --bin rhwp`: 통과
- focused test: 통과
- targeted sweep v3:
  - `2024-09-between20`: `flagged=3/24`, tail `[18, 19]`, question `[]`, line `[11, 19]`, column `[11, 19]`, large `[11, 18, 19]`
  - `2024-11-practice-shape987`: `flagged=1/21`, tail `[12]`
  - `2024-11-practice-above0-between0-below0`: `flagged=0/21`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`

## 남은 후보

- p19의 `question_marker_drift`와 p20 cascade는 재발하지 않았다.
- p19 tail 후보는 p981에서 p27 tail/p980 쪽으로 축소됐고, p18 equation-only tail과 p11 visual overlap은 다음 stage에서 별도 판단한다.

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
  --out output/task1293_stage91_targeted
```

## 작업지시자 승인

2026-06-14 작업지시자가 stage90 커밋 승인 후 "승인 진행"으로 다음 stage 진행을 지시했다.
