#!/usr/bin/env node

import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  findSensitiveTypesettingRiskValues,
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

export function canonicalEvidenceRankingSha256(value) {
  const { outputHash: _outputHash, ...body } = value;
  return sha256Text(canonicalJson(body));
}

function checkedSum(entries, field) {
  return entries.reduce((total, entry) => {
    const value = entry[field];
    if (!Number.isSafeInteger(value) || value < 0 || !Number.isSafeInteger(total + value)) {
      throw new Error(`${field} sum exceeds the non-negative safe integer range`);
    }
    return total + value;
  }, 0);
}

function ppm(numerator, denominator, scale) {
  if (!Number.isSafeInteger(numerator) || numerator < 0
      || !Number.isSafeInteger(denominator) || denominator <= 0) {
    throw new Error('rate numerator or denominator is invalid');
  }
  return Math.round(numerator * scale / denominator);
}

function validateEvidenceRanking(value, contract) {
  if (!isObject(value)
      || value.schemaVersion !== 1
      || value.kind !== contract.publicProjection.inputKind
      || value.issue !== 4962
      || !Array.isArray(value.documentFaces)
      || value.documentFaces.length === 0
      || value.outputHash?.algorithm !== 'sha256') {
    throw new Error('W4-3 evidence ranking envelope is invalid');
  }
  const computed = canonicalEvidenceRankingSha256(value);
  if (computed !== value.outputHash.value
      || computed !== contract.publicProjection.inputCanonicalSha256) {
    throw new Error('W4-3 evidence ranking canonical SHA-256 has drifted');
  }
  const gates = value.gates;
  if (gates?.unsupportedPromotions !== 0
      || gates.identityGuesses !== 0
      || gates.crossBandPromotions !== 0
      || gates.baseRiskMassUnchanged !== true) {
    throw new Error('W4-3 evidence ranking gates are not complete');
  }
  const ranks = value.documentFaces.map(entry => entry.rank);
  const actionRanks = value.documentFaces.map(entry => entry.actionRank);
  if (new Set(ranks).size !== ranks.length
      || new Set(actionRanks).size !== actionRanks.length
      || Math.min(...ranks) !== 1
      || Math.max(...ranks) !== ranks.length
      || Math.min(...actionRanks) !== 1
      || Math.max(...actionRanks) !== actionRanks.length) {
    throw new Error('W4-3 base or action ranks are not a complete permutation');
  }
}

function exactReadiness(status) {
  return {
    verified: 'bytes-verified',
    available: 'acquisition-required',
    unavailable: 'not-available',
    unknown: 'discovery-required',
  }[status] ?? 'discovery-required';
}

function candidateQuestions(entry, contract) {
  const face = entry.documentFace;
  const exact = exactReadiness(entry.exactSource.status);
  const successor = entry.evidenceFlags['government-or-legal-core']
    ? 'direct-anchor-present'
    : 'direct-anchor-required';
  const questions = [
    {
      id: 'exact-installed',
      readiness: exact,
      prompt: `${face} exact font의 bytes와 face index를 고정해 glyph outline, hmtx advance, 첫 조판 divergence는 무엇인가?`,
      relationOutcome: 'identity-exact-or-unknown',
    },
    {
      id: 'exact-removed',
      readiness: exact === 'bytes-verified' ? 'controlled-removal-ready' : exact,
      prompt: `${face} exact font만 제거했을 때 한컴 PDF가 선택하는 subset font와 첫 조판 divergence는 무엇인가?`,
      relationOutcome: 'hancom-missing-font-or-unknown',
    },
    {
      id: 'document-subst-font-only',
      readiness: 'fixture-required',
      prompt: `${face} exact font를 제거하고 문서 substFont만 제공할 때 선택 face와 metric·paint 관계는 무엇인가?`,
      relationOutcome: 'document-substitution-or-unknown',
    },
    {
      id: 'curated-official-successor-only',
      readiness: successor,
      prompt: `${face}의 직접 공개 anchor가 있는 official successor만 설치했을 때 identity가 아닌 successor 관계와 첫 divergence는 무엇인가?`,
      relationOutcome: 'official-successor-or-not-provided',
    },
    {
      id: 'all-related-fonts-missing',
      readiness: 'environment-reset-required',
      prompt: `${face}와 substFont·검증된 successor를 모두 미설치했을 때 한컴의 missing-font 선택과 backend별 재현 차이는 무엇인가?`,
      relationOutcome: 'hancom-missing-font-or-unknown',
    },
  ];
  if (canonicalJson(questions.map(entry_ => entry_.id))
      !== canonicalJson(contract.publicProjection.w5QuestionIds)) {
    throw new Error(`W5 question inventory drifted for ${face}`);
  }
  return questions;
}

function publicBackend(profile) {
  return {
    availability: profile.availability,
    profiles: profile.profiles,
    evidenceStatuses: profile.evidenceStatuses,
    ruleIds: profile.ruleIds,
  };
}

