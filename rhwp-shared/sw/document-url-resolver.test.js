// document-url-resolver 단위 테스트 (#432)
//
// 실행: node --test rhwp-shared/sw/document-url-resolver.test.js

import { test } from 'node:test';
import { strict as assert } from 'node:assert';

import {
  classifyDocumentUrl,
  classifyGithubDocumentUrl,
  isDocumentPath,
  resolveDocumentUrl,
  resolveGithubBlobUrl,
} from './document-url-resolver.js';

// ─── 문서 경로 판정 ────────────────────────────────────

test('hwp/hwpx/hml pathname 감지', () => {
  assert.equal(isDocumentPath('/saved/sample.hwp'), true);
  assert.equal(isDocumentPath('/saved/sample.hwpx'), true);
  assert.equal(isDocumentPath('/saved/sample.hml'), true);
  assert.equal(isDocumentPath('/saved/SAMPLE.HWP'), true);
  assert.equal(isDocumentPath('/saved/SAMPLE.HML'), true);
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

test('GitHub blob HML URL도 raw URL로 변환', () => {
  const input = 'https://github.com/edwardkim/rhwp/blob/devel/samples/hml/formatting_table.hml';
  const expected = 'https://raw.githubusercontent.com/edwardkim/rhwp/devel/samples/hml/formatting_table.hml';

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

// ─── 문서 후보 분류 ───────────────────────────────────

test('classifyDocumentUrl은 GitHub blob HWP URL을 openable로 분류하고 raw URL을 제공', () => {
  const input = 'https://github.com/edwardkim/rhwp/blob/main/samples/2010-01-06.hwp';
  const result = classifyDocumentUrl(input);

  assert.equal(result.status, 'openable');
  assert.equal(result.reason, 'github-blob-document');
  assert.equal(
    result.resolvedUrl,
    'https://raw.githubusercontent.com/edwardkim/rhwp/main/samples/2010-01-06.hwp',
  );
});

test('classifyDocumentUrl은 GitHub blob HWPX URL도 openable로 분류', () => {
  const result = classifyDocumentUrl('https://github.com/edwardkim/rhwp/blob/main/samples/hwpx/sample.hwpx');

  assert.equal(result.status, 'openable');
  assert.equal(result.reason, 'github-blob-document');
  assert.equal(
    result.resolvedUrl,
    'https://raw.githubusercontent.com/edwardkim/rhwp/main/samples/hwpx/sample.hwpx',
  );
});

test('classifyDocumentUrl은 GitHub blob HML URL도 openable로 분류', () => {
  const result = classifyDocumentUrl(
    'https://github.com/edwardkim/rhwp/blob/main/samples/hml/formatting_table.hml',
  );

  assert.equal(result.status, 'openable');
  assert.equal(result.reason, 'github-blob-document');
  assert.equal(
    result.resolvedUrl,
    'https://raw.githubusercontent.com/edwardkim/rhwp/main/samples/hml/formatting_table.hml',
  );
});

test('classifyDocumentUrl은 raw.githubusercontent HWP URL을 openable로 분류', () => {
  const raw = 'https://raw.githubusercontent.com/edwardkim/rhwp/main/samples/2010-01-06.hwp';
  const result = classifyDocumentUrl(raw);

  assert.equal(result.status, 'openable');
  assert.equal(result.reason, 'github-raw-document');
  assert.equal(result.resolvedUrl, raw);
});

test('classifyDocumentUrl은 GitHub edit HWP URL을 not-document로 분류', () => {
  const result = classifyDocumentUrl('https://github.com/edwardkim/rhwp/edit/main/samples/2010-01-06.hwp');

  assert.equal(result.status, 'not-document');
  assert.equal(result.reason, 'github-edit-page');
});

test('classifyDocumentUrl은 GitHub commits HWP URL을 not-document로 분류', () => {
  const result = classifyDocumentUrl('https://github.com/edwardkim/rhwp/commits/main/samples/2010-01-06.hwp');

  assert.equal(result.status, 'not-document');
  assert.equal(result.reason, 'github-commits-page');
});

test('classifyDocumentUrl은 GitHub blame HWP URL을 not-document로 분류', () => {
  const result = classifyDocumentUrl('https://github.com/edwardkim/rhwp/blame/main/samples/2010-01-06.hwp');

  assert.equal(result.status, 'not-document');
  assert.equal(result.reason, 'github-blame-page');
});

test('classifyDocumentUrl은 GitHub tree URL을 not-document로 분류', () => {
  const result = classifyDocumentUrl('https://github.com/edwardkim/rhwp/tree/main/samples');

  assert.equal(result.status, 'not-document');
  assert.equal(result.reason, 'github-tree-page');
});

test('classifyDocumentUrl은 GitHub blob의 query 위장 HWP를 not-document로 분류', () => {
  const result = classifyDocumentUrl('https://github.com/edwardkim/rhwp/blob/main/README.md?file=sample.hwp');

  assert.equal(result.status, 'not-document');
  assert.equal(result.reason, 'github-blob-non-document');
});

test('classifyDocumentUrl은 일반 직접 HWP URL을 openable로 분류', () => {
  const url = 'https://example.com/files/sample.hwp';
  const result = classifyDocumentUrl(url);

  assert.equal(result.status, 'openable');
  assert.equal(result.reason, 'document-path');
  assert.equal(result.resolvedUrl, url);
});

test('classifyDocumentUrl은 query에만 HWP가 있는 일반 URL을 unknown으로 분류', () => {
  const result = classifyDocumentUrl('https://example.com/download?file=sample.hwp');

  assert.equal(result.status, 'unknown');
  assert.equal(result.reason, 'no-document-path');
});

test('classifyGithubDocumentUrl은 URL 객체만 처리', () => {
  assert.equal(
    classifyGithubDocumentUrl('https://github.com/edwardkim/rhwp/blob/main/a.hwp'),
    null,
  );
});
