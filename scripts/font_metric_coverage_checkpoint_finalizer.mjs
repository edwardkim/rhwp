#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';

import {
  canonicalCoverageHash,
  findSensitiveAggregateValues,
  reconcileCoverageAggregate,
} from './font_metric_coverage_contract.mjs';
import {
  readCompleteCoverageCheckpoint,
  validateCompleteCoverageAggregate,
} from './font_metric_coverage_checkpoint_runner.mjs';
import { assertLocalOutputPath } from './font_metric_coverage_pilot_selector.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
const INVESTIGATION = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
);
const CONTRACT = JSON.parse(fs.readFileSync(path.join(
  INVESTIGATION,
  'font_metric_coverage_contract.json',
), 'utf8'));
const DEFAULT_POLICY_PATH = path.join(
  INVESTIGATION,
  'font_metric_coverage_finalizer_policy.json',
);
const FORMATS = ['hwp', 'hwpx'];

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value).sort().map(key => [key, canonical(value[key])]),
    );
  }
  return value;
}

function canonicalJson(value) {
  return JSON.stringify(canonical(value));
}

function sha256Bytes(value) {
  return createHash('sha256').update(value).digest('hex');
}

function sha256Json(value) {
  return sha256Bytes(canonicalJson(value));
}

function safeInteger(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function checkedAdd(left, right, label) {
  if (!safeInteger(left) || !safeInteger(right) || !Number.isSafeInteger(left + right)) {
    throw new Error(`${label} exceeds the safe integer range`);
  }
  return left + right;
}

function exactKeys(value, expected, label) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  const actual = Object.keys(value).sort();
  const wanted = [...expected].sort();
  if (canonicalJson(actual) !== canonicalJson(wanted)) {
    throw new Error(`${label} schema drift`);
  }
}

function validatePolicy(policy) {
  if (policy?.schemaVersion !== 1
      || policy.kind !== 'font-metric-coverage-finalizer-policy'
      || policy.policyVersion !== 'finalizer-v1'
      || policy.merge?.injectFormatIntoUsageIdentity !== true
      || policy.merge?.sumOnlyUsageCountFields !== true
      || policy.merge?.failedDocumentsContributeNoUsage !== true
      || policy.merge?.inputOrderIndependentRows !== true
      || policy.output?.kind !== 'font-metric-coverage-aggregate'
      || policy.output?.privacyChecked !== true
      || policy.output?.canonicalSha256 !== true
      || policy.output?.volatileResourceMetricsExcluded !== true) {
    throw new Error('font metric coverage finalizer policy is invalid');
  }
  for (const field of [
    'aggregateCountFields',
    'usageCountFields',
    'legacyIdentityFields',
    'decisionIdentityFields',
  ]) {
    const values = policy[field];
    if (!Array.isArray(values)
        || values.length === 0
        || values.some(value => typeof value !== 'string' || value.length === 0)
        || new Set(values).size !== values.length) {
      throw new Error(`finalizer policy ${field} is invalid`);
    }
  }
  if (!policy.legacyIdentityFields.includes('format')
      || !policy.decisionIdentityFields.includes('format')) {
    throw new Error('finalizer usage identity must include format');
  }
}

function emptyFailures() {
  return Object.fromEntries(
    [...CONTRACT.collectorRequirements.documentFailureStates].sort().map(key => [key, 0]),
  );
}

function emptyFormatDocuments() {
  return Object.fromEntries(FORMATS.map(format => [format, {
    attempted: 0,
    success: 0,
    failures: emptyFailures(),
  }]));
}

function addNumericFields(target, source, fields, label) {
  exactKeys(source, fields, label);
  for (const field of fields) {
    if (!safeInteger(source[field])) throw new Error(`${label}.${field} is not a safe count`);
    target[field] = checkedAdd(target[field] ?? 0, source[field], `${label}.${field}`);
  }
}

