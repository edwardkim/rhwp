# Task #1319 구현 계획서 — 문단 서식 Undo/Redo 커맨드 체계화

## 전제

- 수행계획서 승인 완료: `mydocs/plans/task_m100_1319.md`
- Stage 1 진단 승인 완료: `mydocs/working/task_m100_1319_stage1.md`
- 핵심 결론:
  - `ParaProperties` 조회 JSON 전체를 undo payload로 쓰면 안 된다.
  - margin/indent/spacing 계열은 조회 시 UI용 px로 반환되지만 적용 parser는 raw 값을 기대한다.
  - 안정적인 Undo/Redo 저장 단위는 문단의 `paraShapeId` before/after다.

## 목표

문단 서식 변경을 공통 `EditCommand` 체계로 편입하여, 사용자 관점에서 다음 동작을 만족시킨다.

- 문단 모양 대화상자 적용 → `Ctrl+Z` → `Ctrl+Y`
- toolbar/menu 정렬 변경 → `Ctrl+Z` → `Ctrl+Y`
- toolbar/menu 줄 간격 변경 → `Ctrl+Z` → `Ctrl+Y`
- #1318 `Shift+Tab` 내어쓰기 → `Ctrl+Z` → `Ctrl+Y`

## 분할 정복 전략

Undo/Redo의 통합 관리와 WASM 아키텍처 최적화는 한 번에 처리하지 않는다.

이번 #1319에서는 사용자 불편이 확인된 문단 서식 변경 Undo/Redo를 기존 `CommandHistory`에 편입하는
1차 전술 구현까지만 수행한다. 즉, "문단 서식 변경도 기존 history stack에서 같은 사용자 경험으로
되돌릴 수 있게 한다"가 목표다.

작업지시자 동작 테스트 중 확인된 글자 서식 Undo/Redo 불일치는 사용자 경험 기준에서 같은 완료 조건으로
포함한다. 구현 도메인은 글자 서식이지만, 사용자는 `Ctrl+Z`/`Ctrl+Y`가 문단 서식과 글자 서식을 일관되게
복원하기를 기대하므로 #1319의 보강 범위로 처리한다.

후속 이슈에서는 rhwp-studio와 브라우저 확장 앱의 WASM 실행 특성을 고려해, 편집 액션 라우터와
transaction/delta 기반 Undo/Redo 아키텍처를 별도로 설계한다.

구분:

| 단계 | 범위 | 목표 |
|---|---|---|
| 1차: #1319 | 문단/글자 서식 변경 Undo/Redo | `ApplyParaFormatCommand`/`ApplyCharFormatCommand` ID before/after 복원 |
| 2차: #1320 | 편집 액션 라우터/트랜잭션 | UI 직접 WASM 호출 제거, command routing 통합, dirty scope 최적화 |

이렇게 나누는 이유:

- #1319에서 전체 편집 라우터를 재설계하면 작업 범위가 커져 문단 Undo/Redo 성공 기준이 흐려진다.
- 표/이미지/객체/붙여넣기/snapshot은 복원 계약이 서로 다르므로, 공통 인터페이스와 도메인별 복원 책임을
  분리해야 한다.
- Chrome/Edge 확장 환경에서는 WASM 호출, canvas invalidation, document snapshot 비용을 함께 고려해야
  하므로 별도 설계/검증이 필요하다.
- #1319는 후속 아키텍처로 가기 위한 첫 번째 안전한 command 편입 사례로 삼는다.

## 구현 원칙

1. 문단 속성 변경은 `ApplyParaFormatCommand` 하나로 기록한다.
2. command는 `paraShapeId` before/after를 저장한다.
3. Undo/Redo는 props 재계산이 아니라 저장된 `paraShapeId` 직접 복원으로 처리한다.
4. 글자 서식 Undo/Redo도 저장된 `charShapeId` 직접 복원으로 처리한다.
5. 본문 문단과 일반 표 셀 문단을 1차 지원한다.
6. 머리말/꼬리말, 각주/미주, 글상자, 중첩 표는 1차 제외 또는 no-op 처리한다.
7. 문단 번호/글머리표 정의 테이블 자체의 undo는 1차 범위에서 제외한다.

## 기존 Undo/Redo 호환성 제약

추가 조사 결과, 새 command는 다음 제약을 지켜야 기존 Undo/Redo와 충돌하지 않는다.

