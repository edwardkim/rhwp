import { registerHooks } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { dirname, resolve } from 'node:path';
import assert from 'node:assert/strict';

const src = resolve(dirname(fileURLToPath(import.meta.url)), '../../src');
registerHooks({ resolve(specifier, context, next) {
  if (specifier.startsWith('@/')) {
    return { url: pathToFileURL(resolve(src, specifier.slice(2) + '.ts')).href, shortCircuit: true };
  }
  if (specifier.startsWith('.') && !/\.[cm]?[jt]s$/.test(specifier)) {
    return { url: new URL(specifier + '.ts', context.parentURL).href, shortCircuit: true };
  }
  return next(specifier, context);
} });
const { onInput, replaceTextAtRaw } = await import(pathToFileURL(resolve(src, 'engine/input-handler-text.ts')));
const { CommandHistory } = await import(pathToFileURL(resolve(src, 'engine/history.ts')));

for (const mode of ['header', 'footer', 'footnote']) {
  for (const [value, requestedOffset] of [['가', 1], ['😀', 1], ['𠮷', 1], ['가😀𠮷', 1], ['가😀𠮷', 3]]) {
    let text = 'AB';
    let offset = requestedOffset;
    const insertedAt = Math.min(requestedOffset, text.length);
    const withInput = input => 'AB'.slice(0, insertedAt) + input + 'AB'.slice(insertedAt);
    const history = new CommandHistory();
    // A trailing inline control adds one logical caret position after the two text scalars.
    const insert = (...args) => {
      const chars = [...text];
      const start = Math.min(args.at(-2), chars.length);
      chars.splice(start, 0, ...args.at(-1));
      text = chars.join('');
      const result = { ok: true, charOffset: start + [...args.at(-1)].length };
      return mode === 'footnote' ? result : JSON.stringify(result);
    };
    const remove = (...args) => {
      const chars = [...text];
      chars.splice(args.at(-2), args.at(-1));
      text = chars.join('');
    };
    const wasm = { snapshotCapacity: () => 100,
      insertTextInHeaderFooter: insert, insertTextInFootnote: insert,
      deleteTextInHeaderFooter: remove, deleteTextInFootnote: remove };
    const cursor = {
      isInHeaderFooter: () => mode !== 'footnote', isInFootnote: () => mode === 'footnote',
      headerFooterMode: mode, hfSectionIdx: 0, hfApplyTo: 0, hfPreviewPage: 0, hfParaIdx: 0,
      fnSectionIdx: 0, fnParaIdx: 0, fnControlIdx: 0, fnFootnoteIndex: 0, fnPageNum: 0, fnInnerParaIdx: 0,
      get hfCharOffset() { return offset; }, get fnCharOffset() { return offset; },
      setHfCursorPosition: (_, at) => { offset = at; },
      setFnCursorPosition: (_, at) => { offset = at; },
    };
    const host = { active: true, textarea: { value: '' }, cursor, wasm,
      getNonEmptyHeaderFooterSelection: () => null, afterEdit() {},
      executeOperation: ({ command }) => history.recordWithoutExecute(command, wasm) };
    for (const input of [value, 'X']) {
      host.textarea.value = input;
      onInput.call(host);
      assert.equal(offset, history.peekUndoTop().editContext().charOffset);
    }
    assert.equal(text, withInput(value + 'X'), `${mode}: subsequent input`);
    assert.equal(offset, insertedAt + 1 + [...value].length);
    for (let cycle = 0; cycle < 2; cycle++) {
      history.undo(wasm);
      assert.equal(text, withInput(value));
      history.undo(wasm);
      assert.equal(text, 'AB');
      assert.equal(history.peekRedoTop().editContext().charOffset, requestedOffset);
      history.redo(wasm);
      assert.equal(text, withInput(value));
      const context = history.peekUndoTop().editContext();
      assert.equal(context.mode, mode === 'footnote' ? 'footnote' : 'headerFooter');
      assert.equal(context.charOffset, insertedAt + [...value].length, `${mode}: redo caret`);
      history.redo(wasm);
      assert.equal(text, withInput(value + 'X'));
      assert.equal(history.peekUndoTop().editContext().charOffset, insertedAt + 1 + [...value].length);
    }
    // Successive IME updates replace just the composing range, including after the last inline control.
    text = 'AB';
    const composition = { cursor, wasm, isComposing: true,
      canInsertTextInFormMode: () => true, canDeleteTextInFormMode: () => true };
    const anchor = { sectionIndex: 0, paragraphIndex: 0, charOffset: requestedOffset };
    let previous = '';
    for (const update of ['ㅎ', '한', '한😀', '']) {
      replaceTextAtRaw.call(composition, anchor, [...previous].length, update);
      assert.equal(text, withInput(update), `${mode}: IME update at ${requestedOffset}`);
      previous = update;
    }
  }
}
console.log('HF_FN_SCALAR_INPUT_OK');
