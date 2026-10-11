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

## V11c API fresh 전쪽 실행 완료

- `00cf12a1f`와 같은 조판 코드의 Mac fresh WASM을 실제 Chrome에서 8개씩 캡처해 전 105쪽 TSV를 산출했다. Native와 전쪽 수치가 동일하다. 35쪽 99.34006%, 36쪽 99.21991%이며 두 fresh review도 직접 판독했다.
- 전체 최저 46.19920%, 미달은 12·16·18·27·28·34·41·44·61·63·65·69·73·74·93·94쪽이다. sweep 종료 1은 이 기존 미달에 대한 정당한 검토 보류이며 빌드 실패가 아니다. 기존 #7731의 미완료 범위를 보존한다.
- 진단-only 배분 간격 보정과 보강한 기존 함수는 별도 최신 실행 중이다. 위 결과를 그 실행의 통과나 최종 전체 nextest 통과로 혼동하지 않는다.

## 배분 정렬의 마지막 간격을 잉크로 세는 진단 보정 (V12)

- `hwp-multi-001.hwp`는 기본 96dpi 전 9쪽 Native/fresh WASM 모두 최저 95.98104%, 미달 0이다. base도 최저 93.57938%로 통과한다. 따라서 문서를 미달 이슈로 넘기거나 baseline을 92로 올릴 근거가 없다.
- `extra_char_spacing`의 양수 값은 글리프를 넓히지 않고 다음 문자 원점을 옮긴다. 실제 SVG/Canvas는 `TextStyle::glyph_fit_advance`에서 이 간격을 빼지만 진단은 마지막 슬롯까지 잉크로 보았다. 같은 fit 계약을 끝 경계에 소비하도록 수정했다. 위치·렌더 트리·paint는 바꾸지 않는다. 커닝·세로·회전·겹침 문자와 손상된 전진폭은 보수적인 기존 상자 경로를 유지한다.
- 생산값 → `replay_positions_for`의 문자 원점 → SVG/Canvas의 cluster fit → 진단의 마지막 가시 글리프 끝을 연결했다. 실제 앞 글리프와 겹치는 이웃은 계속 검출하고 마지막 배분 간격에만 놓인 이웃은 잉크 충돌로 세지 않는다. 기존 검사 함수에 양쪽 관계를 보강했으며 새 함수는 추가하지 않았다.
- frozen 이전 진단 라이브러리에서 보강 사례가 1 FAIL(원래 1건/기대 0건)하는 isolated RED를 보존했다. 정식 harness에서 글자 경계 기존 10개는 모두 PASS다. 실제 9쪽 Native SVG도 전쪽 동일하며 진단 text-overlap은 92→0으로 정정됐다. 직접 확인한 6쪽도 0이다.
- 원래 실패 27개와 글자 경계 10개 재실행: **23 PASS / 14 FAIL / 2567 skip**, 47.440초, 종료 코드 100, run `735564d6-357c-4ba6-a7ef-9c815d82f11d`. 원래 27개만 보면 **13 PASS / 14 FAIL**다. 이는 최종 전체 nextest 통과가 아니다.
- harness는 1451 sources / 6207 static attrs / 48 targets / 최소 6559 cases로 고정 devel 기준 확인했다. baseline·원본·PDF·지침은 변경하지 않았다.

최신 잔여 14개:

- `issue_7413_cell_padding_shrinks_only_when_wrapping_overflows::b_cell_that_cannot_wrap_within_its_height_still_shrinks`
- `issue_5756_superscript_advance_scale::issue_5756_superscript_line_stays_inside_cell`
- `issue_5877_fragment_ghost_vrules::issue_5877_gas_rowspan_keeps_label_owner_and_next_row_minimum`
- `issue_rowbreak_chart_overlap::rowbreak_hwpx_page8_keeps_continued_nested_reference_line`
- `issue_4514_overlay_table_flow::issue_4514_overlay_tables_do_not_overlap`
- `issue_1156_rowbreak_fragment_fit::kps_ai_page37_defers_overflowing_split_row_slice`
- `issue_6797_table_item_clears_float_band::table_item_clears_the_previous_float_band`
- `issue_1692::issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow`
- `issue_7445_biotoxin_source_ownership::cellbreak_boundary_rows_belong_wholly_to_their_pdf_pages`
- `issue_6812_square_picture_tac_table::issue_6812_following_text_respects_column_bottom_after_multiple_pictures`
- `issue_6981_split_straddle_row_height::completed_terminal_cut_does_not_allocate_an_empty_page`
- `text_overlap_baseline::text_overlaps_do_not_grow_partition_2`
- `issue_6981_split_straddle_row_height::continuation_height_respects_varying_page_budgets`
- `body_overflow_baseline::body_overflow_does_not_grow_partition_5`


## 이어받은 물리 행의 예약 검사와 교육과정 쪽 번호 핀 분리

생산 보정 `cb7116471`은 이전 조각에서 넘겨받은 물리 잔여 높이를 바로 수용하던 경로도
기존 병합 셀 내용 높이 검사에 통과시킨다. 물리 최소는 보존하며, 새 쪽 예산을 넘으면
내용 분할·이월 경로에서 판정한다. 높이 허용치를 늘리거나 최종 bbox를 제한하지 않았다.

- 수정 전 여백 변형 41개 중 3개에서 본문 경계 초과가 재현됐다. 최초 초과는 0.77333px이다.
  행 높이 배열 차이라는 가설은 진단에서 배제했다. 시작 행의 잔여 높이를 수용하는 조기 경로가
  병합 내용 요구 검사를 건너뛴 것이 원인이었다.
