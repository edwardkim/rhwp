# Task #1811 Stage 3 — HWPX table cell synthetic lineSeg split 보정

## 배경

Stage 2 에서 원인 1(합성 seg 를 저장 vpos 증거로 오인해 p4 예산이 줄어드는 문제)은
수정되었다. 그러나 `saved_bounds_cumulative_page_break.hwpx` 는 여전히 p4 의 pi52
RowBreak 표가 `end_cut=[2]` 로 끊기며, HWP/한글 2024 기준의 `end_cut=[3]` 과 달랐다.

현재 재현값:

- HWPX: `avail_for_rows=126.6`, row2 `avail=96.2`, cell units
  `31.2/37.9px`, synthetic `vpos=0` 리셋 오인 다수, `end_cut=[2]`
- HWP: `avail_for_rows=123.4`, row2 `avail=93.0`, cell units
  `30.9/24.3px`, 저장 lineSeg 기반, `end_cut=[3]`

`ir-diff` 는 0건이므로 문서 구조 비교만으로는 검출되지 않는다. table cell 내부
synthetic lineSeg 를 사용하는 row cut 측정과 visual sweep 이 필요하다.

추가 분석 결과, HWPX 의 cell paragraph 는 `<hp:linesegarray>` 를 생략하지만 다음 문서
속성은 남아 있었다.

- `TablePageBreak::RowBreak`
- 셀 `height`, `width`, `padding`
- `ParaShape.spacing_before`
- 합성 lineSeg 의 `line_height`, `line_spacing`, `segment_width`
- pi52 셀 끝의 빈 anchor lineSeg (`vertical_pos=12920`, 저장 lineSeg)
- pi57 TAC 표의 `common.height` 와 병합 셀 `height`

따라서 샘플명/페이지/임의 계수 대신 위 속성으로 부족한 synthetic lineSeg 수를 보강한다.

## 수정 방향

1. `table_layout::cell_units` 에서 synthetic lineSeg(`TAG_IMPLEMENTATION_PROPERTY`) 의
   `vpos=0` 을 저장 lineSeg 기반 hard break 근거로 사용하지 않는다.
2. HWPX RowBreak 셀의 synthetic lineSeg 는 이미 reflow 단계에서 line height/spacing 이
   계산된 값이므로, row cut 측정에서 다시 `corrected_line_height` 로 과대 보정하지 않는다.
3. HWPX RowBreak 셀에 저장 anchor lineSeg 가 있거나 TAC 표의 명시 높이가 있는 경우,
   셀 높이 안에 더 들어갈 수 있는 synthetic 줄을 문단 lineSeg 에 보강한다.
4. `recompose_for_cell_width` 는 synthetic lineSeg 가 이미 2개 이상이면 로드 단계에서
   계산된 경계를 존중하고 다시 합쳐서 재분할하지 않는다.
5. `saved_bounds_cumulative_page_break.hwpx` 의 p4 pi52 cut 이 HWP 기준과 같이
   `end_cut=[3]` 이 되는지 확인하고, 기존 #1749 페이지 수/쪽나누기 게이트를 유지한다.

## 포함 변경

- 코드: `src/document_core/commands/document.rs`
- 코드: `src/renderer/composer.rs`
- 코드: `src/renderer/layout/table_layout.rs`
- 회귀 테스트: #1811 의 pi52 split cut parity, pi57 TAC RowBreak synthetic lineSeg 보강
- 문서: 이전 단계에서 보정한 `mydocs/manual/pr_review_workflow.md` 를 이번 변경 범위에 함께 포함

## 현재 검증 메모

- focused test: `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1749_saved_bounds_page_break -- --nocapture` 통과
- `git diff --check` 통과
- HWP/HWPX `dump-pages -p 3/-p 4`: p4/p5 paragraph 배치와 pi52 `end_cut=[3]`,
  p5 `start_cut=[3]` 이 서로 맞는다.
- HWP/HWPX `render-diff -p 4`: 페이지 수 5/5, 구조 불일치 0건. 남은 `OVER` 는 pi52/pi57
  내부 별표 줄폭과 하단 overflow 차이에서 발생한다.
- 한글 2024 PDF 기준 visual sweep 은 아직 p4 `line/tail/large`, p5 `column/tail` 후보가
  남는다. 특히 p4 PDF 는 pi52 본문(`사회기여`)을 같은 페이지에서 더 소비하지만, 현재 rhwp 는
  pi52 내부 mixed nested table 조각을 먼저 배치해 본문 일부가 p5 로 넘어간다. 다음 스테이지에서는
  `본문 텍스트 + 중첩 표`가 함께 있는 RowBreak 셀의 분할 표시 순서를 문서 control 위치와 cut unit
  근거로 다시 맞춘다.
