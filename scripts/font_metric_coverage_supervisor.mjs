#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import {
  findSensitiveAggregateValues,
  reconcileCoverageAggregate,
} from './font_metric_coverage_contract.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT_PATH), '..');
const CONTRACT = JSON.parse(fs.readFileSync(path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
  'font_metric_coverage_contract.json',
), 'utf8'));

const MIB = 1024 * 1024;
const GIB = 1024 * MIB;
const DOCUMENT_FAILURE_EXIT = 20;
const FAILURE_STATES = new Set(CONTRACT.collectorRequirements.documentFailureStates);

export const DEFAULT_ISOLATION_LIMITS = Object.freeze({
  wallTimeoutMillis: 90_000,
  cpuSeconds: 75,
  addressSpaceBytes: 2 * GIB,
  maxStdoutBytes: 128 * MIB,
  rssPollMillis: 10,
});

function integerOption(name, value, minimum, maximum) {
  if (!Number.isSafeInteger(value) || value < minimum || value > maximum) {
    throw new Error(`${name} is outside the supervisor policy`);
  }
  return value;
}

function isolationLimits(overrides = {}) {
  const unknown = Object.keys(overrides).filter(key => !(key in DEFAULT_ISOLATION_LIMITS));
  if (unknown.length > 0) throw new Error('unknown isolation limit option');
  return {
    wallTimeoutMillis: integerOption(
      'wallTimeoutMillis',
      overrides.wallTimeoutMillis ?? DEFAULT_ISOLATION_LIMITS.wallTimeoutMillis,
      10,
      3_600_000,
    ),
    cpuSeconds: integerOption(
      'cpuSeconds',
      overrides.cpuSeconds ?? DEFAULT_ISOLATION_LIMITS.cpuSeconds,
      1,
      3_600,
    ),
    addressSpaceBytes: integerOption(
      'addressSpaceBytes',
      overrides.addressSpaceBytes ?? DEFAULT_ISOLATION_LIMITS.addressSpaceBytes,
      64 * MIB,
      8 * GIB,
    ),
    maxStdoutBytes: integerOption(
      'maxStdoutBytes',
      overrides.maxStdoutBytes ?? DEFAULT_ISOLATION_LIMITS.maxStdoutBytes,
      1024,
      128 * MIB,
    ),
    rssPollMillis: integerOption(
      'rssPollMillis',
      overrides.rssPollMillis ?? DEFAULT_ISOLATION_LIMITS.rssPollMillis,
      5,
      1000,
    ),
  };
}

function safeFailure(failure = 'resource-limit', metrics = {}) {
  return {
    schemaVersion: 1,
    kind: 'font-metric-coverage-document-result',
    status: 'failed',
    failure: FAILURE_STATES.has(failure) ? failure : 'resource-limit',
    metrics,
  };
}

function observedRssBytes(pid) {
  try {
    const status = fs.readFileSync(`/proc/${pid}/status`, 'utf8');
    const match = /^VmRSS:\s+(\d+)\s+kB$/m.exec(status);
    return match ? Number.parseInt(match[1], 10) * 1024 : 0;
  } catch {
    return 0;
  }
}

function killProcessGroup(child) {
  if (!Number.isInteger(child.pid) || child.pid <= 0) return;
  try {
    process.kill(-child.pid, 'SIGKILL');
  } catch {
    try {
      child.kill('SIGKILL');
    } catch {
      // The process may already have exited between observation and termination.
    }
  }
}

function parseWorkerPayload(output) {
  let payload;
  try {
    payload = JSON.parse(output);
  } catch {
    return null;
  }
  if (findSensitiveAggregateValues(payload, CONTRACT).length > 0) return null;
  return payload;
}

function completeEnvelope(aggregate, metrics) {
  if (aggregate?.schemaVersion !== 1
      || aggregate.kind !== 'font-metric-coverage-aggregate'
      || aggregate.status !== 'complete'
      || reconcileCoverageAggregate(aggregate, CONTRACT).length > 0) {
    return safeFailure('resource-limit', metrics);
  }
  return {
    schemaVersion: 1,
    kind: 'font-metric-coverage-document-result',
    status: 'complete',
    aggregate,
    metrics,
  };
}

/**
 * Run exactly one document in its own process group under Linux hard limits.
 * No input path, stderr, signal number, or raw error is returned.
 */
