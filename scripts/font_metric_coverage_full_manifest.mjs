#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { assertLocalOutputPath } from './font_metric_coverage_pilot_selector.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
const INVESTIGATION = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
);
const DEFAULT_POLICY_PATH = path.join(
  INVESTIGATION,
  'font_metric_coverage_full_manifest_policy.json',
);
const DEFAULT_CHECKPOINT_POLICY_PATH = path.join(
  INVESTIGATION,
  'font_metric_coverage_checkpoint_policy.json',
);

function sha256Bytes(value) {
  return createHash('sha256').update(value).digest('hex');
}

function safeInteger(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function validatePolicy(policy) {
  if (policy?.schemaVersion !== 1
      || policy.kind !== 'font-metric-coverage-full-manifest-policy'
      || policy.policyVersion !== 'stage4-a-v1'
      || policy.discovery?.rejectSymlinks !== true
      || policy.discovery?.rejectSpecialFiles !== true
      || !safeInteger(policy.discovery?.maxInputBytes)
      || policy.discovery.maxInputBytes === 0
      || policy.identity?.algorithm !== 'blake3'
      || policy.identity?.allowDuplicateContent !== true
      || policy.identity?.rejectDuplicateSource !== true
      || policy.execution?.verifyStableStatAcrossHash !== true
      || !safeInteger(policy.execution?.hashConcurrency)
      || policy.execution.hashConcurrency === 0
      || policy.storage?.requireCheckpointMaximumPlusReserve !== true
      || policy.privacy?.manifestLocalOnly !== true
      || policy.privacy?.preflightContainsDocumentIdentity !== false
      || policy.privacy?.publishManifest !== false) {
    throw new Error('font metric coverage full manifest policy is invalid');
  }
  for (const field of ['documents', 'candidateBytes', 'ignoredRegularFiles', 'ignoredBytes']) {
    if (!safeInteger(policy.expected?.[field])) {
      throw new Error(`full manifest expected.${field} is invalid`);
    }
  }
  for (const format of ['hwp', 'hwpx']) {
    if (!safeInteger(policy.expected?.formats?.[format])) {
      throw new Error(`full manifest expected format ${format} is invalid`);
    }
  }
  const extensions = policy.discovery.extensions;
  if (extensions?.['.hwp'] !== 'hwp' || extensions?.['.hwpx'] !== 'hwpx') {
    throw new Error('full manifest extension mapping is invalid');
  }
  if (!Array.isArray(policy.discovery.ignoredExtensions)
      || policy.discovery.ignoredExtensions.some(value => typeof value !== 'string')) {
    throw new Error('full manifest ignored extensions are invalid');
  }
}

function withinRoot(root, candidate) {
  const relative = path.relative(root, candidate);
  return relative === '' || (!relative.startsWith(`..${path.sep}`) && relative !== '..' && !path.isAbsolute(relative));
}

function statIdentity(stats) {
  return [stats.dev, stats.ino, stats.size, stats.mtimeNs].map(String).join(':');
}

function discoverCorpus(corpusRoot, policy) {
  const root = fs.realpathSync(corpusRoot);
  const rootStats = fs.statSync(root);
  if (!rootStats.isDirectory()) throw new Error('corpus root must be a directory');
  const candidates = [];
  let ignoredRegularFiles = 0;
  let ignoredBytes = 0;
  const stack = [root];
  while (stack.length > 0) {
    const directory = stack.pop();
    const entries = fs.readdirSync(directory, { withFileTypes: true })
      .sort((left, right) => compareText(left.name, right.name));
    for (const entry of entries) {
      const candidate = path.join(directory, entry.name);
      if (entry.isSymbolicLink()) {
        throw new Error('full manifest corpus contains a symlink');
      }
      if (entry.isDirectory()) {
        stack.push(candidate);
        continue;
      }
      if (!entry.isFile()) throw new Error('full manifest corpus contains a special file');
      const extension = path.extname(entry.name).toLowerCase();
      const format = policy.discovery.extensions[extension];
      const stats = fs.statSync(candidate, { bigint: true });
      if (!format) {
        if (!policy.discovery.ignoredExtensions.includes(extension)) {
          throw new Error('full manifest corpus contains an unexpected file type');
        }
        ignoredRegularFiles += 1;
        ignoredBytes += Number(stats.size);
        continue;
      }
      if (stats.size > BigInt(policy.discovery.maxInputBytes)) {
        throw new Error('full manifest input exceeds the worker byte limit');
      }
      const real = fs.realpathSync(candidate);
      if (!withinRoot(root, real)) throw new Error('full manifest source escapes corpus root');
      candidates.push({
        source: real,
        format,
        sizeBytes: Number(stats.size),
        statIdentity: statIdentity(stats),
      });
    }
  }
  return { root, candidates, ignoredRegularFiles, ignoredBytes };
}

function hashWithAgent(filePath, rhwpAgent) {
  return new Promise((resolve, reject) => {
    execFile(
      rhwpAgent,
      ['hash', filePath, '--json'],
      { encoding: 'utf8', maxBuffer: 64 * 1024 },
      (error, stdout) => {
        if (error) {
          reject(new Error('rhwp-agent failed to hash a corpus document'));
          return;
        }
        try {
          const payload = JSON.parse(stdout);
          resolve({ hash: payload.hash, bytes: payload.bytes });
        } catch {
          reject(new Error('rhwp-agent returned an invalid hash envelope'));
        }
      },
    );
  });
}

async function hashCandidates(candidates, options) {
  const results = new Array(candidates.length);
  let nextIndex = 0;
  let completed = 0;
  const concurrency = Math.min(options.concurrency, candidates.length || 1);
  const workers = Array.from({ length: concurrency }, async () => {
    while (true) {
      const index = nextIndex;
      nextIndex += 1;
      if (index >= candidates.length) return;
      const candidate = candidates[index];
      const result = await options.hashFile(candidate.source, options.rhwpAgent);
      if (!/^[0-9a-f]{64}$/u.test(result?.hash ?? '')
          || result.bytes !== candidate.sizeBytes) {
        throw new Error('full manifest hash envelope does not match the input');
      }
      const after = fs.statSync(candidate.source, { bigint: true });
      if (statIdentity(after) !== candidate.statIdentity) {
        throw new Error('full manifest input changed while hashing');
      }
      results[index] = { ...candidate, blake3: result.hash };
      completed += 1;
      if (options.onProgress) options.onProgress(completed, candidates.length);
    }
  });
  await Promise.all(workers);
  return results;
}

function validateExpected(discovery, policy) {
  const formats = { hwp: 0, hwpx: 0 };
  let candidateBytes = 0;
  let maxInputBytes = 0;
  for (const document of discovery.candidates) {
    formats[document.format] += 1;
    candidateBytes += document.sizeBytes;
    maxInputBytes = Math.max(maxInputBytes, document.sizeBytes);
  }
  if (discovery.candidates.length !== policy.expected.documents
      || formats.hwp !== policy.expected.formats.hwp
      || formats.hwpx !== policy.expected.formats.hwpx
      || candidateBytes !== policy.expected.candidateBytes
      || discovery.ignoredRegularFiles !== policy.expected.ignoredRegularFiles
      || discovery.ignoredBytes !== policy.expected.ignoredBytes) {
    throw new Error('full manifest corpus does not match the frozen Stage 4-A inventory');
  }
  return { formats, candidateBytes, maxInputBytes };
}

function duplicateSummary(documents) {
  const counts = new Map();
  for (const document of documents) {
    const key = `${document.format}:${document.blake3}`;
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  const duplicates = [...counts.values()].filter(count => count > 1);
  return {
    groups: duplicates.length,
    extraInstances: duplicates.reduce((total, count) => total + count - 1, 0),
  };
}

export async function buildFullCoverageManifest(options) {
  const policyBytes = options.policyBytes ?? fs.readFileSync(DEFAULT_POLICY_PATH);
  const policy = options.policy ?? JSON.parse(policyBytes.toString('utf8'));
  validatePolicy(policy);
  if (!/^[0-9a-f]{40}$/u.test(options.sourceHead ?? '')) {
    throw new Error('full manifest sourceHead must be a full Git commit');
  }
  const checkpointPolicyBytes = options.checkpointPolicyBytes
    ?? fs.readFileSync(DEFAULT_CHECKPOINT_POLICY_PATH);
  const checkpointPolicy = JSON.parse(checkpointPolicyBytes.toString('utf8'));
  const discovery = discoverCorpus(options.corpusRoot, policy);
  const inventory = validateExpected(discovery, policy);
  const rhwpAgent = fs.realpathSync(options.rhwpAgent);
  if (!fs.statSync(rhwpAgent).isFile()) throw new Error('rhwp-agent must be a regular file');
  const started = process.hrtime.bigint();
  const hashed = await hashCandidates(discovery.candidates, {
    concurrency: policy.execution.hashConcurrency,
    hashFile: options.hashFile ?? hashWithAgent,
    rhwpAgent,
    onProgress: options.onProgress,
  });
  const documents = hashed
    .map(({ source, format, sizeBytes, blake3 }) => ({ source, format, sizeBytes, blake3 }))
    .sort((left, right) => (
      compareText(left.format, right.format)
      || compareText(left.blake3, right.blake3)
      || left.sizeBytes - right.sizeBytes
      || compareText(left.source, right.source)
    ));
  if (new Set(documents.map(document => document.source)).size !== documents.length) {
    throw new Error('full manifest contains a duplicate source');
  }
  const checkpointFilesystem = fs.statfsSync(options.checkpointFilesystemPath, { bigint: true });
  const availableBytes = checkpointFilesystem.bavail * checkpointFilesystem.bsize;
  const requiredBytes = BigInt(
    checkpointPolicy.storage.maxJournalBytes
      + checkpointPolicy.storage.minimumFreeBytesAfterAppend,
  );
  if (availableBytes < requiredBytes) {
    throw new Error('checkpoint filesystem cannot satisfy maximum journal plus reserve');
  }
  const manifest = {
    schemaVersion: 1,
    kind: 'font-metric-coverage-private-corpus-manifest',
    policyVersion: policy.policyVersion,
    localOnly: true,
    corpusRoot: discovery.root,
    sourceHead: options.sourceHead,
    builder: {
      policySha256: sha256Bytes(policyBytes),
      scriptSha256: sha256Bytes(fs.readFileSync(SCRIPT_PATH)),
      rhwpAgentSha256: sha256Bytes(fs.readFileSync(rhwpAgent)),
      checkpointPolicySha256: sha256Bytes(checkpointPolicyBytes),
    },
    corpus: {
      documents: documents.length,
      formats: inventory.formats,
      candidateBytes: inventory.candidateBytes,
      ignoredRegularFiles: discovery.ignoredRegularFiles,
      ignoredBytes: discovery.ignoredBytes,
    },
    documents,
  };
  const manifestBytes = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`);
  const duplicateContent = duplicateSummary(documents);
  const preflight = {
    schemaVersion: 1,
    kind: 'font-metric-coverage-full-manifest-preflight',
    status: 'complete',
    policyVersion: policy.policyVersion,
    sourceHead: options.sourceHead,
    documents: documents.length,
    formats: inventory.formats,
    candidateBytes: inventory.candidateBytes,
    maximumInputBytes: inventory.maxInputBytes,
    ignoredRegularFiles: discovery.ignoredRegularFiles,
    ignoredBytes: discovery.ignoredBytes,
    duplicateContent,
    manifestSha256: sha256Bytes(manifestBytes),
    hashingElapsedMillis: Math.round(Number(process.hrtime.bigint() - started) / 1_000_000),
    checkpointStorage: {
      availableBytes: Number(availableBytes),
      maximumJournalBytes: checkpointPolicy.storage.maxJournalBytes,
      minimumFreeBytesAfterAppend: checkpointPolicy.storage.minimumFreeBytesAfterAppend,
      maximumPlusReserveSatisfied: true,
    },
    privacy: {
      containsDocumentIdentity: false,
      manifestLocalOnly: true,
    },
  };
  return { manifest, manifestBytes, preflight };
}

function parseArguments(arguments_) {
  const values = {
    policy: DEFAULT_POLICY_PATH,
    checkpointPolicy: DEFAULT_CHECKPOINT_POLICY_PATH,
    rhwpAgent: path.join(ROOT, 'target', 'debug', 'rhwp-agent'),
  };
  const allowed = new Set([
    '--corpus-root',
    '--manifest',
    '--preflight',
    '--source-head',
    '--rhwp-agent',
    '--policy',
    '--checkpoint-policy',
  ]);
  for (let index = 0; index < arguments_.length; index += 2) {
    const option = arguments_[index];
    const value = arguments_[index + 1];
    if (!allowed.has(option) || value === undefined) {
      throw new Error('usage: full manifest --corpus-root dir --manifest file --preflight file --source-head commit');
    }
    const key = option.slice(2).replaceAll(/-([a-z])/gu, (_, letter) => letter.toUpperCase());
    values[key] = value;
  }
  for (const field of ['corpusRoot', 'manifest', 'preflight', 'sourceHead']) {
    if (!values[field]) throw new Error(`full manifest requires ${field}`);
  }
  return values;
}

function writeNewPrivateFile(filePath, bytes) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true, mode: 0o700 });
  fs.chmodSync(path.dirname(filePath), 0o700);
  const descriptor = fs.openSync(filePath, 'wx', 0o600);
  try {
    fs.writeFileSync(descriptor, bytes);
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
  fs.chmodSync(filePath, 0o600);
}

async function main() {
  const arguments_ = parseArguments(process.argv.slice(2));
  const manifestPath = assertLocalOutputPath(arguments_.manifest);
  const preflightPath = assertLocalOutputPath(arguments_.preflight);
  const policyBytes = fs.readFileSync(arguments_.policy);
  const checkpointPolicyBytes = fs.readFileSync(arguments_.checkpointPolicy);
  let lastReported = 0;
  const result = await buildFullCoverageManifest({
    corpusRoot: arguments_.corpusRoot,
    sourceHead: arguments_.sourceHead,
    rhwpAgent: arguments_.rhwpAgent,
    checkpointFilesystemPath: path.dirname(manifestPath),
    policy: JSON.parse(policyBytes.toString('utf8')),
    policyBytes,
    checkpointPolicyBytes,
    onProgress: (completed, total) => {
      if (completed - lastReported >= 500 || completed === total) {
        lastReported = completed;
        process.stderr.write(`full manifest hash progress: ${completed}/${total}\n`);
      }
    },
  });
  writeNewPrivateFile(manifestPath, result.manifestBytes);
  writeNewPrivateFile(preflightPath, Buffer.from(`${JSON.stringify(result.preflight, null, 2)}\n`));
  process.stdout.write(`${JSON.stringify(result.preflight)}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    await main();
  } catch (error) {
    process.stderr.write(`full manifest failed: ${error.message}\n`);
    process.exitCode = 1;
  }
}
