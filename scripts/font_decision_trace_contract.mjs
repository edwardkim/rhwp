#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from './font_rule_ledger.mjs';

const REPOSITORY_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const INVESTIGATION = path.join(
  REPOSITORY_ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4961',
);
const LEDGER_PATH = path.join(
  REPOSITORY_ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4939',
  'font_rule_ledger.json',
);
const EVIDENCE_PREFIX =
  'mydocs/tech/investigations/issue-4939/font_rule_candidates.json#';

export const DEFAULT_CHARACTER_LIMIT = 1024;
export const MAX_CHARACTER_LIMIT = 4096;

const TRACE_STATUSES = new Set(['complete', 'truncated', 'unsupported', 'failed']);
const BACKEND_STATUSES = new Set(['complete', 'unsupported', 'notObserved', 'failed']);
const CERTAINTIES = new Set(['observed', 'resolved', 'planned', 'notObserved', 'unsupported']);
const REASON_CODES = new Set([
  'characterLimitExceeded',
  'backendUnsupported',
  'backendNotObserved',
  'backendJoinMissing',
  'sourceCoordinateUnavailable',
  'ledgerRuleMissing',
  'ledgerSourceDrift',
  'hashUnavailable',
  'serializationFailed',
  'sourceMappingMismatch',
  'recordsOmittedUnknown',
]);
const TRACE_FIELDS = [
  'schemaVersion',
  'status',
  'scope',
  'counts',
  'records',
  'backendSummary',
  'reasons',
  'layoutHash',
  'normalizedHash',
];
const RECORD_FIELDS = [
  'recordId',
  'source',
  'document',
  'layoutName',
  'layoutMetric',
  'paint',
  'provenance',
  'oracle',
];
const IDENTITY_FIELDS = [
  'sourceBoundaryId',
  'candidateKind',
  'sourceFace',
  'targetOrPolicy',
  'conditions',
  'order',
];
const HASH_PATTERN = /^[0-9a-f]{64}$/;
const CANDIDATE_PATTERN = /^candidate\.[0-9a-f]{20}$/;
const RULE_PATTERN = /^rule\.[a-z0-9-]+\.[0-9a-f]{20}$/;
const VOLATILE_HASH_FIELDS = new Set([
  'layoutHash',
  'normalizedHash',
  'timestamp',
  'generatedAt',
  'elapsedMs',
  'durationMs',
  'stack',
]);
const UNORDERED_STRING_ARRAY_FIELDS = new Set([
  'capabilities',
  'failures',
  'knownLimitations',
]);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function nonEmptyString(value) {
  return typeof value === 'string' && value.length > 0;
}

function stringCompare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function rejectUnknownFields(value, allowed, location, errors) {
  for (const key of Object.keys(value)) {
    if (!allowed.includes(key)) errors.push(`${location}: unexpected field ${key}`);
  }
}

function requireFields(value, required, location, errors) {
  for (const field of required) {
    if (!Object.hasOwn(value, field)) errors.push(`${location}.${field} is required`);
  }
}

function assertJsonValue(value, location) {
  if (value === null || typeof value === 'string' || typeof value === 'boolean') return;
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) throw new Error(`${location} must contain finite JSON numbers`);
    return;
  }
  if (Array.isArray(value)) {
    value.forEach((entry, index) => assertJsonValue(entry, `${location}[${index}]`));
    return;
  }
  if (isObject(value)) {
    for (const [key, entry] of Object.entries(value)) {
      assertJsonValue(entry, `${location}.${key}`);
    }
    return;
  }
  throw new Error(`${location} must contain JSON values only`);
}