1. `CommandHistory.redo()`는 `command.execute(wasm)`를 다시 호출한다.
   - `ApplyParaFormatCommand.execute()`는 최초 실행과 redo 실행을 구분해야 한다.
   - `afterParaShapeId`가 이미 있으면 props 재적용 없이 `setParaShapeId*`로 after ID를 복원한다.
2. `mergeWith()`는 항상 `null`을 반환한다.
   - 문단 서식 변경은 사용자가 명시적으로 수행한 작업 단위다.
   - 연속 정렬/줄간격/Shift+Tab 변경을 자동 병합하지 않는다.
3. `recordWithoutExecute()`를 사용하지 않는다.
   - 문단 서식은 사후 기록이 아니라 command 실행 경로에서 적용과 기록을 동시에 처리한다.
4. `SnapshotCommand`를 사용하지 않는다.
   - 문단 서식 변경은 전체 문서 스냅샷보다 `paraShapeId` before/after 복원이 더 작고 명확하다.
5. page-local refresh 대상에 추가하지 않는다.
   - 문단 서식은 줄 높이, 줄 수, 페이지 흐름에 영향을 줄 수 있으므로 full `afterEdit()` 경로를 유지한다.
6. 선택 범위 UX는 구현 단계에서 명시 처리한다.
   - 현재 `executeOperation()`은 `applyCharFormat`만 selection을 유지한다.
   - 문단 모양 대화상자 적용도 selection 유지가 자연스러운지 확인하고, 필요하면 `applyParaFormat`도 특례에 포함한다.

## 중복 제거 원칙

현재 직접 WASM 호출이 분산되어 있으므로 command를 추가하는 것만으로는 충분하지 않다.

- `InputHandler.applyParaFormat()`
- `InputHandler.applyParaPropsToRange()`
- `InputHandler.applyHangingIndentAtCursor()`
- `format:para-num-shape` 일부 직접 `services.wasm.applyParaFormat(...)` 경로

위 경로는 모두 `ApplyParaFormatCommand` 또는 그 helper로 들어오도록 정리한다.

역할 분리는 다음과 같이 둔다.

- `InputHandler`: cursor/selection/context를 해석해 `ParaFormatTarget[]` 산정
- `ApplyParaFormatCommand`: target별 before/after `paraShapeId` 저장, execute/undo/redo 수행
- `WasmBridge`: raw restore API 제공
- Rust core: `para_shape_id` 교체, dirty/rebuild/event 처리

이렇게 하면 `command.ts`에 cursor/selection 해석 로직을 중복하지 않는다.

단, #1319에서 모든 편집 동작을 새 라우터로 이전하지는 않는다. #1319의 중복 제거는 문단 서식 변경 경로에
한정한다.

#1320 후속 아키텍처 이슈에서 검토할 장기 모델:

```text
UI intent
  -> EditActionRouter
  -> context/target resolver
  -> domain command or transaction
  -> WASM mutation
  -> undo delta / redo delta
  -> dirty scope
  -> renderer invalidation
```

후속 모델의 핵심 판단:

- history stack은 하나로 통합한다.
- 복원 payload는 도메인별로 분리한다.
- 가능하면 WASM core가 before/after delta와 dirty scope를 반환한다.
- JS는 단위 변환된 UI props를 복원 payload로 재사용하지 않는다.
- full document snapshot은 복합 구조 변경에만 제한적으로 사용한다.

## Stage 2. WASM restore API 추가

### Rust

`src/document_core/commands/formatting.rs`에 복원 API native helper를 추가한다.

후보 함수:

```rust
pub fn set_para_shape_id_native(
    &mut self,
    sec_idx: usize,
    para_idx: usize,
    para_shape_id: u16,
) -> Result<String, HwpError>

pub fn set_cell_para_shape_id_native(
    &mut self,
    sec_idx: usize,
    parent_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    para_shape_id: u16,
) -> Result<String, HwpError>
```

처리 내용:

- `para_shape_id` 범위 검증
- 대상 문단 `para_shape_id` 교체
- body 문단:
  - section `raw_stream = None`
  - 보수적으로 `rebuild_section(sec_idx)`
  - `DocumentEvent::ParaFormatChanged`
