#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { createHash } from 'node:crypto';
import { StringDecoder } from 'node:string_decoder';
import { fileURLToPath } from 'node:url';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
const DEFAULT_CONTRACT_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
  'font_typesetting_risk_contract.json',
);
const HEADER_MARKER = ',"legacyUsage":[';
const DECISION_MARKER = '"decisionUsage":[';
const MAX_HEADER_BYTES = 4 * 1024 * 1024;
const MAX_ROW_BYTES = 16 * 1024 * 1024;
const FORMATS = ['hwp', 'hwpx'];

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function compareText(left, right) {
  return Buffer.from(String(left), 'utf8').compare(Buffer.from(String(right), 'utf8'));
}

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (isObject(value)) {
    return Object.fromEntries(
      Object.keys(value).sort(compareText).map(key => [key, canonical(value[key])]),
    );
  }
  return value;
}

function canonicalJson(value) {
  return JSON.stringify(canonical(value));
}

function sha256Text(value) {
  return createHash('sha256').update(value).digest('hex');
}

function safeCount(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function checkedAdd(left, right, label) {
  if (!safeCount(left) || !safeCount(right) || !Number.isSafeInteger(left + right)) {
    throw new Error(`${label} exceeds the non-negative safe integer range`);
  }
  return left + right;
}

function exactArray(actual, expected) {
  return canonicalJson(actual) === canonicalJson(expected);
}

function exactKeys(value, expected, label) {
  if (!isObject(value)) throw new Error(`${label} must be an object`);
  if (!exactArray(Object.keys(value).sort(compareText), [...expected].sort(compareText))) {
    throw new Error(`${label} schema drift`);
  }
}

function stringSet(value) {
  return Array.isArray(value)
    && value.length > 0
    && value.every(entry => typeof entry === 'string' && entry.length > 0)
    && new Set(value).size === value.length;
}

export function validateTypesettingRiskContract(contract) {
  const errors = [];
  if (!isObject(contract)) return ['contract must be an object'];
  if (contract.schemaVersion !== 1) errors.push('schemaVersion must be 1');
  if (contract.kind !== 'font-typesetting-risk-contract') {
    errors.push('kind must be font-typesetting-risk-contract');
  }
  if (contract.issue !== 4962) errors.push('issue must be 4962');

  const projection = contract.compatibilityProjection;
  if (projection?.sourceArray !== 'decisionUsage') {
    errors.push('compatibilityProjection.sourceArray must be decisionUsage');
  }
  if (!stringSet(projection?.requiredFields)) {
    errors.push('compatibilityProjection.requiredFields must be a unique string set');
  }
  if (!exactArray(projection?.riskCategories, ['char-miss', 'face-miss', 'heuristic'])) {
    errors.push('riskCategories must preserve the W4 three-category order');
  }
  if (!exactArray(
    [...(projection?.observationalCategories ?? [])].sort(compareText),
    ['exact-hit', 'identity-alias-hit', 'measured-overlay', 'metric-surrogate']
      .sort(compareText),
  )) {
    errors.push('observationalCategories must preserve all coverage-success categories');
  }

  const identity = contract.candidateIdentity;
  if (identity?.documentFaceKey !== 'font'
      || identity.metricRequestClusterKey !== 'metricRequestedFace'
      || !exactArray(identity.splitDocumentFaceBy, [])
      || identity.mergeMetricClustersIntoDocumentFaces !== false
      || identity.nullMetricRequestPolicy !== 'preserve-unavailable-cluster'
      || identity.unknownIdentityGuessing !== false) {
    errors.push('candidate identity must keep document faces exact and clusters diagnostic');
  }
  if (!exactArray(
    [...(identity?.preservedDistributions ?? [])].sort(compareText),
    ['format', 'language', 'bold', 'italic', 'kerning'].sort(compareText),
  )) {
    errors.push('candidate identity distributions do not match the W4 contract');
  }

  const axes = contract.editingAxes;
  if (axes?.compressed !== 'ratio < 100 || spacing < 0'
      || axes.extremeCompressed !== 'ratio <= 90 || spacing <= -5'
      || !exactArray(axes.compressionIndicators, [
        'ratio < 100',
        'ratio <= 90',
        'spacing < 0',
        'spacing <= -5',
      ])) {
    errors.push('editing-axis compression definitions have drifted');
  }
  const frame = axes?.fixedFrameContextProxy;
  if (frame?.outputField !== 'fixedFrameContextProxy'
      || frame.matchMode !== 'context-token-any'
      || frame.geometryClaim !== false
      || !exactArray(
        [...(frame?.tokens ?? [])].sort(compareText),
        ['table-cell', 'text-box', 'caption', 'header', 'footer', 'master-page']
          .sort(compareText),
      )) {
    errors.push('fixed-frame context proxy contract has drifted');
  }
  const lanes = axes?.lineSegLanes;
  if (lanes?.['stored-line-lane']?.storedLineSeg !== true
      || lanes['stored-line-lane'].validityClaim !== false
      || lanes?.['fresh-candidate-lane']?.storedLineSeg !== false
      || lanes['fresh-candidate-lane'].validityClaim !== false
      || lanes?.riskMultiplier !== false) {
    errors.push('LineSeg lanes must remain presence-only and unweighted');
  }

  const risk = contract.riskMass;
  if (risk?.categoryWeights !== false
      || risk.compressionBase !== 1
      || risk.compressionIndicatorIncrement !== 1
      || risk.fixedFrameProxyFactor !== 2
      || risk.otherContextFactor !== 1) {
    errors.push('risk mass coefficients have drifted');
  }
  if (!stringSet(risk?.requiredOutputs) || !stringSet(risk?.reconciliation)) {
    errors.push('risk mass output and reconciliation inventories are required');
  }

  const ranking = contract.ranking;
  if (ranking?.primary !== 'document-face-risk-rank'
      || ranking.secondary !== 'metric-request-cluster-rank'
      || ranking.unstableResultPolicy !== 'publish-cumulative-risk-bands') {
    errors.push('ranking identities or unstable-result policy have drifted');
  }
  const expectedSort = [
    ['baseRiskMass', 'descending'],
    ['compressedFixedContextRiskCharacters', 'descending'],
    ['riskCharacters', 'descending'],
    ['totalUsageCharacters', 'descending'],
    ['documentFaceUtf8', 'ascending'],
  ];
  const actualSort = (ranking?.sort ?? []).map(entry => [entry?.field, entry?.direction]);
  if (!exactArray(actualSort, expectedSort)) errors.push('ranking sort order has drifted');

  const privacy = contract.privacy;
  if (privacy?.rawRowsPersisted !== false
      || privacy.localOutputDirectory !== 'output/poc/font-typesetting-risk'
      || privacy.localOutputMode !== '0600'
      || !stringSet(privacy.forbiddenKeys)
      || !stringSet(privacy.forbiddenStringPatterns)) {
    errors.push('privacy boundary is incomplete');
  }
  const scope = contract.scope;
  for (const field of [
    'rerunPrivateCorpus',
    'changeRenderer',
    'changeMetricDatabase',
    'changeFallback',
    'changeFontAssets',
    'publishPrivateAggregate',
  ]) {
    if (scope?.[field] !== false) errors.push(`scope.${field} must remain false`);
  }
  return errors;
}

function compareObservedFrozen(label, observed, expected, errors) {
  if (!isObject(observed)) {
    errors.push(`${label} observation is missing`);
    return;
  }
  if (observed.mode !== expected.requiredMode) errors.push(`${label} mode does not match 0600`);
  if (observed.bytes !== expected.bytes) errors.push(`${label} byte length has drifted`);
  if (observed.fileSha256 !== expected.fileSha256) {
    errors.push(`${label} file SHA-256 has drifted`);
  }
  if (observed.aggregateSha256 !== expected.aggregateSha256) {
    errors.push(`${label} aggregate SHA-256 has drifted`);
  }
  if (observed.sourceCommit !== expected.sourceCommit) {
    errors.push(`${label} source commit has drifted`);
  }
}

export function validateRiskInputPreconditions(observed, contract) {
  const errors = [];
  compareObservedFrozen('primary', observed?.primary, contract.inputPreconditions.primary, errors);
  compareObservedFrozen(
    'determinism peer',
    observed?.determinismPeer,
    contract.inputPreconditions.determinismPeer,
    errors,
  );
  const ingress = observed?.postMergeIngress;
  const expected = contract.inputPreconditions.postMergeIngress;
  if (!isObject(ingress)) {
    errors.push('post-merge ingress observation is missing');
  } else {
    if (ingress.baselineMode !== expected.requiredMode
        || ingress.currentMode !== expected.requiredMode) {
      errors.push('post-merge ingress modes do not match 0600');
    }
    for (const field of [
      'documentCount',
      'baselineSourceCommit',
      'currentSourceCommit',
      'semanticProjectionSha256',
    ]) {
      if (ingress[field] !== expected[field]) {
        errors.push(`post-merge ingress ${field} has drifted`);
      }
    }
  }
  return errors;
}

function walkSensitive(value, location, forbiddenKeys, findings) {
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
      walkSensitive(entry, `${location}[${index}]`, forbiddenKeys, findings)
    ));
    return;
  }
  if (!isObject(value)) return;
  for (const [key, entry] of Object.entries(value)) {
    const next = `${location}.${key}`;
    if (forbiddenKeys.has(key)) findings.push({ location: next, reason: 'forbiddenKey' });
    walkSensitive(entry, next, forbiddenKeys, findings);
  }
}

