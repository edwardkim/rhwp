import test from 'node:test';
import assert from 'node:assert/strict';

import { resolveCanvasKitFontPlan } from '../src/core/font-loader.ts';

test('CanvasKit font plan groups document aliases that share one bundled face', () => {
  const plan = resolveCanvasKitFontPlan(
    ['HY그래픽', 'Noto Sans KR'],
    { localFontBaseUrl: 'vscode-resource://extension/fonts/' },
  );

  assert.deepEqual(plan.unavailableFonts, []);
  assert.equal(plan.sources.length, 1);
  assert.equal(
    plan.sources[0].url,
    'vscode-resource://extension/fonts/NotoSansKR-Regular.woff2',
  );
  assert.ok(plan.sources[0].aliases.includes('HY그래픽'));
  assert.ok(plan.sources[0].aliases.includes('Noto Sans KR'));
});

test('CanvasKit font plan follows the existing Hanyang Jung Gothic substitution', () => {
  const plan = resolveCanvasKitFontPlan(['한양중고딕']);

  assert.deepEqual(plan.unavailableFonts, []);
  assert.equal(plan.sources.length, 1);
  assert.match(plan.sources[0].url, /NotoSansKR-Regular\.woff2$/);
  assert.ok(plan.sources[0].aliases.includes('한양중고딕'));
  assert.ok(plan.sources[0].aliases.includes('HY중고딕'));
});

test('CanvasKit font plan uses KoPub SFNT originals while CSS keeps web formats', () => {
  const plan = resolveCanvasKitFontPlan([
    'KoPub돋움체 Light',
    'KoPub바탕체 Bold',
    'KoPubWorld돋움체 Medium',
    'KoPubWorld바탕체 Light',
  ]);

  assert.deepEqual(plan.unavailableFonts, []);
  const sourceFor = (family: string) => plan.sources.find(source => source.aliases.includes(family));
  assert.match(sourceFor('KoPub돋움체 Light')?.url ?? '', /font-kopub@1\.0\.2\/fonts\/KoPubDotum-Light\.ttf$/);
  assert.match(sourceFor('KoPub바탕체 Bold')?.url ?? '', /font-kopub@1\.0\.2\/fonts\/KoPubBatang-Bold\.ttf$/);
  assert.match(sourceFor('KoPubWorld돋움체 Medium')?.url ?? '', /font-kopubworld@1\.0\.3\/fonts\/KoPubWorld-Dotum-Medium\.otf$/);
  assert.match(sourceFor('KoPubWorld바탕체 Light')?.url ?? '', /font-kopubworld@1\.0\.3\/fonts\/KoPubWorld-Batang-Light\.otf$/);
});

test('CanvasKit font plan fails closed for unavailable surface fonts', () => {
  const offline = resolveCanvasKitFontPlan(
    ['함초롬바탕', 'Times New Roman'],
    { disableExternalWebFonts: true },
  );
  assert.deepEqual(offline.sources, []);
  assert.deepEqual(offline.unavailableFonts, ['함초롬바탕', 'Times New Roman']);

  const extension = resolveCanvasKitFontPlan(
    ['한컴 윤고딕 230', 'Noto Sans KR'],
    {
      localFontBaseUrl: 'vscode-resource://extension/fonts',
      availableLocalFiles: new Set(['NotoSansKR-Regular.woff2']),
    },
  );
  assert.deepEqual(extension.unavailableFonts, ['한컴 윤고딕 230']);
  assert.equal(extension.sources.length, 1);
});
