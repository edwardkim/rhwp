#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
const DEFAULT_POLICY_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
  'font_metric_coverage_pilot_policy.json',
);
const REQUIRED_NUMERIC_FIELDS = [
  'riskScore',
  'sizeBytes',
  'charCount',
  'compressedFixedFrameCharCount',
  'extremeCompressedCharCount',
  'kerningCharCount',
];

function compareText(left, right) {
  return Buffer.from(left, 'utf8').compare(Buffer.from(right, 'utf8'));
}

function compareByStratum(stratum) {
  const direction = stratum.direction === 'asc' ? 1 : -1;
  return (left, right) => {
    const primary = direction * (left[stratum.field] - right[stratum.field]);
    if (primary !== 0) return primary;
    if (left.riskScore !== right.riskScore) return right.riskScore - left.riskScore;
    if (left.charCount !== right.charCount) return right.charCount - left.charCount;
    if (left.sizeBytes !== right.sizeBytes) return right.sizeBytes - left.sizeBytes;
    return compareText(left.blake3, right.blake3);
  };
}

function validateCandidate(candidate, format) {
  if (candidate?.extension !== format) throw new Error(`candidate format mismatch: ${format}`);
  if (typeof candidate.source !== 'string' || candidate.source.length === 0) {
    throw new Error('candidate source is required');
  }
  if (typeof candidate.blake3 !== 'string' || !/^[0-9a-f]{64}$/u.test(candidate.blake3)) {
    throw new Error('candidate blake3 is invalid');
  }
  for (const field of REQUIRED_NUMERIC_FIELDS) {
    if (!Number.isSafeInteger(candidate[field]) || candidate[field] < 0) {
      throw new Error(`candidate ${field} is invalid`);
    }
  }
}

function validatePolicy(policy) {
  if (policy?.kind !== 'font-metric-coverage-pilot-selection-policy') {
    throw new Error('pilot selection policy kind is invalid');
  }
  if (!Array.isArray(policy.selection?.strata) || policy.selection.strata.length === 0) {
    throw new Error('pilot selection strata are required');
  }
  const perFormat = policy.selection.strata.reduce(
    (sum, stratum) => sum + stratum.countPerFormat,
    0,
  );
  const canaryPerFormat = policy.selection.strata
    .filter(stratum => stratum.tier === 'canary')
    .reduce((sum, stratum) => sum + stratum.countPerFormat, 0);
  if (perFormat !== policy.selection.documentsPerFormat
      || canaryPerFormat !== policy.selection.canaryDocumentsPerFormat) {
    throw new Error('pilot selection quota does not reconcile');
  }
  for (const stratum of policy.selection.strata) {
    if (!REQUIRED_NUMERIC_FIELDS.includes(stratum.field)
        || !['asc', 'desc'].includes(stratum.direction)
        || !['canary', 'full'].includes(stratum.tier)
        || !Number.isSafeInteger(stratum.countPerFormat)
        || stratum.countPerFormat < 1
        || !Number.isSafeInteger(stratum.minimum)
        || stratum.minimum < 0) {
      throw new Error(`pilot selection stratum is invalid: ${stratum.id}`);
    }
  }
}

export function selectFormatCandidates(candidates, format, policy) {
  validatePolicy(policy);
  const required = policy.candidateSources.requiredCandidatesPerFormat;
  if (!Array.isArray(candidates) || candidates.length < required) {
    throw new Error(`${format} candidate pool is smaller than policy`);
  }
  const unique = new Map();
  for (const candidate of candidates) {
    validateCandidate(candidate, format);
    const previous = unique.get(candidate.blake3);
    if (!previous || compareText(candidate.source, previous.source) < 0) {
      unique.set(candidate.blake3, candidate);
    }
  }
  if (unique.size < policy.selection.documentsPerFormat) {
    throw new Error(`${format} unique candidate pool is smaller than cohort`);
  }

  const selectedHashes = new Set();
  const selected = [];
  for (const stratum of policy.selection.strata) {
    const eligible = [...unique.values()]
      .filter(candidate => !selectedHashes.has(candidate.blake3))
      .filter(candidate => candidate[stratum.field] >= stratum.minimum)
      .sort(compareByStratum(stratum));
    if (eligible.length < stratum.countPerFormat) {
      throw new Error(`${format} stratum has insufficient candidates: ${stratum.id}`);
    }
    for (const candidate of eligible.slice(0, stratum.countPerFormat)) {
      selectedHashes.add(candidate.blake3);
      selected.push({
        format,
        tier: stratum.tier,
        stratum: stratum.id,
        source: candidate.source,
        blake3: candidate.blake3,
        sizeBytes: candidate.sizeBytes,
        charCount: candidate.charCount,
        riskScore: candidate.riskScore,
      });
    }
  }
  if (selected.length !== policy.selection.documentsPerFormat) {
    throw new Error(`${format} selected cohort does not reconcile`);
  }
  return selected;
}

