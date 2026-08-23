#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import {
  canonicalJson,
  collectSourceCandidates,
  sha256Text,
} from './font_rule_ledger.mjs';
import { collectRuleCandidates } from './font_rule_candidates.mjs';

const REPOSITORY_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const INVESTIGATION_ROOT = path.join(
  REPOSITORY_ROOT,
  'mydocs',
  'tech',
  'investigations',
);
const SOURCES_PATH = path.join(INVESTIGATION_ROOT, 'issue-4939', 'font_rule_sources.json');
const W1_CANDIDATES_PATH = path.join(INVESTIGATION_ROOT, 'issue-4939', 'font_rule_candidates.json');
const W1_LEDGER_PATH = path.join(INVESTIGATION_ROOT, 'issue-4939', 'font_rule_ledger.json');
const W6_LINEAGE_PATH = path.join(
  INVESTIGATION_ROOT,
  'issue-4964',
  'font_metric_lineage_manifest.json',
);
const RUNTIME_SNAPSHOT_PATH = path.join(
  REPOSITORY_ROOT,
  'scripts',
  'font_rule_runtime_snapshot.mjs',
);
const DEFAULT_OUTPUT_PATH = path.join(
  INVESTIGATION_ROOT,
  'issue-4966',
  'font_rule_projection_baseline.json',
);

const PROJECTION_NAMES = [
  'rustLayoutName',
  'rustLayoutMetric',
  'canvas2dPaint',
  'webfontSupply',
  'canvasKitSfnt',
];

const TARGET_BOUNDARIES = new Map([
  ['rust-style-resolution.legacy-latin', ['rustLayoutName']],
  ['rust-style-resolution.hft', ['rustLayoutName']],
  ['rust-style-resolution.ttf', ['rustLayoutName']],
  ['rust-metric.metric-alias', ['rustLayoutMetric']],
  ['rust-paint-chain.installed-aliases', ['canvas2dPaint']],
  ['studio-substitution.substitution-tables', ['canvas2dPaint']],
  ['studio-substitution.display-chain', ['canvas2dPaint']],
  ['studio-supply.font-list', ['webfontSupply', 'canvasKitSfnt']],
  ['studio-supply.canvaskit-plan', ['canvasKitSfnt']],
  ['studio-detection.sfnt-bytes', ['canvasKitSfnt']],
  ['studio-canvas-patch.css-family-substitution', ['canvas2dPaint']],
]);

