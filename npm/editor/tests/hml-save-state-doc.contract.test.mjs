import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

/** 체크아웃에 따라 CRLF 일 수 있으므로 개행을 LF 로 정규화한 뒤 비교한다. */
function readText(relativePath) {
  return readFileSync(fileURLToPath(new URL(relativePath, import.meta.url)), 'utf8')
    .replace(/\r\n/g, '\n');
}

const declarations = readText('../index.d.ts');
const readme = readText('../README.md');

/** index.d.ts 의 interface 선언에서 필드명 목록을 뽑는다. */
function interfaceFields(name) {
  const match = declarations.match(new RegExp(`export interface ${name} \\{([\\s\\S]*?)\\n\\}`));
  assert.ok(match, `${name} 선언이 index.d.ts 에 존재해야 한다`);
  return match[1]
    .split('\n')
    .map((line) => line.trim().match(/^([A-Za-z][A-Za-z0-9]*)\??:/))
    .filter((matched) => matched !== null)
    .map((matched) => matched[1]);
}

/** README 의 `### <heading>` 절 본문을 다음 heading 직전까지 잘라 반환한다. */
function readmeSection(heading) {
  const start = readme.indexOf(`\n### ${heading}\n`);
  assert.ok(start >= 0, `README 절 "${heading}" 이 존재해야 한다`);
  const rest = readme.slice(start + 1);
  const next = rest.search(/\n#+ /);
  return next < 0 ? rest : rest.slice(0, next);
}

test('README getHmlSaveState 절은 HmlSaveState 선언과 같은 필드를 문서화한다', () => {
  const section = readmeSection('editor.getHmlSaveState()');
  const fields = [
    ...interfaceFields('HmlSaveState'),
    ...interfaceFields('HmlSaveBlocker'),
  ];
  assert.deepEqual(fields, [
    'sourceFormat', 'hmlSavable', 'blockers',
    'code', 'xmlPath', 'message', 'preserved',
  ]);

  for (const field of fields) {
    assert.match(
      section,
      new RegExp(`\\b${field}\\s*:`),
      `README 가 반환값 필드 ${field} 를 객체 키 형태로 보여야 한다`,
    );
  }
});

test('README getHmlSaveState 절은 실재하지 않는 반환 필드를 문서화하지 않는다', () => {
  const section = readmeSection('editor.getHmlSaveState()');
  // `ok` / `blocker` 는 wire DTO 에 없는 키다. 객체 키 형태(`ok:`, `blocker:`)만 금지하므로
  // `blockers:` 나 `blocker.code` 같은 정상 표기는 걸리지 않는다.
  assert.doesNotMatch(section, /\bok\s*:/, 'getHmlSaveState 는 ok 필드를 반환하지 않는다');
  assert.doesNotMatch(section, /\bblocker\s*:/, 'getHmlSaveState 는 blocker 필드를 반환하지 않는다');
});
