import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const mainSource = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
const bridgeSource = readFileSync(new URL('../src/core/wasm-bridge.ts', import.meta.url), 'utf8');
const dialogSource = readFileSync(new URL('../src/ui/hwp-password-dialog.ts', import.meta.url), 'utf8');

function between(source: string, start: string, end: string): string {
  const startIndex = source.indexOf(start);
  assert.notEqual(startIndex, -1, `시작 표식이 있어야 합니다: ${start}`);
  const endIndex = source.indexOf(end, startIndex + start.length);
  assert.notEqual(endIndex, -1, `끝 표식이 있어야 합니다: ${end}`);
  return source.slice(startIndex, endIndex);
}

test('암호 문서는 명시적인 암호 필요 오류에서만 입력 UI로 전환한다', () => {
  const openPath = between(mainSource, 'async function loadDocumentForOpen', 'function showLoadErrorUnlessCancelled');
  assert.match(openPath, /^async function loadDocumentForOpen\(\s*data: Uint8Array,\s*fileName: string,?\s*\)/);
  const plainAttemptBody = openPath.match(
    /try \{(?<body>[\s\S]*?)\n\s*\} catch \(error\) \{/,
  )?.groups?.body.trim();
  assert.equal(plainAttemptBody, 'return wasm.loadDocument(data, fileName);', '일반 문서도 generic 진단 콜백 없이 연다');
  const passwordFallbackBody = openPath.match(
    /\} catch \(error\) \{(?<body>[\s\S]*?)\n\s*\}\n\}/,
  )?.groups?.body.trim();
  assert.equal(passwordFallbackBody, [
    'if (!isPasswordRequiredError(error)) throw error;',
    'return loadPasswordProtectedDocument(data, fileName);',
  ].join('\n    '));
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
  assert.match(mainSource, /async function loadDocumentForOpen[\s\S]*loadPasswordProtectedDocument/,
    'loadFile 이후 암호 감지와 password dialog 경로를 유지해야 합니다');
});

test('암호 입력은 단일 시도에만 쓰고, 취소와 오입력은 영속 경로에 도달하지 않는다', () => {
  const passwordPath = between(mainSource, 'async function loadPasswordProtectedDocument', 'async function loadDocumentForOpen');
  assert.match(passwordPath, /showHwpPasswordDialog\(fileName, retryMessage\)/, '문서 이름만 대화상자에 전달한다');
  assert.match(passwordPath, /^async function loadPasswordProtectedDocument\(\s*data: Uint8Array,\s*fileName: string,?\s*\)/);
  assert.deepEqual(passwordPath.split('\n').map(line => line.trim()).filter(line => /\bpassword\b/.test(line)), [
    'let password = await showHwpPasswordDialog(fileName, retryMessage);',
    'if (password === null) throw new DocumentOpenCancelledError();',
    'return wasm.loadDocumentWithPassword(data, password, fileName);',
    "password = '';",
  ]);
  const attemptBody = passwordPath.match(
    /try \{(?<body>[\s\S]*?)\n\s*\} catch \(error\) \{/,
  )?.groups?.body.trim();
  assert.equal(
    attemptBody,
    'return wasm.loadDocumentWithPassword(data, password, fileName);',
    '암호 시도는 generic 진단 콜백 없이 WASM을 직접 호출한다',
  );
  assert.doesNotMatch(passwordPath, /localStorage|sessionStorage|addRecentDoc|autosave|documentDigest|console\./, '암호값을 영속/로그 경로로 보내지 않는다');
  assert.match(passwordPath, /암호가 일치하지 않거나 문서가 손상되었습니다\. 다시 입력하세요\./, '오입력/암호문 손상은 재입력 상태로 설명한다');
});

test('WasmBridge는 다음 문서를 모두 준비한 뒤에만 기존 문서를 교체한다', () => {
  const atomic = between(bridgeSource, 'private loadDocumentAtomically', 'loadDocument(data: Uint8Array');
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
  const passwordLoad = between(
    bridgeSource,
    'loadDocumentWithPassword',
    'private async populateExternalImagesFromDevServer',
  );
  assert.match(passwordLoad, /HwpDocument\.openWithPassword\(data, password\)/, '암호 전용 WASM API를 노출한다');
  assert.match(passwordLoad, /loadDocumentAtomically\([\s\S]*true/, '암호 load는 보호 의도를 유지한다');
});

test('암호 대화상자는 마스킹·접근성·취소 시 DOM 값 제거를 제공한다', () => {
  assert.match(dialogSource, /this\.input\.type = 'password';/, '암호 입력을 마스킹한다');
  assert.match(dialogSource, /this\.input\.autocomplete = 'off';/, '브라우저 암호 자동완성을 요청하지 않는다');
  assert.match(dialogSource, /label\.htmlFor = 'hwp-password-input';/, '입력 레이블을 연결한다');
  assert.match(dialogSource, /this\.dialog\.setAttribute\('role', 'dialog'\)/, '대화상자 역할을 선언한다');
  assert.match(dialogSource, /this\.dialog\.setAttribute\('aria-modal', 'true'\)/, '모달 상태를 알린다');
  assert.match(dialogSource, /if \(this\.input\) this\.input\.value = '';/, '닫을 때 DOM의 입력값을 비운다');
  assert.match(dialogSource, /event\.key === 'Enter'/, 'Enter 확인을 지원한다');
});
