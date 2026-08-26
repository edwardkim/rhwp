#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from './font_rule_ledger.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const REGISTRY_PATH = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry.json');
const REGISTRY_SCHEMA_PATH = path.join(
  ROOT,
  'assets',
  'font-rules',
  'font_rule_registry.schema.json',
);
const INVESTIGATION_ROOT = path.join(ROOT, 'mydocs', 'tech', 'investigations');
const BASELINE_PATH = path.join(
  INVESTIGATION_ROOT,
  'issue-4966',
  'font_rule_projection_baseline.json',
);
const MIGRATION_PATH = path.join(
  INVESTIGATION_ROOT,
  'issue-4966',
  'font_rule_registry_migration.json',
);
const MIGRATION_SCHEMA_PATH = path.join(
  INVESTIGATION_ROOT,
  'issue-4966',
  'font_rule_registry_migration.schema.json',
);
const W1_CANDIDATES_PATH = path.join(
  INVESTIGATION_ROOT,
  'issue-4939',
  'font_rule_candidates.json',
);
const W1_LEDGER_PATH = path.join(
  INVESTIGATION_ROOT,
  'issue-4939',
  'font_rule_ledger.json',
);
const W6_LINEAGE_PATH = path.join(
  INVESTIGATION_ROOT,
  'issue-4964',
  'font_metric_lineage_manifest.json',
);

const PROJECTIONS = [
  ['rustLayoutName', 'rust-layout-name'],
  ['rustLayoutMetric', 'rust-layout-metric'],
  ['canvas2dPaint', 'canvas2d-paint'],
  ['webfontSupply', 'canvas2d-webfont'],
  ['canvasKitSfnt', 'canvaskit-sfnt'],
];

const PROJECTION_COUNTS = {
  'canvas2d-paint': 281,
  'canvas2d-webfont': 153,
  'canvaskit-sfnt': 158,
  'rust-layout-metric': 67,
  'rust-layout-name': 171,
};

