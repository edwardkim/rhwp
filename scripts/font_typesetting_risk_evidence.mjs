#!/usr/bin/env node

import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  findSensitiveTypesettingRiskValues,
  streamDecisionUsage,
  validateDecisionRow,
  validateTypesettingRiskContract,
} from './font_typesetting_risk_rank.mjs';

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
const VARIANTS = [
  'base',
  'unweighted',
  'frame-neutral',
  'non-extreme',
  'stored-line-lane',
  'fresh-candidate-lane',
];

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

function checkedAdd(left, right, label) {
  const value = left + right;
  if (!Number.isSafeInteger(left) || left < 0
      || !Number.isSafeInteger(right) || right < 0
      || !Number.isSafeInteger(value)) {
    throw new Error(`${label} exceeds the non-negative safe integer range`);
  }
  return value;
}

function parseTsvLine(line) {
  return line.split('\t');
}

export function parseSupplySurveyTsv(text) {
  if (typeof text !== 'string') throw new Error('supply survey must be UTF-8 text');
  const lines = text.replace(/\r\n/gu, '\n').split('\n').filter(line => line.length > 0);
  if (lines.length === 0) throw new Error('supply survey is empty');
  const header = parseTsvLine(lines[0]);
  const expected = [
    'font',
    'search_name',
    'document_count',
    'status',
    'download_available',
    'webfont_usable',
    'webfont_usable_reason',
    'delivery',
    'package',
    'version',
    'license',
    'download_url',
    'note',
  ];
  if (canonicalJson(header) !== canonicalJson(expected)) {
    throw new Error('supply survey header has drifted');
  }
  const faces = new Set();
  return lines.slice(1).map((line, index) => {
    const fields = parseTsvLine(line);
    if (fields.length !== expected.length) {
      throw new Error(`supply survey row ${index + 2} has an invalid field count`);
    }
    const record = Object.fromEntries(expected.map((key, field) => [key, fields[field]]));
    if (record.font.length === 0 || faces.has(record.font)) {
      throw new Error(`supply survey row ${index + 2} has an empty or duplicate font`);
    }
    faces.add(record.font);
    return record;
  });
}

function contextIsFixed(row, contract) {
  const tokens = new Set(row.context.split('+'));
  return contract.editingAxes.fixedFrameContextProxy.tokens.some(token => tokens.has(token));
}

function riskVariantMasses(row, contract) {
  if (!contract.compatibilityProjection.riskCategories.includes(row.coverageCategory)) return null;
  const frameFactor = contextIsFixed(row, contract)
    ? contract.riskMass.fixedFrameProxyFactor
    : contract.riskMass.otherContextFactor;
  const compressionFactor = 1
    + Number(row.ratio < 100)
    + Number(row.ratio <= 90)
    + Number(row.spacing < 0)
    + Number(row.spacing <= -5);
  const nonExtremeFactor = 1 + Number(row.ratio < 100) + Number(row.spacing < 0);
  const base = row.charCount * compressionFactor * frameFactor;
  const masses = {
    base,
    unweighted: row.charCount,
    'frame-neutral': row.charCount * compressionFactor,
    'non-extreme': row.charCount * nonExtremeFactor * frameFactor,
    'stored-line-lane': row.storedLineSeg ? base : 0,
    'fresh-candidate-lane': row.storedLineSeg ? 0 : base,
  };
  if (Object.values(masses).some(value => !Number.isSafeInteger(value) || value < 0)) {
    throw new Error('sensitivity mass exceeds the non-negative safe integer range');
  }
  return masses;
}

