# Stage 234: Label-response rowspan frame 범위

## 목적

Stage 233의 source-band 수용이 `samples/kps-ai.hwp`의 #1073 nested-table page
65/66 경계를 앞당긴 회귀를 제거하면서, 1741000의 한글 COM 2쪽 pin을 유지한다.

## 분리 결과

- Stage 233 뒤 전체 integration은 `issue_1073_nested_table_split`에서 실패했다.
  page 65의 `소프트웨어사업` 제목이 사라져, 표의 첫 조각이 한 physical page
  앞당겨졌음을 확인했다.
- Stage 232와 Stage 233 scanner를 대조하면 원인은 `kps-ai.hwp` `pi=443`,
  `rows 8..11` block이다. 선언 frame은 `243.3px`, 남은 band는 `237.3px`이며
  source cut은 `56.0px`로 완결된다. Stage 233은 content fit만으로 이 block을
  수용해, 뒤의 nested table이 기대한 65/66 page owner를 잃었다.
- 해당 kps-ai block은 rowspan label 오른쪽에 네 개의 독립 평가 열이 있는 grid다.
  선언 blank는 표 전체의 column frame 일부이므로 content만으로 제거할 수 없다.
- 반대로 1741000 `rows 10..12`는 rowspan label 하나와, 각 행에서 label의 오른쪽
  폭 전체를 차지하는 response cell 하나만 있는 label-response form이다. 이 구조의
  아래 blank만 source content가 남은 band에 완결될 때 physical tail을 소유하지 않는다.

## 구현

- Stage 233의 일반 rowspan 조건을 `label-response form` 구조로 강화한다.
- label은 block 시작 행에서 block 전체를 rowspan으로 덮는다.
- block의 각 행에는 label을 제외해 정확히 하나의 response cell만 있고, 그 cell은
  label 오른쪽에서 시작해 남은 모든 table column을 덮어야 한다.
- 다열 평가 grid, nested/control, source LineSeg 누락, hard break는 기존 scanner
  경로를 유지한다. 픽셀 허용치·문서 식별자·행 수 조건은 사용하지 않는다.

## 검증 범위

- `issue_2097_squeeze`: 1741000을 포함한 한글 COM 세 page-count pin.
- `issue_1073_nested_table_split`: kps-ai page 65 첫 조각 제목과 page 66 continuation.
- Stage 232/233의 #3820 집중 gate 및 전체 lib/integration suite.
