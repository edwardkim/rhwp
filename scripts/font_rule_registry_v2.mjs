#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

export const SEALED_V1_ARTIFACTS = Object.freeze({
  'assets/font-rules/font_rule_registry.json':
    'f549ca3a8807be712cc197daf14d96abb1e5f075ac55f1d9142db67c1a56681a',
  'assets/font-rules/font_rule_registry.schema.json':
    '068327e9f49843c54d0f4da16d6f0081bca86b38fe85e362c8416462f15d3ab4',
  'assets/font-rules/font_rule_projection_manifest.json':
    '77089c7dfbb3c6759161d839f5cb8b753c3271e07bb556d6eba87ef45cfaa20d',
  'mydocs/tech/investigations/issue-4966/font_rule_registry_migration.json':
    '11b93350a0702c75af07ffde7bae4aff2dab332c43ad9bb57d1e3cf1a1747e83',
});

function sha256File(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

export function assertSealedV1Artifacts(root = ROOT) {
  const errors = [];
  for (const [relativePath, expectedSha256] of Object.entries(SEALED_V1_ARTIFACTS)) {
    const file = path.join(root, relativePath);
    if (!fs.existsSync(file)) {
      errors.push(`sealed v1 artifact missing: ${relativePath}`);
      continue;
    }
    const actualSha256 = sha256File(file);
    if (actualSha256 !== expectedSha256) {
      errors.push(
        `sealed v1 artifact changed: ${relativePath} expected ${expectedSha256}, got ${actualSha256}`,
      );
    }
  }
  return errors;
}

function notImplemented(capability) {
  const error = new Error(`W7.5 reducer not implemented: ${capability}`);
  error.code = 'ERR_W75_NOT_IMPLEMENTED';
  throw error;
}

export function reduceRegistryV2() {
  return notImplemented('reduceRegistryV2');
}

export function validateChangeSet() {
  return notImplemented('validateChangeSet');
}

export function validateRegistryV2() {
  return notImplemented('validateRegistryV2');
}

export function buildMigrationV1ToV2() {
  return notImplemented('buildMigrationV1ToV2');
}

export function projectActiveRules() {
  return notImplemented('projectActiveRules');
}

export function resolveRuleLifecycle() {
  return notImplemented('resolveRuleLifecycle');
}

function main(args) {
  if (args.length !== 1 || args[0] !== 'check') {
    throw new Error('usage: node scripts/font_rule_registry_v2.mjs check');
  }
  const sealedErrors = assertSealedV1Artifacts(ROOT);
  if (sealedErrors.length > 0) throw new Error(sealedErrors.join('\n'));
  reduceRegistryV2();
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
