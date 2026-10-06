/** #7388: 한컴 저장 높이 0 도형의 실제 WASM 리사이즈와 Studio undo 명령. */
import { runTest, setTestCase, assert } from './helpers.mjs';

runTest('#6806 저장 높이 0 도형의 리사이즈 undo', async ({ page }) => {
  setTestCase('한컴 저장 0 → 확대 400 → undo 0 → redo 400');
  const result = await page.evaluate(async () => {
    const filename = 'issue6023/30269_reform_recommendation.hwp';
    const response = await fetch(`/samples/${filename}`);
    if (!response.ok) throw new Error(`표본 fetch 실패: HTTP ${response.status}`);
    const bytes = new Uint8Array(await response.arrayBuffer());
    window.__wasm.loadDocument(bytes, filename);
    await window.__canvasView.loadDocument();

    const { ResizeObjectCommand } = await import('/src/engine/command.ts');
    const readHeight = () => window.__wasm.getShapeProperties(0, 28, 0).height;
    const original = window.__wasm.getShapeProperties(0, 28, 0);
    const before = { width: original.width, height: original.height };
    const after = { width: original.width, height: 400 };
    const command = new ResizeObjectCommand([{
      sec: 0, ppi: 28, ci: 0, type: 'shape', before, after,
    }]);

    window.__wasm.setShapeProperties(0, 28, 0, after);
    const expanded = readHeight();
    command.undo(window.__wasm);
    const undone = readHeight();
    command.execute(window.__wasm);
    const redone = readHeight();
    return { original: original.height, expanded, undone, redone };
  });

  assert(result.original === 0, `한컴 저장 높이 0 전제: ${result.original}`);
  assert(result.expanded === 400, `실제 WASM 확대 결과: ${result.expanded}`);
  assert(result.undone === 0, `Studio undo 복원 결과: ${result.undone}`);
  assert(result.redone === 400, `Studio redo 재적용 결과: ${result.redone}`);
});