const ALLOWLIST = {
  'rust-layout-name': {
    planes: new Set(['layout-name']),
    relations: new Set(['style-fallback']),
  },
  'rust-layout-metric': {
    planes: new Set(['layout-metric']),
    relations: new Set(['metric-surrogate', 'unknown']),
  },
  'canvas2d-paint': {
    planes: new Set(['paint']),
    relations: new Set([
      'document-substitution',
      'official-successor',
      'paint-substitute',
      'style-fallback',
      'generic-fallback',
    ]),
  },
  'canvas2d-webfont': {
    planes: new Set(['supply']),
    relations: new Set(['supply-source']),
  },
  'canvaskit-sfnt': {
    planes: new Set(['supply', 'detection']),
    relations: new Set(['supply-source', 'capability-detection']),
  },
};

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function sha256File(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function currentGitHead(root) {
  return execFileSync('git', ['rev-parse', 'HEAD'], {
    cwd: root,
    encoding: 'utf8',
  }).trim();
}

function relativePath(file, root = ROOT) {
  return path.relative(root, file).split(path.sep).join('/');
}

function pathDigest(file, root = ROOT) {
  return { path: relativePath(file, root), sha256: sha256File(file) };
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function countBy(values) {
  const counts = {};
  for (const value of values) counts[value] = (counts[value] ?? 0) + 1;
  return Object.fromEntries(
    Object.entries(counts).sort(([left], [right]) => compareText(left, right)),
  );
}

function parseWebfontRequests(runtime) {
  const unicodeRangeByFamily = new Map();
  for (const style of runtime.webfontLoad.css ?? []) {
    const blocks = style.textContent.match(/@font-face\s*\{[^}]+\}/gu) ?? [];
    for (const block of blocks) {
      const family = block.match(/font-family:\s*"([^"]+)"/u)?.[1];
      const unicodeRange = block.match(/unicode-range:\s*([^;]+);/u)?.[1]?.trim() ?? null;
      if (family) unicodeRangeByFamily.set(family, unicodeRange);
    }
  }
  return new Map((runtime.webfontLoad.requests ?? []).map(request => {
    const parsed = request.source.match(/^url\((.+)\) format\('([^']+)'\)$/u);
    if (!parsed) throw new Error(`cannot parse FontFace request for ${request.family}`);
    const sourceUrl = parsed[1].replace(/^['"]|['"]$/gu, '');
    return [request.family, {
      kind: 'canvas2d-webfont',
      fontFamily: request.family,
      sourceUrl,
      format: parsed[2],
      unicodeRange: unicodeRangeByFamily.get(request.family) ?? null,
      external: /^https?:\/\//u.test(sourceUrl),
    }];
  }));
}

function canvasKitSupply(rule, plansByFamily) {
  if (rule.conditions?.profile !== 'canvaskit-sfnt') return null;
  const plan = plansByFamily.get(rule.sourceFace);
  if (!plan) throw new Error(`CanvasKit runtime plan missing for ${rule.sourceFace}`);
  const declaredCapability = rule.targetFaceOrPolicy.startsWith('unavailable:')
    ? 'unavailable'
    : 'sfnt-source';
  const runtimePlanStatus = plan.online.sources.length > 0 ? 'planned' : 'unavailable';
  return {
    kind: 'canvaskit-plan',
    fontFamily: rule.sourceFace,
    declaredCapability,
    runtimePlanStatus,
    capabilityAgreement: (declaredCapability === 'sfnt-source') === (runtimePlanStatus === 'planned'),
    online: plan.online,
    offline: plan.offline,
  };
}

function sourceBoundaryIds(rule, candidateBoundaryById) {
  return [...new Set(rule.candidateIds.map(candidateId => {
    const boundaryId = candidateBoundaryById.get(candidateId);
    if (!boundaryId) throw new Error(`W1 candidate missing for ${rule.ruleId}: ${candidateId}`);
    return boundaryId;
  }))];
}

function buildRule(
  rule,
  projectionId,
  baselineProjectionSha256,
  candidateBoundaryById,
  metricEntriesByName,
  webfontsByFamily,
  plansByFamily,
) {
  const legacy = rule.relationType === 'unknown';
  const metricEntryIds = projectionId === 'rust-layout-metric'
    ? (metricEntriesByName.get(rule.targetFaceOrPolicy) ?? []).map(entry => entry.entryId)
    : [];
  if (projectionId === 'rust-layout-metric' && metricEntryIds.length === 0) {
    throw new Error(`W6 metric entry missing for ${rule.ruleId}: ${rule.targetFaceOrPolicy}`);
  }
  let supply = null;
  if (projectionId === 'canvas2d-webfont') {
    supply = webfontsByFamily.get(rule.sourceFace) ?? null;
    if (!supply) throw new Error(`Canvas2D webfont request missing for ${rule.sourceFace}`);
  } else if (projectionId === 'canvaskit-sfnt') {
    supply = canvasKitSupply(rule, plansByFamily);
  }
  return {
    ruleId: rule.ruleId,
    relationType: rule.relationType,
    decisionPlane: rule.decisionPlane,
    sourceFace: rule.sourceFace,
    targetFaceOrPolicy: rule.targetFaceOrPolicy,
    conditions: rule.conditions,
    order: rule.order,
    projections: [{ id: projectionId, mode: legacy ? 'legacy-preservation' : 'direct' }],
    metricEntryIds,
    supply,
    evidence: {
      w1RuleId: rule.ruleId,
      candidateIds: rule.candidateIds,
      sourceBoundaryIds: sourceBoundaryIds(rule, candidateBoundaryById),
      evidenceStatus: rule.evidenceStatus,
      baselineProjectionSha256,
    },
    status: 'active',
  };
}

export function buildRegistry(
  root = ROOT,
  sourceCommit = currentGitHead(root),
  fixtures = {},
) {
  const baseline = fixtures.baseline ?? readJson(BASELINE_PATH);
  const candidates = fixtures.candidates ?? readJson(W1_CANDIDATES_PATH);
  const lineage = fixtures.lineage ?? readJson(W6_LINEAGE_PATH);
  const candidateBoundaryById = new Map(candidates.ruleCandidates.map(candidate => [
    candidate.candidateId,
    candidate.sourceBoundaryId,
  ]));
  const metricEntriesByName = new Map();
  for (const entry of lineage.entries) {
    const rows = metricEntriesByName.get(entry.metricIdentity.name) ?? [];
    rows.push(entry);
    metricEntriesByName.set(entry.metricIdentity.name, rows);
  }
  for (const entries of metricEntriesByName.values()) {
    entries.sort((left, right) => left.currentIndex - right.currentIndex);
  }
  const webfontsByFamily = parseWebfontRequests(baseline.studioRuntime);
  const plansByFamily = new Map(baseline.studioRuntime.canvasKitPlans.rows.map(row => [
    row.fontName,
    row,
  ]));
  const rules = PROJECTIONS.flatMap(([baselineId, projectionId]) => {
    const projection = baseline.projections[baselineId];
    return projection.rules.map(rule => buildRule(
      rule,
      projectionId,
      projection.projectionSha256,
      candidateBoundaryById,
      metricEntriesByName,
      webfontsByFamily,
      plansByFamily,
    ));
  });
  const registry = {
    schemaVersion: '1.0',
    kind: 'canonical-font-rule-registry',
    issue: 4966,
    sourceCommit,
    schema: pathDigest(REGISTRY_SCHEMA_PATH, root),
    inputs: [
      pathDigest(BASELINE_PATH, root),
      pathDigest(W1_LEDGER_PATH, root),
      pathDigest(W6_LINEAGE_PATH, root),
    ],
    summary: {
      ruleCount: rules.length,
      activeUnknownLegacyPreservationCount: rules.filter(rule => (
        rule.projections[0].mode === 'legacy-preservation'
      )).length,
      metricEntryReferenceCount: rules.reduce((count, rule) => (
        count + rule.metricEntryIds.length
      ), 0),
      countsByProjection: countBy(rules.map(rule => rule.projections[0].id)),
      countsByRelation: countBy(rules.map(rule => rule.relationType)),
    },
    rulesSha256: sha256Text(canonicalJson(rules)),
    rules,
  };
  const errors = validateRegistry(registry, root, { candidates, lineage });
  if (errors.length > 0) throw new Error(errors.join('\n'));
  return registry;
}

function safeRepositoryPath(value) {
  return typeof value === 'string'
    && value.length > 0
    && !path.isAbsolute(value)
    && !value.split('/').includes('..')
    && !value.includes('\\');
}

function decisionKey(rule) {
  return canonicalJson({
    projectionId: rule.projections?.[0]?.id,
    sourceBoundaryIds: rule.evidence?.sourceBoundaryIds,
    sourceFace: rule.sourceFace,
    conditions: rule.conditions,
  });
}

function validateOrderGroups(rules, errors) {
  const groups = new Map();
  for (const rule of rules) {
    const rows = groups.get(decisionKey(rule)) ?? [];
    rows.push(rule);
    groups.set(decisionKey(rule), rows);
  }
  for (const rows of groups.values()) {
    if (new Set(rows.map(rule => rule.targetFaceOrPolicy)).size < 2) continue;
    const orders = rows.map(rule => rule.order);
    if (orders.some(order => !Number.isInteger(order))) {
      errors.push(`ordered decision group has a null order: ${rows[0].ruleId}`);
      continue;
    }
    const unique = [...new Set(orders)].sort((left, right) => left - right);
    if (unique.length !== orders.length || unique.some((order, index) => order !== index)) {
      errors.push(`ordered decision group must use unique contiguous orders: ${rows[0].ruleId}`);
    }
  }
}

function validatePathDigest(item, root, location, errors) {
  if (!item || !safeRepositoryPath(item.path) || !/^[0-9a-f]{64}$/u.test(item.sha256 ?? '')) {
    errors.push(`${location} must contain a safe repository path and SHA-256`);
    return;
  }
  const file = path.join(root, item.path);
  if (!fs.existsSync(file) || sha256File(file) !== item.sha256) {
    errors.push(`${location} digest does not match ${item.path}`);
  }
}

function rejectUnknownFields(value, allowed, location, errors) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    errors.push(`${location} must be an object`);
    return;
  }
  for (const key of Object.keys(value)) {
    if (!allowed.includes(key)) errors.push(`${location}.${key} is not allowed`);
  }
}

