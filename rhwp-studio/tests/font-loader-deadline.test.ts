// 비시각 계약 검사: FontFace 실패/대기 수명과 공급 등록 상태를 검사한다.
// 설치 글꼴·픽셀 좌표·한컴 출력 일치율을 기대값으로 사용하지 않는다.
import assert from 'node:assert/strict';
import test from 'node:test';

let sequence = 0;
async function fixture(t: any, mode: string) {
  const styles = new Map<string, any>();
  const requests: any[] = [], added = new Set<any>();
  const fetches: any[] = [];
  const revoked: string[] = [];
  let blobSequence = 0;
  t.mock.method(URL, 'createObjectURL', () => `blob:font-test-${++blobSequence}`);
  t.mock.method(URL, 'revokeObjectURL', (url: string) => revoked.push(url));
  const sources = new WeakMap<ArrayBuffer, string>();
  class Face {
    resolve!: (value: any) => void;
    reject!: (error: Error) => void;
    family: string;
    source: string;
    constructor(family: string, source: ArrayBuffer) {
      assert.ok(source instanceof ArrayBuffer, 'FontFace must not start an unabortable URL request');
      this.family = family; this.source = sources.get(source)!; requests.push(this);
      if (mode === 'constructor-failure') throw new Error('invalid font source');
    }
    load() {
      if (mode === 'alias-failure') return this.family === '함초롱바탕'
        ? Promise.reject(new Error('alias failure')) : Promise.resolve(this);
      if (mode === 'success' || (mode !== 'local-pending' && !this.source.includes('https://'))) return Promise.resolve(this);
      if (mode === 'failure') return Promise.reject(new Error('network failure'));
      return new Promise((resolve, reject) => { this.resolve = resolve; this.reject = reject; });
    }
  }
  const previous = ['document', 'FontFace', 'fetch'].map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)] as const);
  t.after(() => { for (const [key, value] of previous) {
    if (value) Object.defineProperty(globalThis, key, value);
    else Reflect.deleteProperty(globalThis, key);
  } });
  Object.defineProperty(globalThis, 'document', { configurable: true, value: {
    body: null, getElementById: (id: string) => styles.get(id),
    createElement: () => ({ id: '', textContent: '' }),
    head: { appendChild: (style: any) => styles.set(style.id, style) },
    fonts: { check: () => false, add: (face: any) => added.add(face), delete: (face: any) => added.delete(face) },
  } });
  Object.defineProperty(globalThis, 'FontFace', { configurable: true, value: Face });
  Object.defineProperty(globalThis, 'fetch', { configurable: true, value: (file: string, options: RequestInit) => {
    const bytes = new ArrayBuffer(8);
    sources.set(bytes, file);
    const response = { ok: true, arrayBuffer: async () => bytes };
    const request: any = { file, signal: options.signal };
    fetches.push(request);
    if (mode === 'network-pending' && file.startsWith('https://')) {
      return new Promise(resolve => { request.release = () => resolve(response); });
    }
    if (mode === 'body-pending' && file.startsWith('https://')) {
      return Promise.resolve({ ok: true, arrayBuffer: () => new Promise(resolve => {
        request.release = () => resolve(bytes);
      }) });
    }
    return Promise.resolve(response);
  } });
  const api = await import(`../src/core/font-loader.ts?deadline=${++sequence}`);
  t.mock.timers.enable({ apis: ['setTimeout'] });
  return { api, requests, fetches, added, revoked, css: () => styles.get('rhwp-web-font-faces')?.textContent ?? '' };
}
async function drain() { for (let i = 0; i < 30; i++) await Promise.resolve(); }

