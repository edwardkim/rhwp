import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const ruler = readFileSync(new URL('../src/view/ruler.ts', import.meta.url), 'utf8');

function section(startMarker: string, endMarker: string): string {
  const start = ruler.indexOf(startMarker);
  const end = ruler.indexOf(endMarker, start);
  assert.ok(start >= 0 && end > start, `${startMarker} 범위를 찾을 수 있어야 한다`);
  return ruler.slice(start, end);
}

test('Ruler는 CanvasView의 활성 페이지 snapshot을 직접 구독한다', () => {
  const constructor = section('  constructor(', '\n  private palette()');

  assert.match(constructor, /eventBus\.on\('active-page-changed'/);
  assert.match(constructor, /value\.source === 'editing'/);
  assert.match(constructor, /value\.source === 'viewport'/);
  assert.match(constructor, /this\.activePageSnapshot = value/);
});

test('가로 눈금 좌표와 용지 정보는 활성 pageIdx 하나에서 나온다', () => {
  const screenLeft = section(
    '  private getPageScreenLeft(',
    '\n  /** 드래그 중에는',
  );
  const horizontal = section(
    '  private drawHorizontal(): void {',
    '\n  /** 세로 눈금자',
  );

  assert.match(screenLeft, /getPageLeftResolved\(\s*pageIdx,/);
  assert.match(horizontal, /const pageIdx = this\.rulerPageIndex\(\)/);
  assert.match(horizontal, /this\.wasm\.getPageInfo\(pageIdx\)/);
  assert.match(horizontal, /this\.getPageScreenLeft\(pageIdx, scrollX\)/);
  assert.doesNotMatch(horizontal, /getVisiblePages\(/);
  assert.doesNotMatch(horizontal, /getPageLeftResolved\(\s*0,/);
});

test('세로 눈금과 핀은 활성 페이지 한 쪽에만 속한다', () => {
  const vertical = section(
    '  private drawVertical(): void {',
    '\n  /** 리소스 정리',
  );

  assert.match(vertical, /const pageIdx = this\.rulerPageIndex\(\)/);
  assert.match(vertical, /getPageOffset\(pageIdx\)/);
  assert.match(vertical, /getPageInfo\(pageIdx\)/);
  assert.match(vertical, /this\.vPins\.push\(\{ kind: 'top',[\s\S]*?pageIdx/);
  assert.doesNotMatch(vertical, /getVisiblePages\(/);
  assert.doesNotMatch(vertical, /for \(const pageIdx/);
});

test('여러 쪽 배치에서도 쪽 핀을 표시하되 viewport fallback 문단 핀은 숨긴다', () => {
  const horizontal = section(
    '  private drawHorizontal(): void {',
    '\n  /** 세로 눈금자',
  );

  assert.match(horizontal, /this\.hPins\.push\([\s\S]*?'pageMarginLeft'/);
  assert.match(horizontal, /if \(this\.hasParaInfo && editingContext\)/);
  assert.doesNotMatch(horizontal, /if \(this\.virtualScroll\.isGridMode\(\)\)/);
});

test('가로 핀 드래그는 시작 페이지를 저장해 같은 pageIdx로 커밋한다', () => {
  const down = section('  private onHPinDown(', '\n  /** 마주보는 두 핀');
  const commit = section('  private commitHDrag(', '\n  /** 세로 핀 드롭');

  assert.match(down, /pageIdx: this\.hPageIdx/);
  assert.match(commit, /this\.hDropContext\(drag\.pageIdx\)/);
  assert.match(commit, /pageIdx,/);
});
