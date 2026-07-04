import test from 'node:test';
import assert from 'node:assert/strict';

import {
  clearAutosaveDrafts,
  deleteAutosaveDraft,
  getAutosaveDraft,
  listAutosaveDrafts,
  saveAutosaveDraft,
} from '../src/recovery/autosave-store.ts';

test('autosave store는 IndexedDB가 없으면 메모리 폴백으로 draft를 저장한다', async () => {
  await clearAutosaveDrafts();

  await saveAutosaveDraft({
    id: 'draft-1',
    fileName: '문서.hwp',
    sourceFormat: 'hwp',
    savedAt: 100,
    byteLength: 3,
    data: new Uint8Array([1, 2, 3]),
    dirtyReason: 'typing',
  });

  const loaded = await getAutosaveDraft('draft-1');
  assert.ok(loaded);
  assert.equal(loaded.fileName, '문서.hwp');
  assert.equal(loaded.sourceFormat, 'hwp');
  assert.equal(loaded.byteLength, 3);
  assert.deepEqual([...loaded.data], [1, 2, 3]);

  const listed = await listAutosaveDrafts();
  assert.equal(listed.length, 1);
  assert.equal(listed[0].id, 'draft-1');

  await deleteAutosaveDraft('draft-1');
  assert.equal(await getAutosaveDraft('draft-1'), null);
});

test('autosave store는 draft 데이터를 복사해서 외부 변경과 분리한다', async () => {
  await clearAutosaveDrafts();
  const data = new Uint8Array([7, 8, 9]);

  await saveAutosaveDraft({
    id: 'draft-copy',
    fileName: 'copy.hwp',
    sourceFormat: 'hwp',
    savedAt: 200,
    byteLength: data.byteLength,
    data,
  });
  data[0] = 99;

  const loaded = await getAutosaveDraft('draft-copy');
  assert.ok(loaded);
  assert.deepEqual([...loaded.data], [7, 8, 9]);

  loaded.data[1] = 88;
  const loadedAgain = await getAutosaveDraft('draft-copy');
  assert.ok(loadedAgain);
  assert.deepEqual([...loadedAgain.data], [7, 8, 9]);
});