function createVariantAccumulator(ranking, contract) {
  if (ranking?.kind !== 'font-typesetting-risk-ranking'
      || !Array.isArray(ranking.documentFaces)) {
    throw new Error('W4-2 base ranking is invalid');
  }
  const entries = new Map(ranking.documentFaces.map(entry => [entry.documentFace, {
    baseRank: entry.rank,
    baseRiskMass: entry.baseRiskMass,
    riskCharacters: entry.riskCharacters,
    masses: Object.fromEntries(VARIANTS.map(variant => [variant, 0])),
  }]));
  return {
    rowIndex: 0,
    entries,
    totals: Object.fromEntries(VARIANTS.map(variant => [variant, 0])),
    add(row) {
      validateDecisionRow(row, this.rowIndex, contract);
      this.rowIndex += 1;
      const masses = riskVariantMasses(row, contract);
      if (!masses) return;
      const entry = this.entries.get(row.font);
      if (!entry) throw new Error(`risk row face is missing from W4-2 ranking: ${row.font}`);
      for (const variant of VARIANTS) {
        entry.masses[variant] = checkedAdd(
          entry.masses[variant],
          masses[variant],
          `${variant} face mass`,
        );
        this.totals[variant] = checkedAdd(
          this.totals[variant],
          masses[variant],
          `${variant} total mass`,
        );
      }
    },
  };
}

function bandForShare(share, bands) {
  return bands.find(entry => share < entry.upperExclusiveBeforeShare)?.name
    ?? bands[bands.length - 1].name;
}

function sensitivityProjection(accumulator, contract) {
  if (accumulator.totals.base !== accumulator.totals['stored-line-lane']
      + accumulator.totals['fresh-candidate-lane']) {
    throw new Error('sensitivity LineSeg lane masses do not reconcile');
  }
  for (const entry of accumulator.entries.values()) {
    if (entry.masses.base !== entry.baseRiskMass
        || entry.masses.unweighted !== entry.riskCharacters) {
      throw new Error('sensitivity projection does not match the W4-2 base ranking');
    }
  }
  const byFace = new Map([...accumulator.entries.keys()].map(face => [face, {
    ranks: {},
    bands: {},
  }]));
  const variants = [];
  for (const variant of VARIANTS) {
    const sorted = [...accumulator.entries.entries()].sort((left, right) => (
      right[1].masses[variant] - left[1].masses[variant]
      || left[1].baseRank - right[1].baseRank
      || compareText(left[0], right[0])
    ));
    const totalMass = accumulator.totals[variant];
    let cumulative = 0;
    const bandCounts = Object.fromEntries(
      contract.evidenceAndStability.cumulativeRiskBands.map(entry => [entry.name, 0]),
    );
    sorted.forEach(([face, entry], index) => {
      const shareBefore = totalMass === 0 ? 1 : cumulative / totalMass;
      const band = bandForShare(
        shareBefore,
        contract.evidenceAndStability.cumulativeRiskBands,
      );
      byFace.get(face).ranks[variant] = index + 1;
      byFace.get(face).bands[variant] = band;
      bandCounts[band] += 1;
      cumulative = checkedAdd(cumulative, entry.masses[variant], `${variant} cumulative mass`);
    });
    if (cumulative !== totalMass) throw new Error(`${variant} cumulative mass does not reconcile`);
    variants.push({ name: variant, totalMass, bandCounts });
  }
  return { byFace, variants };
}

function backendProfile(rules, backend) {
  const matched = rules.filter(rule => (
    rule.decisionPlane === 'supply'
    && Array.isArray(rule.backends)
    && rule.backends.includes(backend)
  ));
  const available = matched.filter(rule => (
    rule.status === 'active'
    && !String(rule.targetFaceOrPolicy).startsWith('unavailable:')
  ));
  const unavailable = matched.filter(rule => (
    String(rule.targetFaceOrPolicy).startsWith('unavailable:')
  ));
  let availability = 'unknown';
  if (available.length > 0) availability = 'available';
  else if (unavailable.length > 0) availability = 'unavailable';
  return {
    availability,
    profiles: [...new Set(matched.map(rule => rule.conditions?.profile).filter(Boolean))]
      .sort(compareText),
    evidenceStatuses: [...new Set(matched.map(rule => rule.evidenceStatus).filter(Boolean))]
      .sort(compareText),
    ruleIds: matched.map(rule => rule.ruleId).sort(compareText),
  };
}

function backendDiverges(canvas2d, canvaskit) {
  if (canvas2d.ruleIds.length === 0 || canvaskit.ruleIds.length === 0) return false;
  return canvas2d.availability !== canvaskit.availability
    || canonicalJson(canvas2d.profiles) !== canonicalJson(canvaskit.profiles);
}