function publicRankingEntry(entry, totals, contract, queued) {
  const scale = contract.publicProjection.rateScale;
  return {
    baseRank: entry.rank,
    actionRank: entry.actionRank,
    documentFace: entry.documentFace,
    empiricalRiskBand: entry.empiricalRiskBand,
    w5Queue: queued,
    riskCharacters: entry.riskCharacters,
    riskRatePpm: ppm(entry.riskCharacters, totals.riskCharacters, scale),
    categoryRiskCharacters: entry.categoryRiskCharacters,
    baseRiskMass: entry.baseRiskMass,
    baseRiskMassPpm: ppm(entry.baseRiskMass, totals.baseRiskMass, scale),
    compressedFixedContextRiskCharacters: entry.compressedFixedContextRiskCharacters,
    storedRiskMass: entry.storedRiskMass,
    freshCandidateRiskMass: entry.freshCandidateRiskMass,
    formatCharacters: entry.formatCharacters,
    sensitivity: {
      rankRange: entry.stability.rankRange,
      ranks: entry.stability.ranks,
      bands: entry.stability.bands,
      observedBands: entry.stability.observedBands,
    },
    exactSource: entry.exactSource,
    evidenceFlags: entry.evidenceFlags,
    backendProfiles: {
      canvas2d: publicBackend(entry.backendProfiles.canvas2d),
      canvaskit: publicBackend(entry.backendProfiles.canvaskit),
    },
    supply: {
      status: entry.supply.status,
      downloadAvailable: entry.supply.downloadAvailable ?? 'unknown',
      webfontUsable: entry.supply.webfontUsable ?? 'unknown',
      historicalOnly: true,
    },
    ledgerRuleIds: entry.ledgerRuleIds,
    evidenceAnchors: entry.evidenceAnchors,
    actionRankReasons: entry.actionRankReasons,
  };
}

function queueEntry(entry, totals, contract) {
  return {
    actionRank: entry.actionRank,
    baseRank: entry.rank,
    documentFace: entry.documentFace,
    empiricalRiskBand: entry.empiricalRiskBand,
    whyNow: {
      riskCharacters: entry.riskCharacters,
      riskRatePpm: ppm(
        entry.riskCharacters,
        totals.riskCharacters,
        contract.publicProjection.rateScale,
      ),
      baseRiskMass: entry.baseRiskMass,
      baseRiskMassPpm: ppm(
        entry.baseRiskMass,
        totals.baseRiskMass,
        contract.publicProjection.rateScale,
      ),
      compressedFixedContextRiskCharacters: entry.compressedFixedContextRiskCharacters,
      storedRiskMass: entry.storedRiskMass,
      freshCandidateRiskMass: entry.freshCandidateRiskMass,
      exactSourceStatus: entry.exactSource.status,
      evidenceFlags: entry.evidenceFlags,
      canvas2dAvailability: entry.backendProfiles.canvas2d.availability,
      canvaskitAvailability: entry.backendProfiles.canvaskit.availability,
      supplyStatus: entry.supply.status,
    },
    requiredOracleProfileFields: contract.publicProjection.requiredOracleProfileFields,
    questions: candidateQuestions(entry, contract),
  };
}

