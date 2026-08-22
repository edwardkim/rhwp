#!/usr/bin/env node

import fs from 'node:fs';
import { spawn } from 'node:child_process';

const inputIndex = process.argv.indexOf('--input');
const mode = inputIndex >= 0
  ? fs.readFileSync(process.argv[inputIndex + 1], 'utf8').trim()
  : 'exit';

function aggregate(extra = {}) {
  return {
    schemaVersion: 1,
    kind: 'font-metric-coverage-aggregate',
    status: 'complete',
    counts: {
      layoutCharacters: 0,
      coverageCharacters: 0,
      notApplicableCharacters: 0,
      excludedCharacters: 0,
      truncatedCharacters: 0,
    },
    categories: {
      'measured-overlay': 0,
      'identity-alias-hit': 0,
      'metric-surrogate': 0,
      'exact-hit': 0,
      'char-miss': 0,
      'face-miss': 0,
      heuristic: 0,
    },
    joins: { joined: 0, layoutOnly: 0, excluded: 0 },
    documents: {
      attempted: 1,
      success: 1,
      failures: {
        cancelled: 0,
        drm: 0,
        empty: 0,
        encrypted: 0,
        parser: 0,
        'resource-limit': 0,
        unsupported: 0,
      },
    },
    backends: { requested: 0, complete: 0, failed: 0, notObserved: 0, unsupported: 0 },
    ...extra,
  };
}

if (mode === 'success') {
  process.stdout.write(JSON.stringify(aggregate()));
} else if (mode === 'hang') {
  setInterval(() => {}, 1000);
} else if (mode.startsWith('descendant:')) {
  const marker = mode.slice('descendant:'.length);
  spawn(process.execPath, [
    '-e',
    "setTimeout(() => require('fs').writeFileSync(process.argv[1], 'survived'), 250); setTimeout(() => {}, 1000)",
    marker,
  ], { stdio: 'ignore' });
  setInterval(() => {}, 1000);
} else if (mode === 'overflow') {
  process.stdout.write('x'.repeat(256 * 1024));
} else if (mode === 'sensitive') {
  process.stdout.write(JSON.stringify(aggregate({ path: '/home/private/corpus/document.hwp' })));
} else if (mode === 'parser') {
  process.stdout.write(JSON.stringify({
    schemaVersion: 1,
    kind: 'font-metric-coverage-worker-result',
    status: 'failed',
    failure: 'parser',
  }));
  process.exitCode = 20;
} else if (mode === 'limits') {
  const limits = fs.readFileSync('/proc/self/limits', 'utf8');
  const finiteAddressSpace = /^Max address space\s+\d+\s+\d+\s+bytes\s*$/m.test(limits);
  const finiteCpu = /^Max cpu time\s+\d+\s+\d+\s+seconds\s*$/m.test(limits);
  if (finiteAddressSpace && finiteCpu) process.stdout.write(JSON.stringify(aggregate()));
  else process.exitCode = 9;
} else {
  process.exitCode = 9;
}
