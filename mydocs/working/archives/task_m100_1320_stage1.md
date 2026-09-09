# Task M100-1320 Stage 1 — 편집 액션/Undo 라우팅 현황 인벤토리

## 범위

이번 단계에서는 구현을 변경하지 않고, rhwp-studio의 편집 mutation 경로와 Undo/Redo 기록 경로를
인벤토리했다.

조사 대상:

- `rhwp-studio/src/engine/history.ts`
- `rhwp-studio/src/engine/command.ts`
- `rhwp-studio/src/engine/input-handler*.ts`
- `rhwp-studio/src/command/commands/*.ts`

관점:

- `executeOperation({ kind: 'command' })`
- `executeOperation({ kind: 'snapshot' })`
- `executeOperation({ kind: 'record' })`
- `history.recordWithoutExecute()`
- 직접 `wasm.*` mutation 호출
- 문서 mutation이 아닌 view/query 호출

## 현재 구조 요약

이미 존재하는 골격은 다음과 같다.

- `CommandHistory`
  - 하나의 undo stack과 redo stack을 관리한다.
  - `redo()`는 redo stack의 command에 대해 `execute()`를 다시 호출한다.
  - `recordWithoutExecute()`는 이미 문서에 적용된 mutation을 history에만 기록한다.
- `EditCommand`
  - `execute()`, `undo()`, `mergeWith()`, 선택적 `discard()` 계약을 갖는다.
- `OperationDescriptor`
  - `command`, `snapshot`, `record` 세 경로를 제공한다.
- `InputHandler.executeOperation()`
  - `command`와 `snapshot`은 실행, cursor 이동, refresh까지 처리한다.
  - `record`는 history 기록만 하고 refresh는 호출부 책임으로 남아 있다.
- `SnapshotCommand`
  - paste, object/table 삭제 같은 복합 조작을 before/after snapshot으로 되돌린다.

판단:

현재 구조는 통합 라우터의 뼈대는 갖고 있다. 다만 모든 mutation이 이 입구를 통과하지 않고, 직접
WASM mutation, record-after-mutation, snapshot이 혼재되어 있어 기능별 Undo/Redo 사용자 경험이
달라질 수 있다.

## 라우팅 유형

| 유형 | 현재 사용처 | 장점 | 위험 |
|---|---|---|---|
| `command` | 텍스트 입력/삭제, 문단 분할/병합, 글자/문단 서식 | redo가 `execute()` 재호출로 명확하고 병합 가능 | command가 before/after payload를 정확히 저장해야 함 |
| `snapshot` | paste, object/table cut/delete 등 복합 조작 | 복잡한 구조 변경을 빠르게 Undo/Redo 가능 | snapshot 비용, dirty scope 부재, operation 의미가 흐려질 수 있음 |
| `record` | IME 완료, 표 이동, 개체 이동/크기 조절 일부 | 이미 적용된 interactive mutation을 하나의 history로 묶을 수 있음 | refresh와 mutation 성공 여부가 호출부에 흩어짐 |
| 직접 mutation | page/header-footer/field/insert/object 일부 | 구현이 단순함 | Undo/Redo 누락, 동일 UX 기대치 불일치 |
| view/query | 검색, 렌더링, 보기 옵션, hit-test, cursor 조회 | history 대상이 아님 | 문서 mutation과 섞여 보이면 오분류 위험 |

## 이미 정돈된 경로

### 텍스트 계열

`input-handler-text.ts`, `input-handler-keyboard.ts`, `command.ts` 기준으로 다음은 command 경로를 탄다.

- 일반 텍스트 입력
- Backspace/Delete 단일 삭제
- 문단 병합/분할
- 셀 내부 문단 병합/분할
- 강제 줄넘김
- 탭 입력

IME composition 완료는 예외적으로 이미 문서에 들어간 텍스트를 `InsertTextCommand`로
`record`한다. 이 패턴은 interactive 입력 특성상 허용 가능하지만, "record는 mutation 후 기록"이라는
계약을 명확히 해야 한다.

### 글자/문단 서식