for (const mode of ['success', 'failure', 'pending'] as const) {
  test(`web font ${mode}: finite wait, matching supply, and no repeated failed request`, async t => {
    const f = await fixture(t, mode), progress: number[] = [];
    let complete = false;
    const loading = f.api.loadWebFonts([], (done: number) => progress.push(done)).then(() => { complete = true; });
    await drain();
    if (mode === 'pending') {
      assert.equal(complete, false);
      t.mock.timers.tick(4999); await drain(); assert.equal(complete, false);
      t.mock.timers.tick(1); await drain();
    }
    assert.equal(complete, true, 'unresponsive fonts must stop blocking startup after 5 seconds');
    await loading;
    assert.equal(f.api.getWebFontSupplySnapshot('함초롬바탕').status, mode === 'success' ? 'loaded' : 'catalogued');
    assert.equal(f.css().includes('"함초롬바탕"'), mode === 'success');
    assert.equal(f.css().includes('https://'), false, 'CSS must use already-fetched bytes');
    assert.ok([...f.added].some(face => face.family === 'RHWP Old Hangul'));
    assert.equal(progress.at(-1), progress.length);
    const count = f.requests.length;
    await f.api.loadWebFonts([]);
    assert.equal(f.requests.length, count);
    if (mode === 'pending') {
      for (const face of f.requests.filter(face => face.resolve)) face.resolve(face);
      await drain();
      assert.equal([...f.added].some(face => face.source.includes('https://')), false);
    }
  });
}

test('concurrent requests share pending faces; late rejection remains handled', async t => {
  const f = await fixture(t, 'pending');
  let completed = 0;
  const loads = [f.api.loadWebFonts([]), f.api.loadWebFonts([])].map(p => p.then(() => completed++));
  await drain();
  const identities = f.requests.map(face => `${face.family}:${face.source}`);
  assert.equal(new Set(identities).size, identities.length);
  t.mock.timers.tick(5000); await drain(); assert.equal(completed, 2);
  await Promise.all(loads);
  for (const face of f.requests.filter(face => face.reject)) face.reject(new Error('late failure'));
  await drain();
  assert.equal(f.css().includes('https://'), false);
});

test('external disabled keeps local supply without external requests or deadline waits', async t => {
  const f = await fixture(t, 'pending');
  await f.api.loadWebFonts([], undefined, { disableExternalWebFonts: true });
  assert.equal(f.requests.some(face => face.source.includes('https://')), false);
  assert.ok(f.added.size > 0);
});


test('failure of a file alias removes its CSS and already-added faces consistently', async t => {
  const f = await fixture(t, 'alias-failure');
  await f.api.loadWebFonts(['함초롱바탕']);
  for (const family of ['함초롬바탕', '함초롱바탕']) {
    assert.equal(f.api.getWebFontSupplySnapshot(family).status, 'catalogued');
    assert.equal([...f.added].some(face => face.family === family), false);
    assert.equal(f.css().includes(`"${family}"`), false);
  }
  assert.equal(f.api.getWebFontSupplySnapshot('함초롬돋움').status, 'loaded');
  assert.equal(f.revoked.length, 1, 'the failed alias file releases its shared blob');
  const count = f.requests.length;
  await f.api.loadWebFonts(['한컴바탕']);
  assert.equal(f.requests.length, count, 'a new alias cannot retry the failed file');
});

test('local resource stalls are bounded even when external fonts are disabled', async t => {
  const f = await fixture(t, 'local-pending');
  let complete = false;
  const loading = f.api.loadWebFonts([], undefined, { disableExternalWebFonts: true }).then(() => { complete = true; });
  await drain(); t.mock.timers.tick(5000); await drain();
  assert.equal(complete, true); await loading;
  assert.equal(f.api.getWebFontSupplySnapshot('RHWP Old Hangul').status, 'catalogued');
  assert.equal(f.added.size, 0);
});

test('success just before the deadline remains supplied after the deadline', async t => {
  const f = await fixture(t, 'pending');
  const loading = f.api.loadWebFonts([]);
  await drain(); t.mock.timers.tick(4999); await drain();
  for (const face of f.requests.filter(face => face.resolve)) face.resolve(face);
  await loading;
  const supplied = f.added.size;
  t.mock.timers.tick(5001); await drain();
  assert.equal(f.api.getWebFontSupplySnapshot('함초롬바탕').status, 'loaded');
  assert.equal(f.added.size, supplied);
  assert.equal(f.css().includes('"함초롬바탕"'), true);
});

