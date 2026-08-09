import init, { HwpDocument } from "@rhwp-wasm/rhwp.js";
import { blake3 } from "@noble/hashes/blake3.js";
import { bytesToHex } from "@noble/hashes/utils.js";

import {
  parseCanvasKitDocumentPreflight,
  withCanvasKitSurfaceBlockers,
} from "@/core/canvaskit-document-preflight";
import { resolveCanvasKitFontPlan } from "@/core/font-loader";
import type { PageLayerTree } from "@/core/types";
import {
  RendererSession,
  type RendererSessionSelection,
} from "@/view/renderer-session";

// WASM 렌더러가 호출하는 텍스트 폭 측정 콜백 등록
installMeasureTextWidth();

// VSCode Webview API
const vscode = acquireVsCodeApi();

// DOM 요소
const scrollContainer = document.getElementById("scroll-container")!;
const scrollContent = document.getElementById("scroll-content")!;
const stbPage = document.getElementById("stb-page")!;
const stbMessage = document.getElementById("stb-message")!;
const stbZoomLabel = document.getElementById("stb-zoom-label")!;
const stbZoomMenu = document.getElementById("stb-zoom-menu")!;
const stbZoomPopup = document.getElementById("stb-zoom-popup")!;
const stbZoomOut = document.getElementById("stb-zoom-out")!;
const stbZoomIn = document.getElementById("stb-zoom-in")!;

// 사이드바 요소
const appShell = document.getElementById("app-shell")!;
const navSidebar = document.getElementById("nav-sidebar")!;
const navResizer = document.getElementById("nav-resizer")!;
const navCollapse = document.getElementById("nav-collapse")!;
const navReopen = document.getElementById("nav-reopen")!;
const navTabs = Array.from(document.querySelectorAll<HTMLButtonElement>(".nav-tab"));
const navPanels = new Map<string, HTMLElement>(
  Array.from(document.querySelectorAll<HTMLElement>(".nav-panel")).map((el) => [
    el.dataset.panel!,
    el,
  ])
);
const stbSidebarToggle = document.getElementById("stb-sidebar-toggle")!;

interface ViewerState {
  sidebarWidth?: number;
  collapsedOutlineKeys?: string[];
}

const savedViewerState = (vscode.getState() as ViewerState | undefined) ?? {};
const MIN_SIDEBAR_WIDTH = 180;
const MAX_SIDEBAR_WIDTH = 520;
let sidebarWidth = Math.max(
  MIN_SIDEBAR_WIDTH,
  Math.min(MAX_SIDEBAR_WIDTH, savedViewerState.sidebarWidth ?? 240)
);
const collapsedOutlineKeys = new Set(savedViewerState.collapsedOutlineKeys ?? []);

// 문서 상태
type ZoomMode = "manual" | "fitWidth" | "fitPage";

let hwpDoc: HwpDocument | null = null;
let pageInfos: PageInfo[] = [];
/** 실제 적용 중인 배율. 맞춤 모드에서도 계산된 값이 들어간다. */
let currentZoom = 1.0;
let zoomMode: ZoomMode = "manual";
let currentPage = 0;
let viewMode: "single" | "double" = "single";
let fileName = "";
let documentLoadGeneration = 0;
let rendererSelection: RendererSessionSelection | null = null;
let rendererFallbackScheduled = false;
let outlineHighlight: HTMLDivElement | null = null;
/** 상태 표시줄의 쪽 번호 입력 모드 여부. 초기 레이아웃 변경보다 먼저 초기화한다. */
let pageInputActive = false;
const PREFETCH_MARGIN = 300;
const ZOOM_STEP = 0.1;
const ZOOM_MIN = 0.25;
const ZOOM_MAX = 3.0;
/** .page-row 의 CSS gap 과 일치해야 한다. */
const ROW_GAP = 12;
/** #scroll-container 의 세로 padding 과 일치해야 한다. */
const CONTENT_PADDING = 12;
/** 맞춤 배율에서 쪽 좌우로 남겨 두는 여백. */
const SIDE_MARGIN = 12;

const canvasKitDefaultFontUri = scrollContainer.dataset.canvaskitFontUri ?? "";
const canvasKitFontsBaseUri = scrollContainer.dataset.canvaskitFontsBaseUri ?? "";
const vscodeBundledFontFiles = new Set([
  "NotoSerifKR-Regular.woff2",
  "NotoSerifKR-Bold.woff2",
  "NotoSansKR-Regular.woff2",
  "NotoSansKR-Bold.woff2",
  "NotoSansKR-ExtraLight.woff2",
  "Pretendard-Regular.woff2",
  "Pretendard-Bold.woff2",
  "D2Coding-Regular.woff2",
  "NanumGothic-Regular.woff2",
  "NanumMyeongjo-Regular.woff2",
  "GowunBatang-Regular.woff2",
  "GowunDodum-Regular.woff2",
]);
const canvasKitFontPlan = (requiredFontFamilies: readonly string[]) => resolveCanvasKitFontPlan(
  requiredFontFamilies,
  {
    localFontBaseUrl: canvasKitFontsBaseUri,
    availableLocalFiles: vscodeBundledFontFiles,
    disableExternalWebFonts: true,
  },
);
const rendererSession = new RendererSession(
  { backend: "canvas2d", source: "default" },
  { mode: "default", source: "default" },
  { preference: "auto", requested: "auto" },
  "screen",
  async (mode, surface) => {
    const { CanvasKitLayerRenderer } = await import("@/view/canvaskit-renderer");
    return CanvasKitLayerRenderer.create(
      mode,
      surface,
      {
        ...(canvasKitDefaultFontUri ? { defaultFontUrl: canvasKitDefaultFontUri } : {}),
        requirePreparedFontFamilies: true,
      },
    );
  },
  {
    transformCanvasKitPreflight(report) {
      const plan = canvasKitFontPlan(report.requiredFontFamilies);
      return withCanvasKitSurfaceBlockers(
        report,
        plan.unavailableFonts.map(font => `fontUnavailable:${font}`),
      );
    },
    async prepareCanvasKitDocument(renderer, report) {
      const plan = canvasKitFontPlan(report.requiredFontFamilies);
      if (plan.unavailableFonts.length > 0) {
        throw new Error(`CanvasKit font family가 준비되지 않았습니다: ${plan.unavailableFonts.join(", ")}`);
      }
      await renderer.prepareBundledFonts(plan.sources);
    },
  },
);

interface PageInfo {
  width: number;
  height: number;
  rendered: boolean;
  element: HTMLDivElement | null;
}

