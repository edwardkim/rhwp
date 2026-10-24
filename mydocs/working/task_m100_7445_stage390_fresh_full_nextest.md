# #7445 현재 빌드의 전체 회귀 결과

## 실행 근거와 범위

- 대상: `fix/7445-completed-20261009`, 코드 `072cb1e437677662e5be854bbd6a70be46bf86d4`.
- fetch 후 기준: `upstream/devel` `21ed64071f81f35f37618717693bd30f727392c9`, 뒤처진 커밋 0개.
- 이전 공유 산출물의 삭제된 ready 경로를 제거한 새 바이너리에서 전체를 실행했다. 시험지 입력 제외와 현재 함수 호출 정합은 앞 단계 focused 47 PASS로 검증했다.
- 명령: `cargo nextest run --locked --cargo-profile release-test --target-dir /Users/tsjang/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast`.
- 검사 중 head와 추적 파일 내용은 변경하지 않았다. 공유 Cargo 작업은 순차 실행했다.

## 실제 결과

- **10,285개 실행: 10,249 PASS / 36 FAIL / 45 skip**. 실행 491.838초, 빌드 포함 728.513초, 종료 코드 100.
- manifest 준비와 fmt가 통과했다. 실패 36개는 중복된 최종 실패 목록을 제외한 검사별 수다.
- 36개 실패 블록에서 파일 경로 오류는 0개다. 이전 169 FAIL을 현재 head의 회귀 수로 쓰지 않는다.
- 아래 실패는 아직 해결되지 않았다. baseline·기대값 갱신 또는 삭제를 통과 수단으로 적용하지 않았다. 전체 결과가 실패이므로 PR 제출 게이트는 미완료다.

| 순서 | 실패한 기존 검사 |
| --- | --- |
| 1 | `body_overflow_baseline::body_overflow_does_not_grow_partition_10` |
| 2 | `body_overflow_baseline::body_overflow_does_not_grow_partition_4` |
| 3 | `body_overflow_baseline::body_overflow_does_not_grow_partition_7` |
| 4 | `body_overflow_baseline::body_overflow_does_not_grow_partition_9` |
| 5 | `issue_6761_stored_vpos_rewind_page_break::partial_retreat_inside_a_page_does_not_break` |
| 6 | `issue_7226_rowspan_only_row_cut::the_fix_does_not_add_a_page` |
| 7 | `issue_6034_footnote_area_reflow_width::issue_6034_footnote_area_starts_below_body_text` |
| 8 | `issue_7413_cell_padding_shrinks_only_when_wrapping_overflows::b_cell_that_cannot_wrap_within_its_height_still_shrinks` |
| 9 | `issue_2006_1790387_prep_pagination_pin::prep_1790387_page_count_pin` |
| 10 | `issue_7445_biotoxin_source_ownership::cellbreak_boundary_rows_belong_wholly_to_their_pdf_pages` |
| 11 | `issue_7203_float_table_stored_anchor_top::stored_anchor_after_a_taller_float_consumes_its_span_once` |
| 12 | `issue_7203_float_table_stored_anchor_top::downstream_table_split_and_body_follow_the_oracle_pages` |
| 13 | `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition_14` |
| 14 | `issue_6981_split_straddle_row_height::completed_terminal_cut_does_not_allocate_an_empty_page` |
| 15 | `issue_6981_split_straddle_row_height::continuation_height_respects_varying_page_budgets` |
| 16 | `issue_rowbreak_chart_overlap::rowbreak_hwpx_page8_keeps_continued_nested_reference_line` |
| 17 | `issue_5756_superscript_advance_scale::issue_5756_superscript_line_stays_inside_cell` |
| 18 | `issue_7390_prep_table_rowbreak::prep_page_105_merged_budget_table_keeps_saved_row_boundaries` |
| 19 | `issue_7379_rowbreak_table_footnote_reservation::terminal_caption_withholds_the_last_rowspan_unit_together` |
| 20 | `issue_6035_cell_split_empty_band::issue_6035_does_not_emit_header_only_ghost_page` |
| 21 | `issue_6035_cell_split_empty_band::issue_6035_water_supply_subitems_stay_with_heading` |
| 22 | `issue_1692::issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow` |
| 23 | `issue_1156_rowbreak_fragment_fit::kps_ai_page37_defers_overflowing_split_row_slice` |
| 24 | `issue_4514_overlay_table_flow::issue_4514_overlay_tables_do_not_overlap` |
| 25 | `issue_6797_table_item_clears_float_band::table_item_clears_the_previous_float_band` |
| 26 | `text_overlap_baseline::text_overlaps_do_not_grow_partition_10` |
| 27 | `text_overlap_baseline::text_overlaps_do_not_grow_partition_15` |
| 28 | `text_overlap_baseline::text_overlaps_do_not_grow_partition_6` |
| 29 | `text_overlap_baseline::text_overlaps_do_not_grow_partition_4` |
| 30 | `issue_7203_split_float_anchors_to_paragraph_top::split_fragment_without_a_gap_is_unchanged` |
| 31 | `stored_table_text_tail::hancom_saved_tail_preserves_pdf_baseline_after_the_table` |
| 32 | `issue_7203_float_table_top_uses_stored_anchor::saved_outer_box_anchor_keeps_following_tables_inside_the_body` |
| 33 | `issue_7390_kopub_latin_metrics::kopub_latin_line_width_matches_hancom_in_absolute_px` |
| 34 | `issue_7390_kopub_latin_metrics::kopub_space_stays_half_width` |
| 35 | `issue_6812_square_picture_tac_table::issue_6812_following_text_respects_column_bottom_after_multiple_pictures` |
| 36 | `text_overlap_baseline::text_overlaps_do_not_grow_partition_11` |

## 다음 처리

- 첫 실패는 body-overflow partition 10의 `hwp3-sample4-hwp5.hwp` 증가(4 → 6건)다. 같은 분할의 다른 문서는 증가하지 않았다.
- 해당 입력은 시험지가 아닌 36쪽 리눅스 어셈블리 문서다. 원본·저장 버전·독립 Print PDF와 기준 SHA의 출력을 먼저 확인한 뒤, 본문 넘침이 실제 결함인지 판별한다.
- 실물 검사 변경은 전체 페이지 Native/fresh WASM 90%와 확대 판독을 먼저 충족한다. 기준 head가 미검증이면 회귀 삭제·이관 판정을 하지 않는다.
- 정책연구 215쪽의 기존 208쪽 미달도 유지한다. 현재 head의 전체 시각 검증, 필수 lint·Skia·fresh WASM, PR/CI/merge/후속 처리는 아직 완료하지 않았다.

## 증거 위치

- `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/stage390-fresh-full-nextest/full-nextest.log`
- 같은 디렉터리의 `run-state.json`, `validation-results.json`, `failure-inventory.json`, `failures/01.txt`부터 `36.txt`.
- 로그 SHA-256: `6668ae16df4ef7e3c42b46169483e60a3f5a7a810391f9f3cd5571b838fbd651`.
- 위 로그·중간 JSON은 ignored output에만 보존하며 커밋하지 않는다.
