#!/usr/bin/env node

import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const INVESTIGATION = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4963',
);
const DEFAULT_CONTRACT_PATH = path.join(INVESTIGATION, 'oracle_profile_contract.json');
const DEFAULT_SCHEMA_PATH = path.join(INVESTIGATION, 'oracle_profile.schema.json');
const DEFAULT_FIXTURES_PATH = path.join(INVESTIGATION, 'oracle_profile_public_fixtures.json');
const FROZEN_RANKING_FILE_SHA256 = '6947e9e8a6c67a60a54b04dc6f1abf75e3cc66d9096a978d301ba2c10bb4ee3a';
const FROZEN_RANKING_OUTPUT_SHA256 = '95e7a41d1ed92a60cb66e1705b038c3e9086829b3c8aee48af57e8c2da111a68';
const SHA256_PATTERN = /^[0-9a-f]{64}$/u;
const GIT_SHA_PATTERN = /^[0-9a-f]{40}$/u;

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function exactArray(actual, expected) {
  return JSON.stringify(actual) === JSON.stringify(expected);
}

function exactKeys(value, expected, label, errors) {
  if (!isObject(value)) {
    errors.push(`${label} must be an object`);
    return false;
  }
  const actual = Object.keys(value).sort();
  const wanted = [...expected].sort();
  if (!exactArray(actual, wanted)) {
    errors.push(`${label} schema drift`);
    return false;
  }
  return true;
}

function uniqueNonEmptyStrings(value) {
  return Array.isArray(value)
    && value.length > 0
    && value.every(entry => typeof entry === 'string' && entry.length > 0)
    && new Set(value).size === value.length;
}

