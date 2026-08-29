import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

import type { PageInfo } from '../src/core/types.ts';
import {
  drawPageMarginGuideCorners,
  drawPageMarginGuides,
} from '../src/view/page-margin-guides.ts';

interface ContextCall {
  name: string;
  args: number[];
}

function recordingCanvas(): { canvas: HTMLCanvasElement; calls: ContextCall[] } {
  const calls: ContextCall[] = [];
  const record = (name: string, ...args: number[]): void => {
    calls.push({ name, args });
  };
  const context = {
    save: () => record('save'),
    restore: () => record('restore'),
    setTransform: (...args: number[]) => record('setTransform', ...args),
    beginPath: () => record('beginPath'),
    rect: (...args: number[]) => record('rect', ...args),
    clip: () => record('clip'),
    moveTo: (...args: number[]) => record('moveTo', ...args),
    lineTo: (...args: number[]) => record('lineTo', ...args),
    stroke: () => record('stroke'),
    strokeStyle: '',
    lineWidth: 0,
  } as unknown as CanvasRenderingContext2D;
  const canvas = {
    width: 1200,
    height: 1600,
    parentElement: {},
    getContext: (kind: string) => kind === '2d' ? context : null,
  } as unknown as HTMLCanvasElement;
  return { canvas, calls };
}

const pageInfo: PageInfo = {
  pageIndex: 0,
  width: 600,
  height: 800,
  sectionIndex: 0,
  marginLeft: 60,
  marginRight: 60,
  marginTop: 40,
  marginBottom: 40,
  marginHeader: 20,
  marginFooter: 20,
};

test('focused page patch clips margin-guide repaint to the page-space dirty rect', () => {
  const { canvas, calls } = recordingCanvas();
  const patch = { x: 80, y: 120, width: 240, height: 32 };

  drawPageMarginGuides(pageInfo, canvas, 2, patch);

  assert.deepEqual(calls[0], { name: 'save', args: [] });
  assert.deepEqual(calls[1], { name: 'setTransform', args: [2, 0, 0, 2, 0, 0] });
  assert.deepEqual(calls.find((call) => call.name === 'rect'), {
    name: 'rect',
    args: [patch.x, patch.y, patch.width, patch.height],
  });
  const clipIndex = calls.findIndex((call) => call.name === 'clip');
  const strokeIndex = calls.findIndex((call) => call.name === 'stroke');
  assert.ok(clipIndex >= 0 && clipIndex < strokeIndex, 'dirty rect clip must precede guide stroke');
  assert.equal(calls.filter((call) => call.name === 'stroke').length, 1);
  assert.deepEqual(calls.at(-1), { name: 'restore', args: [] });
});

test('full page render keeps the existing unclipped margin-guide path', () => {
  const { canvas, calls } = recordingCanvas();

  drawPageMarginGuides(pageInfo, canvas, 1);

  assert.equal(calls.some((call) => call.name === 'rect'), false);
  assert.equal(calls.some((call) => call.name === 'clip'), false);
  assert.equal(calls.filter((call) => call.name === 'stroke').length, 1);
});

test('HF band can reuse the exact page-margin corner path', () => {
  const page = recordingCanvas();
  const band = recordingCanvas();

  drawPageMarginGuides(pageInfo, page.canvas, 1);
  drawPageMarginGuideCorners({ x: 60, y: 60, width: 480, height: 680 }, band.canvas, 1);

  const pathCalls = (calls: ContextCall[]) => calls.filter(
    (call) => call.name === 'moveTo' || call.name === 'lineTo',
  );
  assert.deepEqual(pathCalls(band.calls), pathCalls(page.calls));
});

test('PageRenderer forwards the focused patch to the margin-guide clip', async () => {
  const studioRoot = fileURLToPath(new URL('..', import.meta.url));
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { PageRenderer } = await vite.ssrLoadModule('/src/view/page-renderer.ts');
    const { canvas, calls } = recordingCanvas();
    const patch = { pageIndex: 0, x: 80, y: 120, width: 240, height: 32 };
    const wasm = {
      getPageInfo: () => pageInfo,
      getPageOverlayImages: () => JSON.stringify({
        hasBehind: false,
        hasFront: false,
        imageCount: 0,
        rawSvgCount: 0,
        flowImageCount: 0,
        flowRawSvgCount: 0,
        behind: [],
        front: [],
      }),
      renderPagePatchToCanvasFiltered: () => undefined,
    };

    const renderer = new PageRenderer(wasm);
    const result = renderer.renderPage(0, canvas, 2, 1, 1, {
      reason: 'text-edit',
      allowStaticOverlayReuse: true,
      focusedPagePatch: patch,
    });

    assert.equal(result.needsTextEditStaticLayerVerification, false);
    assert.deepEqual(calls.find((call) => call.name === 'rect'), {
      name: 'rect',
      args: [patch.x, patch.y, patch.width, patch.height],
    });
  } finally {
    await vite.close();
  }
});