// WASM 초기화
let wasmReady = false;
const wasmUri = scrollContainer.dataset.wasmUri!;

stbMessage.textContent = "WASM 초기화 중...";
fetch(wasmUri)
  .then((res) => res.arrayBuffer())
  .then(async (buf) => {
    // 동기 initSync는 메인 스레드에서 new WebAssembly.Module()을 실행하여
    // macOS 웹뷰에서 "4KB 초과 버퍼의 메인 스레드 컴파일 금지" 규칙에 차단된다.
    // async init(instantiate 기반)으로 초기화하여 전 플랫폼에서 동작하도록 한다. (#2048)
    await init({ module_or_path: buf });
    wasmReady = true;
    stbMessage.textContent = "문서를 기다리는 중...";
    vscode.postMessage({ type: "ready" });
  })
  .catch((err) => {
    stbMessage.textContent = `WASM 로드 실패: ${err.message ?? err}`;
  });

// Extension Host로부터 HWP 파일 데이터 수신
window.addEventListener("message", (event) => {
  const msg = event.data;

  if (msg.type === "load") {
    void loadDocument(msg);
    return;
  }

  if (msg.type === "exportSvg") {
    if (!hwpDoc) {
      vscode.postMessage({ type: "exportSvgDone", error: "문서가 로드되지 않았습니다" });
      return;
    }
    try {
      const svgs: string[] = [];
      for (let i = 0; i < pageInfos.length; i++) {
        svgs.push(hwpDoc.renderPageSvg(i));
      }
      vscode.postMessage({ type: "exportSvgDone", svgs });
    } catch (err: any) {
      vscode.postMessage({ type: "exportSvgDone", error: err.message ?? String(err) });
    }
  }

  if (msg.type === "exportDebugOverlay") {
    if (!hwpDoc) {
      vscode.postMessage({ type: "debugOverlaySvgs", error: "문서가 로드되지 않았습니다" });
      return;
    }
    try {
      hwpDoc.set_debug_overlay(true);
      const svgs: string[] = [];
      for (let i = 0; i < pageInfos.length; i++) {
        svgs.push(hwpDoc.renderPageSvg(i));
      }
      hwpDoc.set_debug_overlay(false);
      vscode.postMessage({ type: "debugOverlaySvgs", svgs });
    } catch (err: any) {
      hwpDoc.set_debug_overlay(false);
      vscode.postMessage({ type: "debugOverlaySvgs", error: err.message ?? String(err) });
    }
  }
});

async function loadDocument(msg: { fileName: string; fileData: unknown }): Promise<void> {
  if (!wasmReady) {
    stbMessage.textContent = "오류: WASM이 아직 초기화되지 않았습니다";
    return;
  }

  const generation = ++documentLoadGeneration;
  let nextDocument: HwpDocument | null = null;
  try {
    fileName = msg.fileName;
    stbMessage.textContent = `${fileName} 로딩 중...`;
    releaseRenderedDocument();
    const previousDocument = hwpDoc;
    hwpDoc = null;
    rendererSelection = null;
    delete document.documentElement.dataset.rendererBackend;
    delete document.documentElement.dataset.rendererDecisionKey;
    previousDocument?.free();

    const fileBytes = toUint8Array(msg.fileData);
    const digest = `blake3:${bytesToHex(blake3(fileBytes))}`;
    rendererSession.beginDocument(digest);
    nextDocument = new HwpDocument(fileBytes);
    nextDocument.setClipEnabled(false);

    hwpDoc = nextDocument;

    const selection = await rendererSession.resolve({
      getCanvasKitDocumentPreflight(mode, profile) {
        return parseCanvasKitDocumentPreflight(
          nextDocument!.getCanvasKitDocumentPreflight(mode, profile),
          "[VS Code] CanvasKit document preflight",
        );
      },
    });
    if (
      generation !== documentLoadGeneration
      || hwpDoc !== nextDocument
      || !rendererSession.isCurrent(selection)
    ) return;
    applyRendererSelection(selection);

    const docInfo = JSON.parse(nextDocument.getDocumentInfo());
    const pageCount: number = docInfo.page_count ?? docInfo.pageCount ?? 0;

    pageInfos = [];
    for (let i = 0; i < pageCount; i++) {
      const pi = JSON.parse(nextDocument.getPageInfo(i));
      pageInfos.push({
        width: pi.width,
        height: pi.height,
        rendered: false,
        element: null,
      });
    }

    stbMessage.textContent = fileName;
    updateStatusBar();
    buildPageLayout();
    updateVisiblePages();
    buildSidebar();
    await Promise.resolve();
    if (generation !== documentLoadGeneration || hwpDoc !== nextDocument) return;
    const activeSelection = rendererSelection ?? selection;

    vscode.postMessage({
      type: "loaded",
      pageCount,
      renderer: activeSelection.diagnostics,
    });
  } catch (err: any) {
    if (generation !== documentLoadGeneration) return;
    if (hwpDoc === nextDocument) {
      hwpDoc = null;
      nextDocument?.free();
    }
    rendererSelection = null;
    stbMessage.textContent = `오류: ${err.message ?? err}`;
    console.error("HWP 로드 실패:", err);
  }
}

function applyRendererSelection(selection: RendererSessionSelection): void {
  rendererSelection = selection;
  document.documentElement.dataset.rendererBackend = selection.backend;
  document.documentElement.dataset.rendererDecisionKey = selection.diagnostics.decisionKey;
}

// ── 상태 표시줄 업데이트 ──

function updateStatusBar(): void {
  const total = pageInfos.length;
  if (!pageInputActive) {
    stbPage.textContent = total > 0 ? `${currentPage + 1} / ${total} 쪽` : "- / - 쪽";
  }
  stbZoomLabel.textContent = `${Math.round(currentZoom * 100)}%`;
  updateZoomMenuChecks();
}

// ── 통합 배율 메뉴 ──

/** 메뉴 항목 중 현재 상태에 해당하는 것의 data 값. 없으면 null. */
function currentMenuKey(): string | null {
  if (zoomMode === "fitWidth") return "fitWidth";
  if (zoomMode === "fitPage") return viewMode === "double" ? "fitSpread" : "fitPage";
  return String(currentZoom);
}

function updateZoomMenuChecks(): void {
  const key = currentMenuKey();
  for (const item of stbZoomPopup.querySelectorAll<HTMLElement>(".stb-popup-item")) {
    const itemKey = item.dataset.mode ?? item.dataset.zoom ?? "";
    const check = item.querySelector<HTMLElement>(".stb-check");
    if (check) check.textContent = itemKey === key ? "✓" : "";
  }
}

