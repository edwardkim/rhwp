import { registerHooks } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { dirname, resolve } from 'node:path';
import assert from 'node:assert/strict';

const src = resolve(dirname(fileURLToPath(import.meta.url)), '../../src');
registerHooks({ resolve(specifier, context, next) {
  if (specifier.startsWith('@/')) return {
    url: pathToFileURL(resolve(src, specifier.slice(2) + '.ts')).href, shortCircuit: true,
  };
  if (specifier.startsWith('.') && !/\.[cm]?[jt]s$/.test(specifier)) return {
    url: new URL(specifier + '.ts', context.parentURL).href, shortCircuit: true,
  };
  return next(specifier, context);
} });
const { InsertTextCommand } = await import(pathToFileURL(resolve(src, 'engine/command.ts')));
const { CommandHistory } = await import(pathToFileURL(resolve(src, 'engine/history.ts')));
const { onInput } = await import(pathToFileURL(resolve(src, 'engine/input-handler-text.ts')));

for (const kind of ['body', 'cell', 'nested']) {
  for (const input of ['가\r\n😀\r\n끝', '가\r😀\r끝', '가\n😀\n끝']) {
    let value = '앞뒤😀';
    // 코어의 공개 입력 계약을 모형화한다. 실제 fresh WASM은 별도 E2E에서 대조한다.
    const insert = (...args) => {
      const chars = [...value];
      const normalized = args.at(-1).replace(/\r\n?/g, '\n');
      chars.splice(args.at(-2), 0, ...normalized);
      value = chars.join('');
    };
    const remove = (...args) => {
      const chars = [...value];
      chars.splice(args.at(-2), args.at(-1));
      value = chars.join('');
    };
    const wasm = {
      snapshotCapacity: () => 100,
      insertText: insert, deleteText: remove,
      insertTextInCell: insert, deleteTextInCell: remove,
      insertTextInCellByPath: insert, deleteTextInCellByPath: remove,
    };
    const position = { sectionIndex: 0, paragraphIndex: 0, charOffset: 1 };
    if (kind !== 'body') Object.assign(position, {
      parentParaIndex: 0, controlIndex: 0, cellIndex: 0, cellParaIndex: 0,
      ...(kind === 'nested' ? { cellPath: [
        { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
        { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
      ] } : {}),
    });
    const history = new CommandHistory();
    const command = new InsertTextCommand(position, input);
    const after = history.execute(command, wasm);
    assert.equal(value, '앞가\n😀\n끝뒤😀', `${kind}: normalized insertion`);
    assert.equal(after.charOffset, 6, `${kind}: normalized scalar caret`);
    assert.deepEqual(command.getPageLocalTextEditOptions(), { insertedText: '가\n😀\n끝' });
    for (let cycle = 0; cycle < 2; cycle++) {
      history.undo(wasm);
      assert.equal(value, '앞뒤😀', `${kind}: Undo must preserve suffix`);
      const redone = history.redo(wasm);
      assert.equal(value, '앞가\n😀\n끝뒤😀');
      assert.equal(redone.charOffset, 6);
    }
  }
}

// 직접 코어를 호출한 뒤 명령을 기록하는 서브모드도 같은 입력을 사용해야 한다.
for (const mode of ['header', 'footer', 'footnote']) {
  for (const input of ['가\r\n😀\r\n끝', '가\r😀\r끝', '가\n😀\n끝']) {
    let value = '앞뒤😀';
    let offset = 1;
    const history = new CommandHistory();
    const insert = (...args) => {
      const chars = [...value];
      chars.splice(args.at(-2), 0, ...args.at(-1));
      value = chars.join('');
    };
    const remove = (...args) => {
      const chars = [...value];
      chars.splice(args.at(-2), args.at(-1));
      value = chars.join('');
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
    const host = { active: true, textarea: { value: input }, cursor, wasm,
      getNonEmptyHeaderFooterSelection: () => null, afterEdit() {},
      executeOperation: ({ command }) => history.recordWithoutExecute(command, wasm) };
    onInput.call(host);
    assert.equal(value, '앞가\n😀\n끝뒤😀', `${mode}: 입력 경계 정규화`);
    assert.equal(offset, 6, `${mode}: 정규화된 캐럿`);
    history.undo(wasm);
    assert.equal(value, '앞뒤😀');
    history.redo(wasm);
    assert.equal(value, '앞가\n😀\n끝뒤😀');
  }
}
console.log('CRLF_INSERT_HISTORY_OK');
