import type { CanvasView } from '../view/canvas-view';
import type { Ruler } from '../view/ruler';
import type { WasmBridge } from '../core/wasm-bridge';
import type { EventBus } from '../core/event-bus';
import type { PageInfo } from '../core/types';
import type { RenderSurfaceDecision } from '../view/render-surface-budget';
import { clampRenderScale } from '../view/render-backend';
import { currentImageRequest, flowImageState, imageCompletion, observeBoundary, ScrollObservation, surfacePixels, viewportApplied,
  type BoundaryObservation, type ObservedViewport } from './scroll-observation';

/** #6042 Stage 1 관찰 adapter. 제품 DPR/geometry/queue/renderer 결과를 수정하지 않는다. */
interface ProbeView {
  pages: PageInfo[];
  editingPageIndex: number | null;
  currentVisiblePages: number[];
  currentRetainedPages: number[];
  rendererSelectionEpoch: number;
  renderSurfaceDecisions: Map<number, RenderSurfaceDecision>;
  canvasPool: {
    activePages: number[];
    available: HTMLCanvasElement[];
    getCanvas(page: number): HTMLCanvasElement | undefined;
  };
  pageSurfaceLru: {
    takeLookup(key: string): unknown;
    put(entry: unknown): boolean;
    snapshot(): {
      reservedPixels: number;
      cachedPixels: number;
      totalAccountedPixels: number;
      overBudgetMandatory: boolean;
      entryCount: number;
      hits: number;
      misses: number;
      evictions: number;
      rejected: number;
      invalidations: number;
    };
  };
  pageRenderScheduler: {
    setDesiredWork(...args: unknown[]): void;
    snapshot(): {
      generation: number;
      visibleQueued: number;
      prefetchQueued: number;
      frameScheduled: boolean;
      idleScheduled: boolean;
      visibleSlices: number;
      visibleExecuted: number;
      prefetchExecuted: number;
      prefetchAdmissionRejected: number;
      staleDropped: number;
      maxQueueDepth: number;
    };
  };
  pageRenderer: {
    reRenderJobs: Map<number, unknown>;
    prefetchedImageSignatures: Map<number, { documentGeneration: number }>;
    prefetchRequestTokens: Map<number, number>;
  };
}

const FIXTURES = [
  ['exam_kor (필수)', 'exam_kor.hwp'], ['hwpspec (100+쪽)', 'hwpspec.hwp'],
  ['kps-ai', 'kps-ai.hwp'], ['KTX 4-layer', 'basic/KTX.hwp'],
  ['4쪽 실문서', '21868765_별표2_보건소_분장사무.hwp'],
  ['21쪽 다중 레이어', 'issue6280/156742029_prosecutor_transfer_list.hwp'],
] as const;