export function findSensitiveTypesettingRiskValues(value, contract) {
  const findings = [];
  walkSensitive(value, '$', new Set(contract.privacy.forbiddenKeys), findings);
  return findings;
}

function emptyCategoryCounts(contract) {
  return Object.fromEntries(contract.compatibilityProjection.riskCategories.map(key => [key, 0]));
}

function emptyFormatCounts() {
  return Object.fromEntries(FORMATS.map(format => [format, 0]));
}

function emptyCandidate(name, contract) {
  return {
    name,
    totalUsageCharacters: 0,
    riskCharacters: 0,
    categoryRiskCharacters: emptyCategoryCounts(contract),
    compressedFixedContextRiskCharacters: 0,
    storedRiskMass: 0,
    freshCandidateRiskMass: 0,
    baseRiskMass: 0,
    formatCharacters: emptyFormatCounts(),
    riskProfile: {
      formatCharacters: emptyFormatCounts(),
      languageCharacters: new Map(),
      boldCharacters: 0,
      italicCharacters: 0,
      kerningCharacters: 0,
    },
  };
}

function addMapCount(map, key, value, label) {
  map.set(key, checkedAdd(map.get(key) ?? 0, value, label));
}

function contextTokens(context) {
  if (typeof context !== 'string' || context.length === 0) {
    throw new Error('decisionUsage.context must be a non-empty string');
  }
  return context.split('+');
}

