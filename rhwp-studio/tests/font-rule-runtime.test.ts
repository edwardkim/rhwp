import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import {
  FONT_RULE_GOVERNMENT_SUCCESSORS,
  FONT_RULE_SUBSTITUTION_TABLES,
  FONT_RULE_WEBFONT_ENTRIES,
  generatedCanvas2dPaintRuleCount,
  generatedCanvasKitRuleCount,
  projectedSubstituteTargets,
  resolveProjectedCanvasKitFontPlan,
} from '../src/core/font-rule-runtime.ts';

test('Studio runtime은 W7 generated projection의 승인 분모를 전부 소비한다', () => {
  assert.equal(FONT_RULE_SUBSTITUTION_TABLES.length, 7);
  assert.equal(FONT_RULE_SUBSTITUTION_TABLES.flat().length, 265);
  assert.equal(FONT_RULE_GOVERNMENT_SUCCESSORS.length, 10);
  // 공개 글꼴과 정부상징 ROKG v2의 별도 공급 이름을 포함한다.
  assert.equal(FONT_RULE_WEBFONT_ENTRIES.length, 159);
  assert.equal(generatedCanvas2dPaintRuleCount(), 281);
  assert.equal(generatedCanvasKitRuleCount(), 164);
  assert.equal(
    FONT_RULE_SUBSTITUTION_TABLES.flat().every(entry => /^rule\./.test(entry[4])),
    true,
  );
  assert.equal(FONT_RULE_WEBFONT_ENTRIES.every(entry => /^rule\./.test(entry.ruleId)), true);
});

test('legacy SUBST_TABLES와 FONT_LIST literal은 production owner에서 제거됐다', async () => {
  const [substitutionSource, loaderSource] = await Promise.all([
    readFile(new URL('../src/core/font-substitution.ts', import.meta.url), 'utf8'),
    readFile(new URL('../src/core/font-loader.ts', import.meta.url), 'utf8'),
  ]);
  assert.doesNotMatch(substitutionSource, /const SUBST_TABLES[^=]*=\s*\[/);
  assert.doesNotMatch(loaderSource, /const FONT_LIST[^=]*=\s*\[/);
  assert.match(substitutionSource, /FONT_RULE_SUBSTITUTION_TABLES/);
  assert.match(loaderSource, /FONT_RULE_WEBFONT_ENTRIES/);
  assert.match(loaderSource, /resolveProjectedCanvasKitFontPlan/);
});

test('projectedSubstituteTargets는 등록 웹폰트 이름도 치환 대상 체인을 돌려준다', () => {
  const targets = projectedSubstituteTargets('한양중고딕', 0, 0);

  assert.equal(targets.length > 0, true);
  assert.equal(targets[0].face, 'HY중고딕');
  assert.match(targets[0].ruleId, /^rule\./);
  assert.deepEqual(projectedSubstituteTargets('없는글꼴', 0, 0), []);
  assert.deepEqual(projectedSubstituteTargets('', 0, 0), []);
});

// 관계형 공급 계약 검사다. PDF 좌표·픽셀 일치율을 고정하는 시각 검사가 아니다.
test('한겨레결체 원본 CDN은 두 별칭에 같은 프로그램을 공급하며 오프라인 성공으로 위장하지 않는다', () => {
  const origin = 'https://cdn.jsdelivr.net/npm/@noonnu/hankc@0.1.0/fonts/hankc-400.';
  for (const family of ['한겨레결체', 'Hankc']) {
    const entry = FONT_RULE_WEBFONT_ENTRIES.find(rule => rule.name === family);
    assert.equal(entry?.file, `${origin}woff2`);
    assert.equal(entry?.format, 'woff2');
    const online = resolveProjectedCanvasKitFontPlan([family]);
    assert.deepEqual(online.unavailableFonts, []);
    assert.equal(online.sources.length, 1);
    assert.equal(online.sources[0].url, `${origin}woff`);
    assert.ok(online.sources[0].aliases.includes(family));
    const offline = resolveProjectedCanvasKitFontPlan([family], { disableExternalWebFonts: true });
    assert.deepEqual(offline.sources, []);
    assert.deepEqual(offline.unavailableFonts, [family]);
  }
});
