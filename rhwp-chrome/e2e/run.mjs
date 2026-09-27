#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { execFileSync, spawn } from 'node:child_process';
import { mkdir, mkdtemp, readFile, readdir, realpath, lstat, rm, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import os from 'node:os';
import path from 'node:path';

const currentFile = fileURLToPath(import.meta.url);
const root = path.resolve(path.dirname(currentFile), '../..');
const suites = ['extension-smoke', 'download-interceptor', 'extension-lifecycle'];
const selectionVariables = ['RHWP_EXTENSION_SMOKE_REPEAT', 'RHWP_EXTENSION_DOWNLOAD_CASE',
  'RHWP_EXTENSION_LIFECYCLE_REPEAT', 'RHWP_EXTENSION_LIFECYCLE_CASE'];
const within = (parent, child) => {
  const relative = path.relative(parent, child);
  return relative === '' || (relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative));
};

export function parseArgs(args) {
  const options = {};
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--help') return { help: true };
    if (!['--dist', '--output'].includes(args[i]) || !args[i + 1] || args[i + 1].startsWith('--')) {
      throw new Error(`Expected --dist DIRECTORY [--output NEW_DIRECTORY]; invalid argument: ${args[i]}`);
    }
    const key = args[i].slice(2);
    if (options[key]) throw new Error(`Duplicate option: ${args[i]}`);
    options[key] = path.resolve(args[++i]);
  }
  if (!options.dist) throw new Error('--dist is required; the runner never builds an extension');
  return options;
}

export async function inspectCandidate(directory) {
  const files = [];
  async function visit(relative = '') {
    for (const name of (await readdir(path.join(directory, relative))).sort()) {
      const file = path.join(relative, name);
      const full = path.join(directory, file);
      const stat = await lstat(full);
      if (stat.isSymbolicLink()) throw new Error(`Candidate must contain regular files, not symlinks: ${file}`);
      if (stat.isDirectory()) await visit(file);
      else if (stat.isFile()) files.push({ path: file.split(path.sep).join('/'),
        sha256: createHash('sha256').update(await readFile(full)).digest('hex') });
      else throw new Error(`Unsupported candidate entry: ${file}`);
    }
  }
  await visit();
  const manifest = JSON.parse(await readFile(path.join(directory, 'manifest.json'), 'utf8'));
  if (manifest.manifest_version !== 3 || !manifest.version) throw new Error('Expected a versioned MV3 extension');
  for (const required of ['viewer.html', 'wasm/rhwp.js', 'wasm/rhwp_bg.wasm']) {
    if (!files.some(file => file.path === required)) throw new Error(`Incomplete candidate: ${required}`);
  }
  return { directory, version: manifest.version, manifestVersion: manifest.manifest_version,
    // Hash a sorted path/content inventory, excluding timestamps and absolute paths.
    sha256: createHash('sha256').update(JSON.stringify(files)).digest('hex'), files };
}

async function browserEnvironment() {
  const { default: puppeteer } = await import('puppeteer');
  const require = createRequire(import.meta.url);
  let harnessSha = null, harnessDirty = null;
  try {
    harnessSha = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
    harnessDirty = Boolean(execFileSync('git', ['status', '--porcelain', '--', 'rhwp-chrome/e2e',
      'rhwp-chrome/package.json', 'rhwp-chrome/package-lock.json'], { cwd: root, encoding: 'utf8' }).trim());
  } catch { /* A source archive has no Git metadata. Candidate hashes remain authoritative. */ }
  return { node: process.version, platform: process.platform, arch: process.arch,
    puppeteer: require('puppeteer/package.json').version, chrome: await puppeteer.browserVersion(),
    executable: await puppeteer.executablePath(), harnessSha, harnessDirty };
}

async function canonicalPath(value) {
  try { return await realpath(value); } catch (error) {
    if (error.code !== 'ENOENT') throw error;
    return path.join(await canonicalPath(path.dirname(value)), path.basename(value));
  }
}

export async function runChecks({ dist, output }, { runner = runSuite, environment = browserEnvironment, env = process.env } = {}) {
  dist = path.resolve(dist);
  // Never write results into the candidate, including through a symlinked parent.
  const candidateDirectory = await realpath(dist).catch(() => dist);
  const parent = output ? path.dirname(path.resolve(output)) : path.join(root, 'output/chrome-extension-e2e');
  if (within(candidateDirectory, path.resolve(parent)) || (output && within(candidateDirectory, path.resolve(output)))) {
    throw new Error('Output must be outside the candidate');
  }
  const realParent = await canonicalPath(path.resolve(parent));
  if (within(candidateDirectory, realParent)) throw new Error('Output must be outside the candidate');
  await mkdir(realParent, { recursive: true });
  const explicitOutput = Boolean(output);
  output = output ? path.join(realParent, path.basename(output)) : await mkdtemp(path.join(realParent, 'run-'));
  if (explicitOutput) await mkdir(output); // Never overwrite an earlier run, even an empty directory.
  const report = { schemaVersion: 1, status: 'running', startedAt: new Date().toISOString(),
    candidate: null, environment: null,
    results: suites.map(name => ({ name, status: 'not-run' })),
    limitations: ['Unpacked Chrome for Testing only', 'Store installation and upgrades not tested',
      'Edge/Firefox and real user profiles not tested', 'Visual fidelity and release approval require manual review'] };
  const save = () => writeFile(path.join(output, 'result.json'), JSON.stringify(report, null, 2) + '\n');
  await save();
  try {
    for (const name of selectionVariables) if (env[name]) throw new Error(`${name} must be unset for a complete report`);
    report.candidate = await inspectCandidate(candidateDirectory);
    report.environment = await environment();
    await save();
    const started = Date.now();
    for (const [index, name] of suites.entries()) {
      const { log, ...outcome } = await runner([path.join(root, `rhwp-chrome/e2e/${name}.test.mjs`)], {
        env: { ...env, RHWP_EXTENSION_DIST_DIR: candidateDirectory, RHWP_EXTENSION_E2E_OUTPUT_DIR: output },
        timeoutMs: Math.max(1, 220_000 - (Date.now() - started)),
      });
      await writeFile(path.join(output, `${name}.log`), log);
      const passed = outcome.status === 0 && !outcome.timedOut && !outcome.error;
      report.results[index] = { name, ...outcome, exitCode: outcome.status, status: passed ? 'pass' : 'fail' };
      await save();
      if (!passed) break;
    }
    const after = await inspectCandidate(candidateDirectory);
    report.candidateUnchanged = after.sha256 === report.candidate.sha256;
    if (!report.candidateUnchanged) throw new Error('Candidate changed during verification; rerun with a stable candidate');
    report.status = report.results.every(item => item.status === 'pass') ? 'pass' : 'fail';
  } catch (error) {
    report.status = 'error';
    report.error = String(error.message || error);
  } finally {
    report.finishedAt = new Date().toISOString();
    await save();
  }
  return { report, output };
}

if (process.argv[1] && path.resolve(process.argv[1]) === currentFile) {
  try {
    const options = parseArgs(process.argv.slice(2));
    if (options.help) console.log('Usage: npm --prefix rhwp-chrome run test:e2e -- --dist DIRECTORY [--output NEW_DIRECTORY]\nTests an existing unpacked Chrome candidate without building. Results do not replace manual release verification.');
    else {
      const { report, output } = await runChecks(options);
      console.log(JSON.stringify({ status: report.status, output, error: report.error,
        candidateSha256: report.candidate?.sha256, results: report.results }));
      process.exitCode = report.status === 'pass' ? 0 : 1;
    }
  } catch (error) { console.error(error.message); process.exitCode = 1; }
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