function publicSupply(row) {
  if (!row) return {
    status: 'unknown',
    historicalOnly: true,
  };
  return {
    status: row.status || 'unknown',
    downloadAvailable: row.download_available || 'unknown',
    webfontUsable: row.webfont_usable || 'unknown',
    delivery: row.delivery || null,
    package: row.package || null,
    version: row.version || null,
    license: row.license || null,
    historicalOnly: true,
  };
}

function exactSource(face, curated) {
  const verified = curated.exactSourceVerified.find(entry => entry.documentFace === face);
  if (verified) return {
    status: 'verified',
    fontSha256: verified.fontSha256,
    nameTableMatch: verified.nameTableMatch,
  };
  if (curated.exactSourceUnavailableFaces.includes(face)) return { status: 'unavailable' };
  if (curated.exactSourceAvailableFaces.includes(face)) return { status: 'available' };
  return { status: 'unknown' };
}

function evidenceForFace(face, ledgerRules, surveyRow, contract) {
  const curated = contract.evidenceAndStability.curatedEvidence;
  const rules = ledgerRules.filter(rule => rule.sourceFace === face);
  const canvas2d = backendProfile(rules, 'canvas2d');
  const canvaskit = backendProfile(rules, 'canvaskit');
  const sourceStatus = exactSource(face, curated);
  const government = curated.governmentOrLegalCoreFaces.includes(face);
  const divergence = backendDiverges(canvas2d, canvaskit);
  const unknownRelationRules = rules
    .filter(rule => rule.relationType === 'unknown')
    .map(rule => rule.ruleId)
    .sort(compareText);
  const ledgerRuleIds = rules.map(rule => rule.ruleId).sort(compareText);
  const anchors = [];
  if (ledgerRuleIds.length > 0) {
    anchors.push('mydocs/tech/investigations/issue-4939/font_rule_ledger.json');
  }
  if (government) {
    anchors.push(
      'mydocs/tech/investigations/issue-4739/task_m100_4739_government_font_successor_matrix.md',
    );
  }
  if (sourceStatus.status === 'verified') {
    anchors.push('mydocs/working/archives/task_m100_4739_stage5_validation.md');
  } else if (sourceStatus.status === 'available') {
    anchors.push('mydocs/working/archives/task_m100_4764_stage1_kopub_canvaskit_sfnt.md');
  }
  if (surveyRow) {
    anchors.push('mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.tsv');
  }
  return {
    exactSource: sourceStatus,
    evidenceFlags: {
      'government-or-legal-core': government,
      'exact-source-verified': sourceStatus.status === 'verified',
      'exact-source-available': sourceStatus.status === 'available',
      'backend-selection-divergence': divergence,
      'unknown-relation': unknownRelationRules.length > 0,
    },
    backendProfiles: { canvas2d, canvaskit },
    supply: publicSupply(surveyRow),
    ledgerRuleIds,
    unknownRelationRuleIds: unknownRelationRules,
    evidenceAnchors: [...new Set(anchors)].sort(compareText),
  };
}

function evidencePriority(entry) {
  return [
    Number(entry.evidenceFlags['exact-source-verified']),
    Number(entry.evidenceFlags['government-or-legal-core']),
    Number(entry.evidenceFlags['backend-selection-divergence']),
  ];
}

function compareAction(left, right) {
  const leftPriority = evidencePriority(left);
  const rightPriority = evidencePriority(right);
  for (let index = 0; index < leftPriority.length; index += 1) {
    if (leftPriority[index] !== rightPriority[index]) {
      return rightPriority[index] - leftPriority[index];
    }
  }
  return left.rank - right.rank;
}

function actionReasons(entry) {
  if (entry.actionRank === entry.rank) return [];
  const reasons = ['same-band:evidence-priority'];
  reasons.push(entry.actionRank < entry.rank ? 'direction:promoted' : 'direction:deferred');
  const priorityFlags = new Set([
    'exact-source-verified',
    'government-or-legal-core',
    'backend-selection-divergence',
  ]);
  reasons.push(...Object.entries(entry.evidenceFlags)
    .filter(([flag, enabled]) => enabled && priorityFlags.has(flag))
    .map(([flag]) => `evidence:${flag}`));
  if (reasons.length === 2) reasons.push('evidence:peer-priority');
  return [...new Set(reasons)].sort(compareText);
}