- 유지한 교육과정 관계 검사 7개는 모두 통과했다. 글줄과 셀의 포함, 내용의 소속·중복 방지,
  가변 여백·쪽수, 다시 잘리는 병합 셀과 거대 병합 셀의 비적용 대조를 그대로 검사한다.
- 기존 실패 목록 및 대조 재실행은 **32개 중 20 PASS / 12 FAIL / 2580 skip, 49.817초**다.
  run `02d5579e-534b-4145-adf3-db37bdc4e500`이며 전체 회귀 완료로 보고하지 않는다.
  최초 27개 기준으로 **14개 해결 / 부적합 검사 1개 분리 / 12개 잔존**이다.

분리 함수는 `issue_6981_split_straddle_row_height::completed_terminal_cut_does_not_allocate_an_empty_page`
한 개다. 고정 base `21ed64071f81f35f37618717693bd30f727392c9`에 이미 등록되어 있던 검사이며,
378쪽에 다음 구역 `노동인권`을 요구했다. 독립 한컴 2022 Print PDF에서 378쪽은 이전 표의
`선언문 작성` 행을 소유하고 `노동인권` 구역은 380쪽부터 나온다. 숫자만 옮겨 검사하지 않았다.

같은 원본의 전쪽 피델리티와 쪽 소속은 열린 [#7743](https://github.com/edwardkim/rhwp/issues/7743)에
이미 등록돼 있다. 원본 SHA-256은 `623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e`,
이번 대조 PDF SHA-256은 `6d1119be4b5043f75ba057ed9aeb06753a03eef4b82f78014bfcd02729cf05f2`다.
base는 413쪽, 현재는 416쪽, PDF는 415쪽이다. Native 96 DPI의 377~380쪽은 base에서
69.71230 / 24.13970 / 32.10921 / 29.21402%, 보정 직전에는
49.31812 / 70.48507 / 69.25696 / 22.65933%였다. Native 부정 증거이며 전쪽·fresh WASM
90% 통과를 뜻하지 않는다. 대표 [378쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/curriculum-old-pin-review_378.png)를 직접 확인해
쪽 소속과 행 내용 차이를 판독했다. 입력과 PDF는 다른 정상 계약 검사와 공유하므로 원래 경로에 보존한다.
이 함수만 제거했으며 나머지 7개 관계 검사와 자동 corpus의 다른 정상 범위는 유지한다.

Native와 실제 Chrome fresh WASM의 기존 대조 13문서·356쪽 SVG는 각각 직전 증거와 모두 동일하다.
교육과정 전체 Native SVG는 416쪽 중 4쪽(311·331·380·397)에서 변경됐으나, 해당 4쪽의
96 DPI PNG는 수정 전후 바이트가 동일하고 TSV도 동일하다. 현재 fresh WASM 416쪽 SVG는
현재 Native의 비임베딩 SVG와 모두 바이트가 동일하다. 이 동일성은 변화 범위 증거이며,
교육과정 전쪽 피델리티 수용이나 기존 시각 결함 해결을 뜻하지 않는다. 해당 문서는 #7743에서
독립 PDF의 전쪽 소속·내용과 최저 90% 이상을 확인한 뒤 관계 검사로 복원해야 한다.

원시 증거는 Mac 공용 ignored `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/stage433-remaining-relations/`의
`v13b-original-controls-nextest.log`, `v13b-provenance.json`, `v13b-remaining.json`,
`curriculum-pin-classification.json`, `native-v13b-identity.json`, `fresh-v13b-identity.json`,
`curriculum-v13b-identity.json`, `curriculum-affected-raster-identity.json`,
`curriculum-native-fresh-raw-identity.json`에 둔다. 새 회귀 함수·fixture와 지침 변경은 없다.
Mac locked `--no-opt` fresh WASM은 로컬 대체 빌드이며 Docker 최적화 빌드 통과가 아니다.


4쪽의 fresh PNG도 현재 Native와 바이트가 동일했다. 최저값은 22.65933%로 기존 미달이며,
Native/fresh TSV는 311쪽 67.60855%, 331쪽 64.74204%, 380쪽 22.65933%, 397쪽 48.23959%로
일치했다. 원본에서 생성한 해당 4쪽의 전체 글꼴 선언을 정본 `svg_font_face_rules` /
`apply_svg_font_policy`로 공급하고, `check_sweep_embedded_fonts`의 Unicode cmap 검사,
정본 Chrome rasterizer, `write_silhouette_tsv`를 그대로 사용했다. 알고리즘과 2px 관용 거리는
변경하지 않았으며 캡처 동시 실행 상한은 8개다. 증거는 `curriculum-fresh-selected96/provenance.json`,
`embedded_fonts.json`, `silhouette.tsv`다. 별도로 시작한 416쪽 전체의 중복 글꼴 정책 처리
(38.62GiB)은 이 검증 완료 후 종료했으며 exit143을 통과로 세지 않는다. 실제 fresh 416쪽의
raw SVG와 관련 4쪽의 검증 결과는 보존한다.


## KPS 제안서 지침 표의 잘못된 행 컷 핀 교정

`tests/issue_1156_rowbreak_fragment_fit.rs`의 기존 KPS 함수 한 개를 수정했다. 독립
`pdf/kps-ai-2022.pdf`의 37쪽은 성능 요구사항까지, 38쪽은 데이터 요구사항부터 소유한다.
기존 `rows=0..16` / `rows=16..32` 고정 문자열은 데이터 항목을 앞 조각에 요구했다.
고정 base Native는 37쪽 96.63984%, 38쪽 50.03174%였으며 실제 base render tree는 데이터를
앞 조각에 두고 이어받은 조각에서는 누락했다. 현재 Native와 실제 Chrome fresh WASM은 각각
37쪽 **99.75093%**, 38쪽 **98.65303%**이고 PNG도 서로 바이트가 동일하다.
두 쪽의 review를 직접 판독해 항목·작성방법·표 경계와 내용을 대조했다.

새 검사는 제목으로 원본 표의 소속을 찾고, 두 연속 조각에서 성능/데이터 항목의 순서와
소속·중복 방지를 확인한다. 모든 원본 셀의 내용과 정확한 복사 수를 보존하며 원본의 반복
제목행만 예외로 둔다. 본문 안의 표와 셀 안의 글줄을 상대 포함 관계로 검사한다.
절대 쪽 인덱스·행 컷 숫자·dump 문자열 형식에 묶지 않는다. 원본·PDF와 함수 이름은
유지했고 새 테스트 함수, 픽스처, 픽셀 핀은 추가하지 않았다. 같은 파일의 Synam 검사는 바꾸지 않았다.

정식 nextest는 **2 PASS / 0 FAIL / 217 skip, 0.214초**로 두 기존 함수를 모두 통과했다.
원본 27개 실패의 누적 상태는 **15개 해결 / 부적합 1개 분리 / 11개 잔존**이다.
전쪽 77쪽의 시각 수용이나 전체 회귀 완료를 주장하지 않는다. 관측 쪽수는 Native/fresh/PDF
77쪽으로 같으며 이 테스트에 전체 쪽수 핀을 추가하지 않았다. 변경한 경계의 전체 범위는 위 두 쪽이다.

원시 증거는 `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/stage434-next-failure-classification/`의
`kps-nextest.log`, `kps-nextest-command.json`, `kps-base96/kps77/silhouette.tsv`,
`kps-current96/kps77/silhouette.tsv`, `kps-fresh-selected96/silhouette.tsv`,
`kps-fresh-selected96/provenance.json`, `embedded_fonts.json`이다. 대표
[38쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/kps-boundary-review_038.png)를 보존했다.
base의 새 Rust 함수 RED 실행은 하지 않았으며, 실제 base CLI의 내용 소속 관측을 그 실행으로
바꾸어 보고하지 않는다. 처음 잘못 지정한 standalone 필터의 0-test 결과도 통과 수로 세지 않고,
정확한 함수 필터의 1 PASS와 정식 nextest 2 PASS만 기록한다.

## 영월 원본 HWP3 전4쪽 보정과 기존 검사 교정 (V14)

원본 `samples/issue5699/20099369_yeongwol_forms.hwp`와 독립 한컴 2020 Print
`pdf/20099369_yeongwol_forms-2020.pdf`를 보존했다. 고정 devel의 기본96 DPI 전4쪽 최저는
30.74890%, 보정 전 현재 출력은 31.50317%였다. 첫 제목 여백이라는 초기 가설과 달리,
한컴 변환본과 대조하니 제목과 빈 줄의 **쪽 소속** 및 일부 줄 구성이 달랐다.

- HWP3 사양 표12의 bit15는 단/페이지 구분 정보의 유효성이다. 이 비트 하나만으로 줄 전체를
  버리는 시안은 폐기했다. 원본의 줄 시작·공백 보정·좌표·구분이 모두 0이고 글자 높이만 남은
  레코드는 저장 줄 구성을 인증하지 않는다. 해당 경우 문단·개체 속성을 보존하고 공통 composer가
  실제 너비로 줄 구성과 점유 높이를 정한다. 본문에 저장 사다리가 전혀 남지 않으면 한글97의
  마지막 줄 추가 예산을 부여하지 않는다. 실제 용지·여백 수치는 변경하지 않았다.
- 같은 원본의 독립 한컴 HWPX 변환은 engine2020, job
  `2ad1bdde-09bb-4aa6-ae09-9ed7816fb0cb`, 제품 `11.0.0.9136`에서 성공했다.
  서버/클라이언트 61,823바이트와 SHA-256이 일치한다. 변환본은 진단 참고 자료로
  자산 폴더에 보존했으며 원본 HWP나 기준 Print PDF를 대체하지 않는다.
- 실제 Native와 Chrome154 fresh WASM 전4쪽은 각각
  **99.53097 / 100.00000 / 99.94533 / 100.00000%**이고 PNG4개 모두 서로 같다.
  전쪽 TSV 원본을 자산 폴더에 보존했다. Mac locked `--no-opt` 빌드는 로컬 대체 경로다.
- 기존 HWP3 대조 3입력·78쪽의 Native SVG는 보정 전후 모두 바이트가 동일하다.
  `SO-SUEOP`의 기존 미해결 범위를 시각 통과로 승격하는 증거는 아니다.

기존 영월 회귀 함수3개의 계약을 교정했다. `#5699`의 4쪽 개수는 독립 Print와 일치하므로
유지하고, 원본의 별지 제목4개가 각 대응 쪽에서 한 번만 소유되는지 함께 확인한다.
SVG `y>570`은 원본 선행 표와 계약조건 본문의 소유·순서 관계로 바꿨다.
`#7413`의 고정 y/폭 셀 선택과 한 줄 요구는 독립 Print에서 틀렸다. `월별누계`는
`월별` / `누계` 두 줄이며 마지막 줄 뒤 간격을 제외하면 원본 셀 안에 들어간다.
원본 셀과 표의 소속, 내용 순서·누락·중복·프레임 내부 포함으로 수정했다. 기존 두번째 함수의
이름은 `b_uncomputed_hwp3_cell_preserves_the_oracle_wrapped_content`로 변경했다.
다른 입력 `table-text.hwpx`의 기존 여백 계약은 변경하지 않았다. 새 테스트 함수는 추가하지 않았다.

- HWP3 파서 기존 검사: **50 PASS / 0 FAIL / 3959 skip**, 0.128초.
- 정식 하네스의 기존 실패와 관련 정상 대조: **41개 중 31 PASS / 10 FAIL / 3254 skip**,
  53.395초, run `4c996fc0-0b6c-482c-ad01-a237e587bc0f`, 종료100. 영월4개는 모두 PASS다.
- 최초27개 기준 **16개 해결 / 부적합 검사1개 분리 / 10개 잔존**이다.
  전체 nextest와 최종27문서 검증 완료를 뜻하지 않는다. PR 준비는 아직 보류다.

대표 [3쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/yeongwol-review_003.png)와
[4쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/yeongwol-review_004.png)를
직접 확인했다. 표 괘선 농도·래스터 가장자리 차이는 남아 있으며, 2px 관용 실루엣100%를
엄격 픽셀 동일로 보고하지 않는다. 원시 증적은 ignored output의
`stage435-yeongwol-margin/`에 보존한다. 지침 수정·원본 삭제·외부 이슈 게시를 하지 않았다.
이번4쪽 원본은 현재 브랜치에서 보정했으며 별도 이관이 필요하지 않았다.

## SO-SUEOP의 기존 좌표 요구 분리 (V15)

두 원본과 독립 한컴 PDF는 모두 46쪽이다. HWP3 기준은 기존
`pdf/SO-SUEOP-hwp-2020.pdf`(Creator Hwp 2022), HWPX 기준은
`pdf/SO-SUEOP-hwpx-2024.pdf`(Creator Hwp 2024)를 그대로 썼다. 유효한 독립 PDF의
버전 표기 때문에 재산출하거나 원본을 바꾸지 않았다.

현재 Native 전 46쪽 TSV는 HWP3 최저 36.34276%·미달 39쪽, HWPX 최저 43.01145%·미달 9쪽이다.
검사 대상 22쪽은 고정 devel HWP3 81.85474%, 현재 HWP3 81.67681%, 현재 HWPX 96.87848%다.
devel의 1쪽은 35.80495%이며 devel 비교 범위는 1·22쪽이다. 전 46쪽 base 통과로 쓰지 않는다.
실제 Chrome fresh WASM은 두 원본 각각 46쪽 SVG를 생성했으며, 관련 22쪽 PNG는 각각 현재
Native PNG와 같고 점수도 같다. 전쪽 fresh 검증 완료나 전체 피델리티 수용을 주장하지 않는다.

두 22쪽을 직접 읽었다. HWP3의 파란 글자 요청은 `필기`인데 Mac Fontconfig의 정확한 family를
찾지 못했으며 PDF와 글자 모양이 다르다. 유사 필기체로 임의 치환하지 않았고 모든 미달을
글꼴 하나로 단정하지 않았다. 원본 전쪽 미달은 새 [#7807](https://github.com/edwardkim/rhwp/issues/7807)에
등록한 뒤 #7445의 서브 이슈로 연결했다. API로 본문 일치·OPEN·parent의 child 7807을 재확인했다.
기존 child 80개와 열린 이슈의 같은 원본 body/comment를 확인했으며 #7203의 표 여백 계약과
#4680의 왕복 저장 범위는 이 전쪽 시각 미달과 구분했다.

변경은 기존 실패 함수 안의 **두 좌표 assertion 분리**다. 다른 rhwp 형식의 y를 1px 이내로
같게 만드는 비교와 후속 질문의 절대 PDF y 상수를 제거했다. 원본 각주 컨트롤 위치·관계도 문자/
연결선 복원·표의 기존 바깥여백 흐름 계약·본문 내용과 후속 질문 존재/쪽 소속은 보존했다.
전체 함수를 삭제하지 않았다. 생산 렌더러·입력·PDF·지침을 변경하지 않았고 새 테스트 함수는 없다.

정식 모듈 검사 **11 PASS / 0 FAIL / 207 skip**, 0.153초,
run `014fd5d5-e3eb-4167-b077-d1c18b21fc7c`, 종료0이다. 최초 27개 기준
**16개 해결 / 부적합 검사 범위 2개 분리(함수 1·assertion 범위 1) / 9개 잔존**이다.
현재 통과 17개 중 1개는 부적합 좌표를 분리한 정상 계약의 통과이며 문서 피델리티 해결로 세지 않는다.

대표 [HWP3 22쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/so-sueop-hwp3-review_022.png)와
[HWPX 22쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/so-sueop-hwpx-review_022.png),
각 Native 전쪽 TSV·fresh 22쪽 TSV·devel 선택 TSV를 자산으로 보존한다. 로그와 거대한 임베딩 SVG는
ignored `stage436-so-sueop-oracle/`에 둔다. #7807에서 두 원본 전쪽 최저 90% 이상과 직접 판독 뒤
독립 내용·소속 관계로 재구축한다. 현재 전체 검증과 PR 준비는 미완료다.

## 빈 호스트 표의 선행 밴드 회피 (V16, 시각 마무리 진행 중)

양돈 원본 `samples/issue6797/156160455-social-pig-farm-income.hwp`과 기존 독립 한컴 Print PDF
`pdf/156160455-social-pig-farm-income-2020.pdf`는 전 11쪽이다. 수정 전 7쪽은 34.05714%,
나머지 10쪽은 모두 98% 이상이었다. 원래 회귀 검사는 앞 표·그래프 소유 표·뒤 제목의 순서와
두 그림의 셀 내부 포함을 검사하므로 유지했다.

생산 경로는 `empty_float::prepare`의 확정 상자 → `lane_bottom`의 fit →
`commit_empty_float_table`의 배치 계획/높이 예약 → layout의 계획 조회 →
`layout_table_with_wrapper_margin`의 확정 원점이다. 같은 문단의 lane은 앞 문단의 자리차지
밴드를 포함하지 않았다. 기존 공통 `clear_occupied_bands`로 앞 소유자의 밴드를 지난 상자를
fit·예약·paint에 전달했다. 겹침 허용/어울림 형제는 유지하고 이동한 상자의 후속 줄 간격 증거는
분리했다. 초기 host push-floor 보정안은 실패가 지속되어 되돌렸고 커밋하지 않았다.

기존 양돈 검사 5개는 5 PASS / 0 FAIL이다. 정식 관련 검사 38개는 37 PASS / 1 FAIL,
0.974초·종료 100이며, 남은 1개는 기존 다중 그림의 본문 바닥 초과 검사다.
13개 대조 원본 전 356쪽의 unembedded Print SVG는 생산 V14와 byte-identical이다.
대조 SVG 동일은 새로운 PDF/fresh WASM 통과를 뜻하지 않는다.

양돈 Native 전 11쪽에서 7쪽은 **80.90032%**로 개선됐고 다른 10쪽은 여전히 98% 이상이다.
[7쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/pigfarm-v16-review_007.png)를
직접 읽었다. 표 겹침은 없어졌지만 그래프 뒤 제목/본문의 앞 간격이 남아 있다. 이 문서는 기존
[#7770](https://github.com/edwardkim/rhwp/issues/7770) 범위이며 이 브랜치에서 후속 프레임을 마무리한다.
전쪽 90%/fresh 완료나 이슈 해결로 보고하지 않는다. 새 회귀 함수/기대값 변경/지침 수정은 없다.

`sample1-repro.hwp`는 독립 한컴 PDF `sample1-repro-hwp-2020.pdf`를 사용했다.
다른 `sample1-repro-2020.pdf`는 Cairo 출력이어서 독립 기준으로 사용하지 않았다.
Native 전 46쪽은 최저 2.77636%·미달 23쪽, 현재 10쪽 53.82229%, 고정 devel의 같은 10쪽은
50.42130%다. [10쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/overlay4514-review_010.png)에서
ECR-004 앞 항목 반복과 ECR-005 상세 행 누락을 확인했다. 기존 열린 이슈/서브 이슈를 확인한 뒤
[#7808](https://github.com/edwardkim/rhwp/issues/7808)을 등록하고 #7445에 연결했으며 API로 본문·OPEN·
parent child를 확인했다. 정확한 겹침/이어받기 검사는 유지했다. fresh 전쪽은 미검증이다.

원 실패 기준 18개가 현재 통과 중이나, 1개는 부적합 좌표 분리 후 정상 계약 통과이고 1개는
양돈 시각 마무리가 남아 있다. **16개 해결 / 부적합 범위 2개 분리 / 실패 8개 + 양돈 시각 잔여**다.
전체 nextest/최종 대상 문서 검증/PR 준비는 미완료다. 원시 로그는 ignored `stage437-overlay-flow/`다.

## 양돈 11쪽 원본 프레임 복구와 관계 검사 (V17)

V16의 단순 선행 밴드 회피 뒤에도 원본 상자를 이전 호스트 흐름에 옮기는 분기가 남아 있었다.
앞 밴드와 뒤 저장 줄이 전체 원본 상자를 이미 증명한 경우에는 그 쪽 원점을 보존하도록 수정했다.
동일 원점의 부동소수 끝자리 차이는 기존 lane 이동 판정과 같은 상대 epsilon으로 구분하며,
실제 이동을 허용하거나 픽셀 비교의 관용 범위를 늘린 것이 아니다.

같은 원본/독립 PDF/96 DPI/글꼴 조건의 **Native·실제 Chrome fresh WASM 전 11쪽 최저
98.19793%, 미달 0**이다. 7쪽은 34.05714% → 80.90032% → **98.96856%**다.
Mac locked `--no-opt` WASM을 새로 빌드하고 실제 Chrome 154.0.8037.99에서 전 11쪽을 출력·
최대 8개 병렬 캡처했다. 모든 fresh PNG가 Native와 byte-identical이며 재시도도 없었다.
Docker 최적화 빌드로 보고하지 않는다. 13개 대조 원본 전 356쪽 Native Print SVG도 V14와 같다.

[최신 7쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/pigfarm-v17-review_007.png)를
직접 확인했다. 두 그래프·후속 제목·본문의 순서와 내용을 보존하고 앞 간격 차이는 해소됐다.
괘선 농도·글자 래스터 가장자리 차이는 남아 있어 엄격 픽셀 동일로 보고하지 않는다.

원본의 전쪽 검증 뒤 기존 `issue_6812` 함수의 81.24px 위치 핀을 같은 표 앞 입력에서 뒤 그림만
없는 대조군과의 배치 불변 관계로 교정했다. 141HU 여백 상수는 원본 표의 위여백으로,
100px 새 쪽 하단 상수는 수정한 PageDef 본문 안의 포함 관계로 바꿨다. 원본 속성에서 계산하는
크기/비율 및 기존 겹침/내용/쪽 소속 검사는 유지하고 새 회귀 함수는 추가하지 않았다.
양돈 원본 검사 5개는 5 PASS이며 같은 입력의 기존 어울림 검사 20개는 진단 19 PASS / 1 FAIL이다.
기존 다중 그림 변형의 본문 줄 3.64px 초과는 숨기지 않고 유지했다.
정식 nextest도 **24 PASS / 1 FAIL / 198 skip**, 0.359초·종료 100이며
run `93609cdb-4298-4960-99d6-8e8a186320fe`이다. 실패한 본문 포함 assertion은 변경하지 않았다.

최초 27개 기준 **17개 해결 / 부적합 범위 2개 분리 / 실패 8개 잔존**이다.
전체 nextest와 최종 대상 전체 검증은 아직 미완료다. #7770은 통합 전이므로 OPEN을 유지한다.


## 그림 회피 줄의 물리 끝점 공유 (V18)

`issue_6812_following_text_respects_column_bottom_after_multiple_pictures`는 독립 쪽 안의 본문
포함을 검사하는 정확한 관계 검사다. 3.64px 초과 assertion은 유지하고 구현을 보정했다.
원본 저장 줄은 실제 표 본체와 상하 바깥여백을 온전히 소유하며, 다음 원점은 뒤 간격 전량을
증명한다. 기존 분할 후 상한은 이 공간 일부를 회수했지만 paint는 전량 소비해 줄 수용과
출력의 끝점이 갈라졌다.

생산 경로는 `stored_tac::side_wrap_row`의 원본 소유 줄·점유/뒤 간격 → `typeset_tac_table`의
그림 회피 프레임/occupied fit → 동일 `InlineBoxPlacement.advance_end`의 예약 →
`tac_reconcile::capped_bottom`과 `layout_table_control_block`의 같은 끝점 소비다.
가로 위치의 소유는 `resolved_x`로 분리하여, 세로 끝점 유무가 그림 옆 구간의 x를 지우지 않게 했다.
완전한 단일 원본 줄에만 적용하고 앞뒤 본문·캡션·각주·편집/재구성 줄은 기존 경로를 유지한다.
문서 ID·좌표 상수·출력 clamp·픽셀 관용 변경은 없다.

원래 기존 함수 20개는 수정 전 19 PASS / 1 FAIL → 수정 후 20 PASS / 0 FAIL이다.
양돈 원본 검사를 포함한 정식 nextest는 **25 PASS / 0 FAIL / 198 skip**, 0.463초·종료 0,
run `2f4a8614-1fe2-4daf-ba25-5ad4788a07d9`이다. 이후 같은 함수에서 원본 줄 순서·단일 소유와
전체 텍스트 보존을 보완한 진단 20개도 통과했다. 원본 141HU 위치 몫은 표의 실제 위여백 속성으로
바꿨고 새 회귀 함수를 추가하지 않았다. 보완 후 정식 재실행도 **25 PASS / 0 FAIL / 198 skip**,
0.361초·종료 0, run `15033650-56fc-453a-b703-1bebdd781dc4`로 통과했다.

같은 원본·독립 한컴 PDF의 **Native·실제 Chrome fresh WASM 전 11쪽 최저 98.19793%,
미달/누락/쪽수 차이 0**이다. 최대 8개 병렬 캡처, 재시도 0이며 모든 fresh PNG가 Native와 같다.
현재 Native PNG 전 11쪽도 V17과 같고, 13개 정상 대조 원본 356쪽 Print SVG도 V14와 같다.
대조 SVG 동일은 356쪽의 새로운 PDF/fresh 수용을 뜻하지 않는다. Mac locked no-opt WASM을
새로 빌드했으며 Docker 최적화 빌드로 보고하지 않는다.

[최신 1쪽 review](../pr/assets/issue7445/completed-20261009/stored-anchor-repair/pigfarm-v18-review_001.png)와
ignored `stage437-overlay-flow/pigfarm-v18-review/review_007.png`를 직접 확인했다.
표·그림·제목·뒤 본문의 소속과 간격을 보존한다. 괘선 농도·글자 래스터 가장자리 차이는 남아
엄격 픽셀 동일로 보고하지 않는다. 전쪽 Native/fresh TSV와 대표 1쪽 PNG를 보존한다.

최초 27개 기준 **18개 해결 / 부적합 범위 2개 분리 / 실패 7개 잔존**이다.
현재 브랜치에서 새로 생긴 본문/텍스트 초과는 기존 미달이나 이슈 등록으로 제거하지 않고
계속 구현을 확인한다. 전체 nextest·최종 대상 전쪽 검증·PR 준비는 미완료다.

## 저장 프레임 완료 소유를 일반 행 수용보다 먼저 적용 (V19)

원본 `samples/issue5679/10857_delegation_rules.hwp`의 62쪽 표는 현재 변경으로 추가된
본문 아래 넘침 1건이었다. 저장된 첫 조각 높이는 66633HU인데, 큰 병합 블록 안의
마지막 일반 행이 기존 완료 프레임 검사를 건너뛰고 통째 수용 허용량을 먼저 사용했다.
표의 올바른 위여백140HU를 없애거나 baseline을 늘리지 않고 기존 검사의 실행 순서를 고쳤다.

`row_step.rs`는 첫 source frame 끝과 앞 병합 소유가 일치할 때 같은 `RowScanQuery`로
수용 원장·실제 여백·내용 완전 소비를 확인한다. 성공한 경우 source 잔여 높이와 컷·끝행
override를 함께 반환한다. `fragment/emit.rs`의 기존 완료 표식/이어받기 경로가 이를 소비하며,
다음 조각은 남은 빈 물리 밴드와 다음 실제 행을 보존한다. 편집·재조판·중첩/개체가 있는
셀에는 기존 제외 조건을 유지한다. source 파일명 분기·paint clamp·공차 변경은 없다.

실제 표 높이는894.28→888.44px로 원본66633HU와 일치하고, 본문 아래 사건은3→2건으로
기존 baseline 수준에 복구됐다. 마지막 `신변보호조치 해제 및 신고자 통지`의 가시 소유는
62쪽에 한 번이며, 63쪽의 이어받은 빈 밴드는 글자를 재소비하지 않는다. 그 다음 실제 행
`책임감면 / 징계의 감경․면제 요구`를 보존하고, 앞 rowspan의 양쪽 물리 높이 합은 원본
34139HU와 직렬화 오차 범위에서 같다.

- 기존 본문 넘침 검사 및 #6790/#5057/#3820/#6981/#7470/#7288/#7095 대조군:
  run `5ba5ec58-b536-4cc0-84fa-07c759f4f11b`,36 PASS/0 FAIL/1418 skip,47.769초,threads8,exit0.
- 남은6개 원 실패 재실행: run `3796b42d-e2cb-4512-921a-adad7caf8503`,0 PASS/6 FAIL/1081 skip,
  1.367초,exit100. source 파일·검사 기대값은 바꾸지 않았다.
- 앞서 검증한13개 문서356쪽의 Native SVG는 V18과 모두 바이트 동일하다. 이것을
  356쪽의 새 PDF 비교나 fresh 전쪽 수용으로 대신하지 않는다.
- #3820의 원본82쪽은2쪽만 SVG가 바뀌었다. 실제 저장 제품2024에 맞는 독립 Print
  `pdf/issue1891/76076-original-hwp-exact-2024-20261005.pdf`와 generation2024로 확인한
  2쪽은97.34589→97.67017%이며 Native/fresh PNG가 같다. 이전 다른 HWPX용2020 PDF와
  Cairo 산출물은 판정에 사용하지 않았다. 전82쪽 피델리티 수용을 주장하지 않는다.

위임전결 원본은 engine2020의 실제 Hancom PrintMethod0/one-up으로 다시 PDF를 생성했다.
job `9a5db5bb-3d42-4fb5-9bbd-4b0f286d089e`,success,769432byte/103쪽이며 최종 정본을
`pdf/10857_delegation_rules-2020.pdf`에 함께 보존한다. 원본·기존 정본을 삭제하지 않았다.
기존 #5679에도 한컴103쪽이 기록되어 있고 전체 피델리티 잔여는 OPEN #7700에 등록되어
있으므로 중복 이슈를 만들지 않았다.

| 진단용 의미 소속 대조 | V18 Native | V19 Native/fresh | 판정 |
| --- | ---: | ---: | --- |
| Native62쪽 / Print61쪽 |98.67835%|98.08764%|현재 source frame과 본문 포함 확인|
| Native63쪽 / Print62쪽 |60.52857%|84.18323%|개선됐지만 잔여 미달, #7700 유지|

현재 Native/fresh는105쪽,Print는103쪽이다. 위 표는 의미가 같은 표 조각을 대응시킨
**진단 값**이며 같은 쪽 번호의 canonical 전쪽 TSV나 전체 문서 수용이 아니다. 자동 쪽 매핑으로
쪽수 차이를 숨기지 않았다. 실제 Chrome154.0.8037.99에서 fresh187쪽(105+82쪽)을 생성했고,
영향3쪽만 capture8로 래스터 비교했다. 선택3쪽의 Native/fresh PNG는 모두 동일하다.
Mac locked `--no-opt` WASM build exit0이며 Docker 최적화 결과로 표시하지 않는다.

#5756 원본 보도자료는 고정 base·다운로드 원본과 동일한140288byte/SHA-256 입력이다.
자동 Print 재시도 `69f08d05-3061-431f-893c-a00e9a5b6b87`도 output0/failed였다.
기존 문자열 크기 검사10.0/14.2가 현재10.2666/14.6666과 달라 실패하지만, 독립 PDF가 없어
90% 판정 또는 기대값 변경 근거로 사용하지 않았다. 이 기준 확보·재검토를 #7809로 등록하고
#7445의 서브 이슈로 연결했으며 본문·부모 연결을 API로 재확인했다. 검사는 유지한다.

정본은 기존 `stored-anchor-repair/validation.json`의 `v19`,선택 Native/fresh TSV,
`delegation-native062-print061-v19-review.png`, `delegation-native063-print062-v19-review.png`,
`regulatory82-p002-v19-review.png`다. 원시 로그·source/binary hash·진단은 ignored
`stage438-delegation-budget/` 및 `stage439-superscript-print/`에 둔다.
기존27건은19건 해결·2개 부적합 scope 분리·6개 실패 잔여다. 전체 nextest·최종 대상 전쪽
검증·PR 준비는 미완료이며 PR 게시·병합은 전체 검증 후 사용자 승인 전까지 수행하지 않는다.

## 이어받기 빈 prefix와 저장 줄 압축의 소비 대상 정합 (V20)

설문 원본 `samples/task2430/1382000_domestic_violence_survey.hwp`의 24쪽은 현재 변경으로
새 텍스트 겹침·본문 아래 넘침이 생긴 범위였다. `advance_row_cut`이 흐름을 소유하지 않는
빈 시작 유닛을 높이 합에서 제외했지만, 예약과 paint는 이전 시작 컷을 다시 사용했다.
마지막 줄 뒤의 비가시 간격과 원본 쪽 상자의 가운데 정렬 공간을 구분해 보정했다.

`CellUnit`의 빈 줄 소유 판정 → 이어받기 스캔의 실제 점유/수용 예산 → 같은 원본 프레임
Query → 시작 컷 갱신과 같은 예산 재스캔 → `emit_table_fragment` 예약 → partial paint의
같은 프레임 Query로 연결된다. 원래 첫 조각의 빈 밴드는 이어받기 prefix가 아니므로 보존한다.
내용을 소유한 줄·hard break·물리 높이 override·완전한 끝 조각을 같은 빈 prefix로 소비하지
않는다. source 파일명 분기, 출력 좌표 clamp, 테스트 기대값·공차·baseline 변경은 없다.

첫 두 보정안은 가운데 정렬 대조군을 깨뜨려 기각했고, 다음 보정안은 24쪽을 고치지 못해
기각했다. V20d는 정식 25개가 통과했지만 API 35쪽의 원래 첫 빈 프레임이 바뀌어 그대로
채택하지 않았다. 최종 V20e는 원장의 비소유 이어받기 prefix가 있는 경우에만 프레임 소유를
재검토한다. API 105쪽의 SVG는 V19와 모두 바이트 동일해 원래 빈 프레임을 보존한다.

별도로 설문 27·34쪽은 글꼴·크기·글자 수가 같은데 선두 공백의 가로 전진이 달랐다.
양쪽 정렬의 양수 여분 폭에서 제외할 들여쓰기 공백을 자연 폭 초과의 압축에서도 제외해
오른쪽으로 밀렸다. 압축 대상 배열을 글줄 계산과 실제 text run이 함께 소비하도록 수정했다.
원본 글꼴과 기준 PDF는 바꾸지 않았다.

설문의 기존 `-2020-print.pdf`는 Cairo 산출물이었다. 실제 engine 2020 PrintMethod 0/one-up
job `a3ef2118-897a-4427-ac15-375c0a5806f8`로 원본을 새로 출력했고, 기존 Cairo 파일을 같은
정본 경로의 한컴 PDF로 대체한다. 461,873바이트·39쪽이며 이전 유효한 Hwp 2018 40쪽 PDF는
보존했다. 교체 전후 SHA-256과 실제 runtime은 `validation.json`의 `v20`에 기록한다.

| 대조 범위 | 변경 전 | 현재 Native·fresh WASM | 판정 |
| --- | ---: | ---: | --- |
| 설문 24쪽 | V19 40.62366% | 92.21086% | 겹침·본문 넘침 해소, 내용·표·쪽번호 소속 유지 |
| 설문 27쪽 | base 91.78356%, V20d 86.33744% | 91.78356% | base 수준 복구 |
| 설문 34쪽 | base 90.81951%, V20d 89.95164% | 90.81951% | base 수준 복구 |
| 규제분석 전 17쪽 | 입력·Print·설치 글꼴 동일 | 최저 97.71204% | 미달·누락·쪽수 차이 0 |
| 철도 EMC 전 14쪽 | 입력·Print·설치 글꼴 동일 | 최저 98.00998% | 미달·누락·쪽수 차이 0 |
| 자산관리 전 60쪽 | 입력·Print·설치 글꼴 동일 | 최저 93.48019% | 미달·누락·쪽수 차이 0 |
| PrEP 전 140쪽 | 입력·Print·설치 글꼴 동일 | 최저 91.93033% | 미달·누락·쪽수 차이 0 |

위 완료 문서 4종 231쪽은 Native/fresh PNG가 모두 같다. 설문은 영향 24·27·34쪽을 포함해
38쪽이 같고, 변경하지 않은 1쪽의 래스터 차이는 별도로 기록했다. 설문의 전 39쪽은 여전히
11쪽 미달이며 전체 피델리티 수용을 주장하지 않는다. 고정 base에서도 미달인 범위이고 기존
#7095·#7387·#7391에 같은 원본이 등록되어 있어 중복 이슈를 만들지 않았다. 기존 정확한
텍스트 겹침 검사는 제거하지 않고 구현을 고쳤다. 새 전체 판정으로 잠정 쪽수·좌표 핀을
추가하거나 현재 미달을 새로운 기대값으로 고정하지 않았다.

- 최종 기존 분할·정렬·본문·텍스트 겹침 정식 검사: run
  `7cd63220-ba78-4ca8-a4fc-5f307cf6a52d`, 29 PASS / 0 FAIL / 5,047 skip,
  50.057초, threads 8, exit 0. 전체 nextest의 대용이 아니다.
- 13종 356쪽 Native SVG 중 가로 간격이 바뀐 4종 59쪽을 확인했고, 해당 4종은 위와 같이
  전쪽을 새로 비교했다. 나머지 297쪽은 이전 SVG와 같다.
- Native·fresh 각각 270쪽을 실제 래스터화했고 최대 동시 캡처 8개, 재시도 0이다.
  첫 PrEP 명령은 PDF 경로가 잘못되어 캡처 전에 실패했다. 기존 manifest로 경로를 바로잡고
  첫 세 문서를 다시 캡처하지 않은 채 PrEP를 수행했다. 실패 로그도 보존한다.
- 기존 입력·PDF와 설치 글꼴의 해시 변화는 0이다. 기존 Native SVG에서 같은 글꼴 CSS만
  가져왔으며 좌표·PNG를 재사용하지 않았다. raw WASM은 실제 Chrome에서 전쪽을 출력했다.
- Mac locked no-opt WASM은 exit 0이고 Docker 최적화 빌드로 표시하지 않는다.
- 원시 TSV·manifest·로그·SVG는 ignored `stage440-continuation-prefix/`에 둔다.
  커밋에는 수정 소스, 새 Print PDF, 대표 review PNG 3장과 최종 요약만 포함한다.

기존 27건 중 20건 해결·2개 부적합 범위 분리·5개 실패가 남았다. 남은 5개 원 검사는 최종
소스에서 재실행 중이며 제거하거나 기대값을 바꾸지 않았다. 전체 최종 검증과 PR 준비는
미완료이고, 전체 검증 후 사용자 승인 전까지 PR 게시·push·merge는 수행하지 않는다.
