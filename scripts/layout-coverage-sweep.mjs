#!/usr/bin/env node
// 조판 커버리지 측정 — `samples/` 전수에 `rhwp layout-anomaly` 를 돌려
// **한글 오라클 없이 판정 가능한** 이상 신호를 세고, 기준선과 맞댄다.
//
// 왜 이 축인가 — 한글 대비 쪽수 일치율(#5585)은 Windows + 한글 2022 가 있어야 잰다.
// 반면 아래 신호들은 그 자체로 결함이다. 용지 밖에 그린 글자는 어떤 기준으로도 안 보이고,
// 흐름 요소끼리 겹치면 어떤 기준으로도 틀렸다. 그래서 오라클 없이도 개선을 측정할 수 있다.
//
//   off_canvas    용지 상자 밖 또는 y<0        — 보이지 않는 콘텐츠
//   overflow      본문 여백(Body) 밖
//   overlap       겹치면 안 되는 흐름 요소끼리 겹침
//   text_overlap  텍스트 런 bbox 교차
//   empty_page    콘텐츠 없는 중간 쪽
//
// 커버리지 = 신호가 하나도 없는 문서(CLEAN)의 비율.
//
// 사용:
//   node scripts/layout-coverage-sweep.mjs              기준선과 비교, 회귀면 exit 1
//   node scripts/layout-coverage-sweep.mjs --update     현재 값을 기준선으로 기록
//   node scripts/layout-coverage-sweep.mjs --ndjson <경로>  이미 뽑아 둔 결과로 비교

import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = path.resolve(SCRIPT_DIR, '..');
export const BASELINE_PATH = path.join(SCRIPT_DIR, 'layout-coverage-baseline.json');

/** 신호 종류와 NDJSON 필드 이름. 문서 수와 노드 수를 둘 다 센다. */
export const SIGNALS = [
  ['off_canvas', 'offCanvasCount'],
  ['overflow', 'overflowCount'],
  ['overlap', 'overlapCount'],
  ['text_overlap', 'textOverlapCount'],
  ['empty_page', 'emptyPageCount'],
];

/**
 * `layout-anomaly --batch --json` 의 NDJSON 을 집계한다.
 *
 * 파일별 오류 레코드는 신호 필드가 없다 — 문서 수에서 빼고 `errors` 로 따로 센다.
 * 오류를 CLEAN 으로 세면 파싱이 깨질수록 커버리지가 올라가는 지표가 된다.
 */
export function tallySweep(ndjson) {
  const totals = { documents: 0, clean: 0, errors: 0, signals: {} };
  for (const [name] of SIGNALS) totals.signals[name] = { documents: 0, nodes: 0 };

  for (const line of ndjson.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed.startsWith('{')) continue;
    let record;
    try {
      record = JSON.parse(trimmed);
    } catch {
      continue;
    }
    if (typeof record.offCanvasCount !== 'number') {
      totals.errors += 1;
      continue;
    }
    totals.documents += 1;
    if (!record.hasSignal) totals.clean += 1;
    for (const [name, field] of SIGNALS) {
      const n = record[field] ?? 0;
      if (n > 0) {
        totals.signals[name].documents += 1;
        totals.signals[name].nodes += n;
      }
    }
  }
  return totals;
}

/** CLEAN 비율(%). 문서가 0 이면 0 을 돌려준다 — 나눗셈으로 NaN 을 만들지 않는다. */
export function coveragePercent(totals) {
  return totals.documents === 0 ? 0 : (totals.clean / totals.documents) * 100;
}

/**
 * 기준선과 맞대어 회귀와 개선을 가른다.
 *
 * CLEAN 문서 수가 줄거나, 어떤 신호의 문서 수가 늘면 회귀다. 노드 수는 참고값으로만
 * 싣는다 — 한 문서 안에서 노드가 늘고 주는 것은 문서 판정을 바꾸지 않는다.
 */
export function compareCoverage(actual, baseline) {
  const regressions = [];
  const improvements = [];

  if (actual.clean < baseline.clean) {
    regressions.push({ what: 'CLEAN 문서', now: actual.clean, was: baseline.clean });
  } else if (actual.clean > baseline.clean) {
    improvements.push({ what: 'CLEAN 문서', now: actual.clean, was: baseline.clean });
  }

  for (const [name] of SIGNALS) {
    const now = actual.signals[name]?.documents ?? 0;
    const was = baseline.signals[name]?.documents ?? 0;
    if (now > was) regressions.push({ what: `${name} 문서`, now, was });
    else if (now < was) improvements.push({ what: `${name} 문서`, now, was });
  }

  // 파싱 오류가 늘면 문서가 모수에서 빠져 커버리지가 착시로 올라간다.
  if (actual.errors > baseline.errors) {
    regressions.push({ what: '파싱 오류', now: actual.errors, was: baseline.errors });
  }
  return { regressions, improvements };
}

function runSweep() {
  const exe = path.join(REPO_ROOT, 'target', 'release', 'rhwp');
  if (!existsSync(exe)) {
    throw new Error(`릴리스 바이너리가 없다: ${exe}\n먼저 cargo build --release --bin rhwp`);
  }
  const result = spawnSync(exe, ['layout-anomaly', '--batch', 'samples', '--json'], {
    cwd: REPO_ROOT,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  return result.stdout;
}

function report(totals) {
  console.log(`  문서 ${totals.documents}개 (파싱 오류 ${totals.errors})`);
  console.log(`  CLEAN ${totals.clean}  =  ${coveragePercent(totals).toFixed(1)}%`);
  for (const [name] of SIGNALS) {
    const s = totals.signals[name];
    console.log(`    ${name.padEnd(13)} 문서 ${String(s.documents).padStart(4)}  노드 ${s.nodes}`);
  }
}

function main() {
  const args = process.argv.slice(2);
  const ndjsonAt = args.indexOf('--ndjson');
  const ndjson = ndjsonAt >= 0 ? readFileSync(args[ndjsonAt + 1], 'utf8') : runSweep();
  const totals = tallySweep(ndjson);

  if (args.includes('--update')) {
    const doc = {
      _comment: [
        '조판 커버리지 기준선 — CLEAN 은 layout-anomaly 신호가 하나도 없는 문서 수.',
        '갱신: node scripts/layout-coverage-sweep.mjs --update',
        '이 수는 내려갈 수 없다. 올리는 것이 목표다.',
      ],
      ...totals,
      coveragePercent: Number(coveragePercent(totals).toFixed(2)),
    };
    writeFileSync(BASELINE_PATH, `${JSON.stringify(doc, null, 2)}\n`);
    report(totals);
    console.log(`[기록] ${BASELINE_PATH}`);
    return;
  }

  const baseline = JSON.parse(readFileSync(BASELINE_PATH, 'utf8'));
  report(totals);
  const delta = coveragePercent(totals) - coveragePercent(baseline);
  console.log(`  기준선 대비 ${delta >= 0 ? '+' : ''}${delta.toFixed(2)}%p (CLEAN ${baseline.clean} → ${totals.clean})`);

  const { regressions, improvements } = compareCoverage(totals, baseline);
  for (const i of improvements) console.log(`  [개선] ${i.what} ${i.was} → ${i.now}`);
  if (regressions.length > 0) {
    for (const r of regressions) console.error(`  [회귀] ${r.what} ${r.was} → ${r.now}`);
    console.error('개선했다면 --update 로 기준선을 올려라. 아니면 회귀를 고쳐라.');
    process.exit(1);
  }
  console.log('[통과] 커버리지가 기준선 아래로 내려가지 않았다.');
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))) {
  main();
}
