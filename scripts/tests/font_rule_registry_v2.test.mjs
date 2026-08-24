import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  assertSealedV1Artifacts,
  buildMigrationV1ToV2,
  projectActiveRules,
  reduceRegistryV2,
  validateChangeSet,
  validateRegistryV2,
} from '../font_rule_registry_v2.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const FIXTURE_ROOT = path.join(
  ROOT,
  'scripts',
  'tests',
  'fixtures',
  'font-rule-registry-v2',
);
const V1_REGISTRY_PATH = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry.json');

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function fixture(name) {
  return readJson(path.join(FIXTURE_ROOT, `${name}.json`));
}

function clone(value) {
  return structuredClone(value);
}

function applyFixture(name) {
  const base = readJson(path.join(FIXTURE_ROOT, 'base-registry.json'));
  const scenario = fixture(name);
  return { scenario, actual: reduceRegistryV2(base, scenario.changeSets, { root: ROOT }) };
}

test('sealed v1 registry artifacts retain their approved SHA-256 digests', () => {
  assert.deepEqual(assertSealedV1Artifacts(ROOT), []);
});

test('v2 schema vocabulary fixes the approved security bounds', () => {
  const registrySchema = readJson(path.join(
    ROOT,
    'assets',
    'font-rules',
    'font_rule_registry_v2.schema.json',
  ));
  const changeSetSchema = readJson(path.join(
    ROOT,
    'assets',
    'font-rules',
    'font_rule_change_set.schema.json',
  ));

  assert.equal(registrySchema.properties.rules.maxItems, 4096);
  assert.equal(changeSetSchema.properties.operations.maxItems, 64);
  assert.equal(changeSetSchema.properties.evidenceRecords.maxItems, 128);
  assert.equal(registrySchema.$defs.nonEmptyString.maxLength, 2048);
  assert.equal(registrySchema.$defs.rule.properties.metricEntryIds.maxItems, 600);
});

test('positive fixtures cover the five approved lifecycle outcomes', () => {
  assert.deepEqual(
    ['carry-forward', 'evidence-only', 'add-rule', 'retire-rule', 'retire-and-replace']
      .map(name => fixture(name).name),
    ['carry-forward', 'evidence-only', 'add-rule', 'retire-rule', 'retire-and-replace'],
  );
});

for (const name of [
  'carry-forward',
  'evidence-only',
  'add-rule',
  'retire-rule',
  'retire-and-replace',
]) {
  test(`${name} fixture reduces to its declared lifecycle result`, () => {
    const { scenario, actual } = applyFixture(name);
    const activeRuleIds = actual.rules
      .filter(rule => rule.status === 'active')
      .map(rule => rule.ruleId);
    const retiredRuleIds = actual.rules
      .filter(rule => rule.status === 'retired')
      .map(rule => rule.ruleId);

    assert.deepEqual(activeRuleIds, scenario.expected.activeRuleIds);
    assert.deepEqual(retiredRuleIds, scenario.expected.retiredRuleIds);
    assert.deepEqual(validateRegistryV2(actual, ROOT), []);
  });
}

test('initial v1 to v2 migration carries all 830 rules without semantic delta', () => {
  const migration = buildMigrationV1ToV2(readJson(V1_REGISTRY_PATH), { root: ROOT });

  assert.equal(migration.summary.v1RuleCount, 830);
  assert.equal(migration.summary.v2ActiveRuleCount, 830);
  assert.equal(migration.summary.v2RetiredRuleCount, 0);
  assert.equal(migration.summary.carryForwardCount, 830);
  assert.equal(
    migration.mappings.every(mapping => (
      mapping.disposition === 'carry-forward'
        && mapping.v1RuleId === mapping.v2RuleId
        && mapping.beforeSelectionTupleSha256 === mapping.afterSelectionTupleSha256
    )),
    true,
  );
  assert.equal(migration.projectionDeltas.every(delta => delta.status === 'unchanged'), true);
});

test('in-place semantic mutation is rejected instead of reusing a ruleId', () => {
  const changed = clone(fixture('evidence-only').changeSets[0]);
  changed.operations[0] = {
    operationId: 'operation.fixture.in-place-mutation',
    type: 'update-rule',
    ruleId: 'font-rule.fixture.paint.0001',
    targetFaceOrPolicy: 'A different semantic selection',
  };

  assert.match(validateChangeSet(changed, { root: ROOT }).join('\n'), /in-place semantic mutation/);
});

test('stale parent registry digest is rejected fail-closed', () => {
  const changed = clone(fixture('add-rule').changeSets[0]);
  changed.parentRegistrySha256 = '9999999999999999999999999999999999999999999999999999999999999999';

  assert.match(
    validateChangeSet(changed, {
      root: ROOT,
      expectedParentRegistrySha256:
        'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    }).join('\n'),
    /stale parent registry/,
  );
});

test('one change set cannot cross a decision plane', () => {
  const changed = clone(fixture('add-rule').changeSets[0]);
  changed.decisionPlane = 'layout-name';

  assert.match(validateChangeSet(changed, { root: ROOT }).join('\n'), /cross-plane/);
});

test('evidence cycles and self-parent edges are rejected', () => {
  const changed = clone(fixture('evidence-only').changeSets[0]);
  changed.evidenceRecords[0].parentEvidenceIds = [changed.evidenceRecords[0].evidenceId];

  assert.match(validateChangeSet(changed, { root: ROOT }).join('\n'), /evidence cycle|self-parent/);
});

test('retired rules are preserved in the registry but excluded from runtime projection', () => {
  const changed = clone(readJson(path.join(FIXTURE_ROOT, 'base-registry.json')));
  changed.rules[0].status = 'retired';
  changed.rules[0].lifecycle.retiredBy = 'issue-5955.fixture.retire-rule';
  changed.rules[0].lifecycle.retirementReason = 'Synthetic negative fixture';

  assert.deepEqual(projectActiveRules(changed, 'canvas2d-paint'), []);
});

test('unsafe evidence paths and bounded collections fail closed', () => {
  const unsafe = clone(fixture('add-rule').changeSets[0]);
  unsafe.evidenceRecords[0].source = '../private/corpus/font.bin';
  assert.match(validateChangeSet(unsafe, { root: ROOT }).join('\n'), /unsafe.*path|path traversal/);

  const oversized = clone(fixture('add-rule').changeSets[0]);
  oversized.operations = Array.from({ length: 65 }, (_, index) => ({
    ...clone(oversized.operations[0]),
    operationId: `operation.fixture.oversized-${index}`,
  }));
  assert.match(validateChangeSet(oversized, { root: ROOT }).join('\n'), /at most 64 operations/);
});
