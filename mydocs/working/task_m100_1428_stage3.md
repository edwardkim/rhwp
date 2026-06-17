# Task 1428 Stage 3

## 목표

- 마우스로 누름틀 값 바깥을 클릭했을 때 한컴처럼 누름틀 외부 위치로 커서가 이동하도록 한다.

## 문제 인식

- Stage 2에서 마우스로 빈 누름틀 guide 내부에 들어가는 경로를 보정했다.
- 반대로 값이 있는 누름틀 끝 경계 주변을 마우스로 클릭하면 `getFieldInfoAt()`이 end 위치를 필드 내부로 판정하여 누름틀 밖으로 나가지 못할 수 있다.
- 한컴 기준으로 누름틀 오른쪽 바깥을 클릭하면 같은 charOffset이라도 필드 외부 위치로 취급되어야 한다.
- 앞 누름틀 값과 뒤 빈 누름틀이 붙어 있으면 같은 charOffset에서 앞 누름틀 end와 뒤 누름틀 guide start가 겹치므로, 빈 guide를 우선하지 않으면 두 번째 누름틀에 마우스로 들어가지 못한다.

## 구현 방침

- 마우스 클릭 좌표와 누름틀 시작/끝 마커 좌표를 비교해, 클릭이 누름틀 오른쪽 바깥이면 `fieldEndExitKey`를 설정한다.
- 기존 guide 내부 클릭 보정은 유지하되, 값이 있는 필드 끝 바깥 클릭은 활성 필드를 해제한다.
- 키보드 방향키로 들어가고 나오는 기존 경계 상태 모델과 같은 `fieldBoundaryKey`를 재사용한다.
- 같은 charOffset에 빈 guide가 있으면 값 있는 앞 누름틀 경계 이탈 판정보다 guide 진입을 우선한다.

## 구현 결과

- 일반 마우스 클릭 경로에서 page-local x 좌표를 `prepareClickHerePointerEntry()`로 전달하도록 했다.
- 값이 있는 ClickHere의 시작/끝 caret 좌표와 클릭 x 좌표를 비교해 왼쪽/오른쪽 바깥 클릭이면 각각 `fieldStartExitKey`/`fieldEndExitKey`를 설정하도록 했다.
- 필드 시작/끝 caret 좌표 계산을 `getClickHereBoundaryRects()` 헬퍼로 분리해 마커 표시와 마우스 경계 판정이 같은 좌표 계산을 쓰도록 했다.
- 인접 누름틀의 공유 charOffset에서는 빈 ClickHere guide 진입을 먼저 판정하고, active field가 바뀌면 즉시 `document-changed`를 발생시켜 guide 숨김/마커 갱신이 렌더에 반영되도록 했다.

## 검증 계획

- 누름틀 값 `123`의 오른쪽 바깥을 마우스로 클릭했을 때 캐럿이 누름틀 밖으로 이동하는지 확인한다.
- 누름틀 값 내부 클릭과 빈 guide 내부 클릭은 계속 누름틀 내부로 들어가는지 확인한다.
- `123입력하세요`처럼 첫 누름틀 값 뒤에 두 번째 빈 누름틀이 붙은 상황에서 두 번째 guide를 마우스로 클릭해 내부 진입되는지 확인한다.
- `npx tsc --noEmit`, `npm test`, `git diff --check`를 수행한다.

## 검증 결과

- `npx tsc --noEmit`: 통과.
- `npm test`: 통과. `rhwp-studio/tests/*.test.ts` 전체 75개 통과.
- `git diff --check`: 통과.

## 시각 판단 대기

- 마우스로 누름틀 오른쪽 바깥으로 이동되는지 확인이 필요하다.
- 첫 누름틀 값 뒤에 두 번째 빈 누름틀이 붙은 상황에서 두 번째 guide 클릭 진입이 정상인지 확인이 필요하다.
