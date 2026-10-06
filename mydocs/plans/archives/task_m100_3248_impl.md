# rhwp-studio 본문 입력 지연 개선 Implementation Plan

**Goal:** 본문 일반 입력과 한글 IME 교체를 즉시 현재 페이지에 표시하면서, 안정적인 편집의 전체 문서 pagination만 120ms 유휴 시점에 한 번으로 병합한다.

**Architecture:** Rust `DocumentCore`에 delete+insert를 한 번의 reflow로 처리하는 본문 local replace primitive를 추가한다. 이 primitive는 최종 문단 흐름이 유지되면 `render_normalized`와 page-tree만 즉시 일치시키고 전체 pagination 필요 상태를 반환하며, 흐름이 바뀌면 반환 전에 pagination을 끝낸다. Studio는 mutation effect를 cursor 조회 전에 소비하고, 안정 입력은 현재 페이지만 다음 animation frame에 다시 그리며 IME 중간 교체는 단일 WASM 호출로 보낸다.

**Tech Stack:** Rust, `wasm-bindgen`, TypeScript, Node test runner, Vite, Canvas2D, browser runtime

## Global Constraints

- 입력 글자, IME 중간 글자, caret, 현재 문단 reflow, 현재 페이지 paint는 120ms timer를 기다리지 않는다.
- 지원 범위는 같은 본문 문단의 줄바꿈·탭 없는 최대 8자 insert/delete/replace다.
- `flowChanged = true`, 지원 범위 밖 편집, API 누락·결과 불명은 즉시 pagination과 full refresh로 복구한다.
- undo/redo, navigation, Enter/Tab, 구조·서식 편집, save/export, blur, deactivate 전에 pending pagination을 flush한다.
- 현재 worktree의 toolbar, zoom, 접근성, Subsecond 변경을 수정하거나 되돌리지 않는다.
- `src/wasm_api.rs`와 `rhwp-studio/src/engine/input-handler.ts`에는 기존 미커밋 변경이 있으므로 커밋 시 `git add -p`로 이 작업 hunk만 stage한다.
- 원격 push와 PR 생성은 별도 사용자 승인 전에는 수행하지 않는다.
- 긴 PR CI 성격의 전체 `cargo test --verbose`/clippy는 별도 승인 없이는 실행하지 않는다. 아래의 focused Rust tests, Studio tests, WASM build, production build, live browser 검증까지만 기본 범위로 한다.

---

## Task 1: Rust local body replace의 실패 테스트 작성

**Files:**

- Modify: `src/wasm_api/tests.rs`
- Reference: `src/document_core/commands/text_editing.rs:439`
- Reference: `src/document_core/queries/rendering.rs:2814`
- Reference: `src/document_core/queries/rendering.rs:3801`

- [ ] **Step 1: 안정 insert/delete/IME replace 계약 테스트 추가**

`src/wasm_api/tests.rs` 끝의 #2214 cache-coherence 테스트 근처에 다음 세 테스트를 추가한다.

```rust
#[test]
fn local_body_replace_exposes_stable_edit_before_full_pagination() {
    let mut doc = HwpDocument::create_empty();
    let raw = doc
        .replace_body_text_local_native(0, 0, 0, 0, "가")
        .expect("stable local insert");
    let result: Value = serde_json::from_str(&raw).expect("local result json");

    assert_eq!(result["charOffset"].as_u64(), Some(1));
    assert_eq!(result["documentPaginationPending"].as_bool(), Some(true));
    assert_eq!(result["flowChanged"].as_bool(), Some(false));
    assert_eq!(
        doc.get_text_range_native(0, 0, 0, 1).expect("immediate text"),
        "가"
    );
}

#[test]
fn local_body_replace_applies_ime_replacement_as_one_final_state() {
    let mut doc = HwpDocument::create_empty();
    doc.replace_body_text_local_native(0, 0, 0, 0, "ㅎ")
        .expect("initial composition");
    let raw = doc
        .replace_body_text_local_native(0, 0, 0, 1, "하")
        .expect("composition replacement");
    let result: Value = serde_json::from_str(&raw).expect("replace result json");

    assert_eq!(result["charOffset"].as_u64(), Some(1));
    assert_eq!(
        doc.get_text_range_native(0, 0, 0, 1).expect("final composition"),
        "하"
    );
}
```

