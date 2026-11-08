// 관계형: TTF/HFT 요청의 실제 프로그램 소유·공급 순서와 공유 예산을 구분한다.
// 주석 갱신: 2026-11-05T11:07:01+09:00 (Asia/Seoul). 직전 파일 변경: 93f3043ff764d8cb11c67a4870d99bf9bf1d2b00 2026-11-05T09:47:35+09:00.
// 실행 근거: 2026-11-05 11:04 KST, 공개 기본 글꼴 5개 격리에서 11 PASS·0 FAIL·0 SKIP.
// 입력은 이 파일의 합성 family·프로그램 표식 바이트다. 실물 쪽·절대 px·golden은 사용하지 않는다.
// 원본 HWP/HWPX·독립 Print·Native/fresh WASM raw는 이 합성 계약의 직접 입력이 아니므로 비대상이다.
// 공급 모듈의 실제 조판 소비는 별도 전쪽 측정으로 구분한다: c183 Native/fresh WASM 각 80입력·406쪽,
// 최저 raw 92.45232% / 92.87288%, 96 DPI·Print·색상 경계 보정 전 2px 실루엣.
// 근거: mydocs/pr/assets/planet6897_20261103/current-c183a1ac61/{native,wasm}-validation.md.
// 아래 선행 주석과 실행 코드는 보존했다. 새로 덧붙인 첫머리는 검사 동작을 바꾸지 않는다.

import test from 'node:test';
import assert from 'node:assert/strict';
import {prepareNativePuaFonts, prepareHftOtfFonts} from '../visual-sweep-font-registration.mjs';

// 관계형: 선언 체인의 실제 프로그램 공급과 TTF 원본·대체의 소유를 구분한다.
// 주석 갱신: 2026-11-04 15:12 KST. 합성 공급자 바이트이며 실물 좌표 핀이 아니다.
test('콤마 선언 체인에서도 원본 TTF와 HFT의 실제 프로그램을 준비한다', async () => {
  const requests = [
    {family: 'Original, Display', bold: false, italic: false, source_ttf: true},
    {family: 'Missing, Display', bold: false, italic: false, source_ttf: true},
    {family: 'Missing, Legacy', bold: false, italic: false},
  ];
  const fonts = [
    {families: ['Original'], weight: 400, style: 'normal', sha256: 'original', signature: [0,1,0,0]},
    {families: ['Display'], weight: 400, style: 'normal', sha256: 'display', signature: [0,1,0,0]},
    {families: ['Legacy'], weight: 400, style: 'normal', sha256: 'legacy', openTypeCff: true, signature: [79,84,84,79]},
  ];
  const received = [];
  const owner = {getNativeOtfFontRequests: () => JSON.stringify(requests),
    prepareNativeOtfFonts(provider) {for (const request of requests) received.push(provider(JSON.stringify(request)));}};
  const proofs = await prepareHftOtfFonts(owner, fonts, async font => Uint8Array.from(font.signature).buffer);
  assert.ok(received[0] instanceof Uint8Array, '체인 전체를 단일 family 이름으로 조회하지 않는다');
  assert.equal(received[1], null, '다른 표시 프로그램을 원본 TTF로 등록하지 않는다');
  assert.deepEqual([...received[2]], [79,84,84,79]);
  assert.deepEqual(proofs.map(p => p.sha256), ['original','legacy']);
});

test('같은 이름의 HFT와 원본 TrueType 요청은 프로그램을 섞지 않는다', async () => {
  const requests = [
    {family: 'Mixed', bold: false, italic: false},
    {family: 'Mixed', bold: false, italic: false, source_ttf: true},
  ];
  const received = [];
  const owner = {
    getNativeOtfFontRequests: () => JSON.stringify(requests),
    prepareNativeOtfFonts(provider) {
      for (const request of requests) received.push(provider(JSON.stringify(request)));
    },
  };
  const fonts = [
    {families: ['Mixed'], weight: 400, style: 'normal', sha256: 'cff', openTypeCff: true, signature: [79, 84, 84, 79]},
    {families: ['Mixed'], weight: 400, style: 'normal', sha256: 'ttf', openTypeCff: false, signature: [0, 1, 0, 0]},
  ];
  await prepareHftOtfFonts(owner, fonts, async record => Uint8Array.from(record.signature).buffer);
  assert.deepEqual(received.map(bytes => Array.from(bytes.slice(0, 4))), fonts.map(font => font.signature));
});

