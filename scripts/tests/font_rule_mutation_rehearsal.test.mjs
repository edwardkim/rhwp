import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from '../font_rule_ledger.mjs';
import {
  rehearseMutationScenario,
  rehearseRejectedMutation,
  runFixtureRehearsal,
} from '../font_rule_mutation_rehearsal.mjs';
import { reduceRegistryV2 } from '../font_rule_registry_v2.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const FIXTURE_ROOT = path.join(
  ROOT,
  'scripts',
  'tests',
  'fixtures',
  'font-rule-registry-v2',
);
const SCRIPT_PATH = path.join(ROOT, 'scripts', 'font_rule_mutation_rehearsal.mjs');
const SCENARIO_NAMES = [
  'evidence-only',
  'add-rule',
  'retire-rule',
  'retire-and-replace',
];

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function fixture(name) {
  return readJson(path.join(FIXTURE_ROOT, `${name}.json`));
}

function clone(value) {
  return structuredClone(value);
}

test('four future W8 lifecycle paths expose tuple, projection, and rollback evidence', () => {
  const base = fixture('base-registry');
  const results = SCENARIO_NAMES.map(name => rehearseMutationScenario(
    base,
    fixture(name),
    { root: ROOT },
  ));

  assert.deepEqual(results.map(result => result.lifecyclePath), SCENARIO_NAMES);
  assert.deepEqual(results.map(result => result.projectionDelta.activeRuleDelta), [0, 1, -1, 0]);
  assert.equal(results.every(result => result.targetProjectionId === 'canvas2d-paint'), true);
  assert.equal(results.every(result => result.projectionDelta.unchangedProjectionIds.length === 4), true);
  assert.equal(results.every(result => result.rollback.status === 'restored'), true);
  assert.equal(results.every(result => (
    result.rollback.registrySha256 === result.pre.registrySha256
  )), true);

  const evidence = results[0];
  assert.deepEqual(evidence.pre.tuples, evidence.post.tuples.map(tuple => ({
    ...tuple,
    predecessorRuleIds: [],
    successorRuleIds: [],
  })));
  const replacement = results[3];
  assert.equal(replacement.pre.tuples[0].selectionTupleSha256,
    replacement.post.tuples[0].selectionTupleSha256);
  assert.notEqual(replacement.post.tuples[0].selectionTupleSha256,
    replacement.post.tuples[1].selectionTupleSha256);
  assert.equal(replacement.post.tuples[0].successorRuleIds[0], replacement.post.tuples[1].ruleId);
  assert.equal(replacement.post.tuples[1].predecessorRuleIds[0], replacement.post.tuples[0].ruleId);
});

test('the fixture rehearsal is deterministic and leaves the product registry unchanged', () => {
  const productPath = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry_v2.json');
  const before = fs.readFileSync(productPath);
  const first = runFixtureRehearsal({ root: ROOT, fixtureRoot: FIXTURE_ROOT });
  const second = runFixtureRehearsal({ root: ROOT, fixtureRoot: FIXTURE_ROOT });

  assert.deepEqual(first, second);
  assert.equal(first.summary.scenarioCount, 4);
  assert.equal(first.summary.passedCount, 4);
  assert.equal(first.summary.rollbackRestoredCount, 4);
  assert.equal(first.canonicalProductRegistry.status, 'unchanged');
  assert.equal(first.canonicalProductRegistry.sha256, sha256Text(before));
  assert.deepEqual(fs.readFileSync(productPath), before);
});