export function runIsolatedCoverageDocument(inputPath, options = {}) {
  if (process.platform !== 'linux') {
    throw new Error('font metric coverage isolation requires Linux');
  }
  const workerPath = options.workerPath;
  if (typeof workerPath !== 'string' || workerPath.length === 0) {
    throw new Error('workerPath is required');
  }
  const prlimitPath = options.prlimitPath ?? '/usr/bin/prlimit';
  if (!fs.existsSync(prlimitPath)) throw new Error('prlimit is unavailable');
  const limits = isolationLimits(options.limits);
  const workerArguments = Array.isArray(options.workerArguments)
    ? options.workerArguments.map(String)
    : [];
  const analysisOptionsJson = JSON.stringify(options.analysisOptions ?? {});
  const started = process.hrtime.bigint();

  return new Promise((resolve, reject) => {
    const arguments_ = [
      `--as=${limits.addressSpaceBytes}:${limits.addressSpaceBytes}`,
      `--cpu=${limits.cpuSeconds}:${limits.cpuSeconds}`,
      '--',
      workerPath,
      ...workerArguments,
      '--input',
      inputPath,
      '--options-json',
      analysisOptionsJson,
    ];
    const child = spawn(prlimitPath, arguments_, {
      cwd: ROOT,
      detached: true,
      env: {
        LANG: 'C.UTF-8',
        LC_ALL: 'C.UTF-8',
        RUST_BACKTRACE: '0',
      },
      shell: false,
      stdio: ['ignore', 'pipe', 'ignore'],
    });
    const chunks = [];
    let stdoutBytes = 0;
    let peakRssBytes = 0;
    let timedOut = false;
    let outputExceeded = false;
    let spawnError = null;

    const timeout = setTimeout(() => {
      timedOut = true;
      killProcessGroup(child);
    }, limits.wallTimeoutMillis);
    const rssPoll = setInterval(() => {
      peakRssBytes = Math.max(peakRssBytes, observedRssBytes(child.pid));
    }, limits.rssPollMillis);

    child.stdout.on('data', chunk => {
      stdoutBytes += chunk.length;
      if (stdoutBytes > limits.maxStdoutBytes) {
        outputExceeded = true;
        killProcessGroup(child);
        return;
      }
      chunks.push(chunk);
    });
    child.once('error', error => {
      spawnError = error;
    });
    child.once('close', (code, signal) => {
      clearTimeout(timeout);
      clearInterval(rssPoll);
      peakRssBytes = Math.max(peakRssBytes, observedRssBytes(child.pid));
      const elapsedMillis = Number(process.hrtime.bigint() - started) / 1_000_000;
      const metrics = {
        elapsedMillis: Math.round(elapsedMillis),
        peakRssBytes,
      };
      if (spawnError) {
        reject(new Error('isolated worker could not start'));
        return;
      }
      if (timedOut || outputExceeded || signal !== null) {
        resolve(safeFailure('resource-limit', metrics));
        return;
      }

      const payload = parseWorkerPayload(Buffer.concat(chunks).toString('utf8'));
      if (code === 0) {
        resolve(payload ? completeEnvelope(payload, metrics) : safeFailure('resource-limit', metrics));
        return;
      }
      if (code === DOCUMENT_FAILURE_EXIT
          && payload?.schemaVersion === 1
          && payload.kind === 'font-metric-coverage-worker-result'
          && payload.status === 'failed'
          && FAILURE_STATES.has(payload.failure)) {
        resolve(safeFailure(payload.failure, metrics));
        return;
      }
      resolve(safeFailure('resource-limit', metrics));
    });
  });
}

/**
 * Sequential corpus boundary. Completed aggregates are delivered transiently to
 * a callback; the returned summary contains no document path or per-document row.
 */
export async function runIsolatedCoverageDocuments(inputPaths, options = {}) {
  if (!Array.isArray(inputPaths)) throw new Error('inputPaths must be an array');
  const failures = Object.fromEntries([...FAILURE_STATES].sort().map(reason => [reason, 0]));
  const summary = {
    schemaVersion: 1,
    kind: 'font-metric-coverage-supervisor-summary',
    documents: { attempted: 0, success: 0, failures },
    resources: { peakWorkerRssBytes: 0, elapsedMillis: 0 },
  };
  const onDocumentComplete = options.onDocumentComplete;
  const documentOptions = { ...options };
  delete documentOptions.onDocumentComplete;

  for (const inputPath of inputPaths) {
    const result = await runIsolatedCoverageDocument(inputPath, documentOptions);
    summary.documents.attempted += 1;
    summary.resources.peakWorkerRssBytes = Math.max(
      summary.resources.peakWorkerRssBytes,
      result.metrics.peakRssBytes,
    );
    summary.resources.elapsedMillis += result.metrics.elapsedMillis;
    if (result.status === 'complete') {
      summary.documents.success += 1;
      if (onDocumentComplete) await onDocumentComplete(result.aggregate);
    } else {
      summary.documents.failures[result.failure] += 1;
    }
  }
  return summary;
}

function usage() {
  return 'usage: node scripts/font_metric_coverage_supervisor.mjs --worker <path> --input <path>\n';
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  const arguments_ = process.argv.slice(2);
  let workerPath;
  let inputPath;
  while (arguments_.length > 0) {
    const option = arguments_.shift();
    if (option === '--worker' && workerPath === undefined) workerPath = arguments_.shift();
    else if (option === '--input' && inputPath === undefined) inputPath = arguments_.shift();
    else {
      process.stderr.write(usage());
      process.exit(64);
    }
  }
  if (!workerPath || !inputPath) {
    process.stderr.write(usage());
    process.exit(64);
  }
  try {
    const result = await runIsolatedCoverageDocument(inputPath, { workerPath });
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exitCode = result.status === 'complete' ? 0 : DOCUMENT_FAILURE_EXIT;
  } catch {
    process.stdout.write(`${JSON.stringify(safeFailure())}\n`);
    process.exitCode = 70;
  }
}
