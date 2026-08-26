#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import {
  canonicalJson,
  sha256Text,
  validateLedger,
} from './font_rule_ledger.mjs';

const REPOSITORY_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const CANDIDATE_DOCUMENT =
  'mydocs/tech/investigations/issue-4939/font_rule_candidates.json';
const CAUSAL_REPORT = 'mydocs/report/font_metrics_fallback_causal_lineage_20260816.md';
const TASK_PLAN = 'mydocs/plans/task_m100_4939.md';
const GOVERNMENT_MATRIX =
  'mydocs/tech/investigations/issue-4739/task_m100_4739_government_font_successor_matrix.md';
const LOCAL_FONT_REPORT = 'mydocs/tech/investigations/issue-4741/README.md';
const GENERATOR_VERSION = '4.0.0';

const GOVERNMENT_DIGESTS = [
  '정부상징 부처명_16040911.ttf sha256:9ff914274d89c97abe3c22934c1f5f049d5c82de3cf0a3bc6053ac139b8a111a',
  'ROKG_R.ttf sha256:849c61ec05c9b468266a6ee3e7020ddc7c1696c9b3b29469b4986cab5e243a50',
];

const METRIC_SURROGATE_SOURCES = new Set([
  'HY각헤드라인M',
  '본한글',
  '본한글vf',
  '본한글 Medium',
  '본한글M',
  '본고딕',
  '본고딕vf',
  'Source Han Sans',
  'Source Han Sans K',
  'Source Han Sans KR',
  'SourceHanSans',
  'SourceHanSansKR',
  'SourceHanSansK',
  'Noto Sans CJK KR',
  '본명조',
  '본명조vf',
  '본명조M',
  'Source Han Serif',
  'Source Han Serif K',
  'Source Han Serif KR',
  'SourceHanSerif',
  'SourceHanSerifKR',
  'SourceHanSerifK',
  'Noto Serif CJK KR',
]);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function countBy(values) {
  const counts = {};
  for (const value of values) counts[value] = (counts[value] ?? 0) + 1;
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) => left.localeCompare(right)));
}

function candidateReference(candidate) {
  return `${CANDIDATE_DOCUMENT}#${candidate.candidateId}`;
}

function normalizedConditions(candidate, profile = null) {
  const source = candidate.conditions ?? {};
  const conditions = {};
  if (source.languageSlot !== undefined) conditions.languageSlot = String(source.languageSlot);
  if (source.sourceAltType !== undefined || source.targetAltType !== undefined) {
    conditions.altType = `source:${source.sourceAltType ?? '*'}->target:${source.targetAltType ?? '*'}`;
  }
  for (const field of ['bold', 'italic', 'weight', 'availability']) {
    if (source[field] !== undefined) conditions[field] = source[field];
  }
  if (profile !== null) conditions.profile = profile;
  return conditions;
}

function sourceLocation(candidate) {
  return {
    path: candidate.sourceLocation.path,
    symbol: candidate.sourceLocation.symbol,
    selector: candidate.sourceLocation.selector,
  };
}

function baseEvidence(candidate, sourceCommit) {
  return [
    { kind: 'document', reference: candidateReference(candidate) },
    { kind: 'issue', reference: '#4939' },
    { kind: 'commit', reference: sourceCommit },
    { kind: 'document', reference: TASK_PLAN },
    { kind: 'document', reference: CAUSAL_REPORT },
  ];
}

