# Task #1320 완료 보고서

## 요약

편집 액션 라우터와 Undo/Redo 트랜잭션 아키텍처의 1차 기반을 정비했다.

이번 작업의 핵심은 rhwp-studio의 편집 mutation이 개별 호출부에서 흩어지지 않고,
`executeOperation()` 라우터를 중심으로 history, refresh, dirty scope, selection 정책을 점진적으로
표준화할 수 있게 만드는 것이다.

작업지시자 동작 테스트 중 문단과 문단 사이의 표를 삭제할 때 아래 문단의 레이아웃 흐름이 갱신되지 않는
문제가 추가로 확인되었다. 이는 core 표 삭제 후 `line_segs`/vpos 재계산이 빠진 문제였으므로 이번 범위에
포함해 보강했다.

## 변경 내용

### 편집 라우터 metadata

- `rhwp-studio/src/engine/command.ts`
  - `EditDomain`
  - `RefreshPolicy`
  - `DirtyScope`
  - `SelectionPolicy`
  - `OperationMetadata`
  - `OperationDescriptor.meta`

`meta`는 optional로 두어 기존 호출부 호환을 유지했다.

### `executeOperation()` 계약 보강

- `rhwp-studio/src/engine/input-handler.ts`
  - `record` 경로에서 `history.recordWithoutExecute(desc.command, this.wasm)`를 호출하도록 정리
  - `refreshAfterOperation()` 추가
  - `command`, `snapshot`, `record` 경로가 metadata 기반 refresh 정책을 사용할 수 있게 보강

기본 refresh 정책은 기존 동작과 맞췄다.

- `command`: `auto`
- `snapshot`: `full`
- `record`: `none`

### 직접 history 기록 경로 정리

- `rhwp-studio/src/engine/input-handler-picture.ts`
  - 그림/도형 drag 이동 종료 시 직접 `history.recordWithoutExecute()`를 호출하던 경로를
    `executeOperation({ kind: 'record', ... })`로 라우팅

### 쪽/단 나누기 snapshot 라우팅

- `rhwp-studio/src/command/commands/page.ts`
  - `page:break`
  - `page:column-break`

직접 `services.wasm.*` mutation 후 event emit하던 구조를 `executeOperation({ kind: 'snapshot', ... })`
경로로 전환했다.

### 표 삭제 후 레이아웃 흐름 재계산

- `src/document_core/commands/table_ops.rs`
  - `delete_table_control_native()`에서 표 컨트롤 제거 후 해당 문단을 `reflow_paragraph()`로 재계산
  - 삭제 문단부터 `recalculate_section_vpos()` 적용
  - 이후 기존처럼 `recompose_section()`과 `paginate_if_needed()` 실행

- `src/wasm_api/tests.rs`
  - `test_delete_table_control()`에 표 삭제 후 다음 문단 vpos가 위로 당겨지는 회귀 검증 추가

### 문서화

- `mydocs/tech/edit_action_undo_redo_architecture.md`
  - 편집 액션 라우터/Undo/Redo 트랜잭션 아키텍처 계약 정리

- `mydocs/manual/edit_command_review_checklist.md`
  - 새 편집 command 추가 시 검토할 mutation/history/refresh/dirty/selection 체크리스트 추가

- `mydocs/working/task_m100_1320_stage1.md`
  - 기존 직접 mutation/Undo 경로 인벤토리

- `mydocs/working/task_m100_1320_stage2.md`
  - Stage 3 구현 및 추가 보강 보고

## 설계 판단

이번 #1320은 모든 직접 mutation을 한 번에 제거하지 않았다.

대신 다음 원칙으로 1차 기반을 만들었다.

- 기존 `EditCommand` 인터페이스를 깨지 않는다.
- snapshot은 제거 대상이 아니라 복합 편집 fallback으로 유지한다.
- `record` 경로는 이미 적용된 편집을 history에 기록하는 `recordApplied` 성격으로 문서화한다.
- refresh/dirty/selection 정책은 optional metadata로 확장 가능하게 둔다.
- 이후 표, 필드, 머리말/꼬리말, object z-order 등은 같은 라우터 계약에 맞춰 단계적으로 이전한다.

## 지원 범위

성공 판정 범위:

- 기존 텍스트 입력 Undo/Redo 회귀 없음
- 글자 서식 Undo/Redo 회귀 없음
- 문단 서식 Undo/Redo 회귀 없음
- 그림/도형 drag 이동 기록 라우팅
- 쪽 나누기 Undo/Redo snapshot 라우팅
- 단 나누기 Undo/Redo snapshot 라우팅
- 문단 사이 표 삭제 후 아래 문단 레이아웃 흐름 갱신

후속 확장 범위:

- 수식 삽입 transaction화
- 각주/미주 삽입 transaction화
- 머리말/꼬리말 생성/삭제/필드 삽입 transaction화
- 필드 속성 변경 transaction화
- form value 변경 transaction화
- 표 구조 변경 전체 라우팅
- object z-order/group/ungroup 라우팅

## 검증

| 항목 | 결과 |
|---|---|
| `cargo test test_delete_table_control --lib -- --nocapture` | 통과 |
| `cargo test --lib` | 통과 |
| `npm run build` (`rhwp-studio/`) | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 |
| `git diff --check` | 통과 |
| rhwp-studio 동작 테스트 | 통과 |

`cargo test --lib` 결과:

```text
test result: ok. 1602 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 131.51s
```

WASM 빌드 결과:

```text
[INFO]: :-) Done in 3m 00s
[INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.
```

## 판정

작업지시자가 rhwp-studio에서 Undo/Redo 동작 테스트와 표 삭제 후 레이아웃 흐름 갱신 동작 테스트 성공을
확인했다.

이번 이슈는 성공으로 완료 판정한다.
