# #7445 완료 후보 전체 회귀 결과와 수정 판정

- Issue: #7445
- 작업 브랜치: `fix/7445-completed-20261009`
- 검증 head: `68c952e1bd0878c98283d8ee12349c17aad82411`
- Rust 코드 후보: `7589a9323b92ea63a2575f9c943d8fcc6c213be8`; 두 head 사이 Rust 소스·테스트·Cargo 입력 차이는 없다.
- 고정 base: `21ed64071f81f35f37618717693bd30f727392c9`

## 전체 실행 결과

10281개 실행, **10254 PASS / 27 FAIL / 45 skip**, slow 8개, 552.682초, 종료 코드 100.
`--locked --cargo-profile release-test --tests --test-threads 8 --no-fail-fast`, 빌드 작업 2개를 사용했다.
전체 실행을 중단하지 않았으며 명시적인 최종 Summary와 종료 코드를 확인했다.
로그·TSV·중간 JSON은 ignored output에 보존하고 커밋하지 않는다.

- 로그: `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/stage426-full-nextest/nextest.log`
- 실행 요약: validation.json（로컬 중간 자료 `validation.json`）
- 원본 실패 블록: `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/stage426-full-nextest/failures.txt`

## 판정 순서

1. 실패한 실제 문서를 동일한 독립 한컴 Print PDF·글꼴·배율로 base/head 대조한다.
2. 현재 보정한 문서는 전쪽 Native/fresh WASM 증거를 확인한 뒤, 잘못된 좌표·쪽수 핀만 소속·내용 보존·포함·겹침 관계로 교정한다.
3. 정확한 검사이거나 현재 변경으로 새 결함이 생겼으면 현재 브랜치의 구현을 고친다.
4. base부터 시각 미달인 부적합한 검사는 같은 원본·미해결 증상의 기존 이슈와 담당 범위를 조회한다.
   사용자는 필요한 경우 #7445 서브 이슈 등록을 승인했다. 이미 연결된 하위 이슈는 중복 등록하지 않는다.
5. 시각 근거 없이 기대값을 현재 수치로 복사하거나 허용치를 넓히지 않는다. 원본 문서는 보존한다.

## 실패 27개