function validateBoundedString(value, location, errors, { nullable = false } = {}) {
  if (nullable && value === null) return;
  if (typeof value !== 'string' || value.length === 0 || value.length > 2048) {
    errors.push(`${location} must be a non-empty string of at most 2,048 characters`);
  }
}

function validateFontPlan(plan, location, errors) {
  rejectUnknownFields(plan, ['sources', 'unavailableFonts'], location, errors);
  if (!Array.isArray(plan?.sources) || plan.sources.length > 153
      || !Array.isArray(plan?.unavailableFonts) || plan.unavailableFonts.length > 153) {
    errors.push(`${location} exceeds the 153-font runtime bound`);
    return;
  }
  plan.sources.forEach((source, sourceIndex) => {
    const sourceLocation = `${location}.sources[${sourceIndex}]`;
    rejectUnknownFields(source, ['url', 'aliases'], sourceLocation, errors);
    validateBoundedString(source.url, `${sourceLocation}.url`, errors);
    if (!(source.url.startsWith('https://') || source.url.startsWith('fonts/'))) {
      errors.push(`${sourceLocation}.url violates the local/HTTPS boundary`);
    }
    if (!Array.isArray(source.aliases) || source.aliases.length === 0
        || source.aliases.length > 153
        || new Set(source.aliases).size !== source.aliases.length) {
      errors.push(`${sourceLocation}.aliases must be 1..153 unique font names`);
    } else {
      source.aliases.forEach((alias, aliasIndex) => (
        validateBoundedString(alias, `${sourceLocation}.aliases[${aliasIndex}]`, errors)
      ));
    }
  });
  if (new Set(plan.unavailableFonts).size !== plan.unavailableFonts.length) {
    errors.push(`${location}.unavailableFonts must be unique`);
  }
  plan.unavailableFonts.forEach((fontName, index) => (
    validateBoundedString(fontName, `${location}.unavailableFonts[${index}]`, errors)
  ));
}

