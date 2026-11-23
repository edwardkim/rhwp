import type { EventBus } from '@/core/event-bus';
import type { MemoInfo } from '@/core/memo-editor';
import type { CursorRect, SelectionRect } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { CanvasView } from '@/view/canvas-view';
import { MemoMargin, type MemoAction } from './memo-margin';

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
  const highlightLayer = document.createElement('div');
  highlightLayer.className = 'memo-highlight-layer';
  content.append(highlightLayer);
  const virtual = canvas.getVirtualScroll();
  let marks: MemoMark[] | null = null;
  let generation = -1;
  let scheduled = false;
  const margin = new MemoMargin(container, bus, open, schedule);

  function update() {
    scheduled = false;
    if (highlightLayer.parentElement !== scrollContent) {
      scrollContent.append(highlightLayer);
      marks = null;
    }
    if (generation !== wasm.documentGeneration) {
      generation = wasm.documentGeneration;
      marks = null;
      margin.reset();
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
    margin.setHasAnchoredMemos(marks.length > 0);
    highlightLayer.replaceChildren();

    const zoom = canvas.getViewportManager().getZoom();
    const pages = new Set(virtual.getVisiblePages(
      container.scrollTop, container.clientHeight, container.scrollLeft, container.clientWidth,
    ));
    const locate = (rect: CursorRect) => {
      const page = rect.pageIndex;
      const left = virtual.getPageLeftResolved(page, scrollContent.clientWidth);
      return {
        x: left + rect.x * zoom,
        y: virtual.getPageOffset(page) + (rect.y + rect.height) * zoom,
        edge: left + virtual.getPageWidth(page),
      };
    };
    const draftAnchor = margin.draftAnchor;
    const draft = draftAnchor ? wasm.getMemoTarget(draftAnchor, true) : null;
    const located = marks.filter(({ rect }) => pages.has(rect.pageIndex))
      .map(({ memo, rect, replies }) => ({ memo, replies, ...locate(rect) }));
    if (margin.annotations) {
      const paint = (rect: CursorRect, highlight: SelectionRect[], active: boolean, isDraft = false) => {
        for (const r of highlight.length ? highlight : [{ ...rect, width: 2 }]) {
          if (!pages.has(r.pageIndex)) continue;
          const mark = document.createElement('span');
          mark.classList.toggle('is-active', active);
          if (isDraft) mark.classList.add('is-draft');
          const left = virtual.getPageLeftResolved(r.pageIndex, scrollContent.clientWidth);
          mark.style.left = String(left + r.x * zoom) + 'px';
          mark.style.top = String(virtual.getPageOffset(r.pageIndex) + r.y * zoom) + 'px';
          mark.style.width = String(Math.max(2, r.width * zoom)) + 'px';
          mark.style.height = String(r.height * zoom) + 'px';
          highlightLayer.append(mark);
        }
      };
      for (const { memo, rect, highlight } of marks) paint(rect, highlight, margin.active === memo.id);
      if (draft) paint(draft.rect, draft.highlight, true, true);
    }
    margin.update(located, draft ? locate(draft.rect) : null);
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