export function selectPilotCohort(hwpReport, hwpxReport, policy) {
  validatePolicy(policy);
  const reports = { hwp: hwpReport, hwpx: hwpxReport };
  const selections = policy.selection.formats.flatMap(format => {
    const report = reports[format];
    if (!report || report.schemaVersion !== 'poc-font-layout-habits-v2') {
      throw new Error(`${format} POC report schema is invalid`);
    }
    return selectFormatCandidates(report.riskDocuments, format, policy);
  });
  const canary = selections.filter(item => item.tier === 'canary').length;
  if (canary !== policy.executionGate.canaryDocuments
      || selections.length !== policy.executionGate.fullDocuments) {
    throw new Error('selected cohort size does not match execution gate');
  }
  return {
    schemaVersion: 1,
    kind: 'font-metric-coverage-private-pilot-cohort',
    policyVersion: policy.policyVersion,
    localOnly: true,
    inputs: {
      hwpRepositoryHead: hwpReport.repositoryHead,
      hwpxRepositoryHead: hwpxReport.repositoryHead,
    },
    counts: {
      documents: selections.length,
      canaryDocuments: canary,
      hwp: selections.filter(item => item.format === 'hwp').length,
      hwpx: selections.filter(item => item.format === 'hwpx').length,
    },
    selections,
  };
}

export function assertLocalOutputPath(outputPath) {
  const outputRoot = path.join(ROOT, 'output');
  const resolved = path.resolve(outputPath);
  const relative = path.relative(outputRoot, resolved);
  if (relative === '' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    throw new Error('private pilot manifest must stay under output/');
  }
  return resolved;
}

function parseArguments(arguments_) {
  const values = {
    policy: DEFAULT_POLICY_PATH,
    hwp: path.join(ROOT, 'output', 'poc', 'font-layout-habits', 'summary-hwp-v2.json'),
    hwpx: path.join(ROOT, 'output', 'poc', 'font-layout-habits', 'summary-hwpx-v2.json'),
    output: path.join(ROOT, 'output', 'poc', 'font-metric-coverage', 'pilot-cohort-stage3-p1.json'),
  };
  for (let index = 0; index < arguments_.length; index += 2) {
    const option = arguments_[index];
    const value = arguments_[index + 1];
    if (!value || !['--policy', '--hwp', '--hwpx', '--output'].includes(option)) {
      throw new Error('usage: font_metric_coverage_pilot_selector.mjs [--policy file --hwp file --hwpx file --output file]');
    }
    values[option.slice(2)] = value;
  }
  return values;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function main() {
  const options = parseArguments(process.argv.slice(2));
  const outputPath = assertLocalOutputPath(options.output);
  const policy = readJson(options.policy);
  const manifest = selectPilotCohort(readJson(options.hwp), readJson(options.hwpx), policy);
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`, { mode: 0o600 });
  fs.chmodSync(outputPath, 0o600);
  process.stdout.write(
    `pilot cohort selection: ok; ${manifest.counts.documents} documents; `
      + `${manifest.counts.hwp} HWP, ${manifest.counts.hwpx} HWPX; `
      + `${manifest.counts.canaryDocuments} canary; local manifest under output/\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`pilot cohort selection failed: ${error.message}\n`);
    process.exitCode = 1;
  }
}
