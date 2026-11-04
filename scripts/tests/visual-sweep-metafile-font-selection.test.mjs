import assert from 'node:assert/strict';
import test from 'node:test';
import {collectSweepPaintFontRequests, selectMetafileFontFaces, selectSweepPaintFontFaces, selectTextEffectFontFaces} from '../visual-sweep-font-registration.mjs';
const font = (weight, style = 'normal', family = 'Face') => ({families: [family], weight, style});
const request = (weight, italic = false, families = ['Face']) => ({families, weight, italic});
const pick = (fonts, requests) => [...selectMetafileFontFaces(fonts, requests)].sort();
test('요청한 쪽의 그림·효과 글꼴만 조회하며 빈 결과를 전 문서 조회로 되돌리지 않는다', () => {
  const visited = [];
  const owner = {getPagePaintFontRequests(page) {
    visited.push(page);
    return JSON.stringify({metafile: page === 1 ? [{text:'A'}] : [], textEffects: page === 1 ? [{text:'·'}] : []});
  }, getMetafileFontRequests() {throw Error('전 문서 그림을 적재하면 안 됨');},
  getTextShadowFontRequests() {throw Error('전 문서 효과를 적재하면 안 됨');}};
  assert.deepEqual(collectSweepPaintFontRequests(owner,[1]),{metafile:[],effects:[]});
  assert.deepEqual(collectSweepPaintFontRequests(owner,[2,4]),{metafile:[{text:'A'}],effects:[{text:'·'}]});
  assert.deepEqual(visited,[0,1,3]);
  assert.throws(()=>collectSweepPaintFontRequests({getPagePaintFontRequests:()=> '{}'},[1]),/형식/);
});
test('페이지 자원 API가 없는 고정 base WASM은 기존 조회 계약을 유지한다', () => {
  assert.deepEqual(collectSweepPaintFontRequests({getMetafileFontRequests:()=> '[{"text":"old"}]',
    getTextShadowFontRequests:()=> '[{"text":"·"}]'},[1]),{metafile:[{text:'old'}],effects:[{text:'·'}]});
});
test('메타파일과 그림자가 실제 사용하는 글리프를 공급한 뒤 나머지 fallback은 등록하지 않는다', async () => {
  const fonts = ['First', 'Second', 'Unused', 'Shadow'].map(name => font(400, 'normal', name));
  const reads = [];
  const selected = await selectSweepPaintFontFaces({fontMissingText(bytes, index, text) {
    assert.equal(index, 0);
    return text.replaceAll(String.fromCharCode(bytes[0]), '');
  }}, fonts, [{...request(400, false, ['First', 'Second', 'Unused']), text: 'AB'}],
  [{...request(400, false, ['Shadow', 'Unused']), text: 'C'}], async record => {
    const family = record.families[0]; reads.push(family);
    return Uint8Array.of({First: 65, Second: 66, Unused: 88, Shadow: 67}[family]);
  });
  assert.deepEqual([...selected], ['first|400|normal', 'second|400|normal', 'shadow|400|normal']);
  assert.deepEqual(reads, ['First', 'Second', 'Shadow']);
});
test('같은 family의 분할 face가 다른 글리프를 공급하면 두 face를 확인한다', async () => {
  const fonts = [
    {...font(400), openTypeCff: true, id: 'latin'},
    {...font(400), openTypeCff: true, id: 'hangul'},
    {...font(400, 'normal', 'Fallback'), id: 'fallback'},
  ];
  const reads = [];
  const selected = await selectSweepPaintFontFaces({fontMissingText(bytes, index, text) {
    assert.equal(index, 0);
    return text.replaceAll(bytes[0] === 1 ? 'A' : '가', '');
  }}, fonts, [{...request(400, false, ['Face', 'Fallback']), text: 'A가'}], [], async record => {
    reads.push(record.id);
    return Uint8Array.of(record.id === 'latin' ? 1 : 2);
  });
  assert.deepEqual([...selected], ['face|400|normal']);
  assert.deepEqual(reads, ['latin', 'hangul']);
});
test('공급하지 않는 두 번째 TTF가 아니라 실제 fallback을 검사한다', async () => {
  const fonts = [
    {...font(400), id: 'first'},
    {...font(400), id: 'second'},
    {...font(400, 'normal', 'Fallback'), id: 'fallback'},
  ];
  const reads = [];
  const selected = await selectSweepPaintFontFaces({fontMissingText(bytes, index, text) {
    assert.equal(index, 0);
    return bytes[0] === 1 ? text : '';
  }}, fonts, [{...request(400, false, ['Face', 'Fallback']), text: 'A'}], [], async record => {
    reads.push(record.id);
    return Uint8Array.of(record.id === 'first' ? 1 : 2);
  });
  assert.deepEqual([...selected], ['fallback|400|normal']);
  assert.deepEqual(reads, ['first', 'fallback']);
});
test('cmap에 있지만 Canvas에서 제외된 빈 기호는 다음 글꼴에 넘긴다', async () => {
  const fonts = [font(400), font(400, 'normal', 'Fallback')];
  const reads = [];
  const selected = await selectSweepPaintFontFaces({
    fontMissingText: () => '',
    getFontSymbolUnicodeRange(bytes) {
      return bytes[0] === 1 ? 'U+0-26A9,U+26AB-10FFFF' : '';
    },
  }, fonts, [{...request(400, false, ['Face', 'Fallback']), text: '⚪'}], [], async record => {
    reads.push(record.families[0]);
    return Uint8Array.of(record.families[0] === 'Face' ? 1 : 2);
  });
  assert.deepEqual([...selected], ['fallback|400|normal']);
  assert.deepEqual(reads, ['Face', 'Fallback']);
});
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

// 공백 cmap을 모의해 공급 순서만 확인한다. 실제 글꼴이나 시각 검증이 아니다.
test('WMF 공백은 실제 cmap을 검사하고 없으면 다음 공급자를 선택한다', async () => {
  const fonts = [font(400, 'normal', 'First'), font(400, 'normal', 'Second'), font(400, 'normal', 'Unused')];
  const selected = await selectSweepPaintFontFaces({fontMissingTextWithSpaces(bytes, index, text) {
    assert.equal(index, 0);
    return bytes[0] === 2 ? '' : text;
  }}, fonts, [{...request(400, false, ['First', 'Second', 'Unused']), text: ' '}], [], async record =>
    Uint8Array.of({First: 1, Second: 2, Unused: 3}[record.families[0]]));
  assert.deepEqual([...selected], ['second|400|normal']);
});

test('공백 cmap 검사 API가 없으면 전체 fallback을 보존한다', async () => {
  const fonts = [font(400, 'normal', 'First'), font(400, 'normal', 'Second')];
  const selected = await selectSweepPaintFontFaces({fontMissingText: () => ''}, fonts,
    [{...request(400, false, ['First', 'Second']), text: ' '}], [], async () => Uint8Array.of(1));
  assert.deepEqual([...selected], ['first|400|normal', 'second|400|normal']);
});