세 번째 테스트는 빈 문서에 한 글자씩 local append를 반복하여 처음 `flowChanged`가 발생한 시점을 찾고 다음을 검증한다.

```rust
assert_eq!(result["documentPaginationPending"].as_bool(), Some(false));
assert_eq!(result["flowChanged"].as_bool(), Some(true));
assert_eq!(doc.page_count(), doc.pagination.iter().map(|s| s.pages.len()).sum::<usize>() as u32);
```

반복 상한은 512자로 두고, 임계가 없으면 테스트를 실패시켜 fixture/font 변화가 조용히 계약을 무력화하지 않게 한다.

- [ ] **Step 2: warm page tree의 즉시 텍스트 노출 assertion 추가**

첫 테스트에서 편집 전에 `build_page_render_tree(0)`을 호출해 cache를 warm한다. 편집 후 flush 없이 다시 tree를 만들고, `RenderNodeType::TextRun`을 순회해 `"가"`가 나타나는지 검증한다. 이 assertion이 `render_normalized` coherence 누락을 직접 잡아야 한다.

- [ ] **Step 3: focused test가 RED인지 확인**

Run:

```bash
cargo test --lib local_body_replace
```

Expected: `replace_body_text_local_native`가 아직 없어서 compile failure.

- [ ] **Step 4: 테스트 hunk만 stage 준비**

Run:

```bash
git diff -- src/wasm_api/tests.rs
```

Expected: 위 세 테스트와 helper만 표시되고 기존 사용자 변경은 없다.

---

## Task 2: Rust primitive와 WASM binding 구현

**Files:**

- Modify: `src/document_core/commands/text_editing.rs`
- Modify: `src/document_core/queries/rendering.rs`
- Modify: `src/wasm_api.rs`
- Test: `src/wasm_api/tests.rs`

- [ ] **Step 1: 본문 flow signature와 normalized coherence helper 추가**

`text_editing.rs`의 `relative_paragraph_flow_advance` 옆에 본문 판정 helper를 둔다.

```rust
fn body_paragraph_flow_signature(paragraph: &Paragraph) -> (usize, Option<i64>) {
    (
        paragraph.line_segs.len(),
        relative_paragraph_flow_advance(paragraph),
    )
}
```

`rendering.rs`의 cell coherence helper 앞에 다음 API를 추가한다.

```rust
pub(crate) fn refresh_render_normalized_body_paragraph_after_edit(
    &mut self,
    section_idx: usize,
    para_idx: usize,
) {
    let source_para = self.document.sections[section_idx].paragraphs[para_idx].clone();
    let source_composed = self.composed[section_idx][para_idx].clone();
    let Some(Some((paragraphs, composed))) = self.render_normalized.get_mut(section_idx) else {
        return;
    };
    if let Some(target) = paragraphs.get_mut(para_idx) {
        *target = source_para;
    }
    if let Some(target) = composed.get_mut(para_idx) {
        *target = source_composed;
    }
}
```

`recompose_paragraph`가 page-tree cache를 이미 무효화하므로 이 helper는 복사본 coherence만 책임진다.

- [ ] **Step 2: 한 번만 mutate/reflow하는 native replace 구현**

`insert_text_native` 앞에 아래 signature를 추가한다.

```rust
pub fn replace_body_text_local_native(
    &mut self,
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    delete_count: usize,
    text: &str,
) -> Result<String, HwpError>
```

구현 순서는 다음으로 고정한다.

