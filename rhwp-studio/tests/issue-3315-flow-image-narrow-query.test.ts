import test from 'node:test';
import assert from 'node:assert/strict';

import {
  collectFlowImagePaintOps,
  flowImageOpsFromNarrowQuery,
} from '../src/view/flow-image-clip.ts';
import { FlowImageUrlCache } from '../src/view/flow-image-url-cache.ts';

// [#3315] 좁은 질의 경로와 전체 트리 경로는 같은 `FlowImagePaintOp` 를 내는 두 생산자다.
// 둘이 어긋나면 화면이 갈리므로, 여기서는 ①필드 옮김이 맞는지 ②쓸 수 없을 때 부분 결과를
// 내놓지 않고 전부 포기하는지를 고정한다. Rust 쪽 등가성은
// tests/issue_3315_flow_image_narrow_query.rs 가 실문서로 잡는다.

const narrowResponse = (overrides: Record<string, unknown> = {}) => JSON.stringify({
  cacheable: true,
  images: [
    {
      bbox: { x: 10, y: 20, width: 30, height: 40 },
      clip: { x: 12, y: 22, width: 20, height: 30 },
      mime: 'image/png',
      sourceImageKey: 'bin:0:1:src',
      crop: { left: 0, top: 0, right: 100, bottom: 100 },
      originalSizeHu: [4000, 3000],
      effect: 'grayScale',
      brightness: 10,
      contrast: -10,
      transform: { rotation: 90, horzFlip: true, vertFlip: false },
      ...overrides,
    },
  ],
});

const resolveOk = (key: string) => `blob:${key}`;

test('좁은 질의 응답을 FlowImagePaintOp 로 옮긴다', () => {
  const images = flowImageOpsFromNarrowQuery(narrowResponse(), resolveOk);

  assert.ok(images);
  assert.equal(images.length, 1);
  const image = images[0];
  assert.deepEqual(image.bbox, { x: 10, y: 20, width: 30, height: 40 });
  assert.deepEqual(image.clip, { x: 12, y: 22, width: 20, height: 30 });
  assert.equal(image.src, 'blob:bin:0:1:src');
  assert.deepEqual(image.crop, { left: 0, top: 0, right: 100, bottom: 100 });
  assert.deepEqual(image.originalSizeHu, [4000, 3000]);
  assert.equal(image.rotation, 90);
  assert.equal(image.horzFlip, true);
  assert.equal(image.vertFlip, false);
  // 효과는 값으로 받아 studio 가 조립한다 — canvas 경로와 같은 문자열이어야 한다.
  assert.equal(image.filter, 'grayscale(100%) brightness(1.1000) contrast(0.9000)');
});

test('전체 트리 경로는 data URL 을, 좁은 질의 경로는 키별 URL 을 같은 필드에 담는다', () => {
  const fromTree = collectFlowImagePaintOps(
    {
      kind: 'leaf',
      ops: [
        { type: 'image', bbox: { x: 1, y: 2, width: 3, height: 4 }, mime: 'image/png', base64: 'AA==' },
      ],
    },
    (op) => op.type === 'image',
  );
  assert.equal(fromTree[0].src, 'data:image/png;base64,AA==');

  const fromNarrow = flowImageOpsFromNarrowQuery(narrowResponse(), resolveOk);
  assert.ok(fromNarrow);
  assert.match(fromNarrow[0].src, /^blob:/);
});

test('cacheable 이 아니면 이 경로를 쓰지 않는다', () => {
  // 신원 키를 낼 수 없는 합성 그림이 섞인 페이지 — 바이트를 되찾을 방법이 없다.
  const json = JSON.stringify({ cacheable: false, images: [] });
  assert.equal(flowImageOpsFromNarrowQuery(json, resolveOk), null);
});

