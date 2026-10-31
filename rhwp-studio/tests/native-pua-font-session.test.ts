// 글꼴 공급의 비동기 소유 계약 검사다. mock bytes를 실제 cmap/시각 증거로 쓰지 않는다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { NativePuaFontSession } from '../src/core/native-pua-font-session.ts';
import { setHostFontProvider } from '../src/core/local-fonts.ts';
import type { HostFontData } from '../src/core/host-font-provider.ts';

let serial = 0;

test('굵은 HFT 요청에 실제 선택한 일반 OTF를 공급하고 세대가 바뀌면 다시 준비한다', async () => {
  const font = readFileSync(new URL('../../tests/fixtures/fonts/RHWPHftOutline.otf', import.meta.url));
  const bytes = font.buffer.slice(font.byteOffset, font.byteOffset + font.byteLength) as ArrayBuffer;
  const request = {family: 'Fixture', bold: true, italic: false};
  let reads = 0, calls = 0;
  await setHostFontProvider({
    async getSnapshot() { return {revision: 'otf-regular', faces: [{id: 'fixture', family: 'Fixture',
      fullName: 'Fixture', postscriptName: 'Fixture', style: 'Regular', weight: 400, slant: 'normal'}]}; },
    async readFace() { reads++; return {bytes}; }, subscribe() { return () => {}; },
  });
  const owner = {
    getNativePuaFontRequests: () => '[]', registerNativePuaFont: () => false,
    getNativeOtfFontRequests: () => JSON.stringify([request]),
    prepareNativeOtfFonts(provider: (raw: string) => Uint8Array | Uint8Array[] | null) {
      calls++; const selected = provider(JSON.stringify(request));
      assert.ok(selected instanceof Uint8Array);
      assert.equal(new DataView(selected.buffer).getUint32(0), 0x4f54544f); return true;
    },
    clearNativePuaFonts() {},
  };
  try {
    const session = new NativePuaFontSession();
    assert.equal(session.records(owner).length, 1);
    assert.equal(await session.prepare(owner, 1, () => true, () => true), true);
    assert.equal(await session.prepare(owner, 1, () => true, () => true), true);
    assert.equal(calls, 1); assert.equal(reads, 1);
    assert.equal(await session.prepare(owner, 2, () => true, () => true), true);
    assert.equal(calls, 2);
  } finally { await setHostFontProvider(null); }
});

test('같은 family의 분할 OTF를 빠짐없이 준비하고 호스트 순서로 전달한다', async () => {
  const font = readFileSync(new URL('../../tests/fixtures/fonts/RHWPHftOutline.otf', import.meta.url));
  const bytes = font.buffer.slice(font.byteOffset, font.byteOffset + font.byteLength) as ArrayBuffer;
  const request = {family: 'Split', bold: false, italic: false};
  await setHostFontProvider({
    async getSnapshot() { return {revision: 'split-otf', faces: ['Latin', 'Hangul'].map(id => ({
      id, family: 'Split', fullName: `Split ${id}`, postscriptName: `Split-${id}`,
      style: 'Regular', weight: 400, slant: 'normal' as const,
    }))}; },
    async readFace() { return {bytes}; }, subscribe() { return () => {}; },
  });
  const owner = {
    getNativePuaFontRequests: () => '[]', registerNativePuaFont: () => false,
    getNativeOtfFontRequests: () => JSON.stringify([request]),
    prepareNativeOtfFonts(provider: (raw: string) => Uint8Array | Uint8Array[] | null) {
      const selected = provider(JSON.stringify(request));
      assert.ok(Array.isArray(selected)); assert.equal(selected.length, 2);
      assert.ok(selected.every(item => new DataView(item.buffer).getUint32(0) === 0x4f54544f));
      return true;
    },
  };
  try {
    const session = new NativePuaFontSession();
    assert.deepEqual(session.records(owner).map(record => record.postscriptName), ['Split-Latin', 'Split-Hangul']);
    assert.equal(await session.prepare(owner, 1, record => record.postscriptName === 'Split-Latin', () => true), false,
      '일부 face만 준비됐을 때 분할 family의 폭을 먼저 적용하지 않는다');
    assert.equal(await session.prepare(owner, 1, () => true, () => true), true);
  } finally { await setHostFontProvider(null); }
});