function evidenceGateCounts(entries) {
  let unsupportedPromotions = 0;
  for (const entry of entries) {
    const flags = entry.evidenceFlags;
    if (flags['government-or-legal-core']
        && !entry.evidenceAnchors.some(anchor => anchor.includes('government_font'))) {
      unsupportedPromotions += 1;
    }
    if (flags['exact-source-verified'] && !entry.exactSource.fontSha256) {
      unsupportedPromotions += 1;
    }
    if (flags['exact-source-available']
        && !entry.evidenceAnchors.some(anchor => anchor.includes('kopub_canvaskit'))) {
      unsupportedPromotions += 1;
    }
    if (flags['backend-selection-divergence']
        && (entry.backendProfiles.canvas2d.ruleIds.length === 0
          || entry.backendProfiles.canvaskit.ruleIds.length === 0)) {
      unsupportedPromotions += 1;
    }
    if (flags['unknown-relation'] && entry.unknownRelationRuleIds.length === 0) {
      unsupportedPromotions += 1;
    }
  }
  return unsupportedPromotions;
}

function finalizeEvidenceRanking(ranking, accumulator, ledger, surveyRows, contract) {
  if (!isObject(ledger) || !Array.isArray(ledger.rules)) {
    throw new Error('W1 font rule ledger is invalid');
  }
  const supplyByFace = new Map(surveyRows.map(row => [row.font, row]));
  if (supplyByFace.size !== surveyRows.length) throw new Error('supply survey faces are not unique');
  const stability = sensitivityProjection(accumulator, contract);
  const entries = ranking.documentFaces.map(base => {
    const variant = accumulator.entries.get(base.documentFace);
    const sensitivity = stability.byFace.get(base.documentFace);
    const evidence = evidenceForFace(
      base.documentFace,
      ledger.rules,
      supplyByFace.get(base.documentFace),
      contract,
    );
    const observedBands = [...new Set(Object.values(sensitivity.bands))].sort(compareText);
    const ranks = Object.values(sensitivity.ranks);
    return {
      ...base,
      empiricalRiskBand: sensitivity.bands.base,
      variantMasses: Object.fromEntries(
        VARIANTS.filter(name => name !== 'base').map(name => [name, variant.masses[name]]),
      ),
      stability: {
        ranks: sensitivity.ranks,
        bands: sensitivity.bands,
        rankRange: { min: Math.min(...ranks), max: Math.max(...ranks) },
        observedBands,
      },
      ...evidence,
    };
  });
  let nextActionRank = 1;
  for (const band of contract.evidenceAndStability.cumulativeRiskBands) {
    const group = entries.filter(entry => entry.empiricalRiskBand === band.name).sort(compareAction);
    for (const entry of group) {
      entry.actionRank = nextActionRank;
      nextActionRank += 1;
    }
  }
  for (const entry of entries) entry.actionRankReasons = actionReasons(entry);
  const bandRankRanges = new Map(contract.evidenceAndStability.cumulativeRiskBands.map(band => {
    const ranks = entries
      .filter(entry => entry.empiricalRiskBand === band.name)
      .map(entry => entry.rank);
    return [band.name, {
      min: Math.min(...ranks),
      max: Math.max(...ranks),
    }];
  }));
  const crossBandPromotions = entries.filter(entry => {
    const range = bandRankRanges.get(entry.empiricalRiskBand);
    return entry.actionRank < range.min || entry.actionRank > range.max;
  }).length;
  const rankInvariantFaces = entries.filter(entry => (
    entry.stability.rankRange.min === entry.stability.rankRange.max
  )).length;
  const singleBandFaces = entries.filter(entry => entry.stability.observedBands.length === 1).length;
  const globalVariants = ['base', 'unweighted', 'frame-neutral', 'non-extreme'];
  const globalWeightBandStableFaces = entries.filter(entry => (
    new Set(globalVariants.map(name => entry.stability.bands[name])).size === 1
  )).length;
  const result = {
    schemaVersion: 1,
    kind: 'font-typesetting-risk-evidence-ranking',
    issue: 4962,
    input: {
      baseRankingOutputSha256: ranking.outputHash?.value ?? null,
      evidenceInputs: contract.evidenceAndStability.evidenceInputs.map(entry => ({
        artifact: entry.artifact,
        sha256: entry.sha256,
        role: entry.role,
      })),
    },
    totals: ranking.totals,
    stability: {
      cumulativeRiskBands: contract.evidenceAndStability.cumulativeRiskBands,
      variants: stability.variants,
      rankInvariantFaces,
      singleBandFaces,
      globalWeightBandStableFaces,
      maxRankSpan: Math.max(...entries.map(entry => (
        entry.stability.rankRange.max - entry.stability.rankRange.min
      ))),
    },
    evidenceJoin: {
      rankedDocumentFaces: entries.length,
      ledgerJoinedFaces: entries.filter(entry => entry.ledgerRuleIds.length > 0).length,
      supplyJoinedFaces: entries.filter(entry => entry.supply.status !== 'unknown').length,
      exactSourceVerifiedFaces: entries
        .filter(entry => entry.exactSource.status === 'verified').length,
      exactSourceAvailableFaces: entries
        .filter(entry => entry.exactSource.status === 'available').length,
      exactSourceUnavailableFaces: entries
        .filter(entry => entry.exactSource.status === 'unavailable').length,
      backendSelectionDivergenceFaces: entries
        .filter(entry => entry.evidenceFlags['backend-selection-divergence']).length,
      unknownRelationFaces: entries
        .filter(entry => entry.evidenceFlags['unknown-relation']).length,
    },
    gates: {
      unsupportedPromotions: evidenceGateCounts(entries),
      identityGuesses: 0,
      crossBandPromotions,
      baseRiskMassUnchanged: accumulator.totals.base === ranking.totals.baseRiskMass,
    },
    documentFaces: entries,
  };
  if (!result.gates.baseRiskMassUnchanged
      || result.gates.unsupportedPromotions !== 0
      || result.gates.identityGuesses !== 0
      || result.gates.crossBandPromotions !== 0) {
    throw new Error('W4-3 evidence or stability gate failed');
  }
  const privacyFindings = findSensitiveTypesettingRiskValues(result, contract);
  if (privacyFindings.length > 0) {
    throw new Error(`W4-3 output failed privacy validation: ${privacyFindings[0].reason}`);
  }
  return {
    ...result,
    outputHash: {
      algorithm: 'sha256',
      value: sha256Text(canonicalJson(result)),
    },
  };
}