const REFERENCE_REASONS = new Map([
  ['rust-style-resolution.heavy-display', 'paint weight predicate remains hand-written'],
  ['rust-metric.metric-table', 'W6 owns the 600 metric values and stable entry IDs'],
  ['rust-metric.metric-lookup', 'first-match lookup ladder remains hand-written'],
  ['rust-measurement.estimate-width', 'measurement algorithm remains hand-written'],
  ['rust-measurement.hancom-space', 'measurement overlay predicate remains hand-written'],
  ['rust-paint-chain.weight-suffix', 'paint normalization predicate is outside the four projections'],
  ['rust-paint-chain.generic-fallback', 'generic paint classification remains hand-written'],
  ['native-skia.system-family-style', 'native capability lookup is not a finite W7 projection'],
  ['native-skia.text-replay', 'native replay policy remains hand-written'],
  ['paint-resource.resource-table', 'resource identity contract is reference-only'],
  ['paint-resource.fallback-policy', 'resource fallback identifier is reference-only'],
  ['studio-detection.detection-method', 'browser capability detection remains hand-written'],
  ['studio-detection.presence-probe', 'browser presence probe remains hand-written'],
  ['studio-canvas-patch.canvas-install', 'Canvas2D capability installation remains hand-written'],
  ['asset-authority.asset-index', 'font binary and license authority remains separate'],
  ['asset-authority.metric-source-index', 'metric source authority remains separate'],
  ['asset-authority.license-index', 'license authority remains separate'],
  ['tests-history.studio-substitution-test', 'historical evidence anchor is not runtime input'],
  ['tests-history.government-font-matrix', 'Oracle evidence anchor is not runtime input'],
]);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function sha256File(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function currentGitHead(repositoryRoot) {
  return execFileSync('git', ['rev-parse', 'HEAD'], {
    cwd: repositoryRoot,
    encoding: 'utf8',
  }).trim();
}

function runtimeSnapshot(repositoryRoot) {
  return JSON.parse(execFileSync(process.execPath, [RUNTIME_SNAPSHOT_PATH], {
    cwd: repositoryRoot,
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  }));
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

function candidateReferenceIds(rule) {
  const prefix = 'mydocs/tech/investigations/issue-4939/font_rule_candidates.json#';
  return rule.evidence
    .map(entry => entry.reference)
    .filter(reference => reference.startsWith(prefix))
    .map(reference => reference.slice(prefix.length));
}

function stableCandidate(candidate) {
  return {
    candidateId: candidate.candidateId,
    sourceBoundaryId: candidate.sourceBoundaryId,
    candidateKind: candidate.candidateKind,
    decisionPlane: candidate.decisionPlane,
    sourceFace: candidate.sourceFace,
    targetOrPolicy: candidate.targetOrPolicy,
    conditions: candidate.conditions,
    backends: candidate.backends,
    order: candidate.order,
  };
}

function projectionRule(rule, candidateIds) {
  return {
    ruleId: rule.ruleId,
    candidateIds,
    relationType: rule.relationType,
    decisionPlane: rule.decisionPlane,
    sourceFace: rule.sourceFace,
    targetFaceOrPolicy: rule.targetFaceOrPolicy,
    conditions: rule.conditions,
    backends: rule.backends,
    order: rule.order,
    evidenceStatus: rule.evidenceStatus,
    status: rule.status,
  };
}

function projectionDocument(description, rules) {
  const rows = rules.map(({ rule, candidateIds }) => projectionRule(rule, candidateIds));
  return {
    description,
    ruleCount: rows.length,
    countsByRelation: countBy(rows.map(row => row.relationType)),
    countsByEvidenceStatus: countBy(rows.map(row => row.evidenceStatus)),
    projectionSha256: sha256Text(canonicalJson(rows)),
    rules: rows,
  };
}

function candidateSequenceDiff(current, approved) {
  const currentIds = current.map(candidate => candidate.candidateId);
  const approvedIds = approved.map(candidate => candidate.candidateId);
  const currentSet = new Set(currentIds);
  const approvedSet = new Set(approvedIds);
  return {
    added: currentIds.filter(candidateId => !approvedSet.has(candidateId)),
    removed: approvedIds.filter(candidateId => !currentSet.has(candidateId)),
    sequenceEqual: canonicalJson(currentIds) === canonicalJson(approvedIds),
    identityEqual: canonicalJson(current.map(stableCandidate))
      === canonicalJson(approved.map(stableCandidate)),
  };
}

function routeInventory(snapshot) {
  return snapshot.dispositions.map(disposition => {
    const targets = TARGET_BOUNDARIES.get(disposition.sourceBoundaryId) ?? [];
    const referenceReason = REFERENCE_REASONS.get(disposition.sourceBoundaryId) ?? null;
    return {
      sourceBoundaryId: disposition.sourceBoundaryId,
      candidateCount: disposition.candidateCount,
      classification: targets.length > 0 ? 'projection-input' : 'reference-only',
      projections: targets,
      reason: targets.length > 0
        ? 'finite rows or an ordered runtime contract feed only the listed W7 projection'
        : referenceReason,
    };
  });
}

function linkedRules(ledger, candidateBoundaryById, predicate) {
  const rows = [];
  for (const rule of ledger.rules) {
    const candidateIds = candidateReferenceIds(rule);
    const boundaryIds = [...new Set(
      candidateIds.map(candidateId => candidateBoundaryById.get(candidateId)).filter(Boolean),
    )];
    if (predicate(rule, boundaryIds)) rows.push({ rule, candidateIds });
  }
  return rows;
}

function inputDigests(repositoryRoot, currentSnapshot) {
  const paths = new Set([
    path.relative(repositoryRoot, SOURCES_PATH),
    path.relative(repositoryRoot, W1_CANDIDATES_PATH),
    path.relative(repositoryRoot, W1_LEDGER_PATH),
    path.relative(repositoryRoot, W6_LINEAGE_PATH),
    path.relative(repositoryRoot, RUNTIME_SNAPSHOT_PATH),
    ...currentSnapshot.candidates.map(candidate => candidate.path),
  ]);
  return [...paths]
    .sort(compareText)
    .map(relativePath => ({
      path: relativePath,
      sha256: sha256File(path.join(repositoryRoot, relativePath)),
    }));
}

export function buildProjectionBaseline(
  repositoryRoot = REPOSITORY_ROOT,
  sourceCommit = currentGitHead(repositoryRoot),
) {
  const sources = readJson(SOURCES_PATH);
  const approvedCandidates = readJson(W1_CANDIDATES_PATH);
  const ledger = readJson(W1_LEDGER_PATH);
  const lineage = readJson(W6_LINEAGE_PATH);
  const sourceSnapshot = collectSourceCandidates(sources, repositoryRoot, sourceCommit);
  const currentSnapshot = collectRuleCandidates(sourceSnapshot, repositoryRoot);
  const sequenceDiff = candidateSequenceDiff(
    currentSnapshot.ruleCandidates,
    approvedCandidates.ruleCandidates,
  );
  if (sequenceDiff.added.length > 0
      || sequenceDiff.removed.length > 0
      || !sequenceDiff.sequenceEqual
      || !sequenceDiff.identityEqual) {
    throw new Error(`current font rule candidates drift from W1: ${JSON.stringify(sequenceDiff)}`);
  }

  const currentCandidateIds = new Set(
    currentSnapshot.ruleCandidates.map(candidate => candidate.candidateId),
  );
  const candidateBoundaryById = new Map(
    currentSnapshot.ruleCandidates.map(candidate => [
      candidate.candidateId,
      candidate.sourceBoundaryId,
    ]),
  );
  const linkedCandidateIds = new Set(
    ledger.rules.flatMap(rule => candidateReferenceIds(rule)),
  );
  const unlinkedCandidateIds = [...currentCandidateIds]
    .filter(candidateId => !linkedCandidateIds.has(candidateId));
  if (unlinkedCandidateIds.length > 0) {
    throw new Error(`current candidates without W1 ledger rules: ${unlinkedCandidateIds.join(', ')}`);
  }

  const routes = routeInventory(currentSnapshot);
  const classifiedBoundaryIds = new Set([
    ...TARGET_BOUNDARIES.keys(),
    ...REFERENCE_REASONS.keys(),
  ]);
  const unclassifiedBoundaryIds = routes
    .map(route => route.sourceBoundaryId)
    .filter(boundaryId => !classifiedBoundaryIds.has(boundaryId));
  if (unclassifiedBoundaryIds.length > 0) {
    throw new Error(`unclassified W1 source boundaries: ${unclassifiedBoundaryIds.join(', ')}`);
  }

  const boundary = value => (rule, boundaryIds) => boundaryIds.includes(value)
    && rule.status === 'active';
  const boundarySet = values => (rule, boundaryIds) => boundaryIds.some(value => values.has(value))
    && rule.status === 'active';

  const rustLayoutName = linkedRules(
    ledger,
    candidateBoundaryById,
    boundarySet(new Set([
      'rust-style-resolution.legacy-latin',
      'rust-style-resolution.hft',
      'rust-style-resolution.ttf',
    ])),
  );
  const rustLayoutMetric = linkedRules(
    ledger,
    candidateBoundaryById,
    boundary('rust-metric.metric-alias'),
  );
  const canvas2dPaint = linkedRules(
    ledger,
    candidateBoundaryById,
    (rule, boundaryIds) => rule.status === 'active' && (
      boundaryIds.includes('studio-substitution.substitution-tables')
      || boundaryIds.includes('studio-substitution.display-chain')
      || boundaryIds.includes('studio-canvas-patch.css-family-substitution')
      || (boundaryIds.includes('rust-paint-chain.installed-aliases')
        && rule.relationType === 'official-successor')
    ),
  );
  const webfontSupply = linkedRules(
    ledger,
    candidateBoundaryById,
    (rule, boundaryIds) => rule.status === 'active'
      && boundaryIds.includes('studio-supply.font-list')
      && rule.relationType === 'supply-source'
      && rule.backends.includes('canvas2d'),
  );
  const canvasKitSfnt = linkedRules(
    ledger,
    candidateBoundaryById,
    (rule, boundaryIds) => rule.status === 'active' && (
      (boundaryIds.includes('studio-supply.font-list')
        && rule.relationType === 'supply-source'
        && rule.backends.includes('canvaskit'))
      || boundaryIds.includes('studio-supply.canvaskit-plan')
      || boundaryIds.includes('studio-detection.sfnt-bytes')
    ),
  );

  const projections = {
    rustLayoutName: projectionDocument(
      'Rust language and alt-type finite style-name mappings',
      rustLayoutName,
    ),
    rustLayoutMetric: projectionDocument(
      'Rust metric aliases, including active unknown legacy-preservation mappings',
      rustLayoutMetric,
    ),
    canvas2dPaint: projectionDocument(
      'Canvas2D substitution, display-chain and official-successor inputs',
      canvas2dPaint,
    ),
    webfontSupply: projectionDocument(
      'Canvas2D CSS webfont supply entries',
      webfontSupply,
    ),
    canvasKitSfnt: projectionDocument(
      'CanvasKit SFNT supply, substitution plan and byte capability contract',
      canvasKitSfnt,
    ),
  };

  const metricAnchors = lineage.entries.map(entry => ({
    entryId: entry.entryId,
    currentIndex: entry.currentIndex,
    name: entry.metricIdentity.name,
    bold: entry.metricIdentity.bold,
    italic: entry.metricIdentity.italic,
    metricDataSha256: entry.semanticHashes.metricDataSha256,
    widthProjectionSha256: entry.semanticHashes.widthProjectionSha256,
    ruleIds: entry.relations.map(relation => relation.relationId),
  }));
  const activeUnknownRules = ledger.rules.filter(rule => (
    rule.status === 'active' && rule.relationType === 'unknown'
  ));
  const projectedRuleIds = new Set(
    Object.values(projections).flatMap(projection => (
      projection.rules.map(rule => rule.ruleId)
    )),
  );
  const unknownInventory = activeUnknownRules.map(rule => ({
    ruleId: rule.ruleId,
    sourceOwner: rule.sourceOwner,
    decisionPlane: rule.decisionPlane,
    disposition: projectedRuleIds.has(rule.ruleId)
      ? 'legacy-preservation-projection'
      : 'hand-written-runtime-reference',
  }));

  const projectionHashes = Object.fromEntries(
    PROJECTION_NAMES.map(name => [name, projections[name].projectionSha256]),
  );
  const studioRuntime = runtimeSnapshot(repositoryRoot);
  const baseline = {
    schemaVersion: '1.0',
    kind: 'font-rule-projection-pre-migration-baseline',
    issue: 4966,
    sourceCommit,
    inputs: inputDigests(repositoryRoot, currentSnapshot),
    inventory: {
      sourceBoundaryCount: currentSnapshot.candidates.length,
      ruleCandidateCount: currentSnapshot.ruleCandidates.length,
      w1RuleCount: ledger.rules.length,
      currentCandidateProjectionSha256: currentSnapshot.summary.projectionSha256,
      w1LedgerSha256: sha256File(W1_LEDGER_PATH),
      w1CandidateSequenceSha256: sha256Text(canonicalJson(
        currentSnapshot.ruleCandidates.map(stableCandidate),
      )),
      currentMatchesW1: true,
      unlinkedCandidateIds: [],
      routes,
      activeUnknown: {
        count: unknownInventory.length,
        projectedLegacyPreservationCount: unknownInventory.filter(entry => (
          entry.disposition === 'legacy-preservation-projection'
        )).length,
        handWrittenReferenceCount: unknownInventory.filter(entry => (
          entry.disposition === 'hand-written-runtime-reference'
        )).length,
        rules: unknownInventory,
      },
    },
    metricAnchors: {
      entryCount: metricAnchors.length,
      compositionSha256: lineage.baselineHashes.compositionSha256,
      metricDataSha256: lineage.baselineHashes.metricDataSha256,
      widthProjectionSha256: lineage.baselineHashes.widthProjectionSha256,
      lookupProjectionSha256: lineage.baselineHashes.lookupProjectionSha256,
      anchorsSha256: sha256Text(canonicalJson(metricAnchors)),
      entries: metricAnchors,
    },
    studioRuntime,
    projections,
    hashes: {
      registryInputSha256: sha256Text(canonicalJson({
        candidateSequenceSha256: currentSnapshot.summary.projectionSha256,
        ledgerSha256: sha256File(W1_LEDGER_PATH),
        metricAnchorsSha256: sha256Text(canonicalJson(metricAnchors)),
      })),
      ...projectionHashes,
      projectionBundleSha256: sha256Text(canonicalJson(projectionHashes)),
    },
  };
  const errors = validateProjectionBaseline(baseline);
  if (errors.length > 0) throw new Error(errors.join('\n'));
  return baseline;
}

export function validateProjectionBaseline(baseline) {
  const errors = [];
  if (baseline?.kind !== 'font-rule-projection-pre-migration-baseline') {
    return ['baseline kind is invalid'];
  }
  if (baseline.inventory?.sourceBoundaryCount !== 30) {
    errors.push('source boundary population must remain 30');
  }
  if (baseline.inventory?.ruleCandidateCount !== 1352) {
    errors.push('rule candidate population must remain 1352');
  }
  if (baseline.inventory?.w1RuleCount !== 1507) {
    errors.push('W1 ledger population must remain 1507');
  }
  for (const input of baseline.inputs ?? []) {
    if (typeof input.path !== 'string' || !/^[0-9a-f]{64}$/.test(input.sha256 ?? '')) {
      errors.push('every input must preserve a repository path and SHA-256');
    }
  }
  if (new Set((baseline.inputs ?? []).map(input => input.path)).size !== baseline.inputs?.length) {
    errors.push('input paths must be unique');
  }
  const routes = baseline.inventory?.routes ?? [];
  if (routes.length !== baseline.inventory?.sourceBoundaryCount) {
    errors.push('every source boundary must have exactly one route disposition');
  }
  if (new Set(routes.map(route => route.sourceBoundaryId)).size !== routes.length) {
    errors.push('source boundary route dispositions must be unique');
  }
  if (routes.some(route => !route.reason || !Array.isArray(route.projections))) {
    errors.push('every route must preserve a reason and explicit projections');
  }
  if (baseline.metricAnchors?.entryCount !== 600
      || baseline.metricAnchors?.entries?.length !== 600) {
    errors.push('W6 metric anchor population must remain 600');
  }
  const metricIndexes = baseline.metricAnchors?.entries?.map(entry => entry.currentIndex) ?? [];
  if (metricIndexes.some((index, expected) => index !== expected)) {
    errors.push('W6 metric anchor indexes must remain the ordered range 0..599');
  }
  const metricAnchorHash = sha256Text(canonicalJson(baseline.metricAnchors?.entries ?? []));
  if (metricAnchorHash !== baseline.metricAnchors?.anchorsSha256) {
    errors.push('metric anchor hash mismatch');
  }
  const activeUnknown = baseline.inventory?.activeUnknown;
  if (activeUnknown?.count !== 44
      || activeUnknown.projectedLegacyPreservationCount !== 43
      || activeUnknown.handWrittenReferenceCount !== 1) {
    errors.push('active unknown disposition must preserve 43 projected aliases and 1 runtime predicate');
  }
  if ((activeUnknown?.rules ?? []).length !== activeUnknown?.count
      || (activeUnknown?.rules ?? []).some(rule => rule.decisionPlane !== 'layout-metric')) {
    errors.push('all 44 active unknown rules must remain explicit layout-metric evidence');
  }
  const runtimeExpectedCounts = {
    substitution: 265,
    governmentSuccessor: 65,
    displayFallbackProbes: 8,
    registeredFonts: 153,
    webfontSupply: 153,
    canvasKitPlans: 153,
  };
  for (const [name, count] of Object.entries(runtimeExpectedCounts)) {
    const snapshot = baseline.studioRuntime?.[name];
    if (snapshot?.count !== count || snapshot.rows?.length !== count) {
      errors.push(`studio runtime ${name} population must remain ${count}`);
      continue;
    }
    if (sha256Text(canonicalJson(snapshot.rows)) !== snapshot.sha256) {
      errors.push(`studio runtime ${name} hash mismatch`);
    }
  }
  const webfontLoad = baseline.studioRuntime?.webfontLoad;
  if (webfontLoad?.requestCount !== 153
      || sha256Text(canonicalJson({ css: webfontLoad.css, requests: webfontLoad.requests }))
        !== webfontLoad?.sha256) {
    errors.push('studio runtime webfont load projection mismatch');
  }
  const expectedProjectionCounts = {
    rustLayoutName: 171,
    rustLayoutMetric: 67,
    canvas2dPaint: 281,
    webfontSupply: 153,
    canvasKitSfnt: 158,
  };
  for (const name of PROJECTION_NAMES) {
    const projection = baseline.projections?.[name];
    if (!projection || projection.ruleCount !== projection.rules?.length) {
      errors.push(`${name}: rule count mismatch`);
      continue;
    }
    if (projection.ruleCount !== expectedProjectionCounts[name]) {
      errors.push(`${name}: pre-migration population changed`);
    }
    if (projection.rules.some(rule => (
      rule.status !== 'active' || !Array.isArray(rule.candidateIds) || rule.candidateIds.length === 0
    ))) {
      errors.push(`${name}: every projection row must be active and linked to W1 candidates`);
    }
    const hash = sha256Text(canonicalJson(projection.rules));
    if (hash !== projection.projectionSha256 || hash !== baseline.hashes?.[name]) {
      errors.push(`${name}: projection hash mismatch`);
    }
    const duplicateRuleIds = projection.rules
      .map(rule => rule.ruleId)
      .filter((ruleId, index, values) => values.indexOf(ruleId) !== index);
    if (duplicateRuleIds.length > 0) errors.push(`${name}: duplicate ruleId`);
  }
  const projectionHashes = Object.fromEntries(
    PROJECTION_NAMES.map(name => [name, baseline.projections?.[name]?.projectionSha256]),
  );
  if (sha256Text(canonicalJson(projectionHashes)) !== baseline.hashes?.projectionBundleSha256) {
    errors.push('projection bundle hash mismatch');
  }
  const registryInputHash = sha256Text(canonicalJson({
    candidateSequenceSha256: baseline.inventory?.currentCandidateProjectionSha256,
    ledgerSha256: baseline.inventory?.w1LedgerSha256,
    metricAnchorsSha256: baseline.metricAnchors?.anchorsSha256,
  }));
  if (registryInputHash !== baseline.hashes?.registryInputSha256) {
    errors.push('registry input hash mismatch');
  }
  return errors;
}

export function compareProjectionBaseline(expected, actual) {
  return canonicalJson(expected) === canonicalJson(actual)
    ? []
    : ['font rule projection pre-migration baseline differs from current source'];
}

function argumentValue(args, name) {
  const index = args.indexOf(name);
  return index === -1 || index === args.length - 1 ? null : args[index + 1];
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : '';
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    const command = process.argv[2];
    const args = process.argv.slice(3);
    const outputPath = path.resolve(
      process.cwd(),
      argumentValue(args, '--out') ?? DEFAULT_OUTPUT_PATH,
    );
    if (command === 'generate') {
      const baseline = buildProjectionBaseline(REPOSITORY_ROOT);
      fs.mkdirSync(path.dirname(outputPath), { recursive: true });
      fs.writeFileSync(outputPath, canonicalJson(baseline), 'utf8');
      process.stdout.write(
        `font rule projection baseline: ${baseline.inventory.ruleCandidateCount} candidates, ${baseline.metricAnchors.entryCount} metrics -> ${path.relative(REPOSITORY_ROOT, outputPath)}\n`,
      );
    } else if (command === 'check') {
      const expected = readJson(outputPath);
      const actual = buildProjectionBaseline(REPOSITORY_ROOT, expected.sourceCommit);
      const errors = [
        ...validateProjectionBaseline(expected),
        ...compareProjectionBaseline(expected, actual),
      ];
      if (errors.length > 0) throw new Error(errors.join('\n'));
      process.stdout.write('font rule projection baseline: ok\n');
    } else {
      throw new Error('usage: font_rule_projection_baseline.mjs <generate|check> [--out <path>]');
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
