import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test, { after } from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  buildFontRuleLifecycleAudit,
  validateFontRuleLifecycleAudit,
} from '../font_rule_lifecycle_audit.mjs';
import {
  reduceRegistryV2,
  resolveRuleLifecycle,
} from '../font_rule_registry_v2.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const FIXTURE_ROOT = path.join(ROOT, 'scripts', 'tests', 'fixtures', 'font-rule-registry-v2');
const REGISTRY_PATH = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry_v2.json');
const CLI_PATH = path.join(ROOT, 'scripts', 'font_rule_lifecycle_audit.mjs');
const temporaryDirectories = [];

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function fixtureRegistry(name) {
  const base = readJson(path.join(FIXTURE_ROOT, 'base-registry.json'));
  const scenario = readJson(path.join(FIXTURE_ROOT, `${name}.json`));
  return reduceRegistryV2(base, scenario.changeSets, { root: ROOT });
}

function traceWithReferences(ruleIds) {
  return {
    schemaVersion: 1,
    layoutHash: { algorithm: 'sha256', value: '1'.repeat(64) },
    normalizedHash: { algorithm: 'sha256', value: '2'.repeat(64) },
    records: [{
      recordId: 'record.fixture.0001',
      provenance: ruleIds.slice(0, 1).map(ruleId => ({ ruleId })),
      paint: {
        canvas2d: { ruleIds: ruleIds.slice(1) },
        canvaskit: { ruleIds: [] },
        native: {},
      },
    }],
  };
}

function temporaryDirectory(prefix) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  temporaryDirectories.push(directory);
  return directory;
}

