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
const { onInput } = await import(pathToFileURL(resolve(src, 'engine/input-handler-text.ts')));
const { CommandHistory } = await import(pathToFileURL(resolve(src, 'engine/history.ts')));

for (const mode of ['header', 'footer', 'footnote']) {
  for (const value of ['가', '😀', '𠮷', '가😀𠮷']) {
    let text = 'AB';
    let offset = 1;
    const history = new CommandHistory();
    // Plain-text scalar bounds match Paragraph::insert_text_at/delete_text_at.
    const insert = (...args) => {
      const chars = [...text];
      chars.splice(Math.min(args.at(-2), chars.length), 0, ...args.at(-1));
      text = chars.join('');
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
    assert.equal(text, `A${value}XB`, `${mode}: subsequent input must precede B`);
    assert.equal(offset, 2 + [...value].length);
    for (let cycle = 0; cycle < 2; cycle++) {
      history.undo(wasm);
      assert.equal(text, `A${value}B`);
      history.undo(wasm);
      assert.equal(text, 'AB');
      assert.equal(history.peekRedoTop().editContext().charOffset, 1);
      history.redo(wasm);
      assert.equal(text, `A${value}B`);
      const context = history.peekUndoTop().editContext();
      assert.equal(context.mode, mode === 'footnote' ? 'footnote' : 'headerFooter');
      assert.equal(context.charOffset, 1 + [...value].length, `${mode}: redo caret`);
      history.redo(wasm);
      assert.equal(text, `A${value}XB`);
      assert.equal(history.peekUndoTop().editContext().charOffset, 2 + [...value].length);
    }
  }
}
console.log('HF_FN_SCALAR_INPUT_OK');
