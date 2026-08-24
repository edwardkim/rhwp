#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from './font_rule_ledger.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const REGISTRY_V1_PATH = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry.json');
const REGISTRY_V1_SCHEMA_PATH = path.join(
  ROOT,
  'assets',
  'font-rules',
  'font_rule_registry.schema.json',
);
const REGISTRY_V2_PATH = path.join(
  ROOT,
  'assets',
  'font-rules',
  'font_rule_registry_v2.json',
);
const REGISTRY_V2_SCHEMA_PATH = path.join(
  ROOT,
  'assets',
  'font-rules',
  'font_rule_registry_v2.schema.json',
);
const MIGRATION_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-5955',
  'font_rule_registry_v1_to_v2_migration.json',
);
const MIGRATION_SCHEMA_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-5955',
  'font_rule_registry_v1_to_v2_migration.schema.json',
);

const PROJECTION_IDS = [
  'rust-layout-name',
  'rust-layout-metric',
  'canvas2d-paint',
  'canvas2d-webfont',
  'canvaskit-sfnt',
];
const DECISION_PLANES = [
  'layout-name',
  'layout-metric',
  'paint',
  'supply',
  'detection',
];
const RELATION_TYPES = [
  'document-substitution',
  'official-successor',
  'metric-surrogate',
  'paint-substitute',
  'style-fallback',
  'generic-fallback',
  'supply-source',
  'capability-detection',
  'unknown',
];
const OPERATION_TYPES = [
  'augment-evidence',
  'add-rule',
  'retire-rule',
  'retire-and-replace',
];
const PROJECTION_PLANES = {
  'rust-layout-name': new Set(['layout-name']),
  'rust-layout-metric': new Set(['layout-metric']),
  'canvas2d-paint': new Set(['paint']),
  'canvas2d-webfont': new Set(['supply']),
  'canvaskit-sfnt': new Set(['supply', 'detection']),
};
const MIGRATION_EVENT_ID = 'migration:issue-5955';
const SEALED_EVIDENCE_ID = 'evidence.issue-5955.sealed-v1-registry';
const SHA1_PATTERN = /^[0-9a-f]{40}$/u;
const SHA256_PATTERN = /^[0-9a-f]{64}$/u;
const IDENTIFIER_PATTERN = /^[a-z0-9]+(?:[._:-][a-z0-9]+)*$/u;

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

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function sha256File(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function currentGitHead(root) {
  return execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
}

function clone(value) {
  return structuredClone(value);
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function relativePath(file, root = ROOT) {
  return path.relative(root, file).split(path.sep).join('/');
}

function pathDigest(file, root = ROOT) {
  return { path: relativePath(file, root), sha256: sha256File(file) };
}

function countBy(values) {
  const counts = {};
  for (const value of values) counts[value] = (counts[value] ?? 0) + 1;
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) => (
    left < right ? -1 : left > right ? 1 : 0
  )));
}

function safeRepositoryPath(value) {
  return typeof value === 'string'
    && value.length > 0
    && value.length <= 2048
    && !path.isAbsolute(value)
    && !value.split('/').includes('..')
    && !value.includes('\\')
    && !value.includes('://');
}

function repositoryFile(value, root) {
  if (!safeRepositoryPath(value)) return null;
  const candidate = path.resolve(root, value);
  const relative = path.relative(root, candidate);
  if (relative.startsWith('..') || path.isAbsolute(relative) || !fs.existsSync(candidate)) return null;
  if (!fs.statSync(candidate).isFile()) return null;
  const rootReal = fs.realpathSync(root);
  const fileReal = fs.realpathSync(candidate);
  if (fileReal !== rootReal && !fileReal.startsWith(`${rootReal}${path.sep}`)) return null;
  return candidate;
}

function rejectUnknownFields(value, allowed, location, errors) {
  if (!isObject(value)) {
    errors.push(`${location} must be an object`);
    return;
  }
  for (const key of Object.keys(value)) {
    if (!allowed.includes(key)) errors.push(`${location}.${key} is not allowed`);
  }
}

function requireFields(value, required, location, errors) {
  if (!isObject(value)) return;
  for (const field of required) {
    if (!Object.hasOwn(value, field)) errors.push(`${location}.${field} is required`);
  }
}

function validateString(value, location, errors, { nullable = false, identifier = false } = {}) {
  if (nullable && value === null) return;
  if (typeof value !== 'string' || value.length === 0 || value.length > 2048) {
    errors.push(`${location} must be a non-empty string of at most 2,048 characters`);
  } else if (identifier && !IDENTIFIER_PATTERN.test(value)) {
    errors.push(`${location} must be a stable lowercase identifier`);
  }
}

function validateUniqueStrings(values, location, errors, maximum, { minimum = 0 } = {}) {
  if (!Array.isArray(values) || values.length < minimum || values.length > maximum) {
    errors.push(`${location} must contain ${minimum}..${maximum} values`);
    return;
  }
  if (new Set(values).size !== values.length) errors.push(`${location} values must be unique`);
  values.forEach((value, index) => validateString(
    value,
    `${location}[${index}]`,
    errors,
    { identifier: true },
  ));
}

function validatePathDigest(item, root, location, errors) {
  rejectUnknownFields(item, ['path', 'sha256'], location, errors);
  requireFields(item, ['path', 'sha256'], location, errors);
  if (!safeRepositoryPath(item?.path) || !SHA256_PATTERN.test(item?.sha256 ?? '')) {
    errors.push(`${location} must contain a safe repository path and SHA-256`);
    return;
  }
  const file = repositoryFile(item.path, root);
  if (!file || sha256File(file) !== item.sha256) {
    errors.push(`${location} digest does not match ${item.path}`);
  }
}

function validateConditions(conditions, location, errors) {
  rejectUnknownFields(
    conditions,
    ['languageSlot', 'altType', 'availability', 'profile'],
    location,
    errors,
  );
  for (const [key, value] of Object.entries(conditions ?? {})) {
    validateString(value, `${location}.${key}`, errors, { nullable: true });
  }
}

function validateFontPlan(plan, location, errors) {
  rejectUnknownFields(plan, ['sources', 'unavailableFonts'], location, errors);
  requireFields(plan, ['sources', 'unavailableFonts'], location, errors);
  if (!Array.isArray(plan?.sources) || plan.sources.length > 153) {
    errors.push(`${location}.sources exceeds the 153-font bound`);
  } else {
    plan.sources.forEach((source, index) => {
      const sourceLocation = `${location}.sources[${index}]`;
      rejectUnknownFields(source, ['url', 'aliases'], sourceLocation, errors);
      requireFields(source, ['url', 'aliases'], sourceLocation, errors);
      validateString(source?.url, `${sourceLocation}.url`, errors);
      if (!(source?.url?.startsWith('https://') || source?.url?.startsWith('fonts/'))) {
        errors.push(`${sourceLocation}.url violates the local/HTTPS boundary`);
      }
      if (!Array.isArray(source?.aliases) || source.aliases.length === 0
          || source.aliases.length > 153
          || new Set(source.aliases).size !== source.aliases.length) {
        errors.push(`${sourceLocation}.aliases must contain 1..153 unique names`);
      } else {
        source.aliases.forEach((alias, aliasIndex) => (
          validateString(alias, `${sourceLocation}.aliases[${aliasIndex}]`, errors)
        ));
      }
    });
  }
  if (!Array.isArray(plan?.unavailableFonts) || plan.unavailableFonts.length > 153
      || new Set(plan.unavailableFonts ?? []).size !== (plan.unavailableFonts?.length ?? -1)) {
    errors.push(`${location}.unavailableFonts must contain at most 153 unique names`);
  } else {
    plan.unavailableFonts.forEach((fontName, index) => (
      validateString(fontName, `${location}.unavailableFonts[${index}]`, errors)
    ));
  }
}

