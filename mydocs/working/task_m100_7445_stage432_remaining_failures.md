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

## PrEP 전쪽 통과 문서의 관계 검사 교정

- 기본 96dpi Native/fresh WASM 전 140쪽의 최저 91.93033%, 미달 0과 대응 20쪽의 직접 review 판독을 선행했다. V10 전쪽 SVG가 동일하므로 동일 원본·PDF·글꼴·DPI 증거를 재사용한다. 확대 192dpi 결과로 기본 조건을 바꾸지 않는다.
- 기존 세 파일의 검사 20개를 유지했다. 절대 잉크폭·줄 폭 하한과 x/y 핀을 줄·셀 내부 포함, 실제 소유 문단의 커서 문자 전진 비율, 캡션·본문·표 순서, 원본 문단 여백의 가운데 정렬 관계와 저장 줄 사이 간격으로 교정했다. 소스 메트릭의 크기·간격 계약은 유지하며 이전 px 상수를 비율로 옮긴 것이 아니다.
- 140쪽 대조는 독립 Print에서 확인한 쪽수와 표지·마지막 주의 사항 보존, 연구비 표의 105쪽 단독 소유를 함께 검사한다. 105쪽의 금액 열 20행은 한 번씩 이어져야 하고 다음 쪽에 중복 조각이 없어야 한다.
- 개체 캡션의 로컬 vpos와 본문 vpos는 다른 축이다. 초안에서 이들을 섞은 오류를 발견해 실제 caption owner의 차트 뒤 간격과 후속 빈 본문 줄의 높이·간격을 따로 소비하는 관계로 바로잡았다. 표의 소스 구역도 실제 Table 소유 메타데이터에서 찾는다. 초기 컴파일 오류와 초안 실패 로그는 ignored output에 보존했다.
- 정식 harness 재생성 후 **20 PASS / 0 FAIL / 634 skip**, 6.569초다. `issue_7362`의 기존 두 관계 검사는 유지하고 parser·provenance·chart CSV/편집 계약은 제거하지 않았다. 새 검사 함수는 추가하지 않았다.
- 이는 PrEP 변경 범위의 결과이며 남은 기존 실패 17개와 마지막 전체 nextest를 완료한 결과가 아니다.

![PrEP 105쪽 연구비 표의 내용·행 소유 검토](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/prep-review_105.png)

## V10 이후 원래 27개 재실행의 최신 확정 결과

- `c831d36de`에서 **11 PASS / 16 FAIL / 2762 skip**, 49.582초, 종료 코드 100이다. run `7b064ae6-ffcc-4dfd-be0e-affb6be28c7a`이며 이 문서 앞의 19개·17개는 이전 실행의 상태다. 전체 회귀 재실행은 아니다.
- API의 `downstream_table_split_and_body_follow_the_oracle_pages`도 기존 기대값 그대로 통과했다. 새 검사 함수나 baseline 갱신은 없다.
- API 105쪽의 기본 96dpi 전쪽 대조는 base 최저 46.19920%, V10 최저 25.16184%다. 양쪽 각각 17쪽 미달이지만 V10의 36쪽은 새 회귀다. 기존 #7731이 열린 상태임을 확인했으며 이 새 회귀는 구현에서 복구한다. 기존 미달 문서라는 이유로 정확한 분할 검사를 제거하지 않는다.
- `hwp-multi-001.hwp` 기본 96dpi 전 9쪽은 base 최저 93.57938%, V10 최저 95.98104%, 미달 0이다. 6쪽 직접 review에서 `구 분`과 분기 글자는 겹치지 않는다. em 상자 진단의 91→92건과 실제 잉크를 구분한다. fresh WASM과 진단 계약 검토 전 baseline은 갱신하지 않는다.

현재 16개 실패 함수:

