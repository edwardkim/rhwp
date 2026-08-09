#!/usr/bin/env node
/** Python 도구를 Windows와 POSIX에서 같은 npm script로 실행한다. */

import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const [script, ...args] = process.argv.slice(2);

if (!script || script === '--help' || script === '-h') {
  console.error('사용법: python_runner.mjs <python-script> [...args]');
  process.exit(script ? 0 : 2);
}

const python = process.env.PYTHON || (process.platform === 'win32' ? 'python' : 'python3');
const result = spawnSync(python, [resolve(here, script), ...args], { stdio: 'inherit' });

if (result.error) {
  console.error(`Python 실행 실패 (${python}): ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status ?? 1);
