import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  existsSync, mkdirSync, mkdtempSync, readFileSync, readlinkSync, renameSync, rmSync,
  symlinkSync, unlinkSync, writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { dioxusCliSource } from './dioxus-cli-version.mjs';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const PATCH_DIFF_ARGS = [
  'diff', '--unified=0', '--no-ext-diff', '--no-textconv', '--no-color', '--abbrev=8', '--no-renames',
  '--no-indent-heuristic', '--inter-hunk-context=0', '--diff-algorithm=myers',
  '-O/dev/null', '--src-prefix=a/', '--dst-prefix=b/', 'HEAD',
];
const DIOXUS_CLI_PATCHES = [
  'dioxus-cli-hotpatch-tip-dependents.patch',
];
const CLAIM_WAIT = new Int32Array(new SharedArrayBuffer(4));
function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: ROOT,
    encoding: 'utf8',
    ...options,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    const detail = [result.stdout, result.stderr].filter(Boolean).join('\n').trim();
    throw new Error(`${command} ${args.join(' ')} failed (${result.status})${detail ? `\n${detail}` : ''}`);
  }
  return result.stdout?.trim() ?? '';
}

function dioxusCliPatches(root) {
  return DIOXUS_CLI_PATCHES.map(name =>
    readFileSync(path.join(root, 'scripts', 'patches', name), 'utf8'));
}

function dioxusCliPatchDigest(root) {
  return createHash('sha256').update(dioxusCliPatches(root).join('')).digest('hex');
}

export function dioxusCliSourceDir(source, root = ROOT, patchDigest = null) {
  if (source.kind !== 'git') return null;
  const patchKey = (patchDigest ?? dioxusCliPatchDigest(root)).slice(0, 16);
  return path.join(root, 'target', 'dioxus-cli-source', `${source.rev}-${patchKey}`);
}

function dioxusCliIndexEntries(sourceDir) {
  return run('git', ['-C', sourceDir, 'ls-files', '--stage', '-z'])
    .split('\0')
    .filter(Boolean)
    .map(line => {
      const match = /^(\d+) ([0-9a-f]+) 0\t([\s\S]+)$/.exec(line);
      if (!match || match[3].includes('\n')) {
        throw new Error(`Dioxus CLI checkout의 index 항목을 검증할 수 없다: ${line}`);
      }
      return { mode: match[1], expected: match[2], file: match[3] };
    });
}

export function dioxusCliSourceDigest(sourceDir) {
  const digest = createHash('sha256');
  for (const entry of dioxusCliIndexEntries(sourceDir)) {
    const file = path.join(sourceDir, entry.file);
    const contents = entry.mode === '120000' ? readlinkSync(file) : readFileSync(file);
    digest.update(entry.file).update('\0').update(createHash('sha256').update(contents).digest());
  }
  return digest.digest('hex');
}

export function verifyDioxusPristineCheckout(sourceDir) {
  const entries = dioxusCliIndexEntries(sourceDir);
  const regular = entries.filter(entry => entry.mode !== '120000');
  const actual = regular.length === 0 ? [] : run(
    'git', ['-C', sourceDir, 'hash-object', '--no-filters', '--stdin-paths'], {
      input: `${regular.map(entry => entry.file).join('\n')}\n`,
    },
  ).split('\n');
  let regularIndex = 0;
  const mismatches = entries.filter(entry => {
    const hash = entry.mode === '120000'
      ? run('git', ['-C', sourceDir, 'hash-object', '--stdin'], {
        input: readlinkSync(path.join(sourceDir, entry.file)),
      })
      : actual[regularIndex++];
    return entry.expected !== hash;
  });
  if (mismatches.length > 0) {
    const shown = mismatches.slice(0, 20).map(entry => entry.file);
    if (mismatches.length > shown.length) shown.push(`... ${mismatches.length - shown.length}개 더`);
    throw new Error(
      `Dioxus CLI fresh checkout의 raw source가 commit과 다르다:\n${shown.join('\n')}`,
    );
  }
}

