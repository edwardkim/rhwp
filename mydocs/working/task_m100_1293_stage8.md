# Task 1293 Stage 8: 2024-11 미주 사이와 후반 flow mismatch 분석

## 목적

Stage7에서 2024-11 미주 모양 샘플 7종을 검증 세트에 편입하고 `구분선 없음`의 선분 표시와
위/아래 여백 소비를 분리했다. 그러나 새 샘플은 여전히 쪽수 mismatch를 드러낸다.

이번 스테이지의 목표는 다음 잔여 항목을 수치 보정이 아니라 공식 미주 흐름과 저장 LINE_SEG/vpos
해석으로 분리해 줄이는 것이다.

| target | PDF | rhwp | 관찰 |
|---|---:|---:|---|
| `2024-11-practice-shape987` | 21 | 22 | 9/8/7 샘플이 한컴보다 1쪽 늦다. |
| `2024-11-practice-above0-between20-below2` | 22 | 21 | `미주 사이 20mm` 단독 샘플이 한컴보다 1쪽 빠르다. |
| `2024-11-practice-above20-between0-below20` | 21 | 20 | 위/아래 20mm, 사이 0mm 샘플이 한컴보다 1쪽 빠르다. |
| `2024-11-practice-no-separator-above20-between20-below20` | 23 | 21 | 구분선 없음 20/20/20 샘플이 한컴보다 2쪽 빠르다. |

## 분석 계획

1. `above0-between20-below2`와 `no-separator-above20-between20-below20`의 9~11쪽 compare/dump를
   기준으로 `미주 사이 20mm`가 번호 경계에서 충분히 소비되는지 확인한다.
2. 2024-09 `미주사이20`은 24/24로 맞으므로, 기존 `endnote_between_notes_pagination_margin()`
   비율을 무조건 키우는 방식은 금지한다.
3. 2024-11 후반부 문29/문30의 TAC 그림/수식 되감기 문단이 한컴보다 한 단 빠르게 붙는지 확인한다.
4. `between=0` 샘플은 공식적으로 0mm gap이지만 한컴 PDF가 21쪽이므로, 줄/문단 vpos 또는 TAC
   예약 누락이 페이지 수를 줄이는지 분리한다.
5. 수정 후에는 최소한 다음을 동시에 확인한다.
   - `2024-09-between20` 24/24 유지
   - `2024-11-practice-above0-between7-below2` 21/21 유지
   - 새 mismatch target의 쪽수와 대표 compare 개선

## 검증 대기

- `cargo fmt --all -- --check`
- `cargo build --bin rhwp`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage8_between20 --rhwp-bin target/debug/rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage8_no_separator --rhwp-bin target/debug/rhwp`
- 필요 시 `2024-09-between20`, `2024-11-practice-shape987` 회귀 sweep

## Stage8-A: `미주 사이 20mm` 초과분 전체 반영

### 관찰

`2024-11-practice-above0-between20-below2`의 p10 compare는 PDF와 거의 같은 문항 분배를 보였지만,
p20~p21에서 rhwp가 문29 그림/다른풀이를 한 페이지 앞에 붙이고 문30 풀이까지 p21에 넣었다.
기존 2024-09 `미주사이20`은 24/24로 맞고 있었으므로, 먼저 `미주 사이 20mm` 초과분 예약식이
2024-11에서는 부족한지 분리했다.

기존 `endnote_between_notes_pagination_margin()`은 7mm 기본값 초과분의 `3/4`만 pagination에
예약했다. 이를 전체 초과분으로 바꾸어도 2024-09 `미주사이20`은 24/24를 유지했고,
`2024-11-practice-above0-between20-below2`는 21/22에서 22/22로 맞았다.

### 구현

- `src/renderer/typeset.rs`
  - `endnote_between_notes_pagination_margin()`이 7mm 기본값 초과분을 그대로 반환하도록 변경했다.
  - 주석도 "일부 예약" 설명에서 "20mm 초과분은 번호 경계마다 온전히 소비" 설명으로 수정했다.

### page count 재확인

| target | PDF | rhwp | 판정 |
|---|---:|---:|---|
| `2024-09-between20` | 24 | 24 | 유지 |
| `2024-11-practice-above0-between20-below2` | 22 | 22 | 개선 |
| `2024-11-practice-shape987` | 21 | 22 | 남음 |
| `2024-11-practice-above20-between0-below20` | 21 | 20 | 남음 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23 | 21 | 남음 |

남은 두 축은 `between=0`에서 나타나는 문단 간 vpos 되감기/overlap 의심, 그리고 `구분선 없음`
20/20/20의 후반 flow 과소 예약이다.

### 검증

- `cargo fmt --all -- --check` — 통과
- `python3 -m py_compile scripts/task1274_visual_sweep.py` — 통과
- `cargo build --bin rhwp` — 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` — 52 passed
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage8_between20 --rhwp-bin target/debug/rhwp`
  - SVG/render tree 22쪽, PDF 22쪽
  - `flagged=21/22`, frame `[11, 15, 19, 21]`, red `[10, 11, 12, 15, 17, 18, 20, 21, 22]`, line `[9, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 21, 22]`
