import assert from 'node:assert/strict';
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
