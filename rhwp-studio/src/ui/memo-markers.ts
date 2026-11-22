import type { EventBus } from '@/core/event-bus';
import type { MemoInfo } from '@/core/memo-editor';
import type { CursorRect, SelectionRect } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { CanvasView } from '@/view/canvas-view';
import { MemoMargin, type MemoAction } from './memo-margin';
import { t } from '../i18n/index.ts';

export interface MemoMark {
  memo: MemoInfo;
  rect: CursorRect;
  highlight: SelectionRect[];
  replies: MemoInfo[];
}

/** DOM 표시만 갱신한다. Canvas와 문서 데이터에는 손대지 않는다. */
export function installMemoMarkers(
  container: HTMLElement, canvas: CanvasView, wasm: WasmBridge, bus: EventBus, open: MemoAction,
): void {
  const content = container.querySelector<HTMLElement>('#scroll-content');
  if (!content) return;
  const scrollContent = content;
  const layer = document.createElement('div');
  layer.className = 'memo-marker-layer';
  const highlightLayer = document.createElement('div');
  highlightLayer.className = 'memo-highlight-layer';
  content.append(highlightLayer, layer);
  const virtual = canvas.getVirtualScroll();
  let marks: MemoMark[] | null = null;
  let generation = -1;
  let scheduled = false;
  const buttons = new Map<number, HTMLButtonElement>();
  const margin = new MemoMargin(container, bus, open, schedule);


  function update() {
    scheduled = false;
    if (layer.parentElement !== scrollContent) {
      scrollContent.append(highlightLayer, layer);
      marks = null;
    }
    if (generation !== wasm.documentGeneration) {
      generation = wasm.documentGeneration;
      marks = null;
      margin.reset();
      for (const button of buttons.values()) button.remove();
      buttons.clear();
    }
    if (!marks) {
      const memos = wasm.pageCount > 0 ? wasm.getMemos() : [];
      const families = new Map<number, MemoInfo[]>();
      for (const memo of memos) {
        if (memo.parentId === null) continue;
        const replies = families.get(memo.parentId) ?? [];
        replies.push(memo);
        families.set(memo.parentId, replies);
      }
      marks = memos.filter(memo => memo.parentId === null).flatMap(memo => {
        const target = wasm.getMemoTarget(memo, true);
        return target ? [{ memo, rect: target.rect, highlight: target.highlight, replies: families.get(memo.id) ?? [] }] : [];
      });
    }
    layer.hidden = !margin.annotations || margin.visible;
    highlightLayer.replaceChildren();

    const zoom = canvas.getViewportManager().getZoom();
    const pages = new Set(virtual.getVisiblePages(
      container.scrollTop, container.clientHeight, container.scrollLeft, container.clientWidth,
    ));
    const visible = marks.filter(mark => pages.has(mark.rect.pageIndex));
    const ids = new Set(visible.map(mark => mark.memo.id));
    for (const [id, button] of buttons) {
      if (ids.has(id)) continue;
      button.remove();
      buttons.delete(id);
    }
    const located = [];
    const bottom = new Map<number, number>();
    for (const { memo, rect, highlight, replies } of [...visible].sort((a, b) => a.rect.y - b.rect.y)) {
      let button = buttons.get(memo.id);
      if (!button) {
        button = document.createElement('button');
        button.type = 'button';
        button.textContent = '▤';
        button.dataset.memoId = String(memo.id);
        button.addEventListener('click', () => open(memo.id));
        layer.append(button);
        buttons.set(memo.id, button);
      }
      const label = t('dialog.memo.marker', { p1: memo.text.slice(0, 50) });
      button.title = label;
      button.setAttribute('aria-label', label);
      const page = rect.pageIndex;
      const desired = virtual.getPageOffset(page) + rect.y * zoom;
      const top = Math.max(desired, bottom.get(page) ?? desired);
      bottom.set(page, top + 20);
      const right = virtual.getPageLeftResolved(page, scrollContent.clientWidth) + virtual.getPageWidth(page);
      button.style.left = `${right - 20}px`;
      button.style.top = String(top) + 'px';
      located.push({
        memo, replies,
        x: right - virtual.getPageWidth(page) + rect.x * zoom,
        y: desired + rect.height * zoom,
        edge: right - 20,
      });
    }
    if (margin.annotations) {
      for (const { memo, rect, highlight } of marks) {
        for (const r of highlight.length ? highlight : [{ ...rect, width: 2 }]) {
          if (!pages.has(r.pageIndex)) continue;
          const mark = document.createElement('span');
          mark.classList.toggle('is-active', margin.active === memo.id);
          const left = virtual.getPageLeftResolved(r.pageIndex, scrollContent.clientWidth);
          mark.style.left = String(left + r.x * zoom) + 'px';
          mark.style.top = String(virtual.getPageOffset(r.pageIndex) + r.y * zoom) + 'px';
          mark.style.width = String(Math.max(2, r.width * zoom)) + 'px';
          mark.style.height = String(r.height * zoom) + 'px';
          highlightLayer.append(mark);
        }
      }
    }
    margin.update(located);
  }

  function schedule() {
    if (!scheduled) {
      scheduled = true;
      requestAnimationFrame(update);
    }
  }

  for (const event of ['document-changed', 'document-mutated', 'document-layout-refreshed', 'document-view-changed']) {
    bus.on(event, () => { marks = null; schedule(); });
  }
  bus.on('active-page-changed', schedule);
  bus.on('zoom-changed', schedule);
  container.addEventListener('scroll', schedule, { passive: true });
  new ResizeObserver(schedule).observe(container);
  schedule();
}