test('굵은 face가 없는 분할 family는 일반 face에서 다음 PUA 프로그램을 확인한다', async () => {
  const font = readFileSync(new URL('../../tests/fixtures/fonts/RHWPHftOutline.otf', import.meta.url));
  const bytes = font.buffer.slice(font.byteOffset, font.byteOffset + font.byteLength) as ArrayBuffer;
  await setHostFontProvider({
    async getSnapshot() { return {revision: 'split-pua', faces: ['Latin', 'Symbols'].map(id => ({
      id, family: 'Split', fullName: `Split ${id}`, postscriptName: `Split-${id}`,
      style: 'Regular', weight: 400, slant: 'normal' as const,
    }))}; },
    async readFace() { return {bytes}; }, subscribe() { return () => {}; },
  });
  let calls = 0;
  const owner = {
    getNativePuaFontRequests: () => JSON.stringify([{family: 'Split', paint_families: ['Split'], bold: true, italic: false}]),
    registerNativePuaFont() { return ++calls === 2; },
  };
  try {
    const session = new NativePuaFontSession();
    assert.equal(session.records(owner).length, 2);
    assert.equal(await session.prepare(owner, 1, () => true, () => true), true);
    assert.equal(calls, 2);
  } finally { await setHostFontProvider(null); }
});
async function fixture(fallback = false) {
  const revision = `pua-session-${++serial}`;
  const families = fallback ? ['Primary', 'Fallback'] : ['Primary'];
  let read = async (id: string): Promise<HostFontData> =>
    ({ bytes: Uint8Array.of(id === 'Fallback' ? 2 : 1, 2, 3).buffer });
  await setHostFontProvider({
    async getSnapshot() {
      return { revision, faces: families.map(family => ({
        id: family, family, fullName: family, postscriptName: family,
        style: 'Regular', weight: 400, slant: 'normal' as const,
      })) };
    },
    readFace(id) { return read(id); },
    subscribe() { return () => {}; },
  });
  const calls: number[] = [];
  let clears = 0;
  const owner = {
    getNativePuaFontRequests: () => JSON.stringify([
      { family: 'Primary', paint_families: families, bold: false, italic: false },
    ]),
    registerNativePuaFont(_family: string, _bold: boolean, _italic: boolean, bytes: Uint8Array) {
      calls.push(bytes[0]);
      return !fallback || bytes[0] === 2;
    },
    clearNativePuaFonts() { clears++; },
  };
  return { owner, calls, clears: () => clears, session: new NativePuaFontSession(),
    setRead(next: typeof read) { read = next; } };
}

test('미준비 face는 대체 표시를 유지하고 준비된 뒤 공급하며 같은 요청은 멱등이다', { timeout: 5000 }, async () => {
  const f = await fixture();
  try {
    await f.session.prepare(f.owner, 1, () => false, () => true);
    assert.deepEqual(f.calls, []);
    await f.session.prepare(f.owner, 1, () => true, () => true);
    await f.session.prepare(f.owner, 1, () => true, () => true);
    assert.deepEqual(f.calls, [1]);
    f.session.clear(f.owner); f.session.clear(f.owner);
    assert.equal(f.clears(), 1);
  } finally { await setHostFontProvider(null); }
});

test('주 face의 glyph 지원이 없으면 준비한 fallback을 순서대로 공급한다', { timeout: 5000 }, async () => {
  const f = await fixture(true);
  try {
    assert.equal(f.session.records(f.owner).length, 2);
    await f.session.prepare(f.owner, 1, () => true, () => true);
    assert.deepEqual(f.calls, [1, 2]);
    await f.session.prepare(f.owner, 1, () => true, () => true);
    assert.deepEqual(f.calls, [1, 2]);
  } finally { await setHostFontProvider(null); }
});

test('같은 family의 PUA와 리터럴 공백을 별도 등록하고 두 역할을 세대 교체까지 보존한다', { timeout: 5000 }, async () => {
  const f = await fixture();
  const spaces: number[] = [];
  const owner = {
    ...f.owner,
    getNativePuaFontRequests: () => JSON.stringify([
      { family: 'Primary', bold: false, italic: false },
      { family: 'Primary', bold: false, italic: false, literal_figure_space: true },
    ]),
    registerLiteralFigureSpaceFont(_family: string, _bold: boolean, _italic: boolean, bytes: Uint8Array) {
      spaces.push(bytes[0]);
      return true;
    },
  };
  try {
    await f.session.prepare(owner, 1, () => true, () => true);
    await f.session.prepare(owner, 1, () => true, () => true);
    assert.deepEqual(f.calls, [1]);
    assert.deepEqual(spaces, [1]);
    await f.session.prepare(owner, 2, () => true, () => true);
    assert.deepEqual(f.calls, [1, 1]);
    assert.deepEqual(spaces, [1, 1]);
    assert.equal(f.clears(), 1);
  } finally { await setHostFontProvider(null); }
});

for (const cancel of ['문서 교체', '공급 해제'] as const) {
  test(`${cancel} 뒤 도착한 이전 bytes는 새 문서/글꼴 세대에 등록하지 않는다`, { timeout: 5000 }, async () => {
    const f = await fixture();
    try {
      let resolve!: (value: HostFontData) => void;
      let started!: () => void;
      const began = new Promise<void>(done => { started = done; });
      f.setRead(() => { started(); return new Promise(done => { resolve = done; }); });
      let current = true;
      const pending = f.session.prepare(f.owner, 1, () => true, () => current);
      await began;
      if (cancel === '문서 교체') current = false;
      else f.session.clear(f.owner);
      resolve({ bytes: Uint8Array.of(1, 2, 3).buffer });
      assert.equal(await pending, false);
      assert.deepEqual(f.calls, []);
    } finally { await setHostFontProvider(null); }
  });
}
