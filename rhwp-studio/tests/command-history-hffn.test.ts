import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2337] 머리말/꼬리말·각주 편집 히스토리 기록 소스 가드.
//
// 미기록이던 HF/FN 편집을 executeOperation 경유로 기록해 본문 스냅샷 undo 의 무언
// 데이터 손실을 막는다. node --test 는 strip-only TS 라 engine 클래스를 실행할 수
// 없어(이 저장소 undo 테스트 관례), 소스 배선을 정적으로 검증한다. 행위 증명(브라우저
// 데이터손실 재현→차단, undo/redo 왕복)은 PR 검증 섹션에서 별도로 수행한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

const commandSrc = source('src/engine/command.ts');
const textSrc = source('src/engine/input-handler-text.ts');
const keyboardSrc = source('src/engine/input-handler-keyboard.ts');
const inputHandlerSrc = source('src/engine/input-handler.ts');
const historySrc = source('src/engine/history.ts');

/** `export class NAME ...` 부터 다음 `export class` 전까지 클래스 본문을 추출. */
function classBlock(src: string, name: string): string {
  const start = src.indexOf(`export class ${name}`);
  assert.notEqual(start, -1, `${name} 클래스 not found`);
  const rel = src.slice(start + 1).indexOf('\nexport class ');
  return rel === -1 ? src.slice(start) : src.slice(start, start + 1 + rel);
}

/** 클래스 본문에서 execute()/undo() 각 메서드 블록을 분리. */
function executeUndo(block: string): { execute: string; undo: string } {
  const eIdx = block.indexOf('execute(wasm: WasmBridge): DocumentPosition {');
  const uIdx = block.indexOf('undo(wasm: WasmBridge): DocumentPosition {');
  assert.ok(eIdx !== -1 && uIdx !== -1, 'execute/undo 시그니처 not found');
  // execute 가 undo 보다 앞에 오므로 undo 시작으로 execute 를 자른다.
  return {
    execute: block.slice(eIdx, uIdx),
    undo: block.slice(uIdx),
  };
}

// ── (1) 역연산 쌍: execute 와 undo 가 서로 반대 WASM 을 호출해야 한다 ──────────
// 쌍이 어긋나면(예: undo 가 execute 와 같은 방향) undo 가 문서를 되돌리지 못하고
// 오히려 오염시킨다. 이 저장소에서 가장 치명적인 회귀이므로 정면으로 핀한다.

const INVERSE_PAIRS: Array<{ cls: string; fwd: string; inv: string }> = [
  { cls: 'InsertTextInHeaderFooterCommand', fwd: 'insertTextInHeaderFooter', inv: 'deleteTextInHeaderFooter' },
  { cls: 'DeleteTextInHeaderFooterCommand', fwd: 'deleteTextInHeaderFooter', inv: 'insertTextInHeaderFooter' },
  { cls: 'SplitParagraphInHeaderFooterCommand', fwd: 'splitParagraphInHeaderFooter', inv: 'mergeParagraphInHeaderFooter' },
  { cls: 'MergeParagraphInHeaderFooterCommand', fwd: 'mergeParagraphInHeaderFooter', inv: 'splitParagraphInHeaderFooter' },
  { cls: 'InsertTextInFootnoteCommand', fwd: 'insertTextInFootnote', inv: 'deleteTextInFootnote' },
  { cls: 'DeleteTextInFootnoteCommand', fwd: 'deleteTextInFootnote', inv: 'insertTextInFootnote' },
  { cls: 'SplitParagraphInFootnoteCommand', fwd: 'splitParagraphInFootnote', inv: 'mergeParagraphInFootnote' },
  { cls: 'MergeParagraphInFootnoteCommand', fwd: 'mergeParagraphInFootnote', inv: 'splitParagraphInFootnote' },
];

for (const { cls, fwd, inv } of INVERSE_PAIRS) {
  test(`${cls}: execute=${fwd} / undo=${inv} (역연산 쌍 정합)`, () => {
    const { execute, undo } = executeUndo(classBlock(commandSrc, cls));
    assert.match(execute, new RegExp(`wasm\\.${fwd}\\(`), `execute 는 ${fwd} 를 호출해야 함`);
    assert.doesNotMatch(execute, new RegExp(`wasm\\.${inv}\\(`), `execute 에 역연산 ${inv} 가 섞이면 안 됨`);
    assert.match(undo, new RegExp(`wasm\\.${inv}\\(`), `undo 는 ${inv} 를 호출해야 함`);
    assert.doesNotMatch(undo, new RegExp(`wasm\\.${fwd}\\(`), `undo 에 정연산 ${fwd} 가 섞이면 안 됨`);
  });
}

test('모든 HF/FN 커맨드는 editContext() 로 커서 컨텍스트를 노출한다', () => {
  for (const { cls } of INVERSE_PAIRS) {
    const block = classBlock(commandSrc, cls);
    assert.match(block, /editContext\(\): EditContext \{ return this\.lastContext; \}/,
      `${cls} 는 editContext() 를 구현해야 한다(InputHandler 의 커서 모드 복원용)`);
  }
});

