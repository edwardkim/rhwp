import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  mkdirSync, mkdtempSync, readFileSync, readlinkSync, renameSync, rmSync, writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { dioxusCliSource } from './dioxus-cli-version.mjs';
import {
  findSubsecondPdfTool,
  subsecondPdfToolSpecs,
} from './subsecond-pdf-tools.mjs';

export { subsecondPdfToolSpecs } from './subsecond-pdf-tools.mjs';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const PATCH_NAME = 'dioxus-cli-hotpatch-tip-dependents.patch';
const PATCH_DIFF_ARGS = [
  'diff', '--unified=0', '--no-ext-diff', '--no-textconv', '--no-color', '--abbrev=8',
  '--no-renames', '--no-indent-heuristic', '--inter-hunk-context=0', '--diff-algorithm=myers',
  '-O/dev/null', '--src-prefix=a/', '--dst-prefix=b/', 'HEAD',
];

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: ROOT, encoding: 'utf8', ...options });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} failed\n${result.stderr || result.stdout || ''}`);
  }
  return result.stdout?.trim() ?? '';
}

function pdfToolInstallHint(platform) {
  if (platform === 'darwin') return 'macOS: brew install ghostscript poppler';
  if (platform === 'win32') {
    return [
      'Windows: install Ghostscript from https://ghostscript.com/releases/',
      'and Poppler (including pdfinfo.exe), then add their bin directories to PATH.',
    ].join(' ');
  }
  if (platform === 'linux') {
    return 'Linux: install the ghostscript and poppler-utils packages with your distribution package manager.';
  }
  return [
    'Install Ghostscript from https://ghostscript.com/releases/ and Poppler from',
    'https://poppler.freedesktop.org/, then add their command-line tools to PATH.',
  ].join(' ');
}

export function assertSubsecondPdfTools(
  platform = process.platform,
  probe,
) {
  const specs = subsecondPdfToolSpecs(platform);
  const missing = [];
  if (!findSubsecondPdfTool('ghostscript', platform, probe)) {
    missing.push(`Ghostscript (${specs.ghostscript.commands.join(' or ')})`);
  }
  if (!findSubsecondPdfTool('pdfinfo', platform, probe)) {
    missing.push('Poppler pdfinfo (pdfinfo)');
  }
  if (missing.length === 0) return;
  throw new Error([
    `Subsecond PDF reference tools are missing: ${missing.join(', ')}.`,
    pdfToolInstallHint(platform),
    'After installation, open a new terminal and rerun pnpm subsecond:install.',
  ].join('\n'));
}

function trackedSourceState(directory) {
  const entries = run('git', ['-C', directory, 'ls-files', '--stage', '-z'])
    .split('\0').filter(Boolean).map(line => {
      const match = /^(\d+) ([0-9a-f]+) 0\t([^\n]+)$/.exec(line);
      if (!match) throw new Error(`unverifiable Dioxus index entry: ${line}`);
      return { mode: match[1], hash: match[2], file: match[3] };
    });
  const regular = entries.filter(entry => entry.mode !== '120000');
  const hashes = regular.length === 0 ? [] : run(
    'git', ['-C', directory, 'hash-object', '--no-filters', '--stdin-paths'],
    { input: `${regular.map(entry => entry.file).join('\n')}\n` },
  ).split('\n');
  let index = 0;
  let exact = true;
  const digest = createHash('sha256');
  for (const entry of entries) {
    const actual = entry.mode === '120000'
      ? run('git', ['-C', directory, 'hash-object', '--stdin'], {
        input: readlinkSync(path.join(directory, entry.file)),
      })
      : hashes[index++];
    exact &&= entry.hash === actual;
    digest.update(entry.file).update('\0').update(actual);
  }
  return { digest: digest.digest('hex'), exact };
}

function hasOnlyBuildOutputs(directory) {
  const flags = run('git', ['-C', directory, 'ls-files', '-v']).split('\n');
  const untracked = run(
    'git', ['-C', directory, 'status', '--porcelain=v1', '--untracked-files=all'],
  ).split('\n');
  const ignored = run(
    'git', ['-C', directory, 'ls-files', '--others', '--ignored', '--exclude-standard'],
  ).split('\n');
  return !flags.some(line => /^[a-zS] /.test(line))
    && !untracked.some(line => line.startsWith('?? '))
    && !ignored.some(file => file && !file.startsWith('target/'));
}

export function dioxusCliSourceDir(source, root = ROOT, patch = null) {
  if (source.kind !== 'git') return null;
  const contents = patch ?? readFileSync(path.join(root, 'scripts/patches', PATCH_NAME), 'utf8');
  const digest = createHash('sha256').update(contents).digest('hex');
  return path.join(root, 'target/dioxus-cli-source', `${source.rev}-${digest.slice(0, 16)}`);
}

function preparedIsExact(directory, revision, expected, patch) {
  try {
    const marker = readFileSync(path.join(directory, '.git/rhwp-patched'), 'utf8').trim();
    const sourceDigest = marker.startsWith(`${expected} `) ? marker.slice(expected.length + 1) : '';
    return /^[0-9a-f]{64}$/.test(sourceDigest)
      && run('git', ['-C', directory, 'rev-parse', 'HEAD']) === revision
      && run('git', ['-C', directory, ...PATCH_DIFF_ARGS]) === patch.trim()
      && trackedSourceState(directory).digest === sourceDigest
      && hasOnlyBuildOutputs(directory);
  } catch {
    return false;
  }
}

export function prepareDioxusCliSource(source, root = ROOT) {
  if (source.kind !== 'git') return null;
  const patch = readFileSync(path.join(root, 'scripts/patches', PATCH_NAME), 'utf8');
  const digest = createHash('sha256').update(patch).digest('hex');
  const directory = dioxusCliSourceDir(source, root, patch);
  const expected = `${source.rev} ${digest}`;
  if (!preparedIsExact(directory, source.rev, expected, patch)) {
    const parent = path.dirname(directory);
    mkdirSync(parent, { recursive: true });
    const temporary = mkdtempSync(path.join(parent, `${source.rev}.tmp-`));
    try {
      run('git', ['init', temporary]);
      run('git', ['-C', temporary, 'remote', 'add', 'origin', source.git]);
      run('git', ['-C', temporary, 'fetch', '--depth', '1', 'origin', source.rev]);
      run('git', ['-C', temporary, 'checkout', '--detach', 'FETCH_HEAD']);
      if (!trackedSourceState(temporary).exact || !hasOnlyBuildOutputs(temporary)) {
        throw new Error('fresh Dioxus checkout does not match its index');
      }
      run('git', ['-C', temporary, 'apply', '--check', '--unidiff-zero', '-'], { input: patch });
      run('git', ['-C', temporary, 'apply', '--unidiff-zero', '-'], { input: patch });
      if (run('git', ['-C', temporary, 'rev-parse', 'HEAD']) !== source.rev
        || run('git', ['-C', temporary, ...PATCH_DIFF_ARGS]) !== patch.trim()) {
        throw new Error('prepared Dioxus checkout does not match its revision and patch');
      }
      const sourceDigest = trackedSourceState(temporary).digest;
      writeFileSync(path.join(temporary, '.git/rhwp-patched'), `${expected} ${sourceDigest}\n`);
      try {
        renameSync(temporary, directory);
      } catch (error) {
        if (!['EEXIST', 'ENOTEMPTY'].includes(error?.code)
          || !preparedIsExact(directory, source.rev, expected, patch)) throw error;
      }
    } finally {
      rmSync(temporary, { recursive: true, force: true });
    }
  }
  if (!preparedIsExact(directory, source.rev, expected, patch)) {
    throw new Error(`Dioxus checkout does not match ${source.rev}: ${directory}`);
  }
  return directory;
}

export function dioxusCliInstallArgs(source, root = ROOT, prepared = null) {
  return [
    'install', 'dioxus-cli',
    ...(source.kind === 'git'
      ? ['--path', path.join(prepared ?? prepareDioxusCliSource(source, root), 'packages/cli')]
      : ['--version', source.version]),
    '--locked', '--root', path.join(root, 'target/dioxus-cli'),
  ];
}

export function installDioxusCli({
  checkPdfTools = assertSubsecondPdfTools,
  loadSource = dioxusCliSource,
  spawn = spawnSync,
} = {}) {
  checkPdfTools();
  const source = loadSource();
  const result = spawn('cargo', dioxusCliInstallArgs(source, ROOT), {
    cwd: ROOT,
    stdio: 'inherit',
  });
  if (result.error) throw result.error;
  return result.status ?? 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exitCode = installDioxusCli();
}
