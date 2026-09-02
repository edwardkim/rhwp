import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import test, { after } from 'node:test';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from '../font_rule_ledger.mjs';
import { selectionTupleSha256 } from '../font_rule_registry_v2.mjs';
import {
  GENERATED_SENTINEL,
  OUTPUT_CONFIGS,
  buildProjectionBundle,
  compareProjectionBundle,
  validateProjectionManifest,
  writeProjectionBundle,
} from '../font_rule_projection_gen.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const V1_REGISTRY_PATH = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry.json');
const REGISTRY_PATH = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry_v2.json');
const GENERATOR_PATH = path.join(ROOT, 'scripts', 'font_rule_projection_gen.mjs');
const temporaryDirectories = [];
const SEALED_PROJECTION_SHA256 = Object.freeze({
  'canvas2d-paint': 'c959e68087f6928edcafc74a1d3f9cd3885dd7540faf22b7663a49b6ad8835e4',
  'canvas2d-webfont': '730cab042d68ffb019d5867102ee8b2b8e5be41c48170ca5fc75422005e3fbee',
  'canvaskit-sfnt': 'd9019fc756d4fd9334252704309bb2020c251d6a7d04dc0f5a6b2efb0f017668',
  'rust-layout-metric': 'c4659fc40246c5d4ad903578a61807c646681638cb4c8f9b7c802fb3f0c37cc2',
  'rust-layout-name': '595cdcc1c8d81441c9e4585acb393e734f52e6da3e822babf0f722df2c791cee',
});

function readRegistry() {
  return JSON.parse(fs.readFileSync(REGISTRY_PATH, 'utf8'));
}

function temporaryRoot(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  temporaryDirectories.push(root);
  return root;
}

function refreshRegistryHash(registry) {
  for (const rule of registry.rules) rule.selectionTupleSha256 = selectionTupleSha256(rule);
  const active = registry.rules.filter(rule => rule.status === 'active');
  registry.summary = {
    ruleCount: registry.rules.length,
    activeRuleCount: active.length,
    retiredRuleCount: registry.rules.length - active.length,
    countsByProjection: Object.fromEntries(OUTPUT_CONFIGS.map(config => [
      config.projectionId,
      active.filter(rule => rule.projections[0].id === config.projectionId).length,
    ]).sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0))),
  };
  registry.rulesSha256 = sha256Text(canonicalJson(registry.rules));
  return registry;
}

function outputHashes(bundle) {
  return Object.fromEntries(bundle.outputs.map(output => [
    output.projectionId,
    output.contentSha256,
  ]));
}

