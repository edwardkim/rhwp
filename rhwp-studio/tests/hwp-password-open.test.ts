import test from 'node:test';
import { codeOnly } from './support/source-guard.ts';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const mainSource = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
const bridgeSource = readFileSync(new URL('../src/core/wasm-bridge.ts', import.meta.url), 'utf8');
const typesSource = readFileSync(new URL('../src/core/types.ts', import.meta.url), 'utf8');
const fontSubstitutionSource = readFileSync(
  new URL('../src/core/font-substitution.ts', import.meta.url),
  'utf8',
);
const dialogSource = readFileSync(new URL('../src/ui/hwp-password-dialog.ts', import.meta.url), 'utf8');
const runtimeSnapshotSource = readFileSync(
  new URL('../../scripts/font_rule_runtime_snapshot.mjs', import.meta.url),
  'utf8',
);

function between(source: string, start: string, end: string): string {
  const startIndex = source.indexOf(start);
  assert.notEqual(startIndex, -1, `시작 표식이 있어야 합니다: ${start}`);
  const endIndex = source.indexOf(end, startIndex + start.length);
  assert.notEqual(endIndex, -1, `끝 표식이 있어야 합니다: ${end}`);
  return source.slice(startIndex, endIndex);
}

test('암호 문서는 명시적인 암호 필요 오류에서만 입력 UI로 전환한다', () => {
  const openPath = between(mainSource, 'async function loadDocumentForOpen', 'function showLoadErrorUnlessCancelled');
  assert.match(openPath, /wasm\.loadDocument\(data, fileName\)/, '일반 문서 열기를 유지한다');
  assert.match(openPath, /if \(!isPasswordRequiredError\(error\)\) throw error;/, '다른 파싱/DRM/지원 불가 오류는 숨기지 않는다');
  assert.match(openPath, /await openPasswordProtectedDocument\(data, fileName\);/, '암호 필요일 때만 open command로 전환한다');
  assert.match(openPath, /return wasm\.getDocumentInfo\(\);/, '암호 open 성공 뒤 별도 metadata query를 사용한다');
});

test('드롭 문서도 파일 메뉴와 같은 암호 열기 경로를 쓰며 File System Access handle을 capture하지 않는다', () => {
  const dropPath = between(mainSource, "container.addEventListener('drop', async (e) => {", 'function setupZoomControls');
  const imageBranch = dropPath.slice(dropPath.indexOf('if (isImage) {'), dropPath.indexOf('// HWP/HWPX/HML'));
  assert.match(imageBranch, /showDropConfirmDialog\(file\.name\)/, '이미지 드롭 삽입 확인은 유지해야 합니다');
  const docBranch = dropPath.slice(dropPath.indexOf('// HWP/HWPX/HML'));
  assert.doesNotMatch(docBranch, /showDropConfirmDialog/, '문서 드롭 열기는 확인 대화상자를 띄우지 않습니다');
  assert.match(dropPath, /await loadFile\(file\);/, '드롭 문서는 파일 메뉴와 같은 loadFile 경로를 사용해야 합니다');
  assert.doesNotMatch(dropPath, /captureDroppedFileHandle|getAsFileSystemHandle|fileHandle:/,
    '암호 문서 드롭에서 Chromium File System Access IPC를 시작하면 안 됩니다');
  assert.match(mainSource, /async function loadDocumentForOpen[\s\S]*openPasswordProtectedDocument/,
    'loadFile 이후 암호 감지와 password dialog 경로를 유지해야 합니다');
});

test('암호 입력은 단일 시도에만 쓰고, 취소와 오입력은 영속 경로에 도달하지 않는다', () => {
  const passwordPath = between(mainSource, 'async function openPasswordProtectedDocument', 'async function loadDocumentForOpen');
  assert.match(passwordPath, /showHwpPasswordDialog\(fileName, retryMessage\)/, '문서 이름만 대화상자에 전달한다');
  assert.match(passwordPath, /if \(password === null\) throw new DocumentOpenCancelledError\(\);/, '취소를 별도 상태로 전달한다');
  assert.match(passwordPath, /wasm\.loadDocumentWithPassword\(data, password, fileName\)/, 'WASM 암호 열기 API를 사용한다');
  assert.doesNotMatch(passwordPath, /return\s+wasm\.loadDocumentWithPassword/, '암호 open 반환값을 metadata로 전달하지 않는다');
  assert.match(passwordPath, /password = '';/, '시도 뒤 지역 암호 참조를 비운다');
  assert.doesNotMatch(passwordPath, /localStorage|sessionStorage|addRecentDoc|autosave|documentDigest|console\./, '암호값을 영속/로그 경로로 보내지 않는다');
  assert.match(passwordPath, /암호가 일치하지 않거나 문서가 손상되었습니다\. 다시 입력하세요\./, '오입력/암호문 손상은 재입력 상태로 설명한다');
});