function validateIdentity(identity, sourceOwner) {
  if (!isObject(identity)) throw new Error('candidate identity must be an object');
  const errors = [];
  rejectUnknownFields(identity, IDENTITY_FIELDS, 'identity', errors);
  requireFields(identity, IDENTITY_FIELDS, 'identity', errors);
  if (!/^[a-z0-9-]+$/.test(sourceOwner ?? '')) {
    errors.push('sourceOwner must be a lowercase stable owner ID');
  }
  if (!nonEmptyString(identity.sourceBoundaryId)
      || !identity.sourceBoundaryId.startsWith(`${sourceOwner}.`)) {
    errors.push('identity.sourceBoundaryId must belong to sourceOwner');
  }
  if (!nonEmptyString(identity.candidateKind)) {
    errors.push('identity.candidateKind must not be empty');
  }
  if (identity.sourceFace !== null && !nonEmptyString(identity.sourceFace)) {
    errors.push('identity.sourceFace must be null or a non-empty string');
  }
  if (!nonEmptyString(identity.targetOrPolicy)) {
    errors.push('identity.targetOrPolicy must not be empty');
  }
  if (!isObject(identity.conditions)) errors.push('identity.conditions must be an object');
  if (identity.order !== null
      && (!Number.isSafeInteger(identity.order) || identity.order < 0)) {
    errors.push('identity.order must be null or a non-negative safe integer');
  }
  try {
    assertJsonValue(identity, 'identity');
  } catch (error) {
    errors.push(error.message);
  }
  if (errors.length > 0) throw new Error(errors.join('\n'));
}

export function buildCandidateLink(identity, sourceOwner) {
  validateIdentity(identity, sourceOwner);
  const suffix = sha256Text(canonicalJson(identity)).slice(0, 20);
  const candidateId = `candidate.${suffix}`;
  return {
    candidateId,
    ruleId: `rule.${sourceOwner}.${suffix}`,
    evidenceAnchor: `${EVIDENCE_PREFIX}${candidateId}`,
    sourceOwner,
  };
}

export function attachLedgerEvidence(identity, sourceOwner, ledger) {
  const link = buildCandidateLink(identity, sourceOwner);
  const rule = Array.isArray(ledger?.rules)
    ? ledger.rules.find(candidate => candidate?.ruleId === link.ruleId)
    : undefined;
  const hasExactAnchor = rule?.evidence?.some(evidence => (
    evidence?.kind === 'document' && evidence.reference === link.evidenceAnchor
  ));
  if (rule?.sourceOwner !== sourceOwner || !hasExactAnchor) {
    return {
      ...link,
      ruleId: null,
      relationType: null,
      evidenceStatus: null,
      knownLimitations: [],
      reason: 'ledgerRuleMissing',
    };
  }
  return {
    ...link,
    relationType: rule.relationType,
    evidenceStatus: rule.evidenceStatus,
    knownLimitations: [...rule.knownLimitations],
    reason: null,
  };
}

export function normalizeTraceLimits(limits) {
  if (limits === undefined) return { maxCharacters: DEFAULT_CHARACTER_LIMIT };
  if (!isObject(limits)) throw new Error('limits must be an object');
  const keys = Object.keys(limits);
  if (keys.length !== 1 || keys[0] !== 'maxCharacters') {
    throw new Error('limits accepts only maxCharacters');
  }
  const value = limits.maxCharacters;
  if (!Number.isSafeInteger(value) || value < 1 || value > MAX_CHARACTER_LIMIT) {
    throw new Error(`maxCharacters must be an integer in 1..${MAX_CHARACTER_LIMIT}`);
  }
  return { maxCharacters: value };
}

function normalizeForHash(value, parentKey = null) {
  if (Array.isArray(value)) {
    const normalized = value.map(entry => normalizeForHash(entry));
    if (UNORDERED_STRING_ARRAY_FIELDS.has(parentKey)
        && normalized.every(entry => typeof entry === 'string')) {
      return [...new Set(normalized)].sort(stringCompare);
    }
    return normalized;
  }
  if (!isObject(value)) return value;
  return Object.fromEntries(
    Object.keys(value)
      .filter(key => !VOLATILE_HASH_FIELDS.has(key))
      .sort()
      .map(key => [key, normalizeForHash(value[key], key)]),
  );
}

