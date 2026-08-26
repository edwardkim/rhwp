#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from './font_rule_ledger.mjs';
import {
  projectActiveRules,
  reduceRegistryV2,
  validateRegistryV2,
} from './font_rule_registry_v2.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const FIXTURE_ROOT = path.join(
  ROOT,
  'scripts',
  'tests',
  'fixtures',
  'font-rule-registry-v2',
);
const PRODUCT_REGISTRY_PATH = path.join(
  ROOT,
  'assets',
  'font-rules',
  'font_rule_registry_v2.json',
);
const PROJECTION_IDS = [
  'rust-layout-name',
  'rust-layout-metric',
  'canvas2d-paint',
  'canvas2d-webfont',
  'canvaskit-sfnt',
];
const SCENARIO_NAMES = [
  'evidence-only',
  'add-rule',
  'retire-rule',
  'retire-and-replace',
];

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function registrySha256(registry) {
  return sha256Text(canonicalJson(registry));
}

function activeProjectionRows(registry, projectionId) {
  return projectActiveRules(registry, projectionId).map(rule => ({
    ruleId: rule.ruleId,
    selectionTupleSha256: rule.selectionTupleSha256,
    projectionSequence: rule.projectionSequence,
  }));
}

function projectionSnapshot(registry) {
  return Object.fromEntries(PROJECTION_IDS.map(projectionId => {
    const rows = activeProjectionRows(registry, projectionId);
    return [projectionId, {
      activeRuleCount: rows.length,
      semanticSha256: sha256Text(canonicalJson(rows)),
    }];
  }));
}

function touchedRuleIds(operation) {
  if (operation.type === 'augment-evidence' || operation.type === 'retire-rule') {
    return [operation.ruleId];
  }
  if (operation.type === 'add-rule') return [operation.rule.ruleId];
  if (operation.type === 'retire-and-replace') {
    return [operation.retiredRuleId, operation.replacementRule.ruleId];
  }
  return [];
}

function tupleSnapshot(registry, ruleIds) {
  const byId = new Map(registry.rules.map(rule => [rule.ruleId, rule]));
  return ruleIds.map(ruleId => {
    const rule = byId.get(ruleId);
    return {
      ruleId,
      status: rule?.status ?? 'absent',
      selectionTupleSha256: rule?.selectionTupleSha256 ?? null,
      projectionId: rule?.projections?.[0]?.id ?? null,
      projectionSequence: rule?.projectionSequence ?? null,
      predecessorRuleIds: rule?.lifecycle?.predecessorRuleIds ?? [],
      successorRuleIds: rule?.lifecycle?.successorRuleIds ?? [],
    };
  });
}

function assertScenarioContract(scenario, before, after) {
  if (!SCENARIO_NAMES.includes(scenario?.name)) {
    throw new Error(`unsupported rehearsal scenario: ${scenario?.name ?? '<missing>'}`);
  }
  if (!Array.isArray(scenario.changeSets) || scenario.changeSets.length !== 1) {
    throw new Error(`${scenario.name}: rehearsal requires exactly one change set`);
  }
  const operation = scenario.changeSets[0].operations?.[0];
  if (!operation || scenario.changeSets[0].operations.length !== 1) {
    throw new Error(`${scenario.name}: rehearsal requires exactly one operation`);
  }
  const beforeById = new Map(before.rules.map(rule => [rule.ruleId, rule]));
  const afterById = new Map(after.rules.map(rule => [rule.ruleId, rule]));

  if (scenario.name === 'evidence-only') {
    const previous = beforeById.get(operation.ruleId);
    const current = afterById.get(operation.ruleId);
    if (!previous || !current
        || previous.selectionTupleSha256 !== current.selectionTupleSha256
        || current.lifecycle.lastEvidenceChangeBy !== scenario.changeSets[0].changeSetId) {
      throw new Error(`${scenario.name}: rule identity or selection tuple changed`);
    }
  } else if (scenario.name === 'add-rule') {
    const ruleId = operation.rule.ruleId;
    if (beforeById.has(ruleId) || afterById.get(ruleId)?.status !== 'active') {
      throw new Error(`${scenario.name}: new active rule was not introduced`);
    }
  } else if (scenario.name === 'retire-rule') {
    const previous = beforeById.get(operation.ruleId);
    const current = afterById.get(operation.ruleId);
    if (previous?.status !== 'active' || current?.status !== 'retired'
        || previous.selectionTupleSha256 !== current.selectionTupleSha256
        || current.lifecycle.successorRuleIds.length !== 0) {
      throw new Error(`${scenario.name}: historical row was not preserved as retired`);
    }
  } else {
    const previous = beforeById.get(operation.retiredRuleId);
    const retired = afterById.get(operation.retiredRuleId);
    const replacement = afterById.get(operation.replacementRule.ruleId);
    if (previous?.status !== 'active' || retired?.status !== 'retired'
        || replacement?.status !== 'active'
        || previous.selectionTupleSha256 !== retired.selectionTupleSha256
        || previous.selectionTupleSha256 === replacement.selectionTupleSha256
        || replacement.projectionSequence !== previous.projectionSequence
        || !retired.lifecycle.successorRuleIds.includes(replacement.ruleId)
        || !replacement.lifecycle.predecessorRuleIds.includes(retired.ruleId)) {
      throw new Error(`${scenario.name}: replacement lifecycle or active slot is invalid`);
    }
  }
}