after(() => {
  for (const directory of temporaryDirectories) {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test('resolver explains carried, introduced, retired, replaced and dangling rules', () => {
  const base = readJson(path.join(FIXTURE_ROOT, 'base-registry.json'));
  const introduced = fixtureRegistry('add-rule');
  const retired = fixtureRegistry('retire-rule');
  const replaced = fixtureRegistry('retire-and-replace');

  assert.equal(
    resolveRuleLifecycle(base, 'font-rule.fixture.paint.0001', { root: ROOT }).resolution,
    'carried-forward-active',
  );
  assert.equal(
    resolveRuleLifecycle(introduced, 'font-rule.fixture.paint.0002', { root: ROOT }).resolution,
    'introduced-active',
  );
  assert.equal(
    resolveRuleLifecycle(retired, 'font-rule.fixture.paint.0001', { root: ROOT }).resolution,
    'retired',
  );
  assert.equal(
    resolveRuleLifecycle(replaced, 'font-rule.fixture.paint.0001', { root: ROOT }).resolution,
    'replaced',
  );
  assert.equal(
    resolveRuleLifecycle(base, 'font-rule.fixture.paint.missing', { root: ROOT }).resolution,
    'dangling',
  );
});

test('trace audit joins Rust provenance and Studio backend rule IDs without mutating the trace', () => {
  const registry = fixtureRegistry('retire-and-replace');
  const trace = traceWithReferences([
    'font-rule.fixture.paint.0001',
    'font-rule.fixture.paint.0002',
    'font-rule.fixture.paint.missing',
  ]);
  const before = structuredClone(trace);
  const first = buildFontRuleLifecycleAudit(trace, registry, { root: ROOT });
  const second = buildFontRuleLifecycleAudit(trace, registry, { root: ROOT });

  assert.deepEqual(trace, before);
  assert.deepEqual(first, second);
  assert.deepEqual(first.summary, {
    carriedForwardActive: 0,
    introducedActive: 1,
    historicalReferenceOnly: 0,
    traceSourceDrift: 0,
    retired: 0,
    replaced: 1,
    dangling: 1,
  });
  assert.deepEqual(
    first.references.map(reference => reference.location),
    [
      '/records/0/provenance/0/ruleId',
      '/records/0/paint/canvas2d/ruleIds/0',
      '/records/0/paint/canvas2d/ruleIds/1',
    ],
  );
  assert.deepEqual(validateFontRuleLifecycleAudit(first, trace, registry, { root: ROOT }), []);
  const tampered = structuredClone(first);
  tampered.references[0].resolution = 'dangling';
  assert.match(
    validateFontRuleLifecycleAudit(tampered, trace, registry, { root: ROOT }).join('\n'),
    /differs from the validated registry join/,
  );
});

test('all 830 current registry rules resolve as carried-forward active', () => {
  const registry = readJson(REGISTRY_PATH);
  const trace = traceWithReferences(registry.rules.map(rule => rule.ruleId));
  const audit = buildFontRuleLifecycleAudit(trace, registry, { root: ROOT });

  assert.equal(audit.trace.ruleReferenceCount, 830);
  assert.equal(audit.trace.uniqueRuleIdCount, 830);
  assert.equal(audit.summary.carriedForwardActive, 830);
  assert.equal(audit.references.every(reference => reference.status === 'active'), true);
});

test('the complete W1 rule population closes as lifecycle or historical reference-only', () => {
  const registry = readJson(REGISTRY_PATH);
  const ledger = readJson(path.join(
    ROOT,
    'mydocs',
    'tech',
    'investigations',
    'issue-4939',
    'font_rule_ledger.json',
  ));
  const trace = traceWithReferences(ledger.rules.map(rule => rule.ruleId));
  const audit = buildFontRuleLifecycleAudit(trace, registry, { root: ROOT });

  assert.equal(audit.trace.ruleReferenceCount, 1507);
  assert.equal(audit.summary.carriedForwardActive, 830);
  assert.equal(audit.summary.historicalReferenceOnly, 677);
  assert.equal(audit.summary.dangling, 0);
});

test('trace-declared ledger source drift is distinct from an undeclared dangling rule', () => {
  const registry = readJson(REGISTRY_PATH);
  const trace = traceWithReferences(['rule.rust-metric.current-source-drift']);
  trace.records[0].provenance[0].reason = 'ledgerSourceDrift';
  trace.records[0].paint.canvas2d.ruleIds = ['rule.rust-metric.undeclared-dangling'];
  const audit = buildFontRuleLifecycleAudit(trace, registry, { root: ROOT });

  assert.equal(audit.summary.traceSourceDrift, 1);
  assert.equal(audit.summary.dangling, 1);
  assert.equal(audit.references[0].reason.code, 'traceDeclaredLedgerSourceDrift');
  assert.equal(audit.references[1].reason.code, 'ruleIdNotFound');
});

test('corrupt lifecycle graphs fail closed before a trace is classified', () => {
  const registry = fixtureRegistry('retire-and-replace');
  const trace = traceWithReferences(['font-rule.fixture.paint.0001']);

  const cycle = structuredClone(registry);
  cycle.rules[1].lifecycle.successorRuleIds = [cycle.rules[0].ruleId];
  assert.throws(
    () => buildFontRuleLifecycleAudit(trace, cycle, { root: ROOT }),
    /successor|cycle|active lifecycle/,
  );

  const crossPlane = structuredClone(registry);
  crossPlane.rules[1].decisionPlane = 'layout-name';
  assert.throws(
    () => buildFontRuleLifecycleAudit(trace, crossPlane, { root: ROOT }),
    /selection tuple|projection|cross-plane|decision plane/,
  );

  const danglingEvidence = structuredClone(registry);
  danglingEvidence.evidenceRecords[0].parentEvidenceIds = ['evidence.fixture.missing'];
  assert.throws(
    () => buildFontRuleLifecycleAudit(trace, danglingEvidence, { root: ROOT }),
    /evidence.*dangling|parent/,
  );
});

test('trace input bounds and malformed rule IDs fail closed', () => {
  const registry = readJson(path.join(FIXTURE_ROOT, 'base-registry.json'));
  const tooManyRecords = traceWithReferences([]);
  tooManyRecords.records = Array.from({ length: 4097 }, (_, index) => ({
    recordId: `record.fixture.${index}`,
    provenance: [],
    paint: {},
  }));
  assert.throws(
    () => buildFontRuleLifecycleAudit(tooManyRecords, registry, { root: ROOT }),
    /4,096 records/,
  );

  const malformed = traceWithReferences(['Not a stable rule ID']);
  assert.throws(
    () => buildFontRuleLifecycleAudit(malformed, registry, { root: ROOT }),
    /stable rule ID/,
  );

  const sensitiveRecordId = traceWithReferences([]);
  sensitiveRecordId.records[0].recordId = '/home/user/private/trace.json';
  assert.throws(
    () => buildFontRuleLifecycleAudit(sensitiveRecordId, registry, { root: ROOT }),
    /stable bounded identifier/,
  );
});

test('CLI writes a deterministic audit to stdout and rejects caller-selected output', () => {
  const registry = readJson(REGISTRY_PATH);
  const trace = traceWithReferences([registry.rules[0].ruleId]);
  const directory = temporaryDirectory('rhwp-font-lifecycle-audit-');
  const tracePath = path.join(directory, 'trace.json');
  fs.writeFileSync(tracePath, JSON.stringify(trace));

  const first = spawnSync(process.execPath, [CLI_PATH, '--trace', tracePath], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  const second = spawnSync(process.execPath, [CLI_PATH, '--trace', tracePath], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  assert.equal(first.status, 0, first.stderr);
  assert.equal(first.stdout, second.stdout);
  assert.equal(JSON.parse(first.stdout).summary.carriedForwardActive, 1);
  assert.equal(first.stdout.includes(tracePath), false);

  const rejected = spawnSync(
    process.execPath,
    [CLI_PATH, '--trace', tracePath, '--output', path.join(directory, 'audit.json')],
    { cwd: ROOT, encoding: 'utf8' },
  );
  assert.equal(rejected.status, 1);
  assert.match(rejected.stderr, /usage:.*--trace/);

  const symlinkPath = path.join(directory, 'trace-link.json');
  fs.symlinkSync(tracePath, symlinkPath);
  const symlinkRejected = spawnSync(process.execPath, [CLI_PATH, '--trace', symlinkPath], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  assert.equal(symlinkRejected.status, 1);
  assert.match(symlinkRejected.stderr, /regular non-symlink file/);
});
