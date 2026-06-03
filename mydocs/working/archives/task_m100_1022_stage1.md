# Stage 1 완료보고서 — #1022: 측정 알고리즘 비교 조사

- 타스크: #1022 (M100, v1.0.0) / 브랜치 `local/task1022`
- 작성일: 2026-05-20

## 1. 두 측정 경로

### A. `HeightMeasurer::measure_table_impl` (`height_measurer.rs:494~`)

행마다 `row_heights[r]` 산출. 2단계:

**단계 1** — `row_span==1` 셀별로 `cell.height` 의 max:
```
row_heights[r] = max over row_span==1 cells in row r of cell.height(px)
```

**단계 2** — 셀 콘텐츠 측정 후 `required_height` 갱신:
```
per cell (row_span==1):
  text_height = sum over paragraphs of:
    if comp.lines empty: spacing_before + 400HU + spacing_after
    else: spacing_before
        + sum over lines of [
            corrected_line_height(raw_lh, max_fs, ls_type, ls_val)
            + (line_spacing if !is_cell_last_line || cell_para_count > 1 else 0)
          ]
        + spacing_after
  content_height = if has_nested_table { max(last_seg_end_vpos, text_height) }
                   else { text_height + non_inline_controls_height }
  required_height =
    if pad_total > 0.5*cell.height && content_height <= cell.height: cell.height
    else: content_height + pad_total
  row_heights[r] = max(row_heights[r], required_height)
```

### C. `LayoutEngine::cell_units` (+ `row_cut_content_height`) (`table_layout.rs:3405~`)

셀별 유닛 시퀀스 생성, 컷 범위 사이 합:

```
per paragraph:
  if comp.lines empty or has_table_in_para:
    1 atom unit, height = nested_h.max(line_based_h)
      where line_based_h = sum over lines of [
          corrected_h(line)
          + (line_spacing if !is_cell_last_line else 0)
        ] + spacing_before(li==0) + spacing_after(li==line_count-1)
  else (text):
    per-line units, each:
      height = corrected_h(line)
             + (line_spacing if !is_cell_last_line else 0)
             + spacing_before(li==0) + spacing_after(li==line_count-1)

content_sum = sum of unit heights
content_area = cell.height - pad_top - pad_bottom
if content_area > content_sum + 0.5:
  append filler units totaling (content_area - content_sum), ~16px chunks

row_cut_content_height(row, &[], &[]) =
  max over row_span==1 cells of (cell_units sum, all units)

row total in walk: row_cut_content_height + max_padding_for_row
  where max_padding_for_row = max over row_span==1 cells of (pad_top + pad_bottom)
```

## 2. 차이 분석

| # | 항목 | A (HeightMeasurer) | C (cell_units) | 일치 |
|---|------|-------------------|----------------|------|
| 1 | 줄별 corrected_line_height | ✓ | ✓ | 일치 |
| 2 | `is_cell_last_line` trailing line_spacing 제외 | 단일 문단·단일 줄 셀만 제외 (`is_cell_last_line && cell_para_count == 1`) | 모든 cell-last-line 제외 | **불일치** — 다중 문단 셀 마지막 줄에서 A 가 ls 1개 더 큼 |
| 3 | spacing_before / spacing_after | 첫 문단 sb 제외, 마지막 문단 sa 제외 | 동일 | 일치 |
| 4 | 빈 문단 높이 | `sb + 400HU + sa` | 동일 | 일치 |
| 5 | 중첩 표 셀 content_height | `max(last_seg_end_vpos, text_height)` (LINE_SEG vpos 절대값) | `nested_h.max(line_based_h)` (`calc_nested_table_height` 합) | **불일치** — A 는 LINE_SEG 절대 vpos, C 는 nested 재측정 |
| 6 | 비인라인 controls (Picture, Shape) 별도 높이 | `+ measure_non_inline_controls_height` | 없음 | **불일치** — C 가 누락 |
| 7 | cell.height 처리 | row 단계 1 에서 `row_heights = max(cell.height)`, 단계 2 에서 `required_height` 갱신 (특수 케이스: pad>0.5h && content≤h → cell.height) | 셀별 filler 로 `max(content_sum, cell.height-pad)` 산출 | **불일치(구조)** — 동일 의도지만 paddinghouse가 다름 |
| 8 | row 단계 padding 합산 | 셀별 `content_height + pad_total_cell` 그대로 행 max 비교 | 셀별 content_sum 의 행 max + `max_padding_for_row`(별도 row-level 최대) | **불일치(중대)** — 셀마다 pad 가 다르면 결과 달라짐 |

## 3. 행 높이 산출식 비교

A 의 행 높이:
```
row_h = max_over_cells( max(cell.height, content_height + pad_total_cell) )
```

C 의 행 높이:
```
row_h = max_over_cells( max(content_sum, cell.height - pad_total_cell) )
      + max_over_cells( pad_total_cell )
```

같은 셀 콘텐츠에서 두 식이 어긋난다 — 셀마다 패딩이 다른 경우, A 는 셀별 (content + pad) 의 행 max, C 는 행별 content max + 행별 pad max. 행에 패딩 큰 셀과 콘텐츠 큰 셀이 있을 때 차이 발생.

## 4. 데이터 가설 — 페이지 22 pi=226 row 0

- cell[0]: pad_total = 282 HU = 3.76px, cell.height = 1643 HU = 21.94px, 콘텐츠 = "요구사항번호" 1줄 ≈ 14.67px.
- cell[1]: pad_total = 282 HU = 3.76px, cell.height = 1643 HU = 21.94px, 콘텐츠 = " SFR-013" 1줄 ≈ 14.67px (다른 폰트라면 다를 수 있음).
- A 행: max(max(21.94, 14.67+3.76)=21.94, max(21.94, ...)=21.94) = 21.94.
- C 행: max(max(14.67, 21.94-3.76)=18.18, ...)=18.18; + max_padding=3.76; = 21.94.

이 케이스에서 A=C=21.94 — 일치. 따라서 pi=226 row 0 의 18.3px 회귀는 row 0 자체가 아니라 **누적 drift** 에서 발생. 차이 #2/#6/#8 누적이 50건 회귀를 일으키는 부류로 보이며, Stage 2 에서 어떤 차이가 회귀에 결정적인지 핀포인트한다.

## 5. Stage 2 방향

기본안 (a) `C → A` 재구성:

- 차이 #2: C 의 trailing line_spacing 규칙을 A 와 일치 (`!is_cell_last_line || cell_para_count > 1`).
- 차이 #6: `cell_units` 끝에 `measure_non_inline_controls_height` 동치 가산.
- 차이 #8: `row_cut_content_height` 를 per-cell `max(cell.height, content+pad_cell)` 의 행 max 로 재구성 (행 단계 padding 합산 일치).
- 차이 #5: 중첩 표 셀 — `cell_units` 의 nested atom 높이를 `last_seg_end` 기반으로 (단 nested 재측정과의 정합 필요 — Stage 2 에서 후속 확정).
- 차이 #7: cell.height 처리 통일 — filler 제거하고 row aggregation 에서 `max(content+pad, cell.height)` 로 정합.

검증: 측정 통일 후 `LAYOUT_OVERFLOW` 0건 복원 + form-002 PDF 정합 + task993 컷 디스크립터·split 의미 보존.

## 6. 검증

조사 단계 — 소스 변경 없음. `cargo build` 영향 없음.