export function rehearseMutationScenario(baseRegistry, scenario, { root = ROOT } = {}) {
  const baseBytes = canonicalJson(baseRegistry);
  const beforeRegistrySha256 = sha256Text(baseBytes);
  const beforeProjection = projectionSnapshot(baseRegistry);
  const changeSet = scenario.changeSets?.[0];
  const targetProjectionId = changeSet?.expectedDelta?.projectionId;
  const ruleIds = [...new Set((changeSet?.operations ?? []).flatMap(touchedRuleIds))];

  const after = reduceRegistryV2(baseRegistry, scenario.changeSets, { root });
  assertScenarioContract(scenario, baseRegistry, after);
  if (canonicalJson(baseRegistry) !== baseBytes) {
    throw new Error(`${scenario.name}: reducer mutated the caller-owned base registry`);
  }

  const afterProjection = projectionSnapshot(after);
  const declaredUnchanged = changeSet.expectedDelta.unchangedProjectionIds;
  const changedProjectionIds = PROJECTION_IDS.filter(projectionId => (
    beforeProjection[projectionId].semanticSha256
      !== afterProjection[projectionId].semanticSha256
  ));
  const activeRuleDelta = afterProjection[targetProjectionId].activeRuleCount
    - beforeProjection[targetProjectionId].activeRuleCount;
  if (activeRuleDelta !== changeSet.expectedDelta.activeRuleDelta) {
    throw new Error(`${scenario.name}: active projection delta differs from declaration`);
  }
  if (declaredUnchanged.some(projectionId => changedProjectionIds.includes(projectionId))) {
    throw new Error(`${scenario.name}: a declared non-target projection changed`);
  }

  const rollback = JSON.parse(baseBytes);
  const rollbackErrors = validateRegistryV2(rollback, root);
  if (rollbackErrors.length > 0 || registrySha256(rollback) !== beforeRegistrySha256) {
    throw new Error(`${scenario.name}: discard rollback did not restore the base registry`);
  }

  return {
    name: scenario.name,
    lifecyclePath: scenario.name,
    targetProjectionId,
    pre: {
      registrySha256: beforeRegistrySha256,
      tuples: tupleSnapshot(baseRegistry, ruleIds),
    },
    post: {
      registrySha256: registrySha256(after),
      tuples: tupleSnapshot(after, ruleIds),
    },
    projectionDelta: {
      activeRuleDelta,
      changedProjectionIds,
      unchangedProjectionIds: declaredUnchanged,
      beforeSemanticSha256: beforeProjection[targetProjectionId].semanticSha256,
      afterSemanticSha256: afterProjection[targetProjectionId].semanticSha256,
    },
    rollback: {
      mode: 'discard-ephemeral-query-model',
      status: 'restored',
      registrySha256: registrySha256(rollback),
    },
  };
}

export function rehearseRejectedMutation(baseRegistry, changeSets, { root = ROOT } = {}) {
  const baseBytes = canonicalJson(baseRegistry);
  let error = null;
  try {
    reduceRegistryV2(baseRegistry, changeSets, { root });
  } catch (caught) {
    error = caught;
  }
  if (!error) throw new Error('negative mutation unexpectedly succeeded');
  if (canonicalJson(baseRegistry) !== baseBytes) {
    throw new Error('rejected mutation changed the caller-owned base registry');
  }
  return {
    status: 'rejected',
    error: error.message,
    rollbackRegistrySha256: sha256Text(baseBytes),
  };
}

export function runFixtureRehearsal({ root = ROOT, fixtureRoot = FIXTURE_ROOT } = {}) {
  const productBytes = fs.readFileSync(path.join(
    root,
    path.relative(ROOT, PRODUCT_REGISTRY_PATH),
  ));
  const productBeforeSha256 = sha256Text(productBytes);
  const base = readJson(path.join(fixtureRoot, 'base-registry.json'));
  const scenarios = SCENARIO_NAMES.map(name => rehearseMutationScenario(
    base,
    readJson(path.join(fixtureRoot, `${name}.json`)),
    { root },
  ));
  const productAfterSha256 = sha256Text(fs.readFileSync(path.join(
    root,
    path.relative(ROOT, PRODUCT_REGISTRY_PATH),
  )));
  if (productAfterSha256 !== productBeforeSha256) {
    throw new Error('rehearsal changed the canonical product registry');
  }
  return {
    schemaVersion: '1.0',
    kind: 'font-rule-mutation-rehearsal',
    summary: {
      scenarioCount: scenarios.length,
      passedCount: scenarios.length,
      rollbackRestoredCount: scenarios.filter(item => item.rollback.status === 'restored').length,
    },
    canonicalProductRegistry: {
      status: 'unchanged',
      sha256: productAfterSha256,
    },
    scenarios,
  };
}

function main() {
  if (process.argv.length !== 2) {
    process.stderr.write('usage: node scripts/font_rule_mutation_rehearsal.mjs\n');
    process.exitCode = 1;
    return;
  }
  process.stdout.write(`${canonicalJson(runFixtureRehearsal())}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