1. `insert_text_native`와 같은 section/paragraph index 검증
2. `delete_count <= 8`, `text.chars().count() <= 8`, `text`에 `\r`, `\n`, `\t`가 없는지 검증
3. 편집 전 `body_paragraph_flow_signature`, column index, `paragraph_flow_end` 캡처
4. `raw_stream = None`
5. `delete_count > 0`이면 `Paragraph::delete_text_at` 한 번
6. `text`가 비어 있지 않으면 기존 insert의 inactive field start/end 보존 로직과 `Paragraph::insert_text_at` 적용
7. `reflow_paragraph` → `recalculate_section_vpos` → `recompose_paragraph`를 각각 한 번
8. 최종 flow signature 비교
9. 흐름이 유지되면 `refresh_render_normalized_body_paragraph_after_edit`만 호출
10. 흐름이 바뀌면 `paginate()`를 즉시 호출하고 기존 insert/delete와 같은 다단 수렴 루프를 최대 2회 수행
11. caret DocProperties와 DocInfo raw stream을 최종 offset으로 한 번 갱신
12. delete/insert event를 실제 수행된 부분만 기록

결과 JSON은 정확히 다음 필드를 반환한다.

```rust
let new_offset = char_offset + text.chars().count();
let pagination_pending = !flow_changed;
Ok(super::super::helpers::json_ok_with(&format!(
    "\"charOffset\":{},\"documentPaginationPending\":{},\"flowChanged\":{}",
    new_offset, pagination_pending, flow_changed
)))
```

`flowChanged = true`인 결과는 반환 시점에 pagination이 완료되어 있으므로 `documentPaginationPending`은 반드시 `false`다.

- [ ] **Step 3: wasm-bindgen API 노출**

`src/wasm_api.rs`의 `insert_text`/`delete_text` binding 옆에 추가한다.

```rust
#[wasm_bindgen(js_name = replaceBodyTextLocal)]
pub fn replace_body_text_local(
    &mut self,
    section_idx: u32,
    para_idx: u32,
    char_offset: u32,
    delete_count: u32,
    text: &str,
) -> Result<String, JsValue> {
    self.replace_body_text_local_native(
        section_idx as usize,
        para_idx as usize,
        char_offset as usize,
        delete_count as usize,
        text,
    )
    .map_err(|e| e.into())
}
```

- [ ] **Step 4: focused Rust tests GREEN**

Run:

```bash
cargo test --lib local_body_replace
cargo test --lib issue2214_scoped_cache_coherence_preserves_transient_pagination
```

Expected: 모두 PASS. #2214 cell local path도 그대로 유지된다.

- [ ] **Step 5: Rust 변경만 커밋**

`src/wasm_api.rs`에는 기존 사용자 변경이 있으므로 전체 파일을 stage하지 않는다.

```bash
git add src/document_core/commands/text_editing.rs \
        src/document_core/queries/rendering.rs \
        src/wasm_api/tests.rs
git add -p src/wasm_api.rs
git diff --cached --check
git diff --cached --stat
git commit -m "perf: add immediate local body text replace"
```

Expected: staged diff에 Subsecond/기존 WASM 변경이 없다.

---

## Task 3: WasmBridge 결과 검증과 command mutation effect 일반화

**Files:**

- Create: `rhwp-studio/src/core/local-text-replace-result.ts`
- Modify: `rhwp-studio/src/core/wasm-bridge.ts`
- Modify: `rhwp-studio/src/core/mutation-method-registry.ts`
- Modify: `rhwp-studio/src/engine/command.ts`
- Modify: `rhwp-studio/src/engine/history.ts`
- Modify: `rhwp-studio/tests/cell-flow-boundary.test.ts`
- Create: `rhwp-studio/tests/local-text-replace-result.test.ts`
- Modify: `rhwp-studio/tests/mutation-routing-guard.test.ts`

- [ ] **Step 1: bridge parser와 fallback의 실패 테스트 작성**

`local-text-replace-result.test.ts`에서 다음을 먼저 작성한다.

