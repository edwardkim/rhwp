import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
const main = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
const commands = readFileSync(new URL('../src/command/commands/view.ts', import.meta.url), 'utf8');
const dialog = readFileSync(new URL('../src/ui/zoom-dialog.ts', import.meta.url), 'utf8');
const style = readFileSync(new URL('../src/style.css', import.meta.url), 'utf8');
const viewport = readFileSync(new URL('../src/view/viewport-manager.ts', import.meta.url), 'utf8');

test('보기 메뉴와 상태 표시줄 배율 버튼은 같은 확대/축소 대화상자 커맨드를 연다', () => {
  assert.match(html, /data-cmd="view:zoom-dialog"[^>]*>[\s\S]*?화면 확대\/축소/);
  assert.match(html, /<button[^>]*id="sb-zoom-val"/);
  assert.match(main, /sb-zoom-val[\s\S]*?dispatcher\.dispatch\('view:zoom-dialog'\)/);
  assert.match(commands, /id: 'view:zoom-dialog'[\s\S]*?opensDialog: true[\s\S]*?new ZoomDialog/);
});

test('대화상자는 한컴 비율·쪽 모양 선택을 제공하고 저장소 디자인 토큰 CSS를 쓴다', () => {
  for (const label of [
    '폭 맞춤', '쪽 맞춤', '사용자 정의',
    '자동', '한 쪽', '두 쪽', '맞쪽', '여러 쪽',
  ]) {
    assert.ok(dialog.includes(label), `${label} 선택지가 있어야 한다`);
  }
  assert.match(style, /@import '\.\/styles\/zoom-dialog\.css';/);
});

test('확대/축소 적용은 사용자 보기 설정과 보기 이벤트만 바꾸고 문서를 dirty로 만들지 않는다', () => {
  const start = commands.indexOf("id: 'view:zoom-dialog'");
  const end = commands.indexOf("id: 'view:zoom-fit-page'", start);
  const command = commands.slice(start, end);
  assert.match(command, /userSettings\.setPageArrangement/);
  assert.match(command, /eventBus\.emit\('page-arrangement-changed'/);
  assert.doesNotMatch(command, /document-(?:changed|mutated)/);
});

test('ViewportManager는 여러 쪽 최소 배율과 500% 프리셋을 모두 허용한다', () => {
  assert.match(viewport, /MIN_DOCUMENT_ZOOM/);
  assert.match(viewport, /MAX_DOCUMENT_ZOOM/);
});