#1319에서 다음 경로가 command화되었다.

- `ApplyCharFormatCommand`
- `ApplyParaFormatCommand`
- `setCharShapeId`
- `setCharShapeIdInCell`
- `setParaShapeId`
- `setCellParaShapeId`

이는 이번 #1320에서 표준 사례로 삼을 수 있다. 특히 UI 표시용 JSON 전체를 저장하는 대신, core가
되돌릴 수 있는 shape ID 또는 before/after payload를 저장하는 방향이 적절하다.

### 복합 paste/delete

다음은 snapshot 경로를 사용한다.

- object/table cut
- object/table delete
- control paste
- internal/html/image paste

현재 단계에서는 snapshot을 제거할 대상이 아니라, 허용 조건을 문서화할 대상이다.

## 혼재된 경로

### record-after-mutation

표와 그림/도형 조작에서 다음 흐름이 보인다.

1. UI drag/key 조작 중 `wasm.*` mutation을 먼저 호출한다.
2. 변경량을 계산한다.
3. `MoveTableCommand`, `MovePictureCommand`, `MoveShapeCommand`, `ResizeObjectCommand`를 history에 기록한다.
4. `document-changed` emit과 selection refresh를 호출부가 처리한다.

대표 위치:

- `input-handler-table.ts`
  - `moveSelectedTable()`
  - `finishMoveDrag()`
- `input-handler-picture.ts`
  - resize finish 경로
  - keyboard move 경로
  - `finishPictureMoveDrag()`

특히 `finishPictureMoveDrag()`는 `executeOperation({ kind: 'record' })`를 거치지 않고
`this.history.recordWithoutExecute()`를 직접 호출한다. 작은 수정 후보이지만, 현 `record` 경로가
refresh를 처리하지 않기 때문에 단순 치환만으로는 구조 개선 효과가 제한적이다.

## 직접 mutation 경로

사용자 UX 영향도가 큰 직접 mutation 경로는 다음과 같다.

### 쪽/머리말/꼬리말/다단

`rhwp-studio/src/command/commands/page.ts`

- `createHeaderFooter()`
- `insertFieldInHf()`
- `applyHfTemplate()`
- `deleteHeaderFooter()`
- `toggleHideHeaderFooter()`
- `insertPageBreak()`
- `setPageHide()`
- `insertColumnBreak()`
- `setColumnDef()`

이 경로들은 사용자가 명령으로 실행하는 문서 mutation이지만, 현재 history command 또는 snapshot으로
통합되어 있지 않다. 쪽 나누기, 단 나누기, 다단 설정, 머리말/꼬리말 편집은 사용자가 Undo/Redo를
기대할 가능성이 높으므로 우선순위가 높다.

### 삽입/개체 조작

`rhwp-studio/src/command/commands/insert.ts`

- `insertEquation()`
- `insertFootnote()`
- `insertEndnote()`
- `changeShapeZOrder()`
- `deleteShapeControl()`
- `deleteEquationControl()`
- `deletePictureControl()`
- `deleteCellPictureControlByPath()`
- `groupShapes()`
- `ungroupShape()`
- object/picture property setters

수식, 각주, 미주, z-order, 삭제, 묶기/풀기는 사용자의 편집 결과를 바꾸는 명령이므로 history 정책에
편입해야 한다. 다만 각주/미주/묶기/풀기는 구조 변경 폭이 커서 snapshot 또는 별도 command 설계가
필요하다.

### 필드와 폼 값

`rhwp-studio/src/command/commands/edit.ts`, `input-handler.ts`

- `updateClickHereProps()`
- `removeFieldAt()`
- form value setter 계열
- `setActiveField()`/`clearActiveField()`

필드 속성과 form value는 문서 저장 결과에 영향을 줄 수 있으므로 history 대상이다. 반면 active field
상태는 UI focus/selection 상태일 수 있어 문서 mutation인지 별도 판정이 필요하다.

### 스타일/번호/글머리표

`input-handler.ts`

