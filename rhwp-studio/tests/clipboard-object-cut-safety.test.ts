import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { stripTypeScriptTypes } from 'node:module';
import { createServer } from 'vite';
import { functionBodyFrom } from './support/source-guard.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = readFileSync(join(rootDir, 'src/engine/input-handler.ts'), 'utf8');
type Mode = 'picture' | 'table' | 'none';

function deferred() {
  let resolve!: () => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<void>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

async function settle() { await new Promise<void>(resolve => setImmediate(resolve)); }

function fixture(keyboard: any, picture: any, tableObjectClipboardTarget: any, mode: Mode, text: any) {
  const calls: string[] = [];
  const copyCalls: unknown[][] = [];
  const deleteCalls: Array<{ name: string; args: unknown[] }> = [];
  const primary: Array<ReturnType<typeof deferred> & { name: string; args: unknown[] }> = [];
  const fallback: Array<ReturnType<typeof deferred> & { text: string }> = [];
  const listeners = new Map<string, Set<() => void>>();
  const state = {
    mode, ref: { sec: 2, ppi: 8, ci: 3, ...(mode === 'picture' ? { type: 'shape' } : {}) } as any,
    document: '원래 문서', engineResult: '{"ok":true}' as string | Error,
    generation: 7, handle: {} as object | null, undoTop: {} as object | null,
  };
  const position = { sectionIndex: 2, paragraphIndex: 8, charOffset: 0 };
  const write = (name: string, args: unknown[]) => {
    const pending = { ...deferred(), name, args };
    primary.push(pending);
    calls.push('write-rich');
    return pending.promise;
  };
  const deleting = (name: string) => (...args: unknown[]) => {
    deleteCalls.push({ name, args });
    state.document = '삭제한 문서';
    calls.push('delete');
  };
  const eventBus = {
    on: (event: string, listener: () => void) => {
      let group = listeners.get(event);
      if (!group) { group = new Set(); listeners.set(event, group); }
      group.add(listener);
      return () => group!.delete(listener);
    },
    emit: (event: string, ..._args: unknown[]) => {
      calls.push(`event:${event}`);
      for (const listener of listeners.get(event) ?? []) listener();
    },
  };
  const noop = () => {};
  const disposable = { hide: noop, clear: noop, dispose: noop };
  const handler: any = {
    active: true, editMode: 'normal', cancelPendingObjectCut: null,
    isComposing: false, compositionAnchor: null, compositionLength: 0,
    isFormMode() { return this.editMode === 'form'; },
    canInsertTextInFormMode: () => true,
    wasm: {
      get documentGeneration() { return state.generation; },
      borrowDocumentHandle: () => state.handle,
      copyControl: (...args: unknown[]) => {
        copyCalls.push(args);
        calls.push('copy');
        if (state.engineResult instanceof Error) throw state.engineResult;
        return state.engineResult;
      },
      getClipboardText: () => { calls.push('read'); return '선택한 개체'; },
      exportControlHtml: (...args: unknown[]) => {
        assert.deepEqual(args, copyCalls.at(-1));
        calls.push('export');
        return '<p><b>선택한 개체</b></p>';
      },
      deleteShapeControl: deleting('shape'),
      deleteEquationControl: deleting('equation'),
      deletePictureControl: deleting('image'),
      deleteCellPictureControlByPath: deleting('cell-image'),
      deleteTableControl: deleting('table'),
      discardDeleteFragment: noop,
    },
    cursor: {
      isInPictureObjectSelection: () => state.mode === 'picture',
      isInTableObjectSelection: () => state.mode === 'table',
      hasSelection: () => false,
      isInHeaderFooter: () => false,
      isInFootnote: () => false,
      getSelectedPictureRef: () => state.mode === 'picture' ? state.ref : null,
      getSelectedTableRef: () => state.mode === 'table' ? state.ref : null,
      getPosition: () => position,
      clearSelection: noop,
      moveOutOfSelectedPicture: () => { state.mode = 'none'; calls.push('exit-picture'); },
      moveOutOfSelectedTable: () => { state.mode = 'none'; calls.push('exit-table'); },
    },
    history: {
      peekUndoTop: () => state.undoTop,
      clear: () => { state.undoTop = null; },
    },
    pictureObjectRenderer: { clear: () => calls.push('clear-picture') },
    eventBus,
    executeOperation: (operation: any) => {
      assert.equal(operation.kind, 'snapshot');
      assert.equal(operation.operationType, mode === 'table' ? 'cutTable' : 'cutObject');
      calls.push('history');
      assert.equal(operation.operation(handler.wasm), position);
    },
    focusTextarea: () => calls.push('focus'),
    flushDeferredPaginationIfNeeded: noop, cancelDeferredPaginationFlush: noop,
    resetRawTextMutationEffects: noop, stopTextSelectionDragAutoScroll: noop,
    deferredPaginationRunner: { cancel: noop }, caretLayoutReveal: { clear: noop },
    _compositionFragment: null, _imeNavigationGuard: { reset: noop }, _cellBlockLetterImeGuard: { reset: noop, consume: () => false },
    caret: disposable, fieldMarker: disposable, selectionRenderer: disposable,
    container: { removeEventListener: noop },
    textarea: { value: '', remove: noop, removeEventListener: noop },
  };
  const keyboardBoundary = {
    ...keyboard,
    writeTextHtmlToClipboard: (...args: unknown[]) => write('rich', args),
    writeImageToClipboard: (...args: unknown[]) => write('image', args),
  };
  const navigatorBoundary = { clipboard: { writeText: (text: string) => {
    const pending = { ...deferred(), text };
    fallback.push(pending);
    calls.push('write-plain');
    return pending.promise;
  } } };
  const documentBoundary = { removeEventListener: noop, execCommand: (command: string) => { calls.push(command); return true; } };
  // 공개 명령에서 실제 private helper까지 연결하며, 구버전은 원래 공개 메서드 그대로 실행한다.
  for (const name of ['performCopy', 'performCut', 'deactivate', 'dispose', 'deleteObjectControl', 'copySelectedObject', 'cutSelectedObject', 'onCompositionStart']) {
    if (!source.includes(`${name}(`)) continue;
    const signature = name === 'deleteObjectControl' || name === 'onCompositionStart' ? `private ${name}(`
      : name === 'copySelectedObject' || name === 'cutSelectedObject' ? `private async ${name}(` : `${name}(`;
    const method = functionBodyFrom(source, signature).replace('private ', '');
    const code = stripTypeScriptTypes(`const holder = { ${method} };`);
    handler[name] = new Function('_keyboard', '_picture', '_text', 'tableObjectClipboardTarget', 'navigator', 'document', 'window',
      `${code} return holder.${name};`)(keyboardBoundary, picture, text, tableObjectClipboardTarget, navigatorBoundary, documentBoundary, { removeEventListener: noop });
  }
  return {
    handler, state, calls, copyCalls, deleteCalls, primary, fallback, eventBus,
    cut: () => handler.performCut(), copy: () => handler.performCopy(),
    subscriptions: () => [...listeners.values()].reduce((sum, group) => sum + group.size, 0),
  };
}

function assertNoDeletion(f: ReturnType<typeof fixture>) {
  assert.equal(f.state.document, '원래 문서');
  assert.equal(f.deleteCalls.length, 0);
  assert.ok(!f.calls.includes('history'));
  assert.ok(!f.calls.includes('exit-picture') && !f.calls.includes('exit-table'));
}

test('공개 개체 잘라내기는 클립보드 기록을 기다리고 같은 편집 대상만 지운다', async t => {
  const vite = await createServer({ root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true, hmr: false } });
  try {
    const keyboard = await vite.ssrLoadModule('/src/engine/input-handler-keyboard.ts');
    const picture = await vite.ssrLoadModule('/src/engine/input-handler-picture.ts');
    const text = await vite.ssrLoadModule('/src/engine/input-handler-text.ts');
    const { tableObjectClipboardTarget } = await vite.ssrLoadModule('/src/engine/table-object-clipboard-target.ts');
    const make = (mode: Mode) => fixture(keyboard, picture, tableObjectClipboardTarget, mode, text);
    for (const mode of ['picture', 'table'] as const) {
      await t.test(`${mode}: rich 기록 완료 전 삭제하지 않고 완료 후 한 번 지운다`, async () => {
        const f = make(mode);
        assert.equal(f.cut(), undefined, '공개 명령은 기존 void 계약을 유지한다');
        assertNoDeletion(f);
        assert.equal(f.primary.length, 1);
        assert.equal(f.primary[0].name, 'rich');
        assert.equal(f.primary[0].args[0], '선택한 개체');
        assert.match(f.primary[0].args[1] as string, /<p><b>선택한 개체<\/b><\/p>/);
        assert.equal(f.subscriptions(), 2);
        f.primary[0].resolve();
        await settle();
        assert.equal(f.deleteCalls.length, 1);
        assert.equal(f.deleteCalls[0].name, mode === 'table' ? 'table' : 'shape');
        assert.deepEqual(f.deleteCalls[0].args, [2, 8, 3]);
        assert.equal(f.calls.filter(call => call === 'history').length, 1);
        assert.equal(f.state.mode, 'none');
        assert.equal(f.subscriptions(), 0);
      });

      for (const outcome of ['resolve', 'reject'] as const) {
        await t.test(`${mode}: rich 실패 뒤 plain fallback ${outcome}까지 삭제를 기다린다`, async caseTest => {
          caseTest.mock.method(console, 'warn', () => {});
          const f = make(mode);
          f.cut();
          assertNoDeletion(f);
          f.primary[0].reject(new Error('rich 거절'));
          await settle();
          assert.equal(f.fallback.length, 1);
          assert.equal(f.fallback[0].text, '선택한 개체');
          assertNoDeletion(f);
          if (outcome === 'resolve') f.fallback[0].resolve();
          else f.fallback[0].reject(new Error('plain 거절'));
          await settle();
          if (outcome === 'resolve') assert.equal(f.deleteCalls.length, 1);
          else {
            assertNoDeletion(f);
            assert.equal(f.state.mode, mode);
          }
          assert.equal(f.subscriptions(), 0);
        });
      }

      for (const result of ['{"ok":false}', '잘못된 JSON', '{}', '{"ok":1}', new Error('엔진 복사 실패')]) {
        await t.test(`${mode}: 엔진 ${String(result)}이면 쓰기·삭제하지 않는다`, async caseTest => {
          caseTest.mock.method(console, 'warn', () => {});
          const f = make(mode);
          f.state.engineResult = result;
          f.cut();
          await settle();
          assertNoDeletion(f);
          assert.equal(f.primary.length, 0);
          assert.equal(f.fallback.length, 0);
          assert.ok(!f.calls.includes('read') && !f.calls.includes('export'));
          assert.equal(f.state.mode, mode);
          assert.equal(f.subscriptions(), 0);
        });
      }

      const changes: Array<[string, (f: ReturnType<typeof make>) => void]> = [
        ['sec', f => f.state.ref.sec++],
        ['ppi', f => f.state.ref.ppi++],
        ['ci', f => f.state.ref.ci++],
        ['동일 주소 새 선택', f => { f.state.ref = { ...f.state.ref }; }],
        ['선택 모드', f => { f.state.mode = 'none'; }],
        ['문서 세대', f => f.state.generation++],
        ['문서 handle', f => { f.state.handle = {}; }],
        ['해제된 handle', f => { f.state.handle = null; }],
        ['비활성화', f => { f.handler.active = false; }],
        ['양식 모드', f => { f.handler.editMode = 'form'; }],
        ['빈 조합 시작', f => f.handler.onCompositionStart()],
        ['undo top', f => { f.state.undoTop = {}; }],
        ['같은 주소의 크기 변경', f => f.eventBus.emit('document-mutated')],
        ['문서 변경 이벤트', f => f.eventBus.emit('document-changed')],
      ];
      for (const [name, change] of changes) {
        await t.test(`${mode}: 기다리는 동안 ${name} 변경 시 원래 대상을 지우지 않는다`, async () => {
          const f = make(mode);
          f.cut();
          assertNoDeletion(f);
          const ref = f.state.ref;
          const undoTop = f.state.undoTop;
          const handle = f.state.handle;
          const generation = f.state.generation;
          change(f);
          const anchor = f.handler.compositionAnchor;
          if (name === '빈 조합 시작') {
            assert.equal(f.handler.isComposing, true);
            assert.equal(anchor, f.handler.cursor.getPosition());
            assert.equal(f.state.ref, ref);
            assert.equal(f.state.undoTop, undoTop);
            assert.equal(f.state.handle, handle);
            assert.equal(f.state.generation, generation);
          }
          f.primary[0].resolve();
          await settle();
          assertNoDeletion(f);
          if (name === '빈 조합 시작') {
            assert.equal(f.handler.isComposing, true);
            assert.equal(f.handler.compositionAnchor, anchor);
            assert.equal(f.handler.compositionLength, 0);
            assert.equal(f.state.mode, mode);
          }
          assert.equal(f.subscriptions(), 0);
        });
      }

      await t.test(`${mode}: plain fallback을 기다리는 동안 시작한 조합을 보존한다`, async () => {
        const f = make(mode);
        f.cut();
        f.primary[0].reject(new Error('rich 실패'));
        await settle();
        assert.equal(f.fallback.length, 1);
        f.handler.onCompositionStart();
        const anchor = f.handler.compositionAnchor;
        assert.equal(f.handler.isComposing, true);
        f.fallback[0].resolve();
        await settle();
        assertNoDeletion(f);
        assert.equal(f.state.mode, mode);
        assert.equal(f.handler.isComposing, true);
        assert.equal(f.handler.compositionAnchor, anchor);
        assert.equal(f.handler.compositionLength, 0);
        assert.equal(f.subscriptions(), 0);
      });

      await t.test(`${mode}: 중복 cut을 막고 실패 후 재시도한다`, async caseTest => {
        caseTest.mock.method(console, 'warn', () => {});
        const f = make(mode);
        f.cut(); f.cut();
        assert.equal(f.primary.length, 1);
        assertNoDeletion(f);
        f.primary[0].reject(new Error('rich 실패'));
        await settle();
        f.fallback[0].reject(new Error('plain 실패'));
        await settle();
        assertNoDeletion(f);
        assert.equal(f.subscriptions(), 0);
        f.cut();
        assert.equal(f.primary.length, 2);
        f.primary[1].resolve();
        await settle();
        assert.equal(f.deleteCalls.length, 1);
        assert.equal(f.subscriptions(), 0);
      });

      await t.test(`${mode}: deactivate 후 새 요청을 이전 요청의 finally가 해제하지 않는다`, async () => {
        const f = make(mode);
        f.cut();
        assertNoDeletion(f);
        f.handler.deactivate();
        assert.equal(f.subscriptions(), 0);
        f.handler.active = true;
        f.cut();
        assert.equal(f.primary.length, 2);
        assert.equal(f.subscriptions(), 2);
        f.primary[0].resolve();
        await settle();
        assertNoDeletion(f);
        assert.equal(f.subscriptions(), 2);
        f.cut();
        assert.equal(f.primary.length, 2, '새 요청이 진행 중이면 중복 요청을 막는다');
        f.primary[1].resolve();
        await settle();
        assert.equal(f.deleteCalls.length, 1);
        assert.equal(f.subscriptions(), 0);
      });

      await t.test(`${mode}: dispose는 지연 삭제를 취소하고 구독을 해제한다`, async () => {
        const f = make(mode);
        f.cut();
        assertNoDeletion(f);
        f.handler.dispose();
        assert.equal(f.subscriptions(), 0);
        f.primary[0].resolve();
        await settle();
        assertNoDeletion(f);
        assert.equal(f.subscriptions(), 0);
      });

      await t.test(`${mode}: 취소한 A의 늦은 실패가 완료한 B의 클립보드를 덮지 않는다`, async caseTest => {
        caseTest.mock.method(console, 'warn', () => {});
        const f = make(mode);
        f.cut();
        f.handler.deactivate();
        f.state.handle = {};
        f.state.generation++;
        f.state.ref = { ...f.state.ref };
        f.handler.active = true;
        f.cut();
        assert.equal(f.primary.length, 2);
        f.primary[1].resolve();
        await settle();
        assert.equal(f.deleteCalls.length, 1, 'B의 삭제는 완료했다');
        const documentAfterB = f.state.document;
        const historyAfterB = f.calls.filter(call => call === 'history').length;
        f.primary[0].reject(new Error('취소한 A가 뒤늦게 실패했다'));
        await settle();
        assert.equal(f.fallback.length, 0, '취소한 A의 plain fallback을 시작하지 않는다');
        assert.equal(f.state.document, documentAfterB);
        assert.equal(f.deleteCalls.length, 1);
        assert.equal(f.calls.filter(call => call === 'history').length, historyAfterB);
        assert.equal(f.state.mode, 'none');
        assert.equal(f.subscriptions(), 0);
      });

      await t.test(`${mode}: 선택 변경 이벤트가 문서를 바꿔도 새 문서를 지우지 않는다`, async () => {
        const f = make(mode);
        const oldDocument = { deletions: 0 };
        const newDocument = { deletions: 0 };
        const deletionName = mode === 'picture' ? 'deleteShapeControl' : 'deleteTableControl';
        f.handler.wasm[deletionName] = (...args: unknown[]) => {
          assert.deepEqual(args, [2, 8, 3]);
          oldDocument.deletions++;
          f.calls.push('delete-old');
        };
        const replacement = { ...f.handler.wasm, [deletionName]: () => newDocument.deletions++ };
        let deletionsAtEvent = -1;
        const stop = f.eventBus.on(`${mode === 'picture' ? 'picture' : 'table'}-object-selection-changed`, () => {
          deletionsAtEvent = oldDocument.deletions;
          f.handler.wasm = replacement;
          f.state.handle = {};
          f.state.generation++;
          f.eventBus.emit('document-changed');
        });
        f.cut();
        assert.equal(oldDocument.deletions, 0);
        f.primary[0].resolve();
        await settle();
        assert.equal(deletionsAtEvent, 1, '선택 변경 알림은 원래 문서 삭제가 완료된 뒤 보낸다');
        assert.equal(oldDocument.deletions, 1);
        assert.equal(newDocument.deletions, 0);
        assert.equal(f.calls.filter(call => call === 'history').length, 1);
        stop();
        assert.equal(f.subscriptions(), 0);
      });

      await t.test(`${mode}: HTML 준비 중 취소되면 플랫폼 쓰기를 시작하지 않는다`, async () => {
        const f = make(mode);
        const exportHtml = f.handler.wasm.exportControlHtml;
        f.handler.wasm.exportControlHtml = (...args: unknown[]) => {
          const html = exportHtml(...args);
          f.eventBus.emit('document-mutated');
          return html;
        };
        f.cut();
        await settle();
        assertNoDeletion(f);
        assert.equal(f.primary.length, 0);
        assert.equal(f.fallback.length, 0);
        assert.equal(f.subscriptions(), 0);
      });

      await t.test(`${mode}: 공개 Copy는 같은 rich 경로를 사용하고 삭제하지 않는다`, async () => {
        const f = make(mode);
        assert.equal(f.copy(), undefined);
        assert.equal(f.primary.length, 1);
        f.primary[0].resolve();
        await settle();
        assertNoDeletion(f);
        assert.equal(f.state.mode, mode);
        assert.equal(f.subscriptions(), 0);
      });

      for (const guard of ['inactive', 'form', 'missing-ref', 'composition'] as const) {
        await t.test(`${mode}: 처음부터 ${guard}이면 Cut은 쓰기·삭제하지 않는다`, async () => {
          const f = make(mode);
          if (guard === 'inactive') f.handler.active = false;
          if (guard === 'form') f.handler.editMode = 'form';
          if (guard === 'missing-ref') f.state.ref = null;
          if (guard === 'composition') f.handler.onCompositionStart();
          const anchor = f.handler.compositionAnchor;
          f.cut();
          await settle();
          assertNoDeletion(f);
          assert.equal(f.copyCalls.length, 0);
          assert.equal(f.primary.length, 0);
          assert.equal(f.fallback.length, 0);
          assert.equal(f.subscriptions(), 0);
          if (guard === 'composition') {
            assert.equal(f.handler.isComposing, true);
            assert.equal(f.handler.compositionAnchor, anchor);
            assert.equal(f.handler.compositionLength, 0);
          }
        });
      }
    }

    const cellPath = [{ controlIndex: 1, cellIndex: 4, cellParaIndex: 2 }];
    await t.test('셀 그림: 기존 이미지 rich 기록과 셀 삭제 주소를 보존한다', async () => {
      const f = make('picture');
      f.state.ref = { sec: 2, ppi: 8, ci: 3, type: 'image', cellPath };
      f.cut();
      assertNoDeletion(f);
      assert.deepEqual(f.copyCalls, [[2, 8, 3, JSON.stringify(cellPath)]]);
      assert.equal(f.primary[0].name, 'image');
      assert.deepEqual(f.primary[0].args.slice(1, 4), [2, 8, 3]);
      assert.equal(f.primary[0].args[6], JSON.stringify(cellPath));
      assert.equal(typeof f.primary[0].args[7], 'function', '변환 중에도 Cut 요청 유효성을 전달한다');
      f.primary[0].resolve();
      await settle();
      assert.deepEqual(f.deleteCalls, [{ name: 'cell-image', args: [2, 8, cellPath, 3] }]);
    });

    await t.test('셀 그림: 대기 중 cellPath가 바뀌면 새 주소의 그림을 지우지 않는다', async () => {
      const f = make('picture');
      f.state.ref = { sec: 2, ppi: 8, ci: 3, type: 'image', cellPath: structuredClone(cellPath) };
      f.cut();
      assertNoDeletion(f);
      f.state.ref.cellPath[0].cellIndex++;
      f.primary[0].resolve();
      await settle();
      assertNoDeletion(f);
      assert.equal(f.subscriptions(), 0);
    });

    await t.test('중첩 표: Copy는 owner 경로를 쓰고 Cut은 기존 금지 계약을 유지한다', async () => {
      const f = make('table');
      const nested = [...cellPath, { controlIndex: 6, cellIndex: 0, cellParaIndex: 0 }];
      f.state.ref.cellPath = nested;
      f.copy();
      assert.deepEqual(f.copyCalls, [[2, 8, 6, JSON.stringify(cellPath)]]);
      f.primary[0].resolve();
      await settle();
      f.copyCalls.length = 0;
      f.cut();
      await settle();
      assertNoDeletion(f);
      assert.equal(f.state.mode, 'table');
      assert.equal(f.copyCalls.length, 0);
      assert.equal(f.primary.length, 1);
      assert.equal(f.subscriptions(), 0);
    });

    await t.test('텍스트 Cut은 기존 textarea·execCommand 경로를 유지한다', () => {
      const f = make('none');
      f.cut();
      assert.deepEqual(f.calls, ['focus', 'cut']);
      assert.equal(f.primary.length, 0);
    });
  } finally { await vite.close(); }
});

