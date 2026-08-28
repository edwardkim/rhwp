import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
const main = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
const commands = readFileSync(new URL('../src/command/commands/view.ts', import.meta.url), 'utf8');
const dialog = readFileSync(new URL('../src/ui/zoom-dialog.ts', import.meta.url), 'utf8');
const style = readFileSync(new URL('../src/style.css', import.meta.url), 'utf8');
const viewport = readFileSync(new URL('../src/view/viewport-manager.ts', import.meta.url), 'utf8');
const dialogBase = readFileSync(new URL('../src/ui/dialog.ts', import.meta.url), 'utf8');

test('보기 메뉴와 상태 표시줄 통합 배율 버튼은 같은 확대/축소 대화상자 커맨드를 연다', () => {
  assert.match(html, /data-cmd="view:zoom-dialog"[^>]*>[\s\S]*?화면 확대\/축소/);
  assert.match(html, /<button[^>]*id="sb-zoom-display"[^>]*>[\s\S]*?id="sb-zoom-val"/);
  assert.match(main, /sb-zoom-display[\s\S]*?dispatcher\.dispatch\('view:zoom-dialog'\)/);
  assert.match(commands, /id: 'view:zoom-dialog'[\s\S]*?opensDialog: true[\s\S]*?new ZoomDialog/);
});

test('대화상자는 한컴 비율·쪽 모양 선택을 제공하고 저장소 디자인 토큰 CSS를 쓴다', () => {
  for (const label of [
    '폭 맞춤', '쪽 맞춤', '사용자 정의',
    '자동', '한 쪽', '두 쪽', '맞쪽', '여러 쪽',
    '세로 방향', '가로 방향', '마우스 휠을 사용하여 좌우로 스크롤하기',
  ]) {
    assert.ok(dialog.includes(label), `${label} 선택지가 있어야 한다`);
  }
  assert.match(style, /@import '\.\/styles\/zoom-dialog\.css';/);
});

test('여러 쪽에서는 적용되지 않는 비율 선택을 비활성화한다', () => {
  assert.match(
    dialog,
    /querySelectorAll<HTMLInputElement>\('input\[name="zoom-choice"\]'\)[\s\S]*?input\.disabled\s*=\s*multiple/,
  );
  assert.match(dialog, /this\.customInput\.disabled\s*=\s*multiple/);
});

test('잘못된 사용자 배율은 대화상자를 유지하고 오류 접근성·포커스를 제공한다', () => {
  assert.match(dialogBase, /if \(shouldClose !== false\) this\.hide\(\)/);
  assert.match(dialog, /protected onConfirm\(\): boolean/);
  assert.match(dialog, /validateCustomZoomPercent\(this\.customInput\.value\)/);
  assert.match(dialog, /this\.showCustomZoomError\(validation\.message\);\s*return false;/);
  assert.match(dialog, /setAttribute\('aria-describedby', CUSTOM_ZOOM_ERROR_ID\)/);
  assert.match(dialog, /setAttribute\('aria-invalid', 'true'\)/);
  assert.match(dialog, /setAttribute\('role', 'alert'\)/);
  assert.match(dialog, /this\.customInput\.focus\(\);\s*this\.customInput\.select\(\);/);
});

test('사용자 배율 입력의 Enter는 확인 버튼을 사용하고 종료 시 listener를 제거한다', () => {
  assert.match(
    dialog,
    /event\.target === this\.customInput && event\.key === 'Enter'[\s\S]*?confirm\.click\(\)/,
  );
  assert.match(dialog, /document\.addEventListener\('keydown', this\.inputEnterHandler, true\)/);
  assert.match(
    dialog,
    /override hide\(\): void[\s\S]*?document\.removeEventListener\('keydown', this\.inputEnterHandler, true\)/,
  );
});

test('메뉴·대화상자는 한 fit metrics helper와 resolver를 공유한다', () => {
  assert.match(commands, /function getZoomFitMetrics[\s\S]*?resolveZoomFitZoom/);
  assert.doesNotMatch(commands, /calculateFitPageZoom/);
  assert.doesNotMatch(commands, /calculateArrangementFitWidthZoom/);
});

test('확대/축소 적용은 사용자 보기 설정과 보기 이벤트만 바꾸고 문서를 dirty로 만들지 않는다', () => {
  const start = commands.indexOf("id: 'view:zoom-dialog'");
  const end = commands.indexOf("id: 'view:zoom-fit-page'", start);
  const command = commands.slice(start, end);
  assert.match(command, /userSettings\.setPageViewSettings/);
  assert.match(command, /eventBus\.emit\('page-view-settings-changed'/);
  assert.match(
    command,
    /zoom:\s*\{[\s\S]*?value:\s*zoom[\s\S]*?fitMode:[\s\S]*?anchor:\s*CENTER_ZOOM_ANCHOR/,
  );
  assert.doesNotMatch(command, /vm\.setZoom\(/);
  assert.doesNotMatch(command, /document-(?:changed|mutated)/);
});

test('상황 선은 한글 2024 순서로 축소·범위·확대·통합 배율 버튼을 제공한다', () => {
  const orderedIds = [
    'sb-zoom-fit-width',
    'sb-zoom-fit',
    'sb-zoom-out',
    'sb-zoom-range',
    'sb-zoom-in',
    'sb-zoom-display',
  ];
  let cursor = -1;
  for (const id of orderedIds) {
    const next = html.indexOf(`id="${id}"`);
    assert.ok(next > cursor, `${id}가 한글 2024 순서에 있어야 한다`);
    cursor = next;
  }
  assert.doesNotMatch(html, /id="sb-zoom-100"/);
  assert.doesNotMatch(html, /id="sb-zoom-menu"/);
  assert.match(html, /class="stb-zoom-range-wrap[^"]*"[\s\S]*?id="sb-zoom-range"[^>]*type="range"[^>]*min="0"[^>]*max="1000"[\s\S]*?stb-zoom-neutral-mark/);
  assert.match(main, /sb-zoom-range[\s\S]*?zoomSliderPositionToPercent[\s\S]*?setZoom/);
  assert.match(main, /sb-zoom-display[\s\S]*?view:zoom-dialog/);
});

test('상황 선 확대·축소는 플랫폼 단축키를 호버에 표시하고 별도 키 리스너를 만들지 않는다', () => {
  assert.match(commands, /id: 'view:zoom-in'[\s\S]*?shortcutLabel: 'Ctrl\+\+'/);
  assert.match(commands, /id: 'view:zoom-out'[\s\S]*?shortcutLabel: 'Ctrl\+-'/);
  assert.match(main, /zoomPercentShortcutTitle\('확대', 'Ctrl\+\+'/);
  assert.match(main, /zoomPercentShortcutTitle\('축소', 'Ctrl\+-'/);
  const setupStart = main.indexOf('function setupZoomControls()');
  const setupEnd = main.indexOf('\nlet totalSections', setupStart);
  assert.doesNotMatch(
    main.slice(setupStart, setupEnd),
    /document\.addEventListener\('keydown'/,
  );
});

test('ViewportManager는 여러 쪽 최소 배율과 500% 프리셋을 모두 허용한다', () => {
  assert.match(viewport, /MIN_DOCUMENT_ZOOM/);
  assert.match(viewport, /MAX_DOCUMENT_ZOOM/);
});