function validateSupply(supply, location, errors) {
  if (supply === null) return;
  if (supply?.kind === 'canvas2d-webfont') {
    rejectUnknownFields(
      supply,
      ['kind', 'fontFamily', 'sourceUrl', 'format', 'unicodeRange', 'external'],
      location,
      errors,
    );
    requireFields(
      supply,
      ['kind', 'fontFamily', 'sourceUrl', 'format', 'unicodeRange', 'external'],
      location,
      errors,
    );
    validateString(supply.fontFamily, `${location}.fontFamily`, errors);
    validateString(supply.sourceUrl, `${location}.sourceUrl`, errors);
    validateString(supply.unicodeRange, `${location}.unicodeRange`, errors, { nullable: true });
    if (!['woff2', 'woff', 'truetype', 'opentype'].includes(supply.format)) {
      errors.push(`${location}.format is invalid`);
    }
    if (typeof supply.external !== 'boolean') errors.push(`${location}.external must be boolean`);
  } else if (supply?.kind === 'canvaskit-plan') {
    rejectUnknownFields(
      supply,
      [
        'kind',
        'fontFamily',
        'declaredCapability',
        'runtimePlanStatus',
        'capabilityAgreement',
        'online',
        'offline',
      ],
      location,
      errors,
    );
    requireFields(
      supply,
      [
        'kind',
        'fontFamily',
        'declaredCapability',
        'runtimePlanStatus',
        'capabilityAgreement',
        'online',
        'offline',
      ],
      location,
      errors,
    );
    validateString(supply.fontFamily, `${location}.fontFamily`, errors);
    if (!['sfnt-source', 'unavailable'].includes(supply.declaredCapability)) {
      errors.push(`${location}.declaredCapability is invalid`);
    }
    if (!['planned', 'unavailable'].includes(supply.runtimePlanStatus)) {
      errors.push(`${location}.runtimePlanStatus is invalid`);
    }
    if (typeof supply.capabilityAgreement !== 'boolean') {
      errors.push(`${location}.capabilityAgreement must be boolean`);
    }
    validateFontPlan(supply.online, `${location}.online`, errors);
    validateFontPlan(supply.offline, `${location}.offline`, errors);
  } else {
    errors.push(`${location} kind is invalid`);
  }
}

function validateLegacyEvidence(evidence, location, errors) {
  if (evidence === null) return;
  rejectUnknownFields(
    evidence,
    [
      'w1RuleId',
      'candidateIds',
      'sourceBoundaryIds',
      'evidenceStatus',
      'baselineProjectionSha256',
    ],
    location,
    errors,
  );
  requireFields(
    evidence,
    [
      'w1RuleId',
      'candidateIds',
      'sourceBoundaryIds',
      'evidenceStatus',
      'baselineProjectionSha256',
    ],
    location,
    errors,
  );
  validateString(evidence?.w1RuleId, `${location}.w1RuleId`, errors, { identifier: true });
  validateUniqueStrings(evidence?.candidateIds, `${location}.candidateIds`, errors, 8, { minimum: 1 });
  validateUniqueStrings(
    evidence?.sourceBoundaryIds,
    `${location}.sourceBoundaryIds`,
    errors,
    8,
    { minimum: 1 },
  );
  if (!['verified-by-bytes', 'verified-by-test', 'historical', 'unknown']
    .includes(evidence?.evidenceStatus)) {
    errors.push(`${location}.evidenceStatus is invalid`);
  }
  if (!SHA256_PATTERN.test(evidence?.baselineProjectionSha256 ?? '')) {
    errors.push(`${location}.baselineProjectionSha256 is invalid`);
  }
}

function validateEvidenceRecord(record, location, errors, root) {
  rejectUnknownFields(
    record,
    ['evidenceId', 'kind', 'source', 'sha256', 'licenseOrProvenance', 'parentEvidenceIds'],
    location,
    errors,
  );
  requireFields(record, ['evidenceId', 'kind', 'source', 'sha256', 'parentEvidenceIds'], location, errors);
  validateString(record?.evidenceId, `${location}.evidenceId`, errors, { identifier: true });
  if (!['sealed-v1', 'public-fixture', 'public-source', 'aggregate-measurement']
    .includes(record?.kind)) {
    errors.push(`${location}.kind is invalid`);
  }
  validateString(record?.source, `${location}.source`, errors);
  if (!SHA256_PATTERN.test(record?.sha256 ?? '')) errors.push(`${location}.sha256 is invalid`);
  validateString(
    record?.licenseOrProvenance,
    `${location}.licenseOrProvenance`,
    errors,
    { nullable: true },
  );
  validateUniqueStrings(
    record?.parentEvidenceIds,
    `${location}.parentEvidenceIds`,
    errors,
    16,
  );
  if (record?.source?.startsWith('https://')) {
    if (!record.licenseOrProvenance) {
      errors.push(`${location} external evidence requires licenseOrProvenance`);
    }
  } else {
    const file = repositoryFile(record?.source, root);
    if (!file) {
      errors.push(`${location}.source uses an unsafe path or path traversal`);
    } else if (SHA256_PATTERN.test(record?.sha256 ?? '') && sha256File(file) !== record.sha256) {
      errors.push(`${location}.sha256 does not match ${record.source}`);
    }
  }
}

function validateEvidenceGraph(records, externalEvidenceIds, location, errors) {
  const recordById = new Map();
  for (const [index, record] of records.entries()) {
    if (!isObject(record)) continue;
    if (recordById.has(record.evidenceId)) {
      errors.push(`${location}[${index}].evidenceId is duplicated`);
    }
    recordById.set(record.evidenceId, record);
  }
  const allowedIds = new Set([...externalEvidenceIds, ...recordById.keys()]);
  for (const [index, record] of records.entries()) {
    for (const parentId of record.parentEvidenceIds ?? []) {
      if (parentId === record.evidenceId) {
        errors.push(`${location}[${index}] has an evidence self-parent cycle`);
      } else if (!allowedIds.has(parentId)) {
        errors.push(`${location}[${index}] has a dangling evidence parent ${parentId}`);
      }
    }
  }
  const visiting = new Set();
  const visited = new Set();
  function visit(evidenceId) {
    if (visiting.has(evidenceId)) return true;
    if (visited.has(evidenceId) || !recordById.has(evidenceId)) return false;
    visiting.add(evidenceId);
    const cyclic = (recordById.get(evidenceId).parentEvidenceIds ?? []).some(visit);
    visiting.delete(evidenceId);
    visited.add(evidenceId);
    return cyclic;
  }
  if ([...recordById.keys()].some(visit)) errors.push(`${location} contains an evidence cycle`);
}

function validateProjection(projection, location, errors) {
  rejectUnknownFields(projection, ['id', 'mode'], location, errors);
  requireFields(projection, ['id', 'mode'], location, errors);
  if (!PROJECTION_IDS.includes(projection?.id)) errors.push(`${location}.id is invalid`);
  if (!['direct', 'legacy-preservation'].includes(projection?.mode)) {
    errors.push(`${location}.mode is invalid`);
  }
}

