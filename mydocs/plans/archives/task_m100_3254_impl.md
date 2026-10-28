# rhwp-studio Korean IME Atomic Cell Replace Implementation Plan

**Goal:** 1단계 표 셀의 한글 IME 조합 교체를 한 번의 deferred atomic WASM mutation으로 처리해 빠른 연속 입력의 동기 지연을 16ms frame 안으로 줄인다.

**Architecture:** Rust `DocumentCore`가 이전 조합 문자열 삭제와 새 조합 문자열 삽입을 한 mutable access와 한 번의 cell reflow로 처리하고, 안정 흐름에서는 전체 pagination을 미룬다. Studio bridge는 atomic API 지원 여부와 결과를 typed mutation effect로 변환하며, IME `replaceTextAtRaw`만 이 경로를 사용한다. 조합 확정 history, 일반 Backspace/Delete, 중첩 셀은 기존 계약을 유지한다.

**Tech Stack:** Rust, `wasm-bindgen`, TypeScript, Node test runner, Vite, WebAssembly, browser runtime

## Global Constraints

- 승인 설계: `mydocs/plans/task_m100_3254.md`
- 범위는 줄바꿈·탭 없는 최대 8자의 depth-1 셀 IME replace다.
- 일반 Backspace/Delete, depth-2 이상 셀, 본문, 머리말·꼬리말·각주는 변경하지 않는다.
- 안정 replace는 문서 모델과 현재 페이지를 즉시 갱신하고 전체 pagination만 기존 120ms 유휴 경계로 보낸다.
- 실제 `cellFlowChanged` 경계는 caret 조회 전에 기존 동기 flush를 수행한다.
- API 누락 fallback은 mutation 전에 결정하며, atomic API 호출 뒤 오류에서는 delete+insert를 재시도하지 않는다.
- `compositionend`는 최종 문자열 하나만 history에 기록하는 기존 계약을 유지한다.
- toolbar, zoom, 접근성, Subsecond 변경을 수정하거나 되돌리지 않는다.
- `src/wasm_api.rs`와 `rhwp-studio/src/core/wasm-bridge.ts`에는 사용자 미커밋 변경이 있으므로 `git add -p`로 이 작업 hunk만 stage한다.
- 원격 push와 PR 생성은 별도 사용자 승인 전에는 수행하지 않는다.

---

### Task 1: Rust atomic deferred cell replace

**Files:**

- Modify: `src/wasm_api/tests.rs:1776`
- Modify: `src/wasm_api/tests.rs:1920`
- Modify: `src/document_core/commands/text_editing.rs:940`
- Modify: `src/wasm_api.rs:1040`

**Interfaces:**

- Consumes: 기존 `insert_text_in_cell_native_impl`의 cell mutation, reflow, normalized-cache coherence, `cellFlowChanged` 계산
- Produces:

```rust
pub fn replace_text_in_cell_native_deferred_pagination(
    &mut self,
    section_idx: usize,
    parent_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
    delete_count: usize,
    text: &str,
) -> Result<String, HwpError>
```

```ts
replaceTextInCellDeferredPagination(
  sec: number,
  parentPara: number,
  controlIdx: number,
  cellIdx: number,
  cellParaIdx: number,
  charOffset: number,
  deleteCount: number,
  text: string,
): string
```

- [ ] **Step 1: atomic IME replace 실패 테스트 작성**

`create_doc_with_table()` 아래의 셀 편집 테스트에 다음 테스트를 추가한다.