// ── (2) IME 게이트: compositionEnd 가 HF/FN 을 기록해야 한다 ──────────────────

test('onCompositionEnd 게이트가 HF/FN 을 기록에서 제외하지 않는다(데이터 손실 회귀 차단)', () => {
  // 회귀 전 코드: `if (anchor && finalLength > 0 && !isInHeaderFooter() && !isInFootnote())`
  // 로 HF/FN 을 배제해 IME 조합이 미기록 → 무언 파괴. 그 배제 패턴이 되살아나면 실패.
  assert.doesNotMatch(
    textSrc,
    /finalLength > 0 && !this\.cursor\.isInHeaderFooter\(\) && !this\.cursor\.isInFootnote\(\)/,
    'compositionEnd 게이트가 HF/FN 을 다시 배제함 → IME 편집 미기록(데이터 손실)',
  );
  // HF/FN 조합 결과를 실제로 record 한다.
  assert.match(textSrc, /new InsertTextInHeaderFooterCommand\(target,[^)]*composed\)/,
    'compositionEnd 에서 HF 조합 텍스트를 기록해야 함');
  assert.match(textSrc, /new InsertTextInFootnoteCommand\(target,[^)]*composed\)/,
    'compositionEnd 에서 FN 조합 텍스트를 기록해야 함');
});

// ── (3) 라우팅: HF/FN 편집 사이트가 executeOperation record 로 기록한다 ────────

