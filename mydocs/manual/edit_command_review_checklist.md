# 편집 Command/Undo 검토 체크리스트

새 rhwp-studio 편집 기능 또는 PR을 검토할 때 다음을 확인한다.

## 1. 문서 mutation 여부

- 이 액션이 문서 저장 결과를 바꾸는가?
- 문서 저장 결과를 바꾼다면 Undo/Redo 기대치가 있는 사용자 액션인가?
- 단순 조회, export, render, 보기 옵션이라면 history 밖에 두었는가?

## 2. 라우터 통과 여부

- 새 document mutation이 `executeOperation()` 또는 후속 편집 라우터를 통과하는가?
- 직접 `services.wasm.*` 또는 `this.wasm.*` mutation을 호출한다면 허용 예외인가?
- 예외라면 command 내부 저수준 호출인지, drag preview처럼 `recordApplied`가 필요한 경로인지 명확한가?

## 3. Undo/Redo payload

- redo가 `execute()` 재호출로 같은 결과를 만들 수 있는가?
- undo에 필요한 before 값이 command에 저장되는가?
- UI 표시용 JSON 전체가 아니라 core 복원에 안정적인 ID/delta/snapshot을 사용했는가?

## 4. snapshot 사용 기준

snapshot을 사용한다면 다음 중 하나에 해당해야 한다.

- paste/cut/delete처럼 여러 control/resource가 동시에 바뀐다.
- 정확한 delta command가 아직 과도하게 복잡하다.
- 실패 시 문서 복구가 어렵다.

snapshot 사용 시 operation type과 resource discard 경로도 확인한다.

## 5. refresh와 dirty scope

- mutation 후 렌더링 갱신 정책이 명확한가?
- text edit처럼 page-local refresh가 가능한가?
- full refresh가 필요한 구조 변경인가?
- selection/caret 복원 정책이 기존 UX와 일치하는가?

## 6. recordApplied 경로

IME, drag, resize처럼 이미 mutation을 적용한 뒤 history에 기록하는 경우:

- command가 before/after payload를 모두 갖고 있는가?
- `history.recordWithoutExecute()`를 직접 호출하지 않고 router를 통과하는가?
- refresh를 호출부가 이미 처리한다면 `refresh: 'none'`을 명시했는가?

## 7. 수동 판정

최소 확인:

- Undo 1회
- Redo 1회
- 같은 액션 반복 후 Undo/Redo
- selection/caret 위치
- 저장 전 dirty 상태

도메인별 추가 확인:

- 표: 셀/표 selection 복원
- 그림/도형: object selection 복원, 이동/크기/회전 유지
- 쪽/머리말/꼬리말: 페이지네이션 및 바탕쪽/필드 영향
- 필드/form: 저장 후 roundtrip 영향