function policyFor(candidate) {
  const policy = {
    relationType: 'unknown',
    evidenceStatus: 'unknown',
    evidence: [],
    tests: [],
    knownLimitations: [],
    status: 'active',
  };
  const boundary = candidate.sourceBoundaryId;

  if (candidate.candidateKind === 'metric-entry') {
    Object.assign(policy, {
      relationType: 'metric-entry',
      evidenceStatus: 'verified-by-test',
      tests: ['src/renderer/font_metrics_data.rs#index_matches_legacy_linear_scan_exhaustively'],
      knownLimitations: [
        `candidate metric shape: ${canonicalJson(candidate.conditions).trim()}`,
        'The original font bytes and generator provenance are not uniformly available.',
      ],
    });
  } else if (boundary.startsWith('rust-style-resolution.')) {
    Object.assign(policy, {
      relationType: 'style-fallback',
      evidenceStatus: 'verified-by-test',
      tests: ['src/renderer/style_resolver.rs#tests'],
      knownLimitations: ['Name compatibility does not establish SFNT identity or metric equivalence.'],
    });
  } else if (boundary === 'rust-metric.metric-alias') {
    const selfLoop = candidate.sourceFace === candidate.targetOrPolicy;
    if (METRIC_SURROGATE_SOURCES.has(candidate.sourceFace)) {
      Object.assign(policy, {
        relationType: 'metric-surrogate',
        evidenceStatus: 'historical',
        evidence: [{ kind: 'issue', reference: '#259' }],
        tests: ['src/renderer/font_metrics_data.rs#tests'],
        knownLimitations: ['The surrogate is a layout approximation, not a paint or identity alias.'],
      });
    } else {
      Object.assign(policy, {
        relationType: 'unknown',
        evidenceStatus: 'unknown',
        tests: ['src/renderer/font_metrics_data.rs#tests'],
        knownLimitations: [
          selfLoop
            ? 'Self-loop is intentional canonicalization in a single-pass Rust match; it is not byte identity.'
            : 'Determine whether this is a multilingual name alias or a metric surrogate from SFNT and metric provenance.',
        ],
      });
    }
  } else if (boundary === 'rust-metric.metric-lookup') {
    Object.assign(policy, {
      relationType: 'style-fallback',
      evidenceStatus: 'verified-by-test',
      evidence: [{ kind: 'issue', reference: '#4168' }],
      tests: ['src/renderer/font_metrics_data.rs#index_matches_legacy_linear_scan_exhaustively'],
      knownLimitations: ['Physical table order remains part of the lookup contract.'],
    });
  } else if (boundary === 'rust-measurement.hancom-space') {
    Object.assign(policy, {
      relationType: 'measured-overlay',
      evidenceStatus: 'verified-by-test',
      tests: ['src/renderer/layout/text_measurement.rs#tests'],
      knownLimitations: ['The overlay is gated by face and size and must not become a global metric alias.'],
    });
  } else if (boundary === 'rust-measurement.estimate-width') {
    Object.assign(policy, {
      relationType: 'unknown',
      evidenceStatus: 'inferred',
      tests: ['src/renderer/layout/text_measurement.rs#tests'],
      knownLimitations: ['Separate exact metric lookup, heuristic estimation, and synthetic styling in W2 trace output.'],
    });
  } else if (boundary === 'rust-paint-chain.installed-aliases') {
    const government = ['정부상징 부처명_16040911', 'Government_16040911']
      .includes(candidate.sourceFace);
    Object.assign(policy, government ? {
      relationType: 'official-successor',
      evidenceStatus: 'verified-by-bytes',
      evidence: [
        { kind: 'issue', reference: '#4739' },
        { kind: 'document', reference: GOVERNMENT_MATRIX },
        ...GOVERNMENT_DIGESTS.map(reference => ({ kind: 'font-digest', reference })),
      ],
      tests: ['src/renderer/mod.rs#installed_render_font_aliases'],
      knownLimitations: ['Successor similarity is not byte identity; U+3000 and vertical metrics differ.'],
    } : {
      relationType: 'paint-substitute',
      evidenceStatus: 'verified-by-test',
      tests: ['src/renderer/mod.rs#tests'],
      knownLimitations: ['Installed paint substitution does not alter the portable layout metric.'],
    });
  } else if (boundary === 'rust-paint-chain.generic-fallback') {
    Object.assign(policy, {
      relationType: 'generic-fallback',
      evidenceStatus: 'verified-by-test',
      tests: ['src/renderer/mod.rs#tests'],
      knownLimitations: ['Classifier output is a terminal paint chain, not an exact-face assertion.'],
    });
  } else if (boundary.startsWith('rust-paint-chain.')) {
    Object.assign(policy, {
      relationType: 'style-fallback',
      evidenceStatus: 'verified-by-test',
      tests: ['src/renderer/mod.rs#tests'],
      knownLimitations: ['Paint-only normalization must not mutate layout metric selection.'],
    });
  } else if (boundary === 'native-skia.system-family-style'
      || boundary.startsWith('paint-resource.')) {
    Object.assign(policy, {
      relationType: 'capability-detection',
      evidenceStatus: 'verified-by-test',
      tests: [boundary.startsWith('paint-resource.') ? 'src/paint/font.rs#tests' : 'src/renderer/skia/text_replay.rs#tests'],
      knownLimitations: ['Backend resource availability is distinct from CSS name availability.'],
    });
  } else if (boundary === 'native-skia.text-replay') {
    Object.assign(policy, {
      relationType: 'style-fallback',
      evidenceStatus: 'verified-by-test',
      tests: ['src/renderer/skia/text_replay.rs#tests'],
      knownLimitations: ['The chain is native-Skia-specific and does not imply Canvas2D or CanvasKit availability.'],
    });
  } else if (boundary === 'studio-substitution.substitution-tables') {
    const selfLoop = candidate.sourceFace === candidate.targetOrPolicy;
    Object.assign(policy, {
      relationType: 'paint-substitute',
      evidenceStatus: 'historical',
      tests: ['rhwp-studio/tests/font-substitution.test.ts'],
      knownLimitations: [selfLoop
        ? 'Self-loop is bounded by the visited-set and 15-step guard; it is not byte identity.'
        : 'Legacy table provenance is historical; duplicate keys preserve first-entry precedence.'],
    });
  } else if (boundary === 'studio-substitution.display-chain') {
    const target = candidate.targetOrPolicy;
    Object.assign(policy, {
      relationType: target.includes('government successor') ? 'official-successor'
        : target.includes('document substFont') ? 'document-substitution'
          : target.includes('system fallback') ? 'generic-fallback'
            : target.includes('web substitution') ? 'paint-substitute'
              : 'style-fallback',
      evidenceStatus: 'verified-by-test',
      evidence: [
        { kind: 'issue', reference: '#4739' },
        { kind: 'document', reference: GOVERNMENT_MATRIX },
      ],
      tests: ['rhwp-studio/tests/font-substitution.test.ts'],
      knownLimitations: ['This is a Canvas2D display chain; portable layout metrics remain separate.'],
    });
  } else if (boundary.startsWith('studio-supply.')) {
    Object.assign(policy, {
      relationType: 'supply-source',
      evidenceStatus: 'verified-by-test',
      evidence: [{ kind: 'issue', reference: '#4823' }],
      tests: boundary === 'studio-supply.font-list'
        ? ['scripts/frontend-font-assets.test.mjs']
        : ['rhwp-studio/tests/canvaskit-font-plan.test.ts'],
      knownLimitations: ['Supply availability does not establish layout-metric compatibility.'],
    });
  } else if (boundary.startsWith('studio-detection.')) {
    Object.assign(policy, {
      relationType: 'capability-detection',
      evidenceStatus: 'verified-by-test',
      evidence: [
        { kind: 'issue', reference: '#4741' },
        { kind: 'document', reference: LOCAL_FONT_REPORT },
      ],
      tests: ['rhwp-studio/tests/local-fonts.test.ts'],
      knownLimitations: ['Enumeration, raw Canvas probe, and CanvasKit SFNT bytes are independent capabilities.'],
    });
  } else if (boundary.startsWith('studio-canvas-patch.')) {
    Object.assign(policy, {
      relationType: boundary.endsWith('canvas-install') ? 'capability-detection' : 'paint-substitute',
      evidenceStatus: 'verified-by-test',
      evidence: [{ kind: 'document', reference: LOCAL_FONT_REPORT }],
      tests: ['rhwp-studio/tests/font-substitution.test.ts', 'rhwp-studio/tests/local-fonts.test.ts'],
      knownLimitations: ['Presence probing must bypass the installed Canvas2D substitution descriptor.'],
    });
  } else if (boundary.startsWith('asset-authority.')) {
    Object.assign(policy, {
      relationType: 'supply-source',
      evidenceStatus: 'historical',
      tests: ['scripts/frontend-font-assets.test.mjs'],
      knownLimitations: ['The authority row records distribution provenance, not runtime selection.'],
      status: 'historical',
    });
  } else if (boundary === 'tests-history.studio-substitution-test') {
    Object.assign(policy, {
      relationType: 'oracle-observation',
      evidenceStatus: 'verified-by-test',
      tests: ['rhwp-studio/tests/font-substitution.test.ts'],
      knownLimitations: ['A regression test is an executable contract, not a font-byte oracle.'],
      status: 'historical',
    });
  }
  return policy;
}

