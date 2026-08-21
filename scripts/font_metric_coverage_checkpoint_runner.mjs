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
import { assertLocalOutputPath } from './font_metric_coverage_pilot_selector.mjs';
import {
  DEFAULT_ISOLATION_LIMITS,
  runIsolatedCoverageDocument,
} from './font_metric_coverage_supervisor.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
const INVESTIGATION = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
);
const COVERAGE_CONTRACT_PATH = path.join(
  INVESTIGATION,
  'font_metric_coverage_contract.json',
);
const COVERAGE_CONTRACT_BYTES = fs.readFileSync(COVERAGE_CONTRACT_PATH);
const COVERAGE_CONTRACT = JSON.parse(COVERAGE_CONTRACT_BYTES.toString('utf8'));
const DEFAULT_CHECKPOINT_POLICY_PATH = path.join(
  INVESTIGATION,
  'font_metric_coverage_checkpoint_policy.json',
);
const CATEGORY_KEYS = COVERAGE_CONTRACT.categories.map(category => category.id);
const FAILURE_KEYS = [...COVERAGE_CONTRACT.collectorRequirements.documentFailureStates].sort();
const STATE_FILE = 'state.json';
const JOURNAL_FILE = 'journal.ndjson';

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

function validatePolicy(policy) {
  const requiredIdentityFields = [
    'sourceHead',
    'runnerSha256',
    'workerSha256',
    'coverageContractSha256',
    'manifestSha256',
    'manifestPolicyVersion',
    'checkpointPolicySha256',
    'analysisOptionsSha256',
    'isolationLimitsSha256',
    'documentCount',
  ];
  if (policy?.schemaVersion !== 1
      || policy.kind !== 'font-metric-coverage-checkpoint-policy'
      || policy.policyVersion !== 'checkpoint-v1'
      || policy.journal?.appendOncePerCompletedDocument !== true
      || policy.journal?.fsyncBeforeStateCommit !== true
      || policy.state?.atomicRename !== true
      || policy.state?.recordsCommittedJournalBytes !== true
      || policy.resume?.identityDrift !== 'reject'
      || policy.resume?.uncommittedJournalTail !== 'truncate'
      || policy.resume?.replayMustMatchState !== true
      || policy.hashChain?.algorithm !== 'sha256-chain-v1'
      || !safeInteger(policy.storage?.maxJournalBytes)
      || policy.storage.maxJournalBytes === 0
      || !safeInteger(policy.storage?.minimumFreeBytesAfterAppend)
      || policy.storage.limitExceeded !== 'reject-before-append'
      || canonicalJson(policy.identityFields) !== canonicalJson(requiredIdentityFields)) {
    throw new Error('checkpoint policy is invalid');
  }
}

function mergedLimits(overrides = {}) {
  const unknown = Object.keys(overrides).filter(key => !(key in DEFAULT_ISOLATION_LIMITS));
  if (unknown.length > 0) throw new Error('unknown isolation limit option');
  const limits = { ...DEFAULT_ISOLATION_LIMITS, ...overrides };
  for (const [key, value] of Object.entries(limits)) {
    if (!safeInteger(value) || value === 0) throw new Error(`invalid isolation limit: ${key}`);
  }
  return limits;
}

function manifestDocuments(manifest) {
  if (manifest?.localOnly !== true
      || ![
        'font-metric-coverage-private-pilot-cohort',
        'font-metric-coverage-private-corpus-manifest',
      ].includes(manifest.kind)) {
    throw new Error('checkpoint manifest must be a local-only private manifest');
  }
  if (typeof manifest.policyVersion !== 'string' || manifest.policyVersion.length === 0) {
    throw new Error('checkpoint manifest policyVersion is required');
  }
  const documents = manifest?.selections ?? manifest?.documents;
  if (!Array.isArray(documents) || documents.length === 0) {
    throw new Error('checkpoint manifest documents are required');
  }
  const sources = new Set();
  for (const document of documents) {
    if (typeof document?.source !== 'string' || document.source.length === 0) {
      throw new Error('checkpoint manifest source is required');
    }
    if (!['hwp', 'hwpx'].includes(document.format)) {
      throw new Error('checkpoint manifest format is invalid');
    }
    if (!/^[0-9a-f]{64}$/u.test(document.blake3 ?? '')) {
      throw new Error('checkpoint manifest BLAKE3 is invalid');
    }
    if (sources.has(document.source)) {
      throw new Error('checkpoint manifest contains a duplicate source');
    }
    sources.add(document.source);
  }
  return documents;
}