test('실제 선택한 CFF만 전달하고 없는 face와 TTF는 HFT 경로에 남긴다', async () => {
  const requests = [
    {family: 'Installed', bold: true, italic: false},
    {family: 'TrueType', bold: false, italic: false},
    {family: 'Missing', bold: false, italic: false},
  ];
  const received = [];
  const owner = {getNativeOtfFontRequests: () => JSON.stringify(requests),
    prepareNativeOtfFonts(provider) { for (const request of requests) received.push(provider(JSON.stringify(request))); }};
  const fonts = [
    {families: ['Installed'], weight: 400, style: 'normal', sha256: 'ttf', openTypeCff: false},
    {families: ['Installed'], weight: 400, style: 'normal', sha256: 'cff', openTypeCff: true},
    {families: ['TrueType'], weight: 400, style: 'normal', sha256: 'ttf'},
  ];
  const proof = await prepareHftOtfFonts(owner, fonts, async face =>
    Uint8Array.from(face.sha256 === 'cff' ? [79, 84, 84, 79] : [0, 1, 0, 0]).buffer);
  assert.deepEqual([...received[0]], [79, 84, 84, 79]);
  assert.equal(received[1], null);
  assert.equal(received[2], null);
  assert.equal(proof[0].weight, 400, '굵은 face가 없을 때 실제 선택한 일반 face를 전달한다');
  assert.equal(proof.length, 1);
});
test('같은 family의 분할 CFF는 표시 후보 순서로 모두 전달한다', async () => {
  const request = {family: 'Split', bold: false, italic: false};
  let received;
  const owner = {getNativeOtfFontRequests: () => JSON.stringify([request]),
    prepareNativeOtfFonts(provider) { received = provider(JSON.stringify(request)); }};
  const fonts = ['first', 'second'].map((sha256, i) => ({families: ['Split'], weight: 400,
    style: 'normal', sha256, openTypeCff: true, index: i}));
  const proofs = await prepareHftOtfFonts(owner, fonts, async record =>
    Uint8Array.of(79, 84, 84, 79, record.index).buffer);
  assert.ok(Array.isArray(received));
  assert.deepEqual(received.map(bytes => bytes[4]), [0, 1]);
  assert.deepEqual(proofs.map(proof => proof.sha256), ['first', 'second']);
});
test('같은 family의 CFF 합계가 WASM 한도를 넘으면 공급 가능한 앞 face만 전달한다', async () => {
  const request = {family: 'Large split', bold: false, italic: false};
  let received;
  const owner = {getNativeOtfFontRequests: () => JSON.stringify([request]),
    prepareNativeOtfFonts(provider) { received = provider(JSON.stringify(request)); }};
  const fonts = ['first', 'second', 'third'].map(sha256 => ({families: ['Large split'],
    weight: 400, style: 'normal', sha256, openTypeCff: true}));
  const bytes = new ArrayBuffer(24 * 1024 * 1024);
  new DataView(bytes).setUint32(0, 0x4f54544f);
  const proofs = await prepareHftOtfFonts(owner, fonts, async () => bytes);
  assert.equal(received.length, 2);
  assert.deepEqual(proofs.map(proof => proof.sha256), ['first', 'second']);
});
test('한 요청의 여러 family도 하나의 WASM 공급 예산을 공유한다', async () => {
  const request = {family: 'First, Second', bold: false, italic: false};
  let received;
  const owner = {getNativeOtfFontRequests: () => JSON.stringify([request]),
    prepareNativeOtfFonts(provider) { received = provider(JSON.stringify(request)); }};
  const fonts = ['First', 'Second'].flatMap(family => [0, 1].map(index => ({
    families: [family], weight: 400, style: 'normal',
    sha256: `${family}-${index}`, openTypeCff: true,
  })));
  const bytes = new ArrayBuffer(24 * 1024 * 1024);
  new DataView(bytes).setUint32(0, 0x4f54544f);
  const proofs = await prepareHftOtfFonts(owner, fonts, async () => bytes);
  assert.equal(received.length, 2, 'family가 바뀌어도 같은 요청의 예산을 초기화하지 않는다');
  assert.deepEqual(proofs.map(proof => proof.sha256), ['First-0', 'First-1']);
});
test('실제 적재한 face 순서로 PUA 공급을 판단하고 지원하지 않은 주 face 뒤의 fallback을 사용한다',async()=>{
 const oldDoc=globalThis.document,oldFont=globalThis.FontFace;
 const loaded=[];globalThis.document={fonts:new Set()};globalThis.FontFace=class{constructor(family,bytes){this.family=family;loaded.push([family,new Uint8Array(bytes)[0]])}async load(){return this}};
 try{
  const calls=[],owner={getNativePuaFontRequests:()=>JSON.stringify([{family:'Primary',paint_families:['Primary','Fallback'],bold:false,italic:false}]),registerNativePuaFont:(family,bold,italic,bytes)=>{calls.push([family,bytes[0]]);return bytes[0]===2}};
  const fonts=['Primary','Fallback'].map((family,i)=>({families:[family],weight:400,style:'normal',sha256:`sha${i}`,sourceSha256:`source${i}`,id:i+1}));
  const proof=await prepareNativePuaFonts(owner,fonts,async record=>Uint8Array.of(record.id,2,3).buffer);
  assert.deepEqual(loaded,[['Primary',1],['Fallback',2]]);assert.deepEqual(calls,[['Primary',1],['Primary',2]]);assert.equal(proof[0].selectedFamily,'Fallback');assert.equal(proof[0].sourceSha256,'source1');
 }finally{globalThis.document=oldDoc;globalThis.FontFace=oldFont}
});
test('구형 WASM에는 등록을 시도하지 않고 기존 표시 경로를 유지한다',async()=>{assert.deepEqual(await prepareNativePuaFonts({},[],()=>{throw Error('unexpected read')}),[])});

