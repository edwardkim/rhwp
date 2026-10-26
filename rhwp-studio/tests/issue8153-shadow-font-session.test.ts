// 호스트 bytes 공급·세션 소유 검사다. 가짜 bytes를 실제 글꼴이나 시각 증거로 쓰지 않는다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { MetafileFontSession } from '../src/core/metafile-font-session.ts';
import { setHostFontProvider } from '../src/core/local-fonts.ts';
import type { HostFontData } from '../src/core/host-font-provider.ts';

let serial = 0;
async function fixture() {
  let read = async (): Promise<HostFontData> => ({ bytes: Uint8Array.of(31, 32, 33).buffer, faceIndex: 2 });
  await setHostFontProvider({
    async getSnapshot() {
      return { revision: `shadow-session-${++serial}`, faces: [{
        id: 'primary', family: 'Primary', fullName: 'Primary', postscriptName: 'Primary',
        style: 'Regular', weight: 400, slant: 'normal' as const,
      }] };
    },
    readFace() { return read(); },
    subscribe() { return () => {}; },
  });
  const calls: unknown[] = [];
  let shadowRequests = '[]';
  const request = JSON.stringify([{ families: ['Primary'], weight: 400, italic: false, text: '' }]);
  const owner = {
    getTextShadowFontRequests: () => shadowRequests,
    registerMetafileFont(family: string, bytes: Uint8Array, faceIndex: number) {
      calls.push({ family, bytes: [...bytes], faceIndex });
      return true;
    },
  };
  return { owner, calls, request, session: new MetafileFontSession(),
    useShadow() { shadowRequests = request; },
    setRead(next: typeof read) { read = next; } };
}

test('메타파일 없는 연속 그림자도 실제 face를 공급하고 변경된 요청을 반영한다', async () => {
  const f = await fixture();
  try {
    assert.equal(await f.session.prepare(f.owner, 1, () => true), true);
    assert.deepEqual(f.calls, []);
    f.useShadow();
    assert.equal(await f.session.prepare(f.owner, 1, () => true), true);
    await f.session.prepare(f.owner, 1, () => true);
    assert.deepEqual(f.calls, [{ family: 'Primary', bytes: [31, 32, 33], faceIndex: 2 }]);
  } finally { await setHostFontProvider(null); }
});

test('WMF와 글자 그림자가 같은 face를 요청하면 한 번만 공급한다', async () => {
  const f = await fixture();
  try {
    f.useShadow();
    await f.session.prepare({ ...f.owner, getMetafileFontRequests: () => f.request }, 1, () => true);
    assert.equal(f.calls.length, 1);
  } finally { await setHostFontProvider(null); }
});

test('이전 문서의 비동기 그림자 face는 문서 교체 뒤 등록하지 않는다', async () => {
  const f = await fixture();
  try {
    f.useShadow();
    let resolve!: (data: HostFontData) => void;
    let began!: () => void;
    const started = new Promise<void>(done => { began = done; });
    f.setRead(() => { began(); return new Promise(done => { resolve = done; }); });
    let current = true;
    const pending = f.session.prepare(f.owner, 1, () => current);
    await started;
    current = false;
    resolve({ bytes: Uint8Array.of(31, 32, 33).buffer });
    assert.equal(await pending, false);
    assert.deepEqual(f.calls, []);
  } finally { await setHostFontProvider(null); }
});
