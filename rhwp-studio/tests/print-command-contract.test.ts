import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const commandSource = readFileSync(
  new URL('../src/command/commands/file.ts', import.meta.url),
  'utf8',
);
const indexHtml = readFileSync(
  new URL('../index.html', import.meta.url),
  'utf8',
);

test('PDF로 저장과 인쇄는 same-origin iframe print pipeline을 공유한다', () => {
  assert.match(commandSource, /runBrowserPrint\(services, 'pdf'\)/);
  assert.match(commandSource, /runBrowserPrint\(services, 'print'\)/);
  assert.match(commandSource, /surface\.window\.print\(\)/);
  assert.doesNotMatch(commandSource, /window\.open\(/);
  assert.doesNotMatch(commandSource, /about:blank/);
});

test('print pipeline은 명시적인 print profile SVG만 사용한다', () => {
  assert.match(commandSource, /renderPageSvgWithProfile\(i, 'print'\)/);
  assert.doesNotMatch(commandSource, /wasm\.renderPageSvg\(i\)/);
});

test('파일 메뉴는 별도 PDF 진입점과 브라우저의 남은 단계를 노출한다', () => {
  assert.match(indexHtml, /data-cmd="file:print-to-pdf"/);
  assert.match(indexHtml, />PDF로 저장…</);
  assert.match(indexHtml, /대상 → PDF로 저장/);
  assert.match(indexHtml, /data-cmd="file:print"/);
});

test('print pipeline은 저장 handle·파일명·dirty 상태를 변경하지 않는다', () => {
  const printSection = commandSource.slice(
    commandSource.indexOf('async function runBrowserPrint'),
    commandSource.indexOf('export const fileCommands'),
  );
  assert.doesNotMatch(printSection, /\.fileName\s*=/);
  assert.doesNotMatch(printSection, /\.currentFileHandle\s*=/);
  assert.doesNotMatch(printSection, /documentState\.(markDirty|markClean)\(/);
});
