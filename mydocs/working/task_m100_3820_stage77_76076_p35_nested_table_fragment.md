---
kind: implementation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 77 — 76076 p35 nested-table fragment paint

## 입력 계약

대상은 `samples/76076_regulatory_analysis.hwp`와 한컴 기준 PDF
`samples/issue1891/76076_regulatory_analysis-2024.pdf`의 p35이다.

Stage 76 (`e1d112759`)은 p35→p36의 RowBreak rowspan 행 꼬리 소유권만 고쳤다.
그 결과 p36의 다음 행 시작은 PDF와 같은 continuation owner를 사용한다. 그러나 p35의
`8.피규제집단 및 이해관계자` 셀에는 PDF에 있는 내부 표가 RHWP output에 없다.
이는 [p35 direct comparison](../pr/assets/task_m100_3820_stage76_rowbreak_rowspan_band/compare_035.png)에서
왼쪽 RHWP와 가운데 PDF를 직접 대조해 확인한 미해결 결함이다.

## 분석 질문

1. 내부 표 control이 p35 fragment의 pagination item에 존재하는가, 아니면 outer-row split 전에
   버려지는가?
2. render tree에 존재한다면 fragment clip/row-height 계산 때문에 SVG paint만 빠지는가?
3. 수정은 이 내부 표가 포함된 RowBreak row에만 적용되어, Stage 76의 비중첩 행 tail 계약과
   기존 `issue_1748`, `issue_2308`, `issue_nested_table_border`를 변경하지 않는가?

## 방법과 완료 조건

- 동일 release-test binary로 p35 SVG와 render tree를 검토하고 PDF 180 DPI raster·bbox와 대조한다.
- control 존재 → layout height/clip → SVG paint 순서로 최초 소실 지점을 확인한다. 외부 표의
  row height 또는 전역 cell padding을 추측으로 바꾸지 않는다.
- 확정 원인마다 focused regression과 p35--p36 direct visual sweep을 남긴다. 기존 Stage 76
  continuation regression 및 overflow-cell baseline도 다시 실행한다.
- 다음 단계로 넘어가기 전 분석·증적·코드를 한 커밋으로 남긴다.

## 확정 분석

`dump --section 0 --para 347`에서 outer row 6, column 2의 두 번째 문단은 2×3 내부 표를
control로 보유한다. 기준 PDF는 이를 p35에 그린다.

- 정상 비교 사례(p2): 동일한 outer row 형식의 control 문단은 `LINE_SEG vpos=3520,
  line_height=9709`을 보유하며, render tree에 nested `Table` node가 남는다.
- 결함 사례(p35): `피규제자(볼 리프트 등 사용 사업장)` 다음의 control-only 문단은
  `controls=1`, `text_len=0`, **`line_segs=0`**이다. p35 render tree의 outer cell
  `(row=6,col=2)`는 첫 텍스트 `TextLine` 하나만 가지며 nested `Table` node가 없다.
- `RHWP_DIAG_CELLPARA=1` trace도 해당 cell의 `cp=0`만 기록하고 control-only `cp=1`은
  기록하지 않는다. `layout_partial_table_resolved`가 `start_line >= end_line`일 때
  control dispatch 전에 `continue`하기 때문이다.

이는 pagination owner나 SVG clip 문제가 아니라 **partial-table renderer의 line-seg 없는
control-only nested table skip**이다. RowBreak fragment cut이 없는 경우에만 control을
살려 normal-table path와 동일하게 배치하고, source cut이 있는 fragment에서는 기존
unit-range 소유 판정을 유지해야 한다.

## 구현 결과와 PDF 대조

`layout_partial_table_resolved`의 early-skip에 `uncut_control_only_nested_table` 예외를
추가했다. 조건은 (1) source row cut 없음, (2) 문단이 공백뿐, (3) `Control::Table` 존재다.
따라서 결함 HWP5의 `(0,0)` composed line만 구제하며, split fragment의 unit ownership과
비인라인 control 규칙은 바꾸지 않는다.

- p35 render tree에는 outer table 아래 2×3 child `Table` node가 생겼고, `인원수 또는 규모`,
  `피규제자`, `약 200개`가 모두 있다.
- 180 DPI PDF direct comparison에서 p35의 `8.피규제집단 및 이해관계자` 내부 표가
  RHWP에도 나타난다. p36의 continuation owner는 Stage 76 결과를 유지한다.
- 이 수정은 p35--p36을 pixel-identical로 만들었다고 주장하지 않는다. 표 내부의 font paint,
  text spacing 등 남아 있는 ink 차이는 별도 fidelity 항목이다.

증적: [p35 review](../pr/assets/task_m100_3820_stage77_76076_p35_nested_table_fragment/review_035.png),
[p36 review](../pr/assets/task_m100_3820_stage77_76076_p35_nested_table_fragment/review_036.png),
[p35 side-by-side](../pr/assets/task_m100_3820_stage77_76076_p35_nested_table_fragment/compare_035.png),
[p36 side-by-side](../pr/assets/task_m100_3820_stage77_76076_p35_nested_table_fragment/compare_036.png),
[summary](../pr/assets/task_m100_3820_stage77_76076_p35_nested_table_fragment/summary.json).

## 완료 검증

- `cargo test --profile release-test --test issue_3820_rowbreak_rowspan_band -- --nocapture` — 2 passed.
- `cargo test --profile release-test --test issue_1748_rowbreak_straddle_rowspan --test issue_2308_render_normalized_derived_state --test issue_nested_table_border -- --nocapture` — 9 passed.
- `cargo test --profile release-test --tests` — exit 0. `overflow_cell_lines_do_not_grow`를 포함한 전체 integration 회귀가 통과했다.
- `cargo clippy --profile release-test --all-targets -- -D warnings`, `cargo fmt --check`, `git diff --check` — 모두 통과.
