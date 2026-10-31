import assert from 'node:assert/strict';
import test from 'node:test';
import {selectMetafileFontFaces, selectTextEffectFontFaces} from '../visual-sweep-font-registration.mjs';
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

// 이 검사는 공급 순서 계약이다. 모의 cmap 응답을 실제 SFNT·시각 검증으로 취급하지 않는다.
test('첫 실제 공급자를 선택하고 원 TTC 출처와 공급 SFNT의 face index를 구분한다', async () => {
  const fonts = [font(400), {...font(700, 'italic', 'Fallback'), faceIndex: 2}, font(400, 'normal', 'Unused')];
  const reads = [], probes = [];
  const selected = await selectTextEffectFontFaces({fontSupportsText(bytes, index, text) {
    probes.push({bytes: [...bytes], index, text});
    return bytes[0] === 2;
  }}, fonts, [{...request(700, true, ['Absent', 'Face', 'Fallback', 'Unused']), text: '·'}],
  async record => { reads.push(record.families[0]); return Uint8Array.of(record.faceIndex ?? 0).buffer; });
  assert.deepEqual([...selected], ['fallback|700|italic']);
  assert.deepEqual(reads, ['Face', 'Fallback']);
  assert.deepEqual(probes, [{bytes: [0], index: 0, text: '·'}, {bytes: [2], index: 0, text: '·'}]);
});

test('미공급·글리프 부재는 공급 성공으로 바꾸지 않는다', async () => {
  const fonts = [font(400), font(400, 'normal', 'Fallback')];
  let probes = 0;
  const selected = await selectTextEffectFontFaces({fontSupportsText() { probes++; return false; }},
    fonts, [{...request(400, false, ['Face', 'Fallback']), text: '·'}],
    async record => record.families[0] === 'Face' ? null : Uint8Array.of(1));
  assert.deepEqual([...selected], []);
  assert.equal(probes, 1);
});

test('전체 그림자 요청과 구버전 owner는 기존 fallback 사다리를 유지한다', async () => {
  const fonts = [font(400), font(400, 'normal', 'Fallback')];
  const base = request(400, false, ['Face', 'Fallback']);
  const load = () => { throw Error('전체 요청은 glyph probe를 하지 않는다'); };
  const probe = () => { throw Error('빈 text는 전체 공급 요청이다'); };
  const expected = ['face|400|normal', 'fallback|400|normal'];
  assert.deepEqual([...await selectTextEffectFontFaces({fontSupportsText: probe}, fonts,
    [{...base, text: ''}], load)], expected);
  assert.deepEqual([...await selectTextEffectFontFaces({}, fonts,
    [{...base, text: '·'}], load)], expected);
  assert.deepEqual([...await selectTextEffectFontFaces({fontSupportsText: () => true}, fonts,
    [{...base, text: '·'}, {...base, text: ''}], async () => Uint8Array.of(1))], expected);
});

test('서로 다른 실제 face가 맡는 글리프를 공통 missing 결과로 순서대로 공급한다', async () => {
  const fonts = [font(400), font(400, 'normal', 'Fallback'), font(400, 'normal', 'Unused')];
  const probes = [];
  const selected = await selectTextEffectFontFaces({fontMissingText(bytes, index, text) {
    probes.push({index, text});
    return text.replaceAll(bytes[0] === 1 ? 'A' : '·', '');
  }}, fonts, [{...request(400, false, ['Face', 'Fallback', 'Unused']), text: 'A·'}],
  async record => Uint8Array.of(record.families[0] === 'Face' ? 1 : 2));
  assert.deepEqual([...selected], ['face|400|normal', 'fallback|400|normal']);
  assert.deepEqual(probes, [{index: 0, text: 'A·'}, {index: 0, text: '·'}]);
});

test('글꼴 파싱 실패를 모든 글리프 공급으로 간주하지 않는다', async () => {
  const selected = await selectTextEffectFontFaces({fontMissingText: () => undefined},
    [font(400)], [{...request(400), text: 'A·'}], async () => Uint8Array.of(1));
  assert.deepEqual([...selected], []);
});
