// document-url-resolver 단위 테스트 (#432)
//
// 실행: node --test rhwp-shared/sw/document-url-resolver.test.js

import { test } from 'node:test';
import { strict as assert } from 'node:assert';

import {
  isDocumentPath,
  resolveDocumentUrl,
  resolveGithubBlobUrl,
} from './document-url-resolver.js';

// ─── 문서 경로 판정 ────────────────────────────────────

test('hwp/hwpx pathname 감지', () => {
  assert.equal(isDocumentPath('/saved/sample.hwp'), true);
  assert.equal(isDocumentPath('/saved/sample.hwpx'), true);
  assert.equal(isDocumentPath('/saved/SAMPLE.HWP'), true);
});

test('query 문자열에만 hwp가 있으면 미감지', () => {
  assert.equal(isDocumentPath('/download?file=sample.hwp'), false);
  assert.equal(isDocumentPath('/download.do'), false);
});

test('URL 인코딩된 한글 hwp pathname 감지', () => {
  assert.equal(
    isDocumentPath('/samples/2022%EB%85%84%20%EB%AC%B8%EC%84%9C.hwp'),
    true,
  );
});

// ─── GitHub provider ───────────────────────────────────

test('GitHub blob HWP URL을 raw URL로 변환', () => {
  const input = 'https://github.com/edwardkim/rhwp/blob/devel/saved/pr360-edward.hwp';
  const expected = 'https://raw.githubusercontent.com/edwardkim/rhwp/devel/saved/pr360-edward.hwp';

  assert.equal(resolveDocumentUrl(input), expected);
});

test('GitHub blob HWPX URL도 raw URL로 변환', () => {
  const input = 'https://github.com/edwardkim/rhwp/blob/devel/saved/blank_hwpx.hwpx';
  const expected = 'https://raw.githubusercontent.com/edwardkim/rhwp/devel/saved/blank_hwpx.hwpx';

  assert.equal(resolveDocumentUrl(input), expected);
});

test('GitHub blob의 인코딩된 경로를 보존하여 raw URL로 변환', () => {
  const input = 'https://github.com/edwardkim/rhwp/blob/devel/samples/2022%EB%85%84%20%EB%AC%B8%EC%84%9C.hwp';
  const expected = 'https://raw.githubusercontent.com/edwardkim/rhwp/devel/samples/2022%EB%85%84%20%EB%AC%B8%EC%84%9C.hwp';

  assert.equal(resolveDocumentUrl(input), expected);
});

test('GitHub raw.githubusercontent URL은 변환하지 않음', () => {
  const raw = 'https://raw.githubusercontent.com/edwardkim/rhwp/devel/saved/pr360-edward.hwp';

  assert.equal(resolveDocumentUrl(raw), raw);
});

test('GitHub blob이지만 문서 확장자가 아니면 변환하지 않음', () => {
  const url = 'https://github.com/edwardkim/rhwp/blob/devel/README.md';

  assert.equal(resolveDocumentUrl(url), url);
});

test('GitHub blob이지만 query에만 hwp가 있으면 변환하지 않음', () => {
  const url = 'https://github.com/edwardkim/rhwp/blob/devel/README.md?file=sample.hwp';

  assert.equal(resolveDocumentUrl(url), url);
});

test('GitHub가 아닌 일반 HWP URL은 변환하지 않음', () => {
  const url = 'https://example.com/files/sample.hwp';

  assert.equal(resolveDocumentUrl(url), url);
});

test('malformed URL은 원본 반환', () => {
  const url = 'not a url';

  assert.equal(resolveDocumentUrl(url), url);
});

test('resolveGithubBlobUrl은 URL 객체만 처리', () => {
  assert.equal(resolveGithubBlobUrl('https://github.com/edwardkim/rhwp/blob/devel/a.hwp'), null);
});