- `applyStyle()`
- `applyCellStyle()`
- `ensureDefaultNumbering()`
- `ensureDefaultBullet()`

스타일 적용은 문단/글자 서식과 같은 사용자 기대치를 가진다. 번호/글머리표 기본 생성은 사용자 명령의
부수 mutation이므로 독립 history 항목보다는 실제 적용 command와 transaction으로 묶는 편이 좋다.

### 표 편집 명령

`rhwp-studio/src/command/commands/table.ts`

- 표 생성
- 행/열 삽입과 삭제
- 셀 병합/나누기
- 표 삭제
- 표 속성
- 셀 크기 변경
- 셀 문자열 변환

일부는 snapshot으로 감싸고 일부는 직접 mutation한다. 표 구조 편집은 사용자 영향도가 크지만 delta
설계가 복잡하므로, 1차로는 snapshot 허용 대상과 command 전환 대상을 분리해야 한다.

## History 밖에 두어야 할 경로

다음은 문서 mutation이 아니므로 Undo/Redo history에 넣지 않는 것이 맞다.

- 검색과 조회
- SVG/PNG/export
- page info, document info, section/page border 조회
- hit-test, selection rect, cursor rect 조회
- 보기 옵션
  - control code 표시
  - 문단 부호 표시
  - 투명 테두리 표시
  - clip 표시

단, 보기 옵션이 문서 저장 상태에 반영되는 기능으로 바뀐다면 별도 분류가 필요하다.

## 우선순위 제안

### P0 — UX 기대치가 크고 직접 mutation인 명령

- 쪽 나누기 / 단 나누기
- 다단 설정
- 머리말/꼬리말 생성, 삭제, 템플릿, 필드 삽입
- 수식/각주/미주 삽입
- 필드 속성 변경과 form value 변경

### P1 — 이미 history는 있지만 라우팅 계약이 약한 경로

- 그림/도형 drag 이동
- 그림/도형 resize
- 표 drag/key 이동
- IME composition record

이 범주는 `record` 경로의 의미와 refresh 책임을 표준화하면 안정성이 오른다.

### P2 — snapshot 허용 조건 명문화

- paste
- object/table delete
- 표 구조 편집 일부
- 각주/미주/머리말 같은 복합 구조 편집

snapshot은 제거 대상이 아니라 "정밀 command로 만들기 전까지 허용되는 복합 편집 전략"으로 명문화해야
한다.

### P3 — view/query 분리

보기 옵션과 query API는 history 대상에서 제외한다. 이 분류를 명문화하면 이후 직접 `wasm.*` 호출
검색에서 false positive를 줄일 수 있다.

## Stage 1 판단

현재 설계는 "기존 Undo/Redo와 충돌하지 않고 확장할 수 있는 골격"은 갖고 있다. 그러나 사용자 경험
목표를 만족하려면 다음이 부족하다.

- mutation 진입점이 하나로 수렴되지 않았다.
- `recordWithoutExecute()` 사용 조건이 코드 주석 수준에 머문다.
- `record` 경로의 refresh/dirty scope 책임이 불명확하다.
- 직접 mutation 명령의 Undo/Redo 기대치가 기능별로 다르다.
- snapshot 사용 기준과 resource discard 기준은 있으나, 어떤 편집에 허용되는지 제품 규칙이 없다.

따라서 #1320은 대규모 재작성보다 다음 순서가 적절하다.

1. 복원 계약 문서를 먼저 만든다.
2. `command`, `snapshot`, `record`, `view/query` 분류 기준을 명문화한다.
3. 안전한 직접 mutation 일부를 router/snapshot 또는 domain command로 이동한다.
4. 표/이미지/쪽/필드처럼 큰 범위는 후속 세부 이슈로 나누되, 이번 이슈에서 routing 규칙을 고정한다.

## 향후 확장성 보완 검토

Stage 1 기준 설계에서 향후 확장성을 위해 보완해야 할 점은 다음과 같다.

### 1. `CommandServices`에 편집 라우터가 없다

