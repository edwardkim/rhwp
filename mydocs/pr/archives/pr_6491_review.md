# PR #6491 검토 - SQUEEZE 표 셀 안 여백

- 원 PR head: c7b8611c9f4d2b1cf24b5ce0a253f3cc96bc6f32
- 통합 cherry-pick: 159c8679c8ad10e5b5eb0faf45faa658749094de
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 메인터너 보정 됨 수용 가능

## 확인한 범위

Cell::line_wrap == SQUEEZE일 때 overflow 방어가 표의 안 여백을 1px까지 줄이지 않도록 한다.

## 검증 및 증적

issue_6145_squeeze_cell_keeps_inner_margin 2/2와 공통 전체 회귀를 통과했다.

원 PR 증적: mydocs/report/6145-squeeze-cell-inner-margin/{before,after,compare,hangul}.png.

## 다음 조건

현 통합 head에서 samples/issue6145/worklife_balance_index_156607916.hwpx와 확인된 Hancom oracle PDF를 직접 대조하고, 여백 보존과 괘선 내 텍스트 종료를 기록한다.

공통 검증 세부 내용은 pr_6489_6517_planet6897_integration_evidence.md를 따른다.
## 2026-08-31 메인터너 보정 검증

**최종 판정: 메인터너 보정 됨 수용 가능.**

- `rhwp info --json`으로 원본이 Hancom Office 2018 저장본임을 확인해 Hancom `2020` profile로 PDF를 재산출했다: SHA-256 `582fbc7da7be893f5b24b0ad34be80fba3c1ea8958a2ba5e4d811d68a99d1fc2`.
- 6쪽 전부를 현재 후보와 직접 sweep했고 누락 페이지, frame overflow, content-bottom drift, column text-flow collapse, legacy glyph flag가 모두 없었다.
- 검증 이미지는 `mydocs/pr/assets/pr_6489_6517_planet6897_integration_20260831/maintainer-20260831/pr6491-p001-review.png`에 보존했다. 픽셀 일치율은 글꼴 raster 차이를 포함한 보조 지표이며, 렌더링 충실도 통과 수치로 사용하지 않는다.
