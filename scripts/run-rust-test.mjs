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

export function resolveCasePlan(caseName, root = ROOT) {
  if (!CASE_NAME.test(caseName)) {
    throw new Error(`잘못된 Rust test case 이름: ${caseName}`);
  }
  const manifest = loadManifest(root);
  const target = buildCaseIndex(manifest).get(caseName);
  if (!target) {
    throw new Error(`suite manifest에서 case를 찾을 수 없습니다: ${caseName}`);
  }
  return {
    caseName,
    target,
    grouped: Object.hasOwn(manifest.suites, target),
  };
}

export function resolveCase(caseName, root = ROOT) {
  return resolveCasePlan(caseName, root).target;
}

export function nextestArguments(plan, extraArguments = []) {
  const arguments_ = ['nextest', 'run', '--test', plan.target];
  if (plan.grouped) {
    arguments_.push('-E', `test(/(^|::)${plan.caseName}::/)`);
  }
  return [...arguments_, ...extraArguments];
}

export function cargoTestArguments(plan, extraArguments = []) {
  const arguments_ = ['test', ...extraArguments, '--test', plan.target];
  if (plan.grouped) {
    arguments_.push(`${plan.caseName}::`);
  }
  return arguments_;
}

function usage() {
  return (
    '사용법: node scripts/run-rust-test.mjs [--print] [--cargo-test] <case> ' +
    '[-- <cargo 인자>]\n'
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  try {
    const arguments_ = process.argv.slice(2);
    let printOnly = false;
    let cargoTest = false;
    while (arguments_[0] === '--print' || arguments_[0] === '--cargo-test') {
      const option = arguments_.shift();
      printOnly ||= option === '--print';
      cargoTest ||= option === '--cargo-test';
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

    const plan = resolveCasePlan(caseName);
    const cargoArguments = cargoTest
      ? cargoTestArguments(plan, arguments_)
      : nextestArguments(plan, arguments_);
    process.stdout.write(
      `[RustTestSuite] ${caseName} -> ${plan.target}` +
        `${plan.grouped ? `::${caseName}` : ''}\n` +
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