test('HF/FN 삭제 사이트는 WASM 반환의 deletedText 를 역연산 재삽입용으로 확보한다', () => {
  // 삭제된 텍스트 없이는 undo 재삽입이 불가능하므로 반드시 확보해야 한다.
  assert.match(textSrc, /deletedText \?\? ''/, 'HF 삭제가 deletedText 를 커맨드에 전달해야 함');
  assert.match(keyboardSrc, /deletedText \?\? ''/, 'FN 삭제가 deletedText 를 커맨드에 전달해야 함');
  assert.match(textSrc, /new DeleteTextInHeaderFooterCommand\(/, 'HF 삭제 record');
  assert.match(keyboardSrc, /new DeleteTextInFootnoteCommand\(/, 'FN 삭제 record');
  // FN forward-Delete 는 문단 끝에서 clamp 로 실삭제 0 → 유령 undo 엔트리를 막기 위해
  // 실삭제(res.deletedText 비어있지 않음)일 때만 기록해야 한다.
  assert.match(keyboardSrc, /if \(res\.deletedText\)\s*\{[\s\S]*?new DeleteTextInFootnoteCommand\(/,
    'FN Delete 기록은 실삭제 시에만(유령 엔트리 방지) — if (res.deletedText) 가드 필요');
});

test('HF/FN 편집 사이트가 kind:record 로 히스토리에 기록한다', () => {
  // 최소 개수 가드 — 이관 사이트가 조용히 사라지면(=미기록 복귀) 실패.
  const hfCmds = (textSrc.match(/new (Insert|Delete|Merge)\w*InHeaderFooterCommand\(/g) ?? []).length
    + (keyboardSrc.match(/new (Insert|Split|Merge|Delete)\w*InHeaderFooterCommand\(/g) ?? []).length;
  const fnCmds = (textSrc.match(/new (Insert|Delete|Merge)\w*InFootnoteCommand\(/g) ?? []).length
    + (keyboardSrc.match(/new (Insert|Split|Merge|Delete)\w*InFootnoteCommand\(/g) ?? []).length;
  assert.ok(hfCmds >= 5, `HF 편집 커맨드 사이트가 부족: ${hfCmds} (<5) — 라우팅 누락?`);
  assert.ok(fnCmds >= 4, `FN 편집 커맨드 사이트가 부족: ${fnCmds} (<4) — 라우팅 누락?`);
});

// ── (4) undo/redo 커서 복원: 본문 moveTo 를 HF/FN 에서 건너뛴다 ────────────────

test('handleUndo/handleRedo 는 restoreEditContextAfterHistory 로 위임한다(HF/FN 커서 복원)', () => {
  // 회귀 전: handleUndo/Redo 가 곧장 cursor.moveTo(newPos) 만 호출 → HF/FN 커서 깨짐.
  const undoBlock = inputHandlerSrc.slice(
    inputHandlerSrc.indexOf('private handleUndo()'),
    inputHandlerSrc.indexOf('private handleRedo()'),
  );
  const redoBlock = inputHandlerSrc.slice(
    inputHandlerSrc.indexOf('private handleRedo()'),
    inputHandlerSrc.indexOf('private restoreEditContextAfterHistory'),
  );
  assert.match(undoBlock, /restoreEditContextAfterHistory\(this\.history\.peekRedoTop\(\)/,
    'handleUndo 는 방금 되돌린 커맨드(redoStack top)로 컨텍스트 복원해야 함');
  assert.match(redoBlock, /restoreEditContextAfterHistory\(this\.history\.peekUndoTop\(\)/,
    'handleRedo 는 방금 다시 실행한 커맨드(undoStack top)로 복원해야 함');
  assert.doesNotMatch(undoBlock, /this\.cursor\.moveTo\(newPos\)/, 'handleUndo 의 직접 moveTo 는 제거돼야 함');
  assert.doesNotMatch(redoBlock, /this\.cursor\.moveTo\(newPos\)/, 'handleRedo 의 직접 moveTo 는 제거돼야 함');
});

test('restoreEditContextAfterHistory 는 HF/FN 모드로 복원하고 본문 커맨드는 moveTo 한다', () => {
  const block = inputHandlerSrc.slice(inputHandlerSrc.indexOf('private restoreEditContextAfterHistory'));
  const method = block.slice(0, block.search(/\n {2}(?:public |private |protected )?[a-zA-Z][\w]*\s*\(/) + 1 || block.length);
  // HF/FN 컨텍스트면 모드 진입 + 오프셋 복원.
  assert.match(block, /ctx\?\.mode === 'headerFooter'/, 'HF 컨텍스트 분기');
  assert.match(block, /ctx\?\.mode === 'footnote'/, 'FN 컨텍스트 분기');
  assert.match(block, /enterHeaderFooterMode\(/, 'HF 모드 진입');
  assert.match(block, /enterFootnoteMode\(/, 'FN 모드 진입');
  assert.match(block, /setHfCursorPosition\(/, 'HF 커서 오프셋 복원');
  assert.match(block, /setFnCursorPosition\(/, 'FN 커서 오프셋 복원');
  // 본문 커맨드(컨텍스트 없음)면 HF/FN 을 빠져나오고 본문 moveTo.
  assert.match(block, /this\.cursor\.moveTo\(bodyPos\)/, '본문 커맨드는 bodyPos 로 moveTo');
  void method;
});

// ── (5) history 접근자: undo/redo 직후 커맨드 조회 ────────────────────────────

test('history 는 peekUndoTop/peekRedoTop 으로 방금 이동한 커맨드를 노출한다', () => {
  assert.match(historySrc, /peekUndoTop\(\): EditCommand \| null \{ return this\.undoStack\[this\.undoStack\.length - 1\] \?\? null; \}/);
  assert.match(historySrc, /peekRedoTop\(\): EditCommand \| null \{ return this\.redoStack\[this\.redoStack\.length - 1\] \?\? null; \}/);
});

// ── (6) [리뷰 반영] Unicode 단위 · form mode 기록 ────────────────────────────

test('HF/FN 삭제 count 는 char(코드포인트) 단위여야 한다 — astral 문자 over-delete 방지', () => {
  // Rust delete_text_at 은 char 단위인데 JS String.length(UTF-16)를 넘기면 '😀' 같은
  // astral 문자 undo/redo 때 인접 문자를 잃는다(P1). charCount([...s].length)로 계산해야 한다.
  assert.match(commandSrc, /function charCount\(s: string\): number \{\s*return \[\.\.\.s\]\.length;/,
    'charCount(코드포인트 수) 헬퍼 필요');
  // HF/FN 삭제 WASM 호출의 count 인자가 .length 가 아니라 charCount 여야 한다.
  for (const cls of ['DeleteTextInHeaderFooterCommand', 'DeleteTextInFootnoteCommand',
    'InsertTextInHeaderFooterCommand', 'InsertTextInFootnoteCommand']) {
    const block = classBlock(commandSrc, cls);
    assert.doesNotMatch(block, /deleteTextIn(HeaderFooter|Footnote)\([^)]*\.(text|deletedText)\.length\)/,
      `${cls}: 삭제 count 에 .length(UTF-16) 사용 금지 — charCount 사용`);
  }
});

test('form mode 에서 kind:record 는 항상 허용된다 — 이미 적용된 편집의 무언 손실 방지', () => {
  // form mode 에서 HF/FN record 를 드롭하면 뮤테이션이 undo 불가한 미기록 편집으로 남는다(P1).
  const block = inputHandlerSrc.slice(
    inputHandlerSrc.indexOf('private isOperationAllowedInEditMode'),
    inputHandlerSrc.indexOf('private isOperationAllowedInEditMode') + 900,
  );
  assert.match(block, /if \(desc\.kind === 'record'\) return true;/,
    "isOperationAllowedInEditMode 는 kind:'record' 를 항상 허용해야 함");
  // record 허용이 form 게이트(editMode!=='form' → true) 이후에 와야 실효가 있다.
  const idxForm = block.indexOf("this.editMode !== 'form'");
  const idxRecord = block.indexOf("desc.kind === 'record'");
  assert.ok(idxForm !== -1 && idxRecord !== -1 && idxForm < idxRecord,
    'record 허용은 editMode form 체크 뒤에 위치');
});