export function verifyDioxusCliCheckoutFiles(sourceDir) {
  const maskedTracked = run('git', ['-C', sourceDir, 'ls-files', '-v'])
    .split('\n')
    .filter(line => /^[a-zS] /.test(line));
  if (maskedTracked.length > 0) {
    throw new Error(`Dioxus CLI exact checkout에 숨겨진 index flag가 있다:\n${maskedTracked.join('\n')}`);
  }
  const untracked = run('git', ['-C', sourceDir, 'status', '--porcelain=v1', '--untracked-files=all'])
    .split('\n')
    .filter(line => line.startsWith('?? '));
  if (untracked.length > 0) {
    throw new Error(`Dioxus CLI exact checkout에 untracked source가 있다:\n${untracked.join('\n')}`);
  }
  const ignoredSource = run('git', [
    '-C', sourceDir, 'ls-files', '--others', '--ignored', '--exclude-standard',
  ]).split('\n').filter(file => file && !file.startsWith('target/'));
  if (ignoredSource.length > 0) {
    throw new Error(`Dioxus CLI exact checkout에 ignored source가 있다:\n${ignoredSource.join('\n')}`);
  }
}

function dioxusCliSourceIsReady(sourceDir, readyRevision) {
  const readyPath = path.join(sourceDir, '.git', 'rhwp-patched-v2');
  if (!existsSync(readyPath)) return false;
  try {
    return readFileSync(readyPath, 'utf8').trim()
      === `${readyRevision} ${dioxusCliSourceDigest(sourceDir)}`;
  } catch {
    return false;
  }
}

function processExists(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error?.code !== 'ESRCH';
  }
}

function acquireGenerationClaim(sourceDir, readyRevision) {
  const claimPath = `${sourceDir}.building`;
  const owner = String(process.pid);
  const deadline = Date.now() + 30_000;
  while (!dioxusCliSourceIsReady(sourceDir, readyRevision)) {
    try {
      symlinkSync(owner, claimPath);
      return () => {
        try {
          if (readlinkSync(claimPath) === owner) unlinkSync(claimPath);
        } catch (error) {
          if (error?.code !== 'ENOENT') throw error;
        }
      };
    } catch (error) {
      if (error?.code !== 'EEXIST') throw error;
      let currentOwner;
      try {
        currentOwner = Number(readlinkSync(claimPath));
      } catch (readError) {
        if (readError?.code === 'ENOENT') continue;
        throw readError;
      }
      if (Number.isInteger(currentOwner) && !processExists(currentOwner)) {
        throw new Error(
          `Dioxus CLI generation claim owner(${currentOwner})가 종료됐다. 실행 중인 installer가 없는지 확인한 뒤 이 경로를 제거해야 한다: ${claimPath}`,
        );
      }
      if (Date.now() >= deadline) {
        throw new Error(`Dioxus CLI generation build claim 대기 시간이 초과됐다: ${claimPath}`);
      }
      Atomics.wait(CLAIM_WAIT, 0, 0, 50);
    }
  }
  return null;
}

/**
 * Dioxus HEAD의 workspace replay가 tip과 무관한 reverse-dependent까지 고르는 회귀를
 * exact checkout 위에서만 보정한다. 패치가 더는 clean apply되지 않으면 upstream이
 * 움직였다는 뜻이므로 설치를 멈추고 재검토하게 한다.
 */
