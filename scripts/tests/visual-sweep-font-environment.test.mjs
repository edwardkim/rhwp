// 관계형: 실제 프로그램 준비·환경 선택·첫 조판의 순서를 검사한다.
// 합성 호스트 계약이며 특정 실물 페이지의 고정 px나 일치율을 주장하지 않는다.
import assert from 'node:assert/strict';
import test from 'node:test';
import {createHash, webcrypto} from 'node:crypto';
import {applySweepFontEnvironment, selectSweepProgramFaces, hasTrueTypeSfntSignature, hasOpenTypeCffSignature,
  loadSweepFontPrograms} from '../visual-sweep-font-environment.mjs';

function documentFixture() {
  let environment = null;
  const source = Object.freeze({text: '원문 본문', font: 'Missing Original'});
  return {
    source,
    getDocumentFontSelectionPlan: () => JSON.stringify({environmentId: null,
      requests: [{source: source.font, primary: [source.font], substitute: ['Installed Program']}]}),
    buildFontEnvironmentFromAvailablePrograms: json => JSON.parse(json).length
      ? JSON.stringify({id: 'prepared', substitutions: {'Missing Original': 'Installed Program'}}) : 'null',
    setFontEnvironment: json => { environment = JSON.parse(json); },
    // 공급 프로그램의 실제 metrics 선택은 페이지 수 계산보다 먼저 완료되어야 한다.
    pageCount: () => environment === null ? 3 : 2,
  };
}

test('준비된 프로그램의 공통 환경을 첫 페이지 수 계산 전에 적용한다', async () => {
  const doc = documentFixture(), original = doc.source;
  await applySweepFontEnvironment(doc, null, async () => ({programs: [
    {query: 'Installed Program', family: 'Installed Program', programId: 'loaded-program', trueType: true},
  ]}));
  assert.equal(doc.pageCount(), 2, 'portable metrics의 페이지 수로 작업자를 나누면 안 된다');
  assert.equal(doc.source, original, '선택 환경은 source font·본문을 편집하지 않는다');
});

test('대체가 없어도 설치된 원본 HFT 프로그램을 선택한다', async () => {
  const doc = documentFixture();
  doc.getDocumentFontSelectionPlan = () => JSON.stringify({environmentId: null,
    requests: [{primary: ['Installed Program'], substitute: []}]});
  const result = await applySweepFontEnvironment(doc, null, async () => ({programs: [
    {query: 'Installed Program', family: 'Installed Program', programId: 'loaded-cff', openTypeCff: true},
  ]}));
  assert.equal(result.source, 'preparedPrograms');
  assert.equal(doc.pageCount(), 2);
});

test('명시 환경은 자동 선택보다 우선한다', async () => {
  const doc = documentFixture();
  const declared = {id: 'explicit', substitutions: {'Missing Original': 'Requested Program'}};
  doc.buildFontEnvironmentFromAvailablePrograms = () => {
    throw Error('명시 환경을 자동 환경으로 덮어쓰면 안 된다');
  };
  const result = await applySweepFontEnvironment(doc, JSON.stringify(declared), async () => ({programs: []}));
  assert.deepEqual(result.environment, declared);
  assert.equal(doc.pageCount(), 2);
});

test('명시한 공급 프로그램도 첫 조판 전에 로드하고 사용자 환경은 그대로 적용한다', async () => {
  const doc = documentFixture();
  const declared = {id: 'explicit-programs', substitutions: {'Missing Original': 'Public Program'},
    documentGlyphFallbacks: [{languageSlot: 0, fontId: 0, source: 'Missing Original', faces: ['Public Symbol']}]};
  let ready = false;
  doc.buildFontEnvironmentFromAvailablePrograms = () => { throw Error('사용자 환경을 자동 선택으로 바꾸면 안 된다'); };
  doc.setFontEnvironment = json => {
    assert.ok(ready, '실제 프로그램을 준비하지 않으면 Canvas 측정은 설치되지 않은 이름의 시스템 대체 폭을 읽는다');
    assert.deepEqual(JSON.parse(json), declared);
  };
  const result = await applySweepFontEnvironment(doc, JSON.stringify(declared), async plan => {
    const queries = new Set(plan.requests.flatMap(r => [...r.primary, ...r.substitute]));
    assert.ok(queries.has('Public Program'), '명시한 주 프로그램의 바이트 준비');
    assert.ok(queries.has('Public Symbol'), '슬롯별 글리프 대체의 바이트 준비');
    await Promise.resolve();
    ready = true;
    return {programs: [{query: 'Public Program', family: 'Public Program', programId: 'verified-public'}],
      loaded: [{family: 'Public Program', sha256: 'verified-sha'}]};
  });
  assert.deepEqual(result.environment, declared);
});

