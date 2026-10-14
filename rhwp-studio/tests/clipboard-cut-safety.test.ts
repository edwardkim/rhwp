import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { stripTypeScriptTypes } from 'node:module';
import { createServer } from 'vite';
import { functionBodyFrom } from './support/source-guard.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const text = '나🦦다';
const html = '<p><span style="font-weight:bold;color:#cc0000">나🦦다</span></p>';
const nestedPath = [
  { controlIndex: 1, cellIndex: 6, cellParaIndex: 0 },
  { controlIndex: 2, cellIndex: 4, cellParaIndex: 3 },
];
type Mode = 'body' | 'cell' | 'nested' | 'object' | 'headerFooter';
type Failure = 'ok-false' | 'throw' | 'bad-json' | 'ok-missing' | 'ok-not-boolean' | 'null-clipboard' | 'plain-write' | 'html-write';
const failures: Failure[] = [
  'ok-false', 'throw', 'bad-json', 'ok-missing', 'ok-not-boolean', 'null-clipboard', 'plain-write', 'html-write',
];

function withoutMarker(value: string): string {
  return value.replace(/<!--rhwp-studio-clipboard:[A-Za-z0-9._:-]+-->\n?/, '');
}

function fixture(keyboard: any, copy: (this: any, e: any) => boolean, mode: Mode, failure?: Failure) {
  const calls: string[] = [];
  const copyCalls: Array<{ name: string; args: unknown[] }> = [];
  const clipboard = new Map<string, string>([
    ['text/plain', '이전 클립보드'], ['text/html', '<p>이전 클립보드</p>'],
  ]);
  const state = {
    selected: true,
    document: '원래 문서',
    engineResult: '{"ok":true}' as string | Error,
    headerResult: { ok: true, text: '머리말 & 꼬리말\n다음 줄' },
    failure,
  };
  const start: any = { sectionIndex: 2, paragraphIndex: 3, charOffset: 5 };
  const end: any = { sectionIndex: 2, paragraphIndex: 4, charOffset: 7 };
  if (mode === 'cell' || mode === 'nested') {
    Object.assign(start, { parentParaIndex: 8, controlIndex: 1, cellIndex: 6, cellParaIndex: 3 });
    Object.assign(end, { parentParaIndex: 8, controlIndex: 1, cellIndex: 6, cellParaIndex: 4 });
  }
  if (mode === 'nested') {
    start.cellPath = nestedPath;
    end.cellPath = nestedPath.map((part, index) => index === 1 ? { ...part, cellParaIndex: 4 } : part);
  }
  const ref = { sec: 2, ppi: 8, ci: 1, type: 'shape' };
  const headerSelection = {
    start: { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 3, charOffset: 5 },
    end: { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 4, charOffset: 7 },
  };
  const engineCopy = (name: string, args: unknown[]) => {
    calls.push('copy');
    copyCalls.push({ name, args });
    if (state.failure === 'throw') throw new Error('엔진 복사 실패');
    if (state.failure === 'ok-false') return '{"ok":false}';
    if (state.failure === 'bad-json') return '잘못된 JSON';
    if (state.failure === 'ok-missing') return '{}';
    if (state.failure === 'ok-not-boolean') return '{"ok":1}';
    if (state.engineResult instanceof Error) throw state.engineResult;
    return state.engineResult;
  };
  const engineExport = () => { calls.push('export'); return html; };
  const wasm = {
    copySelection: (...args: unknown[]) => engineCopy('body', args),
    copySelectionInCell: (...args: unknown[]) => engineCopy('cell', args),
    copySelectionInCellByPath: (...args: unknown[]) => engineCopy('nested', args),
    copyControl: (...args: unknown[]) => engineCopy('object', args),
    copySelectionInHeaderFooter: (...args: unknown[]) => {
      copyCalls.push({ name: 'headerFooter', args });
      calls.push('copy');
      if (state.failure === 'throw') throw new Error('머리말 복사 실패');
      if (state.failure === 'ok-false') return { ok: false, text: '이전 엔진 클립보드' };
      return state.headerResult;
    },
    getClipboardText: () => { calls.push('read'); return text; },
    exportSelectionHtml: engineExport,
    exportSelectionInCellHtml: engineExport,
    exportSelectionInCellHtmlByPath: engineExport,
    exportControlHtml: engineExport,
    deleteShapeControl: (...args: unknown[]) => {
      assert.deepEqual(args, [2, 8, 1]);
      calls.push('delete-object');
      state.document = '삭제한 문서';
    },
  };
  const handler: any = {
    active: true,
    isFormMode: () => false,
    wasm,
    cursor: {
      isInHeaderFooter: () => mode === 'headerFooter',
      isInPictureObjectSelection: () => mode === 'object',
      getSelectedPictureRef: () => state.selected ? ref : null,
      hasSelection: () => state.selected,
      getSelectionOrdered: () => state.selected ? { start, end } : null,
      getPosition: () => start,
      moveOutOfSelectedPicture: () => { calls.push('exit-object'); state.selected = false; },
    },
    getNonEmptyHeaderFooterSelection: () => state.selected ? headerSelection : null,
    onCopy: function (e: any) { return copy.call(this, e); },
    deleteSelection: () => {
      calls.push('delete-text');
      state.selected = false;
      state.document = '삭제한 문서';
    },
    pictureObjectRenderer: { clear: () => calls.push('clear-object') },
    eventBus: { emit: (name: string, value: boolean) => {
      assert.equal(name, 'picture-object-selection-changed');
      assert.equal(value, false);
      calls.push('selection-event');
    } },
    executeOperation: (operation: any) => {
      assert.equal(operation.kind, 'snapshot');
      assert.equal(operation.operationType, 'cutObject');
      calls.push('history');
      assert.equal(operation.operation(wasm), start);
    },
  };
  const event = {
    preventDefault: () => calls.push('prevent'),
    clipboardData: failure === 'null-clipboard' ? null : {
      setData: (type: string, value: string) => {
        calls.push(`write:${type}`);
        if (type === 'text/plain' && state.failure === 'plain-write') throw new Error('텍스트 기록 실패');
        if (type === 'text/html' && state.failure === 'html-write') throw new Error('HTML 기록 실패');
        clipboard.set(type, value);
      },
    },
  };
  return { handler, event, state, calls, copyCalls, clipboard, cut: () => keyboard.onCut.call(handler, event) };
}