test('키 하나라도 못 풀면 전부 포기한다', () => {
  // 부분 목록을 돌려주면 그림 몇 장이 조용히 사라진다 — 그건 느린 것보다 나쁘다.
  const json = JSON.stringify({
    cacheable: true,
    images: [
      { bbox: { x: 0, y: 0, width: 1, height: 1 }, mime: 'image/png', sourceImageKey: 'bin:0:1:src' },
      { bbox: { x: 5, y: 5, width: 1, height: 1 }, mime: 'image/png', sourceImageKey: 'bin:0:2:src' },
    ],
  });
  const resolveSecondFails = (key: string) => (key === 'bin:0:2:src' ? null : `blob:${key}`);
  assert.equal(flowImageOpsFromNarrowQuery(json, resolveSecondFails), null);
});

test('형식이 어긋난 응답은 되돌림을 유발한다', () => {
  for (const json of [
    'not json',
    'null',
    '[]',
    JSON.stringify({ cacheable: true }),
    JSON.stringify({ cacheable: true, images: [null] }),
    // bbox 없음
    JSON.stringify({ cacheable: true, images: [{ mime: 'image/png', sourceImageKey: 'bin:0:1:src' }] }),
    // 키 없음 — 바이트를 받을 수 없다
    JSON.stringify({ cacheable: true, images: [{ bbox: { x: 0, y: 0, width: 1, height: 1 }, mime: 'image/png' }] }),
  ]) {
    assert.equal(flowImageOpsFromNarrowQuery(json, resolveOk), null, `되돌려야 한다: ${json}`);
  }
});

test('빈 목록은 유효한 결과다 — 그림 없는 쪽에서 전체 트리를 받지 않는다', () => {
  const images = flowImageOpsFromNarrowQuery(
    JSON.stringify({ cacheable: true, images: [] }),
    resolveOk,
  );
  assert.deepEqual(images, []);
});

// ── object URL 캐시 ──

test('object URL 캐시는 키당 한 번만 바이트를 받는다', () => {
  const revoked: string[] = [];
  const created: string[] = [];
  const originalCreate = globalThis.URL.createObjectURL;
  const originalRevoke = globalThis.URL.revokeObjectURL;
  let counter = 0;
  globalThis.URL.createObjectURL = () => {
    const url = `blob:stub/${++counter}`;
    created.push(url);
    return url;
  };
  globalThis.URL.revokeObjectURL = (url: string) => {
    revoked.push(url);
  };

  try {
    const cache = new FlowImageUrlCache();
    let loads = 0;
    const loadBytes = () => {
      loads += 1;
      return new Uint8Array([1, 2, 3]);
    };

    const first = cache.urlFor('bin:0:1:src', 'image/png', loadBytes);
    const second = cache.urlFor('bin:0:1:src', 'image/png', loadBytes);
    assert.equal(first, second, '같은 키는 같은 URL');
    assert.equal(loads, 1, '바이트는 한 번만 받는다 — 편집마다 다시 받으면 캐시가 무의미하다');
    assert.equal(cache.size, 1);
    assert.equal(cache.has('bin:0:1:src'), true);

    cache.urlFor('bin:0:2:src', 'image/png', loadBytes);
    assert.equal(cache.size, 2);

    cache.releaseAll();
    assert.equal(cache.size, 0);
    assert.deepEqual(revoked, created, '회수하지 않으면 문서를 갈아끼울 때마다 URL 이 쌓인다');
  } finally {
    globalThis.URL.createObjectURL = originalCreate;
    globalThis.URL.revokeObjectURL = originalRevoke;
  }
});

test('바이트를 받을 수 없으면 URL 을 만들지 않는다', () => {
  const cache = new FlowImageUrlCache();
  assert.equal(cache.urlFor('bin:9:1:src', 'image/png', () => null), null);
  // 빈 바이트도 그림이 될 수 없다 — 캐시에 남겨 두면 다음 조회가 빈 URL 을 재사용한다.
  assert.equal(cache.urlFor('bin:9:2:src', 'image/png', () => new Uint8Array()), null);
  assert.equal(cache.size, 0);
});
