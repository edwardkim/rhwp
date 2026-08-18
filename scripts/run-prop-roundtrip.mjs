#!/usr/bin/env node

import { existsSync } from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import {
  cargoTestArguments,
  nextestArguments,
  resolveCasePlan,
} from './run-rust-test.mjs';
import { ROOT } from './rust-test-suite-manifest.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);

/** 배선 확인용. 이 원본이 없으면 job 이 실패한다. */
export const REQUIRED_CASES = ['prop_roundtrip_ci'];

/**
 * M04-2 / M04-3 본체. 원본이 아직 없으면 skip — CI 를 실패시키지 않는다.
 * nextest archive 정규 shard 도 tests/cases 원본을 자동 실행한다.
 */
export const OPTIONAL_CASES = ['prop_hwpx_roundtrip', 'prop_hwp5_roundtrip'];

export function caseSourcePath(caseName, root = ROOT) {
  return path.join(root, 'tests', 'cases', `${caseName}.rs`);
}

export function planPropRoundtrip(root = ROOT) {
  const run = [];
  const skipped = [];
  for (const caseName of REQUIRED_CASES) {
    if (!existsSync(caseSourcePath(caseName, root))) {
      throw new Error(`필수 property case 가 없습니다: ${caseName}`);
    }
    run.push(caseName);
  }
  for (const caseName of OPTIONAL_CASES) {
    if (existsSync(caseSourcePath(caseName, root))) {
      run.push(caseName);
    } else {
      skipped.push(caseName);
    }
  }
  return { run, skipped };
}

function usage() {
  return (
    '사용법: node scripts/run-prop-roundtrip.mjs [--print] [--cargo-test]\n'
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
    if (arguments_.length > 0) {
      throw new Error(usage().trim());
    }

    const plan = planPropRoundtrip();
    for (const caseName of plan.skipped) {
      process.stdout.write(
        `[PropRoundtrip] skip ${caseName} (tests/cases/${caseName}.rs 없음)\n`,
      );
    }

    for (const caseName of plan.run) {
      const casePlan = resolveCasePlan(caseName);
      const cargoArguments = cargoTest
        ? cargoTestArguments(casePlan)
        : nextestArguments(casePlan);
      process.stdout.write(
        `[PropRoundtrip] ${caseName} -> ${casePlan.target}` +
          `${casePlan.grouped ? `::${caseName}` : ''}\n` +
          `[PropRoundtrip] cargo ${cargoArguments.join(' ')}\n`,
      );
      if (printOnly) {
        continue;
      }
      const result = spawnSync('cargo', cargoArguments, {
        cwd: ROOT,
        stdio: 'inherit',
        shell: false,
      });
      if (result.error) {
        throw result.error;
      }
      if ((result.status ?? 1) !== 0) {
        process.exitCode = result.status ?? 1;
        break;
      }
    }
  } catch (error) {
    process.stderr.write(
      `[PropRoundtrip] 오류: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