function validateRulePayload(rule, location, errors) {
  const fields = [
    'ruleId',
    'relationType',
    'decisionPlane',
    'sourceFace',
    'targetFaceOrPolicy',
    'conditions',
    'order',
    'projection',
    'projectionSequence',
    'metricEntryIds',
    'supply',
    'evidenceIds',
  ];
  rejectUnknownFields(rule, fields, location, errors);
  requireFields(rule, fields, location, errors);
  validateString(rule?.ruleId, `${location}.ruleId`, errors, { identifier: true });
  if (!RELATION_TYPES.includes(rule?.relationType)) errors.push(`${location}.relationType is invalid`);
  if (!DECISION_PLANES.includes(rule?.decisionPlane)) errors.push(`${location}.decisionPlane is invalid`);
  validateString(rule?.sourceFace, `${location}.sourceFace`, errors, { nullable: true });
  validateString(rule?.targetFaceOrPolicy, `${location}.targetFaceOrPolicy`, errors);
  validateConditions(rule?.conditions, `${location}.conditions`, errors);
  if (!(rule?.order === null || (Number.isInteger(rule?.order) && rule.order >= 0))) {
    errors.push(`${location}.order must be null or a non-negative integer`);
  }
  validateProjection(rule?.projection, `${location}.projection`, errors);
  if (!Number.isInteger(rule?.projectionSequence)
      || rule.projectionSequence < 0
      || rule.projectionSequence > 4095) {
    errors.push(`${location}.projectionSequence must be in 0..4095`);
  }
  validateUniqueStrings(rule?.metricEntryIds, `${location}.metricEntryIds`, errors, 600);
  validateSupply(rule?.supply, `${location}.supply`, errors);
  validateUniqueStrings(rule?.evidenceIds, `${location}.evidenceIds`, errors, 16);
}

function selectionTupleFromPayload(rule) {
  const projection = rule.projection ?? rule.projections?.[0];
  return {
    relationType: rule.relationType,
    decisionPlane: rule.decisionPlane,
    sourceFace: rule.sourceFace,
    targetFaceOrPolicy: rule.targetFaceOrPolicy,
    conditions: rule.conditions,
    order: rule.order,
    projection,
    metricEntryIds: rule.metricEntryIds,
    supply: rule.supply,
  };
}

export function selectionTupleSha256(rule) {
  return sha256Text(canonicalJson(selectionTupleFromPayload(rule)));
}

function validateLifecycle(lifecycle, location, errors) {
  const fields = [
    'introducedBy',
    'lastEvidenceChangeBy',
    'retiredBy',
    'retirementReason',
    'successorRuleIds',
    'predecessorRuleIds',
  ];
  rejectUnknownFields(lifecycle, fields, location, errors);
  requireFields(lifecycle, fields, location, errors);
  validateString(lifecycle?.introducedBy, `${location}.introducedBy`, errors, { identifier: true });
  validateString(
    lifecycle?.lastEvidenceChangeBy,
    `${location}.lastEvidenceChangeBy`,
    errors,
    { nullable: true, identifier: lifecycle?.lastEvidenceChangeBy !== null },
  );
  validateString(
    lifecycle?.retiredBy,
    `${location}.retiredBy`,
    errors,
    { nullable: true, identifier: lifecycle?.retiredBy !== null },
  );
  validateString(
    lifecycle?.retirementReason,
    `${location}.retirementReason`,
    errors,
    { nullable: true },
  );
  validateUniqueStrings(lifecycle?.successorRuleIds, `${location}.successorRuleIds`, errors, 8);
  validateUniqueStrings(lifecycle?.predecessorRuleIds, `${location}.predecessorRuleIds`, errors, 8);
}

function activeRules(registry) {
  return (registry.rules ?? []).filter(rule => isObject(rule) && rule.status === 'active');
}

function activeProjectionSemantics(registry, projectionId) {
  return activeRules(registry)
    .filter(rule => rule.projections[0].id === projectionId)
    .sort((left, right) => left.projectionSequence - right.projectionSequence)
    .map(rule => ({
      ruleId: rule.ruleId,
      selectionTupleSha256: rule.selectionTupleSha256,
      projectionSequence: rule.projectionSequence,
    }));
}

function refreshRegistry(registry) {
  const active = activeRules(registry);
  registry.summary = {
    ruleCount: registry.rules.length,
    activeRuleCount: active.length,
    retiredRuleCount: registry.rules.length - active.length,
    countsByProjection: countBy(active.map(rule => rule.projections[0].id)),
  };
  registry.rulesSha256 = sha256Text(canonicalJson(registry.rules));
  return registry;
}

function validateRule(rule, index, errors) {
  const location = `registry.rules[${index}]`;
  const fields = [
    'ruleId',
    'relationType',
    'decisionPlane',
    'sourceFace',
    'targetFaceOrPolicy',
    'conditions',
    'order',
    'projections',
    'projectionSequence',
    'metricEntryIds',
    'supply',
    'evidence',
    'evidenceIds',
    'selectionTupleSha256',
    'status',
    'lifecycle',
  ];
  rejectUnknownFields(rule, fields, location, errors);
  if (!isObject(rule)) return;
  requireFields(rule, fields, location, errors);
  validateString(rule?.ruleId, `${location}.ruleId`, errors, { identifier: true });
  if (!RELATION_TYPES.includes(rule?.relationType)) errors.push(`${location}.relationType is invalid`);
  if (!DECISION_PLANES.includes(rule?.decisionPlane)) errors.push(`${location}.decisionPlane is invalid`);
  validateString(rule?.sourceFace, `${location}.sourceFace`, errors, { nullable: true });
  validateString(rule?.targetFaceOrPolicy, `${location}.targetFaceOrPolicy`, errors);
  validateConditions(rule?.conditions, `${location}.conditions`, errors);
  if (!(rule?.order === null || (Number.isInteger(rule?.order) && rule.order >= 0))) {
    errors.push(`${location}.order must be null or a non-negative integer`);
  }
  if (!Array.isArray(rule?.projections) || rule.projections.length !== 1) {
    errors.push(`${location}.projections must contain exactly one projection`);
  } else {
    validateProjection(rule.projections[0], `${location}.projections[0]`, errors);
  }
  if (!Number.isInteger(rule?.projectionSequence)
      || rule.projectionSequence < 0
      || rule.projectionSequence > 4095) {
    errors.push(`${location}.projectionSequence must be in 0..4095`);
  }
  validateUniqueStrings(rule?.metricEntryIds, `${location}.metricEntryIds`, errors, 600);
  validateSupply(rule?.supply, `${location}.supply`, errors);
  validateLegacyEvidence(rule?.evidence, `${location}.evidence`, errors);
  validateUniqueStrings(rule?.evidenceIds, `${location}.evidenceIds`, errors, 16);
  if (!SHA256_PATTERN.test(rule?.selectionTupleSha256 ?? '')
      || rule.selectionTupleSha256 !== selectionTupleSha256(rule)) {
    errors.push(`${location}.selectionTupleSha256 does not match the immutable tuple`);
  }
  if (!['active', 'retired'].includes(rule?.status)) errors.push(`${location}.status is invalid`);
  validateLifecycle(rule?.lifecycle, `${location}.lifecycle`, errors);
  const projection = rule?.projections?.[0];
  if (projection && !PROJECTION_PLANES[projection.id]?.has(rule.decisionPlane)) {
    errors.push(`${location} has a cross-plane projection`);
  }
  if (rule.status === 'active') {
    if (rule.lifecycle?.retiredBy !== null
        || rule.lifecycle?.retirementReason !== null
        || (rule.lifecycle?.successorRuleIds?.length ?? 0) !== 0) {
      errors.push(`${location} active lifecycle contains retirement state`);
    }
  } else if (!rule.lifecycle?.retiredBy || !rule.lifecycle?.retirementReason) {
    errors.push(`${location} retired lifecycle is incomplete`);
  }
}

