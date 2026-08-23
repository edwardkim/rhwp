import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from '../font_rule_ledger.mjs';
import {
  buildMigration,
  buildRegistry,
  compareMigration,
  compareRegistry,
  validateMigration,
  validateRegistry,
} from '../font_rule_registry.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const REGISTRY_PATH = path.join(
  ROOT,
  'assets',
  'font-rules',
  'font_rule_registry.json',
);
const BASELINE_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4966',
  'font_rule_projection_baseline.json',
);
const MIGRATION_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4966',
  'font_rule_registry_migration.json',
);
const GENERATOR_PATH = path.join(ROOT, 'scripts', 'font_rule_registry.mjs');

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function clone(value) {
  return structuredClone(value);
}

function projectionHashes(registry) {
  const rowsByProjection = new Map();
  for (const rule of registry.rules) {
    const projectionId = rule.projections[0].id;
    const rows = rowsByProjection.get(projectionId) ?? [];
    rows.push(rule);
    rowsByProjection.set(projectionId, rows);
  }
  return Object.fromEntries([...rowsByProjection].map(([projectionId, rules]) => [
    projectionId,
    sha256Text(canonicalJson(rules)),
  ]));
}

test('canonical registry and one-time migration are deterministic', () => {
  const expectedRegistry = readJson(REGISTRY_PATH);
  const expectedMigration = readJson(MIGRATION_PATH);
  const baseline = readJson(BASELINE_PATH);
  const actualRegistry = buildRegistry(ROOT, expectedRegistry.sourceCommit);
  const actualMigration = buildMigration(actualRegistry, ROOT, { baseline });

  assert.deepEqual(validateRegistry(expectedRegistry, ROOT), []);
  assert.deepEqual(validateMigration(expectedMigration, expectedRegistry, baseline, ROOT), []);
  assert.deepEqual(compareRegistry(expectedRegistry, actualRegistry), []);
  assert.deepEqual(compareMigration(expectedMigration, actualMigration), []);
});

test('830 rules are assigned to exactly one allowlisted backend projection', () => {
  const registry = readJson(REGISTRY_PATH);

  assert.equal(registry.rules.length, 830);
  assert.equal(new Set(registry.rules.map(rule => rule.ruleId)).size, 830);
  assert.deepEqual(registry.summary.countsByProjection, {
    'canvas2d-paint': 281,
    'canvas2d-webfont': 153,
    'canvaskit-sfnt': 158,
    'rust-layout-metric': 67,
    'rust-layout-name': 171,
  });
  assert.equal(registry.rules.every(rule => rule.projections.length === 1), true);
  assert.deepEqual(validateRegistry(registry, ROOT), []);
});

test('all projected rules preserve W1 candidates and Rust metric rules preserve W6 entries', () => {
  const registry = readJson(REGISTRY_PATH);
  const metricRules = registry.rules.filter(rule => (
    rule.projections[0].id === 'rust-layout-metric'
  ));

  assert.equal(registry.rules.every(rule => rule.evidence.candidateIds.length > 0), true);
  assert.equal(registry.rules.every(rule => rule.evidence.sourceBoundaryIds.length > 0), true);
  assert.equal(new Set(registry.rules.flatMap(rule => rule.evidence.candidateIds)).size, 677);
  assert.equal(metricRules.length, 67);
  assert.equal(metricRules.every(rule => rule.metricEntryIds.length > 0), true);
  assert.equal(registry.summary.metricEntryReferenceCount, 97);
});

test('active unknown metric aliases cannot be deleted or semantically promoted', () => {
  const registry = readJson(REGISTRY_PATH);
  const unknownIndex = registry.rules.findIndex(rule => rule.relationType === 'unknown');

  const deleted = clone(registry);
  deleted.rules.splice(unknownIndex, 1);
  assert.match(validateRegistry(deleted, ROOT).join('\n'), /population|43 active unknown/);

  const promoted = clone(registry);
  promoted.rules[unknownIndex].relationType = 'metric-surrogate';
  promoted.rules[unknownIndex].projections[0].mode = 'direct';
  assert.match(validateRegistry(promoted, ROOT).join('\n'), /43 active unknown/);
});

test('supply rules cannot cross into a layout or metric projection', () => {
  const changed = clone(readJson(REGISTRY_PATH));
  const rule = changed.rules.find(row => row.projections[0].id === 'canvas2d-webfont');
  rule.projections[0].id = 'rust-layout-metric';

  assert.match(
    validateRegistry(changed, ROOT).join('\n'),
    /rust-layout-metric rejects supply\/supply-source/,
  );
});

