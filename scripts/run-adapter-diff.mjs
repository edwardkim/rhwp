#!/usr/bin/env node

import { existsSync, readFileSync } from 'node:fs';
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
const MOD_RS = path.join('src', 'render_backend', 'mod.rs');

/** 배선 확인용. 이 원본이 없으면 job 이 실패한다. */
export const REQUIRED_CASES = ['adapter_diff'];

/**
 * 어댑터 원본. 없으면 skip — CI 를 실패시키지 않는다.
 * svg/null/trace 는 devel 에 있고 필수다.
 */
export const ADAPTERS = [
  {
    name: 'svg',
    source: path.join('src', 'render_backend', 'svg_adapter.rs'),
    export: 'SvgBackend',
    required: true,
    cfg: null,
  },
  {
    name: 'null',
    source: path.join('src', 'render_backend', 'backends.rs'),
    export: 'NullBackend',
    required: true,
    cfg: null,
  },
  {
    name: 'trace',
    source: path.join('src', 'render_backend', 'backends.rs'),
    export: 'TraceBackend',
    required: true,
    cfg: null,
  },
  {
    name: 'png',
    source: path.join('src', 'render_backend', 'png_adapter.rs'),
    export: 'PngBackend',
    required: false,
    cfg: 'rhwp_has_png_backend',
  },
  {
    name: 'skia',
    source: path.join('src', 'render_backend', 'skia_adapter.rs'),
    export: 'SkiaBackend',
    required: false,
    cfg: 'rhwp_has_skia_backend',
  },
];

export function adapterSourcePath(rel, root = ROOT) {
  return path.join(root, rel);
}

function readUtf8(filePath) {
  return readFileSync(filePath, 'utf8');
}

export function isExported(modRs, adapter) {
  if (modRs.includes(adapter.export)) {
    return true;
  }
  const stem = path.basename(adapter.source, '.rs');
  return modRs.includes(`mod ${stem}`);
}

export function planAdapterDiff(root = ROOT) {
  const present = [];
  const skipped = [];
  const cfgs = [];
  const modPath = path.join(root, MOD_RS);
  const modRs = existsSync(modPath) ? readUtf8(modPath) : '';

  for (const adapter of ADAPTERS) {
    const sourcePath = adapterSourcePath(adapter.source, root);
    if (!existsSync(sourcePath)) {
      if (adapter.required) {
        throw new Error(`필수 어댑터 원본이 없습니다: ${adapter.name}`);
      }
      skipped.push({ name: adapter.name, reason: 'missing' });
      continue;
    }
    if (!isExported(modRs, adapter)) {
      if (adapter.required) {
        throw new Error(`필수 어댑터가 미등록입니다: ${adapter.name}`);
      }
      skipped.push({ name: adapter.name, reason: 'unexported' });
      continue;
    }
    present.push(adapter.name);
    if (adapter.cfg) {
      cfgs.push(adapter.cfg);
    }
  }

  return { present, skipped, cfgs, run: REQUIRED_CASES };
}

export function rustflagsFor(cfgs, existing = process.env.RUSTFLAGS ?? '') {
  const extra = cfgs.map((cfg) => `--cfg ${cfg}`).join(' ');
  return [existing.trim(), extra].filter((part) => part.length > 0).join(' ');
}

function usage() {
  return '사용법: node scripts/run-adapter-diff.mjs [--print] [--cargo-test]\n';
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

    const plan = planAdapterDiff();
    process.stdout.write(
      `[AdapterDiff] present=${plan.present.join(',')} ` +
        `skipped=${plan.skipped.map((item) => `${item.name}:${item.reason}`).join(',') || '-'}\n`,
    );
    for (const item of plan.skipped) {
      process.stdout.write(
        `[AdapterDiff] skip ${item.name} (${item.reason})\n`,
      );
    }

    const rustflags = rustflagsFor(plan.cfgs);
    if (rustflags) {
      process.stdout.write(`[AdapterDiff] RUSTFLAGS=${rustflags}\n`);
    }

    for (const caseName of plan.run) {
      const casePlan = resolveCasePlan(caseName);
      const cargoArguments = cargoTest
        ? cargoTestArguments(casePlan)
        : nextestArguments(casePlan);
      process.stdout.write(
        `[AdapterDiff] ${caseName} -> ${casePlan.target}` +
          `${casePlan.grouped ? `::${caseName}` : ''}\n` +
          `[AdapterDiff] cargo ${cargoArguments.join(' ')}\n`,
      );
      if (printOnly) {
        continue;
      }
      const result = spawnSync('cargo', cargoArguments, {
        cwd: ROOT,
        stdio: 'inherit',
        shell: false,
        env: rustflags ? { ...process.env, RUSTFLAGS: rustflags } : process.env,
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
      `[AdapterDiff] 오류: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