function validateSuccessorGraph(rules, errors) {
  const byId = new Map(rules.filter(isObject).map(rule => [rule.ruleId, rule]));
  for (const rule of rules) {
    if (!isObject(rule)) continue;
    for (const successorId of rule.lifecycle?.successorRuleIds ?? []) {
      const successor = byId.get(successorId);
      if (!successor) {
        errors.push(`${rule.ruleId} has dangling successor ${successorId}`);
      } else if (successor.decisionPlane !== rule.decisionPlane) {
        errors.push(`${rule.ruleId} has cross-plane successor ${successorId}`);
      } else if (!(successor.lifecycle?.predecessorRuleIds ?? []).includes(rule.ruleId)) {
        errors.push(`${rule.ruleId}/${successorId} predecessor/successor links disagree`);
      }
    }
    for (const predecessorId of rule.lifecycle?.predecessorRuleIds ?? []) {
      const predecessor = byId.get(predecessorId);
      if (!(predecessor?.lifecycle?.successorRuleIds ?? []).includes(rule.ruleId)) {
        errors.push(`${predecessorId}/${rule.ruleId} predecessor/successor links disagree`);
      }
    }
  }
  const visiting = new Set();
  const visited = new Set();
  function visit(ruleId) {
    if (visiting.has(ruleId)) return true;
    if (visited.has(ruleId) || !byId.has(ruleId)) return false;
    visiting.add(ruleId);
    const cyclic = (byId.get(ruleId).lifecycle?.successorRuleIds ?? []).some(visit);
    visiting.delete(ruleId);
    visited.add(ruleId);
    return cyclic;
  }
  if ([...byId.keys()].some(visit)) errors.push('registry lifecycle successor graph contains a cycle');
}

export function assertSealedV1Artifacts(root = ROOT) {
  const errors = [];
  for (const [artifactPath, expectedSha256] of Object.entries(SEALED_V1_ARTIFACTS)) {
    const file = path.join(root, artifactPath);
    if (!fs.existsSync(file)) {
      errors.push(`sealed v1 artifact missing: ${artifactPath}`);
      continue;
    }
    const actualSha256 = sha256File(file);
    if (actualSha256 !== expectedSha256) {
      errors.push(
        `sealed v1 artifact changed: ${artifactPath} expected ${expectedSha256}, got ${actualSha256}`,
      );
    }
  }
  return errors;
}

export function validateRegistryV2(registry, root = ROOT) {
  const errors = [];
  if (registry?.kind !== 'canonical-font-rule-lifecycle-registry'
      || registry.schemaVersion !== '2.0'
      || registry.issue !== 5955) {
    return ['v2 registry envelope is invalid'];
  }
  const envelopeFields = [
    'schemaVersion',
    'kind',
    'issue',
    'sourceCommit',
    'schema',
    'sealedV1Registry',
    'appliedChangeSets',
    'summary',
    'evidenceRecords',
    'rulesSha256',
    'rules',
  ];
  rejectUnknownFields(registry, envelopeFields, 'registry', errors);
  requireFields(registry, envelopeFields, 'registry', errors);
  if (!SHA1_PATTERN.test(registry.sourceCommit ?? '')) {
    errors.push('registry.sourceCommit must be a lowercase 40-character Git SHA');
  }
  validatePathDigest(registry.schema, root, 'registry.schema', errors);
  validatePathDigest(registry.sealedV1Registry, root, 'registry.sealedV1Registry', errors);
  if (registry.sealedV1Registry?.sha256 !== SEALED_V1_ARTIFACTS[
    'assets/font-rules/font_rule_registry.json'
  ]) {
    errors.push('registry.sealedV1Registry is not the approved v1 registry');
  }
  if (!Array.isArray(registry.appliedChangeSets)
      || registry.appliedChangeSets.length > 4096) {
    errors.push('registry.appliedChangeSets exceeds the 4,096 entry bound');
  } else {
    const ids = new Set();
    registry.appliedChangeSets.forEach((reference, index) => {
      const location = `registry.appliedChangeSets[${index}]`;
      rejectUnknownFields(reference, ['changeSetId', 'path', 'sha256'], location, errors);
      requireFields(reference, ['changeSetId', 'path', 'sha256'], location, errors);
      validateString(reference?.changeSetId, `${location}.changeSetId`, errors, { identifier: true });
      if (ids.has(reference?.changeSetId)) errors.push(`${location}.changeSetId is duplicated`);
      ids.add(reference?.changeSetId);
      if (!SHA256_PATTERN.test(reference?.sha256 ?? '')) errors.push(`${location}.sha256 is invalid`);
      if (reference?.path !== null) {
        validateString(reference.path, `${location}.path`, errors);
        const file = repositoryFile(reference.path, root);
        if (!file || sha256File(file) !== reference.sha256) {
          errors.push(`${location}.path or digest is invalid`);
        }
      }
    });
  }
  rejectUnknownFields(
    registry.summary,
    ['ruleCount', 'activeRuleCount', 'retiredRuleCount', 'countsByProjection'],
    'registry.summary',
    errors,
  );
  requireFields(
    registry.summary,
    ['ruleCount', 'activeRuleCount', 'retiredRuleCount', 'countsByProjection'],
    'registry.summary',
    errors,
  );
  const records = registry.evidenceRecords ?? [];
  if (!Array.isArray(records) || records.length > 4096) {
    errors.push('registry.evidenceRecords exceeds the 4,096 entry bound');
  } else {
    records.forEach((record, index) => (
      validateEvidenceRecord(record, `registry.evidenceRecords[${index}]`, errors, root)
    ));
    validateEvidenceGraph(records, new Set(), 'registry.evidenceRecords', errors);
  }
  const evidenceIds = new Set(records.filter(isObject).map(record => record.evidenceId));
  const rules = registry.rules ?? [];
  if (!Array.isArray(rules) || rules.length > 4096) {
    errors.push('registry.rules exceeds the 4,096 rule bound');
  } else {
    rules.forEach((rule, index) => validateRule(rule, index, errors));
  }
  if (new Set(rules.filter(isObject).map(rule => rule.ruleId)).size !== rules.length) {
    errors.push('registry ruleId values must be globally unique across all lifecycle states');
  }
  for (const rule of rules) {
    if (!isObject(rule)) continue;
    if ((rule.evidenceIds ?? []).some(evidenceId => !evidenceIds.has(evidenceId))) {
      errors.push(`${rule.ruleId} has a dangling evidence reference`);
    }
  }
  validateSuccessorGraph(rules, errors);
  for (const projectionId of PROJECTION_IDS) {
    const sequences = activeRules(registry)
      .filter(rule => rule.projections?.[0]?.id === projectionId)
      .map(rule => rule.projectionSequence)
      .sort((left, right) => left - right);
    if (new Set(sequences).size !== sequences.length
        || sequences.some((sequence, index) => sequence !== index)) {
      errors.push(`${projectionId} active projection sequences must be unique and contiguous`);
    }
  }
  const actualSummary = refreshRegistry(clone(registry)).summary;
  if (canonicalJson(registry.summary) !== canonicalJson(actualSummary)) {
    errors.push('registry summary does not match lifecycle state');
  }
  if (registry.rulesSha256 !== sha256Text(canonicalJson(rules))) {
    errors.push('registry.rulesSha256 mismatch');
  }
  return errors;
}