test('명시 프로그램 준비가 실패하면 환경 적용과 첫 조판을 진행하지 않는다', async () => {
  const doc = documentFixture();
  let applied = false;
  doc.setFontEnvironment = () => { applied = true; };
  await assert.rejects(applySweepFontEnvironment(doc,
    JSON.stringify({id: 'unavailable', substitutions: {'Missing Original': 'Public Program'}}),
    async () => { throw Error('OTS rejected actual bytes'); }), /OTS rejected actual bytes/);
  assert.equal(applied, false);
});

test('준비된 프로그램이 없으면 환경을 만들어내지 않는다', async () => {
  const doc = documentFixture();
  const result = await applySweepFontEnvironment(doc, null, async () => ({programs: []}));
  assert.equal(result.environment, null);
  assert.equal(doc.pageCount(), 3);
});

test('자동 선택 API가 없는 과거 빌드는 명시 환경 없이 조용히 다른 조판을 사용하지 않는다', async () => {
  await assert.rejects(applySweepFontEnvironment({}, null, async () => ({programs: []})), /이전 WASM/);
  let applied;
  const explicit = '{"id":"verified-same-programs","substitutions":{}}';
  await applySweepFontEnvironment({setFontEnvironment: json => { applied = json; }}, explicit,
    async () => { throw Error('과거 빌드의 자동 선택을 호출하면 안 된다'); });
  assert.equal(applied, explicit);
});

test('명시 null 프로필은 선택 API를 추측하지 않고 nullable 환경을 해제한다', async () => {
  let applied = 'not-called';
  // 실제 WASM setter는 Option<String>이다. 문자열 "null"은 환경 객체가
  // 아니며, JS null만 None으로 전달된다. 자동 선택 API는 이 대조군에 없다.
  const doc = {setFontEnvironment: json => {
    if (typeof json === 'string') assert.notEqual(JSON.parse(json), null);
    applied = json;
  }};
  const result = await applySweepFontEnvironment(doc, ' \nnull\n', async () => {
    throw Error('명시한 해제를 임의 자동 선택으로 바꾸면 안 된다');
  });
  assert.equal(applied, null);
  assert.deepEqual(result, {source: 'explicit', environment: null});
});

test('일반 400 face를 준비하며 내장 face와 없는 후보를 설치 대체로 주장하지 않는다', () => {
  const plan = {requests: [{primary: ['Original', 'Embedded', 'Absent'], substitute: ['Fallback']}],
    embeddedFamilies: ['Embedded']};
  const regular = {families: ['Original'], weight: 400, style: 'normal'};
  const fallback = {families: ['Fallback'], weight: 400, style: 'normal'};
  const selected = selectSweepProgramFaces(plan, [
    {families: ['Original'], weight: 700, style: 'normal'},
    {families: ['Original'], weight: 400, style: 'italic'}, regular,
    {families: ['Embedded'], weight: 400, style: 'normal'}, fallback,
  ]);
  assert.deepEqual(selected, [{query: 'Original', face: regular}, {query: 'Fallback', face: fallback}]);
});

test('CFF와 collection을 개별 TrueType 프로그램으로 오인하지 않는다', () => {
  const bytes = value => Uint8Array.from(value).buffer;
  assert.equal(hasTrueTypeSfntSignature(bytes([0, 1, 0, 0])), true);
  assert.equal(hasTrueTypeSfntSignature(bytes([116, 114, 117, 101])), true);
  for (const value of [[], [0, 1], [79, 84, 84, 79], [116, 116, 99, 102]]) {
    assert.equal(hasTrueTypeSfntSignature(bytes(value)), false);
  }
  assert.equal(hasOpenTypeCffSignature(bytes([79, 84, 84, 79])), true);
  assert.equal(hasOpenTypeCffSignature(bytes([0, 1, 0, 0])), false);
});