```ts
test('stable local result는 pending page-local effect로 정규화된다', () => {
  assert.deepEqual(parseLocalBodyTextReplaceResult(
    '{"ok":true,"charOffset":4,"documentPaginationPending":true,"flowChanged":false}',
  ), {
    ok: true,
    charOffset: 4,
    documentPaginationPending: true,
    flowChanged: false,
  });
});

test('모순되거나 불완전한 결과는 거부한다', () => {
  assert.throws(() => parseLocalBodyTextReplaceResult(
    '{"ok":true,"charOffset":4,"documentPaginationPending":true,"flowChanged":true}',
  ));
  assert.throws(() => parseLocalBodyTextReplaceResult('{"ok":true}'));
});
```

Run:

```bash
node --test tests/local-text-replace-result.test.ts
```

Expected: module not found로 RED.

- [ ] **Step 2: 순수 parser 구현**

`src/core/local-text-replace-result.ts`를 다음 계약으로 만든다.

```ts
export interface LocalBodyTextReplaceResult {
  ok: true;
  charOffset: number;
  documentPaginationPending: boolean;
  flowChanged: boolean;
}

export function parseLocalBodyTextReplaceResult(
  raw: string,
): LocalBodyTextReplaceResult {
  const parsed = JSON.parse(raw) as Partial<LocalBodyTextReplaceResult>;
  if (
    parsed.ok !== true ||
    !Number.isInteger(parsed.charOffset) ||
    typeof parsed.documentPaginationPending !== 'boolean' ||
    typeof parsed.flowChanged !== 'boolean' ||
    (parsed.flowChanged && parsed.documentPaginationPending)
  ) {
    throw new Error('잘못된 local body text replace 결과');
  }
  return parsed as LocalBodyTextReplaceResult;
}
```

- [ ] **Step 3: WasmBridge API와 보수적 fallback 구현**

`WasmBridge.replaceBodyTextLocal`은 native method가 있으면 parser를 사용한다. 없으면 기존 delete/insert를 즉시 실행하고 다음 결과를 반환한다.

```ts
return {
  ok: true,
  charOffset: charOffset + [...text].length,
  documentPaginationPending: false,
  flowChanged: true,
};
```

fallback의 `flowChanged: true`는 page-local paint를 금지하고 기존 full refresh를 선택하기 위한 보수적 신호다.

`src/core/mutation-method-registry.ts`의 `MUTATING_METHODS`에는
`replaceBodyTextLocal`을 `insertText`/`deleteText`와 같은 text mutation 군에 추가한다.
`mutation-routing-guard.test.ts`의 ledger는 실제 스캔 결과가 달라질 때만 그 정확한
증가분으로 갱신한다. command helper 안에서만 새 bridge method를 호출하면
`input-handler*`의 직접 WASM 호출 수는 늘지 않아야 한다.

- [ ] **Step 4: mutation effect 이름을 body/cell 공통 의미로 변경**

`TextMutationEffects`를 다음으로 바꾸고 command/history/테스트 사용처를 함께 갱신한다.

```ts
export interface TextMutationEffects {
  readonly documentPaginationPending: boolean;
  readonly flowChanged: boolean;
  readonly paginationCompleted: boolean;
}
```

cell 결과는 다음처럼 매핑한다.

```ts
return {
  documentPaginationPending: result.paginationDeferred,
  flowChanged: result.cellFlowChanged,
  paginationCompleted: !result.paginationDeferred,
};
```

본문용 helper를 추가한다.

```ts
export function replaceBodyTextWithMutationEffects(
  wasm: WasmBridge,
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): TextMutationEffects {
  const result = wasm.replaceBodyTextLocal(
    pos.sectionIndex,
    pos.paragraphIndex,
    pos.charOffset,
    deleteCount,
    text,
  );
  return {
    documentPaginationPending: result.documentPaginationPending,
    flowChanged: result.flowChanged,
    paginationCompleted: !result.documentPaginationPending,
  };
}
```

- [ ] **Step 5: Insert/Delete command의 본문 execute만 local path로 전환**

`InsertTextCommand.execute`의 본문은 `replaceBodyTextWithMutationEffects(..., 0, text)`를 사용한다. `DeleteTextCommand.execute`의 본문은 `replaceBodyTextWithMutationEffects(..., count, '')`를 사용한다. cell, nested cell, header/footer, footnote 및 undo는 기존 즉시 API를 유지한다.