```rust
#[test]
fn deferred_cell_replace_applies_ime_atomically() {
    use crate::renderer::render_tree::{RenderNode, RenderNodeType};

    fn contains_text(node: &RenderNode, needle: &str) -> bool {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains(needle) {
                return true;
            }
        }
        node.children
            .iter()
            .any(|child| contains_text(child, needle))
    }

    let mut doc = create_doc_with_table();
    doc.insert_text_in_cell_native_deferred_pagination(0, 0, 0, 0, 0, 2, "ㅎ")
        .expect("seed composition");
    doc.build_page_render_tree(0).expect("warm page tree");

    let raw = doc
        .replace_text_in_cell_native_deferred_pagination(
            0, 0, 0, 0, 0, 2, 1, "하",
        )
        .expect("atomic composition replace");
    let result: Value = serde_json::from_str(&raw).expect("replace result json");

    assert_eq!(result["charOffset"].as_u64(), Some(3));
    assert_eq!(result["cellFlowChanged"].as_bool(), Some(false));
    match &doc.document.sections[0].paragraphs[0].controls[0] {
        Control::Table(table) => {
            let para = &table.cells[0].paragraphs[0];
            assert_eq!(para.text, "셀A하");
            assert_eq!(para.char_count, 3);
            assert_eq!(para.char_offsets, make_char_offsets("셀A하"));
        }
        other => panic!("table control expected: {other:?}"),
    }

    let transient_tree = doc.build_page_render_tree(0).expect("transient page tree");
    assert!(
        contains_text(&transient_tree.root, "하"),
        "warm page tree must expose the final composition before pagination"
    );
    assert_eq!(doc.event_log.len(), 2, "seed insert + atomic replace");
    assert!(matches!(
        doc.event_log.last(),
        Some(crate::model::event::DocumentEvent::CellTextChanged {
            section: 0,
            para: 0,
            ctrl: 0,
            cell: 0,
        })
    ));
}

#[test]
fn deferred_cell_replace_reports_real_flow_boundary() {
    use crate::model::shape::{Caption, CaptionDirection};

    let mut doc = create_doc_with_table();
    match &mut doc.document.sections[0].paragraphs[0].controls[0] {
        Control::Table(table) => {
            table.caption = Some(Caption {
                direction: CaptionDirection::Bottom,
                width: 2_000,
                max_width: 2_000,
                paragraphs: vec![Paragraph {
                    text: "가".to_string(),
                    char_count: 1,
                    char_offsets: make_char_offsets("가"),
                    line_segs: vec![LineSeg {
                        line_height: 400,
                        baseline_distance: 320,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            });
        }
        other => panic!("table control expected: {other:?}"),
    }
    doc.reflow_cell_paragraph(0, 0, 0, 65534, 0);

    let raw = doc
        .replace_text_in_cell_native_deferred_pagination(
            0, 0, 0, 65534, 0, 0, 1, "가나다라마바사아",
        )
        .expect("caption boundary replace");
    let result: Value = serde_json::from_str(&raw).expect("boundary result json");
    assert_eq!(result["cellFlowChanged"].as_bool(), Some(true));
    match &doc.document.sections[0].paragraphs[0].controls[0] {
        Control::Table(table) => assert!(
            table.caption.as_ref().expect("table caption").paragraphs[0]
                .line_segs.len() > 1,
            "replacement must cross a line-flow boundary"
        ),
        other => panic!("table control expected: {other:?}"),
    }
}

#[test]
fn deferred_cell_replace_preserves_clickhere_range_and_offsets() {
    let mut doc = create_doc_with_table();
    doc.insert_click_here_field_at_in_cell(
        0, 0, 0, 0, 0, 2, false, "안내", "메모", "이름", true,
    )
    .expect("insert empty ClickHere");
    doc.insert_text_in_cell_native_deferred_pagination(0, 0, 0, 0, 0, 2, "ㅎ")
        .expect("seed field composition");
    doc.event_log.clear();

    doc.replace_text_in_cell_native_deferred_pagination(
        0, 0, 0, 0, 0, 2, 1, "하",
    )
    .expect("replace field composition");

    match &doc.document.sections[0].paragraphs[0].controls[0] {
        Control::Table(table) => {
            let para = &table.cells[0].paragraphs[0];
            assert_eq!(para.text, "셀A하");
            assert_eq!(para.char_offsets, make_char_offsets("셀A하"));
            assert_eq!(para.field_ranges.len(), 1);
            assert_eq!(para.field_ranges[0].start_char_idx, 2);
            assert_eq!(para.field_ranges[0].end_char_idx, 3);
        }
        other => panic!("table control expected: {other:?}"),
    }
    assert_eq!(doc.event_log.len(), 1, "replace emits only final cell state");
}

#[test]
fn deferred_cell_replace_rejects_invalid_input_before_mutation() {
    let mut doc = create_doc_with_table();
    let before = match &doc.document.sections[0].paragraphs[0].controls[0] {
        Control::Table(table) => table.cells[0].paragraphs[0].text.clone(),
        other => panic!("table control expected: {other:?}"),
    };

    let result = doc.replace_text_in_cell_native_deferred_pagination(
        0, 0, 0, 0, 0, 2, 1, "가나다라마바사아자",
    );

    assert!(result.is_err(), "more than eight replacement chars must fail");
    match &doc.document.sections[0].paragraphs[0].controls[0] {
        Control::Table(table) => assert_eq!(table.cells[0].paragraphs[0].text, before),
        other => panic!("table control expected: {other:?}"),
    }
}
```

