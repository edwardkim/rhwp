import assert from 'node:assert/strict';
import test from 'node:test';
import {openSweepDocument} from '../export-wasm-png-for-sweep.mjs';

test('암호 입력 유무에 따라 같은 원본 바이트를 실제 WASM 문서 열기에 전달한다', () => {
  const bytes = Uint8Array.of(1,2,3), events = [];
  class HwpDocument {
    constructor(data) {events.push({kind:'plain',data});}
    static openWithPassword(data,password) {events.push({kind:'password',data,password});return {encrypted:true};}
  }
  openSweepDocument({HwpDocument},bytes);
  assert.deepEqual(openSweepDocument({HwpDocument},bytes,'  sample pass  '),{encrypted:true});
  openSweepDocument({HwpDocument},bytes,'');
  assert.deepEqual(events,[{kind:'plain',data:bytes},
    {kind:'password',data:bytes,password:'  sample pass  '},{kind:'password',data:bytes,password:''}]);
  assert.throws(()=>openSweepDocument({HwpDocument:class {}},bytes,'value'),/암호 문서/);
});
