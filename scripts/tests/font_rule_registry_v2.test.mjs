import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from '../font_rule_ledger.mjs';
import {
  assertSealedV1Artifacts,
  buildInitialRegistryV2,
  buildMigrationV1ToV2,
  projectActiveRules,
  reduceRegistryV2,
  validateChangeSet,
  validateMigrationV1ToV2,
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
const V2_REGISTRY_PATH = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry_v2.json');
const MIGRATION_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-5955',
  'font_rule_registry_v1_to_v2_migration.json',
);
const GENERATOR_PATH = path.join(ROOT, 'scripts', 'font_rule_registry_v2.mjs');

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
  const v1Registry = readJson(V1_REGISTRY_PATH);
  const v2Registry = buildInitialRegistryV2(v1Registry, ROOT);
  const migration = buildMigrationV1ToV2(v1Registry, { root: ROOT, v2Registry });

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
  assert.equal(v2Registry.rules.every((rule, index) => (
    rule.sourceBoundaryId === v1Registry.rules[index].evidence.sourceBoundaryIds[0]
      && v1Registry.rules[index].evidence.sourceBoundaryIds.length === 1
  )), true);
});

test('canonical v2 registry and migration are deterministic query models', () => {
  const v1Registry = readJson(V1_REGISTRY_PATH);
  const expectedRegistry = readJson(V2_REGISTRY_PATH);
  const expectedMigration = readJson(MIGRATION_PATH);
  const actualRegistry = buildInitialRegistryV2(v1Registry, ROOT, {
    sourceCommit: expectedRegistry.sourceCommit,
  });
  const actualMigration = buildMigrationV1ToV2(v1Registry, {
    root: ROOT,
    v2Registry: actualRegistry,
  });

  assert.deepEqual(actualRegistry, expectedRegistry);
  assert.deepEqual(actualMigration, expectedMigration);
  assert.deepEqual(validateRegistryV2(expectedRegistry, ROOT), []);
  assert.deepEqual(
    validateMigrationV1ToV2(expectedMigration, v1Registry, expectedRegistry, ROOT),
    [],
  );
});

test('initial migration refuses a caller-mutated v1 authority', () => {
  const changed = clone(readJson(V1_REGISTRY_PATH));
  changed.rules[0].targetFaceOrPolicy = 'Caller-mutated selection';

  assert.throws(
    () => buildInitialRegistryV2(changed, ROOT),
    /requires the sealed v1 registry bytes/,
  );
});