function validateExpectedDelta(expectedDelta, decisionPlane, location, errors) {
  rejectUnknownFields(
    expectedDelta,
    ['projectionId', 'activeRuleDelta', 'unchangedProjectionIds'],
    location,
    errors,
  );
  requireFields(
    expectedDelta,
    ['projectionId', 'activeRuleDelta', 'unchangedProjectionIds'],
    location,
    errors,
  );
  if (!PROJECTION_IDS.includes(expectedDelta?.projectionId)) {
    errors.push(`${location}.projectionId is invalid`);
  } else if (!PROJECTION_PLANES[expectedDelta.projectionId].has(decisionPlane)) {
    errors.push(`${location} declares a cross-plane projection`);
  }
  if (!Number.isInteger(expectedDelta?.activeRuleDelta)
      || expectedDelta.activeRuleDelta < -64
      || expectedDelta.activeRuleDelta > 64) {
    errors.push(`${location}.activeRuleDelta must be in -64..64`);
  }
  if (!Array.isArray(expectedDelta?.unchangedProjectionIds)
      || expectedDelta.unchangedProjectionIds.length !== 4
      || new Set(expectedDelta.unchangedProjectionIds).size !== 4
      || expectedDelta.unchangedProjectionIds.includes(expectedDelta.projectionId)
      || expectedDelta.unchangedProjectionIds.some(id => !PROJECTION_IDS.includes(id))) {
    errors.push(`${location}.unchangedProjectionIds must name the other four projections`);
  }
}

function operationRuleIds(operation) {
  if (!isObject(operation)) return [];
  if (operation.type === 'augment-evidence' || operation.type === 'retire-rule') {
    return [operation.ruleId];
  }
  if (operation.type === 'add-rule') return [operation.rule?.ruleId];
  if (operation.type === 'retire-and-replace') {
    return [operation.retiredRuleId, operation.replacementRule?.ruleId];
  }
  return [];
}

function validateOperation(operation, index, changeSet, knownEvidenceIds, errors) {
  const location = `changeSet.operations[${index}]`;
  if (!OPERATION_TYPES.includes(operation?.type)) {
    errors.push(`${location} attempts an in-place semantic mutation or unknown operation`);
    return;
  }
  const commonFields = ['operationId', 'type'];
  const fieldsByType = {
    'augment-evidence': [
      ...commonFields,
      'ruleId',
      'expectedSelectionTupleSha256',
      'evidenceIds',
    ],
    'add-rule': [...commonFields, 'rule'],
    'retire-rule': [
      ...commonFields,
      'ruleId',
      'expectedSelectionTupleSha256',
      'retirementReason',
      'evidenceIds',
    ],
    'retire-and-replace': [
      ...commonFields,
      'retiredRuleId',
      'expectedSelectionTupleSha256',
      'replacementRule',
      'retirementReason',
      'evidenceIds',
    ],
  };
  rejectUnknownFields(operation, fieldsByType[operation.type], location, errors);
  requireFields(operation, fieldsByType[operation.type], location, errors);
  validateString(operation.operationId, `${location}.operationId`, errors, { identifier: true });
  if (operation.type === 'add-rule') {
    validateRulePayload(operation.rule, `${location}.rule`, errors);
    if (operation.rule?.decisionPlane !== changeSet.decisionPlane) {
      errors.push(`${location} has a cross-plane rule`);
    }
    if (operation.rule?.projection?.id !== changeSet.expectedDelta?.projectionId) {
      errors.push(`${location} targets a projection outside expectedDelta`);
    }
    if ((operation.rule?.evidenceIds ?? []).some(evidenceId => !knownEvidenceIds.has(evidenceId))) {
      errors.push(`${location} has a dangling evidence reference`);
    }
  } else {
    const ruleId = operation.type === 'retire-and-replace'
      ? operation.retiredRuleId
      : operation.ruleId;
    validateString(ruleId, `${location}.ruleId`, errors, { identifier: true });
    if (!SHA256_PATTERN.test(operation.expectedSelectionTupleSha256 ?? '')) {
      errors.push(`${location}.expectedSelectionTupleSha256 is invalid`);
    }
    if (operation.type === 'retire-rule' || operation.type === 'retire-and-replace') {
      validateString(operation.retirementReason, `${location}.retirementReason`, errors);
    }
    validateUniqueStrings(operation.evidenceIds, `${location}.evidenceIds`, errors, 16, { minimum: 1 });
    if ((operation.evidenceIds ?? []).some(evidenceId => !knownEvidenceIds.has(evidenceId))) {
      errors.push(`${location} has a dangling evidence reference`);
    }
    if (operation.type === 'retire-and-replace') {
      validateRulePayload(operation.replacementRule, `${location}.replacementRule`, errors);
      if (operation.replacementRule?.decisionPlane !== changeSet.decisionPlane) {
        errors.push(`${location} has a cross-plane replacement`);
      }
      if (operation.replacementRule?.projection?.id !== changeSet.expectedDelta?.projectionId) {
        errors.push(`${location} replacement targets a projection outside expectedDelta`);
      }
      if ((operation.replacementRule?.evidenceIds ?? [])
        .some(evidenceId => !knownEvidenceIds.has(evidenceId))) {
        errors.push(`${location} replacement has a dangling evidence reference`);
      }
    }
  }
}

export function validateChangeSet(changeSet, options = {}) {
  const root = options.root ?? ROOT;
  const errors = [];
  if (changeSet?.kind !== 'font-rule-change-set' || changeSet.schemaVersion !== '2.0') {
    return ['changeSet envelope is invalid'];
  }
  const fields = [
    'schemaVersion',
    'kind',
    'changeSetId',
    'issue',
    'sequence',
    'parentRegistrySha256',
    'decisionPlane',
    'evidenceRecords',
    'operations',
    'expectedDelta',
  ];
  rejectUnknownFields(changeSet, fields, 'changeSet', errors);
  requireFields(changeSet, fields, 'changeSet', errors);
  validateString(changeSet.changeSetId, 'changeSet.changeSetId', errors, { identifier: true });
  if (!Number.isInteger(changeSet.issue) || changeSet.issue < 1) {
    errors.push('changeSet.issue must be a positive integer');
  }
  if (!Number.isInteger(changeSet.sequence)
      || changeSet.sequence < 1
      || changeSet.sequence > 4096) {
    errors.push('changeSet.sequence must be in 1..4096');
  }
  if (!SHA256_PATTERN.test(changeSet.parentRegistrySha256 ?? '')) {
    errors.push('changeSet.parentRegistrySha256 is invalid');
  }
  if (options.expectedParentRegistrySha256
      && changeSet.parentRegistrySha256 !== options.expectedParentRegistrySha256) {
    errors.push('changeSet has a stale parent registry digest');
  }
  if (options.expectedSequence && changeSet.sequence !== options.expectedSequence) {
    errors.push(`changeSet sequence gap: expected ${options.expectedSequence}`);
  }
  if (!DECISION_PLANES.includes(changeSet.decisionPlane)) {
    errors.push('changeSet.decisionPlane is invalid');
  }
  const records = changeSet.evidenceRecords ?? [];
  if (!Array.isArray(records) || records.length > 128) {
    errors.push('changeSet.evidenceRecords must contain at most 128 records');
  } else {
    records.forEach((record, index) => (
      validateEvidenceRecord(record, `changeSet.evidenceRecords[${index}]`, errors, root)
    ));
    validateEvidenceGraph(
      records,
      new Set(options.existingEvidenceIds ?? []),
      'changeSet.evidenceRecords',
      errors,
    );
  }
  const knownEvidenceIds = new Set([
    ...(options.existingEvidenceIds ?? []),
    ...records.filter(isObject).map(record => record.evidenceId),
  ]);
  const operations = changeSet.operations ?? [];
  if (!Array.isArray(operations) || operations.length === 0 || operations.length > 64) {
    errors.push('changeSet must contain at most 64 operations and at least one operation');
  } else {
    operations.forEach((operation, index) => (
      validateOperation(operation, index, changeSet, knownEvidenceIds, errors)
    ));
    const operationIds = operations.map(operation => operation?.operationId);
    if (new Set(operationIds).size !== operationIds.length) {
      errors.push('changeSet operationId values must be unique');
    }
    const touchedRuleIds = operations.flatMap(operationRuleIds).filter(Boolean);
    if (new Set(touchedRuleIds).size !== touchedRuleIds.length) {
      errors.push('changeSet cannot mutate one ruleId more than once');
    }
  }
  validateExpectedDelta(changeSet.expectedDelta, changeSet.decisionPlane, 'changeSet.expectedDelta', errors);
  return errors;
}

