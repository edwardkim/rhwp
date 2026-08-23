import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  analyzeMetricSource,
  assertMeasuredOverlayRegion,
  buildLineageManifest,
  buildPreSplitBaseline,
  compareManifest,
  loadMetricRepositorySource,
  validateLineageManifest,
  verifyApprovedBaseline,
} from '../font_metric_lineage.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const MANIFEST_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4964',
  'font_metric_lineage_manifest.json',
);

function readManifest() {
  return JSON.parse(fs.readFileSync(MANIFEST_PATH, 'utf8'));
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
  assert.deepEqual(verifyApprovedBaseline(ROOT), []);
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
  const original = loadMetricRepositorySource(ROOT);
  const source = original.includes('[FontMetric; 600]')
    ? original.replace('[FontMetric; 600]', '[FontMetric; 599]')
    : original.replace('[FontMetric; 595]', '[FontMetric; 594]');

  assert.throws(() => analyzeMetricSource(source), /parsed .*expected/);
});

test('swapping metric order changes composition and lookup projections', () => {
  const source = loadMetricRepositorySource(ROOT);
  const original = analyzeMetricSource(source);
  const swapped = analyzeMetricSource(swapFirstTwoMetricEntries(source));

  assert.notEqual(swapped.compositionSha256, original.compositionSha256);
  assert.notEqual(swapped.lookupProjectionSha256, original.lookupProjectionSha256);
});

test('a one-unit stored width change changes metric and exhaustive width projections', () => {
  const source = loadMetricRepositorySource(ROOT);
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
  const source = loadMetricRepositorySource(ROOT);
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

test('the 600-row lineage manifest is deterministic and closes W1/W5 evidence', () => {
  const expected = readManifest();
  const actual = buildLineageManifest(ROOT);

  assert.deepEqual(validateLineageManifest(actual, ROOT), []);
  assert.deepEqual(compareManifest(expected, actual), []);
  assert.deepEqual(actual.summary, {
    entryCount: 600,
    stableEntryIdCount: 600,
    w1MetricEntryLinks: 600,
    measuredOverlayEntries: 5,
    unknownOriginEntries: 595,
    fullySourceExactEntries: 0,
    partiallyByteVerifiedFontSources: 1,
    w5OracleProfileLinks: 2,
  });
});

test('unknown provenance cannot omit its reason or be promoted to verified', () => {
  const missingReason = readManifest();
  missingReason.entries[0].origin.reason = '';
  assert.match(validateLineageManifest(missingReason, ROOT).join('\n'), /origin unknown requires a reason/);

  const promoted = readManifest();
  promoted.entries[0].origin.status = 'verified';
  assert.match(validateLineageManifest(promoted, ROOT).join('\n'), /promotes a legacy generated entry/);
});

test('a broken W1 link and evidence digest fail closed', () => {
  const brokenRule = readManifest();
  brokenRule.entries[10].relations[0].relationId = 'rule.rust-metric.broken';
  assert.match(validateLineageManifest(brokenRule, ROOT).join('\n'), /must link its exact W1 metric-entry rule/);

  const brokenEvidence = readManifest();
  brokenEvidence.evidenceCatalog[0].sha256 = '0'.repeat(64);
  assert.match(validateLineageManifest(brokenEvidence, ROOT).join('\n'), /evidence digest drift/);
});

test('metric identity and semantic hashes must match the Rust source', () => {
  const changedIdentity = readManifest();
  changedIdentity.entries[20].metricIdentity.name = 'Changed Times';
  assert.match(validateLineageManifest(changedIdentity, ROOT).join('\n'), /metricIdentity differs from Rust source/);

  const changedWidth = readManifest();
  changedWidth.entries[20].semanticHashes.widthProjectionSha256 = 'f'.repeat(64);
  assert.match(validateLineageManifest(changedWidth, ROOT).join('\n'), /widthProjectionSha256 differs from Rust source/);
});
