# Task M100 #1443 구현계획서

- 이슈: #1443
- 수행계획서: `mydocs/plans/task_m100_1443.md`
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `local/task_m100_1443`
- 작성일: 2026-06-19

## 1. Stage 1 목표

한컴처럼 표 셀 선택 상태에서 마우스 드래그로 셀 블록 범위를 확장할 수 있게 한다.

이번 단계는 마우스 셀 블록 선택에 집중한다. Alt+방향키와 모양복사는 이 단계에서 구현하지 않고, Stage 1 검증
후 후속 단계로 분리한다.

## 2. 관련 코드

| 파일 | 역할 |
|---|---|
| `rhwp-studio/src/engine/input-handler.ts` | 드래그 상태 필드, `hitTestCellRowCol`, `updateCellSelection` |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | `mousedown`, `mousemove`, `mouseup` 처리 |
| `rhwp-studio/src/engine/cursor.ts` | 셀 선택 anchor/focus 상태 |
| `rhwp-studio/src/engine/cell-selection-renderer.ts` | 선택 범위 하이라이트 렌더링 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | F5/방향키/셀 크기 조정 기존 경로 |

## 3. 구현 방향

### 3.1 셀 드래그 상태 추가

`InputHandler`에 셀 선택 드래그 전용 상태를 추가한다.

- 시작 셀 row/col
- 마지막 focus row/col
- 드래그 시작 client 좌표
- 실제 드래그로 인정할 이동 threshold
- 보호 셀 클릭 선택으로 시작했는지 여부

텍스트 선택용 `isDragging`과 구분하여, 셀 블록 선택은 별도 상태로 둔다.

### 3.2 드래그 시작

`input-handler-mouse.ts`의 셀 선택 모드 클릭 처리에서 다음 순서를 유지한다.

1. 우클릭은 기존처럼 선택 유지
2. Shift/Ctrl/Cmd 클릭은 기존 범위/토글 동작 유지
3. 표 경계선 클릭은 기존 리사이즈 또는 표 객체 선택 유지
4. 일반 좌클릭이 셀 내부이면 셀 선택 드래그 후보 시작
5. 셀 밖 일반 좌클릭은 기존처럼 셀 선택 모드 종료

보호 셀 클릭 경로는 #493의 `selectProtectedCell` 흐름을 유지하되, 좌클릭 이후 마우스 이동 시 같은 드래그 상태를
사용할 수 있게 한다.

### 3.3 드래그 중 범위 갱신

`onMouseMove` 초반 드래그 처리 구간에 셀 선택 드래그 분기를 추가한다.

- 마우스가 threshold 이상 움직이면 실제 셀 블록 드래그로 확정한다.
- 현재 포인터 위치를 `hitTestCellRowCol`로 같은 표의 row/col로 변환한다.
- focus가 바뀐 경우 cursor의 선택 focus를 갱신하고 `updateCellSelection()`을 호출한다.
- 포인터가 같은 표 밖으로 나가면 마지막 유효 선택을 유지한다.

기존 `CellSelectionRenderer`를 그대로 쓰므로 새 렌더러는 만들지 않는다.

### 3.4 드래그 종료

`onMouseUp`에서 셀 선택 드래그 상태를 정리한다.

- 실제 드래그가 아니었던 단순 클릭은 기존 단일 셀 선택을 유지한다.
- 실제 드래그였던 경우 선택 범위를 유지하고 textarea focus를 복원한다.
- document-level mouseup listener는 기존 표/텍스트 드래그 패턴과 같은 방식으로 정리한다.

### 3.5 CursorState 보강

현재 `shiftSelectCell(row, col)`은 anchor를 유지하고 focus만 이동한다. 마우스 드래그에서도 같은 동작을
재사용할 수 있지만, 의도를 분명히 하기 위해 필요하면 다음 작은 메서드를 추가한다.

- `setCellSelectionFocus(row, col)`

이 메서드는 excluded cells를 지우고 focus만 갱신한다. 기존 `shiftSelectCell`은 이 메서드를 호출하도록 정리한다.

## 4. 구현 제외

Stage 1에서는 다음을 구현하지 않는다.

- Alt+방향키 셀 크기 조정
- Alt+C 모양복사
- 셀 너비/높이 같게의 선택 범위 기준 재설계
- 중첩 표 전용 마우스 드래그 선택 확장

단, 변경 중 기존 중첩 표 처리를 훼손하지 않는다.

## 5. 수동 검증 체크리스트

- 보호 셀 클릭 후 같은 열 아래로 드래그하면 여러 셀이 하이라이트된다.
- F5 셀 선택 모드에서 다른 셀까지 드래그하면 범위가 하이라이트된다.
- 드래그 선택 후 Ctrl/Cmd+방향키로 선택 셀 크기 조정이 동작한다.
- 셀 내부 텍스트 드래그 선택은 기존처럼 텍스트 선택으로 남는다.
- 표 경계선 드래그 리사이즈는 기존처럼 동작한다.
- 보호 셀에 문자 입력은 계속 차단된다.
