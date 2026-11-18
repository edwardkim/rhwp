// #9165. Synthetic pages have independent faces so stale requests remain observable.
import assert from 'node:assert/strict';
import test from 'node:test';
import { createServer } from 'vite';

test('Canvas2D collects the edited portable page and invalidates uncertain page ownership', async () => {
  const vite = await createServer({ root: import.meta.dirname + '/..', logLevel: 'silent',
    server: { middlewareMode: true } });
  try {
    const { WasmBridge } = await vite.ssrLoadModule('/src/core/wasm-bridge.ts');
    const { setHostFontProvider } = await vite.ssrLoadModule('/src/core/local-fonts.ts');
    const names = ['Alpha', 'Bravo', 'Charlie', 'Delta'];
    await setHostFontProvider({
      getSnapshot: async () => ({ revision: '1', faces: names.map(family => ({
        id: family, family, style: 'Regular', fullName: family, postscriptName: family,
        weight: 400, slant: 'normal',
      })) }),
      readFace: async () => { throw Error('Discovery must not read bytes'); },
      subscribe: () => () => {},
    });
    const bridge = new WasmBridge();
    const pages = names.slice(0, 3);
    const visited: number[] = [];
    let active = false;
    Object.defineProperty(bridge, 'pageCount', {get: () => pages.length});
    bridge.canvasMetricDocument = () => ({
      canvasMetricsActive: () => active, selectCanvasMetrics: (enabled: boolean) => { active = enabled; },
    });
    bridge.getPageLayerTreeObject = (page: number) => {
      visited.push(page);
      return { root: {kind: 'leaf', ops: [
        { type: 'textRun', text: 'A', style: {fontFamily: pages[page]} },
      ]} };
    };
    function check(page: number | null, expected: number[], generation = 1) {
      visited.length = 0;
      const records = bridge.collectDocumentHostFontRequests(null, generation, page);
      assert.deepEqual(visited, expected);
      assert.deepEqual(records.map((r: {family: string}) => r.family), [...new Set(pages)]);
    }
    check(null, [0, 1, 2]);
    pages[1] = 'Delta';
    check(1, [1]);
    pages.pop();
    check(1, [0, 1]);
    check(0, [0, 1], 2);
    bridge.invalidateCanvasMetricFonts();
    check(0, [0, 1], 2);
    active = true;
    check(0, [0, 1], 2);
    check(1, [1], 2);
    assert.equal(active, true);
    active = false;
    bridge._documentGeneration++;
    check(1, [0, 1], 2);
    check(null, [0, 1], 2);
    await setHostFontProvider(null);
  } finally {
    await vite.close();
  }
});
