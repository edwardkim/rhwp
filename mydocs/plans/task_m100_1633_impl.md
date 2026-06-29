# Task M100 #1633 구현 계획서

## 목표

rhwp-studio `셀 테두리/배경 > 대각선` 탭이 선택 셀에 실제로 보이는
대각선/중심선 속성을 초기 상태로 표시하게 한다.

## Stage 1. effective border fill 조회

1. `src/document_core/commands/table_ops.rs`에 선택 셀의 row/col/span을 기준으로
   해당 셀을 덮는 `TableZone`을 찾는 헬퍼를 추가한다.
2. `cell.border_fill_id`보다 `cellzone.border_fill_id`를 UI 조회용 effective 값으로
   우선한다.
3. zone이 여러 개 겹치면 렌더러가 zone을 순서대로 그리는 동작과 맞춰 마지막
   matching zone을 우선한다.

## Stage 2. 회귀 테스트

1. `tests/issue_1623_cellzone_diagonal.rs`에 #1633 테스트를 추가한다.
2. `samples/대각선샘플.hwp`와 `samples/대각선샘플.hwpx`에서 row=2, col=2 또는
   row=2, col=3 셀의 `getCellProperties` 결과가 `bf=11` 대각선 값을 반환하는지
   확인한다.
3. 같은 샘플에서 cellzone 밖 셀은 기존 셀 고유 border fill 기준으로 반환되는지
   최소 확인한다.

## Stage 3. UI 확인

1. rhwp-studio TypeScript 타입/빌드를 확인한다.
2. dev 서버 또는 테스트 스크립트로 `CellBorderBgDialog`가
   `diagonalSlash`/`diagonalBackSlash` 값을 버튼과 미리 보기에 반영하는지 확인한다.

## Stage 4. 꺾은 대각선 렌더링 보정

1. `대각선샘플` 첫 줄 두 번째 칸의 `Crooked=2` 대각선이 한컴 PDF와 다르게 단순 직선으로
   표시되는 문제를 확인한다.
2. HWPX 파서가 `Crooked="2"`를 bool이 아닌 2비트 값으로 보존하도록 보강한다.
3. HWPX serializer가 `Crooked="2"`를 다시 출력하도록 보강한다.
4. 렌더러가 꺾은 대각선을 3개 선분으로 출력하도록 보강한다.
5. 샘플 SVG/PNG 비교와 focused 테스트로 회귀를 막는다.

## 리스크

- 조회 API가 effective 값을 반환하면, 사용자가 확인을 눌렀을 때 selected cell에
  effective border fill 기반 새 border fill이 설정될 수 있다. 기존 cellzone overlay
  편집 UX와는 별도 문제이므로 이번 작업에서는 초기 표시 정합을 우선한다.
- cellzone 전체를 한 번에 수정하는 UX가 필요하면 후속 이슈로 분리한다.
- 한컴의 `셀 테두리/배경 - 하나의 셀처럼 적용` 메뉴는 여러 셀 선택 때만 활성화된다.
  rhwp-studio 컨텍스트 메뉴도 같은 정책을 따라야 하며, 이는 다음 스테이지에서 분리해 처리한다.
