# 기존 실패 27개 재검증과 남은 판정

- 검증 head: `4ddcaaeb6` (`fix/7445-completed-20261009`)
- 고정 devel: `21ed64071f81f35f37618717693bd30f727392c9`
- 재생성한 정식 integration harness에서 기존 실패 27개를 모두 개별 실행했다.
- **8 PASS / 19 FAIL / 3003 skip**, 107.940초, 종료 코드 100. 전체 nextest 실행은 아니다.
- 정본: `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/stage432-original-failures-rerun/nextest.log`

## 남은 검사

| 검사 | 상태 |
| --- | --- |
| `issue_5877_fragment_ghost_vrules::issue_5877_gas_rowspan_keeps_label_owner_and_next_row_minimum` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_5756_superscript_advance_scale::issue_5756_superscript_line_stays_inside_cell` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_7203_split_float_anchors_to_paragraph_top::split_fragment_without_a_gap_is_unchanged` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_4514_overlay_table_flow::issue_4514_overlay_tables_do_not_overlap` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_10` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_7203_float_table_stored_anchor_top::downstream_table_split_and_body_follow_the_oracle_pages` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_7445_biotoxin_source_ownership::cellbreak_boundary_rows_belong_wholly_to_their_pdf_pages` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_rowbreak_chart_overlap::rowbreak_hwpx_page8_keeps_continued_nested_reference_line` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_1692::issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_6797_table_item_clears_float_band::table_item_clears_the_previous_float_band` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_7413_cell_padding_shrinks_only_when_wrapping_overflows::b_cell_that_cannot_wrap_within_its_height_still_shrinks` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_1156_rowbreak_fragment_fit::kps_ai_page37_defers_overflowing_split_row_slice` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_6812_square_picture_tac_table::issue_6812_following_text_respects_column_bottom_after_multiple_pictures` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_2` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_6981_split_straddle_row_height::completed_terminal_cut_does_not_allocate_an_empty_page` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_15` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `issue_6981_split_straddle_row_height::continuation_height_respects_varying_page_budgets` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_4` | 기존 base·독립 PDF·현재 출력 판정 진행 |
| `body_overflow_baseline::body_overflow_does_not_grow_partition_5` | 기존 base·독립 PDF·현재 출력 판정 진행 |

## 비교 조건과 진행 중 보정

- PrEP는 기본 96dpi 전쪽 비교에서 고정 devel 140쪽 최저 91.57672%, 현재 V9 140쪽 최저 91.93033%로 모두 통과한다. 192dpi 확대 자료의 7쪽 미달을 기본 조건의 기존 미달로 판단했던 #7804는 API로 본문을 정정했다. 정상 회귀를 삭제하거나 이관하지 않는다. fresh WASM 전 140쪽도 같은 기본 96dpi 조건에서 최저 91.93033%, 미달 0으로 통과했다. #7804는 기본 조건의 미달 이관 근거가 틀렸으므로 `not_planned`로 닫았으며, 192dpi 확대 자료는 진단 증거로 보존한다.
- 56345 규제영향분석은 기본 96dpi에서도 base/current 모두 21쪽 중 7·8·9·17·18쪽이 미달이다. 최저 12.81871%/12.85380%이며 #7806에서 기존 범위를 추적한다. 현재 수정으로 발생한 11·20쪽 제목 표의 원점은 구현에서 복구했고 두 기존 검사는 통과했다.
- 30269 권고 문서의 두 경로는 SHA가 같다. 96dpi base의 미달은 6·9·12쪽(최저 46.00942%), 현재는 6·12·15쪽(최저 49.32090%)이다. 이미 등록된 #7729의 기존 미달과 별개로 현재 15쪽 제목 문단 위로 TAC 표가 이동하는 겹침은 구현에서 해결한다. 지연 텍스트 사다리에서 역산한 원점만으로 개체 줄의 물리 원점을 확정하던 경로를 확인했다. 실제 Print와 대응되는 이전 본문은 같은 좌표인데 신규 inline 원점만 달랐다.
- 가스 문서 #7803은 32/31쪽 차이로 전체 TSV가 측정 불가다. 이를 90% 미만 수치로 복사하지 않는다. 원본·PDF·검사는 보존한다.
- superscript 3쪽은 독립 수동 Print PDF 대기 상태다. 기대값과 비교 허용치를 현재 출력으로 복사하지 않는다.

로그·TSV·진단 원문은 ignored output에 보존하며 새 테스트 함수·지침 변경은 없다. 원본 입력 파일을 삭제하지 않는다. 현재 보정과 이관 근거를 구분한 뒤 마지막 전체 검증과 사용자 승인을 진행한다.

## 30269 추가 보정 완료 (V10)

- 텍스트 누적 위치에서 추정한 `vpos_lazy_base`는 개체의 물리 쪽 원점을 인증하지 않는다. 저장 단일 개체 줄의 inline 원점 수용에서는 확인된 `vpos_page_base`만 소비하도록 수정했다. 저장 표의 측정 결과 → `stored_single_row_source` → inline placement → 최종 표 paint의 연결을 확인했다. 문서 ID나 좌표 상수 조건을 추가하지 않았다.
- 두 원본 경로 모두 Native 22쪽, 텍스트 겹침 0으로 복구했다. 독립 Print 15쪽의 표는 앞 본문 아래에 있으며 Native/fresh WASM 모두 98.29344%다. 6·12쪽의 기존 미달은 남아 있으며 전체 문서 시각 통과로 보고하지 않는다.
- 정식 `text_overlap_baseline` partition 4·15는 2 PASS / 0 FAIL / 186 skip, 3.931초다. 기존 19개 중 두 개를 추가로 해결했으며 나머지 17개는 후속 판정 대상이다. 최종 전체 nextest를 재실행한 결과가 아니다.
- 정상 대조군 12종·251쪽의 Native SVG와 실제 Chrome fresh WASM SVG는 V9와 전쪽 byte-identical이다. 동일 입력·PDF·글꼴·선언 DPI의 기존 raster 증거를 재사용한다. TAC 3쪽과 PrEP 140쪽의 기본 조건은 96dpi이며 192dpi 확대 점수를 기본 조건과 섞지 않는다.
- #7806 본문에 기본 96dpi에서 base/current 모두 기존 5쪽 미달이라는 근거를 완료 조건 앞에 추가했다. 11·20쪽의 현재 추가 이동은 코드에서 복구했고 정상 계약은 유지했다.

![30269 Print 15쪽 보정 검토](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/reform-review_015.png)