test('같은 family의 원본 TTF와 HFT 요청은 각 유형에 맞는 프로그램을 따로 공급한다', async () => {
  const oldDoc = globalThis.document, oldFont = globalThis.FontFace;
  globalThis.document = {fonts: new Set()};
  globalThis.FontFace = class { async load() { return this; } };
  try {
    const calls = [];
    const owner = {
      getNativePuaFontRequests: () => JSON.stringify([
        {family: 'Mixed', bold: false, italic: false},
        {family: 'Mixed', bold: false, italic: false, source_ttf: true},
      ]),
      registerNativePuaFont() { throw Error('유형별 등록을 사용해야 한다'); },
      registerNativePuaFontForSource(_family, _bold, _italic, sourceTtf, bytes) {
        calls.push([sourceTtf, bytes[0]]); return true;
      },
    };
    const fonts = [
      {families: ['Mixed'], weight: 400, style: 'normal', sha256: 'cff', openTypeCff: true, id: 79},
      {families: ['Mixed'], weight: 400, style: 'normal', sha256: 'ttf', openTypeCff: false, id: 0},
    ];
    await prepareNativePuaFonts(owner, fonts, async font => Uint8Array.of(font.id, 1, 0, 0).buffer);
    assert.deepEqual(calls, [[false, 79], [true, 0]]);
  } finally { globalThis.document = oldDoc; globalThis.FontFace = oldFont; }
});

test('분할 CFF의 첫 face가 PUA를 공급하지 못하면 다음 face를 검사한다', async () => {
  const oldDoc = globalThis.document, oldFont = globalThis.FontFace;
  globalThis.document = {fonts: new Set()};
  globalThis.FontFace = class {constructor(family) { this.family = family; } async load() { return this; }};
  try {
    const calls = [], owner = {getNativePuaFontRequests: () => JSON.stringify([
      {family: 'Split', paint_families: ['Split'], bold: false, italic: false},
    ]), registerNativePuaFont(_family, _bold, _italic, bytes) { calls.push(bytes[4]); return bytes[4] === 2; }};
    const fonts = [1, 2].map(id => ({families: ['Split'], weight: 400, style: 'normal',
      sha256: `sha${id}`, sourceSha256: `source${id}`, openTypeCff: true, id}));
    const proof = await prepareNativePuaFonts(owner, fonts, async record =>
      Uint8Array.of(79, 84, 84, 79, record.id).buffer);
    assert.deepEqual(calls, [1, 2]);
    assert.equal(proof[0].sha256, 'sha2');
  } finally { globalThis.document = oldDoc; globalThis.FontFace = oldFont; }
});

test('같은 실제 face를 한 번 적재하고 PUA·리터럴 공백의 서로 다른 등록 경로에 공급한다', async () => {
  const oldDoc = globalThis.document, oldFont = globalThis.FontFace;
  let reads = 0, installs = 0;
  globalThis.document = { fonts: new Set() };
  globalThis.FontFace = class {
    constructor() { installs++; }
    async load() { return this; }
  };
  try {
    const calls = [];
    const owner = {
      getNativePuaFontRequests: () => JSON.stringify([
        { family: 'Primary', bold: false, italic: false },
        { family: 'Primary', bold: false, italic: false, literal_figure_space: true },
      ]),
      registerNativePuaFont(...args) { assert.equal(this, owner); calls.push(['pua', ...args]); return true; },
      registerLiteralFigureSpaceFont(...args) { assert.equal(this, owner); calls.push(['space', ...args]); return true; },
    };
    const fonts = [{ families: ['Primary'], weight: 400, style: 'normal', sha256: 'same-program', sourceSha256: 'source' }];
    const proofs = await prepareNativePuaFonts(owner, fonts, async () => {
      reads++; return Uint8Array.of(1, 2, 3).buffer;
    });
    assert.equal(reads, 1);
    assert.equal(installs, 1);
    assert.deepEqual(calls.map(call => call[0]), ['pua', 'space']);
    assert.deepEqual(calls[0].slice(1), calls[1].slice(1));
    assert.deepEqual(proofs.map(proof => proof.literalFigureSpace), [false, true]);
  } finally { globalThis.document = oldDoc; globalThis.FontFace = oldFont; }
});