function variantsFor(candidate) {
  if (candidate.sourceBoundaryId === 'studio-supply.font-list') {
    const format = candidate.conditions.format ?? 'unknown';
    return [
      {
        suffix: 'canvas2d',
        profile: `canvas2d-css-${format}`,
        backends: ['studio', 'canvas2d'],
        target: candidate.targetOrPolicy,
        limitations: [`Canvas2D source expression: ${candidate.targetOrPolicy}`],
      },
      {
        suffix: 'canvaskit',
        profile: 'canvaskit-sfnt',
        backends: ['canvaskit'],
        target: candidate.conditions.canvasKitFile
          ?? `unavailable: no CanvasKit SFNT source for ${candidate.targetOrPolicy}`,
        limitations: [candidate.conditions.canvasKitFile
          ? `CanvasKit SFNT source expression: ${candidate.conditions.canvasKitFile}`
          : 'CanvasKit supply is intentionally unavailable and the font plan must fail closed.'],
      },
    ];
  }
  if (candidate.sourceBoundaryId === 'tests-history.government-font-matrix') {
    return [
      {
        suffix: 'source-exact',
        profile: 'source-exact',
        backends: ['oracle'],
        target: '정부상징 부처명_16040911 exact installed face',
        limitations: ['The legacy comparison font provenance is not proven to be an official public distribution.'],
      },
      {
        suffix: 'official-successor',
        profile: 'official-successor',
        backends: ['oracle'],
        target: 'ROKG / 대한민국정부상징체 R successor face',
        limitations: ['The successor is layout-compatible for most shared glyphs but is not byte-identical.'],
      },
      {
        suffix: 'hancom-missing-font',
        profile: 'hancom-missing-font',
        backends: ['oracle'],
        target: '한컴바탕 / Haansoft Batang observed PDF substitute',
        limitations: ['The missing-font PDF oracle must not be used as the source-exact oracle.'],
      },
    ];
  }
  return [{
    suffix: null,
    profile: candidate.backends.includes('canvaskit') && !candidate.backends.includes('canvas2d')
      ? 'canvaskit'
      : candidate.backends.includes('canvas2d') && !candidate.backends.includes('canvaskit')
        ? 'canvas2d'
        : null,
    backends: candidate.backends,
    target: candidate.targetOrPolicy,
    limitations: [],
  }];
}

