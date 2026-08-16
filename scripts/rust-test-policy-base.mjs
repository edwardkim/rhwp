import { execFileSync } from 'node:child_process';

function git(root, args) {
  try {
    return execFileSync('git', args, {
      cwd: root,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    });
  } catch (error) {
    const detail = error?.stderr?.toString().trim();
    throw new Error(
      `PR base 정책을 읽는 git 명령이 실패했습니다: git ${args.join(' ')}` +
        (detail ? `\n${detail}` : ''),
    );
  }
}

export function parseBaseRefArgument(args) {
  if (args.length === 0) {
    return null;
  }
  if (args.length !== 2 || args[0] !== '--base-ref' || args[1].length === 0) {
    throw new Error('--base-ref <Git ref> 형식이 필요합니다.');
  }
  return args[1];
}

export function readJsonAtRef(
  root,
  ref,
  relativePath,
  { optional = false } = {},
) {
  try {
    return JSON.parse(git(root, ['show', `${ref}:${relativePath}`]));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (
      optional &&
      (message.includes('exists on disk, but not in') ||
        message.includes('does not exist in'))
    ) {
      return null;
    }
    throw error;
  }
}

export function renamedFilesSince(root, ref, pathspecs) {
  const output = git(root, [
    '-c',
    'core.quotepath=false',
    'diff',
    '--name-status',
    '--find-renames=80%',
    ref,
    'HEAD',
    '--',
    ...pathspecs,
  ]);
  const renamed = new Map();
  for (const line of output.split('\n')) {
    if (!line.startsWith('R')) {
      continue;
    }
    const [status, previous, current] = line.split('\t');
    if (/^R\d+$/.test(status) && previous && current) {
      renamed.set(current, previous);
    }
  }
  return renamed;
}
