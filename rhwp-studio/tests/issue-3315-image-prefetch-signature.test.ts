import test from 'node:test';
import assert from 'node:assert/strict';

import { shouldSkipImagePrefetch } from '../src/view/image-prefetch-signature.ts';

const KEYS = '{"keys":["bin:0:1:src","bin:0:2:src"]}';

test('같은 그림 집합을 이미 디코드했으면 prefetch 를 건너뛴다', () => {
  assert.equal(
    shouldSkipImagePrefetch({ imageKeys: KEYS, hadRawSvg: false }, KEYS),
    true,
  );
});

test('그림이 바뀌면 건너뛰지 않는다', () => {
  const changed = '{"keys":["bin:1:1:src","bin:1:2:src"]}';
  assert.equal(
    shouldSkipImagePrefetch({ imageKeys: KEYS, hadRawSvg: false }, changed),
    false,
  );
  // 그림이 하나 사라진 경우도 서명이 달라져야 한다.
  assert.equal(
    shouldSkipImagePrefetch({ imageKeys: KEYS, hadRawSvg: false }, '{"keys":["bin:0:1:src"]}'),
    false,
  );
});

test('rawSvg 가 있던 페이지는 그림 키가 같아도 건너뛰지 않는다', () => {
  // 차트/OLE 내용 변화는 그림 신원 키가 덮지 못한다.
  assert.equal(
    shouldSkipImagePrefetch({ imageKeys: KEYS, hadRawSvg: true }, KEYS),
    false,
  );
});

test('판정 재료가 없으면 종전대로 매번 prefetch 한다', () => {
  // 키 조회를 지원하지 않는 구형 WASM
  assert.equal(
    shouldSkipImagePrefetch({ imageKeys: KEYS, hadRawSvg: false }, null),
    false,
  );
  // 아직 한 번도 디코드를 마치지 않은 페이지
  assert.equal(shouldSkipImagePrefetch(undefined, KEYS), false);
});
