import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  analyzeMetricSource,
  assertMeasuredOverlayRegion,
  buildPreSplitBaseline,
  canonicalJson,
  compareBaseline,
} from '../font_metric_lineage.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const SOURCE_PATH = path.join(ROOT, 'src', 'renderer', 'font_metrics_data.rs');
const BASELINE_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4964',
  'font_metric_pre_split_baseline.json',
);

function readBaseline() {
  return JSON.parse(fs.readFileSync(BASELINE_PATH, 'utf8'));
}

function swapFirstTwoMetricEntries(source) {
  const declarationAt = source.indexOf('static FONT_METRICS:');
  const firstAt = source.indexOf('FontMetric {', declarationAt);
  const firstEnd = source.indexOf('    },', firstAt) + '    },'.length;
  const secondAt = source.indexOf('FontMetric {', firstEnd);
  const secondEnd = source.indexOf('    },', secondAt) + '    },'.length;
  return `${source.slice(0, firstAt)}${source.slice(secondAt, secondEnd)}${source.slice(firstEnd, secondAt)}${source.slice(firstAt, firstEnd)}${source.slice(secondEnd)}`;
}

test('pre-split baseline is deterministic and matches the committed snapshot', () => {
  const first = buildPreSplitBaseline(ROOT);
  const second = buildPreSplitBaseline(ROOT);

  assert.equal(canonicalJson(first), canonicalJson(second));
  assert.deepEqual(compareBaseline(readBaseline(), first), []);
});

test('baseline closes the W1 population and the five measured overlays', () => {
  const baseline = buildPreSplitBaseline(ROOT);

  assert.equal(baseline.fontMetrics.entryCount, 600);
  assert.equal(baseline.fontMetrics.uniqueNameCount, 401);
  assert.deepEqual(baseline.fontMetrics.styleCounts, {
    regular: 383,
    bold: 89,
    italic: 79,
    boldItalic: 49,
  });
  assert.equal(baseline.fontMetrics.historicalGeneratedRegion.entryCount, 595);
  assert.deepEqual(baseline.fontMetrics.measuredOverlayRegion.names, [
    'HanyangSinMyeongJo',
    'HanyangJungGothic',
    'HanyangKyunMyeongJo',
    'HanyangKyunGothic',
    'HumanMyeongJo',
  ]);
});

test('a declared entry count mismatch is rejected instead of silently truncating', () => {
  const source = fs.readFileSync(SOURCE_PATH, 'utf8')
    .replace('[FontMetric; 600]', '[FontMetric; 599]');

  assert.throws(() => analyzeMetricSource(source), /FONT_METRICS parsed 600\/599/);
});

test('swapping metric order changes composition and lookup projections', () => {
  const source = fs.readFileSync(SOURCE_PATH, 'utf8');
  const original = analyzeMetricSource(source);
  const swapped = analyzeMetricSource(swapFirstTwoMetricEntries(source));

  assert.notEqual(swapped.compositionSha256, original.compositionSha256);
  assert.notEqual(swapped.lookupProjectionSha256, original.lookupProjectionSha256);
});

test('a one-unit stored width change changes metric and exhaustive width projections', () => {
  const source = fs.readFileSync(SOURCE_PATH, 'utf8');
  const changedSource = source.replace(
    'static FONT_0_LATIN_0: [u16; 95] = [\n    300,',
    'static FONT_0_LATIN_0: [u16; 95] = [\n    301,',
  );
  assert.notEqual(changedSource, source);

  const original = analyzeMetricSource(source);
  const changed = analyzeMetricSource(changedSource);
  assert.notEqual(changed.metricDataSha256, original.metricDataSha256);
  assert.notEqual(changed.widthProjection.sha256, original.widthProjection.sha256);
});

test('changing the final overlay identity is rejected by the population contract', () => {
  const source = fs.readFileSync(SOURCE_PATH, 'utf8');
  const changed = source.replace(
    'name: "HanyangSinMyeongJo"',
    'name: "HanyangSinMyeongJoChanged"',
  );
  const analysis = analyzeMetricSource(changed);

  assert.throws(
    () => assertMeasuredOverlayRegion(analysis.composition),
    /expected #2430 overlays are not the final five metric entries/,
  );
});