test('negative mutations fail closed and preserve their caller-owned registry', () => {
  const base = fixture('base-registry');
  const cases = [];

  const inPlace = clone(fixture('evidence-only').changeSets[0]);
  inPlace.operations[0] = {
    operationId: 'operation.fixture.in-place-mutation',
    type: 'update-rule',
    ruleId: 'font-rule.fixture.paint.0001',
  };
  cases.push(['in-place semantic mutation', [inPlace], /in-place semantic mutation/]);

  const staleParent = clone(fixture('add-rule').changeSets[0]);
  staleParent.parentRegistrySha256 = '9'.repeat(64);
  cases.push(['stale parent', [staleParent], /stale parent/]);

  const crossPlane = clone(fixture('add-rule').changeSets[0]);
  crossPlane.decisionPlane = 'layout-name';
  cases.push(['cross-plane', [crossPlane], /cross-plane/]);

  const evidenceCycle = clone(fixture('evidence-only').changeSets[0]);
  evidenceCycle.evidenceRecords[0].parentEvidenceIds = [
    evidenceCycle.evidenceRecords[0].evidenceId,
  ];
  cases.push(['evidence cycle', [evidenceCycle], /evidence cycle|self-parent/]);

  const unsafePath = clone(fixture('add-rule').changeSets[0]);
  unsafePath.evidenceRecords[0].source = '../private/font.bin';
  cases.push(['unsafe evidence path', [unsafePath], /unsafe.*path|path traversal/]);

  const wrongSlot = clone(fixture('retire-and-replace').changeSets[0]);
  wrongSlot.operations[0].replacementRule.projectionSequence = 1;
  cases.push(['replacement slot', [wrongSlot], /inherit the active projection slot/]);

  const wrongDelta = clone(fixture('add-rule').changeSets[0]);
  wrongDelta.expectedDelta.activeRuleDelta = 0;
  cases.push(['declared delta', [wrongDelta], /active projection delta differs/]);

  const duplicateRule = clone(fixture('add-rule').changeSets[0]);
  duplicateRule.operations[0].rule.ruleId = 'font-rule.fixture.paint.0001';
  cases.push(['duplicate rule ID', [duplicateRule], /already used/]);

  for (const [name, changeSets, errorPattern] of cases) {
    const baseBytes = canonicalJson(base);
    const result = rehearseRejectedMutation(base, changeSets, { root: ROOT });
    assert.equal(result.status, 'rejected', name);
    assert.match(result.error, errorPattern, name);
    assert.equal(result.rollbackRegistrySha256, sha256Text(baseBytes), name);
    assert.equal(canonicalJson(base), baseBytes, name);
  }
});

test('a non-tail retirement is rejected after an ephemeral add and both inputs remain intact', () => {
  const base = fixture('base-registry');
  const add = clone(fixture('add-rule').changeSets[0]);
  const afterAdd = reduceRegistryV2(base, [add], { root: ROOT });
  const retire = clone(fixture('retire-rule').changeSets[0]);
  retire.changeSetId = 'issue-5955.fixture.non-tail-retirement';
  retire.sequence = 2;
  retire.parentRegistrySha256 = sha256Text(canonicalJson(afterAdd));
  retire.evidenceRecords[0].evidenceId = 'evidence.fixture.non-tail-retirement';
  retire.operations[0].operationId = 'operation.fixture.non-tail-retirement';
  retire.operations[0].evidenceIds = ['evidence.fixture.non-tail-retirement'];
  const snapshot = canonicalJson(afterAdd);

  const result = rehearseRejectedMutation(afterAdd, [retire], { root: ROOT });
  assert.match(result.error, /non-tail retirement/);
  assert.equal(canonicalJson(afterAdd), snapshot);
  assert.equal(canonicalJson(base), canonicalJson(fixture('base-registry')));
});

test('CLI emits only the deterministic rehearsal envelope and rejects arguments', () => {
  const first = spawnSync(process.execPath, [SCRIPT_PATH], { cwd: ROOT, encoding: 'utf8' });
  const second = spawnSync(process.execPath, [SCRIPT_PATH], { cwd: ROOT, encoding: 'utf8' });
  assert.equal(first.status, 0, first.stderr);
  assert.equal(first.stderr, '');
  assert.equal(first.stdout, second.stdout);
  assert.equal(JSON.parse(first.stdout).kind, 'font-rule-mutation-rehearsal');

  const rejected = spawnSync(process.execPath, [SCRIPT_PATH, '--output', '/tmp/result.json'], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  assert.equal(rejected.status, 1);
  assert.match(rejected.stderr, /usage:/);
  assert.equal(rejected.stdout, '');
});
