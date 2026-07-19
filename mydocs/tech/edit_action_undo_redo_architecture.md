---
kind: canonical
status: active
canonical: mydocs/tech/edit_action_undo_redo_architecture.md
last_verified: 2026-07-16
---

# 편집 액션 라우터와 Undo/Redo 아키텍처 계약

본 문서는 Task #1320에서 정리한 rhwp-studio 편집 액션 라우팅과 Undo/Redo 복원 계약이다.

## 목적

rhwp-studio의 편집 기능이 늘어나도 다음 사용자 경험을 유지한다.

- 메뉴, 툴바, 단축키, 마우스 조작이 같은 Undo/Redo 정책을 따른다.
- 문서를 바꾸는 액션은 history, dirty, refresh 정책을 명시한다.
- 문서를 바꾸지 않는 보기/조회 액션은 history에 들어가지 않는다.
- 복잡한 편집은 snapshot을 허용하되, 허용 조건을 명확히 한다.

## 기본 원칙

### 1. 문서 mutation은 라우터를 통과한다

새로운 문서 mutation은 원칙적으로 편집 라우터를 통과해야 한다.

허용되는 예외:

- query/read API
- export/render API
- view state API
- `EditCommand` 내부의 저수준 `wasm.*` 호출
- 성능상 drag 중 실시간 preview를 위해 먼저 적용하고 종료 시 `recordApplied`로 기록하는 경로

### 2. history stack은 하나다

Undo/Redo stack은 하나로 유지한다. 도메인별 별도 stack은 만들지 않는다.

도메인별 차이는 command payload와 restore 방식으로 처리한다.

### 3. redo는 command의 `execute()` 재호출이다

`CommandHistory.redo()`는 redo 대상 command의 `execute()`를 다시 호출한다.

따라서 `EditCommand.execute()`는 다음 성질을 가져야 한다.

- 같은 command payload로 재실행 가능해야 한다.
- 최초 실행과 redo 실행이 같은 결과를 만들어야 한다.
- 필요한 before/after 식별자는 command가 보관해야 한다.

### 4. Undo payload는 UI 표시 JSON이 아니라 복원 계약이다

Undo/Redo payload는 가능한 한 core가 안정적으로 복원할 수 있는 값이어야 한다.

권장 순서:

1. 도메인 ID 또는 shape ID
2. before/after property delta
3. 구조 변경 command payload
4. snapshot

## Operation 유형

### `command`

명령이 mutation 실행과 undo를 모두 소유한다.

사용 대상:

- 텍스트 입력/삭제
- 문단 분할/병합
- 글자/문단 서식
- 작은 속성 변경
- z-order처럼 before/after가 명확한 조작

계약:

- `execute(wasm)`는 문서를 변경하고 결과 위치를 반환한다.
- `undo(wasm)`는 이전 상태를 복원한다.
- redo는 `execute(wasm)` 재호출로 동작한다.
- 필요한 before/after payload는 command 내부에 저장한다.

### `snapshot`

라우터가 before/after snapshot으로 복원한다.

사용 대상:

- 붙여넣기
- 복합 object/table 삭제
- 여러 리소스가 동시에 바뀌는 구조 편집
- 아직 안전한 delta command가 없는 복합 편집

허용 조건:

- 정확한 도메인 delta 설계가 현재 과도하게 복잡하다.
- 실패 시 문서 복구가 어렵다.
- 사용자 액션 하나로 여러 control/resource가 바뀐다.

제약:

- operation type을 명시한다.
- snapshot resource는 stack eviction 또는 clear 시 discard되어야 한다.
- 가능한 한 장기적으로 domain command로 이전할 후보를 표시한다.

### `recordApplied`

문서 mutation이 이미 적용된 뒤 history에만 기록한다.

현재 코드의 `record`와 `history.recordWithoutExecute()`는 이 개념에 해당한다.

사용 대상:

- IME composition 완료
- drag move/resize 종료
- 실시간 preview가 필요한 interactive 조작

계약:

- 호출 시점에는 문서가 이미 변경되어 있어야 한다.
- command는 undo와 redo에 필요한 before/after payload를 모두 가져야 한다.
- refresh policy와 dirty scope를 함께 지정해야 한다.
- 일반 호출부는 직접 `history.recordWithoutExecute()`를 호출하지 않고 router를 통과한다.

