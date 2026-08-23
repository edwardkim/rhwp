import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  buildProjectionBaseline,
  compareProjectionBaseline,
  validateProjectionBaseline,
} from '../font_rule_projection_baseline.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const BASELINE_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4966',
  'font_rule_projection_baseline.json',
);

function readBaseline() {
  return JSON.parse(fs.readFileSync(BASELINE_PATH, 'utf8'));
}

function clone(value) {
  return structuredClone(value);
}

test('W7 pre-migration baseline is deterministic and matches current source', () => {
  const expected = readBaseline();
  const actual = buildProjectionBaseline(ROOT, expected.sourceCommit);

  assert.deepEqual(validateProjectionBaseline(expected), []);
  assert.deepEqual(compareProjectionBaseline(expected, actual), []);
});

test('all 30 W1 boundaries and all 1,352 candidates have an explicit disposition', () => {
  const baseline = readBaseline();

  assert.equal(baseline.inventory.sourceBoundaryCount, 30);
  assert.equal(baseline.inventory.ruleCandidateCount, 1352);
  assert.equal(baseline.inventory.routes.length, 30);
  assert.equal(baseline.inventory.unlinkedCandidateIds.length, 0);
  assert.equal(
    baseline.inventory.routes.every(route => (
      route.classification === 'projection-input' || route.classification === 'reference-only'
    )),
    true,
  );
});

test('five backend projections preserve the frozen pre-migration populations', () => {
  const baseline = readBaseline();

  assert.deepEqual(
    Object.fromEntries(Object.entries(baseline.projections).map(([name, value]) => (
      [name, value.ruleCount]
    ))),
    {
      canvas2dPaint: 281,
      canvasKitSfnt: 158,
      rustLayoutMetric: 67,
      rustLayoutName: 171,
      webfontSupply: 153,
    },
  );
  assert.equal(baseline.metricAnchors.entryCount, 600);
  assert.deepEqual(
    {
      substitution: baseline.studioRuntime.substitution.count,
      governmentSuccessor: baseline.studioRuntime.governmentSuccessor.count,
      displayFallbackProbes: baseline.studioRuntime.displayFallbackProbes.count,
      registeredFonts: baseline.studioRuntime.registeredFonts.count,
      webfontSupply: baseline.studioRuntime.webfontSupply.count,
      webfontLoad: baseline.studioRuntime.webfontLoad.requestCount,
      canvasKitPlans: baseline.studioRuntime.canvasKitPlans.count,
    },
    {
      substitution: 265,
      governmentSuccessor: 65,
      displayFallbackProbes: 8,
      registeredFonts: 153,
      webfontSupply: 153,
      webfontLoad: 153,
      canvasKitPlans: 153,
    },
  );
});

test('active unknown rules remain explicit without cross-plane promotion', () => {
  const baseline = readBaseline();
  const inventory = baseline.inventory.activeUnknown;

  assert.equal(inventory.count, 44);
  assert.equal(inventory.projectedLegacyPreservationCount, 43);
  assert.equal(inventory.handWrittenReferenceCount, 1);
  assert.equal(inventory.rules.every(rule => rule.decisionPlane === 'layout-metric'), true);
  assert.equal(
    baseline.projections.rustLayoutMetric.rules
      .filter(rule => rule.relationType === 'unknown').length,
    43,
  );
  for (const name of ['canvas2dPaint', 'webfontSupply', 'canvasKitSfnt']) {
    assert.equal(
      baseline.projections[name].rules.some(rule => rule.relationType === 'unknown'),
      false,
    );
  }
});

test('projection reordering is rejected by the frozen hash', () => {
  const changed = clone(readBaseline());
  const rules = changed.projections.rustLayoutName.rules;
  [rules[0], rules[1]] = [rules[1], rules[0]];

  assert.match(validateProjectionBaseline(changed).join('\n'), /rustLayoutName: projection hash mismatch/);
});

test('metric anchor reordering and active-unknown deletion fail closed', () => {
  const metricChanged = clone(readBaseline());
  [metricChanged.metricAnchors.entries[0], metricChanged.metricAnchors.entries[1]] = [
    metricChanged.metricAnchors.entries[1],
    metricChanged.metricAnchors.entries[0],
  ];
  assert.match(
    validateProjectionBaseline(metricChanged).join('\n'),
    /metric anchor indexes|metric anchor hash/,
  );

  const unknownChanged = clone(readBaseline());
  unknownChanged.inventory.activeUnknown.rules.pop();
  assert.match(
    validateProjectionBaseline(unknownChanged).join('\n'),
    /active unknown rules/,
  );
});