test('#6731 암호 open command와 DocumentInfo query는 반환값 경계로 분리된다', () => {
  assert.equal(
    /async function openPasswordProtectedDocument\(data: Uint8Array, fileName: string\): Promise<void>/.test(
      mainSource,
    ),
    true,
    '암호 open helper는 metadata를 반환하지 않는 command여야 합니다',
  );
  const commandPath = between(
    mainSource,
    'async function openPasswordProtectedDocument',
    'async function loadDocumentForOpen',
  );
  assert.match(
    commandPath,
    /wasm\.loadDocumentWithPassword\(data, password, fileName\);/,
    '암호는 open command의 입력으로만 전달해야 합니다',
  );
  assert.doesNotMatch(
    commandPath,
    /return\s+wasm\.loadDocumentWithPassword/,
    'password 이름을 가진 호출의 반환값을 metadata 운반자로 사용하면 안 됩니다',
  );

  const openPath = between(mainSource, 'async function loadDocumentForOpen', 'function showLoadErrorUnlessCancelled');
  assert.match(
    openPath,
    /await openPasswordProtectedDocument\(data, fileName\);\s*return wasm\.getDocumentInfo\(\);/,
    '암호 open 성공 뒤 별도 query로 DocumentInfo를 읽어야 합니다',
  );

  const bridgeCommand = between(
    bridgeSource,
    'loadDocumentWithPassword',
    'private async populateExternalImagesFromDevServer',
  );
  assert.match(
    bridgeCommand,
    /loadDocumentWithPassword\(data: Uint8Array, password: string, fileName\?: string\): void/,
    'WasmBridge의 암호 open API도 void command여야 합니다',
  );
  assert.doesNotMatch(
    bridgeCommand,
    /return\s+this\.loadDocumentAtomically/,
    'WasmBridge command가 DocumentInfo를 반환하면 안 됩니다',
  );
});

test('#6731 DocumentInfo와 폰트 snapshot digest에는 password 운반 필드가 없다', () => {
  const documentInfo = between(typesSource, 'export interface DocumentInfo', 'export interface PageInfo');
  const fields = [...documentInfo.matchAll(/^\s{2}([A-Za-z][A-Za-z0-9]*)\??:/gm)]
    .map(match => match[1]);
  assert.deepEqual(fields, [
    'version',
    'sectionCount',
    'pageCount',
    'encrypted',
    'hwp3Variant',
    'fallbackFont',
    'fontsUsed',
    'fontSubstitutions',
  ]);
  assert.doesNotMatch(
    codeOnly(documentInfo),
    /password|credential|secret/i,
    'DocumentInfo에 raw credential 필드를 추가하면 안 됩니다',
  );

  const resolver = between(
    fontSubstitutionSource,
    'export function resolveFontWithRules',
    'export function fontFamilyWithFallback',
  );
  assert.match(
    resolver,
    /const cacheKey = langId \+ '\\0' \+ fontName \+ '\\0' \+ altType;/,
    '폰트 cache key는 언어·폰트 이름·폰트 타입만 사용해야 합니다',
  );
  assert.doesNotMatch(
    codeOnly(resolver),
    /password|credential|secret/i,
    '폰트 해소 cache가 credential을 입력으로 받아서는 안 됩니다',
  );

  const imports = runtimeSnapshotSource.slice(0, runtimeSnapshotSource.indexOf('const ROOT'));
  assert.doesNotMatch(
    imports,
    /main\.ts|wasm-bridge|hwp-password/i,
    'Node snapshot은 Studio 암호 open 모듈을 직접 불러오면 안 됩니다',
  );
  const rowsAndHash = between(runtimeSnapshotSource, 'function rowsAndHash', 'function substitutionRows');
  assert.match(
    rowsAndHash,
    /sha256:\s*sha256Text\(canonicalJson\(rows\)\)/,
    'SHA-256 입력은 canonical font snapshot rows로 한정해야 합니다',
  );
  assert.doesNotMatch(
    codeOnly(rowsAndHash),
    /password|credential|secret/i,
    'snapshot digest 경계가 credential을 받아서는 안 됩니다',
  );
});