// 관계형: 같은 원본·style 안의 실제 컨테이너 순서이며 페이지 px에 의존하지 않는다.
// 주석 갱신: 2026-11-07 20:25 KST.
test('같은 원본과 스타일은 TTF → TTC 일치 face → OTF 순서로 선택한다', () => {
  const plan = {requests: [{primary: ['Original'], substitute: []}], embeddedFamilies: []};
  const ttf = {families: ['Original'], weight: 400, style: 'normal', openTypeCff: false};
  const ttc = {...ttf, collectionFace: true};
  const otf = {...ttf, openTypeCff: true};
  for (const candidates of [[otf, ttc, ttf], [ttf, otf, ttc]]) {
    assert.deepEqual(selectSweepProgramFaces(plan, candidates), [{query: 'Original', face: ttf}]);
  }
  assert.deepEqual(selectSweepProgramFaces(plan, [otf, ttc]), [{query: 'Original', face: ttc}]);
  assert.deepEqual(selectSweepProgramFaces(plan, [otf]), [{query: 'Original', face: otf}]);
  const boldTtf = {...ttf, weight: 700};
  assert.deepEqual(selectSweepProgramFaces(plan, [boldTtf, otf]), [{query: 'Original', face: otf}],
    '원래 요청한 normal/400 스타일을 다른 스타일의 TTF로 바꾸지 않는다');
});

function programFixture({badHash = false, loadError = false, responseOk = true} = {}) {
  const bytes = Uint8Array.from([0, 1, 0, 0, 42]).buffer;
  const events = [], fonts = {ready: Promise.resolve(), add: face => events.push(`add:${face.family}`)};
  const face = {url: '/font/0', families: ['Installed', 'Alias'], weight: 400, style: 'normal',
    sha256: badHash ? '0'.repeat(64) : createHash('sha256').update(Buffer.from(bytes)).digest('hex'),
    sourceSha256: 'source-collection-hash', faceIndex: 2, repairs: []};
  const host = {crypto: webcrypto, document: {fonts},
    fetch: async url => { events.push(`fetch:${url}`); return {ok: responseOk, arrayBuffer: async () => bytes}; },
    FontFace: class {
      constructor(family) { this.family = family; this.status = 'unloaded'; }
      async load() {
        events.push(`load:${this.family}`);
        if (loadError) throw Error('OTS rejected');
        this.status = 'loaded'; return this;
      }
    }};
  return {host, face, events, plan: {requests: [{primary: ['Alias'], substitute: ['Installed']}]}};
}

test('실제 프로그램 hash·로드 성공 뒤에만 별칭과 face provenance를 반환한다', async () => {
  const {host, face, events, plan} = programFixture();
  const result = await loadSweepFontPrograms(plan, [face], host);
  assert.deepEqual(events, ['fetch:/font/0', 'load:Alias', 'add:Alias', 'load:Installed', 'add:Installed']);
  assert.deepEqual(result.programs.map(p => [p.query, p.family, p.trueType]),
    [['Alias', 'Installed', true], ['Installed', 'Installed', true]]);
  assert.ok(result.programs.every(p => p.programId === `sweep-${face.sha256}`));
  assert.ok(result.loaded.every(p => p.sourceSha256 === face.sourceSha256 && p.faceIndex === 2));
});

test('PostScript 별칭으로 공급해도 굵기를 합친 typographic family 대신 Native legacy family를 유지한다', async () => {
  const {host, face} = programFixture();
  face.families = ['Generic Family', 'Native Weight Family', 'NativeWeightPS'];
  face.primaryFamily = 'Native Weight Family';
  const result = await loadSweepFontPrograms({requests: [{primary: ['NativeWeightPS'], substitute: []}]}, [face], host);
  assert.equal(result.programs[0].query, 'NativeWeightPS');
  assert.equal(result.programs[0].family, 'Native Weight Family');
  assert.deepEqual(result.loaded.map(font => font.family), ['NativeWeightPS', 'Native Weight Family']);
});