- [ ] **Step 2: focused Rust test가 올바르게 RED인지 확인**

Run:

```bash
cargo test --lib deferred_cell_replace_ -- --nocapture
```

Expected: compile failure containing
`no method named replace_text_in_cell_native_deferred_pagination`.

- [ ] **Step 3: insert 구현을 atomic replace 내부 함수로 일반화**

`insert_text_in_cell_native`와 `insert_text_in_cell_native_deferred_pagination`은 기존 인자를
유지하되 내부 호출에 `delete_count = 0`을 넘긴다.

```rust
pub fn insert_text_in_cell_native(
    &mut self,
    section_idx: usize,
    parent_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
    text: &str,
) -> Result<String, HwpError> {
    self.replace_text_in_cell_native_impl(
        section_idx,
        parent_para_idx,
        control_idx,
        cell_idx,
        cell_para_idx,
        char_offset,
        0,
        text,
        true,
    )
}

pub fn insert_text_in_cell_native_deferred_pagination(
    &mut self,
    section_idx: usize,
    parent_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
    text: &str,
) -> Result<String, HwpError> {
    self.replace_text_in_cell_native_impl(
        section_idx,
        parent_para_idx,
        control_idx,
        cell_idx,
        cell_para_idx,
        char_offset,
        0,
        text,
        false,
    )
}
```

새 public replace API는 지원 범위를 mutation 전에 검증한다.

```rust
pub fn replace_text_in_cell_native_deferred_pagination(
    &mut self,
    section_idx: usize,
    parent_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
    delete_count: usize,
    text: &str,
) -> Result<String, HwpError> {
    let new_chars_count = text.chars().count();
    if delete_count == 0
        || delete_count > 8
        || new_chars_count == 0
        || new_chars_count > 8
        || text.chars().any(|ch| matches!(ch, '\r' | '\n' | '\t'))
    {
        return Err(HwpError::RenderError(
            "deferred 셀 replace는 줄바꿈·탭 없는 1~8자 교체만 지원합니다".to_string(),
        ));
    }
    self.replace_text_in_cell_native_impl(
        section_idx,
        parent_para_idx,
        control_idx,
        cell_idx,
        cell_para_idx,
        char_offset,
        delete_count,
        text,
        false,
    )
}
```

기존 `insert_text_in_cell_native_impl`을 `replace_text_in_cell_native_impl`로 바꾸고
signature에 `delete_count`를 추가한다. 기존 `flow_advance_before`와
`local_contribution_before` 캡처 뒤, inactive field insertion 계산 전에 다음 mutation을
적용한다.

```rust
let deleted_count = if delete_count > 0 {
    cell_para.delete_text_at(char_offset, delete_count)
} else {
    0
};
let new_chars_count = text.chars().count();
let outside_insertions = inactive_field_end_insertions(
    cell_para,
    active_field.as_ref(),
    section_idx,
    cell_para_idx,
    Some(&cell_path),
    char_offset,
);
let before_insertions = inactive_field_start_insertions(
    cell_para,
    active_field.as_ref(),
    section_idx,
    cell_para_idx,
    Some(&cell_path),
    char_offset,
);
if new_chars_count > 0 {
    cell_para.insert_text_at(char_offset, text);
    keep_inactive_field_start_outside(cell_para, &before_insertions, new_chars_count);
    keep_inactive_field_end_outside(cell_para, &outside_insertions, new_chars_count);
    if has_clickhere_field_range(cell_para) {
        rebuild_char_offsets(cell_para);
    }
}
debug_assert!(deleted_count <= delete_count);
```

나머지 reflow, vpos, cell-unit invalidation, `refresh_render_normalized_cell_paragraph_after_edit`,
page-tree invalidation, event 기록은 기존 함수의 순서와 1회 호출을 그대로 유지한다.
최종 offset은 다음처럼 새 문자열 길이만 반영한다.

```rust
let new_offset = char_offset + new_chars_count;
```

- [ ] **Step 4: wasm-bindgen atomic API 노출**

`insertTextInCellDeferredPagination` 바로 뒤에 다음 binding을 추가한다.

