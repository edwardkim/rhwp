# #7642 저장 줄 없는 중첩 행의 재조판 내용 컷

Issue: #7642. 현재 검증한 source58b6694b6b와 독립 한컴 Print를 기준으로
원인을 추적한다. 기존 실행 중 Native/WASM 검증과 공유 target은 보존한다.

## 원본·독립 기준과 관측

`samples/issue6776/78494-virtual-convergence-industry-decree.hwpx`는 실제
HWP5이고 기준은 `pdf/78494-virtual-convergence-industry-decree-2020.pdf`다.
동일한 실제 TrueType 프로그램 프로필로 Native와 PDF를 비교했다.

본문 문단331의5행3열 표는36~38쪽에 이어진다. 그 안의4행2열 중첩 표는
RowBreak를 선언한다. 조직 칸에는 저장 LineSeg가 없고 실제 재조판 텍스트는
세 줄이다.36쪽에는 머리 행 뒤로 조직 칸의 첫 두 줄이 들어가고 마지막 줄은
37쪽 상단으로 이어지는 것이 독립 Print다. 현재 결과는 머리 행만36쪽에 두고
조직 칸 전체를37쪽으로 넘긴다. 다음 인력 칸부터 약44px 밀리고38쪽에는
앞 페이지에 있어야 할 두 줄이 남는다. 빈 공간이 있어도 실제 줄을 수용하지 못한다.

## 원인과 공통 결과

`cell_units_uncached`는 안쪽 RowBreak 행의 선언 높이가 한 줄보다도 작은
경우 등에만 세부 자식 컷을 투영한다. 이번 행은 선언3082HU가 한 줄보다
크지만 현재 세 줄 내용보다 작아 성장했다. 선언 높이는 최소 높이이며,
줄 없는 실제 재조판 내용의 분할을 막는 원자 경계가 아니다.

현재 driver의 공통 CellUnit 높이 합이 선언 행 높이를 넘고, 해당 원본 칸이
저장 줄·별도 개체 없는 재조판 텍스트이면 같은 자식 유닛 컷을 부모에 투영한다.
저장 줄이 있는 행, 내용보다 큰 선언 최소 높이의 빈 공간, 걸침 칸은 이 새
범위에 넣지 않는다. 비율·문서 ID·특정 높이로 대상을 고르지 않는다.

새 범위의 각 조각 높이는 반씩 나눈 여분 높이가 아니라 같은 시작 컷에서의
`row_cut_content_height` 누적 차이로 정한다. 생산된 `NestedTableUnitCut`의
행·시작/끝 셀 컷은 부모 스캔의 예약과 실제 부분 표 배치가 같이 소비한다.
기존 정상 저장 줄·재시작 표식·다단계 중첩 표의 경로는 보존한다.

## 실제 소비 경로와 검증

생산: LayoutEngine::cell_units_uncached → 자식의 현재 줄 CellUnit과 컷.
측정: advance_row_cut_for_physical_band → 부모의 동일 CellUnit 컷과 물리 높이.
예산 실패: scan_block_table_split_rows → 미수용 유닛을 다음 조각으로 이월.
배치: nested_table_split_from_cut_units → 같은 NestedTableCut을 부분 표에 전달.
내용 컷과 물리 패딩·간격을 구별하고 누락·중복·외곽·후속 본문을 대조한다.

독립 Print의36쪽 첫 두 줄/37쪽 마지막 줄과 원본 저장 정보를 직접 확인한
뒤 보정한다. 새 Native/fresh WASM의74쪽·전체 대조군을 같은 조건으로
확인한다. 저장 줄 대조군, 성장하지 않은 선언 행, 걸침 칸, 내용 전부 소비 후
종료 경계는 실제 적용 여부를 구분하여 검사한다. 전체90%·확대 판독 전에는
기존 회귀 기대값을 바꾸거나 새 rendering 회귀를 추가하지 않는다.

## 빌드 순서

현재58b6694b6b의 WASM은 계속 실행한다. b14e28ec25 전용으로 기다리던
두 Python dispatcher는 child와 Cargo 실행이 없음을 확인한 뒤 새 내용과
묶기 위해 종료했고 영수증을 superseded_before_build로 남겼다. 실제 빌드나
시각 캡처는 중단하지 않았다. KoPubWorld와 이번 보정을 다음 고정 source
한 배치로 순차 빌드·전체 검증한다. 새 build/source/TSV가 없으면 완료가 아니다.