function assertPreserved(f: ReturnType<typeof fixture>) {
  assert.equal(f.state.document, '원래 문서', '복사 실패 시 문서가 남는다');
  assert.equal(f.state.selected, true, '복사 실패 시 선택 범위가 남는다');
  assert.ok(!f.calls.some(call => ['delete-text', 'delete-object', 'exit-object', 'clear-object', 'selection-event', 'history'].includes(call)),
    '삭제·선택 해제·변경 이벤트·이력 기록을 하지 않는다');
}

test('ClipboardEvent 잘라내기는 이번 복사가 성공한 뒤에만 문서를 지운다', async (t) => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent',
    server: { middlewareMode: true, hmr: false },
  });
  try {
    const keyboard = await vite.ssrLoadModule('/src/engine/input-handler-keyboard.ts');
    // 실제 InputHandler 위임도 실행해서 성공 여부가 호출자까지 전달되는지 확인한다.
    const method = functionBodyFrom(readFileSync(join(rootDir, 'src/engine/input-handler.ts'), 'utf8'), 'private onCopy(');
    const methodCode = stripTypeScriptTypes(`const holder = { ${method.replace('private ', '')} };`);
    const copy = new Function('_keyboard', `${methodCode} return holder.onCopy;`)(keyboard);

    for (const mode of ['body', 'cell', 'nested'] as const) {
      await t.test(`${mode}: 실제 위임은 plain·서식 HTML 기록 성공을 반환하고 cut을 허용한다`, () => {
        const f = fixture(keyboard, copy, mode);
        assert.equal(f.handler.onCopy(f.event), true);
        assert.equal(f.clipboard.get('text/plain'), text);
        assert.equal(withoutMarker(f.clipboard.get('text/html')!), html);
        assert.match(f.clipboard.get('text/html')!, /<!--rhwp-studio-clipboard:/);
        assert.equal(f.copyCalls[0].name, mode);
        const expectedArgs = mode === 'body' ? [2, 3, 5, 4, 7]
          : mode === 'cell' ? [2, 8, 1, 6, 3, 5, 4, 7]
            : [2, 8, JSON.stringify(nestedPath), 3, 5, 4, 7];
        assert.deepEqual(f.copyCalls[0].args, expectedArgs);
        f.calls.length = 0;
        f.cut();
        assert.equal(f.state.document, '삭제한 문서');
        assert.equal(f.calls.filter(call => call === 'delete-text').length, 1);
        assert.ok(f.calls.indexOf('write:text/html') < f.calls.indexOf('delete-text'));
      });

      for (const failure of failures) {
        await t.test(`${mode}: ${failure}에서 기존 클립보드로 삭제하지 않는다`, (caseTest) => {
          caseTest.mock.method(console, 'warn', () => {});
          const f = fixture(keyboard, copy, mode, failure);
          f.cut();
          assertPreserved(f);
          assert.equal(f.handler.onCopy(f.event), false);
          if (failure !== 'html-write' && failure !== 'plain-write') {
            assert.equal(f.clipboard.get('text/plain'), '이전 클립보드');
            assert.equal(f.clipboard.get('text/html'), '<p>이전 클립보드</p>');
            assert.ok(!f.calls.includes('read'));
            assert.ok(!f.calls.includes('export'));
            assert.ok(!f.calls.some(call => call.startsWith('write:')));
          } else {
            assert.equal(f.clipboard.get('text/html'), '<p>이전 클립보드</p>');
            if (failure === 'plain-write') assert.equal(f.clipboard.get('text/plain'), '이전 클립보드');
          }
        });
      }

      await t.test(`${mode}: 직전 복사 성공이 다음 복사 실패를 허용하지 않는다`, (caseTest) => {
        caseTest.mock.method(console, 'warn', () => {});
        const f = fixture(keyboard, copy, mode);
        assert.equal(f.handler.onCopy(f.event), true);
        f.state.engineResult = '{"ok":false}';
        f.calls.length = 0;
        f.cut();
        assertPreserved(f);
        assert.equal(f.clipboard.get('text/plain'), text);
        assert.deepEqual(f.calls.filter(call => call !== 'prevent'), ['copy']);
      });
    }

    for (const failure of failures) {
      await t.test(`개체: ${failure}에서 선택·문서·이력을 보존한다`, (caseTest) => {
        caseTest.mock.method(console, 'warn', () => {});
        const f = fixture(keyboard, copy, 'object', failure);
        f.cut();
        assertPreserved(f);
        assert.equal(f.handler.onCopy(f.event), false);
        if (failure !== 'html-write' && failure !== 'plain-write') {
          assert.ok(!f.calls.includes('read'));
          assert.ok(!f.calls.includes('export'));
          assert.equal(f.clipboard.get('text/plain'), '이전 클립보드');
        }
      });
    }

    await t.test('개체: 복사 성공 뒤 기존 snapshot 경로가 도형을 한 번 지운다', () => {
      const f = fixture(keyboard, copy, 'object');
      f.cut();
      assert.equal(f.clipboard.get('text/plain'), text);
      assert.equal(withoutMarker(f.clipboard.get('text/html')!), html);
      assert.deepEqual(f.copyCalls, [{ name: 'object', args: [2, 8, 1, ''] }]);
      assert.equal(f.state.document, '삭제한 문서');
      assert.equal(f.state.selected, false);
      assert.deepEqual(f.calls.slice(f.calls.indexOf('write:text/html') + 1),
        ['exit-object', 'clear-object', 'selection-event', 'history', 'delete-object']);
    });

    for (const failure of ['ok-false', 'throw', 'null-clipboard', 'plain-write', 'html-write'] as const) {
      await t.test(`머리말·꼬리말: ${failure}에서 선택 내용을 지우지 않는다`, (caseTest) => {
        caseTest.mock.method(console, 'warn', () => {});
        const f = fixture(keyboard, copy, 'headerFooter', failure);
        f.cut();
        assertPreserved(f);
        assert.equal(f.handler.onCopy(f.event), false);
      });
    }

    await t.test('머리말·꼬리말: 이번 plain·fallback HTML 기록 뒤에 선택을 지운다', () => {
      const f = fixture(keyboard, copy, 'headerFooter');
      assert.equal(f.handler.onCopy(f.event), true);
      assert.equal(f.clipboard.get('text/plain'), f.state.headerResult.text);
      assert.equal(withoutMarker(f.clipboard.get('text/html')!),
        '<html><body>\n<!--StartFragment-->\n<p>머리말 &amp; 꼬리말</p>\n<p>다음 줄</p>\n<!--EndFragment-->\n</body></html>');
      assert.deepEqual(f.copyCalls[0], { name: 'headerFooter', args: [2, true, 1, 3, 5, 4, 7] });
      f.calls.length = 0;
      f.cut();
      assert.equal(f.state.document, '삭제한 문서');
      assert.ok(f.calls.indexOf('write:text/html') < f.calls.indexOf('delete-text'));
    });

    for (const mode of ['body', 'object', 'headerFooter'] as const) {
      for (const guard of ['inactive', 'no-selection', 'form'] as const) {
        await t.test(`${mode}: ${guard}에서는 cut이 엔진·문서를 건드리지 않는다`, () => {
          const f = fixture(keyboard, copy, mode);
          if (guard === 'inactive') f.handler.active = false;
          if (guard === 'no-selection') f.state.selected = false;
          if (guard === 'form') f.handler.isFormMode = () => true;
          f.cut();
          assert.equal(f.state.document, '원래 문서');
          assert.equal(f.copyCalls.length, 0);
          assert.deepEqual(f.calls, guard === 'form' ? ['prevent'] : []);
          if (guard !== 'form') assert.equal(f.handler.onCopy(f.event), false);
        });
      }
    }
  } finally {
    await vite.close();
  }
});