function setZoomMenuOpen(open: boolean): void {
  stbZoomPopup.hidden = !open;
  stbZoomMenu.setAttribute("aria-expanded", String(open));
}

stbZoomMenu.addEventListener("click", (e) => {
  e.stopPropagation();
  setZoomMenuOpen(stbZoomPopup.hidden !== false);
});

document.addEventListener("click", () => setZoomMenuOpen(false));
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") setZoomMenuOpen(false);
});

stbZoomPopup.addEventListener("click", (e) => {
  const item = (e.target as HTMLElement).closest<HTMLElement>(".stb-popup-item");
  if (!item) return;
  setZoomMenuOpen(false);

  // 맞춤 3항목은 쪽 배치까지 함께 결정한다. % 프리셋은 배치를 유지한 채 수동 배율로 바꾼다.
  switch (item.dataset.mode) {
    case "fitWidth":
      applyZoomMode("fitWidth", "single");
      return;
    case "fitPage":
      applyZoomMode("fitPage", "single");
      return;
    case "fitSpread":
      applyZoomMode("fitPage", "double");
      return;
  }

  const zoom = Number(item.dataset.zoom);
  if (Number.isFinite(zoom)) applyZoomMode("manual", viewMode, zoom);
});

// ── 줌 제어 ──

const clampZoom = (z: number): number => Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, z));

/**
 * 맞춤 배율을 계산한다.
 *
 * 쪽 크기가 서로 다른 문서(가로/세로 혼합)에서 스크롤 중 배율이 요동치지 않도록
 * 현재 쪽이 아니라 문서 전체의 최대 폭·최대 높이를 기준으로 삼는다.
 *
 * availW/availH 는 컨테이너의 **content-box** 크기(padding·스크롤바 제외)다.
 * ResizeObserver 의 contentRect 가 정확히 이 값이므로 그대로 넘길 수 있다.
 * 생략하면 clientWidth/clientHeight 에서 padding 을 빼서 같은 기준을 만든다.
 */
function computeFitZoom(mode: "fitWidth" | "fitPage", availW?: number, availH?: number): number {
  if (pageInfos.length === 0) return currentZoom;

  let maxW = 0;
  let maxH = 0;
  for (const pi of pageInfos) {
    if (pi.width > maxW) maxW = pi.width;
    if (pi.height > maxH) maxH = pi.height;
  }
  if (maxW <= 0 || maxH <= 0) return currentZoom;

  // 배치된 콘텐츠의 원본 크기 (1쪽 = 쪽 하나, 2쪽 = 두 쪽 + gap)
  const pagesPerRow = viewMode === "double" ? 2 : 1;
  const docW = maxW * pagesPerRow + ROW_GAP * (pagesPerRow - 1);

  // 가용 뷰포트 (content-box). #scroll-container 의 padding 은 세로에만 있다 (12px 0).
  const viewW = (availW ?? scrollContainer.clientWidth) - SIDE_MARGIN * 2;
  const viewH = availH ?? scrollContainer.clientHeight - CONTENT_PADDING * 2;
  if (viewW <= 0 || viewH <= 0) return currentZoom;

  const fitW = viewW / docW;
  if (mode === "fitWidth") return clampZoom(fitW);
  return clampZoom(Math.min(fitW, viewH / maxH));
}

/**
 * 배율을 적용하고 레이아웃을 재구성한다.
 *
 * @param relayoutAnyway 배율이 그대로여도 레이아웃을 다시 만든다.
 *   1쪽↔2쪽 배치만 바뀌고 배율이 우연히 같을 때 필요하다.
 */
function setZoom(newZoom: number, anchorY?: number, relayoutAnyway = false): void {
  newZoom = clampZoom(newZoom);
  if (newZoom === currentZoom && !relayoutAnyway) return;

  const oldZoom = currentZoom;

  // 앵커 기준점 (기본: 뷰포트 중앙)
  const containerRect = scrollContainer.getBoundingClientRect();
  const anchor = anchorY ?? (containerRect.top + containerRect.height / 2);
  const yInContainer = anchor - containerRect.top;
  const docY = (scrollContainer.scrollTop + yInContainer) / oldZoom;

  currentZoom = newZoom;
  buildPageLayout();
  scrollContainer.scrollTop = docY * newZoom - yInContainer;
  updateVisiblePages();
  updateStatusBar();
}

/** 수동 배율로 전환하고 배율을 적용한다. (−/+ 버튼, Ctrl+휠, % 프리셋) */
function setManualZoom(newZoom: number, anchorY?: number): void {
  zoomMode = "manual";
  setZoom(newZoom, anchorY);
  updateStatusBar();
}

/**
 * 쪽 배치와 맞춤 모드를 함께 설정한다.
 *
 * 맞춤 3항목(폭 맞춤 / 쪽 맞춤 / 두 쪽 맞춤)이 배치까지 결정하는 유일한 진입점이다.
 */
function applyZoomMode(mode: ZoomMode, nextViewMode: "single" | "double", zoom?: number): void {
  const layoutChanged = nextViewMode !== viewMode;
  const keepPage = currentPage;

  viewMode = nextViewMode;
  zoomMode = mode;

  const target = mode === "manual" ? (zoom ?? currentZoom) : computeFitZoom(mode);
  // 배치가 바뀌었는데 배율이 우연히 같으면 setZoom 이 조기 반환하므로 강제 재구성한다.
  setZoom(target, undefined, layoutChanged);

  if (layoutChanged) scrollToPage(keepPage);
  updateStatusBar();
}

// ── 뷰포트 크기 변화 대응 ──
//
// 창/에디터 패널 리사이즈, 사이드바 접기·펼치기로 뷰포트가 바뀌면 맞춤 배율을 다시 계산한다.
// 수동 배율일 때는 크기 변화와 무관하게 고정한다.

/** 새 배율이 현재와 이 비율 미만으로 다르면 무시한다. 스크롤바 출현으로 인한 진동 방지. */
const FIT_HYSTERESIS = 0.01;

let resizeRaf = 0;

const zoomResizeObserver = new ResizeObserver((entries) => {
  if (zoomMode === "manual" || pageInfos.length === 0) return;

  // ResizeObserver 의 contentRect 는 스크롤바를 제외한 크기다.
  // clientWidth 를 쓰면 배율↑ → 스크롤바 출현 → 폭↓ → 배율↓ 진동이 생길 수 있다.
  const rect = entries[entries.length - 1].contentRect;
  const availW = rect.width;
  const availH = rect.height;

  if (resizeRaf) cancelAnimationFrame(resizeRaf);
  resizeRaf = requestAnimationFrame(() => {
    resizeRaf = 0;
    if (zoomMode === "manual") return;

    const next = computeFitZoom(zoomMode, availW, availH);
    if (Math.abs(next - currentZoom) / currentZoom < FIT_HYSTERESIS) return;

    const keepPage = currentPage;
    setZoom(next);
    scrollToPage(keepPage);
    updateStatusBar();
  });
});

