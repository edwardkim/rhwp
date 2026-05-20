# Stage 2 완료보고서 — #1022: 정합 설계

- 타스크: #1022 / 브랜치 `local/task1022`
- 작성일: 2026-05-20
- 단계: Stage 2 — 측정 시스템 정합 설계 확정

## 1. 후보안 평가

Stage 1 차이 8건 중 일치 항목(1,3,4)을 제외한 5건(2,5,6,7,8)을 정합한다. 세 후보:

| 후보 | 방향 | 변경 범위 | 위험 |
|------|------|-----------|------|
| (a) C → A | `cell_units` / `row_cut_content_height` 를 `MeasuredCell` 산식에 맞춤 | `table_layout.rs` 의 cut 함수들만 | 가장 작음 — 베이스라인 LAYOUT_OVERFLOW 0 직접 복원 |
| (b) A → C | `MeasuredCell` 를 `cell_units` 산식으로 교체 | `height_measurer.rs` 광범위 + 의존부 | 큼 — HeightMeasurer 변경이 모든 페이지네이션·렌더링에 파급 |
| (c) 공통 보조 함수 | 양쪽이 호출하는 셀 콘텐츠 측정 헬퍼 추출 | 두 파일 모두 | 중 — 깔끔하나 작업량 큼, task993 cut 의미 보존 검토 필요 |

**선택: (a) C → A.** 변경 최소·베이스라인 동치 복원 직접·task993 cut 디스크립터/split 의미 보존.

## 2. (a) 적용 세부 — cell_units / row_cut_content_height 정합

### 2-1. 줄별 trailing line_spacing 규칙 (차이 #2)

현재 C:
```
is_cell_last_line = is_last_para && li+1 == line_count
lh = if !is_cell_last_line { h + ls } else { h }
```

정합 후 (A 와 동일):
```
include_trailing_ls = !is_cell_last_line || cell_para_count > 1
lh = if include_trailing_ls { h + ls } else { h }
```

영향: 다중 문단 셀의 마지막 줄에서 ls 1개를 가산. `nested/empty atom` 의 `line_based_h` 도 동일.

### 2-2. 비인라인 controls 가산 (차이 #6)

`cell_units` 마지막에 셀의 `non_inline_controls_height`(=`HeightMeasurer::measure_non_inline_controls_height`) 만큼 가산 필요. 비인라인 Picture/Shape 의 height. `cell_units` 의 유닛 시퀀스에는 자연스러운 위치가 없으므로, 별도 필러 유닛(가시 줄 없음, 0~16px 단위로 쪼개 단위 분할 가능)으로 부착.

다만 비인라인 컨트롤은 LINE_SEG 에 미포함되어 분할 어려움 — 이전 task993 의 `cell.height` 필러와 동일 방식으로 처리 (atomic-ish, 셀 콘텐츠 끝에 부착).

### 2-3. 중첩 표 셀 content_height (차이 #5)

현재 C 의 nested atom: `nested_h.max(line_based_h)` (calc_nested_table_height 합).

A: `max(last_seg_end_vpos_px, text_height)`. LINE_SEG 의 절대 vpos+lh 가 권위.

정합: `cell_units` 의 nested 보유 셀에 대해 `last_seg_end_vpos_px` 도 후보로 `max(nested_h, line_based, last_seg_end_vpos_px)` 비교. 가장 작은 변경.

### 2-4. cell.height 처리 (차이 #7) + 행 단계 padding (차이 #8)

현재 C:
```
cell_units sum + filler (to cell.height - pad_cell) 후,
row_cut_content_height = max over cells of cell_units_sum (with filler)
row total in walk = row_cut_content_height + max_padding_for_row
```

이 식은 `max_over_cells(max(content, cell.height-pad_cell)) + max_pad`. A 와 다르다.

정합 후 (A 와 동일):
```
per cell required_h = if pad_total > 0.5*cell.height && content <= cell.height: cell.height
                       else: content + pad_total_cell
row_height = max over cells of max(cell.height, required_h)
            = max over cells of max(cell.height, content+pad_cell)
            (특수 케이스로 압축)
```

구체 변경:
- `cell_units` 의 `cell.height` 필러 제거. cell_units 는 순수 콘텐츠(+ 비인라인 controls + 중첩 표) 측정만 담당.
- `row_cut_content_height` (`(table, row, start_cut, end_cut, styles) -> f64`) 의 의미를 "행의 가시 콘텐츠 높이(패딩 포함)" 로 변경 — 셀별로 `cell.height` 와 `content+pad` 의 max, 그 행 max.
- `advance_row_cut` 의 split 결정에서 `consumed_height` 도 동일한 셀별 max 식 — 단 split 시 cell.height 는 분할 무관(콘텐츠가 짧으면 cell.height 도 짧아짐, A 의 required_height 식 그대로 적용).

### 2-5. 페이지네이터·렌더러 정합

위 정합 후:
- `cut_row_h[r]` = `advance_row_cut(r,&[],MAX).consumed_height + max_padding_for_row(r)` — 이제 A 의 `mt.row_heights[r]` 와 동일 px 값.
- 렌더러 2b 오버라이드의 `row_cut_content_height` 도 같은 식.
- paginator·renderer 양쪽이 새 측정으로 정합 → 베이스라인 LAYOUT_OVERFLOW 0 복원.

## 3. task993 invariants 보존

- `RowCut` 스키마(`start_cut`/`end_cut: Vec<usize>`) 유지.
- `advance_row_cut` 의 vpos 리셋 = hard break, intra-row 분할 단일 권위 — 보존.
- `cell_line_ranges_from_cut` 유닛↔줄 매핑 — 보존(줄별 유닛 매핑은 유지, 줄 높이 식만 정합).
- form-002 vpos 리셋 분할 정합 — 줄 매핑 보존 + 줄 높이 정합 → form-002 PDF 정합 유지.

## 4. Stage 3 구현 진행 항목

1. `cell_units`:
   - `is_cell_last_line` ls 제외 조건 변경(#2).
   - `cell.height` 필러 제거(#7,#8 의 일부).
   - 중첩 표 셀의 `last_seg_end_vpos` 가산(#5).
   - `non_inline_controls_height` 필러 가산(#6).
2. `row_cut_content_height`:
   - 셀별 `max(cell.height, content+pad_cell)` 의 행 max 로 재작성(#7,#8).
3. 페이지네이터 walk:
   - `consumed += cs_before + row_total`(row_total = row_cut_content_height(r, &[], &[])). 분할 행은 `consumed += cs_before + res.consumed_height + max(cell.height_part, ...) ` — 분할 시는 cell.height max 적용 안 함(분할 컷이 권위).
4. 렌더러 2b 오버라이드:
   - 분할 행만 (start_cut/end_cut 비어 있지 않으면). 비분할 행은 resolve_row_heights 유지 — A 와 동일하므로 cut 측정으로도 같은 값이 나와야 함(정합 후 일치 보장).

## 5. 검증 (Stage 4 미리보기)

- 비공개 184페이지 샘플 `LAYOUT_OVERFLOW` 0건.
- 페이지 22 18.3px 본문 초과 해소.
- form-002 한컴 2022 PDF 정합 유지.
- `cargo test`/`clippy` 회귀 0.
- 공개 분할 표 골든 SVG 변동 시 PDF 권위 자료 대조 판정.

## 6. 검증

설계 단계 — 소스 변경 없음.
