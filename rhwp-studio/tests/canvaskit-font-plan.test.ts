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