```rust
#[wasm_bindgen(js_name = replaceTextInCellDeferredPagination)]
pub fn replace_text_in_cell_deferred_pagination(
    &mut self,
    section_idx: u32,
    parent_para_idx: u32,
    control_idx: u32,
    cell_idx: u32,
    cell_para_idx: u32,
    char_offset: u32,
    delete_count: u32,
    text: &str,
) -> Result<String, JsValue> {
    self.replace_text_in_cell_native_deferred_pagination(
        section_idx as usize,
        parent_para_idx as usize,
        control_idx as usize,
        cell_idx as usize,
        cell_para_idx as usize,
        char_offset as usize,
        delete_count as usize,
        text,
    )
    .map_err(|e| e.into())
}
```

- [ ] **Step 5: Rust 테스트 GREEN과 기존 deferred insert 무회귀 확인**

Run:

```bash
cargo test --lib deferred_cell_replace_ -- --nocapture
cargo test --lib issue2214_deferred_ -- --nocapture
cargo test --lib test_insert_text_in_cell -- --nocapture
```

Expected: all selected tests PASS; immediate insert response에는 `cellFlowChanged`가 없고 deferred
insert/replace에는 boolean 신호가 있다.

- [ ] **Step 6: Rust 변경만 커밋**

`src/wasm_api.rs` 전체를 stage하지 않는다.

```bash
git add src/document_core/commands/text_editing.rs src/wasm_api/tests.rs
git add -p src/wasm_api.rs
git diff --cached --check
git diff --cached --stat
git commit -m "perf: add atomic deferred cell text replace"
```

Expected: staged diff에 기존 Subsecond WASM hunk가 없다.

---

### Task 2: Typed Studio bridge and mutation effects

**Files:**

- Modify: `rhwp-studio/src/core/wasm-bridge.ts:84`
- Modify: `rhwp-studio/src/core/wasm-bridge.ts:950`
- Modify: `rhwp-studio/src/engine/command.ts:165`
- Modify: `rhwp-studio/tests/cell-flow-boundary.test.ts:40`

**Interfaces:**

- Consumes: Task 1의 `replaceTextInCellDeferredPagination` WASM method
- Produces:

```ts
export interface DeferredCellTextMutationResult {
  ok: true;
  charOffset: number;
  paginationDeferred: boolean;
  cellFlowChanged: boolean;
}
```

```ts
export function canUseDeferredCellTextReplace(
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): boolean
```

```ts
export function replaceCellTextWithMutationEffects(
  wasm: WasmBridge,
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): TextMutationEffects
```

- [ ] **Step 1: command helper 실패 테스트 작성**

`tests/cell-flow-boundary.test.ts`의 compile import 목록에
`canUseDeferredCellTextReplace`와 `replaceCellTextWithMutationEffects`를 추가한다.
`FakeWasm`에 다음 메서드를 추가한다.

```ts
replaceTextInCellDeferredPagination(...args) {
  this.calls.push({ name: 'replace-deferred', args });
  const result = this.deferredResults.shift();
  assert.ok(result, 'deferred mutation result fixture exhausted');
  return result;
}
```

테스트를 추가한다.

```ts
test('depth-1 IME replace는 atomic deferred mutation 한 번만 사용한다', () => {
  const wasm = new FakeWasm(mutationResult(false));
  const position = depth1Position(7);

  assert.equal(canUseDeferredCellTextReplace(position, 1, '하'), true);
  assert.deepEqual(
    replaceCellTextWithMutationEffects(wasm, position, 1, '하'),
    {
      documentPaginationPending: true,
      flowChanged: false,
      paginationCompleted: false,
    },
  );
  assert.deepEqual(wasm.calls, [{
    name: 'replace-deferred',
    args: [0, 5, 2, 3, 0, 7, 1, '하'],
  }]);
});

test('flow boundary replace effect는 pre-caret flush 신호를 보존한다', () => {
  const wasm = new FakeWasm(mutationResult(true));

  assert.deepEqual(
    replaceCellTextWithMutationEffects(wasm, depth1Position(7), 1, '가나다라마바사아'),
    {
      documentPaginationPending: true,
      flowChanged: true,
      paginationCompleted: false,
    },
  );
});

test('중첩 셀과 빈 replacement는 atomic cell replace 대상이 아니다', () => {
  assert.equal(canUseDeferredCellTextReplace(depth2Position(), 1, '하'), false);
  assert.equal(canUseDeferredCellTextReplace(depth1Position(), 1, ''), false);
  assert.equal(canUseDeferredCellTextReplace(depth1Position(), 0, '하'), false);
  assert.equal(canUseDeferredCellTextReplace(depth1Position(), 1, '가나다라마바사아자'), false);
});

test('atomic API fallback 결과는 immediate-completed effect다', () => {
  const wasm = new FakeWasm({
    ok: true,
    charOffset: 8,
    paginationDeferred: false,
    cellFlowChanged: false,
  });

  assert.deepEqual(
    replaceCellTextWithMutationEffects(wasm, depth1Position(7), 1, '하'),
    {
      documentPaginationPending: false,
      flowChanged: false,
      paginationCompleted: true,
    },
  );
});
```

