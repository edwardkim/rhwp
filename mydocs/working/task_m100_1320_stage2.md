# Task M100-1320 Stage 3 — 편집 라우터/Undo 계약 1차 구현 보고

## 구현 범위

Stage 2 구현 계획 승인에 따라 다음 범위만 1차 구현했다.

- Operation metadata 타입 추가
- `executeOperation()` refresh/record 계약 보강
- 그림/도형 이동 drag 종료 시 직접 `history.recordWithoutExecute()` 호출 제거
- 쪽 나누기/단 나누기 직접 mutation을 snapshot 라우팅으로 전환
- 편집 command/Undo 검토 체크리스트 작성

이번 단계에서는 다음은 의도적으로 하지 않았다.

- 모든 직접 `wasm.*` mutation 제거
- `EditCommand.execute()` 반환 타입 전면 교체
- `record`를 코드 전체에서 `recordApplied`로 rename
- 표/필드/머리말/꼬리말 전체 transaction화

## 코드 변경

### 1. Operation metadata

파일:

- `rhwp-studio/src/engine/command.ts`

추가:

- `EditDomain`
- `RefreshPolicy`
- `DirtyScope`
- `SelectionPolicy`
- `OperationMetadata`
- `OperationDescriptor.meta`

기존 호출부 호환을 위해 `meta`는 optional로 두었다.

### 2. `executeOperation()` refresh/record 정책

파일:

- `rhwp-studio/src/engine/input-handler.ts`

변경:

- `record` 경로에서 `history.recordWithoutExecute(desc.command, this.wasm)`를 호출하도록 변경했다.
  - redo stack snapshot resource discard 경로를 유지하기 위함.
- `refreshAfterOperation()` helper를 추가했다.
- 기본 refresh 정책은 기존 동작을 유지한다.
  - `command`: `auto`
  - `snapshot`: `full`
  - `record`: `none`
- metadata가 있으면 `full`, `pageLocal`, `selectionOnly`, `none`을 명시적으로 적용할 수 있다.

### 3. 그림/도형 drag 이동 record 라우팅

파일:

- `rhwp-studio/src/engine/input-handler-picture.ts`

변경:

- `finishPictureMoveDrag()`의 직접 `this.history.recordWithoutExecute()` 호출을
  `this.executeOperation({ kind: 'record', ... })`로 변경했다.
- metadata:
  - `domain: 'object'`
  - `refresh: 'none'`
  - `dirtyScope: 'object'`

기존 drag 중 `document-changed` emit과 selection refresh 구조는 유지했다.

### 4. 쪽/단 나누기 snapshot 라우팅

파일:

- `rhwp-studio/src/command/commands/page.ts`

변경:

- `page:break`
  - 직접 `services.wasm.insertPageBreak()` 호출 후 event emit하던 구조를
    `ih.executeOperation({ kind: 'snapshot', operationType: 'pageBreak', ... })`로 변경했다.
- `page:column-break`
  - 직접 `services.wasm.insertColumnBreak()` 호출 후 event emit하던 구조를
    `ih.executeOperation({ kind: 'snapshot', operationType: 'columnBreak', ... })`로 변경했다.

판단:

- 쪽/단 나누기는 사용자 Undo 기대치가 큰 직접 mutation이다.
- 정확한 delta command를 만들기 전에는 snapshot fallback이 가장 안전하다.

### 5. 검토 체크리스트

파일:

- `mydocs/manual/edit_command_review_checklist.md`

내용:

- document mutation 여부
- 라우터 통과 여부
- Undo/Redo payload
- snapshot 허용 기준
- refresh/dirty scope
- recordApplied 경로
- 수동 판정 항목

### 6. 표 삭제 후 레이아웃 흐름 재계산 보강

작업지시자 동작 테스트 중 다음 추가 문제가 확인되었다.

- 문단과 문단 사이에 있는 표를 삭제하면 표 아래 문단의 레이아웃 흐름이 위로 당겨지지 않고 기존 위치에 남음

원인:

- 표 삭제 명령은 core에서 `recompose_section()`과 `paginate_if_needed()`는 호출했지만, 삭제된 표가 있던 문단의
  `line_segs`를 다시 계산하고 후속 문단 `vertical_pos`를 재계산하는 단계가 없었다.
- 따라서 표가 차지하던 문단 높이가 `line_segs`/vpos에 남아 아래 문단이 stale 위치를 유지할 수 있었다.

변경:

- `src/document_core/commands/table_ops.rs`
  - `delete_table_control_native()`에서 표 컨트롤 제거 후 해당 문단을 `reflow_paragraph()`로 재계산
  - 삭제 문단부터 `recalculate_section_vpos()` 적용
  - 이후 기존처럼 `recompose_section()`과 `paginate_if_needed()` 실행
- `src/wasm_api/tests.rs`
  - `test_delete_table_control()`에 표 삭제 후 다음 문단 vpos가 위로 당겨지는 회귀 검증 추가

## 검증

자동 검증:

- `npm run build` in `rhwp-studio/` — 통과
- `cargo test --lib` — 통과
  - 1602 passed
  - 0 failed
  - 6 ignored
- `git diff --check` — 통과

## 작업지시자 판정 필요 항목

다음 UX 동작 판정을 요청했다.

1. 기존 텍스트 입력 Undo/Redo 회귀 없음
2. 글자 서식 Undo/Redo 회귀 없음
3. 문단 서식 Undo/Redo 회귀 없음
4. 그림/도형 drag 이동 후 Undo/Redo 회귀 없음
5. 그림/도형 resize Undo/Redo 회귀 없음
6. `Ctrl+Enter` 쪽 나누기 후 Undo/Redo 동작
7. `Ctrl+Shift+Enter` 단 나누기 후 Undo/Redo 동작
8. 문단 사이 표 삭제 후 아래 문단 레이아웃 흐름 갱신

## 작업지시자 판정 결과

- Undo/Redo 동작 테스트 통과
- 표 삭제 후 아래 문단 레이아웃 흐름 갱신 동작 테스트 통과

## 남은 후보

이번 단계에서 아직 처리하지 않은 직접 mutation 후보:

- 수식 삽입
- 각주/미주 삽입
- 머리말/꼬리말 생성/삭제/필드 삽입
- 필드 속성 변경
- form value 변경
- 표 구조 변경 일부
- object z-order/group/ungroup

이들은 #1320의 후속 Stage 또는 별도 세부 이슈에서 라우팅/transaction 전환한다.