function nonNegativeInteger(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function positiveInteger(value) {
  return Number.isSafeInteger(value) && value > 0;
}

function sha256File(file) {
  return createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function walkStrings(value, visit, label = '$') {
  if (typeof value === 'string') {
    visit(value, label);
    return;
  }
  if (Array.isArray(value)) {
    value.forEach((entry, index) => walkStrings(entry, visit, `${label}[${index}]`));
    return;
  }
  if (isObject(value)) {
    for (const [key, entry] of Object.entries(value)) {
      walkStrings(entry, visit, `${label}.${key}`);
    }
  }
}

function validateEvidence(value, label, statuses, errors) {
  if (!isObject(value)) {
    errors.push(`${label} must be an evidence object`);
    return;
  }
  exactKeys(value, ['status', 'value', 'reason'], label, errors);
  if (!statuses.includes(value.status)) {
    errors.push(`${label}.status is not in the contract inventory`);
    return;
  }
  if (value.status === 'observed') {
    if (value.value === null || value.value === undefined) {
      errors.push(`${label} observed evidence requires a value`);
    }
    if (value.reason !== null) {
      errors.push(`${label} observed evidence reason must be null`);
    }
    return;
  }
  if (value.value !== null) {
    errors.push(`${label} ${value.status} evidence value must be null`);
  }
  if (typeof value.reason !== 'string' || value.reason.trim().length === 0) {
    errors.push(`${label} ${value.status} evidence requires a reason`);
  }
}

function validateObservedShapes(profile, contract, errors) {
  const observed = (entry) => entry?.status === 'observed';
  const fontState = profile.fontState ?? {};
  const observations = profile.observations ?? {};

  if (observed(fontState.installedFontSha256)
      && !SHA256_PATTERN.test(fontState.installedFontSha256.value ?? '')) {
    errors.push('fontState.installedFontSha256 observed value must be a SHA-256 digest');
  }
  if (observed(fontState.readbackFace)
      && (typeof fontState.readbackFace.value !== 'string'
        || fontState.readbackFace.value.length === 0)) {
    errors.push('fontState.readbackFace observed value must be a non-empty string');
  }
  if (observed(fontState.readbackFontType)
      && !nonNegativeInteger(fontState.readbackFontType.value)) {
    errors.push('fontState.readbackFontType observed value must be a non-negative integer');
  }
  if (observed(observations.subsetFontName)
      && (typeof observations.subsetFontName.value !== 'string'
        || observations.subsetFontName.value.length === 0)) {
    errors.push('observations.subsetFontName observed value must be a non-empty string');
  }
  if (observed(observations.glyphOutlineDigest)
      && !SHA256_PATTERN.test(observations.glyphOutlineDigest.value ?? '')) {
    errors.push('observations.glyphOutlineDigest observed value must be a SHA-256 digest');
  }

  if (observed(observations.hmtxAdvance)) {
    const value = observations.hmtxAdvance.value;
    if (exactKeys(
      value,
      ['advance', 'unitsPerEm', 'faceIndex', 'sourceFontSha256'],
      'observations.hmtxAdvance.value',
      errors,
    )) {
      if (!nonNegativeInteger(value.advance)) {
        errors.push('observations.hmtxAdvance.value.advance must be a non-negative integer');
      }
      if (!positiveInteger(value.unitsPerEm)) {
        errors.push('observations.hmtxAdvance.value.unitsPerEm must be a positive integer');
      }
      if (!nonNegativeInteger(value.faceIndex)) {
        errors.push('observations.hmtxAdvance.value.faceIndex must be a non-negative integer');
      }
      if (!SHA256_PATTERN.test(value.sourceFontSha256 ?? '')) {
        errors.push('observations.hmtxAdvance.value.sourceFontSha256 must be a SHA-256 digest');
      }
      if (observed(fontState.installedFontSha256)
          && value.sourceFontSha256 !== fontState.installedFontSha256.value) {
        errors.push('hmtx source digest must match the observed installed font digest');
      }
    }
  }

  if (observed(observations.pdfObservedAdvance)) {
    const value = observations.pdfObservedAdvance.value;
    if (exactKeys(
      value,
      ['advance', 'unit', 'glyphOrCid'],
      'observations.pdfObservedAdvance.value',
      errors,
    )) {
      if (typeof value.advance !== 'number'
          || !Number.isFinite(value.advance)
          || value.advance < 0) {
        errors.push('observations.pdfObservedAdvance.value.advance must be finite and non-negative');
      }
      if (value.unit !== 'pdf-user-space') {
        errors.push('observations.pdfObservedAdvance.value.unit must be pdf-user-space');
      }
      if (typeof value.glyphOrCid !== 'string' || value.glyphOrCid.length === 0) {
        errors.push('observations.pdfObservedAdvance.value.glyphOrCid must be a non-empty string');
      }
    }
  }

  if (observed(observations.firstTypesettingDivergence)) {
    const value = observations.firstTypesettingDivergence.value;
    if (exactKeys(
      value,
      ['plane', 'characterIndex', 'lineIndex', 'pageIndex'],
      'observations.firstTypesettingDivergence.value',
      errors,
    )) {
      if (!contract.profilePolicy.firstDivergencePlanes.includes(value.plane)) {
        errors.push('first divergence plane is not in the contract inventory');
      }
      for (const field of ['characterIndex', 'lineIndex', 'pageIndex']) {
        if (value[field] !== null && !nonNegativeInteger(value[field])) {
          errors.push(`first divergence ${field} must be null or a non-negative integer`);
        }
      }
      const locations = [value.characterIndex, value.lineIndex, value.pageIndex];
      if (value.plane === 'none' && locations.some(entry => entry !== null)) {
        errors.push('a none divergence must not claim a location');
      }
      if (value.plane !== 'none' && locations.every(entry => entry === null)) {
        errors.push('a non-none divergence must identify a location');
      }
    }
  }

  for (const field of ['lineCount', 'pageCount']) {
    if (observed(observations[field]) && !nonNegativeInteger(observations[field].value)) {
      errors.push(`observations.${field} observed value must be a non-negative integer`);
    }
  }
}

export function validateOracleProfileContract(contract) {
  const errors = [];
  if (!isObject(contract)) return ['contract must be an object'];
  exactKeys(contract, [
    'schemaVersion',
    'kind',
    'issue',
    'parentIssue',
    'predecessorIssue',
    'inputPreconditions',
    'profilePolicy',
    'environmentPolicy',
    'privacy',
    'resourcePolicy',
    'scope',
  ], 'contract', errors);
  if (contract.schemaVersion !== 1) errors.push('schemaVersion must be 1');
  if (contract.kind !== 'font-oracle-profile-contract') {
    errors.push('kind must be font-oracle-profile-contract');
  }
  if (contract.issue !== 4963 || contract.parentIssue !== 4960
      || contract.predecessorIssue !== 4962) {
    errors.push('issue lineage must remain #4960 -> #4962 -> #4963');
  }

  const input = contract.inputPreconditions;
  exactKeys(input, [
    'ranking', 'queueFaces', 'existingHancom2022Evidence', 'reusePolicy',
  ], 'inputPreconditions', errors);
  exactKeys(input?.ranking, [
    'artifact',
    'fileSha256',
    'outputSha256',
    'kind',
    'queueFaceCount',
    'queueRiskCharacters',
    'queueBaseRiskMass',
    'queueBaseRiskMassPpm',
  ], 'inputPreconditions.ranking', errors);
  if (input?.ranking?.artifact
      !== 'mydocs/report/assets/task_m100_4962/font_typesetting_risk_rank.json'
      || input.ranking.fileSha256 !== FROZEN_RANKING_FILE_SHA256
      || input.ranking.outputSha256 !== FROZEN_RANKING_OUTPUT_SHA256
      || input.ranking.kind !== 'font-typesetting-risk-public-ranking'
      || input.ranking.queueFaceCount !== 17
      || input.ranking.queueRiskCharacters !== 1562076
      || input.ranking.queueBaseRiskMass !== 7015182
      || input.ranking.queueBaseRiskMassPpm !== 810374) {
    errors.push('W4 ranking precondition has drifted');
  }
  if (!uniqueNonEmptyStrings(input?.queueFaces) || input.queueFaces.length !== 17) {
    errors.push('W4 queue must contain 17 unique document faces');
  }
  if (!Array.isArray(input?.existingHancom2022Evidence)
      || input.existingHancom2022Evidence.length !== 2
      || input.existingHancom2022Evidence.some(entry => (
        typeof entry?.artifact !== 'string'
        || !SHA256_PATTERN.test(entry?.sha256 ?? '')
        || typeof entry?.role !== 'string'
        || entry.role.length === 0
      ))) {
    errors.push('existing Hancom 2022 evidence inventory is invalid');
  }
  if (input?.reusePolicy !== 'reuse-hash-matched-evidence-without-full-remeasurement') {
    errors.push('existing evidence reuse policy has drifted');
  }

  const policy = contract.profilePolicy;
  exactKeys(policy, [
    'questionIds',
    'observationStatuses',
    'evidenceClasses',
    'relationTypes',
    'firstDivergencePlanes',
    'requiredW4Fields',
    'requiredEvidenceFields',
    'questionAndExactMissingStateMustMatch',
    'plainNullForbidden',
    'observedRequiresValue',
    'unobservedRequiresNullAndReason',
    'historicalImportMissingProvenanceExplicit',
    'hmtxAndPdfAdvanceSeparated',
    'officialSuccessorRequiresObservedDirectAnchor',
    'unknownIdentityGuessing',
  ], 'profilePolicy', errors);
  for (const field of [
    'questionIds',
    'observationStatuses',
    'evidenceClasses',
    'relationTypes',
    'firstDivergencePlanes',
    'requiredW4Fields',
    'requiredEvidenceFields',
  ]) {
    if (!uniqueNonEmptyStrings(policy?.[field])) {
      errors.push(`profilePolicy.${field} must be a unique non-empty string inventory`);
    }
  }
  if (!exactArray(policy?.questionIds, [
    'exact-installed',
    'exact-removed',
    'document-subst-font-only',
    'curated-official-successor-only',
    'all-related-fonts-missing',
  ])) {
    errors.push('controlled ladder question order has drifted');
  }
  if (!exactArray(policy?.observationStatuses, [
    'observed', 'unavailable', 'not-applicable', 'blocked',
  ])) {
    errors.push('observation status inventory has drifted');
  }
  if (!exactArray(policy?.evidenceClasses, [
    'synthetic-contract-fixture', 'historical-import', 'oracle-run',
  ])) {
    errors.push('evidence class inventory has drifted');
  }
  if (!exactArray(policy?.relationTypes, [
    'identity-exact',
    'identity-alias',
    'official-successor',
    'document-substitution',
    'metric-surrogate',
    'hancom-missing-font',
    'unknown',
  ])) {
    errors.push('relation type inventory has drifted');
  }
  if (!exactArray(policy?.firstDivergencePlanes, [
    'selection', 'glyph', 'advance', 'line', 'page', 'none',
  ])) {
    errors.push('first divergence plane inventory has drifted');
  }
  if (!exactArray(policy?.requiredW4Fields, [
    'inputSha256',
    'hancomVersion',
    'pdfProducer',
    'installedFontSha256',
    'exactMissingState',
    'subsetFontName',
    'glyphOutlineDigest',
    'hmtxAdvance',
    'firstTypesettingDivergence',
    'relationEvidence',
  ])) {
    errors.push('required W4 Oracle Profile field inventory has drifted');
  }
  if (!exactArray(policy?.requiredEvidenceFields, [
    'installedFontSha256',
    'readbackFace',
    'readbackFontType',
    'subsetFontName',
    'glyphOutlineDigest',
    'hmtxAdvance',
    'pdfObservedAdvance',
    'firstTypesettingDivergence',
    'lineCount',
    'pageCount',
  ])) {
    errors.push('required executable evidence field inventory has drifted');
  }
  for (const flag of [
    'questionAndExactMissingStateMustMatch',
    'plainNullForbidden',
    'observedRequiresValue',
    'unobservedRequiresNullAndReason',
    'historicalImportMissingProvenanceExplicit',
    'hmtxAndPdfAdvanceSeparated',
    'officialSuccessorRequiresObservedDirectAnchor',
  ]) {
    if (policy?.[flag] !== true) errors.push(`profilePolicy.${flag} must remain true`);
  }
  if (policy?.unknownIdentityGuessing !== false) {
    errors.push('profilePolicy.unknownIdentityGuessing must remain false');
  }

  const environment = contract.environmentPolicy;
  exactKeys(environment, [
    'acceptanceOracle',
    'hancom2010Role',
    'versionBranching',
    'featureAndEvidenceDetection',
    'ambientFontManifestRequired',
    'processResetRequired',
    'mutableFontState',
    'immutableBundleDisposition',
  ], 'environmentPolicy', errors);
  if (environment?.acceptanceOracle !== 'hancom-2020-or-2022-controlled-environment'
      || environment.hancom2010Role !== 'secondary-historical-only'
      || environment.versionBranching !== false
      || environment.featureAndEvidenceDetection !== true
      || environment.ambientFontManifestRequired !== true
      || environment.processResetRequired !== true
      || environment.mutableFontState !== 'disposable-snapshot-explicit-approval-only'
      || environment.immutableBundleDisposition !== 'blocked') {
    errors.push('environment authority or mutation boundary has drifted');
  }

  exactKeys(contract.privacy, [
    'fontBytesTracked',
    'privateCorpusUsedAsPublicFixture',
    'privateDocumentIdentityPublished',
    'absoluteFontPathsPublished',
    'localFontRoot',
    'localInventoryMode',
    'publicFields',
  ], 'privacy', errors);
  if (contract.privacy?.fontBytesTracked !== false
      || contract.privacy.privateCorpusUsedAsPublicFixture !== false
      || contract.privacy.privateDocumentIdentityPublished !== false
      || contract.privacy.absoluteFontPathsPublished !== false
      || contract.privacy.localFontRoot !== '/home/edward/mygithub/ttfs'
      || contract.privacy.localInventoryMode !== '0600'
      || !exactArray(contract.privacy.publicFields, [
        'font-names',
        'font-digests',
        'license-and-embedding-status',
        'aggregate-observations',
        'evidence-status-and-reason',
      ])) {
    errors.push('privacy boundary has drifted');
  }
  exactKeys(contract.resourcePolicy, [
    'regularFilesOnly',
    'rejectSymlinks',
    'rejectPathTraversal',
    'perItemTimeoutRequired',
    'inputByteLimitRequired',
    'pdfPageLimitRequired',
    'pdfObjectLimitRequired',
    'pdfGlyphLimitRequired',
    'temporaryDirectoryCleanupRequired',
    'batchFailureIsolation',
  ], 'resourcePolicy', errors);
  if (Object.values(contract.resourcePolicy ?? {}).some(value => value !== true)) {
    errors.push('resource policy guards must all remain enabled');
  }
  exactKeys(contract.scope, [
    'productMetricDatabaseChanged',
    'productFallbackChanged',
    'productPaintChanged',
    'all351FacesInvestigated',
    'githubWrite',
  ], 'scope', errors);
  if (Object.values(contract.scope ?? {}).some(value => value !== false)) {
    errors.push('W5-1 scope must not change product behavior or write GitHub state');
  }
  return errors;
}

export function validateOracleProfile(profile, contract) {
  const errors = [];
  if (!isObject(profile)) return ['profile must be an object'];
  exactKeys(profile, [
    'schemaVersion',
    'kind',
    'issue',
    'candidate',
    'questionId',
    'exactMissingState',
    'input',
    'environment',
    'execution',
    'fontState',
    'observations',
    'relationEvidence',
    'privacy',
  ], 'profile', errors);
  if (profile.schemaVersion !== 2) errors.push('profile schemaVersion must be 2');
  if (profile.kind !== 'font-oracle-profile') {
    errors.push('profile kind must be font-oracle-profile');
  }
  if (profile.issue !== 4963) errors.push('profile issue must be 4963');

  const candidate = profile.candidate;
  if (exactKeys(candidate, ['queueRank', 'documentFace'], 'candidate', errors)) {
    const expectedFace = contract.inputPreconditions.queueFaces[candidate.queueRank - 1];
    if (!positiveInteger(candidate.queueRank)
        || candidate.queueRank > contract.inputPreconditions.queueFaces.length
        || candidate.documentFace !== expectedFace) {
      errors.push('candidate queue rank and document face do not match the W4 queue');
    }
  }
  if (!contract.profilePolicy.questionIds.includes(profile.questionId)) {
    errors.push('questionId is not in the controlled ladder');
  }
  if (profile.exactMissingState !== profile.questionId) {
    errors.push('questionId and exactMissingState must match');
  }

  const input = profile.input;
  if (exactKeys(input, [
    'sourceFormat', 'sha256', 'fixtureContractVersion', 'fixtureGeneratorCommit',
  ], 'input', errors)) {
    validateEvidence(
      input.sourceFormat,
      'input.sourceFormat',
      contract.profilePolicy.observationStatuses,
      errors,
    );
    validateEvidence(
      input.sha256,
      'input.sha256',
      contract.profilePolicy.observationStatuses,
      errors,
    );
    if (input.sourceFormat?.status === 'observed'
        && !['hwp', 'hwpx', 'in-memory-hwp'].includes(input.sourceFormat.value)) {
      errors.push('input sourceFormat observed value is invalid');
    }
    if (input.sha256?.status === 'observed'
        && !SHA256_PATTERN.test(input.sha256.value ?? '')) {
      errors.push('input sha256 observed value is invalid');
    }
    if (typeof input.fixtureContractVersion !== 'string'
        || input.fixtureContractVersion.length === 0) {
      errors.push('input fixtureContractVersion is required');
    }
    if (!GIT_SHA_PATTERN.test(input.fixtureGeneratorCommit ?? '')) {
      errors.push('input fixtureGeneratorCommit is invalid');
    }
  }

  const environment = profile.environment;
  if (exactKeys(environment, [
    'os',
    'locale',
    'hancomVersion',
    'pdfProducer',
    'exportRoute',
    'oracleAuthority',
    'ambientFontManifestSha256',
    'processReset',
    'rebooted',
  ], 'environment', errors)) {
    for (const field of [
      'os',
      'locale',
      'hancomVersion',
      'pdfProducer',
      'exportRoute',
      'ambientFontManifestSha256',
      'processReset',
      'rebooted',
    ]) {
      validateEvidence(
        environment[field],
        `environment.${field}`,
        contract.profilePolicy.observationStatuses,
        errors,
      );
    }
    for (const field of ['os', 'locale', 'hancomVersion', 'pdfProducer', 'exportRoute']) {
      if (environment[field]?.status === 'observed'
          && (typeof environment[field].value !== 'string'
            || environment[field].value.length === 0)) {
        errors.push(`environment.${field} observed value must be a non-empty string`);
      }
    }
    if (!['acceptance-primary', 'secondary-historical', 'contract-fixture']
      .includes(environment.oracleAuthority)) {
      errors.push('environment.oracleAuthority is invalid');
    }
    if (environment.ambientFontManifestSha256?.status === 'observed'
        && !SHA256_PATTERN.test(environment.ambientFontManifestSha256.value ?? '')) {
      errors.push('environment ambient font manifest observed digest is invalid');
    }
    for (const field of ['processReset', 'rebooted']) {
      if (environment[field]?.status === 'observed'
          && typeof environment[field].value !== 'boolean') {
        errors.push(`environment.${field} observed value must be boolean`);
      }
    }
  }

  const execution = profile.execution;
  if (exactKeys(execution, [
    'evidenceClass', 'measurementDate', 'startedAt', 'finishedAt', 'repeatIndex',
  ], 'execution', errors)) {
    if (!contract.profilePolicy.evidenceClasses.includes(execution.evidenceClass)) {
      errors.push('execution evidenceClass is invalid');
    }
    if (typeof execution.measurementDate !== 'string'
        || !/^\d{4}-\d{2}-\d{2}$/u.test(execution.measurementDate)) {
      errors.push('execution measurementDate must be YYYY-MM-DD');
    }
    for (const field of ['startedAt', 'finishedAt']) {
      validateEvidence(
        execution[field],
        `execution.${field}`,
        contract.profilePolicy.observationStatuses,
        errors,
      );
      if (execution[field]?.status === 'observed'
          && !Number.isFinite(Date.parse(execution[field].value))) {
        errors.push(`execution.${field} observed value must be a date-time`);
      }
    }
    if (execution.startedAt?.status === 'observed'
        && execution.finishedAt?.status === 'observed'
        && Date.parse(execution.finishedAt.value) < Date.parse(execution.startedAt.value)) {
      errors.push('execution observed timestamps must be monotonic');
    }
    if (!positiveInteger(execution.repeatIndex)) {
      errors.push('execution repeatIndex must be a positive integer');
    }
    const expectedAuthority = {
      'synthetic-contract-fixture': 'contract-fixture',
      'historical-import': 'secondary-historical',
      'oracle-run': 'acceptance-primary',
    }[execution.evidenceClass];
    if (environment?.oracleAuthority !== expectedAuthority) {
      errors.push('evidenceClass and oracleAuthority do not match');
    }
    if (execution.evidenceClass === 'oracle-run') {
      if (environment?.processReset?.status !== 'observed'
          || environment.processReset.value !== true) {
        errors.push('an oracle-run requires an observed reset Hancom process');
      }
      for (const [label, evidence] of [
        ['input.sourceFormat', input?.sourceFormat],
        ['input.sha256', input?.sha256],
        ['environment.ambientFontManifestSha256', environment?.ambientFontManifestSha256],
        ['execution.startedAt', execution.startedAt],
        ['execution.finishedAt', execution.finishedAt],
      ]) {
        if (evidence?.status !== 'observed') {
          errors.push(`an oracle-run requires observed ${label}`);
        }
      }
    }
  }

  const fontState = profile.fontState;
  const fontFields = [
    'requestedFace',
    'relatedFaceSet',
    'installedFontSha256',
    'readbackFace',
    'readbackFontType',
  ];
  if (exactKeys(fontState, fontFields, 'fontState', errors)) {
    if (fontState.requestedFace !== candidate?.documentFace) {
      errors.push('fontState.requestedFace must match candidate.documentFace');
    }
    if (!Array.isArray(fontState.relatedFaceSet)
        || fontState.relatedFaceSet.some(entry => typeof entry !== 'string' || entry.length === 0)
        || new Set(fontState.relatedFaceSet).size !== fontState.relatedFaceSet.length) {
      errors.push('fontState.relatedFaceSet must be a unique string array');
    }
  }
  for (const field of ['installedFontSha256', 'readbackFace', 'readbackFontType']) {
    validateEvidence(
      fontState?.[field],
      `fontState.${field}`,
      contract.profilePolicy.observationStatuses,
      errors,
    );
  }

  const observations = profile.observations;
  const observationFields = [
    'subsetFontName',
    'glyphOutlineDigest',
    'hmtxAdvance',
    'pdfObservedAdvance',
    'firstTypesettingDivergence',
    'lineCount',
    'pageCount',
  ];
  exactKeys(observations, observationFields, 'observations', errors);
  for (const field of observationFields) {
    validateEvidence(
      observations?.[field],
      `observations.${field}`,
      contract.profilePolicy.observationStatuses,
      errors,
    );
  }
  validateObservedShapes(profile, contract, errors);

  const relation = profile.relationEvidence;
  if (exactKeys(relation, ['type', 'anchor'], 'relationEvidence', errors)) {
    if (!contract.profilePolicy.relationTypes.includes(relation.type)) {
      errors.push('relationEvidence.type is not in the contract inventory');
    }
  }
  validateEvidence(
    relation?.anchor,
    'relationEvidence.anchor',
    contract.profilePolicy.observationStatuses,
    errors,
  );
  if (relation?.type === 'official-successor' && relation?.anchor?.status !== 'observed') {
    errors.push('official-successor requires an observed direct anchor');
  } else if (relation?.type !== 'unknown' && relation?.anchor?.status !== 'observed') {
    errors.push(`${relation?.type} requires an observed direct anchor`);
  }

  const privacy = profile.privacy;
  if (exactKeys(privacy, [
    'fontBytesEmbedded',
    'privateDocumentIdentityIncluded',
    'absoluteFontPathIncluded',
  ], 'privacy', errors)
      && Object.values(privacy).some(value => value !== false)) {
    errors.push('privacy flags must all be false');
  }
  walkStrings(profile, (value, label) => {
    if (/^(?:\/home\/|\/mnt\/|[A-Za-z]:[\\/])/u.test(value)) {
      errors.push(`${label} exposes an absolute local path`);
    }
  });
  return errors;
}

export function validateOracleProfileSchema(schema, contract) {
  const errors = [];
  if (schema?.$schema !== 'https://json-schema.org/draft/2020-12/schema'
      || schema?.type !== 'object'
      || schema?.additionalProperties !== false
      || schema?.properties?.schemaVersion?.const !== 2) {
    errors.push('Oracle Profile JSON Schema root has drifted');
  }
  if (!exactArray(schema?.$defs?.questionId?.enum, contract.profilePolicy.questionIds)) {
    errors.push('schema question inventory differs from the contract');
  }
  if (!exactArray(schema?.$defs?.relationType?.enum, contract.profilePolicy.relationTypes)) {
    errors.push('schema relation inventory differs from the contract');
  }
  if (!exactArray(
    schema?.$defs?.evidence?.properties?.status?.enum,
    contract.profilePolicy.observationStatuses,
  )) {
    errors.push('schema evidence status inventory differs from the contract');
  }
  if (!exactArray(
    schema?.properties?.execution?.properties?.evidenceClass?.enum,
    contract.profilePolicy.evidenceClasses,
  )) {
    errors.push('schema evidence class inventory differs from the contract');
  }
  return errors;
}

export function validateFrozenOracleInputs(contract, root = ROOT) {
  const errors = [];
  const rankingPath = path.join(root, contract.inputPreconditions.ranking.artifact);
  if (!fs.existsSync(rankingPath)) return ['frozen W4 ranking artifact is missing'];
  const digest = sha256File(rankingPath);
  if (digest !== contract.inputPreconditions.ranking.fileSha256) {
    errors.push('frozen W4 ranking file SHA-256 does not match');
  }
  const ranking = readJson(rankingPath);
  if (ranking.kind !== contract.inputPreconditions.ranking.kind
      || ranking.outputHash?.value !== contract.inputPreconditions.ranking.outputSha256) {
    errors.push('frozen W4 ranking semantic identity does not match');
  }
  const handoff = ranking.w5Handoff;
  if (handoff?.issue !== 4963
      || !exactArray(handoff.questionIds, contract.profilePolicy.questionIds)
      || !exactArray(handoff.requiredOracleProfileFields, contract.profilePolicy.requiredW4Fields)) {
    errors.push('W4 handoff contract has drifted');
  }
  const faces = (handoff?.queue ?? []).map(entry => entry.documentFace);
  const ranks = (handoff?.queue ?? []).map(entry => entry.actionRank);
  if (!exactArray(faces, contract.inputPreconditions.queueFaces)
      || !exactArray(ranks, Array.from({ length: 17 }, (_, index) => index + 1))) {
    errors.push('W4 action queue order has drifted');
  }
  const selection = ranking.selection;
  const frozen = contract.inputPreconditions.ranking;
  if (selection?.queueFaceCount !== frozen.queueFaceCount
      || selection.queueRiskCharacters !== frozen.queueRiskCharacters
      || selection.queueBaseRiskMass !== frozen.queueBaseRiskMass
      || selection.queueBaseRiskMassPpm !== frozen.queueBaseRiskMassPpm) {
    errors.push('W4 queue totals have drifted');
  }
  for (const entry of contract.inputPreconditions.existingHancom2022Evidence) {
    const evidencePath = path.join(root, entry.artifact);
    if (!fs.existsSync(evidencePath) || sha256File(evidencePath) !== entry.sha256) {
      errors.push(`existing evidence hash does not match: ${entry.artifact}`);
    }
  }
  return errors;
}

export function applyPublicFixtureMutation(profile, mutation) {
  const copy = structuredClone(profile);
  const parts = mutation.path.split('.');
  let parent = copy;
  for (const part of parts.slice(0, -1)) {
    parent = parent[part];
  }
  const key = parts.at(-1);
  if (mutation.operation === 'delete') {
    delete parent[key];
  } else if (mutation.operation === 'set') {
    parent[key] = structuredClone(mutation.value);
  } else if (mutation.operation === 'merge') {
    parent[key] = { ...parent[key], ...structuredClone(mutation.value) };
  } else {
    throw new Error(`unsupported fixture mutation: ${mutation.operation}`);
  }
  return copy;
}

export function validatePublicOracleFixtures(fixtures, contract) {
  const errors = [];
  if (!isObject(fixtures)
      || fixtures.schemaVersion !== 1
      || fixtures.kind !== 'font-oracle-profile-public-fixtures'
      || fixtures.issue !== 4963) {
    return ['public fixture envelope is invalid'];
  }
  errors.push(...validateOracleProfile(fixtures.validProfile, contract)
    .map(error => `valid profile: ${error}`));
  if (!Array.isArray(fixtures.negativeCases)
      || fixtures.negativeCases.length === 0
      || new Set(fixtures.negativeCases.map(entry => entry?.id)).size
        !== fixtures.negativeCases.length) {
    errors.push('negative fixture cases must be a unique non-empty array');
    return errors;
  }
  for (const mutation of fixtures.negativeCases) {
    const mutated = applyPublicFixtureMutation(fixtures.validProfile, mutation);
    const actual = validateOracleProfile(mutated, contract);
    if (actual.length === 0) {
      errors.push(`negative fixture unexpectedly passed: ${mutation.id}`);
    } else if (!actual.includes(mutation.expectedError)) {
      errors.push(`negative fixture ${mutation.id} missed expected error: ${mutation.expectedError}`);
    }
  }
  return errors;
}

function parseArguments(argv) {
  if (argv[0] !== 'check') {
    throw new Error('usage: node scripts/oracle_profile_contract.mjs check [--profile FILE]');
  }
  let profilePath = null;
  for (let index = 1; index < argv.length; index += 1) {
    if (argv[index] === '--profile' && argv[index + 1]) {
      profilePath = path.resolve(argv[index + 1]);
      index += 1;
    } else {
      throw new Error(`unknown argument: ${argv[index]}`);
    }
  }
  return { profilePath };
}

function main() {
  const { profilePath } = parseArguments(process.argv.slice(2));
  const contract = readJson(DEFAULT_CONTRACT_PATH);
  const schema = readJson(DEFAULT_SCHEMA_PATH);
  const fixtures = readJson(DEFAULT_FIXTURES_PATH);
  const errors = [
    ...validateOracleProfileContract(contract),
    ...validateOracleProfileSchema(schema, contract),
    ...validateFrozenOracleInputs(contract),
    ...validatePublicOracleFixtures(fixtures, contract),
  ];
  if (profilePath !== null) {
    errors.push(...validateOracleProfile(readJson(profilePath), contract)
      .map(error => `${profilePath}: ${error}`));
  }
  if (errors.length > 0) {
    throw new Error(errors.join('\n'));
  }
  process.stdout.write(JSON.stringify({
    ok: true,
    issue: 4963,
    frozenQueueFaces: contract.inputPreconditions.queueFaces.length,
    negativeFixtures: fixtures.negativeCases.length,
    profile: profilePath,
  }, null, 2) + '\n');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