function validateRuleShape(rule, index, errors) {
  const location = `registry.rules[${index}]`;
  rejectUnknownFields(rule, [
    'ruleId',
    'relationType',
    'decisionPlane',
    'sourceFace',
    'targetFaceOrPolicy',
    'conditions',
    'order',
    'projections',
    'metricEntryIds',
    'supply',
    'evidence',
    'status',
  ], location, errors);
  validateBoundedString(rule.ruleId, `${location}.ruleId`, errors);
  validateBoundedString(rule.sourceFace, `${location}.sourceFace`, errors, { nullable: true });
  validateBoundedString(rule.targetFaceOrPolicy, `${location}.targetFaceOrPolicy`, errors);
  rejectUnknownFields(
    rule.conditions,
    ['languageSlot', 'altType', 'availability', 'profile'],
    `${location}.conditions`,
    errors,
  );
  for (const [key, value] of Object.entries(rule.conditions ?? {})) {
    validateBoundedString(value, `${location}.conditions.${key}`, errors, { nullable: true });
  }
  if (!(rule.order === null || (Number.isInteger(rule.order) && rule.order >= 0))) {
    errors.push(`${location}.order must be null or a non-negative integer`);
  }
  for (const [projectionIndex, projection] of (rule.projections ?? []).entries()) {
    rejectUnknownFields(
      projection,
      ['id', 'mode'],
      `${location}.projections[${projectionIndex}]`,
      errors,
    );
  }
  rejectUnknownFields(rule.evidence, [
    'w1RuleId',
    'candidateIds',
    'sourceBoundaryIds',
    'evidenceStatus',
    'baselineProjectionSha256',
  ], `${location}.evidence`, errors);
  if ((rule.evidence?.candidateIds?.length ?? 0) > 8
      || (rule.evidence?.sourceBoundaryIds?.length ?? 0) > 8
      || (rule.metricEntryIds?.length ?? 0) > 600) {
    errors.push(`${location} exceeds an evidence or metric reference bound`);
  }
  if (rule.supply?.kind === 'canvas2d-webfont') {
    rejectUnknownFields(rule.supply, [
      'kind',
      'fontFamily',
      'sourceUrl',
      'format',
      'unicodeRange',
      'external',
    ], `${location}.supply`, errors);
  } else if (rule.supply?.kind === 'canvaskit-plan') {
    rejectUnknownFields(rule.supply, [
      'kind',
      'fontFamily',
      'declaredCapability',
      'runtimePlanStatus',
      'capabilityAgreement',
      'online',
      'offline',
    ], `${location}.supply`, errors);
    validateFontPlan(rule.supply.online, `${location}.supply.online`, errors);
    validateFontPlan(rule.supply.offline, `${location}.supply.offline`, errors);
  } else if (rule.supply !== null) {
    errors.push(`${location}.supply kind is invalid`);
  }
}

