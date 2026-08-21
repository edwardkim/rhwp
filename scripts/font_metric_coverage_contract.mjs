#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { collectRuleCandidates } from './font_rule_candidates.mjs';
import {
  canonicalJson,
  collectSourceCandidates,
  sha256Text,
} from './font_rule_ledger.mjs';

const REPOSITORY_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const INVESTIGATION = path.join(
  REPOSITORY_ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
);
const CONTRACT_PATH = path.join(INVESTIGATION, 'font_metric_coverage_contract.json');
const W1_INVESTIGATION = path.join(
  REPOSITORY_ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4939',
);

const CATEGORY_IDS = [
  'measured-overlay',
  'identity-alias-hit',
  'metric-surrogate',
  'exact-hit',
  'char-miss',
  'face-miss',
  'heuristic',
];
const VOLATILE_HASH_FIELDS = new Set([
  'aggregateHash',
  'durationMillis',
  'elapsedMillis',
  'generatedAt',
  'timestamp',
]);
const POC_DISTRIBUTIONS = [
  'alignmentDistribution',
  'baseSizeHwpunitDistribution',
  'contextDistribution',
  'ratioDistribution',
  'spacingDistribution',
];

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function safeCount(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function uniqueStrings(values) {
  return Array.isArray(values)
    && values.length > 0
    && values.every(value => typeof value === 'string' && value.length > 0)
    && new Set(values).size === values.length;
}

function sum(rows, field) {
  if (!Array.isArray(rows)) throw new Error(`${field} rows must be an array`);
  return rows.reduce((total, row, index) => {
    const value = row?.[field];
    if (!safeCount(value)) throw new Error(`${field} at row ${index} must be a non-negative safe integer`);
    return total + value;
  }, 0);
}

export function validateCoverageContract(contract) {
  const errors = [];
  if (!isObject(contract)) return ['contract must be an object'];
  if (contract.schemaVersion !== 1) errors.push('schemaVersion must be 1');
  if (contract.kind !== 'font-metric-coverage-contract') {
    errors.push('kind must be font-metric-coverage-contract');
  }
  if (contract.issue !== 4962) errors.push('issue must be 4962');

  const decisionInputs = {
    matchKinds: ['boldOnly', 'exact', 'nameFirst', 'none'],
    characterMatches: ['hit', 'miss', 'notApplicable'],
  };
  for (const [field, expected] of Object.entries(decisionInputs)) {
    const actual = [...(contract.decisionInputs?.[field] ?? [])].sort(compareText);
    if (canonicalJson(actual) !== canonicalJson(expected)) {
      errors.push(`decisionInputs.${field} does not match the W2 v1 inventory`);
    }
  }

  const actualCategories = Array.isArray(contract.categories)
    ? contract.categories.map(entry => entry?.id)
    : [];
  if (canonicalJson(actualCategories) !== canonicalJson(CATEGORY_IDS)) {
    errors.push('categories must preserve the seven-category priority order');
  }
  contract.categories?.forEach((entry, index) => {
    if (entry?.priority !== index + 1) {
      errors.push(`categories[${index}].priority must be ${index + 1}`);
    }
  });

  const widthSourceGroups = [
    'measuredOverlay',
    'metricHit',
    'heuristicFallback',
    'policyHeuristic',
    'notApplicable',
    'internalOnly',
  ];
  const seenSources = new Map();
  for (const group of widthSourceGroups) {
    const values = contract.widthSources?.[group];
    if (!uniqueStrings(values)) {
      errors.push(`widthSources.${group} must be a non-empty unique string array`);
      continue;
    }
    for (const value of values) {
      if (seenSources.has(value)) {
        errors.push(`widthSource ${value} appears in both ${seenSources.get(value)} and ${group}`);
      } else {
        seenSources.set(value, group);
      }
    }
  }

  if (!uniqueStrings(contract.reusedPocFields)) {
    errors.push('reusedPocFields must be a non-empty unique string array');
  }
  if (!uniqueStrings(contract.deltaFields)) {
    errors.push('deltaFields must be a non-empty unique string array');
  }
  const delta = new Set(contract.deltaFields ?? []);
  for (const field of contract.reusedPocFields ?? []) {
    if (delta.has(field)) errors.push(`field ${field} appears in both reuse and delta contracts`);
  }

  if (!uniqueStrings(contract.privacy?.forbiddenKeys)) {
    errors.push('privacy.forbiddenKeys must be a non-empty unique string array');
  }
  if (!uniqueStrings(contract.privacy?.forbiddenStringPatterns)) {
    errors.push('privacy.forbiddenStringPatterns must be a non-empty unique string array');
  }
  for (const denominator of ['layout', 'coverage', 'join', 'documents', 'backend']) {
    if (typeof contract.denominators?.[denominator] !== 'string'
        || contract.denominators[denominator].length === 0) {
      errors.push(`denominators.${denominator} must be documented`);
    }
  }
  const collector = contract.collectorRequirements;
  if (collector?.traversal !== 'streaming'
      || collector.pageCharacterLimit !== null
      || collector.rawRecordsPersisted !== false
      || collector.longPageTruncation !== 'forbidden') {
    errors.push('collectorRequirements must forbid limits, truncation and raw record persistence');
  }
  for (const [field, expected] of Object.entries({
    sourceJoinStates: ['excluded', 'joined', 'layoutOnly'],
    documentFailureStates: [
      'cancelled',
      'drm',
      'empty',
      'encrypted',
      'parser',
      'resource-limit',
      'unsupported',
    ],
    backendStates: ['complete', 'failed', 'notObserved', 'unsupported'],
  })) {
    const actual = [...(collector?.[field] ?? [])].sort(compareText);
    if (canonicalJson(actual) !== canonicalJson(expected)) {
      errors.push(`collectorRequirements.${field} does not match the contract inventory`);
    }
  }

  const resource = contract.resourcePolicy;
  if (resource?.failureMode !== 'explicit-document-failure'
      || resource.partialAggregateAccepted !== false
      || resource.deadlineChecks !== true
      || resource.cancellationChecks !== true
      || resource.workUnitBudget !== true
      || resource.aggregateRowBudget !== true
      || resource.outputByteBudget !== true
      || resource.corpusWorkerIsolation !== 'required') {
    errors.push('resourcePolicy must fail closed with bounded work, rows, output and isolation');
  }

  const poc = contract.existingPoc;
  if (!isObject(poc) || !/^[0-9a-f]{64}$/.test(poc.projectionSha256 ?? '')) {
    errors.push('existingPoc.projectionSha256 must be a lowercase SHA-256 digest');
  } else {
    if (poc.corpus.hwp + poc.corpus.hwpx !== poc.corpus.discovered) {
      errors.push('existingPoc format counts must equal discovered documents');
    }
    if (poc.corpus.success + poc.corpus.failure !== poc.corpus.attempted) {
      errors.push('existingPoc success and failure must equal attempted documents');
    }
    if (poc.totals.metricMappedChars + poc.totals.metricUnmappedChars !== poc.totals.chars) {
      errors.push('existingPoc mapped and unmapped characters must equal total characters');
    }
  }

  const w1 = contract.w1Baseline;
  for (const field of [
    'boundaryCount',
    'candidateCount',
    'ledgerRuleCount',
    'identityAliasRuleCount',
    'metricSurrogateRuleCount',
    'measuredOverlayRuleCount',
    'metricEntryCount',
  ]) {
    if (!safeCount(w1?.[field])) errors.push(`w1Baseline.${field} must be a safe count`);
  }
  return errors;
}

function sourceSet(contract, group) {
  return new Set(contract.widthSources[group]);
}

function provenanceRelations(record) {
  if (!Array.isArray(record.provenance)) throw new Error('provenance must be an array');
  return record.provenance.filter(isObject);
}

export function classifyCoverageDecision(record, contract) {
  if (!isObject(record)) throw new Error('coverage decision must be an object');
  if (validateCoverageContract(contract).length > 0) throw new Error('coverage contract is invalid');
  const widthSource = record.widthSource;
  if (typeof widthSource !== 'string' || widthSource.length === 0) {
    throw new Error('widthSource must be a non-empty string');
  }
  if (!['hit', 'miss', 'notApplicable'].includes(record.characterMatch)) {
    throw new Error('characterMatch must be hit, miss or notApplicable');
  }
  if (!contract.decisionInputs.matchKinds.includes(record.matchKind)) {
    throw new Error(`unclassified matchKind: ${record.matchKind}`);
  }
  if (record.metricEntry !== null
      && (!Number.isSafeInteger(record.metricEntry) || record.metricEntry < 0)) {
    throw new Error('metricEntry must be null or a non-negative safe integer');
  }

  const groups = Object.fromEntries(
    Object.keys(contract.widthSources).map(group => [group, sourceSet(contract, group)]),
  );
  const known = Object.values(groups).some(values => values.has(widthSource));
  if (!known) throw new Error(`unclassified widthSource: ${widthSource}`);
  if (groups.internalOnly.has(widthSource)) {
    throw new Error(`internal-only widthSource reached aggregate: ${widthSource}`);
  }

  const relations = provenanceRelations(record);
  const identity = relations.filter(entry => (
    entry.relationType === contract.provenance.identityAlias.relationType
  ));
  if (identity.some(entry => (
    !contract.provenance.identityAlias.verifiedEvidenceStatuses.includes(entry.evidenceStatus)
  ))) {
    throw new Error('identity-alias requires verified evidence');
  }
  const hasIdentity = identity.length > 0;
  const hasSurrogate = relations.some(entry => (
    entry.relationType === contract.provenance.metricSurrogate.relationType
  ));
  if (hasIdentity && hasSurrogate) {
    throw new Error('identity-alias and metric-surrogate provenance conflict');
  }
  if (record.characterMatch === 'miss' && record.metricEntry === null) {
    throw new Error('character miss requires a metric entry');
  }

  if (groups.notApplicable.has(widthSource)) {
    if (record.characterMatch !== 'notApplicable' || record.metricEntry !== null) {
      throw new Error(`non-applicable widthSource has metric state: ${widthSource}`);
    }
    return { status: 'not-applicable', reason: widthSource };
  }
  if (groups.measuredOverlay.has(widthSource)) {
    if (record.characterMatch !== 'hit') {
      throw new Error(`measured overlay must be a character hit: ${widthSource}`);
    }
    return { status: 'classified', category: 'measured-overlay' };
  }
  if (hasIdentity) {
    if (record.characterMatch !== 'hit' || record.metricEntry === null) {
      throw new Error('identity-alias-hit requires a metric character hit');
    }
    return { status: 'classified', category: 'identity-alias-hit' };
  }
  if (hasSurrogate) {
    if (record.characterMatch !== 'hit' || record.metricEntry === null) {
      throw new Error('metric-surrogate requires a metric character hit');
    }
    return { status: 'classified', category: 'metric-surrogate' };
  }
  if (groups.metricHit.has(widthSource)) {
    if (record.characterMatch !== 'hit' || record.metricEntry === null) {
      throw new Error(`metric widthSource requires a metric character hit: ${widthSource}`);
    }
    return { status: 'classified', category: 'exact-hit' };
  }
  if (groups.heuristicFallback.has(widthSource)) {
    if (record.metricEntry !== null) {
      if (record.characterMatch !== 'miss') {
        throw new Error(`metric fallback requires characterMatch miss: ${widthSource}`);
      }
      return { status: 'classified', category: 'char-miss' };
    }
    if (record.characterMatch !== 'notApplicable') {
      throw new Error(`face miss requires characterMatch notApplicable: ${widthSource}`);
    }
    return { status: 'classified', category: 'face-miss' };
  }
  if (groups.policyHeuristic.has(widthSource)) {
    if (record.metricEntry !== null || record.characterMatch !== 'notApplicable') {
      throw new Error(`policy heuristic has contradictory metric state: ${widthSource}`);
    }
    return { status: 'classified', category: 'heuristic' };
  }
  throw new Error(`unclassified widthSource: ${widthSource}`);
}

function countField(value, location, errors) {
  if (!safeCount(value)) {
    errors.push(`${location} must be a non-negative safe integer`);
    return 0;
  }
  return value;
}

export function reconcileCoverageAggregate(aggregate, contract) {
  const errors = [];
  if (!isObject(aggregate)) return ['aggregate must be an object'];
  const counts = aggregate.counts ?? {};
  const layout = countField(counts.layoutCharacters, 'counts.layoutCharacters', errors);
  const coverage = countField(counts.coverageCharacters, 'counts.coverageCharacters', errors);
  const notApplicable = countField(
    counts.notApplicableCharacters,
    'counts.notApplicableCharacters',
    errors,
  );
  const excluded = countField(counts.excludedCharacters, 'counts.excludedCharacters', errors);
  const truncated = countField(
    counts.truncatedCharacters,
    'counts.truncatedCharacters',
    errors,
  );
  if (truncated !== 0) errors.push('long-page truncation is forbidden');
  if (layout !== coverage + notApplicable + excluded) {
    errors.push('layout denominator does not reconcile');
  }

  const categoryKeys = Object.keys(aggregate.categories ?? {}).sort(compareText);
  const expectedKeys = contract.categories.map(entry => entry.id).sort(compareText);
  if (canonicalJson(categoryKeys) !== canonicalJson(expectedKeys)) {
    errors.push('categories must contain exactly the seven contract keys');
  }
  const categorySum = expectedKeys.reduce(
    (total, category) => total + countField(
      aggregate.categories?.[category],
      `categories.${category}`,
      errors,
    ),
    0,
  );
  if (categorySum !== coverage) errors.push('category sum does not equal coverageCharacters');

  const expectedJoinKeys = [...contract.collectorRequirements.sourceJoinStates].sort(compareText);
  const actualJoinKeys = Object.keys(aggregate.joins ?? {}).sort(compareText);
  if (canonicalJson(actualJoinKeys) !== canonicalJson(expectedJoinKeys)) {
    errors.push('join states must contain exactly the contract keys');
  }
  const joinSum = expectedJoinKeys.reduce(
    (total, state) => total + countField(aggregate.joins?.[state], `joins.${state}`, errors),
    0,
  );
  if (joinSum !== layout) errors.push('join state sum does not equal layoutCharacters');

  const expectedFailureKeys = [...contract.collectorRequirements.documentFailureStates]
    .sort(compareText);
  const actualFailureKeys = Object.keys(aggregate.documents?.failures ?? {}).sort(compareText);
  if (canonicalJson(actualFailureKeys) !== canonicalJson(expectedFailureKeys)) {
    errors.push('document failure states must contain exactly the contract keys');
  }
  const failureSum = expectedFailureKeys.reduce(
    (total, reason) => total + countField(
      aggregate.documents?.failures?.[reason],
      `documents.failures.${reason}`,
      errors,
    ),
    0,
  );
  const attempted = countField(aggregate.documents?.attempted, 'documents.attempted', errors);
  const success = countField(aggregate.documents?.success, 'documents.success', errors);
  if (attempted !== success + failureSum) errors.push('document outcome sum does not equal attempted');

  const requested = countField(aggregate.backends?.requested, 'backends.requested', errors);
  const expectedBackendKeys = [
    'requested',
    ...contract.collectorRequirements.backendStates,
  ].sort(compareText);
  const actualBackendKeys = Object.keys(aggregate.backends ?? {}).sort(compareText);
  if (canonicalJson(actualBackendKeys) !== canonicalJson(expectedBackendKeys)) {
    errors.push('backend states must contain exactly the contract keys');
  }
  const backendSum = contract.collectorRequirements.backendStates.reduce(
    (total, state) => total + countField(aggregate.backends?.[state], `backends.${state}`, errors),
    0,
  );
  if (requested !== backendSum) errors.push('backend state sum does not equal requested');
  return errors;
}

function normalizeForHash(value) {
  if (Array.isArray(value)) return value.map(normalizeForHash);
  if (!isObject(value)) return value;
  return Object.fromEntries(
    Object.keys(value)
      .filter(key => !VOLATILE_HASH_FIELDS.has(key))
      .sort(compareText)
      .map(key => [key, normalizeForHash(value[key])]),
  );
}

export function canonicalCoverageHash(aggregate) {
  return sha256Text(canonicalJson(normalizeForHash(aggregate)));
}

function walkAggregate(value, location, forbiddenKeys, findings) {
  if (typeof value === 'string') {
    const patterns = [
      ['absoluteHomePath', /(?:^|[\s"'])(?:\/home\/[^/\s]+\/|\/Users\/[^/\s]+\/|[A-Za-z]:\\Users\\[^\\\s]+\\)/],
      ['accessToken', /(?:Bearer\s+[A-Za-z0-9._-]{16,}|gh[pousr]_[A-Za-z0-9]{20,})/],
      ['errorStack', /\n\s*at\s+(?:\S+\s+)?\(?[^\n]+:\d+:\d+\)?/],
    ];
    for (const [reason, pattern] of patterns) {
      if (pattern.test(value)) findings.push({ location, reason });
    }
    return;
  }
  if (Array.isArray(value)) {
    value.forEach((entry, index) => (
      walkAggregate(entry, `${location}[${index}]`, forbiddenKeys, findings)
    ));
    return;
  }
  if (!isObject(value)) return;
  for (const [key, entry] of Object.entries(value)) {
    const next = `${location}.${key}`;
    if (forbiddenKeys.has(key)) findings.push({ location: next, reason: 'forbiddenKey' });
    walkAggregate(entry, next, forbiddenKeys, findings);
  }
}

export function findSensitiveAggregateValues(aggregate, contract) {
  const findings = [];
  walkAggregate(aggregate, '$', new Set(contract.privacy.forbiddenKeys), findings);
  return findings;
}

export function pocV2Projection(summary) {
  if (!isObject(summary)) throw new Error('POC summary must be an object');
  for (const field of ['fonts', 'usage', 'riskDocuments', ...POC_DISTRIBUTIONS]) {
    if (!Array.isArray(summary[field])) throw new Error(`POC summary ${field} must be an array`);
  }
  const mappedChars = summary.usage
    .filter(row => row.metricFace !== null)
    .reduce((total, row) => total + row.charCount, 0);
  return {
    schemaVersion: summary.schemaVersion,
    repositoryHead: summary.repositoryHead,
    corpus: summary.corpus,
    totals: summary.totals,
    fontCount: summary.fonts.length,
    usageRowCount: summary.usage.length,
    riskDocumentCount: summary.riskDocuments.length,
    fontCharCount: sum(summary.fonts, 'charCount'),
    usageCharCount: sum(summary.usage, 'charCount'),
    distributionCharTotals: Object.fromEntries(
      POC_DISTRIBUTIONS.map(name => [name, sum(summary[name], 'chars')]),
    ),
    metricMappedChars: mappedChars,
    metricUnmappedChars: summary.totals.chars - mappedChars,
    fontRowKeys: Object.keys(summary.fonts[0] ?? {}).sort(compareText),
    usageRowKeys: Object.keys(summary.usage[0] ?? {}).sort(compareText),
    riskRowKeys: Object.keys(summary.riskDocuments[0] ?? {}).sort(compareText),
  };
}

export function validatePocV2Baseline(summary, contract) {
  const errors = [];
  let projection;
  try {
    projection = pocV2Projection(summary);
  } catch (error) {
    return [error.message];
  }
  const expected = contract.existingPoc;
  if (projection.schemaVersion !== 'poc-font-layout-habits-v2') {
    errors.push('POC schemaVersion is not v2');
  }
  if (projection.repositoryHead !== expected.outputRepositoryHead) {
    errors.push('POC repositoryHead differs from the frozen baseline');
  }
  if (projection.corpus.hwp + projection.corpus.hwpx !== projection.corpus.discovered) {
    errors.push('POC format counts do not equal discovered documents');
  }
  const failureSum = Object.values(projection.corpus.failureCategories ?? {})
    .reduce((total, value) => total + value, 0);
  if (failureSum !== projection.corpus.failure) {
    errors.push('POC failure category sum does not equal failures');
  }
  if (projection.corpus.success + projection.corpus.failure !== projection.corpus.attempted) {
    errors.push('POC success and failure do not equal attempted documents');
  }
  for (const [name, value] of Object.entries({
    fontCharCount: projection.fontCharCount,
    usageCharCount: projection.usageCharCount,
    ...projection.distributionCharTotals,
  })) {
    if (value !== projection.totals.chars) errors.push(`${name} does not equal totals.chars`);
  }
  if (projection.metricMappedChars + projection.metricUnmappedChars !== projection.totals.chars) {
    errors.push('POC mapped and unmapped characters do not equal totals.chars');
  }
  const digest = sha256Text(canonicalJson(projection));
  if (digest !== expected.projectionSha256) {
    errors.push('POC de-identified projection hash differs from the frozen baseline');
  }
  return errors;
}

function compareAdditiveObject(full, left, right, location, errors) {
  const keys = new Set([
    ...Object.keys(full ?? {}),
    ...Object.keys(left ?? {}),
    ...Object.keys(right ?? {}),
  ]);
  for (const key of [...keys].sort(compareText)) {
    const fullValue = full?.[key] ?? 0;
    const leftValue = left?.[key] ?? 0;
    const rightValue = right?.[key] ?? 0;
    if (![fullValue, leftValue, rightValue].every(safeCount)) {
      errors.push(`${location}.${key} must contain additive safe counts`);
    } else if (fullValue !== leftValue + rightValue) {
      errors.push(`${location}.${key} is not HWP + HWPX additive`);
    }
  }
}

function distributionMap(rows) {
  return new Map((rows ?? []).map(row => [JSON.stringify(row.value), row.chars]));
}

export function validatePocFormatAdditivity(full, hwp, hwpx) {
  const errors = [];
  for (const [name, summary] of [['full', full], ['hwp', hwp], ['hwpx', hwpx]]) {
    if (!isObject(summary) || summary.schemaVersion !== 'poc-font-layout-habits-v2') {
      errors.push(`${name} POC summary is not v2`);
    }
  }
  if (errors.length > 0) return errors;
  if (full.repositoryHead !== hwp.repositoryHead || full.repositoryHead !== hwpx.repositoryHead) {
    errors.push('POC format summaries use different repositoryHead values');
  }
  compareAdditiveObject(
    Object.fromEntries(Object.entries(full.corpus).filter(([, value]) => safeCount(value))),
    Object.fromEntries(Object.entries(hwp.corpus).filter(([, value]) => safeCount(value))),
    Object.fromEntries(Object.entries(hwpx.corpus).filter(([, value]) => safeCount(value))),
    'corpus',
    errors,
  );
  compareAdditiveObject(
    full.corpus.failureCategories,
    hwp.corpus.failureCategories,
    hwpx.corpus.failureCategories,
    'corpus.failureCategories',
    errors,
  );
  compareAdditiveObject(full.totals, hwp.totals, hwpx.totals, 'totals', errors);
  for (const name of POC_DISTRIBUTIONS) {
    const fullRows = distributionMap(full[name]);
    const hwpRows = distributionMap(hwp[name]);
    const hwpxRows = distributionMap(hwpx[name]);
    const keys = new Set([...fullRows.keys(), ...hwpRows.keys(), ...hwpxRows.keys()]);
    for (const key of keys) {
      if ((fullRows.get(key) ?? 0) !== (hwpRows.get(key) ?? 0) + (hwpxRows.get(key) ?? 0)) {
        errors.push(`${name}[${key}] is not HWP + HWPX additive`);
      }
    }
  }
  return errors;
}

export function validateW1LedgerBaseline(ledger, contract, candidateSnapshot = null) {
  const errors = [];
  if (!Array.isArray(ledger?.rules)) return ['W1 ledger must contain rules'];
  const baseline = contract.w1Baseline;
  if (ledger.rules.length !== baseline.ledgerRuleCount) {
    errors.push(`W1 ledger rule count ${ledger.rules.length} differs from contract`);
  }
  const relationCount = relationType => ledger.rules
    .filter(rule => rule?.relationType === relationType).length;
  for (const [relationType, expected] of [
    ['identity-alias', baseline.identityAliasRuleCount],
    ['metric-surrogate', baseline.metricSurrogateRuleCount],
    ['measured-overlay', baseline.measuredOverlayRuleCount],
    ['metric-entry', baseline.metricEntryCount],
  ]) {
    const actual = relationCount(relationType);
    if (actual !== expected) {
      errors.push(`W1 ${relationType} count ${actual} differs from contract ${expected}`);
    }
  }
  const invalidIdentity = ledger.rules.filter(rule => (
    rule?.relationType === 'identity-alias'
      && !contract.provenance.identityAlias.verifiedEvidenceStatuses
        .includes(rule.evidenceStatus)
  ));
  if (invalidIdentity.length > 0) {
    errors.push(`W1 has ${invalidIdentity.length} unverified identity-alias rules`);
  }

  if (candidateSnapshot !== null) {
    if (!Array.isArray(candidateSnapshot?.ruleCandidates)) {
      errors.push('W1 candidate snapshot must contain ruleCandidates');
    } else {
      const linked = new Set(ledger.rules.flatMap(rule => (
        Array.isArray(rule.evidence) ? rule.evidence : []
      )).flatMap(evidence => {
        const match = evidence?.reference?.match(/#(candidate\.[0-9a-f]{20})$/);
        return match ? [match[1]] : [];
      }));
      const missing = candidateSnapshot.ruleCandidates
        .map(candidate => candidate.candidateId)
        .filter(candidateId => !linked.has(candidateId));
      if (missing.length > 0) {
        errors.push(`W1 ledger is missing ${missing.length} current candidate anchors`);
      }
    }
  }
  return errors;
}

function semanticBoundary(boundary) {
  return Object.fromEntries(
    Object.entries(boundary)
      .filter(([key]) => key !== 'sourceSha256')
      .sort(([left], [right]) => compareText(left, right)),
  );
}

function semanticCandidate(candidate) {
  const projection = structuredClone(candidate);
  if (isObject(projection.sourceLocation)) delete projection.sourceLocation.sourceSha256;
  return normalizeForHash(projection);
}

export function auditW1SemanticDrift(previous, current) {
  if (!Array.isArray(previous?.candidates) || !Array.isArray(previous?.ruleCandidates)) {
    throw new Error('previous W1 snapshot is invalid');
  }
  if (!Array.isArray(current?.candidates) || !Array.isArray(current?.ruleCandidates)) {
    throw new Error('current W1 snapshot is invalid');
  }
  const boundaryId = row => `${row.ownerId}.${row.selectorId}`;
  const oldBoundaries = new Map(previous.candidates.map(row => [boundaryId(row), row]));
  const currentBoundaries = new Map(current.candidates.map(row => [boundaryId(row), row]));
  const addedBoundaryIds = [...currentBoundaries.keys()]
    .filter(id => !oldBoundaries.has(id)).sort(compareText);
  const removedBoundaryIds = [...oldBoundaries.keys()]
    .filter(id => !currentBoundaries.has(id)).sort(compareText);
  const digestDriftBoundaries = [...currentBoundaries.entries()].flatMap(([id, row]) => (
    oldBoundaries.has(id) && oldBoundaries.get(id).sourceSha256 !== row.sourceSha256 ? [id] : []
  )).sort(compareText);
  const changedBoundaryIds = [...currentBoundaries.entries()].flatMap(([id, row]) => {
    const old = oldBoundaries.get(id);
    if (!old) return [];
    return canonicalJson(semanticBoundary(old)) === canonicalJson(semanticBoundary(row)) ? [] : [id];
  }).sort(compareText);

  const oldCandidates = new Map(previous.ruleCandidates.map(row => [row.candidateId, row]));
  const currentCandidates = new Map(current.ruleCandidates.map(row => [row.candidateId, row]));
  const addedCandidateIds = [...currentCandidates.keys()]
    .filter(id => !oldCandidates.has(id)).sort(compareText);
  const removedCandidateIds = [...oldCandidates.keys()]
    .filter(id => !currentCandidates.has(id)).sort(compareText);
  const changedCandidateIds = [...currentCandidates.entries()].flatMap(([id, row]) => {
    const old = oldCandidates.get(id);
    if (!old) return [];
    return canonicalJson(semanticCandidate(old)) === canonicalJson(semanticCandidate(row)) ? [] : [id];
  }).sort(compareText);
  return {
    boundaryCount: current.candidates.length,
    candidateCount: current.ruleCandidates.length,
    addedBoundaryIds,
    removedBoundaryIds,
    changedBoundaryIds,
    digestDriftBoundaries,
    addedCandidateIds,
    removedCandidateIds,
    changedCandidateIds,
  };
}

function argumentValue(args, name) {
  const index = args.indexOf(name);
  return index === -1 || index === args.length - 1 ? null : args[index + 1];
}

function runCheck(args) {
  const contract = readJson(CONTRACT_PATH);
  const errors = validateCoverageContract(contract);
  const sources = readJson(path.join(W1_INVESTIGATION, 'font_rule_sources.json'));
  const previous = readJson(path.join(W1_INVESTIGATION, 'font_rule_candidates.json'));
  const ledger = readJson(path.join(W1_INVESTIGATION, 'font_rule_ledger.json'));
  const head = execFileSync('git', ['rev-parse', 'HEAD'], {
    cwd: REPOSITORY_ROOT,
    encoding: 'utf8',
  }).trim();
  const currentBoundaries = collectSourceCandidates(sources, REPOSITORY_ROOT, head);
  const current = collectRuleCandidates(currentBoundaries, REPOSITORY_ROOT);
  const audit = auditW1SemanticDrift(previous, current);
  errors.push(...validateW1LedgerBaseline(ledger, contract, current));
  for (const [field, values] of Object.entries(audit)) {
    if (field === 'digestDriftBoundaries' || !Array.isArray(values)) continue;
    if (values.length > 0) errors.push(`W1 semantic drift ${field}: ${values.length}`);
  }
  if (audit.boundaryCount !== contract.w1Baseline.boundaryCount) {
    errors.push(`W1 boundary count ${audit.boundaryCount} differs from contract`);
  }
  if (audit.candidateCount !== contract.w1Baseline.candidateCount) {
    errors.push(`W1 candidate count ${audit.candidateCount} differs from contract`);
  }

  const pocArgument = argumentValue(args, '--poc');
  if (pocArgument !== null) {
    const pocPath = path.resolve(process.cwd(), pocArgument);
    const poc = readJson(pocPath);
    errors.push(...validatePocV2Baseline(poc, contract));
    const hwpArgument = argumentValue(args, '--poc-hwp');
    const hwpxArgument = argumentValue(args, '--poc-hwpx');
    if ((hwpArgument === null) !== (hwpxArgument === null)) {
      errors.push('--poc-hwp and --poc-hwpx must be provided together');
    } else if (hwpArgument !== null) {
      errors.push(...validatePocFormatAdditivity(
        poc,
        readJson(path.resolve(process.cwd(), hwpArgument)),
        readJson(path.resolve(process.cwd(), hwpxArgument)),
      ));
    }
  }
  if (errors.length > 0) throw new Error(errors.join('\n'));
  process.stdout.write(
    `font metric coverage Stage 1 contracts: ok; W1 ${audit.candidateCount} candidates, ${audit.digestDriftBoundaries.length} digest-only boundaries${pocArgument === null ? '' : '; POC v2 baseline ok'}\n`,
  );
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : '';
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv[2] !== 'check') {
      throw new Error('usage: node scripts/font_metric_coverage_contract.mjs check [--poc <summary-v2.json> --poc-hwp <summary-hwp-v2.json> --poc-hwpx <summary-hwpx-v2.json>]');
    }
    runCheck(process.argv.slice(3));
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
