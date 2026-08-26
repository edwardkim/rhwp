import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  assertLocalOutputPath,
  selectPilotCohort,
} from '../font_metric_coverage_pilot_selector.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const POLICY = JSON.parse(fs.readFileSync(path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
  'font_metric_coverage_pilot_policy.json',
), 'utf8'));

function candidate(format, index) {
  return {
    source: `/private/${format}/${index}.document`,
    blake3: index.toString(16).padStart(64, '0'),
    extension: format,
    sizeBytes: 1000 + index * 17,
    paragraphCount: 10 + index,
    charCount: 2000 + index * 13,
    compressedCharCount: 500 + index,
    extremeCompressedCharCount: 100 + index * 3,
    kerningCharCount: index % 7 === 0 ? 10 + index : 0,
    fixedFrameCharCount: 1000 + index,
    compressedFixedFrameCharCount: 300 + index * 5,
    singleLineFixedFrameParagraphCount: 2 + index,
    riskScore: 5000 + index * 19,
  };
}

function report(format, reverse = false) {
  const riskDocuments = Array.from({ length: 200 }, (_, index) => candidate(format, index));
  if (reverse) riskDocuments.reverse();
  return {
    schemaVersion: 'poc-font-layout-habits-v2',
    repositoryHead: 'fixture-head',
    riskDocuments,
  };
}

test('selection is deterministic, balanced, disjoint, and quota-reconciled', () => {
  const first = selectPilotCohort(report('hwp'), report('hwpx'), POLICY);
  const second = selectPilotCohort(report('hwp', true), report('hwpx', true), POLICY);
  assert.deepEqual(first, second);
  assert.deepEqual(first.counts, {
    documents: 32,
    canaryDocuments: 8,
    hwp: 16,
    hwpx: 16,
  });
  assert.equal(new Set(first.selections.map(item => `${item.format}:${item.blake3}`)).size, 32);
  for (const format of ['hwp', 'hwpx']) {
    const selected = first.selections.filter(item => item.format === format);
    assert.equal(selected.filter(item => item.tier === 'canary').length, 4);
    assert.equal(selected.filter(item => item.stratum === 'kerning-max').length, 1);
  }
});

test('duplicate content chooses the same path regardless of candidate order', () => {
  const firstHwp = report('hwp');
  const duplicate = { ...firstHwp.riskDocuments[199], source: '/private/hwp/z-duplicate.document' };
  firstHwp.riskDocuments.push(duplicate);
  const secondHwp = {
    ...firstHwp,
    riskDocuments: [...firstHwp.riskDocuments].reverse(),
  };
  assert.deepEqual(
    selectPilotCohort(firstHwp, report('hwpx'), POLICY),
    selectPilotCohort(secondHwp, report('hwpx'), POLICY),
  );
});

test('selection fails closed when a required positive stratum is empty', () => {
  const hwp = report('hwp');
  for (const item of hwp.riskDocuments) item.kerningCharCount = 0;
  assert.throws(
    () => selectPilotCohort(hwp, report('hwpx'), POLICY),
    /insufficient candidates: kerning-max/u,
  );
});

test('private manifest output is confined to the gitignored output root', () => {
  assert.equal(
    assertLocalOutputPath(path.join(ROOT, 'output', 'poc', 'pilot.json')),
    path.join(ROOT, 'output', 'poc', 'pilot.json'),
  );
  assert.throws(() => assertLocalOutputPath(path.join(ROOT, 'mydocs', 'pilot.json')), /output\//u);
  assert.throws(() => assertLocalOutputPath(path.join(ROOT, 'output')), /output\//u);
});