zoomResizeObserver.observe(scrollContainer);

stbZoomOut.addEventListener("click", () => setManualZoom(currentZoom - ZOOM_STEP));
stbZoomIn.addEventListener("click", () => setManualZoom(currentZoom + ZOOM_STEP));

// Ctrl+마우스 휠 줌
scrollContainer.addEventListener(
  "wheel",
  (e) => {
    if (!e.ctrlKey) return;
    e.preventDefault();
    const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
    setManualZoom(currentZoom + delta, e.clientY);
  },
  { passive: false }
);

// ── 페이지 레이아웃 ──

function makePageWrapper(i: number): HTMLDivElement {
  const pi = pageInfos[i];
  const wrapper = document.createElement("div");
  wrapper.className = "page-wrapper";
  wrapper.style.width = `${Math.round(pi.width * currentZoom)}px`;
  wrapper.style.height = `${Math.round(pi.height * currentZoom)}px`;
  wrapper.dataset.page = String(i);
  pi.element = wrapper;
  pi.rendered = false;
  return wrapper;
}

function buildPageLayout(): void {
  scrollContent.innerHTML = "";
  if (viewMode === "double") {
    // 두 쪽씩 좌우로 묶어 행(.page-row)으로 배치
    for (let i = 0; i < pageInfos.length; i += 2) {
      const row = document.createElement("div");
      row.className = "page-row";
      row.appendChild(makePageWrapper(i));
      if (i + 1 < pageInfos.length) row.appendChild(makePageWrapper(i + 1));
      scrollContent.appendChild(row);
    }
  } else {
    for (let i = 0; i < pageInfos.length; i++) {
      scrollContent.appendChild(makePageWrapper(i));
    }
  }
}

// ── 가상 스크롤 ──

function updateVisiblePages(): void {
  if (!hwpDoc || pageInfos.length === 0) return;

  const containerRect = scrollContainer.getBoundingClientRect();
  const viewTop = containerRect.top - PREFETCH_MARGIN;
  const viewBottom = containerRect.bottom + PREFETCH_MARGIN;

  for (let i = 0; i < pageInfos.length; i++) {
    const pi = pageInfos[i];
    const el = pi.element;
    if (!el) continue;

    const rect = el.getBoundingClientRect();
    if (rect.bottom >= viewTop && rect.top <= viewBottom) {
      if (!pi.rendered) renderPage(i);
    } else {
      if (pi.rendered) releasePage(i);
    }
  }

  updateCurrentPage(containerRect);
}

scrollContainer.addEventListener("scroll", () => {
  requestAnimationFrame(updateVisiblePages);
});

// ── 페이지 렌더링 ──

const reRenderTimers = new Map<number, ReturnType<typeof setTimeout>[]>();

function renderPage(pageNum: number): void {
  if (!hwpDoc) return;
  const pi = pageInfos[pageNum];
  const wrapper = pi.element;
  if (!wrapper) return;

  const dpr = window.devicePixelRatio || 1;
  const cssW = Math.round(pi.width * currentZoom);
  const cssH = Math.round(pi.height * currentZoom);

  const canvas = document.createElement("canvas");
  canvas.width = Math.round(cssW * dpr);
  canvas.height = Math.round(cssH * dpr);
  canvas.style.width = `${cssW}px`;
  canvas.style.height = `${cssH}px`;

  wrapper.innerHTML = "";
  wrapper.appendChild(canvas);

  const scale = currentZoom * dpr;
  let renderedCanvas: HTMLCanvasElement;
  try {
    renderedCanvas = renderDocumentPage(pageNum, canvas, scale);
  } catch (error) {
    pi.rendered = false;
    stbMessage.textContent = `렌더링 오류: ${error instanceof Error ? error.message : String(error)}`;
    console.error(`HWP 페이지 렌더링 실패 (page=${pageNum}):`, error);
    return;
  }
  renderedCanvas.style.width = `${cssW}px`;
  renderedCanvas.style.height = `${cssH}px`;
  pi.rendered = true;

  cancelReRender(pageNum);
  if (rendererSelection?.backend === "canvaskit") return;
  const timers: ReturnType<typeof setTimeout>[] = [];
  for (const delay of [200, 600]) {
    timers.push(
      setTimeout(() => {
        if (
          pi.rendered
          && hwpDoc
          && renderedCanvas.isConnected
          && rendererSelection?.backend === "canvas2d"
        ) {
          hwpDoc.renderPageToCanvas(pageNum, renderedCanvas, scale);
        }
      }, delay)
    );
  }
  reRenderTimers.set(pageNum, timers);
}

function renderDocumentPage(
  pageNum: number,
  targetCanvas: HTMLCanvasElement,
  scale: number,
): HTMLCanvasElement {
  const documentAtRender = hwpDoc;
  if (!documentAtRender) throw new Error("문서가 로드되지 않았습니다");
  const selection = rendererSelection;
  if (selection?.backend !== "canvaskit" || !selection.canvaskitRenderer) {
    documentAtRender.renderPageToCanvas(pageNum, targetCanvas, scale);
    return targetCanvas;
  }

  const decisionKey = selection.diagnostics.decisionKey;
  let tree: PageLayerTree;
  try {
    tree = parsePageLayerTree(
      documentAtRender.getPageLayerTreeWithProfile(pageNum, "screen"),
      pageNum,
    );
  } catch (error) {
    if (!scheduleRendererFallback(error, decisionKey, "resource")) throw error;
    documentAtRender.renderPageToCanvas(pageNum, targetCanvas, scale);
    return targetCanvas;
  }

  const originalParent = targetCanvas.parentElement;
  const originalIndex = originalParent
    ? Array.prototype.indexOf.call(originalParent.children, targetCanvas)
    : -1;
  let renderedCanvas = targetCanvas;
  try {
    renderedCanvas = selection.canvaskitRenderer.renderPage(tree, targetCanvas, scale);
    const diagnostics = selection.canvaskitRenderer.diagnostics();
    if (!diagnostics.passesRuntimeReadinessGate) {
      throw new Error(
        `CanvasKit runtime readiness 실패: ${diagnostics.readinessBlockers.join(", ")}`,
      );
    }
    return renderedCanvas;
  } catch (error) {
    if (!scheduleRendererFallback(error, decisionKey, "runtime")) throw error;
    renderedCanvas = currentCanvasAt(originalParent, originalIndex, renderedCanvas);
    const canvas2d = replaceCanvas(renderedCanvas);
    documentAtRender.renderPageToCanvas(pageNum, canvas2d, scale);
    return canvas2d;
  }
}

