import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import test, { after } from 'node:test';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from '../font_rule_ledger.mjs';
import {
  GENERATED_SENTINEL,
  OUTPUT_CONFIGS,
  buildProjectionBundle,
  compareProjectionBundle,
  validateProjectionManifest,
  writeProjectionBundle,
} from '../font_rule_projection_gen.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const REGISTRY_PATH = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry.json');
const GENERATOR_PATH = path.join(ROOT, 'scripts', 'font_rule_projection_gen.mjs');
const temporaryDirectories = [];

function readRegistry() {
  return JSON.parse(fs.readFileSync(REGISTRY_PATH, 'utf8'));
}

function temporaryRoot(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  temporaryDirectories.push(root);
  return root;
}

function refreshRegistryHash(registry) {
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
  assert.equal(first.manifest.summary.ruleCount, 830);
  assert.deepEqual(first.manifest.summary.countsByProjection, {
    'canvas2d-paint': 281,
    'canvas2d-webfont': 153,
    'canvaskit-sfnt': 158,
    'rust-layout-metric': 67,
    'rust-layout-name': 171,
  });
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
        .filter(rule => rule.projections[0].id === output.projectionId)
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
      rule.projections[0].id === output.projectionId
    ));
    assert.deepEqual(
      output.rows.map(row => row.sourceBoundaryId),
      inputRules.map(rule => rule.evidence.sourceBoundaryIds[0]),
    );
    assert.equal(inputRules.every(rule => rule.evidence.sourceBoundaryIds.length === 1), true);
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

test('projection order perturbation changes only the affected projection digest', () => {
  const original = readRegistry();
  const changed = structuredClone(original);
  const indexes = changed.rules
    .map((rule, index) => [rule.projections[0].id, index])
    .filter(([projectionId]) => projectionId === 'canvas2d-paint')
    .slice(0, 2)
    .map(([, index]) => index);
  [changed.rules[indexes[0]], changed.rules[indexes[1]]] = [
    changed.rules[indexes[1]],
    changed.rules[indexes[0]],
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
