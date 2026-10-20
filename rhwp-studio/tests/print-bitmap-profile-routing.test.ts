import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

// 실제 PageRenderer 진입점의 프로필 선택을 검사한다. paint·DOM 합성은 별도
// 한컴 Print 대조군의 실제 브라우저 검사와 연결하며 이 대역으로 대신하지 않는다.
let vite: Awaited<ReturnType<typeof createServer>>;
let PageRenderer: any;
before(async () => {
  vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  ({ PageRenderer } = await vite.ssrLoadModule('/src/view/page-renderer.ts'));
});
after(async () => { await vite?.close(); });

for (const profile of ['screen', 'print', 'high-quality', 'fast-preview'] as const) {
  test(`${profile}: 화면 전용 DOM 질의와 프로필별 static paint를 구분한다`, () => {
    const paints: Array<{ filter: string; profile: string }> = [];
    const wasm = {
      renderPageToCanvasFiltered(_page: number, _canvas: unknown, _scale: number,
        filter: string, actualProfile: string) {
        paints.push({ filter, profile: actualProfile });
      },
    };
    const renderer = new PageRenderer(wasm, 'canvas2d', profile);
    const summary = {
      hasBehind: false, hasFront: false, imageCount: 1, rawSvgCount: 0,
      flowImageCount: 1, flowRawSvgCount: 0, flowStaticCount: 1,
      flowStaticOccluded: false, pageBackgroundComplex: false,
      pageBackgroundCss: '#fff', signature: 'source-owned-image',
    };
    let screenQueries = 0;
    const overlay = { images: [] as unknown[], calls: 0 };
    let retryImages = -1;
    renderer.getLayerPlaneSummary = () => summary;
    renderer.getFlowImagePaintOps = () => {
      screenQueries++;
      return [{ sourceImageKey: 'bin:0:1:src' }];
    };
    renderer.drawMarginGuides = () => {};
    renderer.applyOverlays = (...args: any[]) => {
      overlay.images = args[7];
      overlay.calls++;
      return summary;
    };
    renderer.rememberLayerPlaneSummary = () => {};
    renderer.scheduleReRender = (_page: number, _canvas: unknown, _scale: number, images: number) => {
      retryImages = images;
    };
    renderer.renderPage(0, { parentElement: {} }, 1, 1, 1);
    assert.deepEqual(paints, [{ filter: 'flow-dynamic', profile }],
      '동적 본문도 요청한 프로필을 사용한다');
    assert.equal(screenQueries, profile === 'screen' ? 1 : 0,
      '화면 전용 질의가 Print 색·표본 정보를 덮어쓰면 안 된다');
    assert.equal(overlay.calls, 1, '페이지 overlay 합성을 실제로 요청한다');
    assert.equal(overlay.images.length, profile === 'screen' ? 1 : 0,
      '화면 이외의 그림은 같은 프로필의 static canvas가 합성한다');
    assert.equal(retryImages, profile === 'screen' ? 0 : 1,
      'static canvas의 실제 비동기 그림 준비·재렌더를 유지한다');
  });
}

for (const [profile, scale, expectedWidth, expectedHeight] of [
  ['print', 1, 794, 1122],
  ['print', 2, 1587, 2243],
  ['screen', 1, 794, 1123],
  ['screen', 2, 1588, 2245],
] as const) {
  test(`${profile} ${scale}배: CanvasKit 표면은 발행된 paint 종이 크기를 소비한다`, () => {
    // 같은 A4의 저장 조판 상자와 독립 Print595×841pt 종이는 다르다.
    // 실제 Canvas2D/CanvasKit 캡처에서 발견한1122/1123px 차이를 보호한다.
    const pageInfo = { width: 793.7, height: 1122.5 };
    const tree = {
      pageWidth: profile === 'print' ? 595 * 96 / 72 : pageInfo.width,
      pageHeight: profile === 'print' ? 841 * 96 / 72 : pageInfo.height,
    };
    const renderer = new PageRenderer({
      getPageInfo: () => pageInfo,
      getPageLayerTreeObject: () => tree,
    }, 'canvaskit', profile);
    const canvas = { width: 0, height: 0, parentElement: null };
    let calls = 0;
    renderer.canvaskitRenderer = {
      renderPage(actualTree: unknown, target: unknown, actualScale: number, guideInfo: unknown) {
        calls++;
        assert.equal(actualTree, tree);
        assert.equal(target, canvas);
        assert.equal(actualScale, scale);
        assert.equal(guideInfo, pageInfo, '편집·여백 표시의 원본 조판 정보는 유지한다');
        return canvas;
      },
      diagnostics: () => ({}),
      recordRenderFailure: () => assert.fail('정상 paint 종이에서 fallback하지 않는다'),
    };
    renderer.cancelReRender = () => {};
    assert.equal(renderer.renderPageCanvasKit(0, canvas, scale), canvas);
    assert.equal(calls, 1);
    assert.deepEqual([canvas.width, canvas.height], [expectedWidth, expectedHeight]);
  });
}
