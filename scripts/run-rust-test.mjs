#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  buildCaseIndex,
  loadManifest,
  ROOT,
} from './rust-test-suite-manifest.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const CASE_NAME = /^[a-z][a-z0-9_]*$/;

export function resolveCase(caseName, root = ROOT) {
  if (!CASE_NAME.test(caseName)) {
    throw new Error(`잘못된 Rust test case 이름: ${caseName}`);
  }

  const suite = buildCaseIndex(loadManifest(root)).get(caseName);
  if (!suite) {
    throw new Error(`suite manifest에서 case를 찾을 수 없습니다: ${caseName}`);
  }
  return suite;
}

export function nextestArguments(caseName, suite, extraArguments = []) {
  return [
    'nextest',
    'run',
    '--test',
    suite,
    '-E',
    `test(/${caseName}/)`,
    ...extraArguments,
  ];
}

function usage() {
  return (
    '사용법: node scripts/run-rust-test.mjs [--print] <case> ' +
    '[-- <cargo-nextest 인자>]\n'
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    const arguments_ = process.argv.slice(2);
    const printOnly = arguments_[0] === '--print';
    if (printOnly) {
      arguments_.shift();
    }

    const caseName = arguments_.shift();
    if (!caseName) {
      throw new Error(usage().trim());
    }
    if (arguments_.length > 0 && arguments_[0] !== '--') {
      throw new Error(usage().trim());
    }
    if (arguments_[0] === '--') {
      arguments_.shift();
    }

    const suite = resolveCase(caseName);
    const cargoArguments = nextestArguments(caseName, suite, arguments_);
    process.stdout.write(
      `[RustTestSuite] ${caseName} -> ${suite}\n` +
        `[RustTestSuite] cargo ${cargoArguments.join(' ')}\n`,
    );

    if (!printOnly) {
      const result = spawnSync('cargo', cargoArguments, {
        cwd: ROOT,
        stdio: 'inherit',
        shell: false,
      });
      if (result.error) {
        throw result.error;
      }
      process.exitCode = result.status ?? 1;
    }
  } catch (error) {
    process.stderr.write(
      `[RustTestSuite] 오류: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