function portableLayoutProjection(trace) {
  return normalizeForHash({
    schemaVersion: trace.schemaVersion,
    scope: trace.scope,
    counts: trace.counts,
    records: Array.isArray(trace.records)
      ? trace.records.map(record => ({
        recordId: record.recordId,
        source: record.source,
        document: record.document,
        layoutName: record.layoutName,
        layoutMetric: record.layoutMetric,
        provenance: record.provenance,
      }))
      : trace.records,
  });
}

export function portableLayoutHash(trace) {
  return sha256Text(canonicalJson(portableLayoutProjection(trace)));
}

export function normalizedTraceHash(trace) {
  return sha256Text(canonicalJson(normalizeForHash(trace)));
}

function walkStrings(value, location, visit) {
  if (typeof value === 'string') {
    visit(value, location);
    return;
  }
  if (Array.isArray(value)) {
    value.forEach((entry, index) => walkStrings(entry, `${location}[${index}]`, visit));
    return;
  }
  if (isObject(value)) {
    for (const [key, entry] of Object.entries(value)) {
      walkStrings(entry, `${location}.${key}`, visit);
    }
  }
}

export function findSensitiveTraceValues(trace) {
  const findings = [];
  walkStrings(trace, '$', (value, location) => {
    const tests = [
      ['absoluteHomePath', /(?:^|[\s"'])(?:\/home\/[^/\s]+\/|\/Users\/[^/\s]+\/|[A-Za-z]:\\Users\\[^\\\s]+\\)/],
      ['accessToken', /(?:Bearer\s+[A-Za-z0-9._-]{16,}|gh[pousr]_[A-Za-z0-9]{20,})/],
      ['errorStack', /\n\s*at\s+(?:\S+\s+)?\(?[^\n]+:\d+:\d+\)?/],
    ];
    for (const [reason, pattern] of tests) {
      if (pattern.test(value)) findings.push({ location, reason });
    }
  });
  return findings;
}

function validateHash(value, field, trace, compute, errors) {
  if (!isObject(value)) {
    errors.push(`${field} must be an object`);
    return;
  }
  if (value.algorithm !== 'sha256') errors.push(`${field}.algorithm must be sha256`);
  if (value.value === null) {
    const hasUnavailable = trace.reasons?.some(reason => reason?.code === 'hashUnavailable');
    if (!hasUnavailable) errors.push(`${field}.value is null without hashUnavailable`);
  } else if (!HASH_PATTERN.test(value.value ?? '')) {
    errors.push(`${field}.value must be a lowercase SHA-256 digest or null`);
  } else if (value.value !== compute(trace)) {
    errors.push(`${field}.value does not match canonical projection`);
  }
}

function validateBackendDecision(value, location, errors) {
  if (!isObject(value)) {
    errors.push(`${location} must be an object`);
    return;
  }
  if (!BACKEND_STATUSES.has(value.status)) errors.push(`${location}.status is invalid`);
  if (!CERTAINTIES.has(value.certainty)) errors.push(`${location}.certainty is invalid`);
  for (const field of ['candidates', 'capabilities', 'failures']) {
    if (!Array.isArray(value[field])) errors.push(`${location}.${field} must be an array`);
    else if (value[field].length > 64) errors.push(`${location}.${field} exceeds 64 items`);
  }
  if (value.certainty === 'observed' && value.resolved === null) {
    errors.push(`${location}.observed requires a resolved face`);
  }
}

function validateRecord(record, index, errors) {
  const location = `records[${index}]`;
  if (!isObject(record)) {
    errors.push(`${location} must be an object`);
    return;
  }
  rejectUnknownFields(record, RECORD_FIELDS, location, errors);
  requireFields(record, RECORD_FIELDS, location, errors);
  if (!nonEmptyString(record.recordId)) errors.push(`${location}.recordId must not be empty`);
  if (!isObject(record.source) || !nonEmptyString(record.source.character)) {
    errors.push(`${location}.source.character must not be empty`);
  }
  if (!isObject(record.document)) errors.push(`${location}.document must be an object`);
  if (!isObject(record.layoutName)) errors.push(`${location}.layoutName must be an object`);
  if (!isObject(record.layoutMetric)) errors.push(`${location}.layoutMetric must be an object`);
  if (!isObject(record.paint)) {
    errors.push(`${location}.paint must be an object`);
  } else {
    for (const backend of ['native', 'canvas2d', 'canvaskit']) {
      validateBackendDecision(record.paint[backend], `${location}.paint.${backend}`, errors);
    }
  }
  if (!Array.isArray(record.provenance)) {
    errors.push(`${location}.provenance must be an array`);
  } else {
    if (record.provenance.length > 64) errors.push(`${location}.provenance exceeds 64 items`);
    record.provenance.forEach((entry, provenanceIndex) => {
      const provenanceLocation = `${location}.provenance[${provenanceIndex}]`;
      if (!isObject(entry) || !CANDIDATE_PATTERN.test(entry.candidateId ?? '')) {
        errors.push(`${provenanceLocation}.candidateId is invalid`);
        return;
      }
      if (entry.ruleId !== null && !RULE_PATTERN.test(entry.ruleId ?? '')) {
        errors.push(`${provenanceLocation}.ruleId is invalid`);
      }
      if (entry.ruleId === null && entry.reason !== 'ledgerRuleMissing') {
        errors.push(`${provenanceLocation}.ruleId null requires ledgerRuleMissing`);
      }
      if (!entry.evidenceAnchor?.endsWith(`#${entry.candidateId}`)) {
        errors.push(`${provenanceLocation}.evidenceAnchor does not match candidateId`);
      }
    });
  }
  if (!isObject(record.oracle)) errors.push(`${location}.oracle must be an object`);
}

export function validateTraceEnvelope(trace) {
  const errors = [];
  if (!isObject(trace)) return ['trace must be an object'];
  rejectUnknownFields(trace, TRACE_FIELDS, 'trace', errors);
  requireFields(trace, TRACE_FIELDS, 'trace', errors);
  if (trace.schemaVersion !== 1) errors.push('schemaVersion must be 1');
  if (!TRACE_STATUSES.has(trace.status)) errors.push('status is required and must be valid');

  if (!isObject(trace.scope)) {
    errors.push('scope must be an object');
  } else {
    if (!Number.isSafeInteger(trace.scope.pageIndex) || trace.scope.pageIndex < 0) {
      errors.push('scope.pageIndex must be a non-negative safe integer');
    }
    try {
      const requested = normalizeTraceLimits(trace.scope.requestedLimits);
      const applied = normalizeTraceLimits(trace.scope.appliedLimits);
      if (requested.maxCharacters !== applied.maxCharacters) {
        errors.push('scope.appliedLimits must equal requestedLimits; silent clamp is forbidden');
      }
    } catch (error) {
      errors.push(`scope.${error.message}`);
    }
  }

  if (!Array.isArray(trace.records)) {
    errors.push('records must be an array');
  } else {
    if (trace.records.length > MAX_CHARACTER_LIMIT) {
      errors.push(`records exceeds ${MAX_CHARACTER_LIMIT}`);
    }
    trace.records.forEach((record, index) => validateRecord(record, index, errors));
  }

  if (!isObject(trace.counts)) {
    errors.push('counts must be an object');
  } else {
    for (const field of ['runsSeen', 'charactersSeen', 'recordsEmitted']) {
      if (!Number.isSafeInteger(trace.counts[field]) || trace.counts[field] < 0) {
        errors.push(`counts.${field} must be a non-negative safe integer`);
      }
    }
    if (trace.counts.recordsEmitted !== trace.records?.length) {
      errors.push('counts.recordsEmitted must equal records.length');
    }
    if (trace.counts.charactersSeen < trace.counts.recordsEmitted) {
      errors.push('counts.charactersSeen must be at least recordsEmitted');
    }
    if (trace.counts.recordsOmitted !== null
        && (!Number.isSafeInteger(trace.counts.recordsOmitted)
          || trace.counts.recordsOmitted < 0)) {
      errors.push('counts.recordsOmitted must be null or a non-negative safe integer');
    }
    if (trace.status === 'complete' && trace.counts.recordsOmitted !== 0) {
      errors.push('complete trace must have recordsOmitted 0');
    }
  }

  if (!Array.isArray(trace.reasons)) {
    errors.push('reasons must be an array');
  } else {
    trace.reasons.forEach((reason, index) => {
      if (!isObject(reason) || !REASON_CODES.has(reason.code)) {
        errors.push(`reasons[${index}].code is invalid`);
      }
    });
  }
  if (trace.status === 'truncated'
      && !trace.reasons?.some(reason => reason?.code === 'characterLimitExceeded')) {
    errors.push('truncated trace requires characterLimitExceeded');
  }
  if (trace.counts?.recordsOmitted === null
      && !trace.reasons?.some(reason => reason?.code === 'recordsOmittedUnknown')) {
    errors.push('recordsOmitted null requires recordsOmittedUnknown');
  }

  if (!isObject(trace.backendSummary)) {
    errors.push('backendSummary must be an object');
  } else {
    for (const backend of ['layout', 'native', 'canvas2d', 'canvaskit']) {
      const summary = trace.backendSummary[backend];
      if (!isObject(summary) || !BACKEND_STATUSES.has(summary.status)) {
        errors.push(`backendSummary.${backend}.status is invalid`);
      }
      if (!Array.isArray(summary?.reasons)) {
        errors.push(`backendSummary.${backend}.reasons must be an array`);
      }
    }
  }

  validateHash(trace.layoutHash, 'layoutHash', trace, portableLayoutHash, errors);
  validateHash(trace.normalizedHash, 'normalizedHash', trace, normalizedTraceHash, errors);
  for (const finding of findSensitiveTraceValues(trace)) {
    errors.push(`sensitive trace value at ${finding.location}: ${finding.reason}`);
  }
  return errors;
}

function sha256File(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

export function detectLedgerSourceDrift(snapshot, repositoryRoot = REPOSITORY_ROOT) {
  if (!isObject(snapshot) || !Array.isArray(snapshot.candidates)) {
    throw new Error('candidate snapshot must contain candidates');
  }
  const expectedByPath = new Map();
  for (const candidate of snapshot.candidates) {
    if (!nonEmptyString(candidate?.path) || !HASH_PATTERN.test(candidate?.sourceSha256 ?? '')) {
      throw new Error('candidate snapshot contains an invalid source boundary');
    }
    const previous = expectedByPath.get(candidate.path);
    if (previous !== undefined && previous !== candidate.sourceSha256) {
      throw new Error(`candidate snapshot has conflicting digests for ${candidate.path}`);
    }
    expectedByPath.set(candidate.path, candidate.sourceSha256);
  }
  return [...expectedByPath.entries()]
    .sort(([left], [right]) => stringCompare(left, right))
    .flatMap(([sourcePath, expectedSha256]) => {
      const unsafe = path.isAbsolute(sourcePath)
        || sourcePath.split(/[\\/]/).includes('..');
      if (unsafe) throw new Error(`candidate source path must be repository-relative: ${sourcePath}`);
      const absolute = path.resolve(repositoryRoot, sourcePath);
      const actualSha256 = fs.existsSync(absolute) ? sha256File(absolute) : null;
      return actualSha256 === expectedSha256
        ? []
        : [{ path: sourcePath, expectedSha256, actualSha256 }];
    });
}

export function validatePublicFixtures(manifest, repositoryRoot = REPOSITORY_ROOT) {
  const errors = [];
  if (!isObject(manifest) || manifest.schemaVersion !== 1 || manifest.issue !== 4961) {
    return ['public fixture manifest header is invalid'];
  }
  if (!Array.isArray(manifest.fixtures) || manifest.fixtures.length === 0) {
    return ['public fixture manifest must not be empty'];
  }
  for (const [index, fixture] of manifest.fixtures.entries()) {
    const location = `fixtures[${index}]`;
    if (!isObject(fixture) || !nonEmptyString(fixture.path)) {
      errors.push(`${location}.path must not be empty`);
      continue;
    }
    const unsafe = path.isAbsolute(fixture.path)
      || /^[A-Za-z]:[\\/]/.test(fixture.path)
      || fixture.path.split(/[\\/]/).includes('..')
      || !fixture.path.startsWith('samples/')
      || fixture.path.includes('hwpsamples')
      || fixture.path.includes('corpus_10k');
    if (unsafe) {
      errors.push(`${location}.path must be a repository-relative tracked samples path`);
      continue;
    }
    const absolute = path.resolve(repositoryRoot, fixture.path);
    if (!fs.existsSync(absolute)) {
      errors.push(`${location}.path does not exist`);
      continue;
    }
    try {
      execFileSync('git', ['ls-files', '--error-unmatch', '--', fixture.path], {
        cwd: repositoryRoot,
        stdio: 'ignore',
      });
    } catch {
      errors.push(`${location}.path is not tracked by git`);
    }
    if (sha256File(absolute) !== fixture.sha256) errors.push(`${location}.sha256 mismatch`);
    if (fs.statSync(absolute).size !== fixture.size) errors.push(`${location}.size mismatch`);
    if (fixture.privateCorpus !== false) errors.push(`${location}.privateCorpus must be false`);
    if (fixture.visibility !== 'repository-tracked') {
      errors.push(`${location}.visibility must be repository-tracked`);
    }
    if (!['hwp', 'hwpx'].includes(fixture.format)
        || !fixture.path.endsWith(`.${fixture.format}`)) {
      errors.push(`${location}.format does not match path`);
    }
  }
  const formats = new Set(manifest.fixtures.map(fixture => fixture.format));
  if (!formats.has('hwp') || !formats.has('hwpx')) {
    errors.push('public fixture manifest must contain both HWP and HWPX');
  }
  return errors;
}

function checkRepositoryContracts() {
  const vectors = readJson(path.join(INVESTIGATION, 'font_decision_identity_vectors.json'));
  const ledger = readJson(LEDGER_PATH);
  const errors = [];
  for (const vector of vectors.vectors ?? []) {
    try {
      const link = buildCandidateLink(vector.identity, vector.sourceOwner);
      if (link.candidateId !== vector.candidateId) {
        errors.push(`${vector.id}: candidateId mismatch`);
      }
      if (link.ruleId !== vector.ruleId) errors.push(`${vector.id}: ruleId mismatch`);
      const evidence = attachLedgerEvidence(vector.identity, vector.sourceOwner, ledger);
      if (evidence.reason !== null) errors.push(`${vector.id}: ${evidence.reason}`);
    } catch (error) {
      errors.push(`${vector.id}: ${error.message}`);
    }
  }
  errors.push(...validatePublicFixtures(
    readJson(path.join(INVESTIGATION, 'public_fixtures.json')),
    REPOSITORY_ROOT,
  ));
  return errors;
}

function main() {
  const [command] = process.argv.slice(2);
  if (command !== 'check') {
    console.error('usage: node scripts/font_decision_trace_contract.mjs check');
    process.exitCode = 2;
    return;
  }
  const errors = checkRepositoryContracts();
  if (errors.length > 0) {
    errors.forEach(error => console.error(error));
    process.exitCode = 1;
    return;
  }
  console.log('font decision trace Stage 1 contracts: ok');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
