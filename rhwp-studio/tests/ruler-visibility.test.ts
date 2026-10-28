import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const editor = readFileSync(new URL('../src/styles/editor.css', import.meta.url), 'utf8');
const responsive = readFileSync(new URL('../src/styles/responsive.css', import.meta.url), 'utf8')
  .replace(/\/\*[\s\S]*?\*\//g, '');
const [screen, print] = responsive.split(/@media\s+print\s*\{/);

// CSS 구조 계약만 검사한다. 실제 viewport·grid 위치 검증은 responsive E2E가 담당한다.
test('눈금자 20px grid는 모바일·낮은 높이에서도 반응형 override로 교체되지 않는다', () => {
  assert.match(editor, /#editor-area\s*\{[^}]*display:\s*grid;/s);
  assert.match(editor, /grid-template-columns:\s*20px minmax\(0, 1fr\)/);
  assert.match(editor, /grid-template-rows:\s*20px minmax\(0, 1fr\)/);
  assert.doesNotMatch(screen, /#(?:editor-area|h-ruler|v-ruler|ruler-corner)\b/);
});

test('상시 표시는 인쇄의 눈금자 숨김 계약을 바꾸지 않는다', () => {
  assert.ok(print, '인쇄 media가 있어야 한다');
  assert.match(print, /#ruler-corner,\s*#h-ruler,\s*#v-ruler\s*\{\s*display:\s*none !important/);
  assert.match(print, /#editor-area\s*\{\s*display:\s*flex/);
});

test('모바일 문서 스크롤과 확대 제스처를 유지한다', () => {
  assert.match(screen, /#scroll-container\s*\{[^}]*touch-action:\s*pan-x pan-y pinch-zoom;/s);
  assert.doesNotMatch(editor, /touch-action:\s*none/);
});
