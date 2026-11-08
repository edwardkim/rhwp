/** SDK 캐럿 동기화가 중첩 셀의 부모 문단 소유와 Undo를 보존한다(#7713). */
import { resolve } from 'node:path';
import { runTest, assert } from './helpers.mjs';

const modulePath = resolve(import.meta.dirname, '../../npm/editor/index.js').replace(/\\/g, '/');
const moduleUrl = modulePath.startsWith('/') ? `/@fs${modulePath}` : `/@fs/${modulePath}`;
const viteUrl = process.env.VITE_URL || 'http://localhost:7700';

runTest('중첩 셀 캐럿 소유', async ({ page }) => {
  await page.goto(`${viteUrl}/@vite/client`, { waitUntil: 'domcontentloaded' });
  const result = await page.evaluate(async (url) => {
    const { createEditor } = await import(url);
    const host = document.createElement('div');
    document.body.appendChild(host);
    const editor = await createEditor(host, { studioUrl: `${location.origin}/`, handshakeTimeoutMs: 30_000 });
    try {
      const bytes = await fetch('/samples/issue1949_giant_cell_nested_tables_perf.hwp').then(r => r.arrayBuffer());
      await editor.loadFile(bytes, 'issue1949_giant_cell_nested_tables_perf.hwp');
      await editor.plugins.load('hwpctrl');
      const win = editor.element.contentWindow;
      const path = [
        { controlIndex: 2, cellIndex: 2, cellParaIndex: 288 },
        { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
      ];
      // 클릭 좌표를 고정하지 않고, 입력 처리기의 유효한 셀 소유 경로를 캐럿 출발점으로 둔다.
      win.__inputHandler.cursor.moveTo({
        sectionIndex: 0, paragraphIndex: 0, parentParaIndex: 0,
        controlIndex: 2, cellIndex: 2, cellParaIndex: 0,
        cellPath: path, charOffset: 0,
      });
      const before = win.rhwpStudio.automation.getContext().canUndo;
      const at = await editor.hwpctrl.syncCursorFromCaret();
      const afterSync = win.rhwpStudio.automation.getContext().canUndo;
      const created = await editor.hwpctrl.call('CreateField', ['안내', '', '중첩소유검사']);
      const field = win.__wasm.getFieldList().find(f => f.name === '중첩소유검사');
      await editor.hwpctrl.undo();
      const remains = win.__wasm.getFieldList().some(f => f.name === '중첩소유검사');
      return { at, before, afterSync, created, field, remains };
    } finally {
      editor.destroy();
    }
  }, moduleUrl);
  console.log(JSON.stringify(result));
  assert(result.at?.list === 12 && result.at?.para === 0, 'TC1: 부모 문단 288의 안쪽 리스트 12');
  assert(result.before === result.afterSync, 'TC2: 동기화만으로 Undo 생성 안 함');
  assert(result.created && result.field?.location.path[0]?.paraIndex === 288,
    `TC3: CreateField의 실제 소유 문단 (${JSON.stringify(result.field?.location)})`);
  assert(!result.remains, 'TC4: Undo 한 번으로 새 필드 제거');
}, { skipLoadApp: true });