function parsePageLayerTree(json: string, pageNum: number): PageLayerTree {
  let parsed: unknown;
  try {
    parsed = JSON.parse(json);
  } catch (error) {
    throw new Error(`PageLayerTree parse 실패 (page=${pageNum}): ${error}`);
  }
  if (!parsed || typeof parsed !== "object") {
    throw new Error(`PageLayerTree shape 오류 (page=${pageNum}): object가 아닙니다`);
  }
  const tree = parsed as Partial<PageLayerTree>;
  if (
    !tree.root
    || !Number.isFinite(tree.pageWidth)
    || !Number.isFinite(tree.pageHeight)
    || tree.pageWidth! <= 0
    || tree.pageHeight! <= 0
  ) {
    throw new Error(`PageLayerTree shape 오류 (page=${pageNum}): 필수 필드가 없습니다`);
  }
  return tree as PageLayerTree;
}

function currentCanvasAt(
  parent: HTMLElement | null,
  childIndex: number,
  fallback: HTMLCanvasElement,
): HTMLCanvasElement {
  const current = parent && childIndex >= 0 ? parent.children.item(childIndex) : null;
  return current instanceof HTMLCanvasElement ? current : fallback;
}

function replaceCanvas(canvas: HTMLCanvasElement): HTMLCanvasElement {
  const parent = canvas.parentElement;
  if (!parent) return canvas;
  const replacement = canvas.cloneNode(true) as HTMLCanvasElement;
  parent.replaceChild(replacement, canvas);
  return replacement;
}

function scheduleRendererFallback(
  error: unknown,
  expectedDecisionKey: string,
  kind: "resource" | "runtime",
): boolean {
  if (rendererSelection?.backend === "canvas2d") return true;
  const fallback = kind === "resource"
    ? rendererSession.fallbackFromResourceFailure(error, expectedDecisionKey)
    : rendererSession.fallbackFromRuntimeFailure(error, expectedDecisionKey);
  if (!fallback) return false;

  applyRendererSelection(fallback);
  vscode.postMessage({ type: "rendererSelectionChanged", renderer: fallback.diagnostics });
  if (rendererFallbackScheduled) return true;
  rendererFallbackScheduled = true;
  queueMicrotask(() => {
    rendererFallbackScheduled = false;
    if (!rendererSession.isCurrent(fallback)) return;
    for (let pageNum = 0; pageNum < pageInfos.length; pageNum++) releasePage(pageNum);
    buildThumbnails();
    updateVisiblePages();
  });
  return true;
}

function cancelReRender(pageNum: number): void {
  const timers = reRenderTimers.get(pageNum);
  if (timers) {
    for (const t of timers) clearTimeout(t);
    reRenderTimers.delete(pageNum);
  }
}

function releasePage(pageNum: number): void {
  cancelReRender(pageNum);
  const pi = pageInfos[pageNum];
  if (outlineHighlight?.parentElement === pi.element) outlineHighlight = null;
  if (pi.element) pi.element.innerHTML = "";
  pi.rendered = false;
}

// ── 현재 페이지 추적 ──

function updateCurrentPage(containerRect: DOMRect): void {
  const centerY = (containerRect.top + containerRect.bottom) / 2;
  for (let i = 0; i < pageInfos.length; i++) {
    const el = pageInfos[i].element;
    if (!el) continue;
    const rect = el.getBoundingClientRect();
    if (rect.top <= centerY && rect.bottom >= centerY) {
      if (currentPage !== i) {
        currentPage = i;
        updateStatusBar();
        highlightCurrentThumb();
      }
      break;
    }
  }
}

// ── 사이드바: 페이지 이동 ──

/** 지정 페이지가 편집 영역 상단에 오도록 스크롤한다. */
function scrollToPage(pageNum: number): void {
  const el = pageInfos[pageNum]?.element;
  if (!el) return;
  const cRect = scrollContainer.getBoundingClientRect();
  const eRect = el.getBoundingClientRect();
  scrollContainer.scrollTop += eRect.top - cRect.top - 12;
  updateVisiblePages();
}

/** 문단의 조판 좌표까지 이동하고 해당 줄을 강조한다. */
function scrollToDocumentPosition(pageNum: number, y: number, height: number): void {
  const el = pageInfos[pageNum]?.element;
  if (!el) return;
  const cRect = scrollContainer.getBoundingClientRect();
  const eRect = el.getBoundingClientRect();
  scrollContainer.scrollTop += eRect.top - cRect.top + y * currentZoom - 12;
  updateVisiblePages();
  requestAnimationFrame(() => showOutlineHighlight(pageNum, y, height));
}

function showOutlineHighlight(pageNum: number, y: number, height: number): void {
  outlineHighlight?.remove();
  outlineHighlight = null;

  const page = pageInfos[pageNum];
  if (!page?.rendered || !page.element) return;
  const highlight = document.createElement("div");
  highlight.className = "outline-highlight";
  highlight.style.top = `${Math.max(0, y * currentZoom - 4)}px`;
  highlight.style.height = `${Math.max(16, height * currentZoom + 8)}px`;
  page.element.appendChild(highlight);
  outlineHighlight = highlight;
}

// ── 사이드바: 썸네일 ──

/** 문서 로드 후 사이드바 콘텐츠(썸네일/목차/북마크)를 갱신한다. */
function buildSidebar(): void {
  buildThumbnails();
  buildOutline();
  buildBookmarks();
}

/** 빈 패널 안내 요소. */
function navEmpty(text: string): HTMLElement {
  const d = document.createElement("div");
  d.className = "nav-empty";
  d.textContent = text;
  return d;
}

/** (섹션, 문단) 위치의 페이지로 이동한다. */
function navigateToPosition(section: number, para: number): void {
  if (!hwpDoc) return;
  try {
    const res = JSON.parse(hwpDoc.getPageOfPosition(section, para));
    if (res?.ok && typeof res.page === "number") scrollToPage(res.page);
  } catch {
    /* 위치 해석 실패 시 무시 */
  }
}

const THUMB_WIDTH = 160;
let thumbObserver: IntersectionObserver | null = null;