`canUseLocalBodyTextReplace`는 body position, 최대 8자, 줄바꿈·탭 없음 조건을 한곳에서 검사한다. 조건을 벗어나면 기존 immediate insert/delete와 `IMMEDIATE_TEXT_MUTATION_EFFECTS`를 반환한다.

- [ ] **Step 6: command behavior 테스트 갱신 및 실행**

`FakeWasm`에 `replaceBodyTextLocal`을 추가하고 다음을 검증한다.

- 안정 본문 insert는 local call 1회, pending true
- 안정 본문 delete는 local call 1회, pending true
- `flowChanged` 본문 결과는 `paginationCompleted: true`
- cell depth 1/2 기존 routing 불변
- undo는 기존 immediate delete/insert 사용
- history merge/redo가 매 실행 결과를 다시 소비

Run:

```bash
node --test tests/local-text-replace-result.test.ts tests/cell-flow-boundary.test.ts
```

Expected: PASS.

- [ ] **Step 7: bridge/command 변경 커밋**

```bash
git add rhwp-studio/src/core/local-text-replace-result.ts \
        rhwp-studio/src/core/wasm-bridge.ts \
        rhwp-studio/src/core/mutation-method-registry.ts \
        rhwp-studio/src/engine/command.ts \
        rhwp-studio/src/engine/history.ts \
        rhwp-studio/tests/local-text-replace-result.test.ts \
        rhwp-studio/tests/cell-flow-boundary.test.ts
git add -p rhwp-studio/tests/mutation-routing-guard.test.ts
git diff --cached --check
git commit -m "perf: route stable body edits through local replace"
```

---

## Task 4: IME를 단일 replace 호출로 전환하고 현재 페이지를 즉시 그리기

**Files:**

- Modify: `rhwp-studio/src/engine/input-handler-text.ts`
- Modify: `rhwp-studio/src/engine/input-handler.ts`
- Modify: `rhwp-studio/src/engine/input-edit-invalidation.ts`
- Modify: `rhwp-studio/tests/input-edit-invalidation.test.ts`
- Modify: `rhwp-studio/tests/ime-composition-char-count.test.ts`

- [ ] **Step 1: IME 단일 호출과 본문 page-local 판정 실패 테스트 작성**

`input-edit-invalidation.test.ts`를 behavior 중심으로 갱신한다.

```ts
test('같은 본문 문단의 짧은 insert/delete도 page-local 대상이다', () => {
  const body = { sectionIndex: 0, paragraphIndex: 2, charOffset: 3 };
  assert.equal(
    isPageLocalTextEditCommand(
      'insertText',
      body,
      { ...body, charOffset: 4 },
      { insertedText: '가', beforePageIndex: 0, afterPageIndex: 0 },
    ),
    true,
  );
});
```

source contract에는 IME block이 `replaceTextAtRaw(anchor, this.compositionLength, text)`를 정확히 한 번 호출하고, 그 block에 `deleteTextAt(anchor`와 `insertTextAtRaw(anchor`가 없음을 검증한다. iOS fallback도 같은 계약을 적용한다.

Run:

```bash
node --test tests/input-edit-invalidation.test.ts tests/ime-composition-char-count.test.ts
```

Expected: RED.

- [ ] **Step 2: raw replace helper 추가**

`input-handler-text.ts`에 다음 함수를 추가한다.

```ts
export function replaceTextAtRaw(
  this: any,
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): TextMutationEffects
```

본문이고 local 조건을 만족하면 `replaceBodyTextWithMutationEffects`를 한 번 호출한다. 그 외 편집 영역은 기존 delete/insert 경로를 유지하되 `TextMutationEffectAccumulator`로 effect를 합친다.

`InputHandler` wrapper도 단일 effect를 accumulator에 넣는다.

```ts
private replaceTextAtRaw(
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): void {
  this.rawTextMutationEffects.add(
    _text.replaceTextAtRaw.call(this, pos, deleteCount, text),
  );
}
```

- [ ] **Step 3: IME와 iOS 중간 교체를 단일 호출로 변경**

