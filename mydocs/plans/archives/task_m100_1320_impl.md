# Task M100-1320 구현 계획서 — 편집 라우터/Undo 계약 1차 정비

## Stage 1 승인 반영

작업지시자는 Stage 1 인벤토리와 향후 확장성 보완점을 수용했다.

수용된 핵심 방향:

- 현재 구조는 기능 추가는 가능하지만, 기능이 늘어날수록 Undo/Redo 사용자 경험이 갈라질 수 있다.
- `CommandServices`에서 직접 `wasm.*` mutation을 호출하는 구조는 장기적으로 라우터 우회 위험이 있다.
- `OperationDescriptor`에는 history/refresh/dirty/selection 메타데이터가 필요하다.
- `record`는 `recordApplied` 성격으로 계약을 명확히 해야 한다.
- snapshot은 제거 대상이 아니라 허용 기준을 둔 복합 편집 fallback이다.

## 참조 문서

- Stage 1 보고서: `mydocs/working/task_m100_1320_stage1.md`
- 아키텍처 계약: `mydocs/tech/edit_action_undo_redo_architecture.md`

## 구현 원칙

이번 #1320에서 모든 직접 mutation을 해결하지 않는다.

1차 구현은 다음을 목표로 한다.

- 기존 Undo/Redo 동작을 깨지 않는다.
- 기존 `EditCommand` 인터페이스를 즉시 깨지 않는다.
- `OperationDescriptor`는 후방 호환을 유지하며 metadata를 확장한다.
- `record` 경로를 더 명확하게 만들고, 직접 `history.recordWithoutExecute()` 호출을 줄인다.
- 작은 코드 변경으로 이후 page/field/object/table command 이전의 기반을 만든다.

## 단계별 구현

### Stage 3-A — 타입과 계약 보강

대상:

- `rhwp-studio/src/engine/command.ts`

작업:

- `EditDomain` 타입 추가
  - `text`
  - `charFormat`
  - `paraFormat`
  - `table`
  - `object`
  - `page`
  - `field`
  - `view`
  - `unknown`
- `RefreshPolicy` 타입 추가
  - `auto`
  - `full`
  - `pageLocal`
  - `selectionOnly`
  - `none`
- `DirtyScope` 타입 추가
  - `document`
  - `section`
  - `page`
  - `paragraph`
  - `table`
  - `object`
  - `none`
- `OperationMetadata` 타입 추가
  - `actionId?`
  - `domain?`
  - `refresh?`
  - `dirtyScope?`
  - `selection?`
- `OperationDescriptor`에 optional `meta?: OperationMetadata` 추가

주의:

- 기존 호출부를 모두 수정하지 않아도 컴파일되어야 한다.
- `record` 이름은 즉시 변경하지 않고 문서상 `recordApplied`로 해석한다. 코드상 rename은 범위가 넓어 후속
  이슈로 넘길 수 있다.

### Stage 3-B — `executeOperation()` refresh/record 정책 보강

대상:

- `rhwp-studio/src/engine/input-handler.ts`

작업:

- `record` 경로에서 `history.recordWithoutExecute(desc.command, this.wasm)`로 redo stack snapshot resource
  discard를 보장한다.
- `record` 경로도 metadata에 따라 refresh를 처리할 수 있게 한다.
- 기본값은 현재 동작과 최대한 맞춘다.
  - `record` 기본 refresh는 `none`
  - 호출부가 이미 `document-changed` emit을 하는 경로는 기존처럼 유지
  - 새로 이전하는 경로만 `refresh: 'full'` 또는 `pageLocal`을 명시
- `command`와 `snapshot` 경로도 metadata가 있으면 refresh policy를 우선 적용할 수 있게 한다.

판정 기준:

- 기존 텍스트/문단/글자 서식 Undo/Redo가 회귀하지 않는다.
- 기존 drag 경로의 refresh 타이밍이 변하지 않는다.

### Stage 3-C — 직접 `recordWithoutExecute()` 1차 제거

대상:

- `rhwp-studio/src/engine/input-handler-picture.ts`

작업:

- `finishPictureMoveDrag()`에서 직접 `this.history.recordWithoutExecute()`를 호출하는 부분을
  `this.executeOperation({ kind: 'record', ... })`로 통합한다.
- metadata 예:
  - `domain: 'object'`
  - `refresh: 'none'`
  - `dirtyScope: 'object'`
- 기존 drag 중 이미 `document-changed`와 selection refresh가 처리되는 구조는 유지한다.

이유:

- 기능 변화가 작다.
- 이미 command payload가 존재한다.
- direct history access를 줄이는 첫 사례로 적합하다.

### Stage 3-D — 작은 직접 mutation 후보 1개 선정

후보:

1. `insert:equation`
2. `page:break`
3. `page:column-break`
4. field props 변경

권장:

- 이번 #1320에서는 `page:break` 또는 `page:column-break`를 snapshot 경로로 감싸는 방안을 우선 검토한다.

판단 이유:

- 사용자 Undo 기대치가 높다.
- page command 파일의 직접 mutation 문제를 대표한다.
- 정확한 delta command를 만들기 전에 snapshot fallback 기준을 검증할 수 있다.

주의:

- 쪽/단 나누기는 문서 구조와 페이지네이션 영향이 크다. snapshot이 안전하지만 비용이 있을 수 있다.
- 구현 전 간단한 수동 검증 샘플을 정한다.

### Stage 3-E — 감사 체크리스트 추가

대상 후보:

- `mydocs/manual/rhwp_cli_skill_guide.md` 또는 별도 `mydocs/manual/edit_command_review_checklist.md`

작업:

- 새 document mutation 추가 시 확인할 체크리스트를 문서화한다.
- 당장 CI 자동화는 하지 않는다.

## 이번 이슈에서 하지 않을 일

- 모든 `commands/page.ts` 직접 mutation 이전
- 모든 `commands/insert.ts` 직접 mutation 이전
- 표 구조 편집 전체 transaction화
- `EditCommand` 반환 타입 전면 교체
- `record`를 코드 전체에서 `recordApplied`로 rename
- snapshot 제거
- core snapshot을 section/page 단위로 세분화

## 검증 계획

필수:

- `npm run build` in `rhwp-studio/`
- `cargo test --lib`
- `git diff --check`

수동 UX 판정:

- 텍스트 입력 Undo/Redo
- 글자 서식 Undo/Redo
- 문단 서식 Undo/Redo
- 이미지/도형 이동 Undo/Redo
- 이미지/도형 resize Undo/Redo
- 선택한 작은 직접 mutation 후보의 Undo/Redo

필요 시:

- `docker compose --env-file .env.docker run --rm wasm`

## 작업지시자 판정 요청 지점

1. Stage 3-A/B/C 적용 후, 기존 Undo/Redo 회귀 여부 판정
2. Stage 3-D 후보 중 어느 직접 mutation을 이번 이슈에서 다룰지 판정
3. 최종 동작 테스트 판정
