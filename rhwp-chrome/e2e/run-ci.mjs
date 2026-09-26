#!/usr/bin/env node
import { createRequire } from 'node:module';
import { appendFile, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import os from 'node:os';
import path from 'node:path';
import puppeteer from 'puppeteer';

const currentFile = fileURLToPath(import.meta.url);
const root = path.resolve(path.dirname(currentFile), '../..');
if (process.argv[1] && path.resolve(process.argv[1]) === currentFile) await main();

async function main() {
  // The aggregate must always exercise every case exactly once, including locally.
  for (const name of ['RHWP_EXTENSION_SMOKE_REPEAT', 'RHWP_EXTENSION_DOWNLOAD_CASE',
    'RHWP_EXTENSION_LIFECYCLE_REPEAT', 'RHWP_EXTENSION_LIFECYCLE_CASE', 'RHWP_EXTENSION_DIST_DIR']) {
    if (process.env[name]) throw new Error(`${name} must be unset for the complete CI suite`);
  }
  const output = path.resolve(process.env.RHWP_EXTENSION_E2E_OUTPUT_DIR || path.join(root, 'output/chrome-extension-e2e'));
  await mkdir(output, { recursive: true });
  const require = createRequire(import.meta.url);
  const manifest = JSON.parse(await readFile(path.join(root, 'rhwp-chrome/dist/manifest.json'), 'utf8'));
  const versions = {
    puppeteer: require('puppeteer/package.json').version, chrome: await puppeteer.browserVersion(),
    executable: await puppeteer.executablePath(), extension: manifest.version, manifest: manifest.manifest_version,
    sourceSha: process.env.GITHUB_SHA || 'local',
  };
  const started = Date.now();
  const results = [];
  console.log(JSON.stringify(versions));
  for (const name of ['extension-smoke', 'download-interceptor', 'extension-lifecycle']) {
    const result = await runSuite([path.join(root, `rhwp-chrome/e2e/${name}.test.mjs`)], {
      env: { ...process.env, RHWP_EXTENSION_E2E_OUTPUT_DIR: output },
      timeoutMs: Math.max(1, 220_000 - (Date.now() - started)),
    });
    const { log, ...outcome } = result;
    process.stdout.write(log);
    await writeFile(path.join(output, `${name}.log`), log);
    results.push({ name, ...outcome });
    await writeFile(path.join(output, 'result.json'), JSON.stringify({ versions, results, durationMs: Date.now() - started }, null, 2));
    if (result.status !== 0 || result.timedOut || result.error) { process.exitCode = 1; break; }
  }
  const durationMs = Date.now() - started;
  console.log(JSON.stringify({ results, durationMs }));
  if (process.env.GITHUB_STEP_SUMMARY) {
    await appendFile(process.env.GITHUB_STEP_SUMMARY,
      `Chrome ${versions.chrome}, Puppeteer ${versions.puppeteer}, extension ${versions.extension}\n\n`
      + results.map(item => `- ${item.name}: ${item.status === 0 && !item.timedOut ? 'PASS' : 'FAIL'} (${(item.durationMs / 1000).toFixed(1)}s)`).join('\n')
      + `\n\nBrowser suites: ${(durationMs / 1000).toFixed(1)}s; no retries.\n`);
  }
}

export async function runSuite(args, { env = process.env, timeoutMs, killGraceMs = 5_000 }) {
  const temporaryRoot = await mkdtemp(path.join(os.tmpdir(), 'rhwp-e2e-runner-'));
  const started = Date.now();
  let timer, forceKill, log = '', timedOut = false, spawnError;
  try {
    const child = spawn(process.execPath, args, {
      cwd: root, stdio: ['ignore', 'pipe', 'pipe'],
      env: { ...env, TMPDIR: temporaryRoot, TMP: temporaryRoot, TEMP: temporaryRoot },
    });
    const collect = data => { log = `${log}${data}`.slice(-256 * 1024); };
    child.stdout.on('data', collect);
    child.stderr.on('data', collect);
    child.on('error', error => { spawnError = String(error); });
    timer = setTimeout(() => {
      timedOut = true;
      // Puppeteer's SIGTERM handler closes the owned browser; allow its finally
      // blocks to clean up before forcing only this child to stop.
      child.kill('SIGTERM');
      forceKill = setTimeout(() => child.kill('SIGKILL'), killGraceMs);
    }, timeoutMs);
    const [status, signal] = await new Promise(resolve => child.once('close', (...values) => resolve(values)));
    if (timedOut) collect(`\nSuite exceeded ${timeoutMs}ms; stopped without retry.\n`);
    if (spawnError) collect(`\n${spawnError}\n`);
    return { status, signal, timedOut, error: spawnError, durationMs: Date.now() - started, log };
  } finally {
    clearTimeout(timer);
    clearTimeout(forceKill);
    // Profiles/downloads are children of this runner-owned temporary directory.
    await rm(temporaryRoot, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 });
  }
}
