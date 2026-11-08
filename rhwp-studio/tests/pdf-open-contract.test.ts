import test from 'node:test';
import assert from 'node:assert/strict';

import { isSupportedDocumentFileName, pickOpenFileHandle, saveDocumentToFileSystem } from '../src/command/file-system-access.ts';
import { openDocumentViaPicker } from '../src/command/file-open-picker.ts';
import { fileNameForFormat, resolveSaveTarget, markConvertedPdfSaveHandle, forgetConvertedPdfSaveHandle } from '../src/command/save-target.ts';

test('PDF file selection follows runtime capability while existing formats remain supported', () => {
  assert.equal(isSupportedDocumentFileName('report.PDF', true), true);
  assert.equal(isSupportedDocumentFileName('report.pdf', false), false);
  assert.equal(isSupportedDocumentFileName('report.pdf'), false);
  for (const name of ['report.hwp', 'report.hwpx', 'report.hml']) {
    assert.equal(isSupportedDocumentFileName(name, true), true);
    assert.equal(isSupportedDocumentFileName(name, false), true);
  }
});

test('native file picker exposes PDF only for the enabled WASM', async () => {
  for (const enabled of [false, true]) {
    let captured: any;
    const windowLike = { async showOpenFilePicker(options: any) { captured = options; return []; } };
    assert.equal(await pickOpenFileHandle(windowLike, enabled), null);
    assert.equal(Boolean(captured.types[0].accept['application/pdf']), enabled);
    assert.deepEqual(captured.types[0].accept['application/x-hwp'], ['.hwp']);
  }
});

test('picker fallback uses the same PDF capability and still waits for unsaved changes', async () => {
  const input: any = { dataset: {}, accept: '', click() { this.clicked = true; } };
  const deps = {
    canReplace: async () => true, windowLike: {}, findFileInput: () => input,
    emitOpenDocument() {}, warn() {}, alert() {}, pdfImportEnabled: true,
  };
  await openDocumentViaPicker(deps);
  assert.equal(input.clicked, true);
  assert.match(input.accept, /\.pdf/);
  input.clicked = false;
  await openDocumentViaPicker({ ...deps, canReplace: async () => false });
  assert.equal(input.clicked, false);
});

test('PDF import first save chooses a new editable document, never the source handle', () => {
  const inputHandle = { name: 'report.pdf' };
  const target = resolveSaveTarget('pdf', 'report.pdf', inputHandle);
  assert.equal(target.forceSaveAs, true);
  assert.equal(target.format, 'hwpx');
  assert.equal(target.suggestedName, 'report.hwpx');
  assert.equal(fileNameForFormat('report.pdf', 'hwp'), 'report.hwp');
});

test('PDF save writes only the selected output and subsequent saves retain its format', async () => {
  let sourceWrites = 0;
  const source = { name: 'report.pdf', async createWritable() { sourceWrites++; throw new Error('source write'); } };
  const writes: Blob[] = [];
  const output = {
    name: 'converted.hwp',
    async createWritable() { return { async write(data: Blob) { writes.push(data); }, async close() {} }; },
  };
  const target = resolveSaveTarget('pdf', source.name, source);
  const result = await saveDocumentToFileSystem({
    blob: new Blob(['converted content']), suggestedName: 'converted.hwp',
    currentHandle: source, saveFormat: 'hwp', forceSaveAs: target.forceSaveAs,
    windowLike: { async showSaveFilePicker() { return output; } },
  });
  assert.equal(sourceWrites, 0);
  assert.equal(writes.length, 1);
  assert.equal(await writes[0].text(), 'converted content');
  markConvertedPdfSaveHandle(result.handle);
  assert.deepEqual(resolveSaveTarget('pdf', result.fileName, result.handle), {
    format: 'hwp', forceSaveAs: false, suggestedName: 'converted.hwp',
  });
  forgetConvertedPdfSaveHandle(result.handle);
  assert.equal(resolveSaveTarget('pdf', result.fileName, result.handle).forceSaveAs, true);
});

test('Save As refuses the same PDF input handle even if its name looks like HWPX', async () => {
  let writes = 0;
  const original = { name: 'disguised.hwpx', async createWritable() { writes++; throw new Error('source write'); } };
  await assert.rejects(saveDocumentToFileSystem({
    blob: new Blob(['converted content']), suggestedName: original.name,
    currentHandle: original, saveFormat: 'hwpx', forceSaveAs: true,
    windowLike: { async showSaveFilePicker() { return original; } },
  }));
  assert.equal(writes, 0);
});
