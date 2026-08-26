import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

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

test('일반 캐럿과 드래그 캐럿 이벤트가 모두 pageIndex를 전달한다', () => {
  const input = source('src/engine/input-handler.ts');
  const emissions = input.match(
    /emit\('cursor-rect-updated', \{[\s\S]*?pageIndex: [\w.]+\.pageIndex,[\s\S]*?\}\);/g,
  ) ?? [];

  assert.equal(emissions.length, 2);
  assert.match(emissions[0], /adjustedCursorRect\.pageIndex/);
  assert.match(emissions[1], /cursorRect\.pageIndex/);
});

test('CanvasView는 캐럿·개체와 스크롤을 하나의 활성 페이지 관문으로 합친다', () => {
  const view = source('src/view/canvas-view.ts');
  const constructor = section(view, '  constructor(', '\n  /** 문서 로드 후 호출');
  const update = section(
    view,
    '  private updateActivePageSnapshot(): void {',
    '\n  /** 스크롤 중에는',
  );

  assert.match(constructor, /eventBus\.on\('cursor-rect-updated'/);
  assert.match(constructor, /eventBus\.on\('editing-page-changed'/);
  assert.match(update, /resolveActivePage\(\{/);
  assert.match(update, /visiblePages: this\.currentVisiblePages/);
  assert.match(update, /editingPageIndex: this\.editingPageIndex/);
  assert.match(update, /isHorizontalMode\(\)[\s\S]*?getPageAtPoint/);
  assert.match(update, /getRowFirstPageAtY\(viewportCenterY\)/);
  assert.match(update, /this\.eventBus\.emit\('active-page-changed', next\)/);
  assert.match(update, /'current-page-changed',[\s\S]*?next\.pageIndex/);
});

test('document-agent strict render 확인은 X/Y가 겹치는 실제 가시 페이지만 검사한다', () => {
  const view = source('src/view/canvas-view.ts');
  const refresh = section(
    view,
    '  async refreshDocumentAgentMutation(): Promise<void> {',
    '\n  private async refreshInvalidatedPageForMutation',
  );

  assert.match(refresh, /const scrollY = this\.viewportManager\.getScrollY\(\)/);
  assert.match(refresh, /const scrollX = this\.viewportManager\.getScrollX\(\)/);
  assert.match(
    refresh,
    /getVisiblePages\(\s*scrollY,\s*viewport\.height,\s*scrollX,\s*viewport\.width,\s*\)/,
  );
  assert.match(refresh, /visiblePages\.filter\(pageIndex => !this\.canvasPool\.has\(pageIndex\)\)/);
});

test('그림·표 개체 선택도 선택된 실제 페이지를 활성 페이지 입력으로 전달한다', () => {
  const input = source('src/engine/input-handler.ts');
  const picture = source('src/engine/input-handler-picture.ts');
  const tableRender = section(
    input,
    '  private renderTableObjectSelection(): void {',
    '\n  /** 그림/글상자 클릭 감지',
  );
  const pictureRender = section(
    picture,
    'export function renderPictureObjectSelection(this: any): void {',
    '\nexport function exitPictureObjectSelectionIfNeeded',
  );

  assert.match(tableRender, /this\.eventBus\.emit\('editing-page-changed', selectedPage\)/);
  assert.match(pictureRender, /this\.eventBus\.emit\('editing-page-changed', pageIndex\)/);
  assert.match(pictureRender, /this\.eventBus\.emit\('editing-page-changed', p\)/);
});

test('문서를 교체할 때 이전 활성 페이지 snapshot을 소비처에서 지운다', () => {
  const view = source('src/view/canvas-view.ts');
  const reset = section(view, '  private reset(): void {', '\n  private releaseAllRenderedPages');

  assert.match(reset, /const hadActivePage = this\.activePageSnapshot !== null/);
  assert.match(reset, /this\.activePageSnapshot = null/);
  assert.match(reset, /this\.eventBus\.emit\('active-page-changed', null\)/);
});