- [ ] **Step 2: focused Studio test가 올바르게 RED인지 확인**

Run:

```bash
node --test tests/cell-flow-boundary.test.ts
```

Expected: runtime compile 또는 import failure에서
`canUseDeferredCellTextReplace`가 없다고 보고한다.

- [ ] **Step 3: bridge result type과 atomic feature-detected method 구현**

기존 `DeferredCellTextInsertResult`를 다음 이름으로 일반화하고 insert method 반환형도 갱신한다.

```ts
export interface DeferredCellTextMutationResult {
  ok: true;
  charOffset: number;
  paginationDeferred: boolean;
  cellFlowChanged: boolean;
}
```

`insertTextInCellDeferredPagination` 뒤에 다음 method를 추가한다.

```ts
replaceTextInCellDeferredPagination(
  sec: number,
  parentPara: number,
  controlIdx: number,
  cellIdx: number,
  cellParaIdx: number,
  charOffset: number,
  deleteCount: number,
  text: string,
): DeferredCellTextMutationResult {
  if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
  const d = this.doc as unknown as {
    replaceTextInCellDeferredPagination?: (
      sec: number,
      parentPara: number,
      controlIdx: number,
      cellIdx: number,
      cellParaIdx: number,
      charOffset: number,
      deleteCount: number,
      text: string,
    ) => string;
  };

  let raw: string;
  let paginationDeferred = false;
  if (typeof d.replaceTextInCellDeferredPagination === 'function') {
    raw = d.replaceTextInCellDeferredPagination(
      sec,
      parentPara,
      controlIdx,
      cellIdx,
      cellParaIdx,
      charOffset,
      deleteCount,
      text,
    );
    paginationDeferred = true;
  } else {
    if (deleteCount > 0) {
      raw = this.doc.deleteTextInCell(
        sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, deleteCount,
      );
    } else {
      raw = JSON.stringify({ ok: true, charOffset });
    }
    if (text.length > 0) {
      raw = this.doc.insertTextInCell(
        sec, parentPara, controlIdx, cellIdx, cellParaIdx, charOffset, text,
      );
    }
  }

  const parsed = JSON.parse(raw) as Partial<DeferredCellTextMutationResult>;
  if (
    parsed.ok !== true
    || !Number.isInteger(parsed.charOffset)
  ) {
    throw new Error('잘못된 deferred cell text replace 결과');
  }
  return {
    ok: true,
    charOffset: parsed.charOffset!,
    paginationDeferred,
    cellFlowChanged: paginationDeferred && parsed.cellFlowChanged !== false,
  };
}
```

Feature detection은 첫 mutation 전에 끝나며, atomic method가 존재하지만 호출 또는 결과 검증에
실패하면 fallback을 재시도하지 않는다.

- [ ] **Step 4: command의 지원 판정과 effect helper 구현**

`canUseDeferredCellTextInsert` 옆에 다음 판정을 추가한다.

```ts
export function canUseDeferredCellTextReplace(
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): boolean {
  if (!isCell(pos) || isNestedCell(pos)) return false;
  if (!Number.isInteger(deleteCount) || deleteCount < 1 || deleteCount > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS) {
    return false;
  }
  if (charCount(text) < 1 || charCount(text) > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS) return false;
  if (/[\r\n\t]/.test(text)) return false;
  return true;
}
```

`replaceBodyTextWithMutationEffects` 옆에 다음 helper를 추가한다.

