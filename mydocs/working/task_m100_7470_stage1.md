# Task #7470 Stage 1 — RowBreak 조각의 소유와 물리 배치 복원

- 관련 이슈: [#7470](https://github.com/edwardkim/rhwp/issues/7470). 이 변경으로 이슈 전체를 종료하지 않는다.
- 기준 devel: `6b3faf77d8085441f9f26d88d65a49791e910352`.
- 구현 체크포인트: `b119d784ac`(최소 공통 메트릭 포함). 정식 회귀·전체 검증·제출은 진행 중이다.
- 원래 공백·지도 후보 `bac75f50ee4e57839f4c0ac7a259165acf0e3509`는 별도 branch에 clean 상태로 보존한다.

## 범위와 독립 근거

실제 저장 문서 `samples/rowbreak-problem-pages.hwp`와 대응 한컴 출력
`pdf/rowbreak-problem-pages-hwp-2024.pdf`의 전체 18쪽을 비교한다. 입력·LineSeg를 수동
수정하지 않는다. 원본의 줄·셀 높이·바깥 여백으로 닫히는 프레임을 재사용하되,
내용 컷과 물리 공간의 소유를 구분한다. 측정·예산·실제 배치가 같은 결과를 소비한다.

RowBreak 단독 후보는 Native/fresh WASM 7쪽 89.69439%였다. 공백 후보에서 분리한
11개 소스 파일의 최소 메트릭 기초를 더하면 전체 18쪽이 90% 이상이었다. 작업지시자가
이 의존성을 선행 PR에 포함하도록 승인했다. `SpaceMetric`을 줄 구성 결과와 함께 전달하고
공백 측정과 실제 run 배치에 같은 규칙을 적용한다. 줄 교체·편집 때 출처도 함께 무효화한다.
기존 composer 테스트의 새 필드 초기화 12개는 유지하며 새 기대값이나 baseline 완화가 아니다.
어구 문서의 지도·나머지 공백 보정은 포함하지 않고 후속 PR로 유지한다.

## 실제 호출 경로와 반례

| 동작 | 공통 결과의 생산과 소비 | 독립 기준·경계 |
| --- | --- | --- |
| 위 캡션 표의 흐름 | `float_placement::stored_empty_control_table_frame` → `block::whole_fit`/`entry` → `ParagraphFloatPlacement` → `layout_table` | 원본 LineSeg의 다음 줄과 표·캡션·바깥 여백 끝이 닫히는 경우만 수용. Top 캡션은 이미 옮긴 원점에서 다시 더하지 않는다. |
| TAC 저장 상자 | `stored_tac`의 원점·점유 끝 → 예약과 배치 계획 | 캡션이 있는 공백 줄과 없는 빈 줄을 모두 대조. 저장 줄 높이와 실제 객체 상자가 일치해야 한다. |
| 중첩 표의 reset 조각 | 원본 줄 유닛·컷 → `nested_table_mixed_fragment_heights` → 부모 `cell_units` → `table_partial`의 동일 재귀 컷 | 첫 프레임 안 여백과 마지막 프레임 안 여백을 각자 소유. 7→8쪽에서 마지막 줄, 표 경계, 뒤 문단을 함께 확인. |
| 작은 마지막 행 | `row_step`의 저장 reset 컷 → 수용 높이 → 동일 행·줄 컷 배치 | 원래 높이만 들어가는 예산과 마지막 4.2/다음 4.3의 소유를 11→12쪽에서 확인. 빈 문자열 여부로 물리 행을 생략하지 않는다. |
| 부동 표 앞 TAC 줄 | `stored_float_frame_before_tac_line` → 원본 프레임 예약 → 배치 계획 | offset 0인 부동 표의 전체 상자가 같은 host의 TAC 시작에서 닫힌다는 source 근거. offset이 있는 뒤쪽 표는 이 계약이 아니다. |
| 첫 조각 원점 | 분할 예산의 `ParagraphFloatPlacement` → 확정 원장 → `layout_partial_table`의 `resolved_table_top` | 원본 host 줄에서 앞 간격을 뺀 원점이 실제 현재 흐름과 일치할 때 사용. 뒤 배치가 앞 간격을 잃지 않게 한다. |
| 두 물리 프레임의 여백 소유 | 원본 셀 높이·다음 문단 vpos의 닫힘 → `stored_two_frame_successor_origin_hu` / `stored_cut_closes_declared_opening_frame` → `fragment::emit`의 실제 예약·끝 행 높이 → partial paint | 원본 첫 프레임 선언이 마지막 줄의 후행 간격 안에서 닫히고, 전체 셀 높이가 두 프레임·바깥 여백 합과 정확히 이어질 때만 사용. 여러 번 reset하는 정상 긴 표는 제외한다. 컷의 내용 소유는 변경하지 않는다. |
| 가시 문단 뒤 첫 행 원점 | 앞 문단 마지막 원본 vpos+줄 높이+줄간격+host 앞 간격 → `fragment::budget`의 공통 배치 계획 → 조각 높이 예산·실제 원점 | 15쪽에서 앞 문단의 후행 줄간격을 지운 흐름을 복원한다. 단일 행·병합 행·편집·재조판·저장 사다리 불일치는 이 계약이 아니다. |
| 같은 host의 TAC와 부동 표 | `stored_first_tac_line` → `stored_tac::prepare_coanchored_first_line`의 pen/end → `commit_stored_tac_control` → inline metadata를 소비하는 table paint | 원본 TAC 줄의 전체 외곽 높이가 저장 줄 높이와 같고 부동 형제가 그 줄 밖에서 시작할 때, TAC만 확정한다. 뒤 부동 표의 분할 경로를 생략하지 않는다. |
| 완전한 첫 물리 프레임 | 앞 문단의 원본 끝+host 앞 간격 → `fragment::budget`의 마지막 행 잔여 → `fragment::scan`의 행 높이와 `end_row_height_override` → partial paint | 3쪽 원문 개체 높이 49,069HU가 실제 첫 표 조각의 테두리를 닫는다. 마지막 행의 내용 하한을 수용할 때만 빈 잔여를 조정하고 컷 유닛은 보존한다. |
| 두 줄 셀의 저장 분할 경계 | 두 원본 줄 높이·사이 간격·실효 안 여백이 `cellSz`와 정확히 일치 → `native_saved_reset_cut_trailing_trim` → 일반 행 컷의 소비 높이와 `cell_cut_visible_height` | 11→12쪽 4.2/4.3의 분할. 두 0 원점만으로는 수용하지 않고, 원본 셀 높이 및 전체 저장 행 높이가 첫 개체 프레임보다 큰 근거를 확인한다. 완료된 한 줄 형제 셀도 같은 물리 조각의 끝 간격을 공유한다. |

| 중첩 float의 본체·여백 | `nested_table_body_height` → `stored_nested_float_placement` → 컷 높이 및 `table_partial`의 `occupied_bottom` | 바깥 여백을 두 번 더하지 않는다. 잘린 자식은 실제 visible flow 높이를 유지한다. |
| 표 뒤 가시 문단의 공유 경계 | `stored_after_partial_table_shared_spacing_px` → 문단 fit 회수량·실제 원점 | 저장 첫 줄 원점과 표 조각 종료 흐름이 같고 dirty가 아닐 때만 앞 간격을 공유한다. 빈 글자 여부를 공간 소유의 대용으로 쓰지 않는다. |

일반 저장 HWP5/HWPX, 편집·재조판 경로의 적용 조건을 구분한다. RowBreak는 저장 원본의
닫힘 근거를 사용하고 dirty·세션 편집·재조판이면 기존 경로로 돌아간다. TAC는 자신의 저장 줄을
확정하되 같은 host의 뒤 부동 표를 소비한 것으로 처리하지 않는다. 여러 번 reset하는 정상 긴 표를
두 프레임으로 추정했던 중간 후보는 10쪽을 9쪽으로 줄여 폐기했다. 전체 저장 높이와 단일 reset의
닫힘을 확인한 뒤 기존 #7095 검사 10개가 통과했다. 최종 head에서도 다시 검사한다.

### 마지막 행의 빈 밴드

11쪽의 첫 개체 프레임 16,840HU에서 앞 행 합 14,535HU를 빼면 마지막 행의 물리 공간은
2,305HU다. 줄 높이·안 여백만 예약하면 이 공간이 사라져 테두리와 병합 셀 정렬이 위로 이동한다.
`native_saved_two_line_row_frame`의 원본 닫힘 증거를 컷 간격과 첫 프레임 높이가 공유하며,
`fragment::emit`에서 예산 내 프레임·끝 행 높이를 확정해 partial paint가 같은 값을 사용한다.
4.2/4.3의 내용 컷은 유지하고 빈 공간을 뒤 유닛에서 빼지 않는다. 본체 높이가 예산을 넘으면
원래 작은 높이를 먼저 수용한 뒤 paint에서 늘리지 않는다.

### 중첩 표와 뒤 문단

7쪽 원본은 5,280 + offset 1,124 + 위 여백 283 + 본체 13,331 + 아래 여백 283
= 뒤 문단 원점 20,301HU로 닫힌다. 본체와 총 점유 높이를 분리한 뒤에도 partial paint가
`nested_y + table_h_rendered`로 끝을 덮어써 여백을 다시 더했다. 컷 없는 저장 float가
공통 `occupied_bottom`을 소비한 뒤 문단 원점은 511.2→507.4px, 셀 밖 글줄은 2→0개가 됐다.

5쪽 뒤 본문은 저장 첫 줄과 표 종료 흐름이 같지만 문단 앞 간격이 다시 더해졌다.
같은 좌표계의 SVG/PDF 기준선 960.85/947.52px 차이 13.33px를 공통 공유 간격으로 해소했다.
빈 문단뿐 아니라 가시 본문도 같은 물리 경계를 공유할 수 있다. dirty·저장 줄·실제 하단 조건은
유지하고 좌표 clamp로 결과를 숨기지 않는다.

## 회귀 추가 전 시각 선행 조건

Native/fresh WASM 전체 18쪽은 동일했고 최저 2쪽 90.53132%, 5쪽 96.90224%,
7쪽 93.17436%, 11쪽 95.49336%, 12쪽 96.36573%였다. 90% 미달·누락 쪽은 없고
18쪽을 유지했다. 5·7·8·11·12쪽 review/overlay 직접 판독에서 본문 간격, 셀 안 글줄,
4.2/4.3의 쪽 소유와 마지막 행 테두리·여백을 확인했다. 이 구현 상태에서 정식 회귀를 추가한다.

글리프·가로 간격과 작은 테두리 차이는 남으므로 완전한 출력 일치로 쓰지 않는다. 제공된 글꼴을
포함한 로컬 진단과 실제 적용 face를 구분했으며 글꼴 예외를 사용하지 않는다. 글꼴 원본·경로와
글꼴을 포함한 SVG·중간 JSON은 공개하지 않는다. 공개 증적은 선별한 PNG와 요약 기록으로 제한한다.
전체 회귀·lint 및 제출 source SHA의 검증 결과는 완료 후 여기에 연결한다.