composition block은 다음 순서를 갖는다.

```ts
this.resetRawTextMutationEffects();
this.replaceTextAtRaw(anchor, this.compositionLength, text);
this.compositionLength = charCount(text);
const boundaryHandled = this.consumeRawTextMutationBeforeCursor();
```

iOS `_iosLength` 경로도 동일하게 변경한다. cursor offset은 기존 Studio UTF-16 관례인 `anchor.charOffset + text.length`를 유지하고, delete count만 계속 `charCount(text)`를 사용한다.

- [ ] **Step 4: 본문 page-local invalidation 허용**

`isPageLocalTextEditCommand`에서 cell-only guard를 일반화한다.

- body↔body: 같은 section/paragraph만 허용
- cell↔cell: 기존 parent/control/cell/path 비교 유지
- body↔cell 전환: false
- page index가 달라지면 false
- 8자 초과, 줄바꿈, 탭: false

`prepareTextMutationBeforeCursor`는 일반화된 effect를 사용한다.

```ts
if (effects.paginationCompleted) {
  this.cancelDeferredPaginationFlush();
  this.deferredPaginationPending = false;
}
if (effects.flowChanged && effects.paginationCompleted) return true;
if (!effects.documentPaginationPending) return false;

this.deferredPaginationPending = true;
if (!effects.flowChanged) return false;
this.flushDeferredPaginationIfNeeded('text-flow-boundary', false);
return true;
```

안정 결과는 cursor lookup 전에 pending을 등록하지만, 같은 input 처리에서 `afterPageLocalEdit()`가 `document-page-invalidated`를 emit한다. CanvasView의 기존 rAF 경로가 현재 페이지만 다음 frame에 그리므로 timer는 paint 선행 조건이 아니다.

- [ ] **Step 5: focused Studio tests GREEN**

Run:

```bash
node --test tests/input-edit-invalidation.test.ts \
                 tests/ime-composition-char-count.test.ts \
                 tests/cell-flow-boundary.test.ts
```

Expected: PASS.

- [ ] **Step 6: IME/page-local 변경 커밋**

`input-handler.ts`는 기존 접근성 변경과 겹치므로 task hunk만 선택한다.

```bash
git add rhwp-studio/src/engine/input-handler-text.ts \
        rhwp-studio/src/engine/input-edit-invalidation.ts \
        rhwp-studio/tests/input-edit-invalidation.test.ts \
        rhwp-studio/tests/ime-composition-char-count.test.ts
git add -p rhwp-studio/src/engine/input-handler.ts
git diff --cached --check
git commit -m "perf: render IME composition through one local replace"
```

---

## Task 5: 120ms idle flush와 correctness boundary 구현

**Files:**

- Modify: `rhwp-studio/src/engine/input-handler.ts`
- Modify: `rhwp-studio/src/engine/input-handler-keyboard.ts`
- Modify: `rhwp-studio/src/engine/input-handler-text.ts`
- Modify: `rhwp-studio/tests/input-edit-invalidation.test.ts`
- Reference: `rhwp-studio/src/command/commands/file.ts:112`

- [ ] **Step 1: timer와 boundary의 실패 테스트 작성**

source contract 또는 작은 fake-timer harness로 다음을 고정한다.

- delay 상수는 `120`
- page-count 제한 상수와 `shouldAutoFlushDeferredPagination`은 제거
- 새 안정 입력마다 이전 timer 취소 후 새 timer 1개 예약
- `handleUndo`/`handleRedo`는 history mutation 전에 flush
- navigation/Enter/Tab은 cursor 이동 또는 구조 mutation 전에 flush
- IME pending navigation은 `processPendingNav` 시작 시 flush
- textarea blur는 flush
- `deactivate()`는 pending 상태를 버리기 전에 flush
- file save/export의 기존 `flushDeferredPaginationBeforeExplicitOutput`은 유지

Run:

```bash
node --test tests/input-edit-invalidation.test.ts
```

Expected: RED.

- [ ] **Step 2: idle timer를 120ms로 변경**