function mergeUsageRows(target, rows, format, identityFields, countFields, label) {
  if (!Array.isArray(rows)) throw new Error(`${label} must be an array`);
  const inputIdentityFields = identityFields.filter(field => field !== 'format');
  const expectedInputFields = [...inputIdentityFields, ...countFields];
  for (const row of rows) {
    exactKeys(row, expectedInputFields, `${label} row`);
    const identity = { format };
    for (const field of inputIdentityFields) identity[field] = row[field];
    const key = canonicalJson(identity);
    let merged = target.get(key);
    if (!merged) {
      merged = { ...identity, ...Object.fromEntries(countFields.map(field => [field, 0])) };
      target.set(key, merged);
    }
    for (const field of countFields) {
      if (!safeInteger(row[field])) throw new Error(`${label}.${field} is not a safe count`);
      merged[field] = checkedAdd(merged[field], row[field], `${label}.${field}`);
    }
  }
}

function sortedUsageRows(rows) {
  return [...rows.entries()]
    .sort(([leftKey, left], [rightKey, right]) => (
      right.charCount - left.charCount || (leftKey < rightKey ? -1 : leftKey > rightKey ? 1 : 0)
    ))
    .map(([, row]) => row);
}

function sumRows(rows, field) {
  return rows.reduce((total, row) => checkedAdd(total, row[field], field), 0);
}

function ensureUsageReconciliation(aggregate) {
  if (sumRows(aggregate.legacyUsage, 'charCount') !== aggregate.joins.joined
      || sumRows(aggregate.decisionUsage, 'charCount') !== aggregate.joins.joined
      || sumRows(aggregate.legacyUsage, 'runCount') !== aggregate.counts.sourceRunsSeen) {
    throw new Error('finalized usage rows do not reconcile');
  }
}

export function finalizeCoverageCheckpoint(checkpointDirectory, options = {}) {
  const policyBytes = options.policyBytes ?? fs.readFileSync(DEFAULT_POLICY_PATH);
  const policy = options.policy ?? JSON.parse(policyBytes.toString('utf8'));
  validatePolicy(policy);
  const checkpoint = readCompleteCoverageCheckpoint(checkpointDirectory);
  const aggregateCounts = Object.fromEntries(
    policy.aggregateCountFields.map(field => [field, 0]),
  );
  const categories = Object.fromEntries(CONTRACT.categories.map(entry => [entry.id, 0]));
  const joins = Object.fromEntries(
    [...CONTRACT.collectorRequirements.sourceJoinStates].sort().map(key => [key, 0]),
  );
  const backends = Object.fromEntries([
    ['requested', 0],
    ...[...CONTRACT.collectorRequirements.backendStates].sort().map(key => [key, 0]),
  ]);
  const documents = {
    attempted: 0,
    success: 0,
    failures: emptyFailures(),
    formats: emptyFormatDocuments(),
  };
  const legacyRows = new Map();
  const decisionRows = new Map();

  for (const record of checkpoint.records) {
    if (!FORMATS.includes(record.format)) throw new Error('checkpoint record format is invalid');
    documents.attempted = checkedAdd(documents.attempted, 1, 'documents.attempted');
    documents.formats[record.format].attempted = checkedAdd(
      documents.formats[record.format].attempted,
      1,
      `documents.formats.${record.format}.attempted`,
    );
    if (record.status === 'failed') {
      documents.failures[record.failure] = checkedAdd(
        documents.failures[record.failure],
        1,
        `documents.failures.${record.failure}`,
      );
      documents.formats[record.format].failures[record.failure] = checkedAdd(
        documents.formats[record.format].failures[record.failure],
        1,
        `documents.formats.${record.format}.failures.${record.failure}`,
      );
      continue;
    }

    validateCompleteCoverageAggregate(record.aggregate);
    documents.success = checkedAdd(documents.success, 1, 'documents.success');
    documents.formats[record.format].success = checkedAdd(
      documents.formats[record.format].success,
      1,
      `documents.formats.${record.format}.success`,
    );
    addNumericFields(
      aggregateCounts,
      record.aggregate.counts,
      policy.aggregateCountFields,
      'counts',
    );
    addNumericFields(categories, record.aggregate.categories, Object.keys(categories), 'categories');
    addNumericFields(joins, record.aggregate.joins, Object.keys(joins), 'joins');
    addNumericFields(backends, record.aggregate.backends, Object.keys(backends), 'backends');
    mergeUsageRows(
      legacyRows,
      record.aggregate.legacyUsage,
      record.format,
      policy.legacyIdentityFields,
      policy.usageCountFields,
      'legacyUsage',
    );
    mergeUsageRows(
      decisionRows,
      record.aggregate.decisionUsage,
      record.format,
      policy.decisionIdentityFields,
      policy.usageCountFields,
      'decisionUsage',
    );
  }

  const legacyUsage = sortedUsageRows(legacyRows);
  const decisionUsage = sortedUsageRows(decisionRows);
  aggregateCounts.legacyUsageRows = legacyUsage.length;
  aggregateCounts.decisionUsageRows = decisionUsage.length;
  const observedFormats = FORMATS.filter(format => documents.formats[format].attempted > 0);
  const legacyProjection = {
    schemaVersion: 'poc-font-layout-habits-v2',
    format: observedFormats.length === 1 ? observedFormats[0] : 'mixed',
    paragraphs: aggregateCounts.paragraphsSeen,
    chars: joins.joined,
    usage: legacyUsage,
  };
  const aggregate = {
    schemaVersion: 1,
    kind: 'font-metric-coverage-aggregate',
    status: 'complete',
    format: legacyProjection.format,
    checkpoint: {
      identity: checkpoint.state.identity,
      chain: checkpoint.state.summary.chain,
      entries: checkpoint.records.length,
    },
    finalizer: {
      policyVersion: policy.policyVersion,
      policySha256: sha256Bytes(policyBytes),
      scriptSha256: sha256Bytes(fs.readFileSync(SCRIPT_PATH)),
    },
    counts: aggregateCounts,
    categories,
    joins,
    documents,
    backends,
    legacyProjectionHash: { algorithm: 'sha256', value: sha256Json(legacyProjection) },
    aggregateHash: { algorithm: 'sha256', value: '' },
    legacyUsage,
    decisionUsage,
  };
  ensureUsageReconciliation(aggregate);
  const reconciliationErrors = reconcileCoverageAggregate(aggregate, CONTRACT);
  if (reconciliationErrors.length > 0) {
    throw new Error(`finalized aggregate does not reconcile: ${reconciliationErrors.join('; ')}`);
  }
  if (findSensitiveAggregateValues(aggregate, CONTRACT).length > 0) {
    throw new Error('finalized aggregate failed privacy validation');
  }
  aggregate.aggregateHash.value = canonicalCoverageHash(aggregate);
  return aggregate;
}