function duplicatePrecedence(candidates) {
  const groups = new Map();
  for (const candidate of candidates) {
    if (candidate.sourceBoundaryId !== 'studio-substitution.substitution-tables') continue;
    const key = canonicalJson({
      sourceBoundaryId: candidate.sourceBoundaryId,
      sourceFace: candidate.sourceFace,
      conditions: candidate.conditions,
      backends: candidate.backends,
    });
    const rows = groups.get(key) ?? [];
    rows.push(candidate.candidateId);
    groups.set(key, rows);
  }
  const result = new Map();
  for (const rows of groups.values()) {
    if (rows.length > 1) rows.forEach((candidateId, order) => result.set(candidateId, order));
  }
  return result;
}

export function buildEvidenceLedger(snapshot) {
  if (snapshot.ruleCandidateKind !== 'font-rule-candidates') {
    throw new Error('Stage 4 requires a Stage 3 rule candidate snapshot');
  }
  const precedence = duplicatePrecedence(snapshot.ruleCandidates);
  const rules = [];
  for (const candidate of snapshot.ruleCandidates) {
    const policy = policyFor(candidate);
    for (const variant of variantsFor(candidate)) {
      const candidateSuffix = candidate.candidateId.slice('candidate.'.length);
      const suffix = variant.suffix ? `.${variant.suffix}` : '';
      const relationType = candidate.sourceBoundaryId === 'tests-history.government-font-matrix'
        ? 'oracle-observation'
        : policy.relationType;
      const evidenceStatus = candidate.sourceBoundaryId === 'tests-history.government-font-matrix'
        ? (variant.profile === 'hancom-missing-font' ? 'verified-by-oracle' : 'verified-by-bytes')
        : policy.evidenceStatus;
      const extraEvidence = candidate.sourceBoundaryId === 'tests-history.government-font-matrix'
        ? [
          { kind: 'issue', reference: '#4739' },
          { kind: 'document', reference: GOVERNMENT_MATRIX },
          ...GOVERNMENT_DIGESTS.map(reference => ({ kind: 'font-digest', reference })),
        ]
        : policy.evidence;
      rules.push({
        ruleId: `rule.${candidate.ownerId}.${candidateSuffix}${suffix}`,
        sourceOwner: candidate.ownerId,
        sourceLocation: sourceLocation(candidate),
        decisionPlane: candidate.decisionPlane,
        relationType,
        sourceFace: candidate.sourceFace,
        targetFaceOrPolicy: variant.target,
        conditions: normalizedConditions(candidate, variant.profile),
        backends: variant.backends,
        order: candidate.order ?? precedence.get(candidate.candidateId) ?? null,
        evidence: [...baseEvidence(candidate, snapshot.sourceCommit), ...extraEvidence],
        evidenceStatus,
        licenseOrDistribution: candidate.decisionPlane === 'supply'
          ? 'See assets/fonts/FONTS.md, ttfs/FONTS.md, and THIRD_PARTY_LICENSES.md; no new binary redistribution.'
          : candidate.decisionPlane === 'oracle'
            ? 'External comparison bytes are not redistributed; only approved filename and SHA-256 evidence is recorded.'
            : 'Not a font binary distribution rule.',
        tests: policy.tests,
        knownLimitations: [...policy.knownLimitations, ...variant.limitations],
        status: policy.status,
      });
    }
  }
  return {
    schemaVersion: '1.0',
    kind: 'font-rule-investigation-ledger',
    issue: 4939,
    sourceCommit: snapshot.sourceCommit,
    rules,
  };
}