function fixedFrameContextProxy(row, contract) {
  const tokens = new Set(contextTokens(row.context));
  return contract.editingAxes.fixedFrameContextProxy.tokens.some(token => tokens.has(token));
}

function rowRisk(row, contract) {
  const riskCategories = new Set(contract.compatibilityProjection.riskCategories);
  if (!riskCategories.has(row.coverageCategory)) return null;
  const compressed = row.ratio < 100 || row.spacing < 0;
  const compressionFactor = 1
    + Number(row.ratio < 100)
    + Number(row.ratio <= 90)
    + Number(row.spacing < 0)
    + Number(row.spacing <= -5);
  const fixed = fixedFrameContextProxy(row, contract);
  const frameFactor = fixed
    ? contract.riskMass.fixedFrameProxyFactor
    : contract.riskMass.otherContextFactor;
  const mass = row.charCount * compressionFactor * frameFactor;
  if (!Number.isSafeInteger(mass)) throw new Error('row risk mass exceeds the safe integer range');
  return {
    compressed,
    fixedFrameContextProxy: fixed,
    mass,
  };
}

function validateDecisionRow(row, index, contract) {
  exactKeys(
    row,
    contract.compatibilityProjection.requiredFields,
    `decisionUsage row ${index}`,
  );
  if (!FORMATS.includes(row.format)) throw new Error(`decisionUsage row ${index} format is invalid`);
  if (typeof row.font !== 'string' || row.font.length === 0) {
    throw new Error(`decisionUsage row ${index} font is invalid`);
  }
  if (row.metricRequestedFace !== null
      && (typeof row.metricRequestedFace !== 'string' || row.metricRequestedFace.length === 0)) {
    throw new Error(`decisionUsage row ${index} metricRequestedFace is invalid`);
  }
  if (!Number.isSafeInteger(row.ratio) || !Number.isSafeInteger(row.spacing)) {
    throw new Error(`decisionUsage row ${index} ratio or spacing is invalid`);
  }
  if (typeof row.storedLineSeg !== 'boolean') {
    throw new Error(`decisionUsage row ${index} storedLineSeg is invalid`);
  }
  for (const field of ['bold', 'italic', 'kerning']) {
    if (typeof row[field] !== 'boolean') {
      throw new Error(`decisionUsage row ${index} ${field} is invalid`);
    }
  }
  for (const field of ['charCount', 'documentCount', 'paragraphCount', 'runCount']) {
    if (!safeCount(row[field])) throw new Error(`decisionUsage row ${index} ${field} is invalid`);
  }
  const knownCategories = new Set([
    ...contract.compatibilityProjection.riskCategories,
    ...contract.compatibilityProjection.observationalCategories,
  ]);
  if (row.coverageCategory !== null && !knownCategories.has(row.coverageCategory)) {
    throw new Error(`decisionUsage row ${index} coverageCategory is unclassified`);
  }
  if (row.sourceJoinStatus !== 'joined') {
    throw new Error(`decisionUsage row ${index} is not a joined source row`);
  }
}