function parseArguments(arguments_) {
  const values = { policy: DEFAULT_POLICY_PATH };
  const allowed = new Set(['--checkpoint-dir', '--output', '--policy']);
  for (let index = 0; index < arguments_.length; index += 2) {
    const option = arguments_[index];
    const value = arguments_[index + 1];
    if (!allowed.has(option) || value === undefined) {
      throw new Error('usage: checkpoint finalizer --checkpoint-dir dir --output file [--policy file]');
    }
    const key = option.slice(2).replaceAll(/-([a-z])/gu, (_, letter) => letter.toUpperCase());
    values[key] = value;
  }
  if (!values.checkpointDir || !values.output) {
    throw new Error('checkpoint finalizer requires checkpoint-dir and output');
  }
  return values;
}

function writeNewPrivateJson(outputPath, value) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true, mode: 0o700 });
  fs.chmodSync(path.dirname(outputPath), 0o700);
  const descriptor = fs.openSync(outputPath, 'wx', 0o600);
  try {
    fs.writeFileSync(descriptor, `${JSON.stringify(value)}\n`);
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
  fs.chmodSync(outputPath, 0o600);
}

function main() {
  const arguments_ = parseArguments(process.argv.slice(2));
  const checkpointDirectory = assertLocalOutputPath(arguments_.checkpointDir);
  const outputPath = assertLocalOutputPath(arguments_.output);
  const policyBytes = fs.readFileSync(arguments_.policy);
  const aggregate = finalizeCoverageCheckpoint(checkpointDirectory, {
    policy: JSON.parse(policyBytes.toString('utf8')),
    policyBytes,
  });
  writeNewPrivateJson(outputPath, aggregate);
  process.stdout.write(`${JSON.stringify({
    status: 'complete',
    documents: aggregate.documents,
    counts: aggregate.counts,
    aggregateHash: aggregate.aggregateHash,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`checkpoint finalizer failed: ${error.message}\n`);
    process.exitCode = 1;
  }
}