test('undeclared rule dependencies and host paths are rejected structurally', () => {
  const cyclic = clone(readJson(REGISTRY_PATH));
  cyclic.rules[0].dependsOn = [cyclic.rules[1].ruleId];
  cyclic.rules[1].dependsOn = [cyclic.rules[0].ruleId];
  assert.match(validateRegistry(cyclic, ROOT).join('\n'), /dependsOn is not allowed/);

  const hostPath = clone(readJson(REGISTRY_PATH));
  hostPath.rules[0].targetFaceOrPolicy = '/home/example/private-font.ttf';
  assert.match(validateRegistry(hostPath, ROOT).join('\n'), /host-absolute/);
});

test('generator rejects caller-selected output paths', () => {
  const result = spawnSync(
    process.execPath,
    [GENERATOR_PATH, 'generate', '--registry', '/tmp/registry.json'],
    { cwd: ROOT, encoding: 'utf8' },
  );

  assert.equal(result.status, 1);
  assert.match(result.stderr, /fixed checkout-root output paths/);
});

test('broken W1 and W6 anchors fail closed', () => {
  const brokenW1 = clone(readJson(REGISTRY_PATH));
  brokenW1.rules[0].evidence.candidateIds[0] = 'candidate.missing';
  assert.match(validateRegistry(brokenW1, ROOT).join('\n'), /W1 candidate anchor/);

  const brokenW6 = clone(readJson(REGISTRY_PATH));
  const metricRule = brokenW6.rules.find(rule => (
    rule.projections[0].id === 'rust-layout-metric'
  ));
  metricRule.metricEntryIds[0] = 'font-metric.00000000000000000000';
  assert.match(validateRegistry(brokenW6, ROOT).join('\n'), /W6 metric anchors/);
});

test('duplicate precedence in one decision group is rejected', () => {
  const changed = clone(readJson(REGISTRY_PATH));
  const groups = new Map();
  for (const rule of changed.rules) {
    const key = canonicalJson({
      projectionId: rule.projections[0].id,
      sourceBoundaryIds: rule.evidence.sourceBoundaryIds,
      sourceFace: rule.sourceFace,
      conditions: rule.conditions,
    });
    const rows = groups.get(key) ?? [];
    rows.push(rule);
    groups.set(key, rows);
  }
  const ordered = [...groups.values()].find(rows => (
    rows.length > 1
      && rows.every(rule => Number.isInteger(rule.order))
      && new Set(rows.map(rule => rule.targetFaceOrPolicy)).size > 1
  ));
  assert.ok(ordered);
  ordered[1].order = ordered[0].order;

  assert.match(validateRegistry(changed, ROOT).join('\n'), /unique contiguous orders/);
});

test('CanvasKit preserves capability-versus-plan disagreement without claiming load success', () => {
  const registry = readJson(REGISTRY_PATH);
  const migration = readJson(MIGRATION_PATH);
  const supplies = registry.rules.map(rule => rule.supply).filter(supply => (
    supply?.kind === 'canvaskit-plan'
  ));
  const unavailableButPlanned = supplies.filter(supply => (
    supply.declaredCapability === 'unavailable' && supply.runtimePlanStatus === 'planned'
  ));

  assert.equal(supplies.length, 153);
  assert.equal(unavailableButPlanned.length, 125);
  assert.equal(unavailableButPlanned.every(supply => supply.capabilityAgreement === false), true);
  assert.equal(
    supplies.some(supply => Object.hasOwn(supply, 'typefaceLoaded') || Object.hasOwn(supply, 'success')),
    false,
  );
  assert.equal(migration.summary.canvasKitDeclaredUnavailableRuntimePlannedCount, 125);
  assert.equal(migration.summary.canvasKitDeclaredAvailableRuntimeUnavailableCount, 0);
});

test('one rule mutation changes only its backend projection hash', () => {
  const original = readJson(REGISTRY_PATH);
  const changed = clone(original);
  const changedRule = changed.rules.find(rule => (
    rule.projections[0].id === 'canvas2d-paint'
  ));
  changedRule.targetFaceOrPolicy = `${changedRule.targetFaceOrPolicy} changed`;
  const before = projectionHashes(original);
  const after = projectionHashes(changed);
  const changedProjections = Object.keys(before).filter(name => before[name] !== after[name]);

  assert.deepEqual(changedProjections, ['canvas2d-paint']);
});
