/** 링크의 실제 글자 선택 영역 안에서만 hover/열기/우클릭 대상을 찾는다. */
import { hyperlinkTarget, applyHyperlinkFormat } from '@/core/hyperlink';
import type { HyperlinkInfo, HyperlinkTarget } from '@/core/hyperlink';
import type { DocumentPosition } from '@/core/types';
import { showToast } from '@/ui/toast';

type LinkHit = { target: HyperlinkTarget; link: HyperlinkInfo; position: DocumentPosition };

export function hyperlinkAtPointer(self: any, e: MouseEvent): LinkHit | null {
  if (!self.wasm?.getHyperlinkContext || !self.canEditHyperlink?.() || self.connectorDrawingMode
    || self.polygonDrawingMode || self.imagePlacementMode || self.textboxPlacementMode) return null;
  const content = self.container.querySelector('#scroll-content');
  if (!content) return null;
  try {
    const box = content.getBoundingClientRect();
    const x = e.clientX - box.left, y = e.clientY - box.top;
    const page = self.virtualScroll.getPageAtPoint(x, y);
    if (page < 0 || page >= self.wasm.pageCount) return null;
    const zoom = self.viewportManager.getZoom();
    const px = (x - self.virtualScroll.getPageLeftResolved(page, content.clientWidth)) / zoom;
    const py = (y - self.virtualScroll.getPageOffset(page)) / zoom;
    const hit = self.wasm.hitTest(page, px, py);
    const target = hyperlinkTarget(hit);
    const context = self.wasm.getHyperlinkContext(target);
    for (const link of context.links as HyperlinkInfo[]) {
      if (!/^https?:\/\//i.test(link.uri)) continue;
      // caret hit-test는 글자 오른쪽 절반에서 다음 offset을 반환할 수 있다.
      if (hit.charOffset < link.start || hit.charOffset > link.end) continue;
      const path = target.cellPath;
      const rects = path.length ? self.wasm.getSelectionRectsInCellByPath(target.section, target.para,
        JSON.stringify(path.map(([controlIndex, cellIndex, cellParaIndex]) => ({ controlIndex, cellIndex, cellParaIndex }))),
        path[path.length - 1][2], link.start, path[path.length - 1][2], link.end)
        : self.wasm.getSelectionRects(target.section, target.para, link.start, target.para, link.end);
      if (rects.some((r: any) => r.pageIndex === page && px >= r.x && px <= r.x + r.width && py >= r.y && py <= r.y + r.height)) {
        return { target, link, position: { ...hit, charOffset: link.start } };
      }
    }
  } catch { /* 미지원 문서 컨텍스트는 일반 글자 편집으로 처리한다. */ }
  return null;
}

export function hoverHyperlink(self: any, e: MouseEvent): boolean {
  if (self.hyperlinkHover) {
    self.container.removeAttribute('title');
    if (self.container.style.cursor === 'pointer') self.container.style.cursor = '';
    self.hyperlinkHover = false;
  }
  const hit = hyperlinkAtPointer(self, e);
  if (!hit) return false;
  self.container.title = `${hit.link.uri}\n클릭하여 열기 · 우클릭하여 고치기/지우기`;
  self.container.style.cursor = 'pointer';
  self.hyperlinkHover = true;
  return true;
}

export function rememberHyperlinkClick(self: any, e: MouseEvent): void {
  self.hyperlinkClick = null;
  if (e.button !== 0 || e.shiftKey || e.altKey) return;
  const hit = hyperlinkAtPointer(self, e);
  if (hit) self.hyperlinkClick = { hit, x: e.clientX, y: e.clientY, generation: self.wasm.documentGeneration };
}

export function followHyperlinkClick(self: any, e: MouseEvent): void {
  const pending = self.hyperlinkClick;
  self.hyperlinkClick = null;
  if (!pending || e.button !== 0 || Math.hypot(e.clientX - pending.x, e.clientY - pending.y) > 3
    || pending.generation !== self.wasm.documentGeneration || self.cursor.hasSelection()) return;
  const current = hyperlinkAtPointer(self, e);
  if (!current || current.link.fieldId !== pending.hit.link.fieldId || current.link.uri !== pending.hit.link.uri
    || JSON.stringify(current.target) !== JSON.stringify(pending.hit.target)) return;
  try {
    const url = new URL(current.link.uri);
    if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) return;
    // 실제 사용자 mouseup 안에서 열고 원본 탭의 opener 접근을 끊는다.
    const opened = window.open('about:blank', '_blank');
    if (!opened) { showToast({ message: '팝업이 차단되었습니다. 이 사이트의 팝업을 허용해 주세요.' }); return; }
    opened.opener = null;
    opened.location.replace(url.href);
    // 한컴의 방문 색과 같이 글자 서식으로 반영한다. 저장·PDF·undo에도 같은 색이 남는다.
    // 읽기 전용/양식 편집에서는 executeOperation의 기존 정책이 변경을 차단한다.
    self.executeOperation({
      kind: 'snapshot', operationType: 'visitHyperlink', selectionBefore: null,
      operation: (wasm: any) => {
        const { target, link } = current;
        const path = JSON.stringify(target.cellPath.map(([controlIndex, cellIndex, cellParaIndex]) => ({ controlIndex, cellIndex, cellParaIndex })));
        const props = target.cellPath.length
          ? wasm.getCellCharPropertiesAtByPath(target.section, target.para, path, link.start)
          : wasm.getCharPropertiesAt(target.section, target.para, link.start);
        if (props.textColor?.toLowerCase() === '#800080') return null;
        applyHyperlinkFormat(wasm, target, link.start, link.end, '#800080');
        return self.cursor.getPosition();
      },
    });
  } catch (error) {
    showToast({ message: error instanceof Error ? error.message : String(error) });
  }
}