test('실제 이미지 Clipboard writer는 변환 후 유효성을 확인하고 원래 payload를 기록한다', async t => {
  const keyboardSource = readFileSync(join(rootDir, 'src/engine/input-handler-keyboard.ts'), 'utf8');
  const method = functionBodyFrom(keyboardSource, 'export async function writeImageToClipboard(').replace('export ', '');
  const code = stripTypeScriptTypes(method);
  const cellPathJson = JSON.stringify([{ controlIndex: 1, cellIndex: 4, cellParaIndex: 2 }]);
  for (const cancel of [false, true]) {
    await t.test(cancel ? '변환 중 취소되면 플랫폼 쓰기 0회' : '변환 완료 뒤 plain·HTML·PNG를 1회 기록', async () => {
      const converting = deferred();
      const bytes = new Uint8Array([1, 2, 3]);
      const png = new Blob([new Uint8Array([9, 8, 7])], { type: 'image/png' });
      const reads: unknown[][] = [];
      const writes: Array<Array<{ data: Record<string, Blob> }>> = [];
      let current = true;
      const wasm = {
        getControlImageData: (...args: unknown[]) => { reads.push(args); return bytes; },
        getControlImageMime: (...args: unknown[]) => { reads.push(args); return 'image/jpeg'; },
      };
      class ClipboardItemBoundary {
        data: Record<string, Blob>;
        constructor(data: Record<string, Blob>) { this.data = data; }
      }
      const writer = new Function('convertToPngBlob', 'ClipboardItem', 'navigator', 'Blob', `${code}; return writeImageToClipboard;`)(
        (data: Uint8Array, mime: string) => {
          assert.equal(data, bytes);
          assert.equal(mime, 'image/jpeg');
          return converting.promise.then(() => png);
        }, ClipboardItemBoundary,
        { clipboard: { write: async (items: Array<{ data: Record<string, Blob> }>) => { writes.push(items); } } }, Blob,
      );
      const text = '셀 그림 원본';
      const html = '<p><b>셀 그림 원본</b></p>';
      const finished = writer(wasm, 2, 8, 3, text, html, cellPathJson, () => current);
      assert.deepEqual(reads, [[2, 8, 3, cellPathJson], [2, 8, 3, cellPathJson]]);
      assert.equal(writes.length, 0, '변환이 끝나기 전에는 쓰지 않는다');
      if (cancel) current = false;
      converting.resolve();
      await finished;
      assert.equal(writes.length, cancel ? 0 : 1);
      if (!cancel) {
        assert.equal(writes[0].length, 1);
        const payload = writes[0][0].data;
        assert.deepEqual(Object.keys(payload), ['text/plain', 'text/html', 'image/png']);
        assert.equal(await payload['text/plain'].text(), text);
        assert.equal(await payload['text/html'].text(), html);
        assert.equal(payload['image/png'], png);
        assert.equal(payload['text/plain'].type, 'text/plain');
        assert.equal(payload['text/html'].type, 'text/html');
      }
    });
  }
});