function workerSha256(workerPath, override) {
  if (override !== undefined) {
    if (!/^[0-9a-f]{64}$/u.test(override)) throw new Error('worker SHA-256 is invalid');
    return override;
  }
  return sha256Bytes(fs.readFileSync(workerPath));
}

function runIdentity(options, documents, limits) {
  if (!/^[0-9a-f]{40}$/u.test(options.sourceHead ?? '')) {
    throw new Error('sourceHead must be a full Git commit');
  }
  const policyBytes = options.checkpointPolicyBytes;
  const manifestBytes = options.manifestBytes;
  if (!Buffer.isBuffer(policyBytes) || !Buffer.isBuffer(manifestBytes)) {
    throw new Error('raw policy and manifest bytes are required');
  }
  return {
    runnerSchemaVersion: 1,
    checkpointPolicyVersion: options.checkpointPolicy.policyVersion,
    checkpointPolicySha256: sha256Bytes(policyBytes),
    runnerSha256: sha256Bytes(fs.readFileSync(SCRIPT_PATH)),
    coverageContractSha256: sha256Bytes(
      options.coverageContractBytes ?? COVERAGE_CONTRACT_BYTES,
    ),
    manifestPolicyVersion: options.manifest.policyVersion ?? null,
    manifestSha256: sha256Bytes(manifestBytes),
    sourceHead: options.sourceHead,
    workerSha256: workerSha256(options.workerPath, options.workerSha256),
    analysisOptionsSha256: sha256Json(options.analysisOptions ?? {}),
    isolationLimitsSha256: sha256Json(limits),
    documentCount: documents.length,
  };
}

function emptySummary(identity) {
  return {
    documents: {
      attempted: 0,
      success: 0,
      failures: Object.fromEntries(FAILURE_KEYS.map(key => [key, 0])),
    },
    counts: {},
    usageRowSums: { legacyUsageRows: 0, decisionUsageRows: 0 },
    categories: Object.fromEntries(CATEGORY_KEYS.map(key => [key, 0])),
    joins: {},
    backends: {},
    resources: { elapsedMillis: 0, peakWorkerRssBytes: 0 },
    chain: {
      algorithm: 'sha256-chain-v1',
      value: sha256Bytes(`font-metric-coverage-checkpoint-v1\n${canonicalJson(identity)}`),
    },
  };
}

function addNumericObject(target, source, label) {
  if (!source || typeof source !== 'object' || Array.isArray(source)) {
    throw new Error(`${label} must be an object`);
  }
  for (const [key, value] of Object.entries(source)) {
    if (!safeInteger(value)) throw new Error(`${label}.${key} must be a safe count`);
    target[key] = (target[key] ?? 0) + value;
  }
}

function addCoverageCounts(summary, counts) {
  if (!counts || typeof counts !== 'object' || Array.isArray(counts)) {
    throw new Error('counts must be an object');
  }
  for (const [key, value] of Object.entries(counts)) {
    if (!safeInteger(value)) throw new Error(`counts.${key} must be a safe count`);
    if (key === 'legacyUsageRows' || key === 'decisionUsageRows') {
      summary.usageRowSums[key] += value;
    } else {
      summary.counts[key] = (summary.counts[key] ?? 0) + value;
    }
  }
}

function validateMetrics(metrics) {
  if (!safeInteger(metrics?.elapsedMillis) || !safeInteger(metrics?.peakRssBytes)) {
    throw new Error('checkpoint record metrics are invalid');
  }
}

export function validateCompleteCoverageAggregate(aggregate) {
  const failures = aggregate?.documents?.failures;
  if (aggregate?.kind !== 'font-metric-coverage-aggregate'
      || aggregate.status !== 'complete'
      || aggregate.documents?.attempted !== 1
      || aggregate.documents?.success !== 1
      || !failures
      || Object.values(failures).some(value => value !== 0)
      || reconcileCoverageAggregate(aggregate, COVERAGE_CONTRACT).length > 0
      || findSensitiveAggregateValues(aggregate, COVERAGE_CONTRACT).length > 0
      || aggregate.aggregateHash?.algorithm !== 'sha256'
      || !/^[0-9a-f]{64}$/u.test(aggregate.aggregateHash?.value ?? '')
      || canonicalCoverageHash(aggregate) !== aggregate.aggregateHash.value) {
    throw new Error('checkpoint aggregate failed its contract');
  }
}

