import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const canvasView = readFileSync(
  new URL('../src/view/canvas-view.ts', import.meta.url),
  'utf8',
);
const ruler = readFileSync(new URL('../src/view/ruler.ts', import.meta.url), 'utf8');

test('문서 화면이 새로 서면 눈금자에게 알린다', () => {
  // loadDocument 안, 쪽 정보를 다 세운 뒤여야 한다 — 그 전에 알리면 눈금자가 빈 쪽을 읽는다.
  const body = canvasView.match(
    /async loadDocument\(\): Promise<void> \{(?<body>[\s\S]*?)\n  \}\n/,
  )?.groups?.body;
  assert.ok(body, 'loadDocument 본문을 찾지 못했다');
  assert.match(body, /document-view-loaded/);
  assert.ok(
    body.indexOf('this.updateVisiblePages()') < body.indexOf('document-view-loaded'),
    '쪽 배치를 마친 뒤에 알려야 한다',
  );
});

test('눈금자는 문서 로드 알림에 크기·paint를 함께 처리하는 갱신을 예약한다', () => {
  // 캐럿·스크롤·확대 이벤트는 값이 그대로면 오지 않는다. 그 셋에만 기대면 문서를 열어도
  // 빈 쪽 단계에서 그린 눈금 없는 띠가 남는다.
  // bitmap 크기 변경과 두 축 paint의 실제 실행 순서는 ruler-resize의 행위 테스트가 검증한다.
  assert.match(ruler, /eventBus\.on\('document-view-loaded', \(\) => this\.scheduleUpdate\(\)\)/);
});
