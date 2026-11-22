import type { EventBus } from '@/core/event-bus';
import type { MemoInfo } from '@/core/memo-editor';
import type { CursorRect } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { CanvasView } from '@/view/canvas-view';
import { MemoMargin } from './memo-margin';
import { t } from '../i18n/index.ts';

export interface MemoMark {
  memo: MemoInfo;
  rect: CursorRect;
}

/** DOM 표시만 갱신한다. Canvas와 문서 데이터에는 손대지 않는다. */
export function installMemoMarkers(
  container: HTMLElement, canvas: CanvasView, wasm: WasmBridge, bus: EventBus, open: (id: number) => void,
): void {
  const content = container.querySelector<HTMLElement>('#scroll-content');
  if (!content) return;
  const scrollContent = content;
  const layer = document.createElement('div');
  layer.className = 'memo-marker-layer';
  content.append(layer);
  const virtual = canvas.getVirtualScroll();
  const margin = new MemoMargin(container, bus, open, schedule);
  let marks: MemoMark[] | null = null;
  let generation = -1;
  let scheduled = false;
  const buttons = new Map<number, HTMLButtonElement>();

  function update() {
    scheduled = false;
    if (layer.parentElement !== scrollContent) {
      scrollContent.append(layer);
      marks = null;
    }
    if (generation !== wasm.documentGeneration) {
      generation = wasm.documentGeneration;
      marks = null;
      for (const button of buttons.values()) button.remove();
      buttons.clear();
    }
    marks ??= wasm.pageCount > 0 ? wasm.getMemos().flatMap(memo => {
      const target = wasm.getMemoTarget(memo);
      return target ? [{ memo, rect: target.rect }] : [];
    }) : [];

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
    for (const { memo, rect } of [...visible].sort((a, b) => a.rect.y - b.rect.y)) {
      let button = buttons.get(memo.id);
      if (!button) {
        button = document.createElement('button');
        button.type = 'button';
        button.textContent = '◆';
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
        memo,
        x: right - virtual.getPageWidth(page) + rect.x * zoom,
        y: desired + rect.height * zoom / 2,
        edge: right - 20,
      });
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