function recordToken(record) {
  if (record.status === 'complete') {
    return `${record.format}:complete:${record.aggregate.aggregateHash.value}`;
  }
  return `${record.format}:failed:${record.failure}`;
}

function applyRecord(summary, record, expectedIndex) {
  if (record?.schemaVersion !== 1
      || record.kind !== 'font-metric-coverage-checkpoint-record'
      || record.index !== expectedIndex
      || !['hwp', 'hwpx'].includes(record.format)
      || !['complete', 'failed'].includes(record.status)) {
    throw new Error('checkpoint journal record is invalid');
  }
  validateMetrics(record.metrics);
  summary.documents.attempted += 1;
  summary.resources.elapsedMillis += record.metrics.elapsedMillis;
  summary.resources.peakWorkerRssBytes = Math.max(
    summary.resources.peakWorkerRssBytes,
    record.metrics.peakRssBytes,
  );
  if (record.status === 'complete') {
    validateCompleteCoverageAggregate(record.aggregate);
    summary.documents.success += 1;
    addCoverageCounts(summary, record.aggregate.counts);
    addNumericObject(summary.categories, record.aggregate.categories, 'categories');
    addNumericObject(summary.joins, record.aggregate.joins, 'joins');
    addNumericObject(summary.backends, record.aggregate.backends, 'backends');
  } else {
    if (!FAILURE_KEYS.includes(record.failure)) {
      throw new Error('checkpoint failure state is invalid');
    }
    summary.documents.failures[record.failure] += 1;
  }
  summary.chain.value = sha256Bytes(`${summary.chain.value}\n${recordToken(record)}`);
}

function stateValue(identity, summary, nextIndex, journalBytes, status = 'running') {
  return {
    schemaVersion: 1,
    kind: 'font-metric-coverage-checkpoint-state',
    status,
    identity,
    nextIndex,
    journal: { entries: nextIndex, committedBytes: journalBytes },
    summary,
  };
}