class RiskAccumulator {
  constructor(contract) {
    this.contract = contract;
    this.rowCount = 0;
    this.totalUsageCharacters = 0;
    this.riskCharacters = 0;
    this.storedRiskMass = 0;
    this.freshCandidateRiskMass = 0;
    this.baseRiskMass = 0;
    this.categoryRiskCharacters = emptyCategoryCounts(contract);
    this.formatRiskCharacters = emptyFormatCounts();
    this.laneRiskCharacters = {
      'stored-line-lane': 0,
      'fresh-candidate-lane': 0,
    };
    this.documentFaces = new Map();
    this.metricClusters = new Map();
  }

  addRow(row) {
    const index = this.rowCount;
    validateDecisionRow(row, index, this.contract);
    this.rowCount += 1;
    this.totalUsageCharacters = checkedAdd(
      this.totalUsageCharacters,
      row.charCount,
      'total usage characters',
    );
    let documentFace = this.documentFaces.get(row.font);
    if (!documentFace) {
      documentFace = emptyCandidate(row.font, this.contract);
      this.documentFaces.set(row.font, documentFace);
    }
    documentFace.totalUsageCharacters = checkedAdd(
      documentFace.totalUsageCharacters,
      row.charCount,
      `document face ${row.font} total usage`,
    );
    documentFace.formatCharacters[row.format] = checkedAdd(
      documentFace.formatCharacters[row.format],
      row.charCount,
      `document face ${row.font} format usage`,
    );

    const risk = rowRisk(row, this.contract);
    if (!risk) return;
    this.addRisk(documentFace, row, risk, `document face ${row.font}`);
    const clusterKey = canonicalJson(row.metricRequestedFace);
    let cluster = this.metricClusters.get(clusterKey);
    if (!cluster) {
      cluster = emptyCandidate(row.metricRequestedFace, this.contract);
      cluster.documentFaces = new Set();
      this.metricClusters.set(clusterKey, cluster);
    }
    cluster.documentFaces.add(row.font);
    cluster.totalUsageCharacters = checkedAdd(
      cluster.totalUsageCharacters,
      row.charCount,
      'metric request cluster risk usage',
    );
    cluster.formatCharacters[row.format] = checkedAdd(
      cluster.formatCharacters[row.format],
      row.charCount,
      'metric request cluster format usage',
    );
    this.addRisk(cluster, row, risk, 'metric request cluster');

    this.riskCharacters = checkedAdd(this.riskCharacters, row.charCount, 'risk characters');
    this.categoryRiskCharacters[row.coverageCategory] = checkedAdd(
      this.categoryRiskCharacters[row.coverageCategory],
      row.charCount,
      `category ${row.coverageCategory}`,
    );
    this.formatRiskCharacters[row.format] = checkedAdd(
      this.formatRiskCharacters[row.format],
      row.charCount,
      `format ${row.format} risk characters`,
    );
    const lane = row.storedLineSeg ? 'stored-line-lane' : 'fresh-candidate-lane';
    this.laneRiskCharacters[lane] = checkedAdd(
      this.laneRiskCharacters[lane],
      row.charCount,
      `${lane} risk characters`,
    );
    if (row.storedLineSeg) {
      this.storedRiskMass = checkedAdd(this.storedRiskMass, risk.mass, 'stored risk mass');
    } else {
      this.freshCandidateRiskMass = checkedAdd(
        this.freshCandidateRiskMass,
        risk.mass,
        'fresh candidate risk mass',
      );
    }
    this.baseRiskMass = checkedAdd(this.baseRiskMass, risk.mass, 'base risk mass');
  }