function v2RuleFromV1(rule, projectionSequence) {
  return {
    ...clone(rule),
    projectionSequence,
    evidenceIds: [SEALED_EVIDENCE_ID],
    selectionTupleSha256: selectionTupleSha256(rule),
    lifecycle: {
      introducedBy: MIGRATION_EVENT_ID,
      lastEvidenceChangeBy: null,
      retiredBy: null,
      retirementReason: null,
      successorRuleIds: [],
      predecessorRuleIds: [],
    },
  };
}

export function buildInitialRegistryV2(v1Registry, root = ROOT, options = {}) {
  if (sha256Text(canonicalJson(v1Registry)) !== SEALED_V1_ARTIFACTS[
    'assets/font-rules/font_rule_registry.json'
  ]) {
    throw new Error('initial v2 migration requires the sealed v1 registry bytes');
  }
  const sequenceByProjection = new Map(PROJECTION_IDS.map(id => [id, 0]));
  const rules = v1Registry.rules.map(rule => {
    const projectionId = rule.projections[0].id;
    const sequence = sequenceByProjection.get(projectionId);
    sequenceByProjection.set(projectionId, sequence + 1);
    return v2RuleFromV1(rule, sequence);
  });
  const registry = refreshRegistry({
    schemaVersion: '2.0',
    kind: 'canonical-font-rule-lifecycle-registry',
    issue: 5955,
    sourceCommit: options.sourceCommit ?? currentGitHead(root),
    schema: pathDigest(path.join(root, relativePath(REGISTRY_V2_SCHEMA_PATH)), root),
    sealedV1Registry: pathDigest(path.join(root, relativePath(REGISTRY_V1_PATH)), root),
    appliedChangeSets: [],
    summary: {},
    evidenceRecords: [
      {
        evidenceId: SEALED_EVIDENCE_ID,
        kind: 'sealed-v1',
        source: relativePath(REGISTRY_V1_PATH, root),
        sha256: sha256File(path.join(root, relativePath(REGISTRY_V1_PATH))),
        licenseOrProvenance: 'rhwp W7 sealed registry',
        parentEvidenceIds: [],
      },
    ],
    rulesSha256: '',
    rules,
  });
  const errors = validateRegistryV2(registry, root);
  if (errors.length > 0) throw new Error(errors.join('\n'));
  return registry;
}

function changeSetReference(changeSet, options) {
  const sourcePath = options.changeSetPaths?.get(changeSet.changeSetId) ?? null;
  return {
    changeSetId: changeSet.changeSetId,
    path: sourcePath,
    sha256: sha256Text(canonicalJson(changeSet)),
  };
}

function newRuleFromPayload(payload, changeSetId, predecessorRuleIds = []) {
  return {
    ruleId: payload.ruleId,
    relationType: payload.relationType,
    decisionPlane: payload.decisionPlane,
    sourceFace: payload.sourceFace,
    targetFaceOrPolicy: payload.targetFaceOrPolicy,
    conditions: clone(payload.conditions),
    order: payload.order,
    projections: [clone(payload.projection)],
    projectionSequence: payload.projectionSequence,
    metricEntryIds: clone(payload.metricEntryIds),
    supply: clone(payload.supply),
    evidence: null,
    evidenceIds: clone(payload.evidenceIds),
    selectionTupleSha256: selectionTupleSha256(payload),
    status: 'active',
    lifecycle: {
      introducedBy: changeSetId,
      lastEvidenceChangeBy: null,
      retiredBy: null,
      retirementReason: null,
      successorRuleIds: [],
      predecessorRuleIds,
    },
  };
}

function requireCurrentRule(ruleById, ruleId, operation, errors) {
  const rule = ruleById.get(ruleId);
  if (!rule) {
    errors.push(`${operation.operationId}: rule ${ruleId} does not exist`);
  } else if (rule.status !== 'active') {
    errors.push(`${operation.operationId}: rule ${ruleId} is already retired`);
  } else if (rule.selectionTupleSha256 !== operation.expectedSelectionTupleSha256) {
    errors.push(`${operation.operationId}: expected selection tuple is stale`);
  }
  return rule;
}

function validateCurrentRuleTarget(rule, operation, changeSet, errors) {
  if (!rule) return;
  if (rule.decisionPlane !== changeSet.decisionPlane
      || rule.projections[0].id !== changeSet.expectedDelta.projectionId) {
    errors.push(`${operation.operationId}: current rule crosses the declared decision plane`);
  }
}

