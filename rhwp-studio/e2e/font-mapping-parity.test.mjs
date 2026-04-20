import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const studioRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(studioRoot, '..');

function read(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function extractAssignedLiteral(source, startMarker, endMarker) {
  const start = source.indexOf(startMarker);
  if (start === -1) {
    throw new Error(`start marker not found: ${startMarker}`);
  }
  const end = source.indexOf(endMarker, start + startMarker.length);
  if (end === -1) {
    throw new Error(`end marker not found: ${endMarker}`);
  }
  return source.slice(start + startMarker.length, end).trim();
}

function evalLiteral(literal, filename) {
  return vm.runInNewContext(`(${literal})`, {}, { filename });
}

function toPlain(value) {
  return JSON.parse(JSON.stringify(value));
}

function normalizeFontEntries(entries) {
  return Array.from(entries, (entry) => ({
    name: entry.name,
    file: entry.file,
    format: entry.format ?? null,
    weight: entry.weight ?? null,
  }));
}

function loadStudioFontList() {
  const source = read('rhwp-studio/src/core/font-loader.ts');
  const constants = new Map(
    [...source.matchAll(/^const (\w+) = '([^']+)';$/gm)].map((match) => [match[1], match[2]]),
  );
  let literal = extractAssignedLiteral(
    source,
    'const FONT_LIST: FontEntry[] = ',
    ';\n\n/** @font-face에 등록된 폰트 이름 Set */',
  );
  for (const [name, value] of constants) {
    literal = literal.replace(new RegExp(`\\b${name}\\b`, 'g'), JSON.stringify(value));
  }
  return normalizeFontEntries(evalLiteral(literal, 'font-loader.ts'));
}

function loadEditorFontList() {
  const source = read('../rhwp/web/editor.html');
  const literal = extractAssignedLiteral(
    source,
    '        const fonts = ',
    ';\n        // CSS @font-face 규칙 생성 + FontFace API 로드',
  );
  return normalizeFontEntries(evalLiteral(literal, 'editor.html'));
}

function loadStudioSubstTables() {
  const source = read('rhwp-studio/src/core/font-substitution.ts');
  const literal = extractAssignedLiteral(
    source,
    'const SUBST_TABLES: SubstEntry[][] = ',
    ';\n\n// 언어별 치환 해시맵',
  );
  return toPlain(evalLiteral(literal, 'font-substitution.ts'));
}

function loadLegacySubstTables() {
  const source = read('../rhwp/web/font_substitution.js');
  const literal = extractAssignedLiteral(
    source,
    '    const SUBST_TABLES = ',
    ';\n\n    const substMaps = ',
  );
  return toPlain(evalLiteral(literal, 'font_substitution.js'));
}

function loadLegacyFontSubstitutionApi() {
  const source = read('../rhwp/web/font_substitution.js');
  const sandbox = { globalThis: {}, console };
  vm.runInNewContext(source, sandbox, { filename: 'font_substitution.js' });
  return sandbox.globalThis.FontSubstitution;
}

function canonicalFallback(fontName) {
  if (fontName === 'serif' || fontName === 'sans-serif' || fontName === 'monospace') {
    return fontName;
  }
  if (/굴림체|바탕체|gulimche|batangche|coding|courier/i.test(fontName)) {
    return `"${fontName}", "GulimChe", "D2Coding", "NanumGothicCoding", "나눔고딕코딩", "Noto Sans Mono", monospace`;
  }
  if (/[바탕명조궁서]|hymjre|times|palatino|georgia|batang|gungsuh/i.test(fontName)) {
    return `"${fontName}", "Batang", "AppleMyungjo", "Noto Serif KR", "Noto Serif CJK KR", "NanumMyeongjo", "나눔명조", serif`;
  }
  return `"${fontName}", "Malgun Gothic", "Apple SD Gothic Neo", "Noto Sans KR", "Noto Sans CJK KR", "NanumGothic", "나눔고딕", "Pretendard", sans-serif`;
}

const studioFontList = loadStudioFontList();
const editorFontList = loadEditorFontList();
assert.deepStrictEqual(editorFontList, studioFontList, 'legacy editor font list must match studio font-loader entries');

const studioTables = loadStudioSubstTables();
const legacyTables = loadLegacySubstTables();
assert.deepStrictEqual(legacyTables, studioTables, 'legacy font substitution tables must match studio tables');

const legacyApi = loadLegacyFontSubstitutionApi();
const registeredFonts = [...legacyApi.REGISTERED_FONTS].sort();
const studioRegisteredFonts = [...new Set(studioFontList.map((entry) => entry.name))].sort();
assert.deepStrictEqual(registeredFonts, studioRegisteredFonts, 'legacy registered font set must match studio font-loader names');

for (const fontName of ['GulimChe', '궁서', '맑은 고딕', 'serif', 'monospace']) {
  assert.equal(
    legacyApi.fontFamilyWithFallback(fontName),
    canonicalFallback(fontName),
    `fontFamilyWithFallback mismatch for ${fontName}`,
  );
}

for (const { fontName, altType, langId, expected } of [
  { fontName: '가는안상수체', altType: 1, langId: 0, expected: '함초롬돋움' },
  { fontName: '태 가는 헤드라인T', altType: 2, langId: 0, expected: 'HY헤드라인M' },
  { fontName: 'HCI Poppy', altType: 2, langId: 1, expected: 'Palatino Linotype' },
  { fontName: 'Gulimche', altType: 1, langId: 6, expected: '굴림체' },
  { fontName: '없는폰트', altType: 0, langId: 0, expected: '없는폰트' },
]) {
  assert.equal(
    legacyApi.resolveFont(fontName, altType, langId),
    expected,
    `resolveFont mismatch for ${fontName}/${altType}/${langId}`,
  );
}

console.log('font mapping parity ok');
