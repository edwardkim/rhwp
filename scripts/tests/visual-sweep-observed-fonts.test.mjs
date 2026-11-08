import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import test from 'node:test';
import {validateObservedFontReceipt} from '../export-wasm-png-for-sweep.mjs';
import {prepareObservedFontFallbacks} from '../visual-sweep-font-registration.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const request = {family: 'Absent face', fallback: 'Stored fallback', bold: false, italic: false};
const bytes = Uint8Array.of(1, 3, 5).buffer;
const source = {family: 'Installed fallback', sha256: sha('source program'), faceIndex: 2};
const records = [{request, source}];
const font = {families: [source.family, request.fallback], weight: 400, style: 'normal',
  sourceSha256: source.sha256, faceIndex: 2, sha256: sha(new Uint8Array(bytes)), repairs: ['collection-face']};
const receipt = {sourceSha256: sha('document'), declaredFontEnvironment: null, observedFontFallbacks: records};
const owner = calls => ({getDeclaredFontFallbackRequests: () => JSON.stringify([request]),
  setObservedFontFallbacks: value => calls.push(JSON.parse(value))});

test('입력 바이트와 사용자 환경이 같은 관측만 적용한다', () => {
  assert.deepEqual(validateObservedFontReceipt(receipt, Buffer.from('document'), null), records);
  assert.throws(() => validateObservedFontReceipt(receipt, Buffer.from('another document'), null), /원본 입력/);
  assert.throws(() => validateObservedFontReceipt(receipt, Buffer.from('document'), '{"id":"user"}'), /사용자 환경/);
  assert.throws(() => validateObservedFontReceipt({...receipt, declaredFontEnvironment: undefined},
    Buffer.from('document'), null), /사용자 환경/);
});

test('동일 프로그램의 collection face를 설치한 뒤에만 측정 환경을 확정한다', async t => {
  const events = [];
  // Node에는 FontFace가 없으므로 브라우저 경계만 대체한다.
  class Face {
    constructor(family, data, descriptors) {Object.assign(this, {family, data, descriptors});}
    async load() {events.push('load'); return this;}
  }
  for (const [key, value] of Object.entries({FontFace: Face, document: {fonts: {add(face) {
    assert.equal(face.family, source.family);
    assert.equal(face.data, bytes);
    assert.deepEqual(face.descriptors, {weight: '400', style: 'normal'});
    events.push('install');
  }}}})) {
    const original = Object.getOwnPropertyDescriptor(globalThis, key);
    Object.defineProperty(globalThis, key, {configurable: true, writable: true, value});
    t.after(() => {
      if (original) Object.defineProperty(globalThis, key, original);
      else delete globalThis[key];
    });
  }
  const calls = [], target = owner(calls);
  const proofs = await prepareObservedFontFallbacks({...target, setObservedFontFallbacks(value) {
    events.push('apply'); target.setObservedFontFallbacks(value);
  }}, [font], records, async () => bytes);
  assert.deepEqual(events, ['load', 'install', 'apply']);
  assert.deepEqual(calls, [records]);
  assert.equal(proofs[0].source.sha256, source.sha256);
  assert.equal(proofs[0].source.faceIndex, 2);
  assert.equal(proofs[0].suppliedSha256, font.sha256);
});

test('관측이 없으면 이전 WASM에도 새 API나 글꼴 읽기를 요구하지 않는다', async () => {
  assert.deepEqual(await prepareObservedFontFallbacks({}, [], [], () => {throw Error('read');}), []);
});

test('다른 collection face, 변경된 바이트와 원 face 존재를 거부한다', async () => {
  for (const [fonts, loadBytes, pattern] of [
    [[{...font, faceIndex: 0}], async () => bytes, /관측과 다릅니다/],
    [[font], async () => Uint8Array.of(9).buffer, /바이트가 변경/],
    [[font, {...font, families: [request.family]}], async () => bytes, /원 face/],
  ]) {
    const calls = [];
    await assert.rejects(prepareObservedFontFallbacks(owner(calls), fonts, records, loadBytes), pattern);
    assert.deepEqual(calls, [], '공급 검증 실패로 측정 환경이 바뀌면 안 된다');
  }
});

test('같은 family의 다른 프로그램이 Canvas 선택에서 우선하면 거부한다', async () => {
  const calls = [], other = {...font, sourceSha256: sha('another program')};
  await assert.rejects(prepareObservedFontFallbacks(owner(calls), [other, font], records,
    async () => bytes), /관측과 다릅니다/);
  assert.deepEqual(calls, []);
});

test('원본에 없는 요청과 중복 관측으로 실제 공급을 시작하지 않는다', async () => {
  for (const values of [[{request: {...request, family: 'Unrelated'}, source}], [...records, ...records]]) {
    const calls = [];
    await assert.rejects(prepareObservedFontFallbacks(owner(calls), [font], values,
      async () => bytes), /원본에 없는|중복/);
    assert.deepEqual(calls, []);
  }
});