- cell 문단:
  - cell paragraph `para_shape_id` 교체
  - 부모 표 dirty 마킹
  - section `raw_stream = None`
  - 보수적으로 `rebuild_section(sec_idx)`
  - `DocumentEvent::ParaFormatChanged`

lineSeg 재계산은 1차에서 보수적으로 rebuild를 사용한다. 줄 간격 변경의 이전/이후 ParaShape 복원도 조판에
반영되어야 하므로, 필요 시 기존 `apply_para_format_native()`의 `reflow_line_segs()` 호출 조건을 참고해
추가 보강한다.

### WASM binding

`src/wasm_api.rs`에 다음 export 추가:

```rust
#[wasm_bindgen(js_name = setParaShapeId)]
pub fn set_para_shape_id(&mut self, sec_idx: usize, para_idx: usize, para_shape_id: u16)

#[wasm_bindgen(js_name = setCellParaShapeId)]
pub fn set_cell_para_shape_id(
    &mut self,
    sec_idx: usize,
    parent_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    para_shape_id: u16,
)
```

### TypeScript bridge

`rhwp-studio/src/core/wasm-bridge.ts`에 wrapper 추가:

```ts
setParaShapeId(sec: number, para: number, paraShapeId: number): string
setCellParaShapeId(sec: number, parentPara: number, controlIdx: number, cellIdx: number, cellParaIdx: number, paraShapeId: number): string
```

## Stage 3. ApplyParaFormatCommand 추가

`rhwp-studio/src/engine/command.ts`에 문단 서식 command 추가.

### 타입

```ts
type ParaFormatTarget =
  | { kind: 'body'; sec: number; para: number }
  | { kind: 'cell'; sec: number; parentPara: number; controlIdx: number; cellIdx: number; cellParaIdx: number };

interface ParaFormatHistoryEntry {
  target: ParaFormatTarget;
  beforeParaShapeId: number;
  afterParaShapeId?: number;
}
```

### execute

1. target validation
2. 최초 실행이면 각 target의 현재 `paraShapeId` 저장
3. target별 기존 `applyParaFormat*` 호출
4. 적용 후 `paraShapeId` 재조회
5. `afterParaShapeId` 저장
6. 커서 위치 반환

redo 시에는 `afterParaShapeId`가 있으면 props를 다시 적용하지 않고 `setParaShapeId*`로 복원한다.
이렇게 해야 `find_or_create_para_shape()`가 새로운 ID를 다시 만들거나 부수 definition을 늘리는 일을 줄일 수 있다.

### undo

각 entry의 `beforeParaShapeId`를 `setParaShapeId*`로 복원한다.

### mergeWith

1차 구현에서는 병합하지 않는다.

이유:

- 문단 모양 대화상자 적용, toolbar 정렬, 줄 간격, Shift+Tab은 각각 사용자가 명시적으로 수행한 작업이다.
- 연속 정렬 변경을 병합하면 사용자가 기대하는 Ctrl+Z 단계와 어긋날 수 있다.

### executeOperation 연동

command type은 `applyParaFormat`으로 둔다.

`executeOperation()` 연동 시 다음을 확인한다.

- `applyParaFormat`은 page-local refresh 대상이 아니므로 full `afterEdit()`가 수행되어야 한다.
- selection 유지가 필요한 경우 `applyCharFormat` 특례와 같은 방식으로 `applyParaFormat`을 추가한다.
- `record` kind는 사용하지 않는다.

## Stage 4. 대상 산정 helper 추가

`InputHandler`에 문단 서식 대상 산정 helper를 추가한다.

후보:

```ts
private getParaFormatTargetsForRange(start: DocumentPosition, end: DocumentPosition): ParaFormatTarget[]
private getParaFormatTargetsAtCursor(): ParaFormatTarget[]
```

1차 규칙:

- body:
  - selection이 있으면 `start.paragraphIndex..end.paragraphIndex`
  - selection이 없으면 현재 문단 1개
- cell:
  - 같은 parentPara/control/cell 내부 cellPara 범위만 지원
  - 다른 셀로 걸친 선택, 중첩 cellPath, 글상자 문맥은 no-op
- note/header/footer:
  - 1차 no-op 또는 기존 직접 경로 유지 금지
  - 사용자-facing 오류 대신 console info로 후속 범위 안내

## Stage 5. 호출 경로 전환

### `applyParaPropsToRange()`

문단 모양 대화상자의 진입점이다.

변경:

```ts
const targets = this.getParaFormatTargetsForRange(start, end);
const cmd = new ApplyParaFormatCommand(targets, props, start);
this.executeOperation({ kind: 'command', command: cmd });
```

### `applyParaFormat()`

toolbar/menu 정렬, 줄 간격, 문단 머리 계열이 사용하는 내부 helper다.

변경:

- 대상 산정 helper를 사용해 command 실행
- footnote/header/footer 문맥은 1차 제외 처리
- `afterEdit()` 직접 호출 제거

### `applyHangingIndentAtCursor()`

#1318 Shift+Tab 경로다.

변경:

- 기존 좌표 산식으로 `indent` props 계산
- 직접 `wasm.applyParaFormat*` 호출 대신 `ApplyParaFormatCommand` 실행
- 성공 시 `true`, unsupported 문맥은 기존처럼 `false`

### `format:para-num-shape`

현재 일부 경로에서 `services.wasm.applyParaFormat(...)`를 직접 호출한다.

변경:

- `ih.applyParaPropsToRange()` 또는 새 `ih.applyParaPropsAtCursor()` helper로 우회
- 단, numbering restart 또는 definition 변경 자체는 1차 Undo 범위에서 제외 가능
- 직접 `services.eventBus.emit('document-changed')`만 호출하는 우회 경로를 남기지 않는다.

## Stage 6. 검증

로컬 검증:

```text
npm run build
cargo fmt --all -- --check
cargo test --lib
```

필요 시 WASM:

```text
docker compose --env-file .env.docker run --rm wasm
```

수동 검증:

1. 본문 문단
   - 문단 모양 대화상자에서 정렬/여백/내어쓰기 변경
   - `Ctrl+Z`로 복원
   - `Ctrl+Y`로 재적용
2. toolbar/menu 정렬
   - 왼쪽/가운데/오른쪽 정렬 변경 후 Undo/Redo
3. 줄 간격
   - 줄 간격 변경 후 Undo/Redo
4. Shift+Tab
   - 커서 기준 내어쓰기 적용 후 Undo/Redo
5. 일반 표 셀
   - 셀 문단에서 문단 모양/Shift+Tab Undo/Redo
6. 기존 history 회귀
   - 텍스트 입력/삭제 Undo/Redo
   - 글자 서식 Undo/Redo
   - 객체 이동/크기 조절 Undo/Redo
   - paste/delete snapshot Undo/Redo

## 성공 기준

1. 문단 속성 변경이 history stack에 기록된다.
2. `Ctrl+Z` 한 번으로 직전 문단 속성 변경 전 조판으로 돌아간다.
3. `Ctrl+Y` 한 번으로 직전 문단 속성 변경 후 조판으로 돌아간다.
4. body/cell 문맥에서 동작한다.
5. unsupported 문맥에서 콘솔 오류가 발생하지 않는다.
6. 기존 텍스트/글자 서식/객체 history 동작과 충돌하지 않는다.
7. #1319 구현 범위와 후속 아키텍처 이슈 범위가 분리되어, 현재 작업이 과도한 라우터 재설계로 확장되지 않는다.

## 리스크와 대응

| 리스크 | 대응 |
|---|---|
| `paraShapeId` 직접 복원 후 lineSeg가 오래된 값으로 남음 | restore API에서 보수적 rebuild, 필요 시 lineSeg reflow 추가 |
| 문단 번호/글머리표 definition table이 undo되지 않음 | 1차는 문단의 `paraShapeId` 복원까지만 수용, definition cleanup은 후속 |
| selection이 여러 셀/중첩 표에 걸침 | 1차 no-op 또는 명시 제외 |
| 머리말/각주/미주 문맥 회귀 | 1차 제외, 후속 이슈로 분리 |
| redo 시 props 재적용으로 새 ParaShape ID 증가 | afterParaShapeId 직접 복원 |
| 직접 WASM 호출 경로가 남아 Undo/Redo를 우회함 | 모든 문단 서식 UI 진입점을 `InputHandler` helper와 command 경로로 통합 |
| `executeOperation()`이 selection을 해제함 | 문단 모양 적용 UX 기준에 맞춰 `applyParaFormat` selection 유지 여부를 구현 단계에서 결정 |

## 승인 요청

본 구현 계획서를 승인하면 Stage 2부터 소스 수정을 시작한다.