for (const lateOutcome of ['success', 'failure']) {
  test(`concurrent aliases: a failed file revokes supplied faces and excludes late ${lateOutcome}`, async t => {
    const f = await fixture(t, 'pending');
    const first = f.api.loadWebFonts([]);
    await drain();
    for (const face of f.requests.filter(face => face.resolve)) face.resolve(face);
    await first;
    assert.equal(f.api.getWebFontSupplySnapshot('함초롬바탕').status, 'loaded');
    const loads = [f.api.loadWebFonts(['함초롱바탕']), f.api.loadWebFonts(['한컴바탕'])];
    await drain();
    const failing = f.requests.find(face => face.family === '함초롱바탕');
    const late = f.requests.find(face => face.family === '한컴바탕');
    assert.ok(failing && late);
    failing.reject(new Error('alias network failure'));
    await loads[0];
    assert.equal([...f.added].some(face => face.family === '함초롬바탕'), false);
    if (lateOutcome === 'success') late.resolve(late);
    else late.reject(new Error('late alias failure'));
    await loads[1];
    for (const family of ['함초롬바탕', '함초롱바탕', '한컴바탕']) {
      assert.equal(f.api.getWebFontSupplySnapshot(family).status, 'catalogued');
      assert.equal([...f.added].some(face => face.family === family), false);
      assert.equal(f.css().includes(`"${family}"`), false);
    }
    assert.equal(f.api.getWebFontSupplySnapshot('함초롬돋움').status, 'loaded');
    const count = f.requests.length;
    await f.api.loadWebFonts(['함초롱바탕', '한컴바탕']);
    assert.equal(f.requests.length, count);
  });
}

test('synchronous FontFace construction failure still completes and removes CSS', async t => {
  const f = await fixture(t, 'constructor-failure');
  await f.api.loadWebFonts([]);
  assert.equal(f.added.size, 0);
  assert.equal(f.css(), '');
  const count = f.requests.length;
  await f.api.loadWebFonts([]);
  assert.equal(f.requests.length, count);
});

for (const phase of ['network-pending', 'body-pending']) {
test(`${phase} aborts the shared request and ignores even a fetch that resolves after abort`, async t => {
  const f = await fixture(t, phase);
  const loads = [f.api.loadWebFonts(['함초롱바탕']), f.api.loadWebFonts(['한컴바탕'])];
  await drain();
  assert.equal(new Set(f.fetches.map(r => r.file)).size, f.fetches.length, 'aliases share network bytes');
  assert.equal(f.css().includes('https://'), false);
  t.mock.timers.tick(5000); await drain(); await Promise.all(loads);
  for (const request of f.fetches.filter(r => r.release)) {
    assert.equal(request.signal.aborted, true);
    request.release();
  }
  await drain();
  assert.equal(f.requests.some(face => face.source.includes('https://')), false);
  assert.equal(f.api.getWebFontSupplySnapshot('함초롬바탕').status, 'catalogued');
  const count = f.fetches.length;
  await f.api.loadWebFonts(['한컴바탕']);
  assert.equal(f.fetches.length, count);
});
}

test('successful aliases share fetched bytes and a CSS URL without a new network request', async t => {
  const f = await fixture(t, 'success');
  await f.api.loadWebFonts([]);
  const count = f.fetches.length;
  await Promise.all([f.api.loadWebFonts(['함초롱바탕']), f.api.loadWebFonts(['한컴바탕'])]);
  assert.equal(f.fetches.length, count);
  const rules = f.css().split('\n').filter((line: string) => /"(함초롬바탕|함초롱바탕|한컴바탕)"/.test(line));
  assert.equal(rules.length, 3);
  assert.equal(new Set(rules.map((line: string) => line.match(/url\("([^"]+)"\)/)![1])).size, 1);
  assert.equal(f.revoked.length, 0);
});
