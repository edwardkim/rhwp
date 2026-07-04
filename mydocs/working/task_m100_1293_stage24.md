# Task 1293 Stage 24: 미주 설정 샘플 전체 sweep

## 목적

Stage23에서 `2024-11-practice-above20-between0-below20` target의 renderer overflow는
0건이 되었다. 이번 단계에서는 script에 등록된 전체 target을 다시 실행해 새 샘플과 기존
교육/실전 샘플에서 남은 미주 overflow 또는 시각 후보를 확인한다.

## 대상

- `scripts/task1274_visual_sweep.py --target all`
- 특히 새 미주 설정 샘플:
  - `2024-11-practice-shape987`
  - `2024-11-practice-above0-between0-below0`
  - `2024-11-practice-above0-between7-below2`
  - `2024-11-practice-above0-between7-below20`
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between0-below20`
  - `2024-11-practice-above20-between7-below2`
  - `2024-11-practice-no-separator-above20-between20-below20`

## 검증 계획

- `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage24_full_sweep --rhwp-bin target/debug/rhwp`
- summary에서 page count, `overflow_lines`, `frame_overflow_pages`, title/order/equation 후보를 확인한다.

## 실행 결과

- 실행 완료: `output/task1293_stage24_full_sweep/summary.json`
- 전체 target 수: 15개
- 모든 target에서 SVG/render tree/PDF page count가 1:1로 일치했다.
- `frame_overflow_pages`, `question_title_text_overlap_pages`,
  `line_order_overlap_pages`, `equation_text_overlap_pages`는 전 target에서 비어 있었다.
- 다만 renderer `LAYOUT_OVERFLOW` 로그가 여러 target에 남아 있어 미주 기능 완료로 판단할 수 없다.

| target | page count | overflow_lines | 판단 |
|---|---:|---:|---|
| `2022-09` | 23/23/23 | 2 | 기존 2022/2024-09 계열 동일 pi=948 잔여 |
| `2023-09` | 20/20/20 | 2 | page 12 우측 단 partial 잔여 |
| `2024-09-below20` | 23/23/23 | 2 | 2022-09와 같은 pi=948 잔여 |
| `2024-09-between20` | 24/24/24 | 8 | page 21/22 하단 잔여 |
| `2024-09-below20-above20` | 23/23/23 | 6 | page 18/21 잔여 |
| `2022-10` | 18/18/18 | 0 | overflow 없음 |
| `2022-11-practice` | 21/21/21 | 0 | overflow 없음 |
| `2024-11-practice-shape987` | 21/21/21 | 15 | 구분선위9/미주사이8/구분선아래7 설정 잔여 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 16 | 0/0/0 설정 잔여 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 0 | overflow 없음 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | 0 | overflow 없음 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 51 | 미주 사이 20mm 단독 계열 최우선 분석 대상 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | Stage23 보정 유지 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 15 | 구분선위20 + 미주사이7 + 구분선아래2 잔여 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 38 | 구분선 없음 + 20/20/20 계열 최우선 분석 대상 |

## 구현 계획서 목적 대비 판단

`mydocs/plans/task_m100_1293_impl.md`의 목적은 증상별 y/gap 보정을 멈추고 한컴 공식
`미주 모양` 의미를 IR과 렌더 계산식에 반영하는 것이다. 현재까지 정규화 접근자와 sweep
샘플 확대는 목적에 맞게 진행되었지만, Stage24 결과는 다음 항목이 아직 미완료임을 보여준다.

1. `미주 사이` 값이 큰 샘플에서 번호 경계의 flow 예약과 실제 렌더 하단 검출이 일치하지 않는다.
2. `구분선 없음` 샘플에서 separator block이 사라진 뒤에도 `구분선 위/아래` 값과 미주 사이 값이
   페이지네이션에 어떻게 소비되어야 하는지 아직 충분히 모델링되지 않았다.
3. `구분선위=0`, `미주사이=0`, `구분선아래=0` 샘플도 overflow가 남아 있어, 단순 큰 gap 문제만이
   아니라 compact 미주 paragraph 내부 vpos/line height 계산도 함께 봐야 한다.

## 다음 단계

Stage25에서는 overflow가 가장 큰 두 target을 먼저 분석한다.

- `2024-11-practice-above0-between20-below2`
  - `미주 사이 20mm`가 번호 경계에서 어떻게 소비되어야 하는지 확인한다.
  - page 9~11의 `pi=497`, `pi=543` 이후 overflow chain을 dump/render tree/PDF로 비교한다.
- `2024-11-practice-no-separator-above20-between20-below20`
  - 구분선이 없을 때 `구분선 위/아래`가 separator line이 아니라 미주 block 앞뒤 간격으로
    소비되는지 확인한다.
  - page 9의 `pi=464` 이후 overflow chain을 우선 분석한다.
