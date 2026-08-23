import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import {
  FONT_RULE_GOVERNMENT_SUCCESSORS,
  FONT_RULE_SUBSTITUTION_TABLES,
  FONT_RULE_WEBFONT_ENTRIES,
  generatedCanvas2dPaintRuleCount,
  generatedCanvasKitRuleCount,
} from '../src/core/font-rule-runtime.ts';

test('Studio runtime은 W7 generated projection의 승인 분모를 전부 소비한다', () => {
  assert.equal(FONT_RULE_SUBSTITUTION_TABLES.length, 7);
  assert.equal(FONT_RULE_SUBSTITUTION_TABLES.flat().length, 265);
  assert.equal(FONT_RULE_GOVERNMENT_SUCCESSORS.length, 10);
  assert.equal(FONT_RULE_WEBFONT_ENTRIES.length, 153);
  assert.equal(generatedCanvas2dPaintRuleCount(), 281);
  assert.equal(generatedCanvasKitRuleCount(), 158);
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