function applyChangeSet(registry, changeSet, options) {
  const next = clone(registry);
  const errors = validateChangeSet(changeSet, {
    root: options.root,
    expectedParentRegistrySha256: sha256Text(canonicalJson(registry)),
    expectedSequence: registry.appliedChangeSets.length + 1,
    existingEvidenceIds: registry.evidenceRecords.map(record => record.evidenceId),
  });
  if (next.appliedChangeSets.some(reference => reference.changeSetId === changeSet.changeSetId)) {
    errors.push(`changeSet ${changeSet.changeSetId} is duplicated`);
  }
  const ruleById = new Map(next.rules.map(rule => [rule.ruleId, rule]));
  if ([...(changeSet.evidenceRecords ?? [])].some(record => (
    next.evidenceRecords.some(existing => existing.evidenceId === record.evidenceId)
  ))) {
    errors.push(`changeSet ${changeSet.changeSetId} reuses an evidenceId`);
  }
  if (errors.length > 0) throw new Error(errors.join('\n'));

  const beforeSemantics = Object.fromEntries(PROJECTION_IDS.map(projectionId => [
    projectionId,
    canonicalJson(activeProjectionSemantics(next, projectionId)),
  ]));
  next.evidenceRecords.push(...clone(changeSet.evidenceRecords));
  for (const operation of changeSet.operations) {
    if (operation.type === 'augment-evidence') {
      const rule = requireCurrentRule(ruleById, operation.ruleId, operation, errors);
      validateCurrentRuleTarget(rule, operation, changeSet, errors);
      if (rule) {
        rule.evidenceIds = [...new Set([...rule.evidenceIds, ...operation.evidenceIds])];
        rule.lifecycle.lastEvidenceChangeBy = changeSet.changeSetId;
      }
    } else if (operation.type === 'add-rule') {
      if (ruleById.has(operation.rule.ruleId)) {
        errors.push(`${operation.operationId}: ruleId ${operation.rule.ruleId} is already used`);
      } else {
        const rule = newRuleFromPayload(operation.rule, changeSet.changeSetId);
        next.rules.push(rule);
        ruleById.set(rule.ruleId, rule);
      }
    } else if (operation.type === 'retire-rule') {
      const rule = requireCurrentRule(ruleById, operation.ruleId, operation, errors);
      validateCurrentRuleTarget(rule, operation, changeSet, errors);
      if (rule) {
        const projectionRules = activeRules(next).filter(candidate => (
          candidate.projections[0].id === rule.projections[0].id
        ));
        const lastSequence = Math.max(...projectionRules.map(candidate => candidate.projectionSequence));
        if (rule.projectionSequence !== lastSequence) {
          errors.push(`${operation.operationId}: non-tail retirement would break immutable sequences`);
        }
        rule.status = 'retired';
        rule.evidenceIds = [...new Set([...rule.evidenceIds, ...operation.evidenceIds])];
        rule.lifecycle.retiredBy = changeSet.changeSetId;
        rule.lifecycle.retirementReason = operation.retirementReason;
      }
    } else if (operation.type === 'retire-and-replace') {
      const oldRule = requireCurrentRule(ruleById, operation.retiredRuleId, operation, errors);
      validateCurrentRuleTarget(oldRule, operation, changeSet, errors);
      if (ruleById.has(operation.replacementRule.ruleId)) {
        errors.push(`${operation.operationId}: replacement ruleId is already used`);
      } else if (oldRule) {
        const newRule = newRuleFromPayload(
          operation.replacementRule,
          changeSet.changeSetId,
          [oldRule.ruleId],
        );
        if (newRule.decisionPlane !== oldRule.decisionPlane
            || newRule.projections[0].id !== oldRule.projections[0].id) {
          errors.push(`${operation.operationId}: replacement crosses a decision plane or projection`);
        }
        if (newRule.projectionSequence !== oldRule.projectionSequence) {
          errors.push(`${operation.operationId}: replacement must inherit the active projection slot`);
        }
        if (newRule.selectionTupleSha256 === oldRule.selectionTupleSha256) {
          errors.push(`${operation.operationId}: replacement must have a new semantic tuple`);
        }
        oldRule.status = 'retired';
        oldRule.evidenceIds = [...new Set([...oldRule.evidenceIds, ...operation.evidenceIds])];
        oldRule.lifecycle.retiredBy = changeSet.changeSetId;
        oldRule.lifecycle.retirementReason = operation.retirementReason;
        oldRule.lifecycle.successorRuleIds = [newRule.ruleId];
        next.rules.push(newRule);
        ruleById.set(newRule.ruleId, newRule);
      }
    }
  }
  if (errors.length > 0) throw new Error(errors.join('\n'));
  next.appliedChangeSets.push(changeSetReference(changeSet, options));
  refreshRegistry(next);
  const targetProjection = changeSet.expectedDelta.projectionId;
  const beforeTargetCount = JSON.parse(beforeSemantics[targetProjection]).length;
  const afterTargetCount = activeProjectionSemantics(next, targetProjection).length;
  if (afterTargetCount - beforeTargetCount !== changeSet.expectedDelta.activeRuleDelta) {
    errors.push(`${changeSet.changeSetId}: active projection delta differs from declaration`);
  }
  for (const projectionId of changeSet.expectedDelta.unchangedProjectionIds) {
    if (beforeSemantics[projectionId]
        !== canonicalJson(activeProjectionSemantics(next, projectionId))) {
      errors.push(`${changeSet.changeSetId}: non-target projection ${projectionId} changed`);
    }
  }
  errors.push(...validateRegistryV2(next, options.root));
  if (errors.length > 0) throw new Error(errors.join('\n'));
  return next;
}

export function reduceRegistryV2(baseRegistry, changeSets = [], options = {}) {
  const root = options.root ?? ROOT;
  const initialErrors = validateRegistryV2(baseRegistry, root);
  if (initialErrors.length > 0) throw new Error(initialErrors.join('\n'));
  let registry = clone(baseRegistry);
  for (const changeSet of changeSets) {
    registry = applyChangeSet(registry, changeSet, {
      root,
      changeSetPaths: options.changeSetPaths,
    });
  }
  return registry;
}

export function projectActiveRules(registry, projectionId) {
  if (!PROJECTION_IDS.includes(projectionId)) throw new Error(`unknown projection: ${projectionId}`);
  return activeRules(registry)
    .filter(rule => rule.projections[0].id === projectionId)
    .sort((left, right) => left.projectionSequence - right.projectionSequence);
}

function migrationProjectionRowsV1(v1Registry, projectionId) {
  let sequence = 0;
  return v1Registry.rules
    .filter(rule => rule.projections[0].id === projectionId)
    .map(rule => ({
      ruleId: rule.ruleId,
      selectionTupleSha256: selectionTupleSha256(rule),
      projectionSequence: sequence++,
    }));
}

export function buildMigrationV1ToV2(v1Registry, options = {}) {
  const root = options.root ?? ROOT;
  const v2Registry = options.v2Registry ?? buildInitialRegistryV2(v1Registry, root, options);
  const mappings = v1Registry.rules.map((v1Rule, index) => {
    const v2Rule = v2Registry.rules[index];
    return {
      v1RuleId: v1Rule.ruleId,
      v2RuleId: v2Rule.ruleId,
      disposition: 'carry-forward',
      beforeRuleSha256: sha256Text(canonicalJson(v1Rule)),
      afterRuleSha256: sha256Text(canonicalJson(v2Rule)),
      beforeSelectionTupleSha256: selectionTupleSha256(v1Rule),
      afterSelectionTupleSha256: v2Rule.selectionTupleSha256,
    };
  });
  const projectionDeltas = PROJECTION_IDS.map(projectionId => {
    const beforeRows = migrationProjectionRowsV1(v1Registry, projectionId);
    const afterRows = activeProjectionSemantics(v2Registry, projectionId);
    return {
      projectionId,
      beforeCount: beforeRows.length,
      afterCount: afterRows.length,
      beforeSemanticSha256: sha256Text(canonicalJson(beforeRows)),
      afterSemanticSha256: sha256Text(canonicalJson(afterRows)),
      status: 'unchanged',
    };
  });
  const migration = {
    schemaVersion: '2.0',
    kind: 'font-rule-registry-v1-to-v2-migration',
    issue: 5955,
    sourceCommit: v2Registry.sourceCommit,
    schema: pathDigest(path.join(root, relativePath(MIGRATION_SCHEMA_PATH)), root),
    fromSchema: pathDigest(path.join(root, relativePath(REGISTRY_V1_SCHEMA_PATH)), root),
    fromRegistry: pathDigest(path.join(root, relativePath(REGISTRY_V1_PATH)), root),
    toSchema: pathDigest(path.join(root, relativePath(REGISTRY_V2_SCHEMA_PATH)), root),
    toRegistry: {
      path: relativePath(REGISTRY_V2_PATH, root),
      sha256: sha256Text(canonicalJson(v2Registry)),
    },
    summary: {
      v1RuleCount: v1Registry.rules.length,
      v2ActiveRuleCount: activeRules(v2Registry).length,
      v2RetiredRuleCount: v2Registry.rules.length - activeRules(v2Registry).length,
      carryForwardCount: mappings.filter(mapping => mapping.disposition === 'carry-forward').length,
    },
    mappings,
    projectionDeltas,
    mappingsSha256: sha256Text(canonicalJson(mappings)),
  };
  const errors = validateMigrationV1ToV2(migration, v1Registry, v2Registry, root);
  if (errors.length > 0) throw new Error(errors.join('\n'));
  return migration;
}