  addRisk(target, row, risk, label) {
    target.riskCharacters = checkedAdd(target.riskCharacters, row.charCount, `${label} risk`);
    target.categoryRiskCharacters[row.coverageCategory] = checkedAdd(
      target.categoryRiskCharacters[row.coverageCategory],
      row.charCount,
      `${label} category`,
    );
    if (risk.compressed && risk.fixedFrameContextProxy) {
      target.compressedFixedContextRiskCharacters = checkedAdd(
        target.compressedFixedContextRiskCharacters,
        row.charCount,
        `${label} compressed fixed context`,
      );
    }
    if (row.storedLineSeg) {
      target.storedRiskMass = checkedAdd(target.storedRiskMass, risk.mass, `${label} stored mass`);
    } else {
      target.freshCandidateRiskMass = checkedAdd(
        target.freshCandidateRiskMass,
        risk.mass,
        `${label} fresh mass`,
      );
    }
    target.baseRiskMass = checkedAdd(target.baseRiskMass, risk.mass, `${label} base mass`);
    target.riskProfile.formatCharacters[row.format] = checkedAdd(
      target.riskProfile.formatCharacters[row.format],
      row.charCount,
      `${label} risk format`,
    );
    addMapCount(
      target.riskProfile.languageCharacters,
      String(row.language),
      row.charCount,
      `${label} risk language`,
    );
    if (row.bold) {
      target.riskProfile.boldCharacters = checkedAdd(
        target.riskProfile.boldCharacters,
        row.charCount,
        `${label} bold risk`,
      );
    }
    if (row.italic) {
      target.riskProfile.italicCharacters = checkedAdd(
        target.riskProfile.italicCharacters,
        row.charCount,
        `${label} italic risk`,
      );
    }
    if (row.kerning) {
      target.riskProfile.kerningCharacters = checkedAdd(
        target.riskProfile.kerningCharacters,
        row.charCount,
        `${label} kerning risk`,
      );
    }
  }
}

function compareCandidates(left, right) {
  return right.baseRiskMass - left.baseRiskMass
    || right.compressedFixedContextRiskCharacters
      - left.compressedFixedContextRiskCharacters
    || right.riskCharacters - left.riskCharacters
    || right.totalUsageCharacters - left.totalUsageCharacters
    || compareText(left.name ?? '', right.name ?? '');
}

function publicDocumentFace(entry, rank) {
  return {
    rank,
    documentFace: entry.name,
    totalUsageCharacters: entry.totalUsageCharacters,
    riskCharacters: entry.riskCharacters,
    categoryRiskCharacters: entry.categoryRiskCharacters,
    compressedFixedContextRiskCharacters: entry.compressedFixedContextRiskCharacters,
    storedRiskMass: entry.storedRiskMass,
    freshCandidateRiskMass: entry.freshCandidateRiskMass,
    baseRiskMass: entry.baseRiskMass,
    formatCharacters: entry.formatCharacters,
  };
}

function publicMetricCluster(entry, rank) {
  return {
    rank,
    metricRequestedFace: entry.name,
    documentFaceCount: entry.documentFaces.size,
    riskCharacters: entry.riskCharacters,
    categoryRiskCharacters: entry.categoryRiskCharacters,
    compressedFixedContextRiskCharacters: entry.compressedFixedContextRiskCharacters,
    storedRiskMass: entry.storedRiskMass,
    freshCandidateRiskMass: entry.freshCandidateRiskMass,
    baseRiskMass: entry.baseRiskMass,
    formatCharacters: entry.formatCharacters,
  };
}

function publicRiskProfile(entry, field) {
  return {
    [field]: entry.name,
    formatCharacters: entry.riskProfile.formatCharacters,
    languageCharacters: Object.fromEntries(
      [...entry.riskProfile.languageCharacters.entries()].sort(([left], [right]) => (
        compareText(left, right)
      )),
    ),
    boldCharacters: entry.riskProfile.boldCharacters,
    italicCharacters: entry.riskProfile.italicCharacters,
    kerningCharacters: entry.riskProfile.kerningCharacters,
  };
}

function sumValues(value) {
  return Object.values(value).reduce((total, count) => checkedAdd(total, count, 'count sum'), 0);
}