function referencePath(reference) {
  return reference.split('#')[0];
}

function candidateRefs(rule) {
  return rule.evidence
    .filter(entry => entry.kind === 'document' && entry.reference.startsWith(`${CANDIDATE_DOCUMENT}#candidate.`))
    .map(entry => entry.reference.slice(CANDIDATE_DOCUMENT.length + 1));
}

function decisionKey(rule) {
  return canonicalJson({
    sourceOwner: rule.sourceOwner,
    sourceLocation: rule.sourceLocation,
    decisionPlane: rule.decisionPlane,
    sourceFace: rule.sourceFace,
    conditions: rule.conditions,
    backends: rule.backends,
  });
}

function cycleAudit(snapshot, ledger) {
  const candidates = new Map(snapshot.ruleCandidates.map(candidate => [candidate.candidateId, candidate]));
  const edgesByScope = new Map();
  for (const rule of ledger.rules) {
    const ref = candidateRefs(rule)[0];
    const candidate = candidates.get(ref);
    if (!candidate || !['finite-mapping', 'ordered-chain'].includes(candidate.candidateKind)) continue;
    if (!rule.sourceFace || !candidate.targetOrPolicy || candidate.targetOrPolicy.includes(':')) continue;
    const scope = canonicalJson({
      sourceOwner: rule.sourceOwner,
      decisionPlane: rule.decisionPlane,
      profile: rule.conditions.profile ?? null,
      backends: rule.backends,
    });
    const edges = edgesByScope.get(scope) ?? [];
    edges.push({ from: rule.sourceFace, to: rule.targetFaceOrPolicy, rule });
    edgesByScope.set(scope, edges);
  }
  const cycles = [];
  for (const [scope, edges] of edgesByScope) {
    const adjacency = new Map();
    for (const edge of edges) {
      const list = adjacency.get(edge.from) ?? [];
      list.push(edge);
      adjacency.set(edge.from, list);
    }
    const nodes = new Set(edges.flatMap(edge => [edge.from, edge.to]));
    let nextIndex = 0;
    const indexes = new Map();
    const lows = new Map();
    const stack = [];
    const onStack = new Set();
    function visit(node) {
      indexes.set(node, nextIndex);
      lows.set(node, nextIndex);
      nextIndex += 1;
      stack.push(node);
      onStack.add(node);
      for (const edge of adjacency.get(node) ?? []) {
        if (!indexes.has(edge.to)) {
          visit(edge.to);
          lows.set(node, Math.min(lows.get(node), lows.get(edge.to)));
        } else if (onStack.has(edge.to)) {
          lows.set(node, Math.min(lows.get(node), indexes.get(edge.to)));
        }
      }
      if (lows.get(node) === indexes.get(node)) {
        const members = [];
        let member;
        do {
          member = stack.pop();
          onStack.delete(member);
          members.push(member);
        } while (member !== node);
        const selfLoop = members.length === 1
          && (adjacency.get(members[0]) ?? []).some(edge => edge.to === members[0]);
        if (members.length > 1 || selfLoop) {
          const memberSet = new Set(members);
          const cycleRules = edges.filter(edge => memberSet.has(edge.from) && memberSet.has(edge.to));
          cycles.push({ scope: JSON.parse(scope), members: members.sort(), rules: cycleRules.map(edge => edge.rule.ruleId) });
        }
      }
    }
    for (const node of nodes) if (!indexes.has(node)) visit(node);
  }
  return cycles;
}

