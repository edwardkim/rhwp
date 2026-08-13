import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('플랫 스킨 CSS의 모든 규칙은 data-theme-skin="flat" 스코프 아래에 있다', () => {
  const css = source('src/styles/theme-flat.css');
  const selectors = css
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .split('}')
    .map((block) => block.split('{')[0]?.trim())
    .filter((selector): selector is string => !!selector);

  assert.ok(selectors.length > 0, '규칙이 최소 1개는 있어야 한다');
  for (const selector of selectors) {
    assert.match(
      selector,
      /\[data-theme-skin="flat"\]/,
      `옵트인 스코프가 없는 셀렉터: ${selector}`,
    );
  }
});

test('플랫 스킨의 라이트 팔레트는 다크 모드를 덮지 않도록 가드된다', () => {
  const css = source('src/styles/theme-flat.css');
  assert.match(css, /:root\[data-theme-skin="flat"\]:not\(\[data-theme-effective="dark"\]\)/);
  // 색 변수 재정의는 다크 가드 블록에만 존재해야 한다 — 가드 없는 :root 단독 블록의
  // 변수는 형태 값(radius/shadow)만 허용한다.
  const ungardedRoot = css.match(/:root\[data-theme-skin="flat"\]\s*\{([\s\S]*?)\}/);
  if (ungardedRoot) {
    assert.doesNotMatch(ungardedRoot[1], /--ui-bg|--accent-primary|--ruler-bg/);
  }
});

test('보기 메뉴는 스킨 선택 항목을 노출하고 FOUC 방지 스크립트가 스킨을 읽는다', () => {
  const html = source('index.html');
  assert.match(html, /data-cmd="view:skin-default"[^>]*data-theme-skin-choice="default"/);
  assert.match(html, /data-cmd="view:skin-flat"[^>]*data-theme-skin-choice="flat"/);

  const themeInit = source('public/theme-init.js');
  assert.match(themeInit, /theme\.skin === 'flat'/);
  assert.match(themeInit, /themeSkin = skin/);
});