/** 썸네일 목록을 생성한다. IntersectionObserver로 보이는 것만 지연 렌더. */
function buildThumbnails(): void {
  const panel = navPanels.get("thumb");
  if (!panel) return;
  panel.innerHTML = "";
  thumbObserver?.disconnect();

  thumbObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const thumb = entry.target as HTMLElement;
        renderThumbnail(Number(thumb.dataset.page));
        thumbObserver!.unobserve(thumb);
      }
    },
    { root: panel, rootMargin: "200px" }
  );

  for (let i = 0; i < pageInfos.length; i++) {
    const pi = pageInfos[i];
    const thumb = document.createElement("div");
    thumb.className = "nav-thumb";
    thumb.dataset.page = String(i);

    const cssH = Math.round((pi.height / pi.width) * THUMB_WIDTH);
    const canvas = document.createElement("canvas");
    canvas.style.width = `${THUMB_WIDTH}px`;
    canvas.style.height = `${cssH}px`;

    const label = document.createElement("div");
    label.className = "nav-thumb-label";
    label.textContent = String(i + 1);

    thumb.appendChild(canvas);
    thumb.appendChild(label);
    thumb.addEventListener("click", () => scrollToPage(i));
    panel.appendChild(thumb);

    thumbObserver.observe(thumb);
  }
  highlightCurrentThumb();
}

function renderThumbnail(pageNum: number): void {
  if (!hwpDoc) return;
  const pi = pageInfos[pageNum];
  const panel = navPanels.get("thumb");
  const thumb = panel?.querySelector<HTMLElement>(`.nav-thumb[data-page="${pageNum}"]`);
  const canvas = thumb?.querySelector("canvas");
  if (!canvas) return;

  const dpr = window.devicePixelRatio || 1;
  const scale = THUMB_WIDTH / pi.width;
  canvas.width = Math.round(pi.width * scale * dpr);
  canvas.height = Math.round(pi.height * scale * dpr);
  try {
    renderDocumentPage(pageNum, canvas, scale * dpr);
  } catch (error) {
    console.error(`HWP 썸네일 렌더링 실패 (page=${pageNum}):`, error);
  }
}

/** 현재 페이지 썸네일을 강조하고 보이도록 스크롤한다. */
function highlightCurrentThumb(): void {
  const panel = navPanels.get("thumb");
  if (!panel) return;
  panel.querySelectorAll(".nav-thumb.current").forEach((el) => el.classList.remove("current"));
  const cur = panel.querySelector<HTMLElement>(`.nav-thumb[data-page="${currentPage}"]`);
  if (cur) {
    cur.classList.add("current");
    if (navSidebar.offsetWidth > 0 && !navPanels.get("thumb")!.hidden) {
      cur.scrollIntoView({ block: "nearest" });
    }
  }
}

// ── 사이드바: 탭 전환 / 접기 ──

function switchTab(name: string): void {
  navTabs.forEach((t) => t.classList.toggle("active", t.dataset.tab === name));
  navPanels.forEach((panel, key) => {
    panel.hidden = key !== name;
  });
  if (name === "thumb") highlightCurrentThumb();
}

navTabs.forEach((tab) => {
  tab.addEventListener("click", () => switchTab(tab.dataset.tab!));
});

/** 사이드바 열기/닫기. collapse 미지정 시 현재 상태 반전. */
function toggleSidebar(collapse?: boolean): void {
  const next = collapse ?? !navSidebar.classList.contains("collapsed");
  navSidebar.classList.toggle("collapsed", next);
  appShell.classList.toggle("sidebar-collapsed", next);
  if (!next) highlightCurrentThumb();
}

function maxSidebarWidth(): number {
  const availableWidth = appShell.clientWidth || window.innerWidth;
  return Math.max(
    MIN_SIDEBAR_WIDTH,
    Math.min(MAX_SIDEBAR_WIDTH, availableWidth - 240)
  );
}

function saveViewerState(): void {
  vscode.setState({
    sidebarWidth,
    collapsedOutlineKeys: Array.from(collapsedOutlineKeys),
  });
}

function setSidebarWidth(width: number, save = true): void {
  sidebarWidth = Math.round(
    Math.max(MIN_SIDEBAR_WIDTH, Math.min(maxSidebarWidth(), width))
  );
  appShell.style.setProperty("--nav-sidebar-width", `${sidebarWidth}px`);
  navResizer.setAttribute("aria-valuenow", String(sidebarWidth));
  navResizer.setAttribute("aria-valuemin", String(MIN_SIDEBAR_WIDTH));
  navResizer.setAttribute("aria-valuemax", String(maxSidebarWidth()));
  if (save) saveViewerState();
}

navResizer.addEventListener("pointerdown", (event) => {
  if (event.button !== 0 || navSidebar.classList.contains("collapsed")) return;
  event.preventDefault();

  const startX = event.clientX;
  const startWidth = sidebarWidth;
  navResizer.setPointerCapture(event.pointerId);
  appShell.classList.add("sidebar-resizing");

  const resize = (moveEvent: PointerEvent) => {
    setSidebarWidth(startWidth + moveEvent.clientX - startX, false);
  };
  const finish = () => {
    navResizer.removeEventListener("pointermove", resize);
    navResizer.removeEventListener("pointerup", finish);
    navResizer.removeEventListener("pointercancel", finish);
    appShell.classList.remove("sidebar-resizing");
    saveViewerState();
  };

  navResizer.addEventListener("pointermove", resize);
  navResizer.addEventListener("pointerup", finish);
  navResizer.addEventListener("pointercancel", finish);
});

navResizer.addEventListener("keydown", (event) => {
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    setSidebarWidth(sidebarWidth - 16);
  } else if (event.key === "ArrowRight") {
    event.preventDefault();
    setSidebarWidth(sidebarWidth + 16);
  } else if (event.key === "Home") {
    event.preventDefault();
    setSidebarWidth(MIN_SIDEBAR_WIDTH);
  } else if (event.key === "End") {
    event.preventDefault();
    setSidebarWidth(maxSidebarWidth());
  }
});

setSidebarWidth(sidebarWidth, false);
stbSidebarToggle.addEventListener("click", () => toggleSidebar());
navCollapse.addEventListener("click", () => toggleSidebar(true));
navReopen.addEventListener("click", () => toggleSidebar(false));


// ── 사이드바: 개요 ──

interface OutlineNavigationItem {
  level: number;
  number: string;
  title: string;
  page: number;
  section: number;
  paragraph: number;
}

interface OutlineTreeNode {
  entry: OutlineNavigationItem;
  children: OutlineTreeNode[];
}