after(() => {
  for (const directory of temporaryDirectories) {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test('five backend projections are deterministic and close all 830 registry rules', () => {
  const registry = readRegistry();
  const first = buildProjectionBundle(registry);
  const second = buildProjectionBundle(registry);

  assert.deepEqual(first, second);
  assert.deepEqual(validateProjectionManifest(
    first.manifest,
    first.outputs,
    registry,
  ), []);
  assert.equal(first.manifest.summary.outputCount, 5);
  assert.equal(first.manifest.summary.activeRuleCount, 830);
  assert.equal(first.manifest.summary.retiredRuleCount, 0);
  assert.deepEqual(first.manifest.summary.countsByProjection, {
    'canvas2d-paint': 281,
    'canvas2d-webfont': 153,
    'canvaskit-sfnt': 158,
    'rust-layout-metric': 67,
    'rust-layout-name': 171,
  });
  assert.deepEqual(
    Object.fromEntries(first.outputs.map(output => [
      output.projectionId,
      output.projectionSha256,
    ]).sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0))),
    SEALED_PROJECTION_SHA256,
  );
});

test('sealed v1 cannot be used as the current projection authority', () => {
  const v1 = JSON.parse(fs.readFileSync(V1_REGISTRY_PATH, 'utf8'));
  assert.throws(() => buildProjectionBundle(v1), /v2 registry envelope is invalid/);
});

test('output allowlist, language and ruleId order match the canonical registry', () => {
  const registry = readRegistry();
  const bundle = buildProjectionBundle(registry);

  assert.deepEqual(
    bundle.outputs.map(output => ({
      projectionId: output.projectionId,
      language: output.language,
      path: output.path,
    })),
    OUTPUT_CONFIGS.map(({ projectionId, language, path: outputPath }) => ({
      projectionId,
      language,
      path: outputPath,
    })),
  );
  for (const output of bundle.outputs) {
    assert.deepEqual(
      output.rows.map(row => row.ruleId),
      registry.rules
        .filter(rule => (
          rule.status === 'active' && rule.projections[0].id === output.projectionId
        ))
        .sort((left, right) => left.projectionSequence - right.projectionSequence)
        .map(rule => rule.ruleId),
    );
    assert.equal(output.content.startsWith(`${GENERATED_SENTINEL}\n`), true);
  }
});

test('all projections retain one source boundary and Rust emits allocation-free match lookups', () => {
  const registry = readRegistry();
  const bundle = buildProjectionBundle(registry);
  const rustOutputs = bundle.outputs.filter(output => output.language === 'rust');

  for (const output of bundle.outputs) {
    const inputRules = registry.rules.filter(rule => (
      rule.status === 'active' && rule.projections[0].id === output.projectionId
    )).sort((left, right) => left.projectionSequence - right.projectionSequence);
    assert.deepEqual(
      output.rows.map(row => row.sourceBoundaryId),
      inputRules.map(rule => rule.sourceBoundaryId),
    );
    assert.equal(inputRules.every(rule => typeof rule.sourceBoundaryId === 'string'), true);
  }

  for (const output of rustOutputs) {
    assert.match(output.content, /pub\(crate\) fn find_font_rule_layout_/);
    assert.match(output.content, /match (?:source_face|\(source_boundary_id, source_face\))/);
    assert.doesNotMatch(output.content, /\.iter\(\)/);
  }
});

test('a single rule mutation changes only its backend source output', () => {
  const original = readRegistry();
  const changed = structuredClone(original);
  const paintRule = changed.rules.find(rule => rule.projections[0].id === 'canvas2d-paint');
  paintRule.targetFaceOrPolicy = `${paintRule.targetFaceOrPolicy} changed`;
  refreshRegistryHash(changed);

  const before = outputHashes(buildProjectionBundle(original));
  const afterMutation = outputHashes(buildProjectionBundle(changed));
  const changedOutputs = Object.keys(before).filter(name => before[name] !== afterMutation[name]);

  assert.deepEqual(changedOutputs, ['canvas2d-paint']);
});

test('projection sequence perturbation changes only the affected projection digest', () => {
  const original = readRegistry();
  const changed = structuredClone(original);
  const indexes = changed.rules
    .map((rule, index) => [rule.projections[0].id, index])
    .filter(([projectionId]) => projectionId === 'canvas2d-paint')
    .slice(0, 2)
    .map(([, index]) => index);
  [changed.rules[indexes[0]].projectionSequence, changed.rules[indexes[1]].projectionSequence] = [
    changed.rules[indexes[1]].projectionSequence,
    changed.rules[indexes[0]].projectionSequence,
  ];
  refreshRegistryHash(changed);

  const before = buildProjectionBundle(original);
  const afterPerturbation = buildProjectionBundle(changed);
  const changedProjections = before.outputs
    .filter((output, index) => (
      output.projectionSha256 !== afterPerturbation.outputs[index].projectionSha256
    ))
    .map(output => output.projectionId);

  assert.deepEqual(changedProjections, ['canvas2d-paint']);
});

test('retired rules remain in the registry but never reach runtime projections', () => {
  const changed = structuredClone(readRegistry());
  const retired = changed.rules
    .filter(rule => rule.projections[0].id === 'canvas2d-paint')
    .sort((left, right) => left.projectionSequence - right.projectionSequence)
    .at(-1);
  retired.status = 'retired';
  retired.lifecycle.retiredBy = 'issue-5955.fixture.retire-tail';
  retired.lifecycle.retirementReason = 'Synthetic active-only projection contract';
  refreshRegistryHash(changed);

  const bundle = buildProjectionBundle(changed);
  const paint = bundle.outputs.find(output => output.projectionId === 'canvas2d-paint');
  assert.equal(paint.ruleCount, 280);
  assert.equal(paint.rows.some(row => row.ruleId === retired.ruleId), false);
  assert.equal(bundle.manifest.summary.activeRuleCount, 829);
  assert.equal(bundle.manifest.summary.retiredRuleCount, 1);
});

test('check detects missing, manually edited and unexpected generated outputs', () => {
  const root = temporaryRoot('rhwp-font-rule-projection-check-');
  const bundle = buildProjectionBundle(readRegistry());
  writeProjectionBundle(bundle, root);
  assert.deepEqual(compareProjectionBundle(bundle, root), []);

  const firstPath = path.join(root, bundle.outputs[0].path);
  fs.appendFileSync(firstPath, '// manual edit\n');
  assert.match(compareProjectionBundle(bundle, root).join('\n'), /stale or manually edited/);

  writeProjectionBundle(bundle, root);
  fs.rmSync(firstPath);
  assert.match(compareProjectionBundle(bundle, root).join('\n'), /generated projection is missing/);

  writeProjectionBundle(bundle, root);
  const unexpected = path.join(path.dirname(firstPath), 'manual.rs');
  fs.writeFileSync(unexpected, '// hand-written\n');
  assert.match(compareProjectionBundle(bundle, root).join('\n'), /unexpected files/);

  fs.rmSync(unexpected);
  fs.rmSync(firstPath);
  const externalRoot = temporaryRoot('rhwp-font-rule-projection-symlink-source-');
  const externalFile = path.join(externalRoot, 'layout_name.rs');
  fs.writeFileSync(externalFile, bundle.outputs[0].content);
  fs.symlinkSync(externalFile, firstPath);
  assert.match(compareProjectionBundle(bundle, root).join('\n'), /regular non-symlink file/);
});

test('whole-file ownership refuses overwrite and preserves the existing set', () => {
  const root = temporaryRoot('rhwp-font-rule-projection-owner-');
  const bundle = buildProjectionBundle(readRegistry());
  const target = path.join(root, bundle.outputs[0].path);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  const sentinel = '// existing hand-written source\n';
  fs.writeFileSync(target, sentinel);

  assert.throws(
    () => writeProjectionBundle(bundle, root),
    /without the whole-file ownership sentinel/,
  );
  assert.equal(fs.readFileSync(target, 'utf8'), sentinel);
  assert.equal(fs.existsSync(path.join(root, bundle.outputs[1].path)), false);
});

test('preflight failure cannot leave a partial generated set', () => {
  const root = temporaryRoot('rhwp-font-rule-projection-partial-');
  const bundle = buildProjectionBundle(readRegistry());
  writeProjectionBundle(bundle, root);
  const before = bundle.outputs.map(output => (
    fs.readFileSync(path.join(root, output.path), 'utf8')
  ));
  const blocked = path.join(root, bundle.outputs[4].path);
  fs.rmSync(blocked);
  fs.mkdirSync(blocked);

  assert.throws(() => writeProjectionBundle(bundle, root), /non-allowlisted|regular non-symlink/);
  for (const [index, output] of bundle.outputs.entries()) {
    const target = path.join(root, output.path);
    if (index === 4) {
      assert.equal(fs.statSync(target).isDirectory(), true);
    } else {
      assert.equal(fs.readFileSync(target, 'utf8'), before[index]);
    }
  }
});

test('mid-commit failure rolls the complete generated set back', () => {
  const root = temporaryRoot('rhwp-font-rule-projection-rollback-');
  const registry = readRegistry();
  const original = buildProjectionBundle(registry);
  writeProjectionBundle(original, root);

  const changedRegistry = structuredClone(registry);
  const paintRule = changedRegistry.rules.find(rule => (
    rule.projections[0].id === 'canvas2d-paint'
  ));
  paintRule.targetFaceOrPolicy = `${paintRule.targetFaceOrPolicy} changed`;
  refreshRegistryHash(changedRegistry);
  const changed = buildProjectionBundle(changedRegistry);

  assert.throws(
    () => writeProjectionBundle(changed, root, {
      beforeCommit(index) {
        if (index === 2) throw new Error('injected commit failure');
      },
    }),
    /injected commit failure/,
  );
  assert.deepEqual(compareProjectionBundle(original, root), []);
});

test('checkout-escaping generated directory symlinks are rejected', () => {
  const root = temporaryRoot('rhwp-font-rule-projection-root-');
  const external = temporaryRoot('rhwp-font-rule-projection-external-');
  const generatedDirectory = path.join(root, 'src', 'renderer', 'font_rule_projections');
  fs.mkdirSync(path.dirname(generatedDirectory), { recursive: true });
  fs.symlinkSync(external, generatedDirectory, 'dir');

  assert.throws(
    () => writeProjectionBundle(buildProjectionBundle(readRegistry()), root),
    /symlink|non-allowlisted|escapes the checkout/,
  );
  assert.deepEqual(fs.readdirSync(external), []);
});

test('CLI rejects caller-selected output paths', () => {
  const result = spawnSync(
    process.execPath,
    [GENERATOR_PATH, 'generate', '--output', '/tmp/font-rules.ts'],
    { cwd: ROOT, encoding: 'utf8' },
  );

  assert.equal(result.status, 1);
  assert.match(result.stderr, /fixed checkout-root output paths/);
});