- `issue_7413_cell_padding_shrinks_only_when_wrapping_overflows::b_cell_that_cannot_wrap_within_its_height_still_shrinks`
- `issue_5877_fragment_ghost_vrules::issue_5877_gas_rowspan_keeps_label_owner_and_next_row_minimum`
- `issue_4514_overlay_table_flow::issue_4514_overlay_tables_do_not_overlap`
- `issue_7203_split_float_anchors_to_paragraph_top::split_fragment_without_a_gap_is_unchanged`
- `issue_5756_superscript_advance_scale::issue_5756_superscript_line_stays_inside_cell`
- `issue_rowbreak_chart_overlap::rowbreak_hwpx_page8_keeps_continued_nested_reference_line`
- `text_overlap_baseline::text_overlaps_do_not_grow_partition_10`
- `issue_7445_biotoxin_source_ownership::cellbreak_boundary_rows_belong_wholly_to_their_pdf_pages`
- `issue_1156_rowbreak_fragment_fit::kps_ai_page37_defers_overflowing_split_row_slice`
- `issue_1692::issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow`
- `issue_6797_table_item_clears_float_band::table_item_clears_the_previous_float_band`
- `issue_6812_square_picture_tac_table::issue_6812_following_text_respects_column_bottom_after_multiple_pictures`
- `issue_6981_split_straddle_row_height::completed_terminal_cut_does_not_allocate_an_empty_page`
- `text_overlap_baseline::text_overlaps_do_not_grow_partition_2`
- `issue_6981_split_straddle_row_height::continuation_height_respects_varying_page_budgets`
- `body_overflow_baseline::body_overflow_does_not_grow_partition_5`

![hwp-multi-001 6쪽 직접 검토](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/multi-review_006.png)

## API 첫 조각의 여백 예약 보정 (V11c)

- `fragment_host_placement`의 첫 조각 사전 검사는 통째 개체의 아래 바깥여백을 뺐지만, 실제 분할 예산은 `partial_rowbreak_fragment_spacing_px`가 반환한 조각의 아래 여백을 썼다. API pi797의 실제 가용 28.0px이 사전 검사에서는 24.3px이 되어 25.1px의 첫 줄 하한을 거부했다. 35쪽에 들어가는 원본 빈 줄 유닛도 36쪽으로 옮겨져 뒤 본문을 밀었다.
- 같은 helper의 첫 조각 여백을 사전 수용·first overhead·분할 예산에서 소비하도록 맞췄다. 측정 하한을 건너뛰거나 강제 유닛을 수용하는 초안은 폐기했다. 수용 불가 이월, 앵커 거리 소진, 원자 행의 기존 조건과 내용 컷은 유지한다.
- API는 105쪽을 유지하며 페이지 계획과 전쪽 SVG는 35·36쪽만 바뀌었다. 35쪽은 빈 시작 유닛 `endCut=[1]`, 36쪽은 `startCut=[1]` 이어받기로 복구했고 뒤 `See Also`·`EditMode` 내용이 제 위치에 있다. 직접 review를 확인했다.
- Native 기본 96dpi 35쪽 **99.34006%**, 36쪽 **99.21991%**다. 나머지 103쪽은 전쪽 SVG byte-identical 증거로 기존 같은 원본·PDF·글꼴·96dpi raster를 재사용했다. 정본 알고리즘으로 전체 105쪽 TSV를 다시 계산했으며 최저 **46.19920%**, 기존 미달 16쪽이다. 문서 전체 통과가 아니다.
- 기존 API 세 검사·저장 앵커 다섯 검사와 `#6950` 대조군 등 **36 PASS / 0 FAIL / 572 skip**, 1.252초다. run `6187b598-c671-4ee2-99cf-9b0fe5da761a`이며 기존 16개 실패 중 API 첫 조각 검사 한 개를 해결했다. 나머지 15개는 후속 판정·최신 후보 재실행 대상이다.
- 정상 12종·251쪽 Native 및 실제 Chrome fresh WASM 전쪽 SVG는 V10과 동일하다. Mac locked `--no-opt` fresh 빌드 종료 0도 확인했다. API의 전 105쪽 fresh raster·TSV는 계속 실행 중이며 아직 시각 검증 완료로 기록하지 않는다.
- 같은 원본 전체 피델리티는 열린 #7731에 이미 등록되어 있다. 새 36쪽 회귀는 현재 브랜치에서 복구했으며 기존 부적합 범위와 정상 관계 검사를 일괄 제거하지 않는다. 검사 함수·baseline·지침은 이 보정에서 변경하지 않았다.

![API 35쪽의 빈 시작 조각](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/api-review_035.png)
![API 36쪽의 코드·후속 본문](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/api-review_036.png)