| 검사 | 실제 입력 | 관측 실패 |
| --- | --- | --- |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_10` | `실패 로그의 증가 문서를 개별 대조` | 보이는 글자끼리 겹치는 사건(layout-anomaly text-overlap)이 늘었다. |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_2` | `실패 로그의 증가 문서를 개별 대조` | 보이는 글자끼리 겹치는 사건(layout-anomaly text-overlap)이 늘었다. |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_15` | `실패 로그의 증가 문서를 개별 대조` | 보이는 글자끼리 겹치는 사건(layout-anomaly text-overlap)이 늘었다. |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_7` | `실패 로그의 증가 문서를 개별 대조` | 보이는 글자끼리 겹치는 사건(layout-anomaly text-overlap)이 늘었다. |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_4` | `실패 로그의 증가 문서를 개별 대조` | 보이는 글자끼리 겹치는 사건(layout-anomaly text-overlap)이 늘었다. |
| `issue_7203_split_float_anchors_to_paragraph_top::split_fragment_without_a_gap_is_unchanged` | `samples/hwpctl_API_v2.4.hwp` | 34쪽에서 문단 797 의 표를 찾지 못했다 |
| `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition_14` | `실패 로그의 증가 문서를 개별 대조` | 한글이 뽑은 쪽수와의 격차가 커졌다. |
| `issue_7413_cell_padding_shrinks_only_when_wrapping_overflows::b_cell_that_cannot_wrap_within_its_height_still_shrinks` | `samples/hwpx/table-text.hwpx`<br>`samples/issue5699/20099369_yeongwol_forms.hwp` | assertion `left == right` failed: 정답지 전제가 깨졌다 — 대상 칸(y=865.4, w=41.9)을 하나로 못 집었다. 쪽 구성이          바뀌었으면 이 검사의 좌표부터 다시 정해야 한다. 후보=[] |
| `issue_7390_kopub_latin_metrics::kopub_space_stays_half_width` | `samples/issue2006/1790387_prep_final_report.hwpx` | 108쪽 대상 줄 |
| `issue_7390_kopub_latin_metrics::kopub_latin_line_width_matches_hancom_in_absolute_px` | `samples/issue2006/1790387_prep_final_report.hwpx` | 108쪽의 `withmenthroughthenetworkscale-upmethodin` 줄 |
| `issue_7445_biotoxin_source_ownership::cellbreak_boundary_rows_belong_wholly_to_their_pdf_pages` | `samples/issue5941/1480000-201900698-native-neartop-reset.hwp` | assertion `left == right` failed: 경계 행의 셀은 같은 쪽에 완전히 소속 |
| `issue_2006_1790387_prep_pagination_pin::prep_1790387_page_count_pin` | `samples/issue2006/1790387_prep_final_report.hwpx` | assertion `left == right` failed: issue2006 1790387 동일 원본 한컴2024 PDF 140쪽과 달라짐: 실제 141쪽 |
| `issue_7203_float_table_stored_anchor_top::stored_anchor_after_a_taller_float_consumes_its_span_once` | `samples/hwpctl_API_v2.4.hwp`<br>`samples/issue6111/56345_regulatory_impact_analysis.hwp`<br>`samples/rowbreak-problem-pages.hwp` | 저장 간격과 뒤 표 위여백 34.31px 대신 26.76px를 소비했다 |
| `issue_7203_float_table_stored_anchor_top::downstream_table_split_and_body_follow_the_oracle_pages` | `samples/hwpctl_API_v2.4.hwp`<br>`samples/issue6111/56345_regulatory_impact_analysis.hwp`<br>`samples/rowbreak-problem-pages.hwp` | 16쪽에 첫 데이터 행을 미리 소비했다 |
| `issue_6981_split_straddle_row_height::completed_terminal_cut_does_not_allocate_an_empty_page` | `samples/issue6795/1341000-201100013-cyber-university-application.hwp`<br>`samples/issue6803/1376496-neighborhood-facility-land-table.hwp`<br>`samples/task2287/1342000_edu_curriculum_map.hwp` | next section must follow the completed table |
| `issue_6981_split_straddle_row_height::continuation_height_respects_varying_page_budgets` | `samples/issue6795/1341000-201100013-cyber-university-application.hwp`<br>`samples/issue6803/1376496-neighborhood-facility-land-table.hwp`<br>`samples/task2287/1342000_edu_curriculum_map.hwp` | table bottom 700.9066666666668 exceeds body bottom 700.1333333333334 |
| `issue_5877_fragment_ghost_vrules::issue_5877_gas_rowspan_keeps_label_owner_and_next_row_minimum` | `samples/issue5877/fragment_ghost_vrules.hwp` | assertion `left == right` failed: 정본 전체 쪽 소유를 보존해야 한다 |
| `issue_rowbreak_chart_overlap::rowbreak_hwpx_page8_keeps_continued_nested_reference_line` | `samples/rowbreak-problem-pages.hwp`<br>`samples/rowbreak-problem-pages.hwpx` | HWPX page 8 continued line is clipped above the cell: line_top=80.89, cell_top=98.24 |
| `issue_5756_superscript_advance_scale::issue_5756_superscript_line_stays_inside_cell` | `samples/issue5756/156732409_superscript_advance.hwp` | 위첨자 글리프(0.7배×√r)가 있어야 한다 |
| `issue_7390_prep_table_rowbreak::prep_page_105_merged_budget_table_keeps_saved_row_boundaries` | `samples/issue2006/1790387_prep_final_report.hwpx` | 해당 행의 세로 병합 없는 셀 |
| `body_overflow_baseline::body_overflow_does_not_grow_partition_5` | `실패 로그의 증가 문서를 개별 대조` | 본문 바닥 아래에 그려지는 요소가 늘었다(layout-anomaly overflow 의 over_bottom, 공차 2px). |
| `issue_1692::issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow` | `samples/SO-SUEOP.hwp`<br>`samples/SO-SUEOP.hwpx` | HWP3 p22 first body y=337 must match HWPX y=339.7 |
| `issue_7203_float_table_top_uses_stored_anchor::saved_outer_box_anchor_keeps_following_tables_inside_the_body` | `samples/hwpctl_API_v2.4.hwp`<br>`samples/issue6111/56345_regulatory_impact_analysis.hwp` | stored flow origin: actual=663.8933333333337, expected=656.3466666666667 |
| `issue_1156_rowbreak_fragment_fit::kps_ai_page37_defers_overflowing_split_row_slice` | `samples/kps-ai.hwp`<br>`samples/synam-001.hwp` | page 37 should end at the last fully fitting row: |
| `issue_6797_table_item_clears_float_band::table_item_clears_the_previous_float_band` | `samples/issue6797/156160455-social-pig-farm-income.hwp`<br>`samples/synam-001.hwp` | pi=71 표는 pi=70 자리차지 표 바닥 아래에서 시작해야 한다 — #6797 회귀          (순서 부족분 124.9px; pi=70 181.5..294.9, pi=71 170.0..335.7;           수정 전 부족분 120.1px · 실제 상자 교집합 113.4px) |
| `issue_4514_overlay_table_flow::issue_4514_overlay_tables_do_not_overlap` | `samples/issue4514/sample1-repro.hwp` | page 9 에서 최상위 표 겹침이 재발했다: [(139, 158, 4.426666666666847)] |
| `issue_6812_square_picture_tac_table::issue_6812_following_text_respects_column_bottom_after_multiple_pictures` | `samples/issue6797/156160455-social-pig-farm-income.hwp` | 그림 뒤 문단 줄의 실제 하단은 단 안에 있어야 한다: BoundingBox { x: 115.58666666666667, y: 1026.7866666666669, width: 602.5333333333334, height: 20.0 }, column_bottom=1043.1466666666668 |

## 현재 확인한 원인 경계

겹침 증가 6개 입력은 자산관리 서명 보정 직전 바이너리와 현재 바이너리가 같은 이상탐지 결과다.
이 사실은 서명 보정으로 새로 생긴 결함이 아님을 설명하며, 기존 base 대비 증가를 정상으로 인정하는 근거는 아니다.
31쪽 가스 표의 현재 32쪽 실패는 고정 devel 바이너리에서도 32쪽이다. 전체 시각 대조와 독립 내용 소속을
확인하기 전 쪽수 검사 변경·제거를 판정하지 않는다.

PR 생성·push·병합은 전체 검증과 사용자 승인 이후 진행한다.
