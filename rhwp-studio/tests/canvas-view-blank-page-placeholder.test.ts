import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { VirtualScroll } from '../src/view/virtual-scroll.ts';

// 문서를 교체하면 prepareDocumentLoad 가 scrollContent 를 비우고, 새 문서 첫 쪽이 그려질
// 때까지 회색 작업 영역(--doc-workspace)이 그대로 드러난다. 그 구간이 로드 진행 표시와
// 겹쳐 화면이 깜빡이는 것처럼 보였다. 빈 흰 쪽 자리표시자가 그 구간을 덮는다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function section(text: string, startMarker: string, endMarker: string): string {
  const start = text.indexOf(startMarker);
  const end = text.indexOf(endMarker, start);
  assert.ok(start >= 0 && end > start, `${startMarker} 범위를 찾을 수 있어야 한다`);
  return text.slice(start, end);
}

test('문서 교체는 비운 직후 빈 쪽 자리표시자를 놓는다', () => {
  const view = source('src/view/canvas-view.ts');
  const prepare = section(view, '  prepareDocumentLoad(): void {', '\n  /**');

  const resetIndex = prepare.indexOf('this.reset();');
  const showIndex = prepare.indexOf('this.showBlankPagePlaceholder();');
  assert.ok(resetIndex >= 0, '문서 교체는 이전 문서 뷰를 비워야 한다');
  assert.ok(showIndex > resetIndex, '비운 뒤에 빈 쪽 자리표시자를 놓아야 한다');
});

test('빈 쪽 자리표시자는 첫 쪽을 그린 뒤에만 걷는다', () => {
  const view = source('src/view/canvas-view.ts');
  const load = section(view, '  async loadDocument(): Promise<void> {', '\n  /** WASM 문서 교체');

  const visibleIndex = load.indexOf("this.updateVisiblePages('initial');");
  const clearIndex = load.indexOf('this.clearBlankPagePlaceholder();');
  assert.ok(visibleIndex >= 0, '문서 로드는 보이는 쪽을 그려야 한다');
  assert.ok(clearIndex > visibleIndex, '쪽을 그린 뒤에 자리표시자를 걷어야 한다');
});

test('자리표시자는 문서 용지 색을 쓴다', () => {
  const css = source('src/styles/editor.css');
  const rule = section(css, '#scroll-content .page-placeholder {', '}');

  assert.match(rule, /background:\s*var\(--doc-paper\)/);
});

test('자리표시자 배치는 실제 쪽 여백과 같은 값을 쓴다', () => {
  const virtualScroll = new VirtualScroll();
  const pages = [{ width: 800, height: 1000 }] as never;
  virtualScroll.setPageDimensions(pages, 1, 1200);

  assert.equal(virtualScroll.getPageGap(), 10);
  assert.equal(virtualScroll.getPageOffset(0), virtualScroll.getPageGap());
});