export function validateEvidenceLedger(snapshot, ledger, repositoryRoot = REPOSITORY_ROOT) {
  const errors = [...validateLedger(ledger)];
  const candidates = new Map(snapshot.ruleCandidates.map(candidate => [candidate.candidateId, candidate]));
  const coverage = new Map([...candidates.keys()].map(id => [id, []]));
  const commitValidity = new Map();

  for (const rule of ledger.rules ?? []) {
    const refs = candidateRefs(rule);
    if (refs.length !== 1) {
      errors.push(`${rule.ruleId}: expected exactly one candidate evidence reference`);
      continue;
    }
    if (!candidates.has(refs[0])) errors.push(`${rule.ruleId}: orphan candidate reference ${refs[0]}`);
    else coverage.get(refs[0]).push(rule);

    if (rule.relationType === 'identity-alias') {
      const hasDigest = rule.evidence.some(entry => entry.kind === 'font-digest');
      if (rule.evidenceStatus !== 'verified-by-bytes' || !hasDigest) {
        errors.push(`${rule.ruleId}: identity-alias requires verified-by-bytes and font-digest evidence`);
      }
    }
    if (rule.backends.includes('canvas2d') && rule.backends.includes('canvaskit')) {
      errors.push(`${rule.ruleId}: Canvas2D and CanvasKit profiles must be separate`);
    }
    for (const entry of rule.evidence) {
      if (!['document', 'test', 'oracle'].includes(entry.kind)) continue;
      const localPath = referencePath(entry.reference);
      if (!localPath || !fs.existsSync(path.resolve(repositoryRoot, localPath))) {
        errors.push(`${rule.ruleId}: orphan ${entry.kind} evidence ${entry.reference}`);
      }
    }
    for (const testReference of rule.tests) {
      if (!fs.existsSync(path.resolve(repositoryRoot, referencePath(testReference)))) {
        errors.push(`${rule.ruleId}: orphan test ${testReference}`);
      }
    }
    for (const entry of rule.evidence.filter(value => value.kind === 'commit')) {
      if (!commitValidity.has(entry.reference)) {
        try {
          execFileSync('git', ['cat-file', '-e', `${entry.reference}^{commit}`], {
            cwd: repositoryRoot,
            stdio: 'ignore',
          });
          commitValidity.set(entry.reference, true);
        } catch {
          commitValidity.set(entry.reference, false);
        }
      }
      if (!commitValidity.get(entry.reference)) {
        errors.push(`${rule.ruleId}: orphan commit ${entry.reference}`);
      }
    }
    for (const entry of rule.evidence.filter(value => value.kind === 'font-digest')) {
      if (!/sha256:[0-9a-f]{64}$/.test(entry.reference)) {
        errors.push(`${rule.ruleId}: malformed font digest ${entry.reference}`);
      }
    }
  }

  for (const [candidateId, rules] of coverage) {
    const candidate = candidates.get(candidateId);
    const expected = candidate.sourceBoundaryId === 'studio-supply.font-list' ? 2
      : candidate.sourceBoundaryId === 'tests-history.government-font-matrix' ? 3
        : 1;
    if (rules.length !== expected) {
      errors.push(`${candidateId}: ledger coverage ${rules.length} != expected ${expected}`);
    }
    if (expected > 1) {
      const profiles = rules.map(rule => rule.conditions.profile);
      if (new Set(profiles).size !== expected || profiles.some(profile => !profile)) {
        errors.push(`${candidateId}: profile split must use ${expected} unique non-empty profiles`);
      }
    }
  }

  const groups = new Map();
  for (const rule of ledger.rules ?? []) {
    const key = decisionKey(rule);
    const rows = groups.get(key) ?? [];
    rows.push(rule);
    groups.set(key, rows);
  }
  let conflictGroupCount = 0;
  for (const rows of groups.values()) {
    const targets = new Set(rows.map(rule => rule.targetFaceOrPolicy));
    if (targets.size !== rows.length) {
      errors.push(`${rows[0].ruleId}: duplicate target for the same decision key`);
    }
    if (targets.size < 2) continue;
    conflictGroupCount += 1;
    const orders = rows.map(rule => rule.order);
    if (orders.some(order => order === null) || new Set(orders).size !== orders.length) {
      errors.push(`${rows[0].ruleId}: conflicting targets require unique explicit order`);
    }
  }

  const cycles = cycleAudit(snapshot, ledger);
  const rulesById = new Map((ledger.rules ?? []).map(rule => [rule.ruleId, rule]));
  for (const cycle of cycles) {
    for (const ruleId of cycle.rules) {
      const documented = rulesById.get(ruleId).knownLimitations
        .some(value => /self-loop|cycle|visited-set/i.test(value));
      if (!documented) errors.push(`${ruleId}: cycle is not documented in knownLimitations`);
    }
  }
  return { errors, cycles, conflictGroupCount };
}

