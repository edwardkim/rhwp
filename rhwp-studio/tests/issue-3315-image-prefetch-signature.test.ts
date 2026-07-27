import test from 'node:test';
import assert from 'node:assert/strict';

import {
  cacheableImageKeySignature,
  collectImagePrefetchDataUrls,
  completeImagePrefetch,
  shouldSkipImagePrefetch,
} from '../src/view/image-prefetch-signature.ts';

const KEYS = '{"keys":["bin:0:1:src","bin:0:2:src"]}';
const DOC_A = 'blake3:aaaa';
const DOC_B = 'blake3:bbbb';
const GENERATION_A = 1;
const GENERATION_B = 2;

test('중첩 bbox가 mime 앞에 있어도 image prefetch URL을 수집한다', () => {
  const urls: string[] = [];
  collectImagePrefetchDataUrls({
    root: {
      kind: 'leaf',
      ops: [{
        type: 'image',
        bbox: { x: 12, y: 34, width: 56, height: 78 },
        wrap: 'flow',
        mime: 'image/png',
        base64: 'AA==',
      }],
    },
  }, urls);
  assert.deepEqual(urls, ['data:image/png;base64,AA==']);
});

test('같은 그림 집합을 이미 디코드했으면 prefetch 를 건너뛴다', () => {
  assert.equal(
    shouldSkipImagePrefetch({ documentDigest: DOC_A, documentGeneration: GENERATION_A, imageKeys: KEYS, hadRawSvg: false }, KEYS, DOC_A, GENERATION_A, 0),
    true,
  );
});

test('그림이 바뀌면 건너뛰지 않는다', () => {
  const changed = '{"keys":["bin:1:1:src","bin:1:2:src"]}';
  assert.equal(
    shouldSkipImagePrefetch({ documentDigest: DOC_A, documentGeneration: GENERATION_A, imageKeys: KEYS, hadRawSvg: false }, changed, DOC_A, GENERATION_A, 0),
    false,
  );
  // 그림이 하나 사라진 경우도 서명이 달라져야 한다.
  assert.equal(
    shouldSkipImagePrefetch({ documentDigest: DOC_A, documentGeneration: GENERATION_A, imageKeys: KEYS, hadRawSvg: false }, '{"keys":["bin:0:1:src"]}', DOC_A, GENERATION_A, 0),
    false,
  );
});

test('rawSvg 가 있던 페이지는 그림 키가 같아도 건너뛰지 않는다', () => {
  // 차트/OLE 내용 변화는 그림 신원 키가 덮지 못한다.
  assert.equal(
    shouldSkipImagePrefetch({ documentDigest: DOC_A, documentGeneration: GENERATION_A, imageKeys: KEYS, hadRawSvg: true }, KEYS, DOC_A, GENERATION_A, 0),
    false,
  );
});

test('판정 재료가 없으면 종전대로 매번 prefetch 한다', () => {
  // 키 조회를 지원하지 않는 구형 WASM
  assert.equal(
    shouldSkipImagePrefetch({ documentDigest: DOC_A, documentGeneration: GENERATION_A, imageKeys: KEYS, hadRawSvg: false }, null, DOC_A, GENERATION_A, 0),
    false,
  );
  // 아직 한 번도 디코드를 마치지 않은 페이지
  assert.equal(shouldSkipImagePrefetch(undefined, KEYS, DOC_A, GENERATION_A, 0), false);
});

test('다른 문서의 서명은 키가 같아도 재사용하지 않는다', () => {
  // bin_data_id 와 세대 번호가 문서마다 다시 시작하므로, 서로 다른 두 문서의 0쪽 첫
  // 그림은 키 문자열이 똑같이 나온다. 문서 신원이 없으면 여기서 오판한다.
  assert.equal(
    shouldSkipImagePrefetch({ documentDigest: DOC_A, documentGeneration: GENERATION_A, imageKeys: KEYS, hadRawSvg: false }, KEYS, DOC_B, GENERATION_A, 0),
    false,
  );
});

test('문서 신원을 모르면 재사용하지 않는다', () => {
  assert.equal(
    shouldSkipImagePrefetch({ documentDigest: DOC_A, documentGeneration: GENERATION_A, imageKeys: KEYS, hadRawSvg: false }, KEYS, null, GENERATION_A, 0),
    false,
  );
});

test('현재 rawSvg가 새로 생기면 이전 raster 서명을 재사용하지 않는다', () => {
  assert.equal(
    shouldSkipImagePrefetch({ documentDigest: DOC_A, documentGeneration: GENERATION_A, imageKeys: KEYS, hadRawSvg: false }, KEYS, DOC_A, GENERATION_A, 1),
    false,
  );
});

test('decode가 모두 끝난 뒤에만 서명을 기록한다', async () => {
  let resolveDecode!: (ok: boolean) => void;
  const pending = new Promise<boolean>((resolve) => { resolveDecode = resolve; });
  let recorded = false;
  const completion = completeImagePrefetch([pending], () => true, () => { recorded = true; });
  await Promise.resolve();
  assert.equal(recorded, false, 'decode 대기 중에는 완료 서명을 기록하면 안 됨');
  resolveDecode(true);
  assert.equal(await completion, true);
  assert.equal(recorded, true);
});

test('decode 실패나 빈 작업은 완료 서명으로 기록하지 않는다', async () => {
  let recorded = false;
  assert.equal(await completeImagePrefetch([Promise.resolve(false)], () => true, () => { recorded = true; }), false);
  assert.equal(await completeImagePrefetch([], () => true, () => { recorded = true; }), false);
  assert.equal(recorded, false);
});

test('같은 파일을 다시 열면 문서 세대가 달라 서명을 재사용하지 않는다', () => {
  assert.equal(
    shouldSkipImagePrefetch(
      { documentDigest: DOC_A, documentGeneration: GENERATION_A, imageKeys: KEYS, hadRawSvg: false },
      KEYS,
      DOC_A,
      GENERATION_B,
      0,
    ),
    false,
  );
});

test('늦게 끝난 이전 prefetch는 최신 서명을 덮어쓰지 않는다', async () => {
  let recorded = false;
  assert.equal(
    await completeImagePrefetch([Promise.resolve(true)], () => false, () => { recorded = true; }),
    false,
  );
  assert.equal(recorded, false);
});

test('안정된 키가 없는 합성 그림 페이지는 서명 캐시 대상이 아니다', () => {
  assert.equal(cacheableImageKeySignature('{"cacheable":false,"keys":[null]}'), null);
  assert.equal(
    cacheableImageKeySignature('{"cacheable":true,"keys":["bin:0:1:src"]}'),
    '{"cacheable":true,"keys":["bin:0:1:src"]}',
  );
  assert.equal(cacheableImageKeySignature('not-json'), null);
});