### `none`

문서 mutation이 아닌 액션이다.

사용 대상:

- search/query
- export/render
- 보기 옵션
- cursor/hit-test/selection rect 조회

계약:

- history에 기록하지 않는다.
- document dirty 상태를 변경하지 않는다.

## 메타데이터

편집 라우터는 장기적으로 다음 메타데이터를 가져야 한다.

| 필드 | 목적 |
|---|---|
| `actionId` | 메뉴/툴바/단축키 액션 추적 |
| `domain` | text, charFormat, paraFormat, table, object, page, field 등 |
| `historyPolicy` | command, snapshot, recordApplied, none |
| `refreshPolicy` | full, pageLocal, selectionOnly, none |
| `dirtyScope` | document, section, page, paragraph, table, object |
| `selectionPolicy` | keep, moveToResult, restoreObjectSelection, none |
| `mergeKey` | 연속 입력/drag 병합 기준 |

초기 구현에서는 모든 필드를 강제하지 않는다. 다만 문서 mutation을 추가할 때 history/refresh 정책은 명시하는
방향으로 점진 이전한다.

## CommandResult

현재 `EditCommand.execute()`와 `undo()`는 `DocumentPosition`만 반환한다. 확장성을 위해 장기적으로는 다음
형태가 필요하다.

```ts
interface CommandResult {
  cursor?: DocumentPosition;
  selection?: unknown;
  dirtyScope?: DirtyScope;
  refreshPolicy?: RefreshPolicy;
  mutated: boolean;
}
```

단기 구현에서는 기존 `DocumentPosition` 반환을 유지하고, router가 이를 `CommandResult`로 감싸는 호환
레이어를 둔다.

## `TextMutationEffects` 계약

Task #2214부터 셀 텍스트 편집은 mutation 결과와 커서 정합성을 연결하기 위해 다음 effect를 전달한다.
이는 단순 렌더링 힌트가 아니라, pagination을 미룬 상태에서 stale page tree로 커서를 조회하지 않기 위한
실행 계약이다. WASM의 `paginationDeferred`는 Studio의 `deferredPagination`으로 매핑한다.

| 필드 | 의미와 처리 계약 |
|---|---|
| `deferredPagination` | 현재 mutation이 전체 pagination을 미뤘다. `true`이면 deferred pending을 등록한다. 이 값만으로 동기 full flush를 요구하지는 않는다. |
| `cellFlowChanged` | deferred mutation 전후 셀 문단의 상대 line advance가 달라졌다. `deferredPagination`이 `true`일 때만 의미가 있으며, `true`이면 exact cursor lookup 전에 경계 flush를 한 번 수행한다. `false`이면 동기 full flush 없이 page-local 갱신을 허용한다. |
| `paginationCompleted` | 현재 mutation이 immediate 경로에서 pagination까지 완료했다. 기존 deferred pending과 예약 timer를 제거한다. |

IME/iOS처럼 여러 raw mutation을 묶으면 세 값은 각각 OR 누적되므로 `paginationCompleted`와
`deferredPagination`이 함께 `true`일 수 있다. 소비자는 이 경우 **기존 pending 제거 → 새 deferred pending
등록 → flow 경계 flush 판정** 순서로 처리해야 한다.

### 캡처와 one-shot 소비

- command의 `execute()`는 실행 전에 이전 effect를 비우고, 방금 수행한 mutation 결과만 저장한다.
- `consumeTextMutationEffects()`와 history의 `consumeLastExecutionEffects()`는 effect를 한 번 반환한 즉시
  `NO_TEXT_MUTATION_EFFECTS`로 되돌린다.
- history는 command 실행 직후 effect를 캡처한다. command가 이전 history entry와 병합되더라도 직전
  effect를 병합된 entry에 승계하거나 다음 입력으로 누수하지 않는다.
- redo는 저장된 effect를 재사용하지 않고 `execute()`의 실제 결과를 다시 캡처한다. 같은 payload라도
  현재 셀 흐름에 따라 `cellFlowChanged`가 달라질 수 있기 때문이다.