export function validateMigrationV1ToV2(migration, v1Registry, v2Registry, root = ROOT) {
  const errors = [];
  if (migration?.kind !== 'font-rule-registry-v1-to-v2-migration'
      || migration.schemaVersion !== '2.0'
      || migration.issue !== 5955
      || migration.sourceCommit !== v2Registry.sourceCommit) {
    return ['v1 to v2 migration envelope is invalid'];
  }
  const fields = [
    'schemaVersion',
    'kind',
    'issue',
    'sourceCommit',
    'schema',
    'fromSchema',
    'fromRegistry',
    'toSchema',
    'toRegistry',
    'summary',
    'mappings',
    'projectionDeltas',
    'mappingsSha256',
  ];
  rejectUnknownFields(migration, fields, 'migration', errors);
  requireFields(migration, fields, 'migration', errors);
  validatePathDigest(migration.schema, root, 'migration.schema', errors);
  validatePathDigest(migration.fromSchema, root, 'migration.fromSchema', errors);
  validatePathDigest(migration.fromRegistry, root, 'migration.fromRegistry', errors);
  validatePathDigest(migration.toSchema, root, 'migration.toSchema', errors);
  rejectUnknownFields(migration.toRegistry, ['path', 'sha256'], 'migration.toRegistry', errors);
  if (migration.toRegistry?.path !== relativePath(REGISTRY_V2_PATH, root)
      || migration.toRegistry?.sha256 !== sha256Text(canonicalJson(v2Registry))) {
    errors.push('migration.toRegistry path or digest is invalid');
  }
  rejectUnknownFields(
    migration.summary,
    ['v1RuleCount', 'v2ActiveRuleCount', 'v2RetiredRuleCount', 'carryForwardCount'],
    'migration.summary',
    errors,
  );
  if (migration.summary?.v1RuleCount !== 830
      || migration.summary?.v2ActiveRuleCount !== 830
      || migration.summary?.v2RetiredRuleCount !== 0
      || migration.summary?.carryForwardCount !== 830) {
    errors.push('initial migration must remain 830 active, 0 retired and 830 carry-forward');
  }
  const mappings = migration.mappings ?? [];
  if (!Array.isArray(mappings) || mappings.length !== 830) {
    errors.push('initial migration must contain exactly 830 mappings');
  } else {
    mappings.forEach((mapping, index) => {
      const v1Rule = v1Registry.rules[index];
      const v2Rule = v2Registry.rules[index];
      const expected = {
        v1RuleId: v1Rule.ruleId,
        v2RuleId: v2Rule.ruleId,
        disposition: 'carry-forward',
        beforeRuleSha256: sha256Text(canonicalJson(v1Rule)),
        afterRuleSha256: sha256Text(canonicalJson(v2Rule)),
        beforeSelectionTupleSha256: selectionTupleSha256(v1Rule),
        afterSelectionTupleSha256: v2Rule.selectionTupleSha256,
      };
      rejectUnknownFields(mapping, Object.keys(expected), `migration.mappings[${index}]`, errors);
      if (canonicalJson(mapping) !== canonicalJson(expected)) {
        errors.push(`migration mapping ${index} differs from the v1/v2 rules`);
      }
      if (mapping.beforeSelectionTupleSha256 !== mapping.afterSelectionTupleSha256) {
        errors.push(`migration mapping ${index} changes the selection tuple`);
      }
      if (canonicalJson(v2Rule?.evidence) !== canonicalJson(v1Rule?.evidence)
          || v2Rule?.status !== 'active'
          || canonicalJson(v2Rule?.evidenceIds) !== canonicalJson([SEALED_EVIDENCE_ID])
          || v2Rule?.lifecycle?.introducedBy !== MIGRATION_EVENT_ID
          || v2Rule?.lifecycle?.lastEvidenceChangeBy !== null
          || v2Rule?.lifecycle?.retiredBy !== null
          || v2Rule?.lifecycle?.retirementReason !== null
          || (v2Rule?.lifecycle?.successorRuleIds ?? []).length !== 0
          || (v2Rule?.lifecycle?.predecessorRuleIds ?? []).length !== 0) {
        errors.push(`migration mapping ${index} does not preserve the carry-forward lifecycle`);
      }
    });
  }
  if (migration.mappingsSha256 !== sha256Text(canonicalJson(mappings))) {
    errors.push('migration.mappingsSha256 mismatch');
  }
  const projectionDeltas = migration.projectionDeltas ?? [];
  if (!Array.isArray(projectionDeltas)
      || projectionDeltas.length !== 5
      || new Set(projectionDeltas.map(delta => delta?.projectionId)).size !== 5
      || projectionDeltas.some(delta => !PROJECTION_IDS.includes(delta?.projectionId))) {
    errors.push('migration must contain exactly five unique projection deltas');
  } else {
    for (const delta of projectionDeltas) {
      const beforeRows = migrationProjectionRowsV1(v1Registry, delta.projectionId);
      const afterRows = activeProjectionSemantics(v2Registry, delta.projectionId);
      if (delta.beforeCount !== beforeRows.length
          || delta.afterCount !== afterRows.length
          || delta.beforeSemanticSha256 !== sha256Text(canonicalJson(beforeRows))
          || delta.afterSemanticSha256 !== sha256Text(canonicalJson(afterRows))
          || delta.beforeSemanticSha256 !== delta.afterSemanticSha256
          || delta.status !== 'unchanged') {
        errors.push(`${delta.projectionId} migration semantic delta is not zero`);
      }
    }
  }
  return errors;
}

export function resolveRuleLifecycle() {
  throw new Error('W7.5 lifecycle resolver is reserved for Stage W7.5-4');
}

function compareJson(expected, actual, label) {
  return canonicalJson(expected) === canonicalJson(actual) ? [] : [`${label} differs`];
}

function writeCanonical(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, canonicalJson(value), 'utf8');
}

function main(args) {
  if (args.length !== 1 || !['generate', 'check'].includes(args[0])) {
    throw new Error('usage: node scripts/font_rule_registry_v2.mjs <generate|check>');
  }
  const sealedErrors = assertSealedV1Artifacts(ROOT);
  if (sealedErrors.length > 0) throw new Error(sealedErrors.join('\n'));
  const v1Registry = readJson(REGISTRY_V1_PATH);
  if (args[0] === 'generate') {
    const v2Registry = buildInitialRegistryV2(v1Registry, ROOT);
    const migration = buildMigrationV1ToV2(v1Registry, { root: ROOT, v2Registry });
    writeCanonical(REGISTRY_V2_PATH, v2Registry);
    writeCanonical(MIGRATION_PATH, migration);
    process.stdout.write(
      `font rule registry v2: ${v2Registry.summary.activeRuleCount} active, `
        + `${v2Registry.summary.retiredRuleCount} retired\n`,
    );
  } else {
    const expectedRegistry = readJson(REGISTRY_V2_PATH);
    const expectedMigration = readJson(MIGRATION_PATH);
    const actualRegistry = buildInitialRegistryV2(v1Registry, ROOT, {
      sourceCommit: expectedRegistry.sourceCommit,
    });
    const actualMigration = buildMigrationV1ToV2(v1Registry, {
      root: ROOT,
      v2Registry: actualRegistry,
    });
    const errors = [
      ...validateRegistryV2(expectedRegistry, ROOT),
      ...validateMigrationV1ToV2(expectedMigration, v1Registry, expectedRegistry, ROOT),
      ...compareJson(expectedRegistry, actualRegistry, 'canonical v2 registry'),
      ...compareJson(expectedMigration, actualMigration, 'v1 to v2 migration'),
    ];
    if (errors.length > 0) throw new Error(errors.join('\n'));
    process.stdout.write('font rule registry v2: ok\n');
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