export function validateRegistry(registry, root = ROOT, fixtures = {}) {
  const errors = [];
  if (registry?.kind !== 'canonical-font-rule-registry'
      || registry.schemaVersion !== '1.0'
      || registry.issue !== 4966) {
    return ['registry envelope is invalid'];
  }
  if (!/^[0-9a-f]{40}$/u.test(registry.sourceCommit ?? '')) {
    errors.push('registry sourceCommit must be a lowercase 40-character Git SHA');
  }
  rejectUnknownFields(registry, [
    'schemaVersion',
    'kind',
    'issue',
    'sourceCommit',
    'schema',
    'inputs',
    'summary',
    'rulesSha256',
    'rules',
  ], 'registry', errors);
  rejectUnknownFields(registry.summary, [
    'ruleCount',
    'activeUnknownLegacyPreservationCount',
    'metricEntryReferenceCount',
    'countsByProjection',
    'countsByRelation',
  ], 'registry.summary', errors);
  const serializedRegistry = canonicalJson(registry);
  if (/\/(?:home|Users)\//u.test(serializedRegistry)
      || /(?:^|[\s"'])[A-Za-z]:[\\/]/mu.test(serializedRegistry)
      || serializedRegistry.includes('file://')) {
    errors.push('registry must not contain host-absolute or file URL paths');
  }
  validatePathDigest(registry.schema, root, 'registry.schema', errors);
  if (!Array.isArray(registry.inputs) || registry.inputs.length !== 3) {
    errors.push('registry must preserve exactly three input digests');
  } else {
    registry.inputs.forEach((input, index) => (
      validatePathDigest(input, root, `registry.inputs[${index}]`, errors)
    ));
    if (new Set(registry.inputs.map(input => input.path)).size !== registry.inputs.length) {
      errors.push('registry input paths must be unique');
    }
  }

  const rules = registry.rules ?? [];
  rules.forEach((rule, index) => validateRuleShape(rule, index, errors));
  if (rules.length !== 830 || registry.summary?.ruleCount !== 830) {
    errors.push('registry rule population must remain 830');
  }
  if (new Set(rules.map(rule => rule.ruleId)).size !== rules.length) {
    errors.push('registry ruleId values must be unique');
  }
  if (sha256Text(canonicalJson(rules)) !== registry.rulesSha256) {
    errors.push('registry rulesSha256 mismatch');
  }
  const projectionCounts = countBy(rules.map(rule => rule.projections?.[0]?.id));
  if (canonicalJson(projectionCounts) !== canonicalJson(PROJECTION_COUNTS)
      || canonicalJson(projectionCounts) !== canonicalJson(registry.summary?.countsByProjection)) {
    errors.push('registry projection populations changed');
  }
  if (canonicalJson(countBy(rules.map(rule => rule.relationType)))
      !== canonicalJson(registry.summary?.countsByRelation)) {
    errors.push('registry relation summary mismatch');
  }

  const candidates = fixtures.candidates ?? readJson(path.join(root, relativePath(W1_CANDIDATES_PATH)));
  const ledger = fixtures.ledger ?? readJson(path.join(root, relativePath(W1_LEDGER_PATH)));
  const lineage = fixtures.lineage ?? readJson(path.join(root, relativePath(W6_LINEAGE_PATH)));
  const baseline = fixtures.baseline ?? readJson(path.join(root, relativePath(BASELINE_PATH)));
  const candidateIds = new Set(candidates.ruleCandidates.map(candidate => candidate.candidateId));
  const candidateBoundaryById = new Map(candidates.ruleCandidates.map(candidate => [
    candidate.candidateId,
    candidate.sourceBoundaryId,
  ]));
  const w1RuleIds = new Set(ledger.rules.map(rule => rule.ruleId));
  const metricEntryIds = new Set(lineage.entries.map(entry => entry.entryId));
  const metricEntryById = new Map(lineage.entries.map(entry => [entry.entryId, entry]));
  const baselineHashByProjection = new Map(PROJECTIONS.map(([baselineId, projectionId]) => [
    projectionId,
    baseline.projections[baselineId].projectionSha256,
  ]));
  for (const rule of rules) {
    if (rule.status !== 'active' || rule.projections?.length !== 1) {
      errors.push(`${rule.ruleId}: every rule must be active with exactly one projection`);
      continue;
    }
    const projection = rule.projections[0];
    const allowed = ALLOWLIST[projection.id];
    if (!allowed
        || !allowed.planes.has(rule.decisionPlane)
        || !allowed.relations.has(rule.relationType)) {
      errors.push(`${rule.ruleId}: ${projection.id} rejects ${rule.decisionPlane}/${rule.relationType}`);
    }
    if (rule.relationType === 'unknown') {
      if (projection.id !== 'rust-layout-metric'
          || projection.mode !== 'legacy-preservation'
          || rule.evidence?.evidenceStatus !== 'unknown') {
        errors.push(`${rule.ruleId}: active unknown alias must remain layout-metric legacy-preservation`);
      }
    } else if (projection.mode !== 'direct') {
      errors.push(`${rule.ruleId}: only active unknown aliases may use legacy-preservation`);
    }
    if (!/^[a-z0-9]+(?:[.-][a-z0-9]+)*$/u.test(rule.ruleId ?? '')
        || !w1RuleIds.has(rule.ruleId)) {
      errors.push(`${rule.ruleId}: W1 rule anchor is missing or invalid`);
    }
    if (!Array.isArray(rule.evidence?.candidateIds)
        || rule.evidence.candidateIds.length === 0
        || new Set(rule.evidence.candidateIds).size !== rule.evidence.candidateIds.length
        || rule.evidence.candidateIds.some(candidateId => !candidateIds.has(candidateId))) {
      errors.push(`${rule.ruleId}: W1 candidate anchor is missing or invalid`);
    }
    const expectedBoundaryIds = [...new Set((rule.evidence?.candidateIds ?? [])
      .map(candidateId => candidateBoundaryById.get(candidateId)).filter(Boolean))];
    if (rule.evidence?.w1RuleId !== rule.ruleId
        || !Array.isArray(rule.evidence?.sourceBoundaryIds)
        || rule.evidence.sourceBoundaryIds.length === 0
        || canonicalJson(rule.evidence.sourceBoundaryIds) !== canonicalJson(expectedBoundaryIds)) {
      errors.push(`${rule.ruleId}: W1 rule/boundary evidence is incomplete`);
    }
    if (!['verified-by-bytes', 'verified-by-test', 'historical', 'unknown']
      .includes(rule.evidence?.evidenceStatus)
      || rule.evidence?.baselineProjectionSha256 !== baselineHashByProjection.get(projection.id)) {
      errors.push(`${rule.ruleId}: evidence status or pre-migration projection hash is invalid`);
    }
    const metricIds = rule.metricEntryIds ?? [];
    if (projection.id === 'rust-layout-metric') {
      const expectedMetricIds = lineage.entries
        .filter(entry => entry.metricIdentity.name === rule.targetFaceOrPolicy)
        .sort((left, right) => left.currentIndex - right.currentIndex)
        .map(entry => entry.entryId);
      if (metricIds.length === 0
          || new Set(metricIds).size !== metricIds.length
          || metricIds.some(entryId => !metricEntryIds.has(entryId))
          || metricIds.some(entryId => (
            metricEntryById.get(entryId)?.metricIdentity.name !== rule.targetFaceOrPolicy
          ))
          || canonicalJson(metricIds) !== canonicalJson(expectedMetricIds)) {
        errors.push(`${rule.ruleId}: W6 metric anchors are missing or invalid`);
      }
    } else if (metricIds.length !== 0) {
      errors.push(`${rule.ruleId}: only Rust metric projections may reference W6 entries`);
    }
    if (projection.id === 'canvas2d-webfont') {
      if (rule.supply?.kind !== 'canvas2d-webfont') {
        errors.push(`${rule.ruleId}: Canvas2D supply payload is missing`);
      } else {
        if (rule.supply.fontFamily !== rule.sourceFace
            || !['woff2', 'woff', 'truetype', 'opentype'].includes(rule.supply.format)
            || typeof rule.supply.external !== 'boolean'
            || (rule.supply.external
              ? !rule.supply.sourceUrl.startsWith('https://')
              : !rule.supply.sourceUrl.startsWith('fonts/'))) {
          errors.push(`${rule.ruleId}: Canvas2D supply payload violates its family/format/URL boundary`);
        }
        validateBoundedString(
          rule.supply.unicodeRange,
          `${rule.ruleId}.supply.unicodeRange`,
          errors,
          { nullable: true },
        );
      }
    } else if (projection.id === 'canvaskit-sfnt'
      && rule.conditions?.profile === 'canvaskit-sfnt') {
      if (rule.supply?.kind !== 'canvaskit-plan') {
        errors.push(`${rule.ruleId}: CanvasKit finite supply payload is missing`);
      } else {
        if (rule.supply.fontFamily !== rule.sourceFace
            || !['sfnt-source', 'unavailable'].includes(rule.supply.declaredCapability)
            || !['planned', 'unavailable'].includes(rule.supply.runtimePlanStatus)
            || typeof rule.supply.capabilityAgreement !== 'boolean') {
          errors.push(`${rule.ruleId}: CanvasKit supply classification is invalid`);
        }
        const agreement = (rule.supply.declaredCapability === 'sfnt-source')
          === (rule.supply.runtimePlanStatus === 'planned');
        if (agreement !== rule.supply.capabilityAgreement) {
          errors.push(`${rule.ruleId}: CanvasKit capability agreement is inconsistent`);
        }
      }
    } else if (rule.supply !== null) {
      errors.push(`${rule.ruleId}: non-finite supply projection must not carry a supply payload`);
    }
  }
  const legacy = rules.filter(rule => rule.projections?.[0]?.mode === 'legacy-preservation');
  if (legacy.length !== 43
      || registry.summary?.activeUnknownLegacyPreservationCount !== 43
      || legacy.some(rule => rule.relationType !== 'unknown')) {
    errors.push('registry must preserve exactly 43 active unknown metric aliases');
  }
  const metricReferenceCount = rules.reduce((count, rule) => (
    count + (rule.metricEntryIds?.length ?? 0)
  ), 0);
  if (metricReferenceCount !== registry.summary?.metricEntryReferenceCount) {
    errors.push('registry metric entry reference summary mismatch');
  }
  validateOrderGroups(rules, errors);
  return errors;
}

export function buildMigration(registry, root = ROOT, fixtures = {}) {
  const baseline = fixtures.baseline ?? readJson(BASELINE_PATH);
  const baselineRules = new Map(PROJECTIONS.flatMap(([baselineId, projectionId]) => (
    baseline.projections[baselineId].rules.map(rule => [`${projectionId}\u0000${rule.ruleId}`, rule])
  )));
  const mappings = registry.rules.map(rule => {
    const projection = rule.projections[0];
    const baselineRule = baselineRules.get(`${projection.id}\u0000${rule.ruleId}`);
    if (!baselineRule) throw new Error(`pre-migration rule missing: ${rule.ruleId}`);
    return {
      ruleId: rule.ruleId,
      projectionId: projection.id,
      mode: projection.mode,
      candidateIds: rule.evidence.candidateIds,
      metricEntryIds: rule.metricEntryIds,
      baselineRuleSha256: sha256Text(canonicalJson(baselineRule)),
      registryRuleSha256: sha256Text(canonicalJson(rule)),
      disposition: projection.mode === 'legacy-preservation'
        ? 'migrated-active-unknown'
        : 'migrated-active',
    };
  });
  const canvasKitSupplies = registry.rules
    .map(rule => rule.supply)
    .filter(supply => supply?.kind === 'canvaskit-plan');
  const migration = {
    schemaVersion: '1.0',
    kind: 'font-rule-registry-migration',
    issue: 4966,
    sourceCommit: registry.sourceCommit,
    schema: pathDigest(MIGRATION_SCHEMA_PATH, root),
    baseline: pathDigest(BASELINE_PATH, root),
    registry: {
      path: relativePath(REGISTRY_PATH, root),
      sha256: sha256Text(canonicalJson(registry)),
    },
    summary: {
      mappingCount: mappings.length,
      directCount: mappings.filter(mapping => mapping.mode === 'direct').length,
      legacyPreservationCount: mappings.filter(mapping => (
        mapping.mode === 'legacy-preservation'
      )).length,
      w1CandidateLinkCount: mappings.reduce((count, mapping) => (
        count + mapping.candidateIds.length
      ), 0),
      w1UniqueCandidateCount: new Set(mappings.flatMap(mapping => mapping.candidateIds)).size,
      w6MetricEntryReferenceCount: mappings.reduce((count, mapping) => (
        count + mapping.metricEntryIds.length
      ), 0),
      canvasKitDeclaredUnavailableRuntimePlannedCount: canvasKitSupplies.filter(supply => (
        supply.declaredCapability === 'unavailable' && supply.runtimePlanStatus === 'planned'
      )).length,
      canvasKitDeclaredAvailableRuntimeUnavailableCount: canvasKitSupplies.filter(supply => (
        supply.declaredCapability === 'sfnt-source' && supply.runtimePlanStatus === 'unavailable'
      )).length,
    },
    mappingsSha256: sha256Text(canonicalJson(mappings)),
    mappings,
  };
  const errors = validateMigration(migration, registry, baseline, root);
  if (errors.length > 0) throw new Error(errors.join('\n'));
  return migration;
}

export function validateMigration(migration, registry, baseline, root = ROOT) {
  const errors = [];
  if (migration?.kind !== 'font-rule-registry-migration'
      || migration.schemaVersion !== '1.0'
      || migration.issue !== 4966
      || migration.sourceCommit !== registry.sourceCommit) {
    return ['migration envelope is invalid'];
  }
  rejectUnknownFields(migration, [
    'schemaVersion',
    'kind',
    'issue',
    'sourceCommit',
    'schema',
    'baseline',
    'registry',
    'summary',
    'mappingsSha256',
    'mappings',
  ], 'migration', errors);
  rejectUnknownFields(migration.summary, [
    'mappingCount',
    'directCount',
    'legacyPreservationCount',
    'w1CandidateLinkCount',
    'w1UniqueCandidateCount',
    'w6MetricEntryReferenceCount',
    'canvasKitDeclaredUnavailableRuntimePlannedCount',
    'canvasKitDeclaredAvailableRuntimeUnavailableCount',
  ], 'migration.summary', errors);
  validatePathDigest(migration.schema, root, 'migration.schema', errors);
  validatePathDigest(migration.baseline, root, 'migration.baseline', errors);
  if (migration.registry?.path !== relativePath(REGISTRY_PATH, root)
      || migration.registry?.sha256 !== sha256Text(canonicalJson(registry))) {
    errors.push('migration registry digest mismatch');
  }
  const mappings = migration.mappings ?? [];
  if (mappings.length !== 830 || migration.summary?.mappingCount !== 830) {
    errors.push('migration mapping population must remain 830');
  }
  if (sha256Text(canonicalJson(mappings)) !== migration.mappingsSha256) {
    errors.push('migration mappingsSha256 mismatch');
  }
  const registryRules = new Map(registry.rules.map(rule => [rule.ruleId, rule]));
  for (const [index, mapping] of mappings.entries()) {
    rejectUnknownFields(mapping, [
      'ruleId',
      'projectionId',
      'mode',
      'candidateIds',
      'metricEntryIds',
      'baselineRuleSha256',
      'registryRuleSha256',
      'disposition',
    ], `migration.mappings[${index}]`, errors);
    if ((mapping.candidateIds?.length ?? 0) > 8
        || (mapping.metricEntryIds?.length ?? 0) > 600) {
      errors.push(`${mapping.ruleId}: migration mapping exceeds a reference bound`);
    }
    const rule = registryRules.get(mapping.ruleId);
    if (!rule
        || rule.projections[0].id !== mapping.projectionId
        || rule.projections[0].mode !== mapping.mode
        || sha256Text(canonicalJson(rule)) !== mapping.registryRuleSha256) {
      errors.push(`${mapping.ruleId}: migration does not map exactly to the registry rule`);
    }
  }
  const directCount = mappings.filter(mapping => mapping.mode === 'direct').length;
  const legacyCount = mappings.filter(mapping => mapping.mode === 'legacy-preservation').length;
  if (directCount !== 787 || migration.summary?.directCount !== 787
      || legacyCount !== 43 || migration.summary?.legacyPreservationCount !== 43) {
    errors.push('migration must preserve 787 direct and 43 legacy mappings');
  }
  const candidateLinks = mappings.flatMap(mapping => mapping.candidateIds ?? []);
  if (candidateLinks.length !== 830
      || migration.summary?.w1CandidateLinkCount !== 830
      || new Set(candidateLinks).size !== 677
      || migration.summary?.w1UniqueCandidateCount !== 677) {
    errors.push('migration must preserve 830 W1 candidate links over 677 unique candidates');
  }
  const canvasKitSupplies = registry.rules.map(rule => rule.supply)
    .filter(supply => supply?.kind === 'canvaskit-plan');
  const unavailablePlanned = canvasKitSupplies.filter(supply => (
    supply.declaredCapability === 'unavailable' && supply.runtimePlanStatus === 'planned'
  )).length;
  const availableUnavailable = canvasKitSupplies.filter(supply => (
    supply.declaredCapability === 'sfnt-source' && supply.runtimePlanStatus === 'unavailable'
  )).length;
  if (unavailablePlanned !== 125
      || migration.summary?.canvasKitDeclaredUnavailableRuntimePlannedCount !== 125
      || availableUnavailable !== 0
      || migration.summary?.canvasKitDeclaredAvailableRuntimeUnavailableCount !== 0) {
    errors.push('migration must preserve the 125/0 CanvasKit capability-plan mismatches');
  }
  if (migration.baseline?.sha256 !== sha256File(BASELINE_PATH)
      || baseline.kind !== 'font-rule-projection-pre-migration-baseline') {
    errors.push('migration baseline authority is invalid');
  }
  return errors;
}

export function compareRegistry(expected, actual) {
  return canonicalJson(expected) === canonicalJson(actual)
    ? []
    : ['canonical font rule registry differs from current W1/W6/W7 inputs'];
}

export function compareMigration(expected, actual) {
  return canonicalJson(expected) === canonicalJson(actual)
    ? []
    : ['font rule registry migration differs from the canonical registry'];
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : '';
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    const command = process.argv[2];
    if (process.argv.length > 3) {
      throw new Error('font rule registry uses fixed checkout-root output paths');
    }
    if (command === 'generate') {
      const registry = buildRegistry(ROOT);
      const migration = buildMigration(registry, ROOT);
      fs.mkdirSync(path.dirname(REGISTRY_PATH), { recursive: true });
      fs.mkdirSync(path.dirname(MIGRATION_PATH), { recursive: true });
      fs.writeFileSync(REGISTRY_PATH, canonicalJson(registry), 'utf8');
      fs.writeFileSync(MIGRATION_PATH, canonicalJson(migration), 'utf8');
      process.stdout.write(
        `font rule registry: ${registry.rules.length} rules, ${migration.mappings.length} mappings\n`,
      );
    } else if (command === 'check') {
      const expectedRegistry = readJson(REGISTRY_PATH);
      const expectedMigration = readJson(MIGRATION_PATH);
      const baseline = readJson(BASELINE_PATH);
      const actualRegistry = buildRegistry(ROOT, expectedRegistry.sourceCommit);
      const actualMigration = buildMigration(actualRegistry, ROOT, { baseline });
      const errors = [
        ...validateRegistry(expectedRegistry, ROOT),
        ...validateMigration(expectedMigration, expectedRegistry, baseline, ROOT),
        ...compareRegistry(expectedRegistry, actualRegistry),
        ...compareMigration(expectedMigration, actualMigration),
      ];
      if (errors.length > 0) throw new Error(errors.join('\n'));
      process.stdout.write('font rule registry: ok\n');
    } else {
      throw new Error('usage: font_rule_registry.mjs <generate|check>');
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
