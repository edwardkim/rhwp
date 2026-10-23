import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import type { DocumentPosition } from '../src/core/types.ts';

// 제품 CursorState/CommandHistory/DeleteSelectionCommand를 실행한다.
// WASM 대역은 주소 조회·내용·스냅샷만 제공하며 geometry는 이 검사의 대상이 아니다.
const flat = (cellParaIndex = 0): DocumentPosition => ({
  sectionIndex: 0, paragraphIndex: cellParaIndex, charOffset: 0,
  parentParaIndex: 3, controlIndex: 2, cellIndex: 1, cellParaIndex,
});
const entry = (cellParaIndex = 0) => ({ controlIndex: 2, cellIndex: 1, cellParaIndex });

await test('#8102 셀 선택과 삭제의 실제 제품 경로', async (t) => {
  const vite = await createServer({
    root: new URL('..', import.meta.url).pathname, appType: 'custom',
    logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const { DeleteSelectionCommand } = await vite.ssrLoadModule('/src/engine/command.ts');
    const { CommandHistory } = await vite.ssrLoadModule('/src/engine/history.ts');

    function fixture() {
      let paragraphs = ['alpha', 'beta', 'gamma'];
      let saves = 0;
      let mutations = 0;
      const paths: any[] = [];
      const snapshots = new Map<number, string[]>();
      const readPath = (json: string) => {
        const path = JSON.parse(json);
        assert.ok(path.length > 0, '빈 경로는 유효한 셀이 아니다');
        paths.push(path);
        return path;
      };
      const wasm = {
        snapshotCapacity: () => 100,
        getCellParagraphCountByPath(_s: number, _p: number, json: string) { readPath(json); return paragraphs.length; },
        getCellParagraphLengthByPath(_s: number, _p: number, json: string) {
          const path = readPath(json); return paragraphs[path.at(-1).cellParaIndex]?.length ?? 0;
        },
        getCellParagraphCount() { return paragraphs.length; },
        getCellParagraphLength(_s: number, _p: number, _c: number, _i: number, para: number) { return paragraphs[para].length; },
        saveSnapshot() { snapshots.set(++saves, [...paragraphs]); return saves; },
        restoreSnapshot(id: number) { paragraphs = [...snapshots.get(id)!]; },
        discardSnapshot(id: number) { snapshots.delete(id); },
        deleteRangeInCellByPath(_s: number, _p: number, json: string, sp: number, so: number, ep: number, eo: number) {
          readPath(json); mutations++;
          paragraphs.splice(sp, ep - sp + 1, paragraphs[sp].slice(0, so) + paragraphs[ep].slice(eo));
        },
      };
      const cursor = new CursorState(wasm);
      cursor.updateRect = () => {};
      return { wasm, cursor, paths, text: () => paragraphs, saves: () => saves, mutations: () => mutations };
    }

    for (const mode of ['flat', 'empty-path', 'clicked', 'nested', 'textbox']) {
      await t.test(`${mode}: 다중 문단 선택·삭제·undo·redo·undo 재진입`, () => {
        const f = fixture();
        const pos = flat(1);
        if (mode === 'empty-path') pos.cellPath = [];
        if (mode === 'clicked' || mode === 'textbox') pos.cellPath = [entry(1)];
        if (mode === 'nested') pos.cellPath = [entry(4), { controlIndex: 3, cellIndex: 2, cellParaIndex: 1 }];
        if (mode === 'textbox') pos.isTextBox = true;
        const original = structuredClone(pos);
        f.cursor.moveTo(pos);
        assert.equal(f.cursor.selectAllInCell(), true);
        const selected = f.cursor.getSelectionOrdered();
        assert.ok(selected);
        assert.equal(selected.start.charOffset, 0);
        assert.equal(selected.end.charOffset, 5);
        assert.equal(selected.start.cellPath.at(-1).cellParaIndex, 0);
        assert.equal(selected.end.cellPath.at(-1).cellParaIndex, 2);
        if (mode === 'nested') assert.equal(selected.end.cellPath[0].cellParaIndex, 4);
        assert.deepEqual(pos, original, '호출자의 위치를 변경하지 않는다');
        const history = new CommandHistory();
        history.execute(new DeleteSelectionCommand(selected.start, selected.end), f.wasm);
        assert.deepEqual(f.text(), ['']);
        const restored = history.undo(f.wasm);
        assert.deepEqual(f.text(), ['alpha', 'beta', 'gamma']);
        history.redo(f.wasm);
        assert.deepEqual(f.text(), ['']);
        f.cursor.moveTo(history.undo(f.wasm));
        assert.ok(restored);
        assert.equal(f.cursor.selectAllInCell(), true);
        const again = f.cursor.getSelectionOrdered();
        history.execute(new DeleteSelectionCommand(again.start, again.end), f.wasm);
        assert.deepEqual(f.text(), ['']);
      });
    }

    await t.test('flat 위치를 직접 받는 삭제도 같은 셀로 정규화한다', () => {
      const f = fixture();
      const cmd = new DeleteSelectionCommand(flat(), { ...flat(2), charOffset: 5 });
      cmd.execute(f.wasm);
      assert.deepEqual(f.paths.at(-1), [entry()]);
      assert.deepEqual(f.text(), ['']);
    });

    await t.test('누락·잘못된 좌표, 다른 셀, 역방향 범위는 저장·변경 전에 거부한다', () => {
      for (const bad of [
        { controlIndex: undefined }, { cellIndex: undefined }, { cellParaIndex: undefined },
        { cellIndex: -1 }, { cellIndex: NaN }, { cellIndex: 0.5 },
        { cellPath: [{ ...entry(), cellIndex: undefined }] },
        { cellPath: [{ ...entry(), controlIndex: -1 }] },
      ]) {
        const f = fixture();
        assert.throws(() => new DeleteSelectionCommand({ ...flat(), ...bad }, { ...flat(), charOffset: 5 }).execute(f.wasm));
        assert.equal(f.saves(), 0);
        assert.equal(f.mutations(), 0);
      }
      for (const end of [
        { ...flat(), cellIndex: 2 }, { ...flat(), sectionIndex: 1 },
        { ...flat(), parentParaIndex: 4 }, { sectionIndex: 0, paragraphIndex: 0, charOffset: 5 },
        { ...flat(), charOffset: -1 },
      ]) {
        const f = fixture();
        assert.throws(() => new DeleteSelectionCommand(flat(1), end).execute(f.wasm));
        assert.equal(f.mutations(), 0);
      }
    });

    await t.test('잘못된 셀 전체 선택은 기존 위치·선택을 변경하지 않는다', () => {
      const f = fixture();
      const bad = { ...flat(), cellIndex: undefined };
      f.cursor.moveTo(bad);
      f.cursor.setAnchor();
      const before = f.cursor.getSelection();
      assert.throws(() => f.cursor.selectAllInCell());
      assert.deepEqual(f.cursor.getSelection(), before);
      assert.equal(f.mutations(), 0);
    });

    await t.test('존재하지 않는 문단·오프셋을 변경 전에 거부한다', () => {
      for (const end of [{ ...flat(3), charOffset: 0 }, { ...flat(2), charOffset: 6 }]) {
        const f = fixture();
        const start = { ...flat(), cellPath: [entry()] };
        assert.throws(() => new DeleteSelectionCommand(start, { ...end, cellPath: [entry(end.cellParaIndex)] }).execute(f.wasm));
        assert.equal(f.mutations(), 0);
        assert.deepEqual(f.text(), ['alpha', 'beta', 'gamma']);
      }
    });
  } finally { await vite.close(); }
});
