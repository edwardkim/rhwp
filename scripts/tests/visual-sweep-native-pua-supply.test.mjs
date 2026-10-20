import test from 'node:test';
import assert from 'node:assert/strict';
import {prepareNativePuaFonts} from '../visual-sweep-font-registration.mjs';
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