export function enrichTypesettingRiskRanking({
  ranking,
  decisionRows,
  ledger,
  surveyRows,
  contract,
}) {
  const errors = validateTypesettingRiskContract(contract);
  if (errors.length > 0) throw new Error(`invalid typesetting risk contract: ${errors.join('; ')}`);
  if (ranking.input
      && ranking.outputHash?.value !== contract.evidenceAndStability.baseRankingOutputSha256) {
    throw new Error('W4-2 base ranking output hash has drifted');
  }
  const accumulator = createVariantAccumulator(ranking, contract);
  for (const row of decisionRows) accumulator.add(row);
  return finalizeEvidenceRanking(ranking, accumulator, ledger, surveyRows, contract);
}

function sha256File(file) {
  const hash = createHash('sha256');
  const descriptor = fs.openSync(file, 'r');
  try {
    const buffer = Buffer.allocUnsafe(1024 * 1024);
    let bytes;
    do {
      bytes = fs.readSync(descriptor, buffer, 0, buffer.length, null);
      if (bytes > 0) hash.update(buffer.subarray(0, bytes));
    } while (bytes > 0);
  } finally {
    fs.closeSync(descriptor);
  }
  return hash.digest('hex');
}

function loadEvidenceInputs(contract) {
  const byRole = new Map();
  for (const expected of contract.evidenceAndStability.evidenceInputs) {
    const artifact = path.resolve(ROOT, expected.artifact);
    if (sha256File(artifact) !== expected.sha256) {
      throw new Error(`evidence input hash has drifted: ${expected.role}`);
    }
    byRole.set(expected.role, artifact);
  }
  return {
    ledger: JSON.parse(fs.readFileSync(byRole.get('backend-and-relation-ledger'), 'utf8')),
    surveyRows: parseSupplySurveyTsv(
      fs.readFileSync(byRole.get('historical-supply-only'), 'utf8'),
    ),
  };
}