- undo는 immediate pagination 복원 경로를 사용하고 실행·redo에서 남은 effect를 먼저 제거한다. 입력
  라우터는 이를 `paginationCompleted`로 처리해 기존 deferred pending을 정리한다.
- 이미 적용된 raw mutation을 `recordApplied`로 history에 기록할 때는 history effect를 새로 만들지 않는다.
  raw 경로에서 캡처한 effect를 커서 이동 전에 별도로 소비한다.

### IME/iOS raw 입력과 수명주기

- 한 번의 IME/iOS 치환 과정에 포함된 삭제·삽입 effect는 `TextMutationEffectAccumulator`에 OR 누적하고
  한 번만 소비한다.
- raw effect는 각 치환 사이클과 composition 시작·종료에서 초기화한다.
- `deactivate()`와 `dispose()`는 예약 flush, deferred pending, raw effect를 함께 제거한다. history
  `clear()`도 소비되지 않은 실행 effect를 제거한다. 새 문서나 새 편집 세션으로 effect를 넘기지 않는다.

### 커서 조회 전 처리 순서

텍스트 mutation 이후에는 다음 순서를 지킨다.

1. `paginationCompleted`이면 기존 pending과 timer를 제거한다.
2. `deferredPagination`이면 새 pending을 먼저 등록한다.
3. `cellFlowChanged`이면 같은 호출에서 경계 flush를 한 번 수행한다. 실패 시 pending은 보존하되 같은
   호출에서 반복 시도하지 않는다.
4. 그 뒤에만 `cursor.moveTo()` 등 exact cursor lookup을 수행한다.
5. 경계 flush를 이미 수행했다면 후속 refresh가 full pagination을 중복 실행하지 않게 전달한다. 안정
   입력(`cellFlowChanged=false`)은 동기 full flush 없이 기존 idle/manual/full-edit 마감 정책을 따른다.

### WASM 호환성

- deferred API가 없는 구버전 WASM은 immediate 삽입 API로 폴백하고
  `paginationCompleted=true`, `deferredPagination=false`로 취급한다.
- deferred API는 있지만 `cellFlowChanged` 필드가 없거나 boolean `false`가 아닌 구버전·비정상
  신호는 stale cursor를 방지하기 위해 보수적으로 `true`로 취급한다. 명시적인 `false`만 안정
  입력으로 인정한다.
- 기본 응답의 `ok`/`charOffset` shape가 잘못됐거나 JSON을 해석할 수 없으면 mutation 결과 오류로
  처리한다. 반면 command에 effect 소비 API가 없으면 `NO_TEXT_MUTATION_EFFECTS`로 취급한다.

## 도메인별 payload 기준

| 도메인 | 권장 payload |
|---|---|
| text | 위치, 삽입/삭제 문자열 |
| char format | range, before/after charShapeId 또는 property delta |
| para format | target paragraph, before/after paraShapeId |
| table move/resize | table ref, before/after offset 또는 cell size |
| object transform | object ref, before/after bbox/offset/size/rotation |
| page control | target section/paragraph, before/after control snapshot |
| field/form | field id/path, before/after props 또는 value |

## Transaction

하나의 사용자 액션이 여러 mutation으로 구성되면 transaction으로 묶는다.

예:

- 번호/글머리표 리소스 생성 + 문단 서식 적용
- 머리말/꼬리말 생성 + 필드 삽입
- 표 행/열 삽입 + selection 복원
- 이미지 삽입 + 셀 높이 조정

초기 구현에서는 `TransactionCommand`를 바로 강제하지 않는다. 먼저 snapshot과 command의 허용 기준을
문서화하고, transaction이 필요한 액션을 후보로 표시한다.

## 감사 규칙

PR 검토 시 다음 질문을 확인한다.

- 새 `wasm.*` 호출이 문서를 변경하는가?
- 변경한다면 router 또는 `EditCommand` 내부인가?
- Undo/Redo 기대치가 있는 사용자 액션인가?
- refresh와 dirty 상태가 일관되게 처리되는가?
- snapshot을 사용한다면 허용 조건에 맞는가?

향후 `rhwp-studio/src/command/commands`와 `rhwp-studio/src/engine/input-handler*.ts`의 mutation성
`wasm.*` 호출을 탐지하는 정적 점검을 추가한다.