export function ledgerSummary(snapshot, ledger, audit) {
  const unknownRules = ledger.rules.filter(rule => rule.relationType === 'unknown'
    || rule.evidenceStatus === 'unknown');
  return {
    generatorVersion: GENERATOR_VERSION,
    candidateCount: snapshot.ruleCandidates.length,
    ledgerRuleCount: ledger.rules.length,
    profileSplitCandidateCount: snapshot.ruleCandidates.filter(candidate =>
      candidate.sourceBoundaryId === 'studio-supply.font-list'
      || candidate.sourceBoundaryId === 'tests-history.government-font-matrix').length,
    countsByOwner: countBy(ledger.rules.map(rule => rule.sourceOwner)),
    countsByRelation: countBy(ledger.rules.map(rule => rule.relationType)),
    countsByEvidenceStatus: countBy(ledger.rules.map(rule => rule.evidenceStatus)),
    countsByProfile: countBy(ledger.rules.map(rule => rule.conditions.profile ?? 'shared-or-not-applicable')),
    unknownRuleCount: unknownRules.length,
    unknownRuleIds: unknownRules.map(rule => rule.ruleId),
    conflictGroupCount: audit.conflictGroupCount,
    documentedCycleCount: audit.cycles.length,
    validationErrorCount: audit.errors.length,
    ledgerSha256: sha256Text(canonicalJson(ledger)),
  };
}

function markdownCell(value) {
  return String(value).replaceAll('|', '\\|').replaceAll('\n', ' ');
}