function finalizeRiskAccumulator(accumulator, aggregate, contract, inputIdentity = undefined) {
  const riskCategories = contract.compatibilityProjection.riskCategories;
  const expectedRisk = riskCategories.reduce(
    (total, category) => checkedAdd(
      total,
      aggregate.categories?.[category],
      `aggregate category ${category}`,
    ),
    0,
  );
  if (accumulator.rowCount !== aggregate.counts?.decisionUsageRows) {
    throw new Error('decisionUsage row count does not match aggregate counts');
  }
  if (accumulator.totalUsageCharacters !== aggregate.joins?.joined) {
    throw new Error('decisionUsage character sum does not match joined characters');
  }
  for (const category of riskCategories) {
    if (accumulator.categoryRiskCharacters[category] !== aggregate.categories[category]) {
      throw new Error(`risk category ${category} does not reconcile`);
    }
  }
  if (accumulator.riskCharacters !== expectedRisk) {
    throw new Error('risk category total does not reconcile');
  }
  if (accumulator.storedRiskMass + accumulator.freshCandidateRiskMass
      !== accumulator.baseRiskMass) {
    throw new Error('LineSeg lane risk masses do not reconcile');
  }

  const documentEntries = [...accumulator.documentFaces.values()]
    .filter(entry => entry.riskCharacters > 0)
    .sort(compareCandidates);
  const clusterEntries = [...accumulator.metricClusters.values()].sort(compareCandidates);
  const documentFaces = documentEntries.map((entry, index) => publicDocumentFace(entry, index + 1));
  const metricRequestClusters = clusterEntries
    .map((entry, index) => publicMetricCluster(entry, index + 1));
  const documentRiskSum = documentFaces.reduce(
    (total, entry) => checkedAdd(total, entry.riskCharacters, 'document face risk sum'),
    0,
  );
  const clusterRiskSum = metricRequestClusters.reduce(
    (total, entry) => checkedAdd(total, entry.riskCharacters, 'metric cluster risk sum'),
    0,
  );
  if (documentRiskSum !== expectedRisk || clusterRiskSum !== expectedRisk) {
    throw new Error('candidate risk character sums do not reconcile');
  }
  if (sumValues(accumulator.formatRiskCharacters) !== expectedRisk
      || sumValues(accumulator.laneRiskCharacters) !== expectedRisk
      || sumValues(accumulator.categoryRiskCharacters) !== expectedRisk) {
    throw new Error('format, category or LineSeg risk character sums do not reconcile');
  }

  const result = {
    schemaVersion: 1,
    kind: 'font-typesetting-risk-ranking',
    issue: 4962,
    ...(inputIdentity === undefined ? {} : { input: inputIdentity }),
    totals: {
      totalUsageCharacters: accumulator.totalUsageCharacters,
      riskCharacters: accumulator.riskCharacters,
      storedRiskMass: accumulator.storedRiskMass,
      freshCandidateRiskMass: accumulator.freshCandidateRiskMass,
      baseRiskMass: accumulator.baseRiskMass,
    },
    reconciliation: {
      decisionUsageRows: accumulator.rowCount,
      inputJoinedCharacters: aggregate.joins.joined,
      inputRiskCharacters: expectedRisk,
      documentFaceRiskCharacters: documentRiskSum,
      metricRequestClusterRiskCharacters: clusterRiskSum,
      categoryRiskCharacters: accumulator.categoryRiskCharacters,
      formatRiskCharacters: accumulator.formatRiskCharacters,
      lineSegLaneRiskCharacters: accumulator.laneRiskCharacters,
      compressedFixedContextRiskCharacters: documentFaces.reduce(
        (total, entry) => checkedAdd(
          total,
          entry.compressedFixedContextRiskCharacters,
          'compressed fixed context risk sum',
        ),
        0,
      ),
      observedDocumentFaces: accumulator.documentFaces.size,
      rankedDocumentFaces: documentFaces.length,
      namedMetricRequestClusters: metricRequestClusters
        .filter(entry => entry.metricRequestedFace !== null).length,
      unavailableMetricRequestClusters: metricRequestClusters
        .filter(entry => entry.metricRequestedFace === null).length,
      exact: true,
    },
    documentFaces,
    metricRequestClusters,
    documentFaceRiskProfiles: documentEntries.map(entry => (
      publicRiskProfile(entry, 'documentFace')
    )),
    metricRequestClusterRiskProfiles: clusterEntries.map(entry => (
      publicRiskProfile(entry, 'metricRequestedFace')
    )),
  };
  const privacyFindings = findSensitiveTypesettingRiskValues(result, contract);
  if (privacyFindings.length > 0) {
    throw new Error(`risk ranking failed privacy validation: ${privacyFindings[0].reason}`);
  }
  return {
    ...result,
    outputHash: {
      algorithm: 'sha256',
      value: sha256Text(canonicalJson(result)),
    },
  };
}

export function rankTypesettingRiskAggregate(aggregate, contract, options = {}) {
  const contractErrors = validateTypesettingRiskContract(contract);
  if (contractErrors.length > 0) {
    throw new Error(`invalid typesetting risk contract: ${contractErrors.join('; ')}`);
  }
  if (!isObject(aggregate) || !Array.isArray(aggregate.decisionUsage)) {
    throw new Error('coverage aggregate decisionUsage must be an array');
  }
  const accumulator = new RiskAccumulator(contract);
  for (const row of aggregate.decisionUsage) accumulator.addRow(row);
  return finalizeRiskAccumulator(accumulator, aggregate, contract, options.inputIdentity);
}

