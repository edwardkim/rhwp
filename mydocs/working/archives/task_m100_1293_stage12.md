# Task 1293 Stage 12: 2024-11 미주 잔여 시각 flag 판별

## 목적

Stage11에서 2024-11 미주 모양 샘플 8종은 모두 PDF/SVG/render tree 쪽수 1:1까지 맞췄다.
그러나 sweep 지표에는 아직 frame/red/line drift 후보가 남아 있고, 특히
`2024-11-practice-shape987`은 `question_title_text_overlap`과 `line_order_overlap` 후보가 남았다.

쪽수만 맞추고 끝내면 문항 제목/수식 겹침을 놓칠 수 있으므로, Stage12에서는 자동 지표 후보를 실제
compare PNG/annotated PNG/render tree 기준으로 판별한다.

## 현재 기준

Stage11 전체 sweep:

- 산출물: `output/task1293_stage11_sample_check_all_v2/summary.json`
- compare PNG: `output/task1293_stage11_sample_check_all_v2/*/compare/compare_*.png`
- annotation PNG: `output/task1293_stage11_sample_check_all_v2/*/analysis/annotated_*.png`

요약:

| target | PDF/SVG/tree | frame 후보 | title 후보 | order 후보 |
|---|---:|---:|---:|---:|
| `2024-11-practice-shape987` | 21/21/21 | 6 | 1 | 2 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 3 | 0 | 0 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 2 | 0 | 0 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | 1 | 0 | 0 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 4 | 0 | 0 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 2 | 0 | 0 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 0 | 0 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 2 | 0 | 0 |

## 진행 계획

1. `shape987`의 title/order 후보 페이지를 우선 열람해 실제 문항 제목 겹침인지 판별한다.
2. 실제 겹침이면 해당 페이지의 render tree bbox와 미주 paragraph split 정보를 추적한다.
3. 오탐이면 sweep 기준을 더 정확하게 조정하고, 오탐으로 판단한 근거를 문서화한다.
4. frame overflow 후보는 쪽수 정합 이후에도 실제 하단 bleed가 보이는 페이지만 별도 후속 대상으로 남긴다.

## 분석 결과

### p14 문27 제목-본문 겹침

`compare_014.png` 기준 `shape987` p14 오른쪽 단 하단에서 rhwp는 문27 제목과 첫 본문 줄이 겹쳤다.
PDF에도 문27 제목과 첫 본문 줄이 같은 페이지 하단에 있으므로, 새 미주 전체를 다음 페이지로 넘기는 문제가
아니라 렌더 단계에서 제목 다음 본문을 너무 위로 당기는 문제였다.

임시 trace와 `RHWP_VPOS_DEBUG` 확인:

- `pi=669` 문27 제목: pagination `en_advance=21.8px`로 정상 전진
- `pi=670` 문27 첫 본문: sequential `y_in=1006.33px`, 저장 vpos 보정 `end_y=985.91px`
- 기존 `compact_endnote_title_tail_backtrack`이 최대 16px backtrack을 적용해 `result=990.33px`로 렌더
- 직전 제목의 실제 하단은 약 `1000.3px`이므로 첫 본문이 제목 bbox 안으로 올라갔다.

수정:

- `compact_endnote_title_tail_backtrack` 결과를 `prev_content_bottom_y + 2px` 이상으로 제한했다.
- 저장 vpos backtrack을 완전히 끄지 않고, 제목 실제 하단 아래에서만 허용한다.

### p12 잔여 order 후보

Stage12 후 `question_title_text_overlap`은 사라졌고, `line_order_overlap`은 p12 후보 하나만 남았다.
이 후보는 `next_pi=0`인 표/수식 node와 p12 하단 표 주변에서 잡힌 것으로, 문항 제목-본문 겹침과는 다른
하단 frame overflow 계열이다. 다음 스테이지에서 p12/p14/p19/p20/p21 frame overflow 후보를 묶어서
분리 분석한다.

## 검증 대기

- [x] `cargo fmt --all -- --check`
- [x] `cargo build --bin rhwp`
- [x] `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- [x] `2024-11-practice-shape987` 단독 sweep
  - 21/21/21 유지
  - `question_title_text_overlap_pages: []`
  - `line_order_overlap_pages: [12]`
- [x] 2024-11 신규 샘플 8종 sweep

| target | PDF | SVG | render tree | title 후보 | order 후보 |
|---|---:|---:|---:|---|---|
| `2024-11-practice-shape987` | 21 | 21 | 21 | 없음 | 12 |
| `2024-11-practice-above0-between0-below0` | 21 | 21 | 21 | 없음 | 없음 |
| `2024-11-practice-above0-between7-below2` | 21 | 21 | 21 | 없음 | 없음 |
| `2024-11-practice-above0-between7-below20` | 21 | 21 | 21 | 없음 | 없음 |
| `2024-11-practice-above0-between20-below2` | 22 | 22 | 22 | 없음 | 없음 |
| `2024-11-practice-above20-between0-below20` | 21 | 21 | 21 | 없음 | 없음 |
| `2024-11-practice-above20-between7-below2` | 21 | 21 | 21 | 없음 | 없음 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23 | 23 | 23 | 없음 | 없음 |

- [x] 2024-09 회귀 sweep
  - `2024-09-between20`: 24/24/24
  - `2024-09-below20-above20`: 23/23/23

대표 산출물:

- `output/task1293_stage12_shape987_check/summary.json`
- `output/task1293_stage12_sample_check_all/summary.json`
- `output/task1293_stage12_2024_09_regression/summary.json`
