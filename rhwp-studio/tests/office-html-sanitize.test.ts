import test from 'node:test';
import assert from 'node:assert/strict';
import { sanitizeOfficeHtmlForCore } from '../src/engine/office-html-sanitize.ts';

// 코어 HTML 파서는 `<!--[` 로 시작하는 주석과 `<body>` 안 `<style>` 을 못 걸러,
// 뒤 본문이 통째로 한 문단이 되거나 CSS 가 글자로 새어 나온다. 여기서 미리 걷어낸다.

test('한글 CF_HTML 의 문서모델 주석과 조각 마커를 걷어내고 본문은 남긴다', () => {
  const html =
    '<!--StartFragment--><p>본문</p><!--EndFragment-->' +
    '<!--[data-hwpjson]{"ro":{"hp":0}}-->';
  assert.equal(sanitizeOfficeHtmlForCore(html), '<p>본문</p>');
});

test('style·script·xml 은 통째로 사라지고 표는 남는다', () => {
  const html =
    '<xml><o:DocumentProperties/></xml>' +
    '<style>p { mso-style-name: 표준; }</style>' +
    '<table><tr><td>칸</td></tr></table>' +
    '<script>alert(1)</script>';
  assert.equal(sanitizeOfficeHtmlForCore(html), '<table><tr><td>칸</td></tr></table>');
});

test('downlevel-revealed 조건부 블록을 걷어낸다', () => {
  const html = '<![if !supportEmptyParas]>&nbsp;<![endif]><p>본문</p>';
  assert.equal(sanitizeOfficeHtmlForCore(html), '<p>본문</p>');
});

// --- 아래 3건은 정규식 치환이 놓치던 경우다(치환 → 한 번 훑기로 바꾼 이유). ---

test('주석이 겹쳐 있어도 여는 표지가 남지 않는다', () => {
  // `.replace(/<!--[\s\S]*?-->/g, '')` 는 안쪽만 지워 바깥 `<!--` 를 남긴다.
  const out = sanitizeOfficeHtmlForCore('<!--<!-- -->--><p>본문</p>');
  assert.ok(!out.includes('<!--'), `주석 여는 표지가 남았다: ${out}`);
});

test('닫는 태그 이름 뒤에 공백이 있어도 요소로 인식한다', () => {
  // `</script>` 만 찾는 정규식은 `</script >` 를 놓쳐 본문에 스크립트가 남는다.
  const out = sanitizeOfficeHtmlForCore('<script>alert(1)</script >본문');
  assert.equal(out, '본문');
});

test('닫는 태그가 아예 없으면 그 뒤를 전부 버린다', () => {
  assert.equal(sanitizeOfficeHtmlForCore('<p>앞</p><script>alert(1)'), '<p>앞</p>');
});

test('이름이 겹치는 다른 태그는 건드리지 않는다', () => {
  const html = '<styles-note>남는다</styles-note><scriptorium>이것도</scriptorium>';
  assert.equal(sanitizeOfficeHtmlForCore(html), html);
});

test('스스로 닫은 태그는 그 태그만 사라진다', () => {
  assert.equal(sanitizeOfficeHtmlForCore('<xml/><p>본문</p>'), '<p>본문</p>');
});

test('대소문자가 섞여도 같은 결과를 낸다', () => {
  assert.equal(sanitizeOfficeHtmlForCore('<STYLE>a{}</Style><p>본문</p>'), '<p>본문</p>');
});
