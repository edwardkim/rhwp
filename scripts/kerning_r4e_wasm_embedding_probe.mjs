#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';

import { canonicalJson, sha256 } from './kerning_q1_baseline.mjs';

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const defaults = {
  wasm: path.join(repoRoot, 'pkg/rhwp_bg.wasm'),
  noto: path.join(repoRoot, 'ttfs/opensource/NotoSansKR-Regular.ttf'),
  smoke: path.join(repoRoot, 'tests/fixtures/fonts/RHWPExactKerningSmoke.ttf'),
  fixture: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/fixtures/kerning_runtime_fixture.hwpx'),
  output: path.join(repoRoot, 'mydocs/tech/investigations/issue-4968/kerning_r4e_wasm_embedding_probe.json'),
};

function parseArgs(argv) {
  const options = { ...defaults };
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    const name = key.startsWith('--') ? key.slice(2) : '';
    if (!(name in options)) throw new Error(`unknown option: ${key}`);
    const value = argv[++index];
    if (!value) throw new Error(`missing value for ${key}`);
    options[name] = path.resolve(value);
  }
  return options;
}

function digest(bytes) {
  return { bytes: bytes.length, sha256: sha256(bytes) };
}

export function probeEmbedding(wasm, inputs) {
  const probes = [
    ['full-noto-bytes', inputs.noto],
    ['full-smoke-face-bytes', inputs.smoke],
    ['runtime-fixture-bytes', inputs.fixture],
    ['smoke-fixture-file-name', Buffer.from('RHWPExactKerningSmoke.ttf')],
    ['runtime-fixture-file-name', Buffer.from('kerning_runtime_fixture.hwpx')],
    ['smoke-fixture-sha256', Buffer.from(sha256(inputs.smoke))],
    ['private-home-marker', Buffer.from('/home/edward/')],
    ['private-corpus-marker', Buffer.from('corpus_10k')],
    ['private-samples-marker', Buffer.from('hwpsamples')],
  ].map(([name, needle]) => ({ name, present: wasm.indexOf(needle) >= 0 }));
  const present = probes.filter((probe) => probe.present);
  if (present.length > 0) {
    throw new Error(`WASM embedding boundary failed: ${present.map((probe) => probe.name).join(',')}`);
  }
  return { status: 'pass', probeCount: probes.length, probes };
}

export function generateEmbeddingEvidence(options) {
  const wasm = fs.readFileSync(options.wasm);
  const inputs = {
    noto: fs.readFileSync(options.noto),
    smoke: fs.readFileSync(options.smoke),
    fixture: fs.readFileSync(options.fixture),
  };
  const output = {
    schemaVersion: 1,
    issue: 4968,
    stage: 'W9-Q3-5R4E-3',
    wasm: digest(wasm),
    inputs: {
      noto: digest(inputs.noto),
      smoke: digest(inputs.smoke),
      fixture: digest(inputs.fixture),
    },
    embedding: probeEmbedding(wasm, inputs),
  };
  output.evidenceCanonicalSha256 = sha256(canonicalJson(output));
  return output;
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const output = generateEmbeddingEvidence(options);
  fs.mkdirSync(path.dirname(options.output), { recursive: true });
  fs.writeFileSync(options.output, `${JSON.stringify(output, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify({
    status: output.embedding.status,
    probeCount: output.embedding.probeCount,
    wasmSha256: output.wasm.sha256,
    evidenceCanonicalSha256: output.evidenceCanonicalSha256,
  })}\n`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(new URL(import.meta.url).pathname)) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error.stack ?? error}\n`);
    process.exitCode = 1;
  }
}
