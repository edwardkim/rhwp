import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import {
  buildHtmlExportFile,
  collectDocumentHtml,
  htmlExportBaseName,
  unwrapEngineHtmlFragment,
  type DocumentHtmlEngine,
} from '../src/command/export-html.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function engine(overrides: Partial<DocumentHtmlEngine> = {}): DocumentHtmlEngine {
  return {
    getSectionCount: () => 0,
    getParagraphCount: () => 0,
    getTextRange: () => '',
    exportSelectionHtml: () => '',
    ...overrides,
  };
}

test('unwrapEngineHtmlFragment는 클립보드 래퍼와 fragment 마커를 제거한다', () => {
  const inner = unwrapEngineHtmlFragment(
    '<html><body>\n<!--StartFragment-->\n<p>본문</p>\n<!--EndFragment-->\n</body></html>',
  );
  assert.equal(inner, '<p>본문</p>');
});

test('unwrapEngineHtmlFragment는 래퍼가 없으면 원문을 유지한다', () => {
  assert.equal(unwrapEngineHtmlFragment('<p>그대로</p>'), '<p>그대로</p>');
});

test('collectDocumentHtml은 섹션별 selection HTML을 문서 전체 범위로 이어붙인다', () => {
  const calls: number[][] = [];
  const html = collectDocumentHtml(engine({
    getSectionCount: () => 2,
    getParagraphCount: (section) => (section === 0 ? 2 : 1),
    getTextRange: () => 'abc',
    exportSelectionHtml: (section, sp, so, ep, eo) => {
      calls.push([section, sp, so, ep, eo]);
      return `<p>sec${section}</p>`;
    },
  }));

  assert.equal(html, '<p>sec0</p>\n<p>sec1</p>');
  // 각 섹션: 첫 문단 0 오프셋 ~ 마지막 문단 끝(텍스트 길이 3)
  assert.deepEqual(calls, [[0, 0, 0, 1, 3], [1, 0, 0, 0, 3]]);
});

test('collectDocumentHtml은 한 섹션의 실패를 건너뛰고 나머지를 유지한다', () => {
  const html = collectDocumentHtml(engine({
    getSectionCount: () => 2,
    getParagraphCount: () => 1,
    getTextRange: () => 'x',
    exportSelectionHtml: (section) => {
      if (section === 0) throw new Error('section 0 failed');
      return '<p>sec1</p>';
    },
  }));
  assert.equal(html, '<p>sec1</p>');
});

test('htmlExportBaseName은 문서 확장자를 제거하고 빈 이름에 기본값을 쓴다', () => {
  assert.equal(htmlExportBaseName('보고서.hwp'), '보고서');
  assert.equal(htmlExportBaseName('보고서.HWPX'), '보고서');
  assert.equal(htmlExportBaseName('   '), '문서');
  assert.equal(htmlExportBaseName(undefined), '문서');
});

test('buildHtmlExportFile(html)은 완전한 HTML 문서와 이스케이프된 제목을 만든다', () => {
  const file = buildHtmlExportFile(
    engine({
      getSectionCount: () => 1,
      getParagraphCount: () => 1,
      getTextRange: () => 'ab',
      exportSelectionHtml: () => '<html><body><!--StartFragment--><p>본문</p></body></html>',
    }),
    'html',
    '<제목>.hwp',
  );

  assert.equal(file.fileName, '<제목>.html');
  assert.equal(file.mimeType, 'text/html;charset=utf-8');
  assert.match(file.content, /^<!DOCTYPE html>/);
  assert.match(file.content, /<title>&lt;제목&gt;<\/title>/);
  assert.match(file.content, /<p>본문<\/p>/);
  // 엔진 래퍼가 벗겨져 html/body 태그는 최종 문서에 한 번씩만 남는다.
  assert.equal(file.content.match(/<html/gi)?.length, 1);
  assert.equal(file.content.match(/<body/gi)?.length, 1);
  assert.ok(!file.content.includes('StartFragment'));
});

test('buildHtmlExportFile(doc)은 Word 호환 mso 네임스페이스 래퍼를 쓴다', () => {
  const file = buildHtmlExportFile(engine(), 'doc', '문서.hwp');

  assert.equal(file.fileName, '문서.doc');
  assert.equal(file.mimeType, 'application/msword');
  assert.match(file.content, /xmlns:w="urn:schemas-microsoft-com:office:word"/);
});

test('파일 메뉴는 HTML/Word 내보내기 항목을 노출한다', () => {
  const html = readFileSync(join(rootDir, 'index.html'), 'utf8');

  assert.match(html, /data-cmd="file:export-html"/);
  assert.match(html, /data-cmd="file:export-doc"/);
});
