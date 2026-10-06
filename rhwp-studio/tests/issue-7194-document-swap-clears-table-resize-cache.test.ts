// [Issue #7194] 문서를 열어도 표 크기 조절 캐시가 초기화되지 않는다.
//
// `clearTableResizeRuntimeCache()` 는 하나뿐인 초기화 루틴인데, 문서 열기 쪽 구독이
// `open-document-bytes` 이벤트에만 걸려 있었다. 바이트로 문서를 여는 **공통 깔때기**는
// `main.ts loadBytes()` 이고, 그 이벤트를 거치는 호출부는 여섯 중 하나뿐이다 —
// 드롭 · 파일 input · `?url=` · 자동저장 복구 · 호스트 API 는 깔때기를 직접 부른다.
//
// `cachedTableRef` 는 `{sec, ppi, ci}` 만 담아 문서가 바뀌어도 신선도 검사를 통과하므로,
// 그 경로로 연 문서 위에 이전 문서의 칸 좌표가 살아남아 가이드가 엉뚱한 자리에 그려지고
// `resolveTableResizeHit` 가 옛 `cellIdx` 로 엉뚱한 행을 잡는다 (3184241 신고).
//
// 그래서 신호를 이벤트에서 깔때기로 옮겼다 — `loadBytes()` 가 문서를 갈아치운 직후
// `document-swapped` 를 내고 `InputHandler` 가 그것을 듣는다.
//
// # 이 시험이 잠그는 것
//
// 초기화가 **깔때기에 붙어 있는지**를 소스로 고정한다. 구독을 다시 좁은 이벤트로 옮기거나
// 깔때기에서 신호를 빼면 실패한다.
//
// # 잠그지 않는 것
//
// 캐시 내용물(무엇을 비우는지)은 `table-resize-undo-cache-1491.test.ts` 가 잠근다.

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function loadBytesBody(): string {
  const main = source('src/main.ts');
  const start = main.indexOf('async function loadBytes(');
  assert.notEqual(start, -1, 'loadBytes 를 찾지 못했다');
  const next = main.indexOf('\nasync function ', start + 1);
  const alt = main.indexOf('\nfunction ', start + 1);
  const end = [next, alt].filter((i) => i !== -1).sort((a, b) => a - b)[0];
  return main.slice(start, end === undefined ? undefined : end);
}

test('문서 교체 신호는 공통 깔때기 loadBytes 가 낸다', () => {
  const body = loadBytesBody();
  assert.match(
    body,
    /eventBus\.emit\('document-swapped'\)/,
    "loadBytes 가 'document-swapped' 를 내야 모든 열기 경로가 같은 보장을 받는다",
  );
});

test('표 resize 캐시 정리는 깔때기 신호를 듣는다', () => {
  const inputHandler = source('src/engine/input-handler.ts');
  const start = inputHandler.indexOf("eventBus.on('document-swapped'");
  assert.notEqual(start, -1, "InputHandler 가 'document-swapped' 를 구독해야 한다");
  const block = inputHandler.slice(start, start + 400);
  assert.match(block, /clearTableResizeRuntimeCache\(\)/, '문서 교체에서 표 resize 캐시를 비워야 한다');
});

test('좁은 open-document-bytes 구독으로 되돌아가지 않는다', () => {
  const inputHandler = source('src/engine/input-handler.ts');
  const start = inputHandler.indexOf("eventBus.on('open-document-bytes'");
  if (start === -1) return; // 구독 자체가 없으면 통과
  const block = inputHandler.slice(start, start + 400);
  assert.doesNotMatch(
    block,
    /clearTableResizeRuntimeCache\(\)/,
    'open-document-bytes 는 열기 경로 여섯 중 하나만 덮는다 — 깔때기에서 듣는다',
  );
});

test('깔때기를 우회하는 열기 경로가 없다', () => {
  const main = source('src/main.ts');
  // loadBytes 를 부르는 자리가 여럿이라는 것이 이 이슈의 전제다. 그 수가 1 이 되면
  // 깔때기 논거 자체가 사라지므로, 전제가 살아 있는지 함께 확인한다.
  const callers = main.match(/(?<!async function )\bloadBytes\(/g) ?? [];
  assert.ok(
    callers.length >= 3,
    `loadBytes 호출부가 ${callers.length}곳 — 깔때기가 여러 경로를 모은다는 전제가 유지되어야 한다`,
  );
});