```ts
export function replaceCellTextWithMutationEffects(
  wasm: WasmBridge,
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): TextMutationEffects {
  const result = wasm.replaceTextInCellDeferredPagination(
    pos.sectionIndex,
    pos.parentParaIndex!,
    pos.controlIndex!,
    pos.cellIndex!,
    pos.cellParaIndex!,
    pos.charOffset,
    deleteCount,
    text,
  );
  return {
    documentPaginationPending: result.paginationDeferred,
    flowChanged: result.paginationDeferred && result.cellFlowChanged,
    paginationCompleted: !result.paginationDeferred,
  };
}
```

- [ ] **Step 5: focused Studio test GREEN과 기존 command effects 무회귀 확인**

Run:

```bash
node --test tests/cell-flow-boundary.test.ts
node --test tests/local-text-replace-result.test.ts
```

Expected: both test files PASS.

- [ ] **Step 6: bridge/helper 변경만 커밋**

`wasm-bridge.ts` 전체를 stage하지 않는다.

```bash
git add rhwp-studio/src/engine/command.ts rhwp-studio/tests/cell-flow-boundary.test.ts
git add -p rhwp-studio/src/core/wasm-bridge.ts
git diff --cached --check
git diff --cached --stat
git commit -m "perf: expose deferred cell replacement effects"
```

Expected: staged diff에 기존 Subsecond bridge hunk가 없다.

---

### Task 3: Route Korean IME replacement atomically

**Files:**

- Modify: `rhwp-studio/src/engine/input-handler-text.ts:1`
- Modify: `rhwp-studio/src/engine/input-handler-text.ts:671`
- Modify: `rhwp-studio/tests/input-edit-invalidation.test.ts:98`

**Interfaces:**

- Consumes: Task 2의 `canUseDeferredCellTextReplace`와 `replaceCellTextWithMutationEffects`
- Produces: depth-1 셀 `replaceTextAtRaw`가 delete+insert 대신 atomic mutation effect를 반환

- [ ] **Step 1: IME route 실패 테스트 작성**

`tests/input-edit-invalidation.test.ts`의 raw cell input 테스트 뒤에 다음 테스트를 추가한다.

```ts
test('depth-1 셀 IME replacement는 body fallback보다 먼저 atomic helper를 사용한다', () => {
  const textSource = readFileSync(
    new URL('../src/engine/input-handler-text.ts', import.meta.url),
    'utf8',
  );
  const replaceStart = textSource.indexOf('export function replaceTextAtRaw(');
  const deleteStart = textSource.indexOf('export function deleteTextAt(', replaceStart);
  const replaceSource = textSource.slice(replaceStart, deleteStart);

  assert.match(
    replaceSource,
    /canUseDeferredCellTextReplace\(pos, deleteCount, text\)/,
  );
  assert.match(
    replaceSource,
    /return replaceCellTextWithMutationEffects\(this\.wasm, pos, deleteCount, text\);/,
  );
  assert.ok(
    replaceSource.indexOf('canUseDeferredCellTextReplace') <
      replaceSource.indexOf('canUseLocalBodyTextReplace'),
    'cell atomic route must be checked before the body-only route',
  );
});
```

- [ ] **Step 2: route test가 올바르게 RED인지 확인**

Run:

```bash
node --test tests/input-edit-invalidation.test.ts
```

Expected: FAIL because `replaceTextAtRaw` does not mention
`canUseDeferredCellTextReplace`.

- [ ] **Step 3: input-handler import와 atomic route 구현**

`./command`의 기존 named import 목록에서
`canUseLocalBodyTextReplace` 바로 앞에 두 helper를 추가한다.

```ts
  insertTextWithMutationEffects,
  replaceBodyTextWithMutationEffects,
  canUseDeferredCellTextReplace,
  replaceCellTextWithMutationEffects,
  canUseLocalBodyTextReplace,
```

`replaceTextAtRaw`에서 form-mode guard 뒤, body local branch보다 먼저 다음을 추가한다.

```ts
if (
  !this.cursor.isInHeaderFooter()
  && !this.cursor.isInFootnote()
  && canUseDeferredCellTextReplace(pos, deleteCount, text)
) {
  return replaceCellTextWithMutationEffects(
    this.wasm,
    pos,
    deleteCount,
    text,
  );
}
```

기존 body local branch와 unsupported delete+insert accumulator는 그대로 둔다.

- [ ] **Step 4: IME route와 관련 회귀 테스트 GREEN 확인**

Run:

```bash
node --test tests/input-edit-invalidation.test.ts
node --test tests/ime-composition-char-count.test.ts
node --test tests/cell-flow-boundary.test.ts
```

