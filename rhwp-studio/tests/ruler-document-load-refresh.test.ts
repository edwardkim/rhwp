import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const canvasView = readFileSync(
  new URL('../src/view/canvas-view.ts', import.meta.url),
  'utf8',
);
const ruler = readFileSync(new URL('../src/view/ruler.ts', import.meta.url), 'utf8');

test('문서 화면이 새로 서면 눈금자에게 알린다', () => {
  // 동기 finish가 쪽 정보를 다 세운 뒤에만 바깥 loadDocument가 알림을 보낸다.
  const body = canvasView.match(
    /async loadDocument[^{]*\{(?<body>[\s\S]*?)\n  \}\n\n  \/\*\*/,
  )?.groups?.body;
  assert.ok(body, 'loadDocument 본문을 찾지 못했다');
  const finish = canvasView.match(
    /private finishDocumentLoad[^{]*\{(?<body>[\s\S]*?)\n  \}\n\n  \/\*\*/,
  )?.groups?.body;
  assert.ok(finish, 'finishDocumentLoad 본문을 찾지 못했다');
  assert.match(finish, /this\.updateVisiblePages\(\)/);
  assert.match(body, /document-view-loaded/);
  assert.ok(
    body.indexOf('this.finishDocumentLoad(') < body.indexOf('document-view-loaded'),
    '쪽 배치를 마친 뒤에 알려야 한다',
  );
});

test('눈금자는 문서 로드 알림에 크기와 눈금을 모두 다시 잡는다', () => {
  // 캐럿·스크롤·확대 이벤트는 값이 그대로면 오지 않는다. 그 셋에만 기대면 문서를 열어도
  // 빈 쪽 단계에서 그린 눈금 없는 띠가 남는다.
  const handler = ruler.match(
    /eventBus\.on\('document-view-loaded', \(\) => \{(?<body>[^}]*)\}/,
  )?.groups?.body;
  assert.ok(handler, '눈금자가 document-view-loaded 를 구독하지 않는다');
  assert.match(handler, /this\.resize\(\)/);
  assert.match(handler, /this\.scheduleUpdate\(\)/);
});
