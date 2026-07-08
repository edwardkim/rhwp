import init, { HwpDocument } from "@rhwp-wasm/rhwp.js";

// WASM 렌더러가 호출하는 텍스트 폭 측정 콜백 등록
installMeasureTextWidth();

// VSCode Webview API
const vscode = acquireVsCodeApi();

// DOM 요소
const scrollContainer = document.getElementById("scroll-container")!;
const scrollContent = document.getElementById("scroll-content")!;
const stbPage = document.getElementById("stb-page")!;
const stbMessage = document.getElementById("stb-message")!;
const stbZoomVal = document.getElementById("stb-zoom-val")!;
const stbZoomOut = document.getElementById("stb-zoom-out")!;
const stbZoomIn = document.getElementById("stb-zoom-in")!;

// 문서 상태
let hwpDoc: HwpDocument | null = null;
let pageInfos: PageInfo[] = [];
let currentZoom = 1.0;
let currentPage = 0;
let fileName = "";
const PREFETCH_MARGIN = 300;
const ZOOM_STEP = 0.1;
const ZOOM_MIN = 0.25;
const ZOOM_MAX = 3.0;

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
    if (!wasmReady) {
      stbMessage.textContent = "오류: WASM이 아직 초기화되지 않았습니다";
      return;
    }
    try {
      fileName = msg.fileName;
      stbMessage.textContent = `${fileName} 로딩 중...`;

      const fileBytes = toUint8Array(msg.fileData);
      hwpDoc = new HwpDocument(fileBytes);
      hwpDoc.setClipEnabled(false);

      const docInfo = JSON.parse(hwpDoc.getDocumentInfo());
      const pageCount: number = docInfo.page_count ?? docInfo.pageCount ?? 0;

      pageInfos = [];
      for (let i = 0; i < pageCount; i++) {
        const pi = JSON.parse(hwpDoc.getPageInfo(i));
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

      vscode.postMessage({ type: "loaded", pageCount });
    } catch (err: any) {
      stbMessage.textContent = `오류: ${err.message ?? err}`;
      console.error("HWP 로드 실패:", err);
    }
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

// ── 상태 표시줄 업데이트 ──

function updateStatusBar(): void {
  const total = pageInfos.length;
  stbPage.textContent = total > 0 ? `${currentPage + 1} / ${total} 쪽` : "- / - 쪽";
  stbZoomVal.textContent = `${Math.round(currentZoom * 100)}%`;
}

// ── 줌 제어 ──

function applyZoom(newZoom: number, anchorY?: number): void {
  newZoom = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, newZoom));
  if (newZoom === currentZoom) return;

  const oldZoom = currentZoom;

  // 앵커 기준점 (기본: ��포트 중앙)
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

stbZoomOut.addEventListener("click", () => applyZoom(currentZoom - ZOOM_STEP));
stbZoomIn.addEventListener("click", () => applyZoom(currentZoom + ZOOM_STEP));

// Ctrl+마우스 휠 줌
scrollContainer.addEventListener(
  "wheel",
  (e) => {
    if (!e.ctrlKey) return;
    e.preventDefault();
    const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
    applyZoom(currentZoom + delta, e.clientY);
  },
  { passive: false }
);

// ── 페이지 레이아웃 ──

function buildPageLayout(): void {
  scrollContent.innerHTML = "";
  for (let i = 0; i < pageInfos.length; i++) {
    const pi = pageInfos[i];
    const w = Math.round(pi.width * currentZoom);
    const h = Math.round(pi.height * currentZoom);

    const wrapper = document.createElement("div");
    wrapper.className = "page-wrapper";
    wrapper.style.width = `${w}px`;
    wrapper.style.height = `${h}px`;
    wrapper.dataset.page = String(i);

    scrollContent.appendChild(wrapper);
    pi.element = wrapper;
    pi.rendered = false;
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
  hwpDoc.renderPageToCanvas(pageNum, canvas, scale);
  pi.rendered = true;

  cancelReRender(pageNum);
  const timers: ReturnType<typeof setTimeout>[] = [];
  for (const delay of [200, 600]) {
    timers.push(
      setTimeout(() => {
        if (pi.rendered && hwpDoc && canvas.isConnected) {
          hwpDoc.renderPageToCanvas(pageNum, canvas, scale);
        }
      }, delay)
    );
  }
  reRenderTimers.set(pageNum, timers);
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
      }
      break;
    }
  }
}

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

// 기본 컨텍스트 메뉴 억제
document.addEventListener("contextmenu", (e) => {
  e.preventDefault();
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