async function readCoverageHeader(inputPath) {
  const stream = fs.createReadStream(inputPath);
  const decoder = new StringDecoder('utf8');
  let prefix = '';
  for await (const chunk of stream) {
    prefix += decoder.write(chunk);
    const markerIndex = prefix.indexOf(HEADER_MARKER);
    if (markerIndex !== -1) {
      stream.destroy();
      return JSON.parse(`${prefix.slice(0, markerIndex)}}`);
    }
    if (Buffer.byteLength(prefix, 'utf8') > MAX_HEADER_BYTES) {
      stream.destroy();
      throw new Error('coverage aggregate header exceeds the bounded prefix');
    }
  }
  prefix += decoder.end();
  throw new Error('coverage aggregate legacyUsage marker is missing');
}

function consumeDecisionText(text, state, onRow) {
  if (state.arrayDone) {
    state.suffix += text;
    return;
  }
  for (let index = 0; index < text.length; index += 1) {
    const character = text[index];
    if (!state.inObject) {
      if (/\s/u.test(character) || character === ',') continue;
      if (character === ']') {
        state.arrayDone = true;
        state.suffix += text.slice(index + 1);
        return;
      }
      if (character !== '{') throw new Error('decisionUsage contains a non-object row');
      state.inObject = true;
      state.depth = 1;
      state.row = '{';
      continue;
    }
    state.row += character;
    if (Buffer.byteLength(state.row, 'utf8') > MAX_ROW_BYTES) {
      throw new Error('decisionUsage row exceeds the bounded row size');
    }
    if (state.inString) {
      if (state.escape) {
        state.escape = false;
      } else if (character === '\\') {
        state.escape = true;
      } else if (character === '"') {
        state.inString = false;
      }
      continue;
    }
    if (character === '"') {
      state.inString = true;
    } else if (character === '{' || character === '[') {
      state.depth += 1;
    } else if (character === '}' || character === ']') {
      state.depth -= 1;
      if (state.depth === 0) {
        onRow(JSON.parse(state.row));
        state.inObject = false;
        state.row = '';
      }
    }
  }
}

async function streamDecisionUsage(inputPath, onRow) {
  const hash = createHash('sha256');
  const decoder = new StringDecoder('utf8');
  const state = {
    markerFound: false,
    search: '',
    inObject: false,
    inString: false,
    escape: false,
    depth: 0,
    row: '',
    arrayDone: false,
    suffix: '',
  };
  for await (const chunk of fs.createReadStream(inputPath)) {
    hash.update(chunk);
    const text = decoder.write(chunk);
    if (!state.markerFound) {
      state.search += text;
      const markerIndex = state.search.indexOf(DECISION_MARKER);
      if (markerIndex === -1) {
        state.search = state.search.slice(-(DECISION_MARKER.length - 1));
        continue;
      }
      state.markerFound = true;
      const remainder = state.search.slice(markerIndex + DECISION_MARKER.length);
      state.search = '';
      consumeDecisionText(remainder, state, onRow);
    } else {
      consumeDecisionText(text, state, onRow);
    }
  }
  const tail = decoder.end();
  if (tail.length > 0) consumeDecisionText(tail, state, onRow);
  if (!state.markerFound) throw new Error('coverage aggregate decisionUsage marker is missing');
  if (!state.arrayDone || state.inObject || state.inString || state.depth !== 0) {
    throw new Error('coverage aggregate decisionUsage is incomplete');
  }
  if (!/^\}\s*$/u.test(state.suffix)) {
    throw new Error('coverage aggregate has unexpected data after decisionUsage');
  }
  return hash.digest('hex');
}

function validateCoverageHeader(header) {
  if (header?.schemaVersion !== 1
      || header.kind !== 'font-metric-coverage-aggregate'
      || header.status !== 'complete'
      || header.format !== 'mixed') {
    throw new Error('coverage aggregate header is not a complete mixed W3 aggregate');
  }
  if (!isObject(header.counts)
      || !isObject(header.categories)
      || !isObject(header.joins)
      || !isObject(header.documents)
      || !isObject(header.aggregateHash)
      || !isObject(header.checkpoint?.identity)) {
    throw new Error('coverage aggregate header is incomplete');
  }
}

function observedPrimary(inputPath, header, fileSha256) {
  const stat = fs.statSync(inputPath);
  return {
    mode: (stat.mode & 0o777).toString(8).padStart(4, '0'),
    bytes: stat.size,
    fileSha256,
    aggregateSha256: header.aggregateHash.value,
    sourceCommit: header.checkpoint.identity.sourceHead,
  };
}

function validatePrimaryObservation(observed, expected) {
  const errors = [];
  compareObservedFrozen('primary', observed, expected, errors);
  return errors;
}