```ts
const DOCUMENT_PAGINATION_IDLE_FLUSH_DELAY_MS = 120;
```

`scheduleDeferredPaginationFlush`는 문서 page count와 무관하게 항상 timer를 예약한다. 연속 입력은 기존 timer를 취소하고 120ms를 다시 센다.

- [ ] **Step 3: undo/redo와 deactivate 경계 추가**

```ts
private handleUndo(): void {
  this.flushDeferredPaginationIfNeeded('before-undo', false);
  const newPos = this.history.undo(this.wasm);
  // existing restoration
}

private handleRedo(): void {
  this.flushDeferredPaginationIfNeeded('before-redo', false);
  const newPos = this.history.redo(this.wasm);
  // existing restoration
}
```

`deactivate()` 첫 부분은 `flushDeferredPaginationIfNeeded('before-deactivate', false)`를 호출한 뒤 active/session state를 정리한다.

- [ ] **Step 4: keyboard/navigation 경계 추가**

`onKeyDown`에서 IME 분기 뒤 일반 navigation을 처리하기 전에 Arrow/Home/End/PageUp/PageDown/Enter/Tab이면 `flushDeferredPaginationIfNeeded('before-navigation', false)`를 호출한다. IME가 key를 예약한 경우에는 `input-handler-text.ts`의 `processPendingNav` 첫 줄에서 같은 flush를 호출한다.

문서 구조 mutation을 수행하는 Enter/Tab은 flush 이후 기존 command를 실행한다. 단순 문자 입력/Backspace/Delete는 이 pre-flush 목록에 넣지 않는다.

- [ ] **Step 5: blur listener 수명주기 추가**

constructor에서 bound handler를 만든다.

```ts
this.onInputBlurBound = () => {
  this.flushDeferredPaginationIfNeeded('input-blur', false);
};
```

textarea에 listener를 등록하고 `dispose()`에서 같은 bound function을 제거한다. inline closure를 사용해 remove가 불가능해지는 형태는 피한다.

- [ ] **Step 6: focused boundary tests GREEN**

Run:

```bash
node --test tests/input-edit-invalidation.test.ts \
                 tests/cell-flow-boundary.test.ts \
                 tests/ime-composition-char-count.test.ts
```

Expected: PASS.

- [ ] **Step 7: timer/boundary 변경 커밋**

```bash
git add rhwp-studio/src/engine/input-handler-keyboard.ts \
        rhwp-studio/src/engine/input-handler-text.ts \
        rhwp-studio/tests/input-edit-invalidation.test.ts
git add -p rhwp-studio/src/engine/input-handler.ts
git diff --cached --check
git commit -m "perf: flush document pagination at idle boundaries"
```

---

## Task 6: WASM 재빌드와 정적 회귀 검증

**Files:**

- Generated: `pkg/`
- Verify: Rust library and `rhwp-studio`

- [ ] **Step 1: focused Rust tests 재실행**

Run from repository root:

```bash
cargo test --lib local_body_replace
cargo test --lib issue2214_scoped_cache_coherence_preserves_transient_pagination
```

Expected: PASS.

- [ ] **Step 2: WASM package 재빌드**

Run from repository root:

```bash
wasm-pack build --target web --out-dir pkg
```

Expected: success; generated JS/TS binding includes `replaceBodyTextLocal`.

- [ ] **Step 3: Studio focused tests와 full Node test suite**

Run from `rhwp-studio`:

```bash
node --test tests/local-text-replace-result.test.ts \
                 tests/cell-flow-boundary.test.ts \
                 tests/input-edit-invalidation.test.ts \
                 tests/ime-composition-char-count.test.ts
npm test
```

Expected: PASS. 기존 unrelated untracked tests도 full suite에 포함되므로 실패 시 먼저 이 작업과의 인과를 분리해 보고한다.

- [ ] **Step 4: production build**

Run:

```bash
npm run build
```

Expected: TypeScript와 Vite build PASS.

- [ ] **Step 5: generated binding과 task hunk만 stage/commit**