function outlineKey(entry: OutlineNavigationItem): string {
  return `${entry.section}:${entry.paragraph}`;
}

/**
 * 지금 그려진 개요 항목의 `data-outline-key` → 항목 정보.
 *
 * 방향키 이동이 초점을 옮긴 요소에서 이동 대상을 되찾는 데 쓴다. 접혀서 그려지지
 * 않은 하위 항목은 들어 있지 않다 — 목록을 다시 그릴 때마다 비운다.
 */
const outlineEntryByKey = new Map<string, OutlineNavigationItem>();

/** 개요 수준을 이용해 평면 목록을 부모/자식 트리로 구성한다. */
function buildOutlineTree(entries: OutlineNavigationItem[]): OutlineTreeNode[] {
  const roots: OutlineTreeNode[] = [];
  const ancestors: OutlineTreeNode[] = [];

  for (const entry of entries) {
    const node: OutlineTreeNode = { entry, children: [] };
    while (
      ancestors.length > 0
      && ancestors[ancestors.length - 1].entry.level >= entry.level
    ) {
      ancestors.pop();
    }
    const parent = ancestors[ancestors.length - 1];
    (parent ? parent.children : roots).push(node);
    ancestors.push(node);
  }

  return roots;
}

/** 지금 화면에 그려진 개요 항목(접혀서 숨은 하위는 제외)을 위에서 아래 순서로 준다. */
function outlineItemElements(): HTMLElement[] {
  const panel = navPanels.get("outline");
  return panel ? Array.from(panel.querySelectorAll<HTMLElement>(".nav-outline-item")) : [];
}

function outlineElement(key: string, childSelector = ""): HTMLElement | null {
  const panel = navPanels.get("outline");
  const selector = `.nav-outline-item[data-outline-key="${CSS.escape(key)}"]${childSelector}`;
  return panel?.querySelector<HTMLElement>(selector) ?? null;
}

/** 재렌더로 사라진 초점을 같은 개요 항목의 접기/펼치기 버튼으로 되돌린다. */
function focusOutlineToggle(key: string): void {
  outlineElement(key, " .nav-outline-toggle")?.focus();
}

/** 재렌더로 사라진 초점을 같은 개요 항목으로 되돌린다. */
function focusOutlineItem(key: string): void {
  outlineElement(key)?.focus();
}

/**
 * 접기/펼치기 상태를 바꾸고 목록을 다시 그린다.
 *
 * `buildOutline()` 이 패널을 통째로 다시 그려 초점 요소가 DOM 에서 사라지므로,
 * 키보드로 조작했으면 같은 항목의 어디로 초점을 돌려놓을지 함께 받는다.
 */
function setOutlineCollapsed(
  key: string,
  collapsed: boolean,
  refocus: "item" | "toggle" | null,
): void {
  collapsedOutlineKeys[collapsed ? "add" : "delete"](key);
  saveViewerState();
  buildOutline();
  if (refocus === "toggle") focusOutlineToggle(key);
  else if (refocus === "item") focusOutlineItem(key);
}

/**
 * 개요 목록에서 초점만 옮긴다.
 *
 * 방향키는 훑기 전용이다 — 본문을 따라 움직이면 목록을 지나가는 동안 화면이 계속
 * 튄다. 이동은 `Enter`/`Space` 로만 일으킨다.
 */
function moveOutlineFocus(from: HTMLElement, to: number | "first" | "last"): void {
  const items = outlineItemElements();
  if (items.length === 0) return;

  const current = items.indexOf(from);
  const index = to === "first" ? 0 : to === "last" ? items.length - 1 : current + to;
  if (index < 0 || index >= items.length || index === current) return;

  items[index].focus();
}

/** 상위 수준 개요로 초점을 올린다. 위쪽에서 자기보다 수준이 낮은 첫 항목을 찾는다. */
function focusOutlineParent(from: HTMLElement, level: number): void {
  const items = outlineItemElements();
  for (let index = items.indexOf(from) - 1; index >= 0; index -= 1) {
    const entry = outlineEntryByKey.get(items[index].dataset.outlineKey ?? "");
    if (entry && entry.level < level) {
      items[index].focus();
      return;
    }
  }
}

function renderOutlineTree(panel: HTMLElement, nodes: OutlineTreeNode[]): void {
  for (const node of nodes) {
    const { entry } = node;
    const hasChildren = node.children.length > 0;
    const key = outlineKey(entry);
    const expanded = !collapsedOutlineKeys.has(key);
    const item = document.createElement("div");
    item.className = "nav-item nav-outline-item";
    item.style.paddingLeft = `${(Math.max(1, entry.level) - 1) * 12 + 2}px`;
    item.tabIndex = 0;
    item.dataset.outlineKey = key;
    outlineEntryByKey.set(key, entry);
    const label = `${entry.number} ${entry.title}`.trim() || "(제목 없음)";
    item.title = label;

    if (hasChildren) {
      const toggle = document.createElement("button");
      toggle.className = "nav-outline-toggle";
      toggle.type = "button";
      toggle.textContent = expanded ? "▾" : "▸";
      toggle.title = expanded ? "하위 개요 접기" : "하위 개요 펼치기";
      toggle.setAttribute("aria-label", `${label} 하위 개요 ${expanded ? "접기" : "펼치기"}`);
      toggle.setAttribute("aria-expanded", String(expanded));
      toggle.addEventListener("click", (event) => {
        event.stopPropagation();
        // 초점을 되돌리지 않으면 재렌더로 activeElement 가 body 로 떨어져 두 번째
        // Enter/Space 부터 아무 데도 가지 않는다 — 키보드로는 한 번밖에 접지 못한다.
        const refocus = document.activeElement === toggle ? "toggle" : null;
        setOutlineCollapsed(key, expanded, refocus);
      });
      item.appendChild(toggle);
    } else {
      const spacer = document.createElement("span");
      spacer.className = "nav-outline-spacer";
      spacer.setAttribute("aria-hidden", "true");
      item.appendChild(spacer);
    }

    const labelElement = document.createElement("span");
    labelElement.className = "nav-outline-label";
    labelElement.textContent = label;
    item.appendChild(labelElement);
    item.addEventListener("click", () => navigateToOutline(entry));
    item.addEventListener("keydown", (event) => {
      // 접기/펼치기 버튼에 초점이 있는 Enter/Space 는 버튼의 기본 동작(click)이다.
      // 여기서 받아 preventDefault 하면 그 기본 동작이 취소되고 이동까지 겹쳐
      // 키보드만으로는 접기/펼치기가 동작하지 않는다.
      if (event.target !== item) return;

      switch (event.key) {
        case "Enter":
        case " ":
          event.preventDefault();
          navigateToOutline(entry);
          return;
        // 방향키는 목록 훑기 전용 — 본문은 따라가지 않는다. 기본 스크롤은 막는다.
        case "ArrowDown":
          event.preventDefault();
          moveOutlineFocus(item, 1);
          return;
        case "ArrowUp":
          event.preventDefault();
          moveOutlineFocus(item, -1);
          return;
        case "Home":
          event.preventDefault();
          moveOutlineFocus(item, "first");
          return;
        case "End":
          event.preventDefault();
          moveOutlineFocus(item, "last");
          return;
        // 오른쪽: 접혀 있으면 펼치고, 이미 펼쳐져 있으면 첫 하위로 내려간다.
        case "ArrowRight":
          event.preventDefault();
          if (hasChildren && !expanded) setOutlineCollapsed(key, false, "item");
          else if (hasChildren) moveOutlineFocus(item, 1);
          return;
        // 왼쪽: 펼쳐져 있으면 접고, 아니면 상위 개요로 올라간다.
        case "ArrowLeft":
          event.preventDefault();
          if (hasChildren && expanded) setOutlineCollapsed(key, true, "item");
          else focusOutlineParent(item, entry.level);
          return;
      }
    });
    panel.appendChild(item);

    if (hasChildren && expanded) renderOutlineTree(panel, node.children);
  }
}