현재 command registry의 `CommandDef.execute()`는 `CommandServices`를 통해 `wasm`에 직접 접근할 수 있다.
이 구조는 새 메뉴/툴바/확장 명령을 추가하기는 쉽지만, 새 명령 작성자가 Undo/Redo 정책을 우회하기도
쉽다.

보완 방향:

- `CommandServices`에 문서 mutation용 router API를 추가한다.
- command module은 가능하면 `services.wasm.*` mutation을 직접 호출하지 않고 router에 작업을 위임한다.
- `services.wasm` 직접 접근은 조회, export, view 설정, 또는 명시적으로 허용된 저수준 command 구현 내부로
  제한한다.

예상 효과:

- 메뉴, 툴바, 단축키, 확장 command가 같은 history 정책을 공유한다.
- 향후 플러그인/확장 명령이 늘어도 Undo/Redo 누락 가능성을 줄인다.

### 2. `OperationDescriptor` 메타데이터가 부족하다

현재 `OperationDescriptor`는 `command`, `snapshot`, `record`만 구분한다. 기능이 늘어나면 다음 정보가
필요해진다.

- action id 또는 operation type
- domain: text, charFormat, paraFormat, table, object, page, field 등
- history policy: command, snapshot, record-applied, none
- refresh policy: full document, page-local, selection-only, none
- dirty scope: section/page/paragraph/table/object 단위
- selection/caret restore policy
- merge/coalesce key
- 실패 시 no-op 처리 규칙

보완 방향:

- `OperationDescriptor` 또는 후속 router descriptor에 위 메타데이터를 단계적으로 추가한다.
- 처음부터 모든 필드를 강제하지 말고 기본값을 제공하되, mutating command에는 history/refresh 정책을
  명시하게 한다.

### 3. command 결과가 cursor 하나로 제한되어 있다

현재 `EditCommand.execute()`와 `undo()`는 `DocumentPosition`만 반환한다. 그러나 표/이미지/쪽/필드로
확장하면 command 결과는 cursor 위치만으로 부족하다.

필요한 결과 정보:

- cursor 위치
- selection 복원 또는 유지 여부
- dirty scope
- 렌더링 refresh 범위
- 후속 UI 이벤트
- object/table selection 재선택 정보
- command가 실제 mutation을 했는지 여부

보완 방향:

- 즉시 기존 인터페이스를 깨기보다, router 단계에서 `DocumentPosition`을 `CommandResult`로 감싸는
  호환 레이어를 둔다.
- 장기적으로 `EditCommand`는 `CommandResult`를 반환하도록 이전한다.

### 4. record-after-mutation 계약이 확장에 취약하다

drag, IME, resize처럼 이미 mutation이 적용된 뒤 history만 기록하는 경로는 필요하다. 문제는 현재
`record`가 refresh와 dirty 처리를 책임지지 않는다는 점이다.

보완 방향:

- `record`를 `recordApplied`처럼 의미가 분명한 이름으로 정리한다.
- `recordApplied`는 반드시 다음을 요구한다.
  - 이미 적용된 mutation이라는 명시
  - Undo/Redo에 필요한 before/after payload
  - refresh policy
  - dirty scope
- 직접 `history.recordWithoutExecute()` 호출은 예외로 두고, 일반 호출부는 router를 통과시킨다.

### 5. 복합 편집을 묶는 transaction 모델이 필요하다

번호/글머리표 기본 생성 후 문단 적용, 머리말/꼬리말 템플릿 적용 후 편집 모드 진입, 표 구조 편집,
필드 삽입처럼 하나의 사용자 액션이 여러 mutation으로 구성되는 경우가 있다.

보완 방향:

- `TransactionCommand` 또는 `EditTransaction` 계층을 도입한다.
- transaction은 여러 domain command 또는 snapshot을 하나의 Undo/Redo 항목으로 묶는다.
- transaction 내부에는 rollback 또는 snapshot fallback을 허용한다.

즉시 필요한 예:

- 번호/글머리표 리소스 생성 + 문단 서식 적용
- 머리말/꼬리말 생성 + 필드 삽입
- 표 행/열 삽입 + selection 복원
- 이미지 삽입 + 셀 높이 조정