export async function rankTypesettingRiskFile(inputPath, contract, options = {}) {
  const contractErrors = validateTypesettingRiskContract(contract);
  if (contractErrors.length > 0) {
    throw new Error(`invalid typesetting risk contract: ${contractErrors.join('; ')}`);
  }
  const header = await readCoverageHeader(inputPath);
  validateCoverageHeader(header);
  const accumulator = new RiskAccumulator(contract);
  const fileSha256 = await streamDecisionUsage(inputPath, row => accumulator.addRow(row));
  const observed = observedPrimary(inputPath, header, fileSha256);
  if (options.enforceFrozenInput !== false) {
    const inputErrors = validatePrimaryObservation(observed, contract.inputPreconditions.primary);
    if (inputErrors.length > 0) {
      throw new Error(`frozen W3 input precondition failed: ${inputErrors.join('; ')}`);
    }
  }
  const expected = contract.inputPreconditions.expectedW3;
  if (options.enforceFrozenInput !== false) {
    const failures = Object.values(header.documents.failures ?? {})
      .reduce((total, value) => checkedAdd(total, value, 'document failure sum'), 0);
    const checks = [
      [header.documents.attempted, expected.attemptedDocuments, 'attempted documents'],
      [header.documents.success, expected.successfulDocuments, 'successful documents'],
      [failures, expected.failedDocuments, 'failed documents'],
      [header.counts.layoutCharacters, expected.layoutCharacters, 'layout characters'],
      [header.counts.coverageCharacters, expected.coverageCharacters, 'coverage characters'],
      [header.counts.decisionUsageRows, expected.decisionUsageRows, 'decision usage rows'],
    ];
    for (const [actual, wanted, label] of checks) {
      if (actual !== wanted) throw new Error(`W3 ${label} precondition has drifted`);
    }
  }
  const result = finalizeRiskAccumulator(accumulator, header, contract, {
    sourceCommit: observed.sourceCommit,
    aggregateSha256: observed.aggregateSha256,
    fileSha256: observed.fileSha256,
    bytes: observed.bytes,
    mode: observed.mode,
  });
  if (options.enforceFrozenInput !== false
      && result.totals.riskCharacters !== expected.riskCharacters) {
    throw new Error('W3 risk character precondition has drifted');
  }
  return result;
}

function parseArguments(arguments_) {
  const values = { contract: DEFAULT_CONTRACT_PATH };
  const allowed = new Set(['--input', '--output', '--contract']);
  for (let index = 0; index < arguments_.length; index += 2) {
    const option = arguments_[index];
    const value = arguments_[index + 1];
    if (!allowed.has(option) || value === undefined) {
      throw new Error('usage: font typesetting risk rank --input file --output file [--contract file]');
    }
    values[option.slice(2)] = value;
  }
  if (!values.input || !values.output) {
    throw new Error('font typesetting risk rank requires input and output');
  }
  return values;
}

function assertInputPath(inputPath, contract) {
  const expected = path.resolve(ROOT, contract.inputPreconditions.primary.artifact);
  const actual = path.resolve(inputPath);
  if (actual !== expected) throw new Error('input must be the frozen W3 primary artifact');
  const stat = fs.lstatSync(actual);
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error('input must be a regular non-symlink file');
  }
  return actual;
}

function assertRiskOutputPath(outputPath, contract) {
  const root = path.resolve(ROOT, contract.privacy.localOutputDirectory);
  const actual = path.resolve(outputPath);
  if (actual === root || !actual.startsWith(`${root}${path.sep}`)) {
    throw new Error('output must be a file below the W4 local-only output directory');
  }
  fs.mkdirSync(path.dirname(actual), { recursive: true, mode: 0o700 });
  fs.chmodSync(path.dirname(actual), 0o700);
  return actual;
}

function writeNewPrivateJson(outputPath, value) {
  const descriptor = fs.openSync(outputPath, 'wx', 0o600);
  try {
    fs.writeFileSync(descriptor, `${JSON.stringify(value)}\n`);
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
  fs.chmodSync(outputPath, 0o600);
}

async function main() {
  const arguments_ = parseArguments(process.argv.slice(2));
  const contract = JSON.parse(fs.readFileSync(arguments_.contract, 'utf8'));
  const inputPath = assertInputPath(arguments_.input, contract);
  const outputPath = assertRiskOutputPath(arguments_.output, contract);
  const result = await rankTypesettingRiskFile(inputPath, contract);
  writeNewPrivateJson(outputPath, result);
  process.stdout.write(`${JSON.stringify({
    status: 'complete',
    outputHash: result.outputHash,
    totals: result.totals,
    documentFaceCount: result.documentFaces.length,
    metricRequestClusterCount: result.metricRequestClusters.length,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  main().catch(error => {
    process.stderr.write(`font typesetting risk rank failed: ${error.message}\n`);
    process.exitCode = 1;
  });
}