/** 문단 모양의 개요 번호만 개요 패널에 렌더한다. */
function buildOutline(): void {
  const panel = navPanels.get("outline");
  if (!panel || !hwpDoc) return;
  panel.innerHTML = "";
  outlineEntryByKey.clear();

  let outline: OutlineNavigationItem[] = [];
  try {
    outline = JSON.parse(hwpDoc.getOutlineNavigation()).outline ?? [];
  } catch {
    outline = [];
  }
  if (outline.length === 0) {
    panel.appendChild(navEmpty("개요 번호가 없습니다"));
    return;
  }

  renderOutlineTree(panel, buildOutlineTree(outline));
}

/** 개요 문단의 조판 좌표로 이동한다. 좌표를 못 찾으면 쪽 번호로 폴백한다. */
function navigateToOutline(entry: OutlineNavigationItem): void {
  if (!hwpDoc) return;
  try {
    const rect = JSON.parse(hwpDoc.getCursorRect(entry.section, entry.paragraph, 0));
    if (
      typeof rect?.pageIndex === "number"
      && typeof rect?.y === "number"
      && typeof rect?.height === "number"
    ) {
      scrollToDocumentPosition(rect.pageIndex, rect.y, rect.height);
      return;
    }
  } catch {
    // 개요 문단의 정확한 조판 좌표가 없으면 아래 쪽 이동으로 폴백한다.
  }
  if (entry.page > 0) scrollToPage(entry.page - 1);
}

// ── 사이드바: 북마크 ──

interface BookmarkItem {
  name: string;
  sec: number;
  para: number;
}

/** 사용자 북마크 목록을 북마크 패널에 렌더한다. */
function buildBookmarks(): void {
  const panel = navPanels.get("bookmark");
  if (!panel || !hwpDoc) return;
  panel.innerHTML = "";

  let list: BookmarkItem[] = [];
  try {
    list = JSON.parse(hwpDoc.getBookmarks());
  } catch {
    list = [];
  }
  if (list.length === 0) {
    panel.appendChild(navEmpty("북마크가 없습니다"));
    return;
  }

  for (const b of list) {
    const item = document.createElement("div");
    item.className = "nav-item";
    const label = b.name || "(이름 없음)";
    item.textContent = label;
    item.title = label;
    item.addEventListener("click", () => navigateToPosition(b.sec, b.para));
    panel.appendChild(item);
  }
}

// ── 상태 표시줄: 쪽 번호 이동 ──

stbPage.style.cursor = "pointer";
stbPage.title = "쪽 번호로 이동";
stbPage.addEventListener("click", () => {
  if (pageInputActive || pageInfos.length === 0) return;
  pageInputActive = true;

  const input = document.createElement("input");
  input.type = "number";
  input.min = "1";
  input.max = String(pageInfos.length);
  input.value = String(currentPage + 1);
  input.style.width = "52px";
  input.style.height = "18px";
  input.style.fontSize = "12px";

  const restore = (): void => {
    pageInputActive = false;
    updateStatusBar();
  };
  const commit = (): void => {
    const n = parseInt(input.value, 10);
    if (!Number.isNaN(n) && n >= 1 && n <= pageInfos.length) {
      restore();
      scrollToPage(n - 1);
    } else {
      restore();
    }
  };
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") commit();
    else if (e.key === "Escape") restore();
  });
  input.addEventListener("blur", restore);

  stbPage.textContent = "";
  stbPage.appendChild(input);
  input.focus();
  input.select();
});

// ── 유틸리티 ──

function toUint8Array(data: unknown): Uint8Array {
  if (data instanceof Uint8Array) return data;
  if (data instanceof ArrayBuffer) return new Uint8Array(data);
  if (ArrayBuffer.isView(data)) return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  if (data && typeof data === "object") {
    const values = Object.values(data as Record<string, number>);
    return new Uint8Array(values);
  }
  throw new Error(`Uint8Array로 변환할 수 없는 데이터: ${typeof data}`);
}

function releaseRenderedDocument(): void {
  for (let pageNum = 0; pageNum < pageInfos.length; pageNum++) releasePage(pageNum);
  pageInfos = [];
  scrollContent.innerHTML = "";
  thumbObserver?.disconnect();
  const thumbPanel = navPanels.get("thumb");
  if (thumbPanel) thumbPanel.innerHTML = "";
}

// 기본 컨텍스트 메뉴 억제
document.addEventListener("contextmenu", (e) => {
  e.preventDefault();
});

window.addEventListener("unload", () => {
  rendererSession.dispose();
  hwpDoc?.free();
  hwpDoc = null;
});

function installMeasureTextWidth(): void {
  if ((globalThis as any).measureTextWidth) return;
  let ctx: CanvasRenderingContext2D | null = null;
  let lastFont = "";
  (globalThis as any).measureTextWidth = (font: string, text: string): number => {
    if (!ctx) ctx = document.createElement("canvas").getContext("2d");
    if (font !== lastFont) { ctx!.font = font; lastFont = font; }
    return ctx!.measureText(text).width;
  };
}

declare function acquireVsCodeApi(): {
  postMessage(msg: unknown): void;
  getState(): unknown;
  setState(state: unknown): void;
};