### 6. snapshot 사용 기준과 비용 관리가 필요하다

snapshot은 복잡한 구조 변경에 현실적인 해법이다. 하지만 사용처가 늘어나면 메모리 비용과 dirty scope가
문제가 된다.

보완 방향:

- snapshot 허용 기준을 명문화한다.
  - 구조 변경 폭이 크고 정확한 delta가 아직 없는 경우
  - paste/cut/delete처럼 여러 control과 리소스가 동시에 바뀌는 경우
  - command 실패 시 복구가 어려운 경우
- snapshot command에는 operation type, 설명 label, discard 보장, 크기 제한 정책을 둔다.
- 장기적으로 core가 section/page/object 단위 snapshot을 지원할 수 있는지 검토한다.

### 7. 도메인별 before/after payload 표준이 필요하다

모든 편집을 같은 payload로 처리하면 확장성이 떨어진다. 도메인별로 안정적인 최소 payload를 정해야 한다.

권장 payload 예:

- text: 위치, 삽입/삭제 문자열
- char format: range, before/after charShapeId 또는 property delta
- para format: target para, before/after paraShapeId
- table move/resize: table ref, before/after offset 또는 cell size
- object transform: object ref, before/after bbox/offset/size/rotation
- page control: target section/paragraph, before/after control snapshot
- field/form: field id/path, before/after props 또는 value

### 8. 문서 상태와 보기 상태 분리가 더 명시적이어야 한다

현재 보기 옵션과 문서 mutation이 모두 `wasm.*` 호출로 보이기 때문에 검색만으로는 구분이 어렵다.

보완 방향:

- router에 `mutatesDocument: true/false` 개념을 둔다.
- `view/query` command는 history 밖에 두고, document dirty 상태를 변경하지 않게 한다.
- `setActiveField()`처럼 UI state인지 document state인지 애매한 API는 분류를 재검토한다.

### 9. 회귀 방지용 감사 도구가 필요하다

향후 기능이 늘어나면 직접 `wasm.*` mutation이 다시 추가될 수 있다.

보완 방향:

- `rhwp-studio/src/command/commands`와 `input-handler*.ts`에서 mutation성 `wasm.*` 호출을 탐지하는
  정적 점검 스크립트를 둔다.
- 허용 목록을 관리한다.
- PR 검토 시 "새 document mutation은 router를 통과하는가"를 체크리스트로 삼는다.

### 10. 최종 판단

현재 구조는 "기능 추가가 가능한 구조"이지만, "기능이 많이 늘어나도 일관된 편집 사용자 경험을 보장하는
구조"라고 보기에는 아직 부족하다. #1320에서 바로 모든 편집 기능을 transaction화하기보다는, 다음
확장 포인트를 먼저 고정하는 것이 좋다.

1. mutation router를 `InputHandler` 내부 구현에서 `CommandServices`가 사용할 수 있는 편집 서비스로 승격
2. `OperationDescriptor`에 history/refresh/dirty/selection 정책 추가
3. `recordApplied` 계약 명문화
4. snapshot 허용 기준 명문화
5. 직접 mutation 금지 원칙과 허용 예외 목록 작성
6. 작은 mutation부터 router로 이전

## Stage 2 권장안

다음 단계에서는 구현 전에 `mydocs/plans/task_m100_1320_impl.md`를 작성한다.

포함할 내용:

- `OperationDescriptor` 계약 재정의
- `record` 허용 조건
- snapshot 허용 조건
- direct mutation 금지/예외 기준
- refresh/dirty scope 기본 정책
- 1차 코드 변경 후보
  - `finishPictureMoveDrag()`의 직접 `history.recordWithoutExecute()` 제거 또는 router 통합
  - `record` 경로 refresh 정책 보강
  - 쪽 나누기/단 나누기 같은 작은 명령을 snapshot 또는 command 경로로 이동할지 판단

## 승인 요청

Stage 2에서 복원 계약 표준안과 1차 구현 계획서를 작성한다.
