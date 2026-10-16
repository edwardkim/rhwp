import assert from 'node:assert/strict';
import test from 'node:test';
import {createHash, webcrypto} from 'node:crypto';
import {applySweepFontEnvironment, selectSweepProgramFaces, hasTrueTypeSfntSignature,
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

test('명시 대체가 없으면 Native와 같은 null 환경이며 미사용 사전 face를 선택하지 않는다', async () => {
  const doc = documentFixture();
  doc.getDocumentFontSelectionPlan = () => JSON.stringify({environmentId: null,
    requests: [{primary: ['Unused Legacy Face'], substitute: []}]});
  const result = await applySweepFontEnvironment(doc, null, async () => {
    throw Error('조판 선택에 쓰이지 않는 사전 face의 로드를 요구하면 안 된다');
  });
  assert.equal(result.source, 'noDeclaredSubstitutions');
  assert.equal(result.environment, null);
  assert.equal(doc.pageCount(), 3);
  assert.deepEqual(result.loaded, []);
});

test('명시 환경은 자동 선택보다 우선한다', async () => {
  const doc = documentFixture();
  const declared = {id: 'explicit', substitutions: {'Missing Original': 'Requested Program'}};
  const result = await applySweepFontEnvironment(doc, JSON.stringify(declared), async () => {
    throw Error('명시 환경을 자동 환경으로 덮어쓰면 안 된다');
  });
  assert.deepEqual(result.environment, declared);
  assert.equal(doc.pageCount(), 2);
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

test('공급 실패·hash 변경·실제 font 로드 거절은 준비 완료로 기록하지 않는다', async () => {
  for (const [options, expected] of [
    [{responseOk: false}, /바이트 공급/], [{badHash: true}, /hash/], [{loadError: true}, /OTS rejected/],
  ]) {
    const {host, face, events, plan} = programFixture(options);
    await assert.rejects(loadSweepFontPrograms(plan, [face], host), expected);
    assert.ok(!events.some(event => event.startsWith('add:')));
  }
});
