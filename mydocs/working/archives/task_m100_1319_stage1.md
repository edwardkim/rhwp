# Task #1319 Stage 1 — 문단 서식 Undo/Redo 진단

## 범위

- 이슈: #1319 `문단 서식 변경 Undo/Redo 커맨드 체계화`
- 브랜치: `local/task1319`
- 수행계획서: `mydocs/plans/task_m100_1319.md`
- 단계 목표: 소스 수정 없이 현재 문단 서식 변경 경로와 Undo/Redo 구현 전략을 확정한다.

## 현재 히스토리 구조

`rhwp-studio/src/engine/history.ts`의 `CommandHistory`는 `EditCommand` 기반이다.

- `execute(command, wasm)`:
  - `command.execute(wasm)` 실행
  - undo stack에 push
  - redo stack clear
- `undo(wasm)`:
  - undo stack pop
  - `command.undo(wasm)` 실행
  - redo stack push
- `redo(wasm)`:
  - redo stack pop
  - `command.execute(wasm)` 재실행
  - undo stack push

따라서 문단 서식 Undo/Redo도 `EditCommand`로 편입하는 것이 기존 구조와 맞다.

## 기존 Undo/Redo 충돌 조사

기존 history 동작과 충돌 가능성이 있는 지점을 추가 점검했다.

### `redo()`는 `execute()`를 다시 호출한다

`CommandHistory.redo()`는 redo stack의 command를 꺼낸 뒤 `command.execute(wasm)`를 다시 호출한다.

따라서 `ApplyParaFormatCommand.execute()`가 항상 props를 재적용하면 redo 때마다
`find_or_create_para_shape()`가 다시 동작할 수 있다. 이는 중복 ParaShape 생성, definition 증가, 적용 후 ID
불안정으로 이어질 수 있다.

대응:

- 최초 execute: before ID 저장 → props 적용 → after ID 저장
- redo execute: `afterParaShapeId`가 이미 있으면 props 재적용 없이 `setParaShapeId*`로 after ID 복원

이 구조는 `SnapshotCommand`의 최초 실행/redo 분기와 같은 패턴이지만, 문서 전체 스냅샷은 쓰지 않는다.

### `mergeWith()` 병합 금지

`CommandHistory.execute()`는 모든 command에 대해 직전 command와 `mergeWith()`를 시도한다.

문단 모양 대화상자 적용, 정렬 변경, 줄 간격 변경, Shift+Tab 내어쓰기는 사용자가 명시적으로 수행한 단일
작업이므로 자동 병합하면 Ctrl+Z 단계 기대와 어긋날 수 있다.

대응:

- `ApplyParaFormatCommand.mergeWith()`는 항상 `null` 반환

### `recordWithoutExecute()` 사용 금지

`recordWithoutExecute()`는 IME composition, 드래그 중 이미 문서에 반영된 변경처럼 "이미 적용된 변경"을
history에 사후 기록할 때 쓰인다.

문단 서식 적용은 아직 대부분 UI handler에서 직접 WASM 호출을 하고 있으므로, 이 경로를 유지한 채
`recordWithoutExecute()`만 추가하면 적용/기록 순서가 분산된다.

대응:

- 문단 서식은 `executeOperation({ kind: 'command', command })`로만 실행
- 기존 직접 `wasm.applyParaFormat*` 호출은 command 생성 경로로 모은다.

### page-local refresh와 충돌 없음

`shouldUsePageLocalRefresh()`는 현재 셀 내부 단일 `insertText/deleteText`만 page-local refresh 대상으로 본다.

문단 서식 변경은 줄 재계산과 페이지 흐름에 영향을 줄 수 있으므로 full `afterEdit()` 경로를 타는 것이 맞다.
따라서 `applyParaFormat` 계열을 page-local command로 추가하지 않는다.

### 선택 영역 보존

`executeOperation()`은 현재 `applyCharFormat`만 선택 영역을 유지하고, 그 외 command는 `cursor.moveTo(newPos)`를
호출한다.

문단 모양 대화상자에서 선택 범위에 문단 속성을 적용한 직후 selection을 유지하는 것이 UX상 자연스러울 수
있다. 새 command가 selection을 반드시 보존해야 하는지는 구현 단계에서 확인이 필요하지만, 최소한 기존
글자 서식처럼 `executeOperation()`의 특례 대상에 `applyParaFormat`을 추가할지 검토해야 한다.

## 글자 서식 선례

`ApplyCharFormatCommand`는 이미 존재한다.

- 적용 전 `charShapeId`를 저장
- execute에서 `applyCharFormat*` 실행
- undo에서 이전 `charShapeId`로 복원

문단 서식도 이와 비슷하게 처리할 수 있다. 다만 문단 서식은 현재 조회/적용 단위 계약이 글자 서식보다
복잡하다.

## 현재 문단 서식 변경 경로

확인된 직접 적용 경로:

- `InputHandler.applyParaFormat()`
  - 정렬
  - 줄 간격
  - 문단 머리/글머리표/번호 적용 일부
- `InputHandler.applyParaPropsToRange()`
  - 문단 모양 대화상자
  - 선택 범위 문단 속성 적용
- `InputHandler.applyHangingIndentAtCursor()`
  - #1318 Shift+Tab 커서 기준 내어쓰기
- `format:para-num-shape` 일부
  - `services.wasm.applyParaFormat(...)` 직접 호출

이 경로들은 WASM `applyParaFormat*`을 직접 호출하고 `afterEdit()` 또는 `document-changed`만 수행하므로
history stack에 문단 속성 변경이 기록되지 않는다.

## 단위 계약 문제

문단 속성 조회 타입:

```ts
interface ParaProperties {
  alignment?: string;
  lineSpacing?: number;
  lineSpacingType?: string;
  marginLeft?: number;     // px
  marginRight?: number;    // px
  indent?: number;         // px
  spacingBefore?: number;  // px
  spacingAfter?: number;   // px
  paraShapeId?: number;
}
```

Rust `build_para_properties_json()`은 UI 표시를 위해 margin/indent/spacing 계열을 px로 변환해 반환한다.

반면 `parse_para_shape_mods()`는 `marginLeft`, `indent`, `spacingBefore`, `spacingAfter`를 raw 값으로
해석한다.

따라서 다음 방식은 위험하다.

```text
prevProps = getParaPropertiesAt(...)
undo: applyParaFormat(JSON.stringify(prevProps))
```

이 방식은 px 값을 raw 값으로 다시 적용하게 되어 여백/내어쓰기/문단 위아래 간격이 틀어질 수 있다.

## 안정적인 Undo/Redo 저장 단위

가장 안전한 저장 단위는 `paraShapeId`다.

이유:

1. 현재 문단 서식 변경은 내부적으로 `find_or_create_para_shape(base_id, mods)`를 통해 새 ParaShape ID를
   만들거나 기존 동일 ParaShape ID를 재사용한다.
2. 실제 문단에는 `para_shape_id`만 바뀐다.
3. 이전 `para_shape_id`와 적용 후 `para_shape_id`를 저장하면 단위 변환 없이 정확히 복원할 수 있다.
4. 문단 속성 테이블에 unused ParaShape가 남을 수 있지만, 이는 현재 `find_or_create_para_shape` 방식에서도
   이미 허용되는 구조다.

권장 저장 모델:

```ts
interface ParaFormatHistoryEntry {
  target: ParaFormatTarget;
  beforeParaShapeId: number;
  afterParaShapeId?: number;
}
```

execute 최초 실행:

1. 대상 문단의 현재 `paraShapeId` 조회
2. 기존 `applyParaFormat*`으로 props 적용
3. 적용 후 `paraShapeId` 재조회
4. `beforeParaShapeId`, `afterParaShapeId` 저장

undo:

1. 대상 문단의 `paraShapeId`를 `beforeParaShapeId`로 직접 복원
2. reflow/rebuild/event 처리

redo:

1. `afterParaShapeId`가 있으면 대상 문단의 `paraShapeId`를 직접 복원
2. 없으면 최초 execute와 동일하게 props 적용

## 필요한 WASM 보강

현재 TypeScript에서 문단 `paraShapeId`를 직접 설정하는 공개 WASM API는 확인되지 않았다.

따라서 정밀 커맨드를 안정적으로 구현하려면 다음 계열의 API가 필요하다.

```text
setParaShapeId(sec, para, paraShapeId)
setCellParaShapeId(sec, parentPara, controlIdx, cellIdx, cellParaIdx, paraShapeId)
```

또는 범용적으로:

```text
restoreParaShapeId(targetJson, paraShapeId)
```

복원 API에서 해야 할 일:

- 대상 문단 `para_shape_id` 교체
- body 문단은 `raw_stream = None`
- 셀 문단은 부모 표 dirty 마킹
- line spacing 관련 ParaShape가 바뀔 수 있으므로 보수적으로 lineSeg/rebuild 처리
- `rebuild_section(sec)`
- `DocumentEvent::ParaFormatChanged` 기록

## 1차 지원 문맥 판단

### 본문 문단

지원 가능하다.

- 조회: `getParaPropertiesAt(sec, para)` → `paraShapeId`
- 적용: `applyParaFormat(sec, para, propsJson)`
- 복원: 새 API `setParaShapeId(sec, para, id)` 필요

### 일반 표 셀 문단

지원 가능하다.

- 조회: `getCellParaPropertiesAt(sec, parentPara, controlIdx, cellIdx, cellParaIdx)` → `paraShapeId`
- 적용: `applyParaFormatInCell(...)`
- 복원: 새 API `setCellParaShapeId(...)` 필요

### 선택 범위 다중 문단

지원 가능하다.

