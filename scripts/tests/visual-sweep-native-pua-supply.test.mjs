import test from 'node:test';
import assert from 'node:assert/strict';
import {prepareNativePuaFonts, prepareHftOtfFonts} from '../visual-sweep-font-registration.mjs';

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
    {families: ['Installed'], weight: 400, style: 'normal', sha256: 'cff'},
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