function fsyncDirectory(directory) {
  const descriptor = fs.openSync(directory, 'r');
  try {
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
}

function atomicWriteJson(filePath, value) {
  const temporary = `${filePath}.next`;
  const descriptor = fs.openSync(temporary, 'w', 0o600);
  try {
    fs.writeFileSync(descriptor, `${JSON.stringify(value, null, 2)}\n`);
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
  fs.chmodSync(temporary, 0o600);
  fs.renameSync(temporary, filePath);
  fsyncDirectory(path.dirname(filePath));
}

function journalLine(record) {
  return `${JSON.stringify(record)}\n`;
}

function assertJournalCapacity(directory, committedBytes, lineBytes, storagePolicy) {
  if (committedBytes + lineBytes > storagePolicy.maxJournalBytes) {
    throw new Error('checkpoint journal storage limit exceeded');
  }
  const filesystem = fs.statfsSync(directory, { bigint: true });
  const availableBytes = filesystem.bavail * filesystem.bsize;
  const requiredBytes = BigInt(lineBytes + storagePolicy.minimumFreeBytesAfterAppend);
  if (availableBytes < requiredBytes) {
    throw new Error('checkpoint journal free-space reserve would be exceeded');
  }
}

function appendJournal(filePath, line) {
  const descriptor = fs.openSync(filePath, 'a', 0o600);
  try {
    fs.writeFileSync(descriptor, line);
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
  fs.chmodSync(filePath, 0o600);
  return Buffer.byteLength(line);
}

function replayJournal(journalPath, committedBytes, identity, options = {}) {
  const truncateUncommittedTail = options.truncateUncommittedTail ?? true;
  const enforcePermissions = options.enforcePermissions ?? true;
  if (!fs.existsSync(journalPath)) {
    if (committedBytes === 0) return { summary: emptySummary(identity), entries: 0 };
    throw new Error('committed checkpoint journal is missing');
  }
  const size = fs.statSync(journalPath).size;
  if (size < committedBytes) throw new Error('checkpoint journal is shorter than committed state');
  if (size > committedBytes) {
    if (!truncateUncommittedTail) {
      throw new Error('completed checkpoint journal has an uncommitted tail');
    }
    fs.truncateSync(journalPath, committedBytes);
    const descriptor = fs.openSync(journalPath, 'r+');
    try {
      fs.fsyncSync(descriptor);
    } finally {
      fs.closeSync(descriptor);
    }
  }
  if (enforcePermissions) fs.chmodSync(journalPath, 0o600);
  const bytes = fs.readFileSync(journalPath).subarray(0, committedBytes);
  if (bytes.length > 0 && bytes.at(-1) !== 0x0A) {
    throw new Error('committed checkpoint journal lacks a record boundary');
  }
  const lines = bytes.toString('utf8').split('\n').filter(Boolean);
  const summary = emptySummary(identity);
  const records = lines.map((line, index) => {
    const record = JSON.parse(line);
    applyRecord(summary, record, index);
    return record;
  });
  return { summary, entries: lines.length, records };
}

function initializeOrResume(checkpointDirectory, identity) {
  fs.mkdirSync(checkpointDirectory, { recursive: true, mode: 0o700 });
  fs.chmodSync(checkpointDirectory, 0o700);
  const statePath = path.join(checkpointDirectory, STATE_FILE);
  const journalPath = path.join(checkpointDirectory, JOURNAL_FILE);
  if (!fs.existsSync(statePath)) {
    if (fs.existsSync(journalPath) && fs.statSync(journalPath).size > 0) {
      throw new Error('checkpoint journal exists without state');
    }
    fs.writeFileSync(journalPath, '', { mode: 0o600 });
    fs.chmodSync(journalPath, 0o600);
    const state = stateValue(identity, emptySummary(identity), 0, 0);
    atomicWriteJson(statePath, state);
    return { state, statePath, journalPath };
  }
  const state = JSON.parse(fs.readFileSync(statePath, 'utf8'));
  fs.chmodSync(statePath, 0o600);
  if (state?.schemaVersion !== 1
      || state.kind !== 'font-metric-coverage-checkpoint-state'
      || !['running', 'complete'].includes(state.status)
      || canonicalJson(state.identity) !== canonicalJson(identity)
      || !safeInteger(state.nextIndex)
      || state.journal?.entries !== state.nextIndex
      || !safeInteger(state.journal?.committedBytes)) {
    throw new Error('checkpoint state identity or schema drift');
  }
  const replay = replayJournal(journalPath, state.journal.committedBytes, identity);
  if (replay.entries !== state.nextIndex
      || canonicalJson(replay.summary) !== canonicalJson(state.summary)) {
    throw new Error('checkpoint journal replay does not match state');
  }
  return { state, statePath, journalPath };
}

function checkpointRecord(index, format, result) {
  validateMetrics(result?.metrics);
  if (result.status === 'complete') {
    validateCompleteCoverageAggregate(result.aggregate);
    if (result.aggregate.format !== format) {
      throw new Error('checkpoint aggregate format does not match manifest');
    }
    return {
      schemaVersion: 1,
      kind: 'font-metric-coverage-checkpoint-record',
      index,
      format,
      status: 'complete',
      aggregate: result.aggregate,
      metrics: result.metrics,
    };
  }
  if (result.status === 'failed' && FAILURE_KEYS.includes(result.failure)) {
    return {
      schemaVersion: 1,
      kind: 'font-metric-coverage-checkpoint-record',
      index,
      format,
      status: 'failed',
      failure: result.failure,
      metrics: result.metrics,
    };
  }
  throw new Error('isolated document result is invalid');
}

export function readCompleteCoverageCheckpoint(checkpointDirectory) {
  const statePath = path.join(checkpointDirectory, STATE_FILE);
  const journalPath = path.join(checkpointDirectory, JOURNAL_FILE);
  if (!fs.existsSync(statePath)) throw new Error('completed checkpoint state is missing');
  const state = JSON.parse(fs.readFileSync(statePath, 'utf8'));
  if (state?.schemaVersion !== 1
      || state.kind !== 'font-metric-coverage-checkpoint-state'
      || state.status !== 'complete'
      || !safeInteger(state.nextIndex)
      || state.nextIndex !== state.identity?.documentCount
      || state.journal?.entries !== state.nextIndex
      || !safeInteger(state.journal?.committedBytes)) {
    throw new Error('completed checkpoint state is invalid');
  }
  const replay = replayJournal(
    journalPath,
    state.journal.committedBytes,
    state.identity,
    { truncateUncommittedTail: false, enforcePermissions: false },
  );
  if (replay.entries !== state.nextIndex
      || canonicalJson(replay.summary) !== canonicalJson(state.summary)) {
    throw new Error('completed checkpoint journal replay does not match state');
  }
  return { state, records: replay.records };
}

export async function runResumableCoverage(options) {
  validatePolicy(options.checkpointPolicy);
  const documents = manifestDocuments(options.manifest);
  const limits = mergedLimits(options.limits);
  const identity = runIdentity(options, documents, limits);
  const checkpoint = initializeOrResume(options.checkpointDirectory, identity);
  if (checkpoint.state.status === 'complete') {
    if (checkpoint.state.nextIndex !== documents.length) {
      throw new Error('completed checkpoint has an incomplete index');
    }
    return checkpoint.state;
  }
  const runDocument = options.runDocument ?? runIsolatedCoverageDocument;
  for (let index = checkpoint.state.nextIndex; index < documents.length; index += 1) {
    const document = documents[index];
    const result = await runDocument(document.source, {
      workerPath: options.workerPath,
      analysisOptions: options.analysisOptions ?? {},
      limits,
    });
    const record = checkpointRecord(index, document.format, result);
    const nextSummary = structuredClone(checkpoint.state.summary);
    applyRecord(nextSummary, record, index);
    const line = journalLine(record);
    const bytesAdded = Buffer.byteLength(line);
    assertJournalCapacity(
      options.checkpointDirectory,
      checkpoint.state.journal.committedBytes,
      bytesAdded,
      options.checkpointPolicy.storage,
    );
    appendJournal(checkpoint.journalPath, line);
    const nextIndex = index + 1;
    const committedBytes = checkpoint.state.journal.committedBytes + bytesAdded;
    const status = nextIndex === documents.length ? 'complete' : 'running';
    checkpoint.state = stateValue(identity, nextSummary, nextIndex, committedBytes, status);
    atomicWriteJson(checkpoint.statePath, checkpoint.state);
    if (options.onProgress) {
      options.onProgress({
        completed: nextIndex,
        total: documents.length,
        status: record.status,
        metrics: record.metrics,
      });
    }
  }
  return checkpoint.state;
}

function parseArguments(arguments_) {
  const values = {
    policy: DEFAULT_CHECKPOINT_POLICY_PATH,
    analysisOptionsJson: '{}',
  };
  const allowed = new Set([
    '--manifest',
    '--checkpoint-dir',
    '--worker',
    '--source-head',
    '--policy',
    '--analysis-options-json',
  ]);
  for (let index = 0; index < arguments_.length; index += 2) {
    const option = arguments_[index];
    const value = arguments_[index + 1];
    if (!allowed.has(option) || value === undefined) {
      throw new Error('usage: checkpoint runner --manifest file --checkpoint-dir dir --worker file --source-head commit');
    }
    const key = option.slice(2).replaceAll(/-([a-z])/gu, (_, letter) => letter.toUpperCase());
    values[key] = value;
  }
  for (const key of ['manifest', 'checkpointDir', 'worker', 'sourceHead']) {
    if (!values[key]) throw new Error(`checkpoint runner requires ${key}`);
  }
  return values;
}

async function main() {
  const arguments_ = parseArguments(process.argv.slice(2));
  const manifestPath = assertLocalOutputPath(arguments_.manifest);
  const checkpointDirectory = assertLocalOutputPath(arguments_.checkpointDir);
  const manifestBytes = fs.readFileSync(manifestPath);
  const policyBytes = fs.readFileSync(arguments_.policy);
  const state = await runResumableCoverage({
    manifest: JSON.parse(manifestBytes.toString('utf8')),
    manifestBytes,
    checkpointPolicy: JSON.parse(policyBytes.toString('utf8')),
    checkpointPolicyBytes: policyBytes,
    checkpointDirectory,
    workerPath: path.resolve(arguments_.worker),
    sourceHead: arguments_.sourceHead,
    analysisOptions: JSON.parse(arguments_.analysisOptionsJson),
    onProgress: progress => {
      process.stderr.write(
        `checkpoint progress: ${progress.completed}/${progress.total}; ${progress.status}; `
          + `elapsed=${progress.metrics.elapsedMillis}ms; peakRss=${progress.metrics.peakRssBytes}\n`,
      );
    },
  });
  process.stdout.write(`${JSON.stringify(state)}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    await main();
  } catch (error) {
    process.stderr.write(`checkpoint runner failed: ${error.message}\n`);
    process.exitCode = 1;
  }
}