- 본문: start paragraph부터 end paragraph까지 entries 생성
- 셀: 같은 셀 내부 cellParaIndex 범위 우선 지원
- 서로 다른 셀/중첩 경로 선택은 1차 제외 권장

### 머리말/꼬리말, 각주/미주

조회/적용 API는 존재한다.

- `getParaPropertiesInHf`, `applyParaFormatInHf`
- `getParaPropertiesInFootnote`, `applyParaFormatInFootnote`

하지만 커서 문맥과 대상 식별자가 일반 문단/셀과 다르고, 최근 각주/미주 커서 회귀 이력이 있어 1차에서는
제외하는 것이 안전하다.

### 글상자/중첩 표

현재 #1318에서도 no-op 처리한 영역이다. 1차 범위에서 제외한다.

## 호출 경로 전환 제안

1차 구현에서 전환할 경로:

- `applyParaPropsToRange()`
- `applyParaFormat()`
- `applyHangingIndentAtCursor()`

직접 WASM 호출 대신 다음 형태로 바꾼다.

```ts
const cmd = new ApplyParaFormatCommand(targets, props, cursorBefore);
this.executeOperation({ kind: 'command', command: cmd });
```

예외:

- numbering dialog에서 직접 `services.wasm.applyParaFormat(...)` 호출하는 경로는 `InputHandler` helper를 통해
  우회하지 않도록 정리해야 한다.
- 문단 번호 정의 자체 생성/변경은 1차 Undo 대상에서 제외할 수 있다. 다만 문단에 적용된 `paraShapeId` 복원은 가능하다.

## 중복 코드 조사

현재 중복 또는 우회 위험이 있는 경로:

- `InputHandler.applyParaFormat()`
  - toolbar/menu 정렬, 줄 간격, 문단 머리 계열이 사용
  - 직접 `wasm.applyParaFormat*` 호출 후 `afterEdit()`
- `InputHandler.applyParaPropsToRange()`
  - 문단 모양 대화상자 적용 경로
  - 직접 `wasm.applyParaFormat*` 호출 후 `afterEdit()`
- `InputHandler.applyHangingIndentAtCursor()`
  - #1318 Shift+Tab 경로
  - 직접 `wasm.applyParaFormat*` 호출 후 `afterEdit()`
- `rhwp-studio/src/command/commands/format.ts`의 `format:para-num-shape`
  - 일부 분기에서 `services.wasm.applyParaFormat(...)` 직접 호출

새 command를 추가해도 이 직접 호출 경로가 남으면 일부 문단 작업은 계속 Undo/Redo를 우회한다.

중복을 줄이는 방향:

1. 문단 서식 target 산정은 `InputHandler` helper에 둔다.
2. `ApplyParaFormatCommand`는 이미 산정된 target, props, before/after ID만 처리한다.
3. 문단 번호 모양 대화상자도 `InputHandler` helper를 통해 command 경로로 진입시킨다.
4. `command.ts` 내부에는 cursor/selection 해석을 새로 복제하지 않는다.

## 구현 계획서 권장안

Stage 2 구현 계획서는 다음 방향으로 작성하는 것을 권장한다.

1. Rust/WASM에 body/cell paraShapeId restore API 추가
2. TypeScript `ApplyParaFormatCommand` 추가
3. command는 `paraShapeId` before/after를 저장
4. `applyParaPropsToRange()`, `applyParaFormat()`, `applyHangingIndentAtCursor()`를 command 경로로 전환
5. 머리말/각주/미주/글상자/중첩 표는 명시 no-op 또는 후속 이슈 처리
6. 수동 검증은 문단 모양 대화상자, 정렬, 줄간격, Shift+Tab을 본문/일반 셀에서 수행

## 리스크

- 새 `setParaShapeId*` API가 lineSeg 재계산을 충분히 하지 않으면 줄 간격 undo 후 조판이 즉시 맞지 않을 수 있다.
- 문단 번호/글머리표 정의 테이블 자체는 undo하지 않으므로 unused numbering/bullet 정의가 남을 수 있다.
- 다중 문단 적용 중 일부 대상만 성공한 뒤 실패하면 command entry 정합이 깨질 수 있다. execute 단계에서 target
  validation을 먼저 끝내는 방식이 필요하다.
- Redo가 props 재적용 방식이면 새 ParaShape ID가 다시 만들어질 수 있으므로, 최초 execute 후에는
  `afterParaShapeId` 직접 복원 방식이 안전하다.

## 결론

문단 서식 Undo/Redo는 구현 가능하다. 단, `ParaProperties` 조회 JSON 전체를 undo payload로 쓰는 방식은
단위 계약 때문에 위험하다.

가장 안전한 1차 구현은 `paraShapeId` before/after를 저장하고, 새 WASM restore API로 문단의
`para_shape_id`를 직접 되돌리는 방식이다.

다음 단계에서는 이 결론을 기준으로 구현 계획서(`mydocs/plans/task_m100_1319_impl.md`)를 작성한다.
