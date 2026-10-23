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
    const { DeleteSelectionCommand, cellPathForEdit } = await vite.ssrLoadModule('/src/engine/command.ts');
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
      const forwardEnd = { ...flat(), charOffset: 1 };
      for (const end of [
        { ...forwardEnd, cellIndex: 2 }, { ...forwardEnd, sectionIndex: 1 },
        { ...forwardEnd, controlIndex: 3 },
        { ...forwardEnd, parentParaIndex: 4 }, { sectionIndex: 0, paragraphIndex: 0, charOffset: 1 },
        { ...forwardEnd, charOffset: -1 },
      ]) {
        const f = fixture();
        assert.throws(() => new DeleteSelectionCommand(flat(), end).execute(f.wasm));
        assert.equal(f.saves(), 0);
        assert.equal(f.mutations(), 0);
      }
      const reversed = fixture();
      assert.throws(() => new DeleteSelectionCommand(flat(1), flat()).execute(reversed.wasm));
      assert.equal(reversed.saves(), 0);
      assert.equal(reversed.mutations(), 0);
    });

    await t.test('중간 경로의 다른 문단·control·셀도 정방향 범위에서 거부한다', () => {
      const start = { ...flat(), cellPath: [entry(1), { controlIndex: 3, cellIndex: 2, cellParaIndex: 0 }] };
      for (const changed of [{ cellParaIndex: 2 }, { controlIndex: 4 }, { cellIndex: 2 }]) {
        const f = fixture();
        const end = { ...start, charOffset: 1,
          cellPath: [{ ...start.cellPath[0], ...changed }, { ...start.cellPath[1] }] };
        assert.throws(() => new DeleteSelectionCommand(start, end).execute(f.wasm));
        assert.equal(f.saves(), 0);
        assert.equal(f.mutations(), 0);
      }
    });

    await t.test('flat/path 주소가 WASM u32 상한을 넘으면 조회·저장·변경 전에 거부한다', () => {
      for (const usePath of [false, true]) {
        for (const field of ['sectionIndex', 'parentParaIndex', 'controlIndex', 'cellIndex', 'cellParaIndex']) {
          for (const value of [2 ** 32, Number.MAX_SAFE_INTEGER]) {
            const f = fixture();
            const pos: DocumentPosition = { ...flat(), [field]: value };
            if (usePath) pos.cellPath = [{ controlIndex: pos.controlIndex!, cellIndex: pos.cellIndex!, cellParaIndex: pos.cellParaIndex! }];
            assert.throws(() => new DeleteSelectionCommand(pos, { ...pos, charOffset: 1 }).execute(f.wasm), `${field}=${value}, path=${usePath}`);
            assert.equal(f.paths.length, 0);
            assert.equal(f.saves(), 0);
            assert.equal(f.mutations(), 0);
            assert.deepEqual(f.text(), ['alpha', 'beta', 'gamma']);
          }
        }
      }
      const maximum = { sectionIndex: 0xffff_ffff, paragraphIndex: 0, charOffset: 0,
        parentParaIndex: 0xffff_ffff, controlIndex: 0xffff_ffff, cellIndex: 0xffff_ffff, cellParaIndex: 0xffff_ffff };
      assert.deepEqual(cellPathForEdit(maximum), [{ controlIndex: 0xffff_ffff, cellIndex: 0xffff_ffff, cellParaIndex: 0xffff_ffff }]);
    });

    await t.test('잘못된 셀 전체 선택은 기존 위치·선택을 변경하지 않는다', () => {
      for (const bad of [
        { ...flat(), cellIndex: undefined },
        { ...flat(), sectionIndex: 2 ** 32 },
        { ...flat(), parentParaIndex: 2 ** 32 },
      ]) {
        const f = fixture();
        f.cursor.moveTo(bad);
        f.cursor.setAnchor();
        const before = f.cursor.getSelection();
        assert.throws(() => f.cursor.selectAllInCell());
        assert.deepEqual(f.cursor.getSelection(), before);
        assert.equal(f.saves(), 0);
        assert.equal(f.mutations(), 0);
      }
    });

    await t.test('존재하지 않는 문단·오프셋을 변경 전에 거부한다', () => {
      for (const end of [{ ...flat(3), charOffset: 0 }, { ...flat(2), charOffset: 6 }]) {
        const f = fixture();
        const start = { ...flat(), cellPath: [entry()] };
        assert.throws(() => new DeleteSelectionCommand(start, { ...end, cellPath: [entry(end.cellParaIndex)] }).execute(f.wasm));
        assert.equal(f.saves(), 0);
        assert.equal(f.mutations(), 0);
        assert.deepEqual(f.text(), ['alpha', 'beta', 'gamma']);
      }
    });
  } finally { await vite.close(); }
});