export function installPageScrollProbe(
  canvasView: CanvasView, ruler: Ruler, wasm: WasmBridge, events: EventBus,
  loadFixture: (path: string) => Promise<void>,
): () => void {
  const cv = canvasView as unknown as ProbeView;
  const vm = canvasView.getViewportManager();
  const vs = canvasView.getVirtualScroll();
  const trace = new ScrollObservation();
  let restores: (() => void)[] = [];
  let enabled = false;
  let batch = false;
  let disposed = false;
  let frameId = 0;
  let activeAt = 0;
  let activeScope = '';
  let settledFrames = 0;
  let geometry: { zoom: number; at: number } | null = null;
  let rulerFrame: { zoom: number; at: number } | null = null;
  const errors: { at: number; boundary: string; message: string }[] = [];
  const longTasks: { at: number; ms: number }[] = [];
  const images = new Map<number, { scope: string; kind: 'none' | 'pending' | 'decoded' | 'cached' | 'failed' }>();
  const flowImages = new Map<number, { layer: HTMLElement; images: HTMLImageElement[] }>();
  let contentEpoch = 0;
  const now = () => performance.now();
  const scope = () => `${wasm.documentGeneration}/${cv.rendererSelectionEpoch}/${contentEpoch}`;
  const viewport = (): ObservedViewport => ({ scope: scope(), zoom: vm.getZoom(), x: vm.getScrollX(), y: vm.getScrollY() });
  let applied: ObservedViewport | null = null;
  // 동일 runner에서 off/on 모두 구독한다. 원 메서드 복원 상태에서도 scroll rAF ack를 기다려야 한다.
  const subscriptions = ['viewport-scroll', 'viewport-resize', 'zoom-changed', 'document-view-loaded', 'page-view-settings-changed']
    .map(event => events.on(event, () => { applied = viewport(); }));
  subscriptions.push(events.on('document-page-invalidated', () => {
    contentEpoch++; applied = viewport(); images.clear(); flowImages.clear();
    trace.finish('interrupted', 'content invalidated');
  }));
  const diagnostic = (error: unknown, boundary = 'observer') => {
    if (errors.length === 128) errors.shift();
    errors.push({ at: now(), boundary, message: String(error) });
  };
  const observer = typeof PerformanceObserver !== 'undefined'
    && PerformanceObserver.supportedEntryTypes.includes('longtask')
    ? new PerformanceObserver(list => {
      if (!enabled || trace.id === null) return;
      for (const e of list.getEntries()) {
        if (e.startTime < activeAt) continue;
        if (longTasks.length === 256) longTasks.shift();
        longTasks.push({ at: e.startTime, ms: e.duration });
      }
    }) : null;
  observer?.observe({ type: 'longtask' });

  // cached state/dataset only. No page-info/render-tree query or DOM geometry scan in hot paths.
  const rendered = (page: number) => {
    const canvas = cv.canvasPool.getCanvas(page);
    const info = cv.pages[page];
    const decision = cv.renderSurfaceDecisions.get(page);
    if (!canvas || !info || !decision || !canvas.parentElement) return false;
    const scale = clampRenderScale(info, vm.getZoom() * decision.effectiveDpr);
    return Math.abs(Number(canvas.dataset.rhwpRenderScale) - scale) < 0.000001;
  };
  const pending = (page: number) => cv.pageRenderer.reRenderJobs.has(page);
  const flowState = (page: number) => {
    const entry = flowImages.get(page);
    return entry?.layer.isConnected ? flowImageState(entry.images) : 'ready';
  };
  const imageState = (page: number) => imageCompletion(images.get(page)?.scope === scope() ? images.get(page)?.kind : undefined);
  const stable = () => viewportApplied(viewport(), applied) && !vm.isZoomAnimating() && cv.currentVisiblePages.length > 0
    && cv.currentRetainedPages.every(p => rendered(p) && !pending(p))
    && cv.pageRenderScheduler.snapshot().visibleQueued === 0
    && cv.pageRenderScheduler.snapshot().prefetchQueued === 0;

  const inspectMilestones = () => {
    const id = trace.id;
    if (id === null || scope() !== activeScope) return;
    const at = now();
    if (vm.isZoomAnimating() || !viewportApplied(viewport(), applied) || !geometry) return;
    const visible = cv.currentVisiblePages;
    if (visible.some(rendered)) trace.mark(id, 'visibleFirst', at);
    const ready = (page: number) => rendered(page) && !pending(page) && flowState(page) === 'ready'
      && imageState(page) !== 'pending';
    const image = (page: number) => images.get(page);
    const failed = cv.currentRetainedPages.some(p => flowState(p) === 'failed'
      || (image(p)?.scope === activeScope && image(p)?.kind === 'failed'));
    if (failed && stable()) {
      trace.finish('interrupted', 'image-prefetch-failed/fallback: sharp completion unproven');
      return;
    }
    const focus = cv.editingPageIndex;
    const knownImageReady = (page: number) => image(page)?.scope === activeScope
      && ['none', 'decoded', 'cached'].includes(image(page)!.kind);
    if (focus !== null && visible.includes(focus) && ready(focus) && knownImageReady(focus)) {
      trace.mark(id, 'focusedSharp', at);
    }
    // These two are known-render-work boundaries, not compositor presentation timestamps.
    if (visible.length && visible.every(ready)) trace.mark(id, 'visibleStable', at);
    if (stable() && cv.currentRetainedPages.every(ready)) trace.mark(id, 'retainedComplete', at);
  };

  const tick = () => {
    frameId = 0;
    if (disposed || !enabled || trace.id === null) return;
    if (scope() !== activeScope) { trace.finish('interrupted', 'document/revision changed'); return; }
    trace.frame(trace.id, now());
    inspectMilestones();
    if (trace.id === null) return;
    settledFrames = stable() && geometry && cv.currentRetainedPages.every(p => flowState(p) === 'ready'
      && imageState(p) !== 'pending') ? settledFrames + 1 : 0;
    if (settledFrames >= 2) {
      trace.finish('complete', 'known render work only; presentation and decoder completeness are separate evidence');
    } else if (now() - activeAt > 12_000) {
      trace.finish('timeout', 'pending work after 12s');
    } else frameId = requestAnimationFrame(tick);
  };
  const begin = (source: string) => {
    if (!enabled) return;
    if (frameId) cancelAnimationFrame(frameId);
    activeAt = now(); activeScope = scope(); settledFrames = 0; geometry = null; rulerFrame = null;
    trace.begin(activeScope, source, activeAt);
    frameId = requestAnimationFrame(tick);
  };

  const observe = (name: string, call: BoundaryObservation) => {
    const page = /^(wasm\.|raster\.|page\.|pool\.|image\.)/.test(name) && typeof call.args[0] === 'number' ? call.args[0] : null;
    const units = name === 'visibility.snapshot'
      ? Number((call.result as { queryStats?: { pagesExamined?: number } } | null)?.queryStats?.pagesExamined ?? 0)
      : 0;
    trace.count(call.token, name, call.startedAt, call.endedAt, units, page);
    if (call.failed) { diagnostic(call.error, name); trace.finish('interrupted', name); }
    if (name === 'document.begin' || name === 'document.blank' || name === 'document.refresh') {
      contentEpoch++;
      images.clear(); flowImages.clear();
      trace.finish('interrupted', name);
      if (name === 'document.refresh') applied = viewport();
    }
    if (name === 'geometry.zoom' || name === 'visibility.update') {
      geometry = { zoom: vm.getZoom(), at: call.endedAt };
    }
    if (name === 'ruler.update') rulerFrame = { zoom: vm.getZoom(), at: call.endedAt };
    // 두 rAF callback 사이에서 일치한 preview를 다음 observer rAF가 놓치지 않도록 경계에서 기록한다.
    if (geometry && rulerFrame && Math.abs(geometry.zoom - rulerFrame.zoom) < 0.000001) {
      trace.mark(call.token, 'preview', Math.max(geometry.at, rulerFrame.at));
    }
    if (name === 'image.flowLayer' && page !== null && call.result instanceof HTMLElement) {
      if (flowImages.size >= 512 && !flowImages.has(page)) flowImages.delete(flowImages.keys().next().value!);
      // DOM 열거는 layer 생성 경계에만 한정한다. rAF는 보유 참조의 complete/naturalWidth만 읽는다.
      flowImages.set(page, { layer: call.result, images: [...call.result.querySelectorAll('img')] });
    }
    if (name === 'image.schedule' && page !== null) {
      if (images.size >= 512 && !images.has(page)) images.delete(images.keys().next().value!);
      const count = Number(call.args[3]);
      const cached = cv.pageRenderer.prefetchedImageSignatures.get(page)?.documentGeneration === wasm.documentGeneration;
      images.set(page, { scope: scope(), kind: count <= 0 ? 'none' : pending(page) ? 'pending' : cached ? 'cached' : 'pending' });
    }
    if (name === 'image.prefetch' && page !== null && call.result instanceof Promise) {
      const capturedScope = scope();
      const token = Number(call.args[2]);
      // 원 Promise를 반환한다. 추가 관찰 reaction의 비용은 A/B에서 별도로 확인한다.
      void call.result.then((decoded: unknown) => {
        if (disposed || !enabled || !currentImageRequest(scope(), capturedScope,
          cv.pageRenderer.prefetchRequestTokens.get(page), token)) return;
        images.set(page, { scope: capturedScope, kind: decoded ? 'decoded' : 'failed' });
      }, () => {
        if (!disposed && enabled && currentImageRequest(scope(), capturedScope,
          cv.pageRenderer.prefetchRequestTokens.get(page), token)) images.set(page, { scope: capturedScope, kind: 'failed' });
      });
    }
  };

  const boundaries: [object, string, string][] = [
    [vm, 'setZoom', 'zoom.set'], [vm, 'smoothZoomTo', 'zoom.smooth'],
    [vs, 'getVisibilitySnapshot', 'visibility.snapshot'],
    [cv, 'updateVisiblePages', 'visibility.update'], [cv, 'refreshRenderSurfacePlan', 'budget.refresh'],
    [cv, 'renderCanvas', 'raster.main'], [cv, 'renderPage', 'page.render'],
    [cv.pageSurfaceLru, 'takeLookup', 'cache.take'], [cv.pageSurfaceLru, 'put', 'cache.put'],
    [cv.pageRenderScheduler, 'setDesiredWork', 'scheduler.desired'],
    [cv, 'onZoomChanged', 'geometry.zoom'], [cv, 'prepareDocumentLoad', 'document.begin'],
    [cv, 'showBlankPage', 'document.blank'], [cv, 'refreshPages', 'document.refresh'],
    [cv.canvasPool, 'release', 'pool.release'], [cv.canvasPool, 'acquire', 'pool.acquire'],
    [cv.pageRenderer, 'reRenderPageCanvases', 'raster.image'],
    [cv.pageRenderer, 'scheduleReRender', 'image.schedule'],
    [cv.pageRenderer, 'prefetchLayerImages', 'image.prefetch'],
    [cv.pageRenderer, 'createOrReuseFlowImageLayer', 'image.flowLayer'],
    [wasm, 'renderPageToCanvasFiltered', 'wasm.layerRaster'],
    [wasm, 'getPageInfo', 'wasm.pageInfo'],
    [ruler, 'update', 'ruler.update'],
  ];
  const setEnabled = (next: boolean) => {
    if (next === enabled) return;
    trace.finish('interrupted', 'observation toggled');
    if (frameId) cancelAnimationFrame(frameId);
    frameId = 0;
    for (const restore of restores.reverse()) restore();
    restores = []; images.clear(); flowImages.clear(); enabled = false;
    if (next) {
      try {
        for (const [target, key, name] of boundaries) restores.push(observeBoundary(target, key, now,
          () => trace.id, call => observe(name, call), diagnostic));
        enabled = true;
      } catch (error) {
        for (const restore of restores.reverse()) restore();
        restores = []; diagnostic(error); throw error;
      }
    }
  };

  const panel = document.createElement('aside');
  panel.id = 'page-scroll-probe';
  panel.style.cssText = 'position:fixed;top:0;right:0;z-index:99999;background:white;color:#111;font:11px system-ui;padding:3px;border:1px solid #aaa;max-width:98vw';
  panel.innerHTML = `<label>기준선 문서 <select aria-label="기준선 문서">${FIXTURES.map(([name], i) => `<option value="${i}">${name}</option>`).join('')}</select></label>
    <button data-open>실문서 열기</button><label><input type="checkbox" aria-label="스크롤 관찰" checked>관찰</label>
    <select aria-label="검증 배치"><option value="single">한 쪽</option><option value="double">두 쪽</option><option value="four">네 열</option><option value="auto">자동</option></select>
    ${[34, 50, 100, 200].map(n => `<button data-zoom="${n}">${n}% 줌</button>`).join('')}
    <button data-top>처음으로</button><button data-step>다음 행</button><button data-bench>왕복 20회</button>
    <button data-overhead>관찰 비용 A/B</button><button data-read>관찰 결과</button><button data-clear>기록 초기화</button>
    <output aria-label="관찰 상태">준비</output><details><summary>Stage 1 JSON</summary><pre style="max-height:45vh;max-width:95vw;overflow:auto;user-select:text"></pre></details>`;
  document.body.append(panel);
  const status = (message: string) => { panel.querySelector('output')!.textContent = message; };
  const show = (value: unknown) => {
    panel.querySelector('pre')!.textContent = JSON.stringify(value, null, 2);
    panel.querySelector('details')!.open = true;
  };
  const snapshot = () => {
    const pages = cv.canvasPool.activePages.map(page => {
      const main = cv.canvasPool.getCanvas(page)!;
      const overlays = [...document.querySelectorAll<HTMLCanvasElement>(`canvas[data-rhwp-overlay-page="${page}"]`)];
      const surfaces = [main, ...overlays].map(c => ({ kind: c.dataset.rhwpLayerKind ?? 'main', width: c.width, height: c.height }));
      const box = main.getBoundingClientRect();
      return { page, visible: cv.currentVisiblePages.includes(page), dpr: Number(main.dataset.rhwpEffectiveDpr),
        scale: Number(main.dataset.rhwpRenderScale), surfaces, pixels: surfacePixels(surfaces),
        box: { x: box.x, y: box.y, width: box.width, height: box.height }, image: images.get(page) ?? null,
        flowImages: flowState(page) };
    });
    const pool = cv.canvasPool.available.map(c => ({ width: c.width, height: c.height }));
    const activePixels = pages.reduce((sum, p) => sum + p.pixels, 0);
    const cache = cv.pageSurfaceLru.snapshot();
    const scheduler = cv.pageRenderScheduler.snapshot();
    return { enabled, browser: navigator.userAgent, dpr: devicePixelRatio, viewport: { width: innerWidth, height: innerHeight },
      targetViewport: viewport(), appliedViewport: applied,
      pageCount: wasm.pageCount, layoutPageCount: cv.pages.length, scope: scope(), zoom: vm.getZoom(),
      columns: vs.getColumns(), focused: cv.editingPageIndex, visible: cv.currentVisiblePages, retained: cv.currentRetainedPages,
      scroll: { x: vm.getScrollX(), y: vm.getScrollY() }, pages, pool, activePixels, idlePoolPixels: surfacePixels(pool),
      detachedCache: cache,
      scheduler,
      totalAllocatedPixels: activePixels + surfacePixels(pool) + cache.cachedPixels,
      pendingImages: cv.pageRenderer.reRenderJobs.size,
      pendingPrefetch: scheduler.prefetchQueued, traces: trace.snapshot(), errors, longTasks,
      note: 'RGBA 환산=actual pixels×4; not GPU/RSS. Timings are known-render-work/next-frame opportunities, not compositor presentation. Focus sharp requires observed decode/no-image evidence. Bounds/readback only on explicit snapshot.' };
  };
  const nextFrame = () => new Promise<number>(resolve => requestAnimationFrame(() => resolve(now())));
  const waitStable = async () => {
    const start = now(); let frames = 0;
    while (now() - start < 12_000) {
      await nextFrame();
      frames = stable() ? frames + 1 : 0;
      if (frames >= 2) return;
    }
    throw new Error('known render work did not settle in 12s');
  };
  const rowStep = () => vs.getPageHeight(cv.currentVisiblePages[0] ?? 0) + vs.getPageGap();
  const move = (y: number) => { vm.setScrollTop(y); };
  const run = async (body: () => Promise<void>) => {
    if (batch) return;
    batch = true; status('진행 중');
    panel.querySelector('details')!.open = false;
    try { await body(); status('완료'); }
    catch (error) { diagnostic(error, 'scenario'); status(`실패: ${error}`); show(snapshot()); }
    finally { batch = false; }
  };
  panel.querySelector<HTMLButtonElement>('[data-open]')!.onclick = () => void run(async () => {
    trace.finish('interrupted', 'fixture change');
    const index = Number(panel.querySelector<HTMLSelectElement>('[aria-label="기준선 문서"]')!.value);
    await loadFixture(FIXTURES[index][1]); await waitStable();
  });
  panel.querySelector<HTMLInputElement>('[aria-label="스크롤 관찰"]')!.onchange = e => setEnabled((e.target as HTMLInputElement).checked);
  panel.querySelector<HTMLSelectElement>('[aria-label="검증 배치"]')!.onchange = e => {
    const kind = (e.target as HTMLSelectElement).value;
    events.emit('page-view-settings-changed', { arrangement: kind === 'four' ? { kind: 'multiple', columns: 4, rows: 2 } : { kind }, pageMovement: { direction: 'vertical', wheelHorizontal: false } });
  };
  for (const button of panel.querySelectorAll<HTMLButtonElement>('[data-zoom]')) button.onclick = () => {
    begin('probe-smooth-zoom'); vm.smoothZoomTo(Number(button.dataset.zoom) / 100);
  };
  panel.querySelector<HTMLButtonElement>('[data-top]')!.onclick = () => { begin('probe-scroll-top'); move(0); };
  panel.querySelector<HTMLButtonElement>('[data-step]')!.onclick = () => { begin('probe-scroll-row'); move(vm.getScrollY() + rowStep()); };
  panel.querySelector<HTMLButtonElement>('[data-read]')!.onclick = () => show(snapshot());
  panel.querySelector<HTMLButtonElement>('[data-clear]')!.onclick = () => { trace.clear(); errors.length = 0; longTasks.length = 0; status('기록 초기화'); };
  panel.querySelector<HTMLButtonElement>('[data-bench]')!.onclick = () => void run(async () => {
    const step = rowStep(); move(step * 2); await waitStable(); trace.clear();
    const samples = [];
    for (let round = 0; round < 20; round++) {
      const y = step * (round % 2 ? 2 : 5);
      begin(`scripted-scroll-${round}`); const start = now(); move(y); const syncMs = now() - start;
      await waitStable(); samples.push({ round, y: vm.getScrollY(), syncMs, knownWorkNextFrameMs: now() - start });
    }
    show({ kind: 'baseline-scroll', samples, evidence: snapshot() });
  });
  panel.querySelector<HTMLButtonElement>('[data-overhead]')!.onclick = () => void run(async () => {
    const previous = enabled; const step = rowStep(); const samples = [];
    try {
      for (let round = 0; round < 12; round++) {
        for (const on of round % 2 ? [true, false] : [false, true]) {
          setEnabled(on); move(step * 2); await waitStable();
          begin(`overhead-${round}-${on}`); const start = now(); move(step * 5); const syncMs = now() - start;
          await waitStable(); samples.push({ round, enabled: on, syncMs, knownWorkNextFrameMs: now() - start });
        }
      }
    } finally { setEnabled(previous); }
    show({ kind: 'observation-overhead', samples, evidence: snapshot(), note: 'first 2 rounds warm-up; off restores original methods; same scenario runner on both sides' });
  });
  const capture = (event: Event) => {
    if (!enabled || batch || panel.contains(event.target as Node)) return;
    if (event instanceof WheelEvent) begin(event.ctrlKey || event.metaKey ? 'user-zoom-wheel' : 'user-plain-wheel');
    else if (event instanceof KeyboardEvent && ['PageDown', 'PageUp'].includes(event.key)) begin('user-page-key');
  };
  document.addEventListener('wheel', capture, true);
  document.addEventListener('keydown', capture, true);
  setEnabled(true);
  return () => {
    setEnabled(false); disposed = true; observer?.disconnect(); panel.remove();
    for (const unsubscribe of subscriptions) unsubscribe();
    document.removeEventListener('wheel', capture, true); document.removeEventListener('keydown', capture, true);
  };
}