test('WasmBridge는 다음 문서를 모두 준비한 뒤에만 기존 문서를 교체한다', () => {
  const atomic = between(bridgeSource, 'private loadDocumentAtomically', 'loadDocument(data: Uint8Array');
  assert.match(atomic, /createDocument: \(\) => HwpDocument,\s*\): void/, '원자적 교체 helper도 void command여야 한다');
  assert.doesNotMatch(atomic, /getDocumentInfo\(\)|return\s+info/, '교체 command 안에서 metadata query 결과를 만들거나 반환하지 않는다');
  assert.doesNotMatch(atomic, /this\.releaseDocument\(\)/, '실패 전에 현재 문서를 해제하지 않는다');
  assert.match(atomic, /requiresPasswordForSave: boolean/, '다음 문서의 보호 의도를 명시적으로 받는다');
  assert.match(atomic, /nextDoc = createDocument\(\);/, '일반/암호 문서 생성 경로를 한 계약으로 묶는다');
  assert.match(atomic, /nextDoc\.convertToEditable\(\);/, '교체 전에 편집 가능 상태를 준비한다');
  assert.match(atomic, /const previousDoc = this\.doc;\s*this\.doc = nextDoc;/, '준비 성공 뒤에만 현재 문서를 교체한다');
  assert.match(atomic, /this\._requiresPasswordForSave = requiresPasswordForSave;/,
    '문서 교체가 성공한 뒤 같은 commit 구간에서 보호 의도를 갱신한다');
  assert.match(atomic, /if \(previousDoc\)[\s\S]*previousDoc\.free\(\)/, '교체 완료 뒤에만 이전 문서를 해제한다');

  const plainLoad = between(bridgeSource, 'loadDocument(data: Uint8Array', 'loadDocumentWithPassword');
  assert.match(plainLoad, /loadDocumentAtomically\([\s\S]*false/, '평문 load는 보호 의도를 해제한다');
  assert.match(plainLoad, /return this\.getDocumentInfo\(\);/, '평문 public API도 교체 성공 뒤 별도 metadata query를 사용한다');
  const passwordLoad = between(
    bridgeSource,
    'loadDocumentWithPassword',
    'private async populateExternalImagesFromDevServer',
  );
  assert.match(passwordLoad, /HwpDocument\.openWithPassword\(data, password\)/, '암호 전용 WASM API를 노출한다');
  assert.match(passwordLoad, /loadDocumentAtomically\([\s\S]*true/, '암호 load는 보호 의도를 유지한다');
});

test('암호 대화상자는 마스킹·접근성·취소 시 DOM 값 제거를 제공한다', () => {
  assert.match(codeOnly(dialogSource), /this\.input\.type = 'password';/, '암호 입력을 마스킹한다');
  assert.match(codeOnly(dialogSource), /this\.input\.autocomplete = 'off';/, '브라우저 암호 자동완성을 요청하지 않는다');
  assert.match(codeOnly(dialogSource), /label\.htmlFor = 'hwp-password-input';/, '입력 레이블을 연결한다');
  assert.match(dialogSource, /this\.dialog\.setAttribute\('role', 'dialog'\)/, '대화상자 역할을 선언한다');
  assert.match(dialogSource, /this\.dialog\.setAttribute\('aria-modal', 'true'\)/, '모달 상태를 알린다');
  assert.match(codeOnly(dialogSource), /if \(this\.input\) this\.input\.value = '';/, '닫을 때 DOM의 입력값을 비운다');
  assert.match(codeOnly(dialogSource), /event\.key === 'Enter'/, 'Enter 확인을 지원한다');
});