export function prepareDioxusCliSource(source, root = ROOT) {
  if (source.kind !== 'git') return null;
  const patches = dioxusCliPatches(root);
  const patch = patches.join('');
  const patchDigest = createHash('sha256').update(patch).digest('hex');
  const sourceDir = dioxusCliSourceDir(source, root, patchDigest);
  const sourceParent = path.dirname(sourceDir);
  const readyRevision = `${source.rev} ${patchDigest}`;
  mkdirSync(sourceParent, { recursive: true });
  const ready = dioxusCliSourceIsReady(sourceDir, readyRevision);
  if (!ready) {
    const releaseClaim = acquireGenerationClaim(sourceDir, readyRevision);
    if (releaseClaim) {
      let temporaryDir = null;
      try {
        if (!dioxusCliSourceIsReady(sourceDir, readyRevision)) {
          if (existsSync(sourceDir)) {
            throw new Error(`Dioxus CLI immutable source cache가 손상됐다: ${sourceDir}`);
          }
          temporaryDir = mkdtempSync(path.join(sourceParent, `${source.rev}.tmp-`));
          run('git', ['clone', '--filter=blob:none', '--no-checkout', source.git, temporaryDir], { stdio: 'inherit' });
          run('git', ['-C', temporaryDir, 'fetch', '--depth', '1', 'origin', source.rev], { stdio: 'inherit' });
          run('git', ['-C', temporaryDir, 'checkout', '--detach', source.rev], { stdio: 'inherit' });
          const preparedHead = run('git', ['-C', temporaryDir, 'rev-parse', 'HEAD']);
          if (preparedHead !== source.rev) {
            throw new Error(`Dioxus CLI temporary checkout HEAD(${preparedHead})가 exact pin(${source.rev})과 다르다`);
          }
          verifyDioxusPristineCheckout(temporaryDir);
          for (const currentPatch of patches) {
            run('git', ['-C', temporaryDir, 'apply', '--check', '--unidiff-zero', '-'], { input: currentPatch });
            run('git', ['-C', temporaryDir, 'apply', '--unidiff-zero', '-'], { input: currentPatch });
          }
          const preparedDiff = run('git', ['-C', temporaryDir, ...PATCH_DIFF_ARGS]);
          if (preparedDiff !== patch.trimEnd()) {
            throw new Error('Dioxus CLI temporary checkout의 diff가 선언된 workaround와 다르다');
          }
          const preparedSourceDigest = dioxusCliSourceDigest(temporaryDir);
          writeFileSync(
            path.join(temporaryDir, '.git', 'rhwp-patched-v2'),
            `${readyRevision} ${preparedSourceDigest}\n`,
          );
          try {
            renameSync(temporaryDir, sourceDir);
          } catch (error) {
            if (!['EEXIST', 'ENOTEMPTY'].includes(error?.code)
              || !dioxusCliSourceIsReady(sourceDir, readyRevision)) {
              throw error;
            }
          }
        }
      } finally {
        try {
          if (temporaryDir) rmSync(temporaryDir, { recursive: true, force: true });
        } finally {
          releaseClaim();
        }
      }
    }
  }

  const head = run('git', ['-C', sourceDir, 'rev-parse', 'HEAD']);
  if (head !== source.rev) {
    throw new Error(`Dioxus CLI source HEAD(${head})가 exact pin(${source.rev})과 다르다: ${sourceDir}`);
  }
  verifyDioxusCliCheckoutFiles(sourceDir);

  const actual = run('git', ['-C', sourceDir, ...PATCH_DIFF_ARGS]);
  if (actual !== patch.trimEnd()) {
    throw new Error('Dioxus CLI exact checkout의 diff가 선언된 workaround와 다르다');
  }

  return sourceDir;
}

export function dioxusCliInstallArgs(source, root = ROOT, preparedSourceDir = null) {
  const sourceArgs = source.kind === 'git'
    ? ['--path', path.join(preparedSourceDir ?? dioxusCliSourceDir(source, root), 'packages', 'cli')]
    : ['--version', source.version];
  return [
    'install',
    'dioxus-cli',
    ...sourceArgs,
    '--locked',
    '--root',
    path.join(root, 'target', 'dioxus-cli'),
  ];
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const source = dioxusCliSource();
  const preparedSourceDir = prepareDioxusCliSource(source);
  const result = spawnSync('cargo', dioxusCliInstallArgs(source, ROOT, preparedSourceDir), {
    cwd: ROOT,
    stdio: 'inherit',
  });
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
}
