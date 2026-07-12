import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const source = readFileSync(new URL('../src/ui/toolbar.ts', import.meta.url), 'utf8');
const styles = readFileSync(new URL('../src/styles/style-bar.css', import.meta.url), 'utf8');

test('글꼴 메뉴는 문서 글꼴을 기본으로 열고 시스템 글꼴은 명시 선택 때만 조회한다', () => {
  const initStart = source.indexOf('initFontDropdown(docFonts?: string[]): void');
  const initEnd = source.indexOf('private refreshFontDropdown()', initStart);
  const initMethod = source.slice(initStart, initEnd);
  const entriesStart = source.indexOf('private getFontMenuEntries(');
  const entriesEnd = source.indexOf('private uniqueFontMenuEntries(', entriesStart);
  const entriesMethod = source.slice(entriesStart, entriesEnd);

  assert.match(initMethod, /this\.fontMenuCategory = this\.fontMenuDocumentFonts\.length > 0 \? 'document' : 'current'/);
  assert.match(source, /event\.preventDefault\(\);\s*this\.toggleFontMenu\(\);/);
  assert.match(entriesMethod, /case 'system':\s*return getLocalFonts\(\)/);
  assert.match(entriesMethod, /case 'all':[\s\S]*getLocalFonts\(\)/);
  assert.doesNotMatch(entriesMethod.slice(0, entriesMethod.indexOf("case 'system':")), /getLocalFonts\(\)/);
});

test('한컴형 글꼴 메뉴는 범주 목록과 기존 글꼴 적용 이벤트를 함께 사용한다', () => {
  assert.match(source, /label: '모든 글꼴'/);
  assert.match(source, /label: '현재 글꼴'/);
  assert.match(source, /label: '문서 글꼴'/);
  assert.match(source, /label: '대표 글꼴'/);
  assert.match(source, /label: '시스템 글꼴'/);
  assert.match(source, /menu\.className = 'font-picker-menu'/);
  assert.match(source, /this\.fontName\.dispatchEvent\(new Event\('change', \{ bubbles: true \}\)\)/);
});

test('시스템 글꼴 목록은 고정 높이 메뉴 안에서 스크롤된다', () => {
  assert.match(styles, /\.font-picker-menu \{[\s\S]*height: min\(350px, calc\(100vh - 8px\)\)/);
  assert.match(styles, /\.font-picker-content \{[\s\S]*min-height: 0/);
  assert.match(styles, /\.font-picker-list \{[\s\S]*flex: 1 1 auto;[\s\S]*overflow: auto/);
});

test('로컬 글꼴 재감지는 캐럿 글꼴이 아니라 문서 전체 글꼴 목록을 보존한다', () => {
  const refreshStart = source.indexOf('private refreshFontDropdown(): void');
  const refreshEnd = source.indexOf('/** 문서 로드 시 스타일 목록', refreshStart);
  const refreshMethod = source.slice(refreshStart, refreshEnd);

  assert.match(refreshMethod, /this\.initFontDropdown\(this\.fontMenuDocumentFonts\)/);
  assert.doesNotMatch(refreshMethod, /this\.initFontDropdown\(this\.lastFontFamilies\)/);
});
