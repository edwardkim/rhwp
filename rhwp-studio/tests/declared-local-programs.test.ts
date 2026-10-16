import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { prepareDeclaredLocalPrograms } from '../src/core/declared-font-environment.ts';
import { getDetectedOSFonts } from '../src/core/font-loader.ts';
import { fontFamilyCandidatesForDisplay } from '../src/core/font-substitution.ts';
import { detectLocalFonts, resetLocalFontsForTests } from '../src/core/local-fonts.ts';

test('선언 프로그램은 승인된 face를 선택하고 검증되지 않은 local() 이름을 공급하지 않는다', async () => {
  const keys = ['FontFace', 'document', 'queryLocalFonts', 'localStorage'] as const;
  const previous = new Map(keys.map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  const created: Array<{ family: string; source: ArrayBuffer }> = [];
  const supplied: string[] = [];
  class LoadedFace {
    status = 'loaded';
    family: string;
    source: ArrayBuffer;
    constructor(family: string, source: ArrayBuffer) { this.family = family; this.source = source; created.push({ family, source }); }
    async load() { return this; }
  }
  const values = {
    FontFace: LoadedFace,
    document: { fonts: { add(face: LoadedFace) { supplied.push(face.family); } } },
    queryLocalFonts: async () => [{
      family: 'Verified Paint Face', fullName: 'Verified Paint Face Regular',
      postscriptName: 'VerifiedPaintFace-Regular', style: 'Regular', blob: async () => new Blob([new Uint8Array(4)]),
    }],
    localStorage: { getItem() { return null; }, setItem() {}, removeItem() {} },
  };
  for (const key of keys) Object.defineProperty(globalThis, key, { configurable: true, value: values[key] });
  resetLocalFontsForTests();
  const detected = getDetectedOSFonts() as Set<string>;
  const previouslyDetected = detected.has('휴먼명조');
  const plan = {
    environmentId: null,
    requests: [{ source: 'Unverified Original', primary: ['Unverified Original', 'Verified Paint Face'], substitute: [] }],
  };
  try {
    await detectLocalFonts({ force: true, includeRegistered: true });
    detected.add('휴먼명조');
    assert.notEqual(fontFamilyCandidatesForDisplay('휴먼명조')[0], '휴먼명조', '폭만 다른 플랫폼 placeholder를 원 face로 선택하지 않는다');
    const programs = await prepareDeclaredLocalPrograms(plan, () => true);
    assert.deepEqual(programs.map(program => [program.query, program.family]), [['Verified Paint Face', 'Verified Paint Face']]);
    assert.equal(programs[0].trueType, false, 'load 성공만으로 형식 미확인 바이트를 TrueType으로 선언하지 않는다');
    assert.ok(created.every(face => face.source instanceof ArrayBuffer));
    assert.ok(created.every(face => face.source === created[0].source), '별칭과 공급자는 같은 승인 프로그램 바이트를 사용한다');
    assert.ok(supplied.includes('Verified Paint Face'));
    assert.ok(supplied.includes('Unverified Original'), '원본 별칭은 확인된 대체 프로그램에 연결한다');
    const exactAlias = await prepareDeclaredLocalPrograms({
      environmentId: null,
      requests: [{ source: 'Verified Paint Face Regular', primary: ['Verified Paint Face Regular'], substitute: [] }],
    }, () => true);
    assert.deepEqual(exactAlias.map(program => [program.query, program.family]), [['Verified Paint Face Regular', 'Verified Paint Face Regular']], '같은 family의 다른 face를 덮어쓰지 않고 요청한 face 별칭을 유지한다');
    const previousSupply = supplied.length;
    assert.deepEqual(await prepareDeclaredLocalPrograms(plan, () => false), []);
    assert.equal(supplied.length, previousSupply, '오래된 문서의 공급은 적용하지 않는다');
    resetLocalFontsForTests();
    assert.deepEqual(await prepareDeclaredLocalPrograms(plan, () => true), [], '이전 프로그램 캐시가 현재 face 확인을 대신하지 않는다');
    assert.equal(supplied.length, previousSupply);
  } finally {
    if (!previouslyDetected) detected.delete('휴먼명조');
    resetLocalFontsForTests();
    for (const key of keys) {
      const descriptor = previous.get(key);
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
  }
});

test('승인된 SFNT의 같은 glyph 프로그램을 정규화한 뒤 선언 환경과 paint에 공급한다', async () => {
  const fixture = readFileSync(new URL('../../tests/fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf', import.meta.url));
  const original = fixture.buffer.slice(fixture.byteOffset, fixture.byteOffset + fixture.byteLength);
  const source = new DataView(original);
  const count = source.getUint16(4, false);
  const directoryEnd = 12 + count * 16;
  // SFNT 표는 4바이트 경계에 있어야 한다. 저장된 glyph/hmtx는 그대로 두고
  // 디렉터리 정렬만 잘못된 실제 로드 경계를 재현한다.
  const unaligned = new Uint8Array(original.byteLength + 1);
  unaligned.set(new Uint8Array(original, 0, directoryEnd));
  unaligned.set(new Uint8Array(original, directoryEnd), directoryEnd + 1);
  const raw = new DataView(unaligned.buffer);
  for (let i = 0; i < count; i++) {
    const record = 12 + i * 16;
    raw.setUint32(record + 8, source.getUint32(record + 8, false) + 1, false);
  }
  const keys = ['FontFace', 'document', 'queryLocalFonts', 'localStorage'] as const;
  const previous = new Map(keys.map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  const supplied: Array<{ family: string; source: ArrayBuffer }> = [];
  class ValidatedFace {
    status = 'loaded';
    family: string;
    source: ArrayBuffer;
    constructor(family: string, source: ArrayBuffer) { this.family = family; this.source = source; }
    async load() {
      const view = new DataView(this.source);
      for (let i = 0; i < view.getUint16(4, false); i++) {
        if (view.getUint32(12 + i * 16 + 8, false) % 4 !== 0) {
          throw new SyntaxError('unaligned SFNT table');
        }
      }
      return this;
    }
  }
  const values = {
    FontFace: ValidatedFace,
    document: { fonts: { add(face: ValidatedFace) { supplied.push(face); } } },
    queryLocalFonts: async () => [{
      family: 'RHWP Bitmap SVG Glyph Smoke', fullName: 'RHWP Bitmap SVG Glyph Smoke Regular',
      postscriptName: 'RHWPBitmapSvgGlyphSmoke-Regular', style: 'Regular',
      blob: async () => new Blob([unaligned]),
    }],
    localStorage: { getItem() { return null; }, setItem() {}, removeItem() {} },
  };
  for (const key of keys) Object.defineProperty(globalThis, key, { configurable: true, value: values[key] });
  resetLocalFontsForTests();
  try {
    await detectLocalFonts({ force: true, includeRegistered: true });
    const programs = await prepareDeclaredLocalPrograms({
      environmentId: null,
      requests: [{ source: 'Missing source', primary: ['Missing source'], substitute: ['RHWPBitmapSvgGlyphSmoke-Regular'] }],
    }, () => true);
    assert.equal(programs.length, 1, '로드 가능한 원본 프로그램을 디렉터리 오류만으로 버리지 않는다');
    assert.equal(programs[0].trueType, true, '실제 로드한 SFNT 형식을 공통 선택에 전달한다');
    const provider = supplied.find(face => face.family === 'RHWPBitmapSvgGlyphSmoke-Regular');
    assert.ok(provider);
    assert.ok(supplied.every(face => face.source === provider.source), '선택과 표시 별칭은 같은 바이트를 소비한다');
    const normalized = new DataView(provider.source);
    for (const tag of [0x676c7966, 0x686d7478]) { // glyf, hmtx
      const sourceRecord = Array.from({ length: count }, (_, i) => 12 + i * 16).find(offset => source.getUint32(offset, false) === tag);
      const targetRecord = Array.from({ length: normalized.getUint16(4, false) }, (_, i) => 12 + i * 16).find(offset => normalized.getUint32(offset, false) === tag);
      assert.notEqual(sourceRecord, undefined);
      assert.notEqual(targetRecord, undefined);
      const sourceOffset = source.getUint32(sourceRecord! + 8, false);
      const targetOffset = normalized.getUint32(targetRecord! + 8, false);
      const length = source.getUint32(sourceRecord! + 12, false);
      assert.deepEqual(new Uint8Array(provider.source, targetOffset, length), new Uint8Array(original, sourceOffset, length), '원본 윤곽·전진폭은 정규화로 바뀌지 않는다');
    }
  } finally {
    resetLocalFontsForTests();
    for (const key of keys) {
      const descriptor = previous.get(key);
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
  }
});
