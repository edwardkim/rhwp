import assert from 'node:assert/strict';
import test from 'node:test';
import {selectMetafileFontFaces} from '../visual-sweep-font-registration.mjs';
const font = (weight, style = 'normal', family = 'Face') => ({families: [family], weight, style});
const request = (weight, italic = false, families = ['Face']) => ({families, weight, italic});
const pick = (fonts, requests) => [...selectMetafileFontFaces(fonts, requests)].sort();
test('요청하지 않은 100·200·300·500·600·800·900은 paint에 복제하지 않는다', () => {
  assert.deepEqual(pick([100,200,300,400,500,600,700,800,900].map(w => font(w)), [request(400),request(700)]), ['face|400|normal','face|700|normal']);
});
test('400과 500의 우선순위 및 굵고 가는 탐색 방향은 fontdb와 같다', () => {
  for (const [wanted, available, expected] of [[400,[300,500],500],[500,[400,600],400],[300,[200,400],200],[700,[600,900],900],[700,[400],400],[400,[100],100],[350,[500,700],500]]) {
    assert.deepEqual(pick(available.map(w => font(w)),[request(wanted)]),[`face|${expected}|normal`]);
  }
});
test('style을 먼저 고르고 누락 style은 fontdb의 순서로 대체한다', () => {
  assert.deepEqual(pick([font(400),font(700,'italic')],[request(700)]),['face|400|normal']);
  assert.deepEqual(pick([font(400),font(700,'oblique')],[request(400,true)]),['face|700|oblique']);
});
test('동일 source의 별칭·fallback을 보존하고 없는 family는 만들지 않는다', () => {
  const fonts = [font(400),font(700,'normal','Fallback')];
  assert.deepEqual(pick(fonts,[request(700,false,['FACE','Fallback','Absent']),request(400)]),['face|400|normal','fallback|700|normal']);
  assert.deepEqual(pick(fonts,[]),[]);
});