test('공급 실패·hash 변경·실제 font 로드 거절은 준비 완료로 기록하지 않는다', async () => {
  for (const [options, expected] of [
    [{responseOk: false}, /바이트 공급/], [{badHash: true}, /hash/], [{loadError: true}, /OTS rejected/],
  ]) {
    const {host, face, events, plan} = programFixture(options);
    await assert.rejects(loadSweepFontPrograms(plan, [face], host), expected);
    assert.ok(!events.some(event => event.startsWith('add:')));
  }
});


test('문서 지정 대체가 없어도 조건부 공개 후보를 첫 페이지 수 이전에 준비한다', async () => {
  const doc = documentFixture();
  const publicName = 'RHWP Public Batang';
  doc.getDocumentFontSelectionPlan = () => JSON.stringify({environmentId: null,
    requests: [{source: 'Missing Original', primary: ['Missing Original'], substitute: [], publicFallback: [publicName]}]});
  doc.buildFontEnvironmentFromAvailablePrograms = json => {
    const programs = JSON.parse(json);
    return programs.some(p => p.query === publicName)
      ? JSON.stringify({id: 'actual-public-program', substitutions: {'Missing Original': publicName}}) : 'null';
  };
  let prepared = false;
  const result = await applySweepFontEnvironment(doc, null, async plan => {
    prepared = true;
    assert.deepEqual(plan.requests[0].publicFallback, [publicName]);
    return {programs: [{query: publicName, family: publicName, programId: 'supplied-public', trueType: true}]};
  });
  assert.equal(prepared, true);
  assert.equal(result.source, 'preparedPrograms');
  assert.equal(result.environment.substitutions['Missing Original'], publicName);
  assert.equal(doc.pageCount(), 2);
});

test('공개 fallback도 실제 공급 후보에 포함하며 내장 프로그램 이름은 보호한다', () => {
  const publicFace = {families: ['RHWP Public Batang'], weight: 400, style: 'normal'};
  const plan = {requests: [{primary: ['Missing'], substitute: [], publicFallback: ['RHWP Public Batang']}]};
  assert.deepEqual(selectSweepProgramFaces(plan, [publicFace]), [{query: 'RHWP Public Batang', face: publicFace}]);
  assert.deepEqual(selectSweepProgramFaces({...plan, embeddedFamilies: ['RHWP Public Batang']}, [publicFace]), []);
});

// 관계형: 고정 base의 이전 API에만 새 컨테이너 metadata를 제외하며 프로그램 신원은 보존한다.
// 주석 갱신: 2026-11-07 20:43 KST.
test('이전 base API의 collectionFace 필드 거부는 같은 프로그램으로만 호환한다', async () => {
  const program = {query: 'Original', family: 'Original', programId: 'same-ttc-face', trueType: true, collectionFace: true};
  const calls = [];
  const doc = {
    getDocumentFontSelectionPlan: () => JSON.stringify({environmentId: null, requests: [{primary: ['Original'], substitute: []}]}),
    buildFontEnvironmentFromAvailablePrograms: raw => {
      const programs = JSON.parse(raw); calls.push(programs);
      if (programs.some(program => 'collectionFace' in program)) throw Error('unknown field `collectionFace`');
      return JSON.stringify({id: 'base', substitutions: {Original: 'Original'}});
    },
    setFontEnvironment: () => {},
  };
  const result = await applySweepFontEnvironment(doc, null, async () => ({programs: [program]}));
  assert.deepEqual(calls[0], [program]);
  assert.deepEqual(calls[1], [{query: 'Original', family: 'Original', programId: 'same-ttc-face', trueType: true}]);
  assert.equal(result.collectionFaceProtocol, 'legacy-without-container-metadata');
  doc.buildFontEnvironmentFromAvailablePrograms = raw => {
    if (JSON.parse(raw).some(program => 'collectionFace' in program)) {
      throw Error('invalid font program snapshot: data did not match any variant of untagged enum AvailableFontProgramInput');
    }
    return 'null';
  };
  assert.equal((await applySweepFontEnvironment(doc, null, async () => ({programs: [program]})))
    .collectionFaceProtocol, 'legacy-without-container-metadata');
  doc.buildFontEnvironmentFromAvailablePrograms = () => { throw Error('selected program hash does not match'); };
  await assert.rejects(applySweepFontEnvironment(doc, null, async () => ({programs: [program]})), /hash does not match/,
    '실제 프로그램 오류를 호환 처리로 숨기지 않는다');
});
