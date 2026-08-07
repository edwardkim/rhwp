import test from 'node:test';
import assert from 'node:assert/strict';

import {
  DesktopFileHandle,
  createDesktopFileSystemWindow,
  dialogFiltersFromPickerTypes,
  type TauriFileSystemBridge,
} from '../src/command/desktop-file-system.ts';
import { SAVE_FORMAT_DETAILS } from '../src/command/save-format.ts';
import { saveDocumentToFileSystem } from '../src/command/file-system-access.ts';

interface FakeBridge extends TauriFileSystemBridge {
  openCalls: unknown[];
  saveCalls: unknown[];
  written: Map<string, Uint8Array>;
}

function createBridge(options: {
  openResult?: string | null;
  saveResult?: string | null;
  files?: Record<string, string>;
} = {}): FakeBridge {
  const files = new Map<string, Uint8Array>(
    Object.entries(options.files ?? {}).map(([path, text]) => [
      path,
      new TextEncoder().encode(text),
    ]),
  );
  const bridge: FakeBridge = {
    openCalls: [],
    saveCalls: [],
    written: new Map(),
    async openDialog(opts) {
      bridge.openCalls.push(opts);
      return options.openResult ?? null;
    },
    async saveDialog(opts) {
      bridge.saveCalls.push(opts);
      return options.saveResult ?? null;
    },
    async readFile(path) {
      const bytes = files.get(path);
      if (!bytes) throw new Error(`no such file: ${path}`);
      return bytes;
    },
    async writeFile(path, data) {
      bridge.written.set(path, data);
    },
  };
  return bridge;
}

test('dialogFiltersFromPickerTypes는 MIME accept 맵을 확장자 필터로 옮긴다', () => {
  const filters = dialogFiltersFromPickerTypes([{
    description: 'HWP/HWPX/HML 문서',
    accept: {
      'application/x-hwp': ['.hwp'],
      'application/hwp+zip': ['.hwpx'],
      'application/xml': ['.hml'],
      'text/xml': ['.hml'],
    },
  }]);

  assert.deepEqual(filters, [{
    name: 'HWP/HWPX/HML 문서',
    // 중복 확장자(.hml)는 한 번만, 앞의 '.' 은 제거된 형태여야 한다.
    extensions: ['hwp', 'hwpx', 'hml'],
  }]);
});

test('dialogFiltersFromPickerTypes는 types가 없으면 undefined를 반환한다', () => {
  assert.equal(dialogFiltersFromPickerTypes(undefined), undefined);
  assert.equal(dialogFiltersFromPickerTypes([]), undefined);
});

test('열기 다이얼로그가 고른 경로로 파일 핸들을 만든다', async () => {
  const bridge = createBridge({
    openResult: 'C:\\문서\\보고서.hwpx',
    files: { 'C:\\문서\\보고서.hwpx': 'hwpx-bytes' },
  });
  const windowLike = createDesktopFileSystemWindow(bridge);

  const handles = await windowLike.showOpenFilePicker!({
    types: [{ description: 'HWP 문서', accept: { 'application/x-hwp': ['.hwp'] } }],
  });

  assert.equal(handles.length, 1);
  // 경로가 아닌 파일명만 name 으로 노출한다 (웹 핸들과 같은 계약).
  assert.equal(handles[0].name, '보고서.hwpx');
  const file = await handles[0].getFile();
  assert.equal(await file.text(), 'hwpx-bytes');
  assert.deepEqual(bridge.openCalls, [{
    multiple: false,
    directory: false,
    filters: [{ name: 'HWP 문서', extensions: ['hwp'] }],
  }]);
});

test('열기 다이얼로그 취소는 웹 picker와 같은 AbortError로 전달된다', async () => {
  const windowLike = createDesktopFileSystemWindow(createBridge({ openResult: null }));

  await assert.rejects(
    () => windowLike.showOpenFilePicker!({}),
    (error: unknown) => error instanceof DOMException && error.name === 'AbortError',
  );
});

test('저장 다이얼로그 취소도 AbortError로 전달된다', async () => {
  const windowLike = createDesktopFileSystemWindow(createBridge({ saveResult: null }));

  await assert.rejects(
    () => windowLike.showSaveFilePicker!({ suggestedName: '문서.hwp' }),
    (error: unknown) => error instanceof DOMException && error.name === 'AbortError',
  );
});

test('저장 다이얼로그는 제안 파일명을 defaultPath로 넘긴다', async () => {
  const bridge = createBridge({ saveResult: 'C:\\문서\\새문서.hwp' });
  const windowLike = createDesktopFileSystemWindow(bridge);

  const handle = await windowLike.showSaveFilePicker!({
    suggestedName: '새문서.hwp',
    types: [SAVE_FORMAT_DETAILS.hwp.pickerType],
  });

  assert.equal(handle.name, '새문서.hwp');
  assert.deepEqual(bridge.saveCalls, [{
    defaultPath: '새문서.hwp',
    filters: [{ name: 'HWP 문서', extensions: ['hwp'] }],
  }]);
});

test('writable은 close 시점에 한 번만 경로에 기록한다', async () => {
  const bridge = createBridge();
  const handle = new DesktopFileHandle('C:\\문서\\출력.hwp', bridge);

  const writable = await handle.createWritable();
  await writable.write(new Blob(['부분1']));
  // close 이전에는 파일을 건드리지 않는다 — 중간 실패로 원본이 잘리지 않게 한다.
  assert.equal(bridge.written.size, 0);
  await writable.write(new Blob(['부분2']));
  await writable.close();

  assert.equal(bridge.written.size, 1);
  assert.equal(
    new TextDecoder().decode(bridge.written.get('C:\\문서\\출력.hwp')),
    '부분1부분2',
  );
});

test('isSameEntry는 경로가 같을 때만 참이다', async () => {
  const bridge = createBridge();
  const a = new DesktopFileHandle('C:\\문서\\같은.hwp', bridge);
  const b = new DesktopFileHandle('C:\\문서\\같은.hwp', bridge);
  const c = new DesktopFileHandle('D:\\다른\\같은.hwp', bridge);

  assert.equal(await a.isSameEntry(b), true);
  assert.equal(await a.isSameEntry(c), false);
});

test('권한 조회는 항상 granted — 사용자가 직접 고른 경로만 다룬다', async () => {
  const handle = new DesktopFileHandle('C:\\문서\\a.hwp', createBridge());
  assert.equal(await handle.queryPermission(), 'granted');
  assert.equal(await handle.requestPermission(), 'granted');
});

test('저장 로직은 데스크톱 핸들을 웹 핸들과 동일하게 재사용한다(덮어쓰기)', async () => {
  const bridge = createBridge();
  const currentHandle = new DesktopFileHandle('C:\\문서\\원본.hwp', bridge);

  const result = await saveDocumentToFileSystem({
    blob: new Blob(['저장본']),
    suggestedName: '원본.hwp',
    currentHandle,
    windowLike: createDesktopFileSystemWindow(bridge),
    forceSaveAs: false,
    saveFormat: 'hwp',
  });

  // picker를 다시 열지 않고 원본 경로에 그대로 덮어써야 한다.
  assert.equal(result.method, 'current-handle');
  assert.equal(bridge.saveCalls.length, 0);
  assert.equal(
    new TextDecoder().decode(bridge.written.get('C:\\문서\\원본.hwp')),
    '저장본',
  );
});
