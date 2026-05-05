import type { WasmBridge } from './wasm-bridge';

interface IdOverlayEntry {
  section: number;
  para: number;
  x: number;
  y: number;
  width: number;
  height: number;
  text: string;
}

interface SearchResult {
  section: number;
  paragraph: number;
  charOffset: number;
  page: number;
  x: number;
  y: number;
}

let overlayContainer: HTMLElement | null = null;

function getScrollContainer(): HTMLElement | null {
  return document.getElementById('scroll-container');
}

function createOverlayContainer(): HTMLElement {
  if (overlayContainer) return overlayContainer;
  const sc = getScrollContainer();
  if (!sc) throw new Error('scroll-container not found');
  const el = document.createElement('div');
  el.id = 'rhwp-dev-overlay';
  el.style.cssText = 'position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:9999;';
  sc.style.position = 'relative';
  sc.appendChild(el);
  overlayContainer = el;
  return el;
}

function removeOverlay(): void {
  if (overlayContainer) {
    overlayContainer.remove();
    overlayContainer = null;
  }
}

export function initRhwpDev(wasm: WasmBridge): void {
  const dev = {
    showAllIds(pageNum?: number): void {
      removeOverlay();
      const container = createOverlayContainer();
      const totalPages = wasm.getPageCount();
      const startPage = pageNum ?? 0;
      const endPage = pageNum != null ? pageNum + 1 : totalPages;

      for (let p = startPage; p < endPage; p++) {
        let layout: string;
        try {
          layout = (wasm as any).doc.getPageTextLayout(p);
        } catch { continue; }
        const data = JSON.parse(layout);
        if (!data || !Array.isArray(data.runs)) continue;

        const pageEls = document.querySelectorAll(`[data-page="${p}"]`);
        const pageEl = pageEls[0] as HTMLElement | undefined;
        if (!pageEl) continue;
        const pageRect = pageEl.getBoundingClientRect();
        const scrollRect = getScrollContainer()!.getBoundingClientRect();
        const offsetX = pageRect.left - scrollRect.left + getScrollContainer()!.scrollLeft;
        const offsetY = pageRect.top - scrollRect.top + getScrollContainer()!.scrollTop;

        for (const run of data.runs) {
          const label = `s${run.section ?? '?'}:pi=${run.para ?? '?'}`;
          const tag = document.createElement('div');
          tag.style.cssText = `position:absolute;left:${offsetX + (run.x ?? 0)}px;top:${offsetY + (run.y ?? 0) - 10}px;font-size:8px;color:#e63946;background:rgba(255,255,255,0.85);padding:0 2px;border-radius:2px;white-space:nowrap;`;
          tag.textContent = label;
          container.appendChild(tag);
        }
      }
      console.log(`[rhwpDev] showAllIds: ${container.children.length} IDs overlaid`);
    },

    hideAllIds(): void {
      removeOverlay();
      console.log('[rhwpDev] overlay removed');
    },

    search(text: string): SearchResult | null {
      const result = wasm.searchText(text, 0, 0, 0, true, false);
      if (!result) {
        console.warn(`[rhwpDev] search("${text}"): not found`);
        return null;
      }
      const pos = result as any;
      const out: SearchResult = {
        section: pos.section ?? 0,
        paragraph: pos.paragraph ?? 0,
        charOffset: pos.charOffset ?? 0,
        page: pos.page ?? 0,
        x: pos.x ?? 0,
        y: pos.y ?? 0,
      };
      console.log(`[rhwpDev] search("${text}"): s${out.section}:pi=${out.paragraph} offset=${out.charOffset} page=${out.page}`);
      return out;
    },

    findNearest(targetId: number, pageNum?: number): { id: number; distance: number; text: string } | null {
      const totalPages = wasm.getPageCount();
      const page = pageNum ?? 0;
      if (page >= totalPages) return null;

      let layout: string;
      try {
        layout = (wasm as any).doc.getPageTextLayout(page);
      } catch { return null; }
      const data = JSON.parse(layout);
      if (!data || !Array.isArray(data.runs)) return null;

      let nearest: { id: number; distance: number; text: string } | null = null;
      for (const run of data.runs) {
        const id = run.para as number;
        if (id == null) continue;
        const dist = Math.abs(id - targetId);
        if (!nearest || dist < nearest.distance) {
          nearest = { id, distance: dist, text: (run.text ?? '').slice(0, 30) };
        }
      }
      if (nearest) {
        console.log(`[rhwpDev] findNearest(${targetId}): closest pi=${nearest.id} (distance=${nearest.distance}) "${nearest.text}"`);
      }
      return nearest;
    },

    help(): void {
      console.log(`%c[rhwpDev]%c Debugging Toolkit
  .showAllIds(page?)  — overlay paragraph IDs on rendered pages
  .hideAllIds()       — remove overlay
  .search("text")    — find paragraph/position for text content
  .findNearest(id, page?) — find nearest valid ID to a given ID
  .help()            — this message`, 'color:#2563eb;font-weight:bold', 'color:inherit');
    },
  };

  (window as any).rhwpDev = dev;
  console.log('%c[rhwpDev]%c Debugging toolkit loaded — rhwpDev.help() for usage', 'color:#2563eb;font-weight:bold', 'color:inherit');
}