function parseArguments(values) {
  const parsed = { contract: DEFAULT_CONTRACT_PATH };
  const allowed = new Set(['--input', '--ranking', '--output', '--contract']);
  for (let index = 0; index < values.length; index += 2) {
    const option = values[index];
    const value = values[index + 1];
    if (!allowed.has(option) || value === undefined) {
      throw new Error('usage: W4-3 evidence --input file --ranking file --output file [--contract file]');
    }
    parsed[option.slice(2)] = value;
  }
  if (!parsed.input || !parsed.ranking || !parsed.output) {
    throw new Error('W4-3 evidence requires input, ranking and output');
  }
  return parsed;
}

function assertFrozenInput(input, contract) {
  const actual = path.resolve(input);
  const expected = path.resolve(ROOT, contract.inputPreconditions.primary.artifact);
  if (actual !== expected) throw new Error('input must be the frozen W3 primary artifact');
  const stat = fs.lstatSync(actual);
  const mode = (stat.mode & 0o777).toString(8).padStart(4, '0');
  if (!stat.isFile() || stat.isSymbolicLink()
      || stat.size !== contract.inputPreconditions.primary.bytes
      || mode !== contract.inputPreconditions.primary.requiredMode) {
    throw new Error('frozen W3 primary file identity has drifted');
  }
  return actual;
}

function assertLocalOutput(output, contract) {
  const root = path.resolve(ROOT, contract.privacy.localOutputDirectory);
  const actual = path.resolve(output);
  if (actual === root || !actual.startsWith(`${root}${path.sep}`)) {
    throw new Error('output must remain below the W4 local-only output directory');
  }
  fs.mkdirSync(path.dirname(actual), { recursive: true, mode: 0o700 });
  fs.chmodSync(path.dirname(actual), 0o700);
  return actual;
}

function writeNewPrivateJson(output, value) {
  const descriptor = fs.openSync(output, 'wx', 0o600);
  try {
    fs.writeFileSync(descriptor, `${JSON.stringify(value)}\n`);
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
  fs.chmodSync(output, 0o600);
}

async function main() {
  const arguments_ = parseArguments(process.argv.slice(2));
  const contract = JSON.parse(fs.readFileSync(arguments_.contract, 'utf8'));
  const errors = validateTypesettingRiskContract(contract);
  if (errors.length > 0) throw new Error(`invalid typesetting risk contract: ${errors.join('; ')}`);
  const input = assertFrozenInput(arguments_.input, contract);
  const ranking = JSON.parse(fs.readFileSync(arguments_.ranking, 'utf8'));
  if (ranking.outputHash?.value !== contract.evidenceAndStability.baseRankingOutputSha256) {
    throw new Error('W4-2 base ranking output hash has drifted');
  }
  const evidence = loadEvidenceInputs(contract);
  const accumulator = createVariantAccumulator(ranking, contract);
  const fileSha256 = await streamDecisionUsage(input, row => accumulator.add(row));
  if (fileSha256 !== contract.inputPreconditions.primary.fileSha256) {
    throw new Error('frozen W3 primary SHA-256 has drifted');
  }
  const result = finalizeEvidenceRanking(
    ranking,
    accumulator,
    evidence.ledger,
    evidence.surveyRows,
    contract,
  );
  const output = assertLocalOutput(arguments_.output, contract);
  writeNewPrivateJson(output, result);
  process.stdout.write(`${JSON.stringify({
    status: 'complete',
    outputHash: result.outputHash,
    evidenceJoin: result.evidenceJoin,
    gates: result.gates,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  main().catch(error => {
    process.stderr.write(`W4-3 evidence ranking failed: ${error.message}\n`);
    process.exitCode = 1;
  });
}