export function buildPublicTypesettingRiskRanking(evidence, contract) {
  const contractErrors = validateTypesettingRiskContract(contract);
  if (contractErrors.length > 0) {
    throw new Error(`invalid typesetting risk contract: ${contractErrors.join('; ')}`);
  }
  validateEvidenceRanking(evidence, contract);
  const queueBands = new Set(contract.publicProjection.queueBands);
  const reserveBands = new Set(contract.publicProjection.reserveBands);
  const knownBands = new Set([...queueBands, ...reserveBands]);
  if (evidence.documentFaces.some(entry => !knownBands.has(entry.empiricalRiskBand))) {
    throw new Error('W4-3 ranking contains an unknown empirical band');
  }
  const queue = evidence.documentFaces
    .filter(entry => queueBands.has(entry.empiricalRiskBand))
    .sort((left, right) => left.actionRank - right.actionRank);
  const queueFaces = new Set(queue.map(entry => entry.documentFace));
  const ranking = evidence.documentFaces
    .slice()
    .sort((left, right) => left.rank - right.rank)
    .map(entry => publicRankingEntry(
      entry,
      evidence.totals,
      contract,
      queueFaces.has(entry.documentFace),
    ));
  const queueRiskCharacters = checkedSum(queue, 'riskCharacters');
  const queueBaseRiskMass = checkedSum(queue, 'baseRiskMass');
  const queueOutsideSelectedBands = queue.filter(entry => (
    !queueBands.has(entry.empiricalRiskBand)
  )).length;
  const result = {
    schemaVersion: 1,
    kind: 'font-typesetting-risk-public-ranking',
    issue: 4962,
    parentIssue: 4960,
    nextIssue: 4963,
    generatedFrom: {
      evidenceRankingCanonicalSha256: evidence.outputHash.value,
      w3AggregateSha256: contract.inputPreconditions.primary.aggregateSha256,
      w3ExecutionCommit: contract.inputPreconditions.primary.sourceCommit,
    },
    rate: {
      unit: contract.publicProjection.rateUnit,
      scale: contract.publicProjection.rateScale,
      rounding: 'nearest-integer',
    },
    totals: evidence.totals,
    selection: {
      policy: contract.publicProjection.selectionPolicy,
      queueBands: contract.publicProjection.queueBands,
      reserveBands: contract.publicProjection.reserveBands,
      queueFaceCount: queue.length,
      reserveFaceCount: evidence.documentFaces.length - queue.length,
      queueRiskCharacters,
      queueRiskCharactersPpm: ppm(
        queueRiskCharacters,
        evidence.totals.riskCharacters,
        contract.publicProjection.rateScale,
      ),
      queueBaseRiskMass,
      queueBaseRiskMassPpm: ppm(
        queueBaseRiskMass,
        evidence.totals.baseRiskMass,
        contract.publicProjection.rateScale,
      ),
    },
    stability: evidence.stability,
    evidenceJoin: evidence.evidenceJoin,
    gates: {
      baseRiskMassUnchanged: evidence.gates.baseRiskMassUnchanged,
      unsupportedPromotions: evidence.gates.unsupportedPromotions,
      identityGuesses: evidence.gates.identityGuesses,
      crossBandPromotions: evidence.gates.crossBandPromotions,
      queueOutsideSelectedBands,
      privateIdentityFindings: 0,
    },
    w5Handoff: {
      issue: 4963,
      productBehaviorChange: false,
      oracleProfileSchemaImplemented: false,
      controlledLadderStarted: false,
      requiredOracleProfileFields: contract.publicProjection.requiredOracleProfileFields,
      questionIds: contract.publicProjection.w5QuestionIds,
      queue: queue.map(entry => queueEntry(entry, evidence.totals, contract)),
    },
    githubHandoff: {
      writesPerformed: false,
      issue4960ChecklistCandidate: 'W3+W4 evidence complete; maintainer approval pending',
      issue4962CompletionCandidate: 'W3+W4 local deliverables complete; maintainer approval pending',
      issue4963InputCandidate: 'A+B action queue and five controlled-ladder questions prepared',
    },
    ranking,
  };
  if (result.gates.queueOutsideSelectedBands !== 0) {
    throw new Error('W5 queue crossed the selected empirical bands');
  }
  const privacyFindings = findSensitiveTypesettingRiskValues(result, contract);
  if (privacyFindings.length > 0) {
    throw new Error(`public ranking failed privacy validation: ${privacyFindings[0].reason}`);
  }
  return {
    ...result,
    outputHash: {
      algorithm: 'sha256',
      value: sha256Text(canonicalJson(result)),
    },
  };
}

function parseArguments(values) {
  const parsed = { contract: DEFAULT_CONTRACT_PATH };
  const allowed = new Set(['--input', '--output', '--contract']);
  for (let index = 0; index < values.length; index += 2) {
    const option = values[index];
    const value = values[index + 1];
    if (!allowed.has(option) || value === undefined) {
      throw new Error('usage: W4 publish --input evidence.json --output ranking.json [--contract file]');
    }
    parsed[option.slice(2)] = value;
  }
  if (!parsed.input || !parsed.output) {
    throw new Error('W4 publish requires input and output');
  }
  return parsed;
}

function assertInput(input, contract) {
  const actual = path.resolve(input);
  const localRoot = path.resolve(ROOT, contract.privacy.localOutputDirectory);
  if (!actual.startsWith(`${localRoot}${path.sep}`)) {
    throw new Error('W4-3 input must remain below the local-only output directory');
  }
  const stat = fs.lstatSync(actual);
  if (!stat.isFile() || stat.isSymbolicLink()
      || (stat.mode & 0o777).toString(8).padStart(4, '0') !== '0600') {
    throw new Error('W4-3 input must be a mode 0600 regular non-symlink file');
  }
  return actual;
}

function assertOutput(output, contract) {
  const expected = path.resolve(ROOT, contract.publicProjection.artifact);
  const actual = path.resolve(output);
  if (actual !== expected) throw new Error('public ranking output path has drifted');
  fs.mkdirSync(path.dirname(actual), { recursive: true });
  const relative = path.relative(ROOT, actual);
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error('public ranking must remain inside the repository');
  }
  if (fs.existsSync(actual) && fs.lstatSync(actual).isSymbolicLink()) {
    throw new Error('public ranking output must not be a symlink');
  }
  return actual;
}

function main() {
  const arguments_ = parseArguments(process.argv.slice(2));
  const contract = JSON.parse(fs.readFileSync(arguments_.contract, 'utf8'));
  const input = assertInput(arguments_.input, contract);
  const evidence = JSON.parse(fs.readFileSync(input, 'utf8'));
  const result = buildPublicTypesettingRiskRanking(evidence, contract);
  const output = assertOutput(arguments_.output, contract);
  fs.writeFileSync(output, `${JSON.stringify(result, null, 2)}\n`, { mode: 0o644 });
  process.stdout.write(`${JSON.stringify({
    status: 'complete',
    artifact: path.relative(ROOT, output),
    outputHash: result.outputHash,
    selection: result.selection,
    gates: result.gates,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`W4 public ranking failed: ${error.message}\n`);
    process.exitCode = 1;
  }
}