먼저 `git status --short`와 `git diff -- pkg`로 generated scope를 확인한다. 기존 unrelated generated 변경이 없을 때만 다음을 실행한다.

```bash
git add pkg
git diff --cached --check
git commit -m "build: refresh wasm bindings for local text replace"
```

`pkg`가 ignore되거나 build 결과가 tracked diff를 만들지 않으면 이 commit은 생략한다.

---

## Task 7: 실제 43쪽 문서에서 즉시 표시와 latency 재측정

**Files:**

- Verify live: `http://localhost:7701/`
- Update: `mydocs/plans/task_m100_3248.md` only if measured numbers need a final evidence section

- [ ] **Step 1: live page reload와 문서 상태 확인**

실제 브라우저 브라우저 자동화로 `/`를 reload하고 현재 문서가 43쪽인지, cursor가 본문인지 확인한다. UI interaction은 snapshot/ref를 우선 사용한다.

- [ ] **Step 2: non-destructive runtime instrumentation 설치**

브라우저에서 아래 호출을 감싼 뒤 각 duration/count를 기록한다.

- `replaceBodyTextLocal`
- `flushDeferredPagination`
- `document-page-invalidated`
- `document-changed`
- current page renderer
- input event 시작
- 첫 `requestAnimationFrame` paint

instrumentation은 측정 후 반드시 원래 function으로 복구한다.

- [ ] **Step 3: 일반 입력 즉시 표시 검증**

본문 cursor 위치에 임시 문자 `A` 한 글자를 입력하고 다음을 확인한다.

- input dispatch 중 `replaceBodyTextLocal` 1회
- input dispatch 중 `flushDeferredPagination` 0회
- input handler < 16ms
- current-page first frame < 50ms
- `document-page-invalidated`가 120ms idle flush보다 먼저 발생
- 120ms 유휴 뒤 `flushDeferredPagination` 정확히 1회

- [ ] **Step 4: 한글 IME 교체 즉시 표시 검증**

`ㅎ → 하` composition update를 실제 input/composition event 순서로 발생시키고 다음을 확인한다.

- 각 composition update당 `replaceBodyTextLocal` 1회
- 교체 update에 별도 body delete+insert WASM 호출 0회
- input handler < 16ms
- current-page first frame < 50ms
- `"하"`가 idle timer 전에 현재 page tree/canvas에 존재

- [ ] **Step 5: flow boundary와 명시 boundary 검증**

줄 끝 근처에서 wrap을 일으키는 문자를 임시 입력해, 해당 input에서 pagination과 full refresh가 paint 전에 완료되는지 확인한다. 이어 안정 입력 후 Arrow, save/export 진입, undo/redo 각각에서 pending pagination이 먼저 flush되는지 호출 순서로 검증한다.

- [ ] **Step 6: 모든 probe 편집 복구**

입력한 `A`, `ㅎ`, `하`, wrap probe를 제거하고 원래 cursor 위치와 문서 dirty 상태를 복구한다. runtime wrapper도 모두 해제한다. 제거 뒤 페이지 수와 원문 일부를 다시 읽어 probe 잔존이 없음을 확인한다.

- [ ] **Step 7: 최종 diff와 완료 기준 검토**

Run:

```bash
git status --short
git diff --check
git log --oneline -7
```

확인 항목:

- unrelated toolbar/zoom/accessibility/Subsecond 변경이 commit에 포함되지 않음
- 안정 입력/IME가 즉시 현재 페이지에 표시됨
- 전체 pagination은 stable input에서 120ms 유휴까지 병합됨
- flow boundary/save/undo/navigation correctness 유지
- focused tests, `npm test`, `npm run build`, live measurement 결과가 기록됨

---

## Final Handoff

- 작업 결과는 local commits와 검증 근거까지만 준비한다.
- remote push/PR은 사용자 승인 뒤 `devel` 대상으로 진행한다.
- 실제 측정이 목표를 못 맞추면 성공으로 선언하지 않고, 남은 duration을 `local replace`, cursor lookup, page render로 다시 분해해 다음 병목을 보고한다.