Expected: all selected tests PASS.

- [ ] **Step 5: IME route만 커밋**

```bash
git add rhwp-studio/src/engine/input-handler-text.ts \
        rhwp-studio/tests/input-edit-invalidation.test.ts
git diff --cached --check
git diff --cached --stat
git commit -m "perf: route Korean cell IME through atomic replace"
```

---

### Task 4: Rebuild, verify, and remeasure the exact live flow

**Files:**

- Verify: `pkg/rhwp.js`
- Verify: `pkg/rhwp_bg.wasm`
- Verify: `rhwp-studio/`
- Verify: live `http://localhost:7701/`

**Interfaces:**

- Consumes: Tasks 1–3
- Produces: focused regression proof, production build proof, exact 43-page live latency and restoration evidence

- [ ] **Step 1: focused Rust and Studio verification**

Run:

```bash
cargo test --lib deferred_cell_replace_ -- --nocapture
cargo test --lib issue2214_deferred_ -- --nocapture
cd rhwp-studio
node --test tests/cell-flow-boundary.test.ts \
                 tests/input-edit-invalidation.test.ts \
                 tests/ime-composition-char-count.test.ts \
                 tests/local-text-replace-result.test.ts
```

Expected: all selected tests PASS with zero failures.

- [ ] **Step 2: rebuild browser WASM**

Run from repository root:

```bash
wasm-pack build --target web --out-dir pkg
```

Expected: exit 0 and regenerated `pkg/rhwp.js` exposes
`replaceTextInCellDeferredPagination`.

- [ ] **Step 3: Studio full test and production build**

Run:

```bash
cd rhwp-studio
npm test
npm run build
```

Expected: all Studio/Editor tests PASS and Vite production build exits 0.

- [ ] **Step 4: live functional IME proof**

Use 실제 브라우저 브라우저 자동화 on the same 43-page contract and exact depth-1 cell:

```text
section=0
parentPara=6
control=0
cell=0
cellPara=1
charOffset=246
```

Install reversible instance wrappers around:

```text
replaceTextInCellDeferredPagination
deleteTextInCellByPath
insertTextInCellDeferredPagination
flushDeferredPagination
getCursorRectByPathNear
```

Dispatch `compositionstart`, then composition inputs `ㅎ`, `하`, and
`compositionend`. Assert:

```text
ㅎ → 하 update:
  replaceTextInCellDeferredPagination = 1 call
  deleteTextInCellByPath = 0 calls
  insertTextInCellDeferredPagination = 0 calls
  synchronous flushDeferredPagination = 0 calls for stable flow
compositionend:
  undo stack gains exactly 1 entry
document text contains exactly 하 at the anchor
caret offset = anchor + 1
```

- [ ] **Step 5: rapid 20-update performance proof**

From the same anchor, run 20 reversible composition replacements, alternating equal-flow Korean
composition strings. Record synchronous input duration for every update and compute:

```text
median < 8.0ms
p95 < 16.0ms
deleteTextInCellByPath total calls = 0
replaceTextInCellDeferredPagination total calls = 20
```

If a selected update truly changes `cellFlowChanged`, exclude that boundary sample from the stable
latency percentile and separately verify that it performs exactly one pre-caret pagination flush.

- [ ] **Step 6: restore the shared browser state**

Complete composition, call undo exactly once through the real history path, clear only the probe-created
redo entry, restore wrapped methods, and move the cursor back to offset 246. Verify:

```text
cell paragraph length = 260
text[236..256] = "급 계약조건을 제시하기 위해 마련되었"
undo stack size = baseline
redo stack size = baseline
page count = 43
```

- [ ] **Step 7: final scope audit**

Run:

```bash
git status --short
git log -4 --oneline
git diff HEAD~3 -- \
  src/document_core/commands/text_editing.rs \
  src/wasm_api.rs \
  src/wasm_api/tests.rs \
  rhwp-studio/src/core/wasm-bridge.ts \
  rhwp-studio/src/engine/command.ts \
  rhwp-studio/src/engine/input-handler-text.ts \
  rhwp-studio/tests/cell-flow-boundary.test.ts \
  rhwp-studio/tests/input-edit-invalidation.test.ts
```

Expected: only approved V1 code/test hunks appear across the three implementation commits; unrelated
dirty toolbar, zoom, accessibility, and Subsecond files remain unstaged and unchanged by this work.