export function renderLedgerSummary(snapshot, ledger, audit) {
  const summary = ledgerSummary(snapshot, ledger, audit);
  const relationRows = Object.entries(summary.countsByRelation)
    .map(([name, count]) => `| \`${name}\` | ${count} |`).join('\n');
  const ownerRows = Object.entries(summary.countsByOwner)
    .map(([name, count]) => `| \`${name}\` | ${count} |`).join('\n');
  const evidenceRows = Object.entries(summary.countsByEvidenceStatus)
    .map(([name, count]) => `| \`${name}\` | ${count} |`).join('\n');
  const profileRows = Object.entries(summary.countsByProfile)
    .map(([name, count]) => `| \`${name}\` | ${count} |`).join('\n');
  const unknownRows = ledger.rules
    .filter(rule => rule.relationType === 'unknown' || rule.evidenceStatus === 'unknown')
    .map(rule => `| \`${rule.ruleId}\` | ${markdownCell(rule.sourceFace ?? '(predicate)')} | ${markdownCell(rule.targetFaceOrPolicy)} | ${markdownCell(rule.knownLimitations[0])} |`)
    .join('\n');
  const cycles = audit.cycles
    .map(cycle => `| ${markdownCell(cycle.scope.sourceOwner)} | ${markdownCell(cycle.scope.decisionPlane)} | ${markdownCell(cycle.members.join(' -> '))} | ${markdownCell(cycle.rules.join(', '))} |`)
    .join('\n');
  return `---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4939.md
last_verified: 2026-08-16
---

# #4939 Font Rule Ledger 요약

## 판정 결과

- source candidate: **${summary.candidateCount.toLocaleString('en-US')}개**
- ledger rule: **${summary.ledgerRuleCount.toLocaleString('en-US')}개**
- 허용된 profile split candidate: **${summary.profileSplitCandidateCount}개**
  - Studio \`FONT_LIST\` 153개는 Canvas2D CSS supply와 CanvasKit SFNT supply를 분리했다.
  - 정부상징 oracle 1개는 source exact, official successor, Hancom missing-font 3개 profile로 분리했다.
- 미확정 relation 또는 evidence: **${summary.unknownRuleCount}개**
- 설명된 상충 target group: **${summary.conflictGroupCount}개**
- 설명된 cycle: **${summary.documentedCycleCount}개**
- validator error: **${summary.validationErrorCount}개**
- ledger canonical SHA-256: \`${summary.ledgerSha256}\`

원장은 현재 source를 설명하는 investigation snapshot이다. runtime registry가 아니며 제품 코드가 이
JSON을 import하지 않는다. \`identity-alias\`는 SFNT/byte 근거 없이는 한 건도 승격하지 않았다.

## relation별 수량

| relation | rule |
| --- | ---: |
${relationRows}

## owner별 수량

| owner | rule |
| --- | ---: |
${ownerRows}

## evidence status별 수량

| evidence status | rule |
| --- | ---: |
${evidenceRows}

## backend/profile 분리

| profile | rule |
| --- | ---: |
${profileRows}

Canvas2D의 CSS family 사용 가능성과 CanvasKit의 SFNT byte 조달을 같은 행에 두지 않았다. CanvasKit
source가 없는 \`FONT_LIST\` entry도 누락시키지 않고 \`unavailable\` 정책으로 보존했다. 정부상징
missing-font PDF 관찰은 source exact 또는 ROKG successor의 정답지로 사용하지 않는다.

## 충돌·순환 감사

상충 target 14개 group은 두 부류다.

- 6개는 source에 이미 \`order\`가 있는 lookup/fallback chain이다.
- 8개는 Studio \`SUBST_TABLES\`의 동일 source·language·altType 다중 target이다. runtime의
  \`Map\` 구축이 첫 entry를 보존하므로 물리 배열 순서를 원장 \`order\` 0, 1로 복원했다.

동일 decision key의 order 중복은 0개다. 탐지한 cycle은 모두 self-loop이며, 다단 순환은 0개다.

| owner | plane | member | rule |
| --- | --- | --- | --- |
${cycles}

Rust metric self-loop는 단일 match의 canonical spelling 반환이다. Studio self-loop는 visited-set과
15단계 상한으로 종료된다. 어느 쪽도 byte identity 증거로 사용하지 않는다.

## 미확정 규칙과 후속 질문

| rule | source | target/policy | 후속 질문 |
| --- | --- | --- | --- |
${unknownRows}

미확정 규칙은 삭제하거나 임의로 \`identity-alias\`로 바꾸지 않았다. W2/W8에서 다음 순서로
판정한다.

1. source/target SFNT name table과 허용된 byte digest로 multilingual name alias인지 확인한다.
2. name alias가 아니면 실제 advance·coverage 차이를 측정해 \`metric-surrogate\` 여부를 판정한다.
3. generic width estimator는 exact metric miss, heuristic, faux styling provenance를 W2 trace에서
   분리한 뒤 더 구체적인 relation으로 승격한다.

## 재생성과 검사

\`\`\`bash
node scripts/font_rule_ledger_evidence.mjs build \\
  --candidates mydocs/tech/investigations/issue-4939/font_rule_candidates.json \\
  --ledger mydocs/tech/investigations/issue-4939/font_rule_ledger.json \\
  --summary mydocs/tech/investigations/issue-4939/font_rule_ledger_summary.md
node scripts/font_rule_ledger_evidence.mjs check \\
  --candidates mydocs/tech/investigations/issue-4939/font_rule_candidates.json \\
  --ledger mydocs/tech/investigations/issue-4939/font_rule_ledger.json
node --test scripts/tests/font_rule_ledger_evidence.test.mjs
\`\`\`
`;
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
    const candidatePath = path.resolve(process.cwd(), argumentValue(args, '--candidates') ?? '');
    const ledgerPath = path.resolve(process.cwd(), argumentValue(args, '--ledger') ?? '');
    if (!command || !argumentValue(args, '--candidates') || !argumentValue(args, '--ledger')) {
      throw new Error('usage: font_rule_ledger_evidence.mjs <build|check> --candidates <path> --ledger <path>');
    }
    const snapshot = readJson(candidatePath);
    const ledger = command === 'build' ? buildEvidenceLedger(snapshot) : readJson(ledgerPath);
    const audit = validateEvidenceLedger(snapshot, ledger, REPOSITORY_ROOT);
    if (audit.errors.length > 0) throw new Error(audit.errors.join('\n'));
    if (command === 'build') {
      fs.writeFileSync(ledgerPath, canonicalJson(ledger), 'utf8');
      const summaryArgument = argumentValue(args, '--summary');
      if (summaryArgument) {
        fs.writeFileSync(
          path.resolve(process.cwd(), summaryArgument),
          renderLedgerSummary(snapshot, ledger, audit),
          'utf8',
        );
      }
    }
    else if (command !== 'check') throw new Error(`unknown command: ${command}`);
    const summary = ledgerSummary(snapshot, ledger, audit);
    process.stdout.write(`${JSON.stringify(summary)}\n`);
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
