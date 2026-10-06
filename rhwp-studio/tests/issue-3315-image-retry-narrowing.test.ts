import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [#3315] `resetImageRetryState()` 의 blanket 리셋 제거.
//
// 종전 재시도 키는 `imageCount:rawSvgCount:overlaySignature` 였고, 그 재료는 그림 **내용**
// 변화를 보지 못한다 — 밝기/대비를 켜면 워터마크 bake 로 바이트가 바뀌는데 개수는 그대로다.
// 그 위험을 막으려고 `refreshPages` 가 매 편집에 재시도 상태를 전부 비웠는데, 그러면 페이지마다
// 재렌더가 한 번 더 돈다(prefetch 가 서명으로 건너뛰어 `finish()` 즉시 호출 → 다시 그림).
//
// 비우는 대신 키가 봐야 할 것을 직접 들게 했다. 여기서 고정하는 것은 그 구조다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

const methodBody = (src: string, signature: string): string => {
  const start = src.indexOf(signature);
  assert.notEqual(start, -1, `${signature} 를 찾지 못했다`);
  const rest = src.slice(start);
  return rest.slice(0, rest.indexOf('\n  }'));
};

test('[#3315] resetImageRetryState 는 재시도 키를 비우지 않는다', () => {
  const renderer = source('src/view/page-renderer.ts');
  const body = methodBody(renderer, 'resetImageRetryState(): void {');

  assert.doesNotMatch(
    body,
    /imageRetryCounts/,
    '이 메서드는 편집마다 불린다(refreshPages → releaseAllRenderedPages). 여기서 재시도 키를 '
      + '비우면 페이지마다 재렌더가 한 번 더 돈다',
  );
  // 파생 상태 정리는 그대로 남는다 — 캔버스를 버렸으므로 이들은 무효다.
  for (const field of ['prefetchRequestTokens', 'layerSummaryCache', 'canvaskitDiagnosticsByPage']) {
    assert.match(body, new RegExp(`${field}\\.clear\\(\\)`), `${field} 정리는 유지해야 한다`);
  }
});

test('[#3315] 재시도 키는 문서 신원과 그림 내용을 직접 든다', () => {
  const renderer = source('src/view/page-renderer.ts');
  const body = methodBody(renderer, 'private buildImageRetryKey(');

  // 그림 내용 — variant(`src`↔`wmpng`)가 갈리면 키가 달라진다.
  assert.match(body, /getPageSourceImageKeys\(pageIdx\)/);
  assert.match(body, /cacheableImageKeySignature/, '합성 그림이 섞인 페이지는 판정 대상이 아니다');

  // 문서 경계 — 세대는 문서마다 0 에서 시작해 `bin:0:1:src` 가 충돌한다.
  assert.match(body, /documentDigest/);
  assert.match(body, /documentGeneration/);

  // RawSvg는 source-image key에 없고 decoder cache readiness도 별도이므로, 같은 개수로
  // 재시도 안전망을 생략하면 안 된다.
  assert.match(
    body,
    /if \(rawSvgCount > 0\) return null;/,
    'RawSvg가 있으면 timer/fallback 재렌더를 재사용으로 건너뛰면 안 된다',
  );

  // 판정 재료가 없으면 재사용을 포기한다(안전망 쪽으로 기운다).
  assert.match(body, /return null;/);
});

test('[#3315] 판정 재료가 없으면 재사용 조기 반환이 일어나지 않는다', () => {
  const renderer = source('src/view/page-renderer.ts');

  // `null` 키로 `Map.get(pageIdx) === null` 비교가 우연히 맞는 일이 없도록 명시적으로 가른다.
  assert.match(
    renderer,
    /if \(retryKey !== null && this\.imageRetryCounts\.get\(pageIdx\) === retryKey\s*&& !this\.reRenderJobs\.has\(pageIdx\)\) return;/,
    'null 키에서는 조기 반환하지 않아야 한다 — 판정할 수 없으면 안전망을 돌려야 한다',
  );
  // 그리고 null 키는 기록하지 않는다 — 기록하면 다음 렌더가 그 값과 비교해 버린다.
  assert.match(
    renderer,
    /if \(retryKey === null\) this\.imageRetryCounts\.delete\(pageIdx\);/,
    'null 키는 저장하지 않고 항목을 지워야 한다',
  );
});

test('[#3315] dispose 는 재시도 키를 거둔다', () => {
  const renderer = source('src/view/page-renderer.ts');
  const body = methodBody(renderer, '  dispose(): void {');
  assert.match(
    body,
    /this\.imageRetryCounts\.clear\(\)/,
    'renderer 를 버리면 다시 조회될 일이 없으므로 여기서 정리한다',
  );
});

test('[#3315] 문서 교체 경로는 여전히 파생 상태를 정리한다', () => {
  const view = source('src/view/canvas-view.ts');
  const body = methodBody(view, 'private reset(): void {');
  assert.match(
    body,
    /this\.releaseAllRenderedPages\(\)/,
    '문서 교체는 렌더된 페이지를 버려야 한다 — 재시도 키의 문서 신원이 그 경계를 가른다',
  );
});
