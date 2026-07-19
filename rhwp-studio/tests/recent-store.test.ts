import test from 'node:test';
import assert from 'node:assert/strict';

import {
  addRecentDoc,
  clearRecentDocs,
  listRecentDocs,
  removeRecentDoc,
} from '../src/recent/recent-store.ts';
import type { FileSystemFileHandleLike } from '../src/command/file-system-access.ts';

/**
 * PR #2286 리뷰 회귀 테스트 (#2285 범위 + 메타-only 확장):
 * - 핸들 있으면 라이브 재열기용 저장, 없으면 메타-only 기록 (바이트 미보관)
 * - 동일 파일 판정은 isSameEntry 권위 (같은 파일명·다른 파일 공존)
 * - 최대 8개 상한, 목록 지우기
 * node 환경(IndexedDB 없음)이라 메모리 폴백 경로를 검증한다 — 스토어 로직은
 * withDb 양쪽 분기에 동일 규칙으로 구현되어 있다.
 */

/** isSameEntry가 참조 동일성으로 동작하는 테스트용 핸들. */
function makeHandle(key: string): FileSystemFileHandleLike {
  const self = {
    kind: 'file' as const,
    name: key,
    isSameEntry: async (other: unknown) => other === self,
    getFile: async () => {
      throw new Error('not used in store tests');
    },
  };
  return self as unknown as FileSystemFileHandleLike;
}

test('핸들 없는 열기는 메타-only 로 기록된다 (드롭/input/URL)', async () => {
  await clearRecentDocs();
  await addRecentDoc({ fileName: '드롭.hwp', sourceFormat: 'hwp' });
  await addRecentDoc({ fileName: '인풋.hwpx', sourceFormat: 'hwpx', handle: null });
  const docs = await listRecentDocs();
  assert.equal(docs.length, 2, '핸들 없는 열기도 목록에 남는다');
  for (const d of docs) {
    assert.equal(d.handle, undefined, '메타-only 항목은 핸들을 갖지 않는다');
    assert.ok(!('bytes' in d), '바이트 스냅샷을 보관하면 안 된다');
  }
  assert.deepEqual(docs.map((d) => d.fileName).sort(), ['드롭.hwp', '인풋.hwpx']);
});

test('같은 파일명은 핸들 유무와 무관하게 파일명 폴백으로 최신화된다 (메타-only)', async () => {
  await clearRecentDocs();
  await addRecentDoc({ fileName: 'dup.hwp', sourceFormat: 'hwp' });
  await new Promise((r) => setTimeout(r, 5));
  await addRecentDoc({ fileName: 'dup.hwp', sourceFormat: 'hwp' });
  const docs = await listRecentDocs();
  assert.equal(docs.length, 1, '핸들 없는 동명 문서는 파일명 비교로 중복 제거');
});

test('저장 항목은 핸들+메타만 보관하고 바이트를 갖지 않는다', async () => {
  await clearRecentDocs();
  await addRecentDoc({ fileName: 'a.hwp', sourceFormat: 'hwp', handle: makeHandle('a') });
  const [doc] = await listRecentDocs();
  assert.ok(doc.handle);
  assert.equal(doc.fileName, 'a.hwp');
  assert.ok(!('bytes' in doc), '바이트 스냅샷을 보관하면 안 된다 (#2285 보존 정책)');
});

test('같은 파일명이라도 isSameEntry=false면 별도 항목으로 공존한다', async () => {
  await clearRecentDocs();
  const h1 = makeHandle('dirA/문서.hwp');
  const h2 = makeHandle('dirB/문서.hwp');
  await addRecentDoc({ fileName: '문서.hwp', sourceFormat: 'hwp', handle: h1 });
  await addRecentDoc({ fileName: '문서.hwp', sourceFormat: 'hwp', handle: h2 });
  const docs = await listRecentDocs();
  assert.equal(docs.length, 2, '다른 경로의 동명 문서가 병합되면 안 된다');
  const handles = new Set(docs.map((d) => d.handle));
  assert.ok(handles.has(h1) && handles.has(h2));
});

test('동일 핸들(isSameEntry=true) 재열기는 중복 없이 최신화된다', async () => {
  await clearRecentDocs();
  const h = makeHandle('same.hwp');
  await addRecentDoc({ fileName: 'same.hwp', sourceFormat: 'hwp', handle: h });
  const firstAt = (await listRecentDocs())[0].openedAt;
  await new Promise((r) => setTimeout(r, 5));
  await addRecentDoc({ fileName: 'same.hwp', sourceFormat: 'hwp', handle: h });
  const docs = await listRecentDocs();
  assert.equal(docs.length, 1);
  assert.ok(docs[0].openedAt >= firstAt);
});

test('최대 8개 상한 — 가장 오래된 항목부터 밀려난다', async () => {
  await clearRecentDocs();
  for (let i = 0; i < 10; i++) {
    await addRecentDoc({ fileName: `f${i}.hwp`, sourceFormat: 'hwp', handle: makeHandle(`f${i}`) });
    await new Promise((r) => setTimeout(r, 2));
  }
  const docs = await listRecentDocs();
  assert.equal(docs.length, 8);
  assert.equal(docs[0].fileName, 'f9.hwp', '최신이 맨 앞');
  const names = docs.map((d) => d.fileName);
  assert.ok(!names.includes('f0.hwp') && !names.includes('f1.hwp'), '가장 오래된 2개 제거');
});

test('removeRecentDoc / clearRecentDocs', async () => {
  await clearRecentDocs();
  await addRecentDoc({ fileName: 'x.hwp', sourceFormat: 'hwp', handle: makeHandle('x') });
  await addRecentDoc({ fileName: 'y.hwp', sourceFormat: 'hwp', handle: makeHandle('y') });
  const docs = await listRecentDocs();
  await removeRecentDoc(docs[0].id);
  assert.equal((await listRecentDocs()).length, 1);
  await clearRecentDocs();
  assert.equal((await listRecentDocs()).length, 0);
});