test('v2 generator rejects caller-selected output paths', () => {
  const result = spawnSync(
    process.execPath,
    [GENERATOR_PATH, 'generate', '--registry', '/tmp/font-rule-registry-v2.json'],
    { cwd: ROOT, encoding: 'utf8' },
  );

  assert.equal(result.status, 1);
  assert.match(result.stderr, /usage:.*<generate\|check>/);
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

test('v2 change sets preserve the sealed v1 projection semantic boundaries', () => {
  const mixedRelation = clone(fixture('add-rule').changeSets[0]);
  mixedRelation.operations[0].rule.relationType = 'supply-source';
  assert.match(
    validateChangeSet(mixedRelation, { root: ROOT }).join('\n'),
    /mixes supply-source with canvas2d-paint/,
  );

  const mixedMetric = clone(fixture('add-rule').changeSets[0]);
  mixedMetric.operations[0].rule.metricEntryIds = ['metric.fixture.invalid-paint-anchor'];
  assert.match(
    validateChangeSet(mixedMetric, { root: ROOT }).join('\n'),
    /only rust-layout-metric may reference metric entries/,
  );

  const mixedSupply = clone(fixture('add-rule').changeSets[0]);
  mixedSupply.operations[0].rule.supply = {
    kind: 'canvas2d-webfont',
    fontFamily: 'Unrelated Family',
    sourceUrl: 'file:///home/private/font.ttf',
    format: 'truetype',
    unicodeRange: null,
    external: false,
  };
  assert.match(
    validateChangeSet(mixedSupply, { root: ROOT }).join('\n'),
    /host-absolute|file URL|non-supply projection/,
  );

  const absolutePayload = clone(fixture('add-rule').changeSets[0]);
  absolutePayload.operations[0].rule.targetFaceOrPolicy = '/tmp/private/font.ttf';
  assert.match(
    validateChangeSet(absolutePayload, { root: ROOT }).join('\n'),
    /host-absolute or file URL paths/,
  );
});

test('new supply rules enforce family URL and capability agreement', () => {
  const webfont = clone(fixture('add-rule').changeSets[0]);
  webfont.decisionPlane = 'supply';
  webfont.expectedDelta.projectionId = 'canvas2d-webfont';
  webfont.expectedDelta.unchangedProjectionIds = [
    'rust-layout-name',
    'rust-layout-metric',
    'canvas2d-paint',
    'canvaskit-sfnt',
  ];
  Object.assign(webfont.operations[0].rule, {
    relationType: 'supply-source',
    decisionPlane: 'supply',
    projection: { id: 'canvas2d-webfont', mode: 'direct' },
    supply: {
      kind: 'canvas2d-webfont',
      fontFamily: 'Unrelated Family',
      sourceUrl: 'fonts/../private/font.woff2',
      format: 'woff2',
      unicodeRange: null,
      external: false,
    },
  });
  assert.match(
    validateChangeSet(webfont, { root: ROOT }).join('\n'),
    /Canvas2D family\/URL\/external boundary/,
  );

  const canvasKit = clone(webfont);
  canvasKit.expectedDelta.projectionId = 'canvaskit-sfnt';
  canvasKit.expectedDelta.unchangedProjectionIds = [
    'rust-layout-name',
    'rust-layout-metric',
    'canvas2d-paint',
    'canvas2d-webfont',
  ];
  Object.assign(canvasKit.operations[0].rule, {
    conditions: { profile: 'canvaskit-sfnt' },
    projection: { id: 'canvaskit-sfnt', mode: 'direct' },
    supply: {
      kind: 'canvaskit-plan',
      fontFamily: 'Fixture Serif',
      declaredCapability: 'sfnt-source',
      runtimePlanStatus: 'unavailable',
      capabilityAgreement: true,
      online: { sources: [], unavailableFonts: [] },
      offline: { sources: [], unavailableFonts: [] },
    },
  });
  assert.match(
    validateChangeSet(canvasKit, { root: ROOT }).join('\n'),
    /CanvasKit family\/capability agreement/,
  );
});

test('change sets cannot introduce a new unknown legacy-preservation rule', () => {
  const canonical = readJson(V2_REGISTRY_PATH);
  const sealedUnknown = canonical.rules.find(rule => (
    rule.relationType === 'unknown' && rule.projections[0].id === 'rust-layout-metric'
  ));
  const changed = clone(fixture('add-rule').changeSets[0]);
  changed.decisionPlane = 'layout-metric';
  changed.expectedDelta.projectionId = 'rust-layout-metric';
  changed.expectedDelta.unchangedProjectionIds = [
    'rust-layout-name',
    'canvas2d-paint',
    'canvas2d-webfont',
    'canvaskit-sfnt',
  ];
  changed.operations[0].rule = {
    ruleId: 'font-rule.fixture.metric.unknown-new',
    relationType: 'unknown',
    decisionPlane: 'layout-metric',
    sourceBoundaryId: sealedUnknown.sourceBoundaryId,
    sourceFace: sealedUnknown.sourceFace,
    targetFaceOrPolicy: sealedUnknown.targetFaceOrPolicy,
    conditions: clone(sealedUnknown.conditions),
    order: sealedUnknown.order,
    projection: clone(sealedUnknown.projections[0]),
    projectionSequence: 67,
    metricEntryIds: clone(sealedUnknown.metricEntryIds),
    supply: null,
    evidenceIds: ['evidence.fixture.add-rule'],
  };

  assert.match(
    validateChangeSet(changed, { root: ROOT }).join('\n'),
    /cannot introduce a new unknown legacy rule/,
  );
});

test('a command cannot relabel the decision plane of an existing rule', () => {
  const base = readJson(path.join(FIXTURE_ROOT, 'base-registry.json'));
  const changed = clone(fixture('evidence-only').changeSets[0]);
  changed.decisionPlane = 'supply';
  changed.expectedDelta.projectionId = 'canvas2d-webfont';
  changed.expectedDelta.unchangedProjectionIds = [
    'rust-layout-name',
    'rust-layout-metric',
    'canvas2d-paint',
    'canvaskit-sfnt',
  ];

  assert.throws(
    () => reduceRegistryV2(base, [changed], { root: ROOT }),
    /current rule crosses the declared decision plane/,
  );
});

test('registry validator rejects a successor in another projection of the same decision plane', () => {
  const changed = clone(readJson(V2_REGISTRY_PATH));
  const retired = changed.rules
    .filter(rule => (
      rule.status === 'active' && rule.projections[0].id === 'canvas2d-webfont'
    ))
    .sort((left, right) => left.projectionSequence - right.projectionSequence)
    .at(-1);
  const successor = changed.rules.find(rule => (
    rule.status === 'active'
      && rule.decisionPlane === retired.decisionPlane
      && rule.projections[0].id === 'canvaskit-sfnt'
  ));

  retired.status = 'retired';
  retired.lifecycle.retiredBy = 'change.fixture.cross-projection';
  retired.lifecycle.retirementReason = 'Synthetic cross-projection successor';
  retired.lifecycle.successorRuleIds = [successor.ruleId];
  successor.lifecycle.predecessorRuleIds = [retired.ruleId];

  const active = changed.rules.filter(rule => rule.status === 'active');
  const countsByProjection = {};
  for (const rule of active) {
    const projectionId = rule.projections[0].id;
    countsByProjection[projectionId] = (countsByProjection[projectionId] ?? 0) + 1;
  }
  changed.summary = {
    ruleCount: changed.rules.length,
    activeRuleCount: active.length,
    retiredRuleCount: changed.rules.length - active.length,
    countsByProjection: Object.fromEntries(Object.entries(countsByProjection).sort()),
  };
  changed.rulesSha256 = sha256Text(canonicalJson(changed.rules));

  assert.match(validateRegistryV2(changed, ROOT).join('\n'), /cross-projection successor/);
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

test('manual validators reject malformed nested values without throwing', () => {
  const malformedRegistry = clone(readJson(path.join(FIXTURE_ROOT, 'base-registry.json')));
  malformedRegistry.rules[0] = null;
  let registryErrors;
  assert.doesNotThrow(() => {
    registryErrors = validateRegistryV2(malformedRegistry, ROOT);
  });
  assert.match(registryErrors.join('\n'), /registry\.rules\[0\] must be an object/);

  const malformedRecords = clone(readJson(path.join(FIXTURE_ROOT, 'base-registry.json')));
  malformedRecords.evidenceRecords = {};
  assert.doesNotThrow(() => {
    registryErrors = validateRegistryV2(malformedRecords, ROOT);
  });
  assert.match(registryErrors.join('\n'), /evidenceRecords/);

  const malformedProjection = clone(readJson(path.join(FIXTURE_ROOT, 'base-registry.json')));
  malformedProjection.rules[0].projections = null;
  assert.doesNotThrow(() => {
    registryErrors = validateRegistryV2(malformedProjection, ROOT);
  });
  assert.match(registryErrors.join('\n'), /projections must contain exactly one projection/);

  for (const [name, mutate] of [
    ['evidence parent IDs', registry => { registry.evidenceRecords[0].parentEvidenceIds = {}; }],
    ['rule evidence IDs', registry => { registry.rules[0].evidenceIds = {}; }],
    ['successor rule IDs', registry => { registry.rules[0].lifecycle.successorRuleIds = {}; }],
    ['predecessor rule IDs', registry => { registry.rules[0].lifecycle.predecessorRuleIds = {}; }],
  ]) {
    const malformedCollection = clone(readJson(path.join(FIXTURE_ROOT, 'base-registry.json')));
    mutate(malformedCollection);
    let malformedCollectionErrors;
    assert.doesNotThrow(
      () => { malformedCollectionErrors = validateRegistryV2(malformedCollection, ROOT); },
      `${name} must fail closed without throwing`,
    );
    assert.ok(malformedCollectionErrors.length > 0, `${name} must return validation errors`);
  }

  const malformedChangeSet = clone(fixture('add-rule').changeSets[0]);
  malformedChangeSet.operations[0] = null;
  let changeSetErrors;
  assert.doesNotThrow(() => {
    changeSetErrors = validateChangeSet(malformedChangeSet, { root: ROOT });
  });
  assert.match(changeSetErrors.join('\n'), /unknown operation/);

  const malformedChangeSetRecords = clone(fixture('add-rule').changeSets[0]);
  malformedChangeSetRecords.evidenceRecords = {};
  assert.doesNotThrow(() => {
    changeSetErrors = validateChangeSet(malformedChangeSetRecords, { root: ROOT });
  });
  assert.match(changeSetErrors.join('\n'), /evidenceRecords/);

  for (const [name, mutate] of [
    ['evidence parent IDs', changeSet => { changeSet.evidenceRecords[0].parentEvidenceIds = {}; }],
    ['rule evidence IDs', changeSet => { changeSet.operations[0].rule.evidenceIds = {}; }],
  ]) {
    const malformedCollection = clone(fixture('add-rule').changeSets[0]);
    mutate(malformedCollection);
    let malformedCollectionErrors;
    assert.doesNotThrow(
      () => {
        malformedCollectionErrors = validateChangeSet(malformedCollection, { root: ROOT });
      },
      `${name} must fail closed without throwing`,
    );
    assert.ok(malformedCollectionErrors.length > 0, `${name} must return validation errors`);
  }
});

test('new and replacement rules cannot cite undeclared evidence', () => {
  const add = clone(fixture('add-rule').changeSets[0]);
  add.operations[0].rule.evidenceIds = ['evidence.fixture.missing'];
  assert.match(validateChangeSet(add, { root: ROOT }).join('\n'), /dangling evidence reference/);

  const replace = clone(fixture('retire-and-replace').changeSets[0]);
  replace.operations[0].replacementRule.evidenceIds = ['evidence.fixture.missing'];
  assert.match(validateChangeSet(replace, { root: ROOT }).join('\n'), /dangling evidence reference/);
});

test('source boundary is required and protected by the immutable selection tuple', () => {
  const add = clone(fixture('add-rule').changeSets[0]);
  delete add.operations[0].rule.sourceBoundaryId;
  assert.match(validateChangeSet(add, { root: ROOT }).join('\n'), /sourceBoundaryId is required/);

  const changedRegistry = clone(readJson(path.join(FIXTURE_ROOT, 'base-registry.json')));
  changedRegistry.rules[0].sourceBoundaryId = 'boundary.fixture.changed';
  assert.match(
    validateRegistryV2(changedRegistry, ROOT).join('\n'),
    /selectionTupleSha256|differs from sealed legacy evidence/,
  );
});
