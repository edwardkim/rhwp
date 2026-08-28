#!/usr/bin/env node
// 셀 안 문단의 줄 나눔이 한/글이 저장해 둔 것과 같은지 잰다.
//
// HWP 는 문단마다 한/글이 확정한 줄 나눔을 `PARA_LINE_SEG` 로 저장한다. rhwp 는 그것을
// 캐시로 두고, 프레임이 계산한 값과 정확히 같을 때만 받아들인다(`resolve_stored_line_segs_in_frame`).
// 캐시가 거부되면 rhwp 가 자기 폭으로 다시 나눈다. 그 재계산이 한/글과 갈리면 행 높이가 틀어진다.
//
// **왜 셀만 재는가** — 본문 문단은 `composer::lineseg_compare` 가 이미 잰다(60개 문서
// 9,180 문단에서 99.43% 일치). 셀 안 문단은 코드 경로가 다른데 재는 도구가 없었고,
// 첫 측정에서 6,668 문단 중 90.0% 였다. 본문보다 17배 나쁘다.
//
// 불일치는 **전부 한 방향**이다 — rhwp 가 줄을 더 많이 만든다(667건 전부, 줄이 적게 나온
// 경우 0건). 셀 가용 폭을 너무 좁게 잡아 일찍 끊는다는 뜻이다.
// 근거: `mydocs/report/2026-08-28-cell-min-line-width.md`
//
// 짝짓기 한계: `dump` 의 셀 순서와 render tree 의 Cell 순서로 맞춘다. 개수가 다르면
// 그 문서는 **건너뛴다** — 억지로 맞추면 거짓 불일치가 나온다. 건너뛴 문서 수도 회귀로 센다.
//
// 사용:
//   node scripts/cell-lineseg-agreement.mjs            기준선과 비교, 회귀면 exit 1
//   node scripts/cell-lineseg-agreement.mjs --update    현재 값을 기준선으로 기록

import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = path.resolve(SCRIPT_DIR, '..');
export const BASELINE_PATH = path.join(SCRIPT_DIR, 'cell-lineseg-agreement-baseline.json');

const CELL_LINE = /셀\[(\d+)\].*?paras=(\d+) text="/;
const LINE_SEG = /ls\[(\d+)\] ts=/g;

/** `rhwp dump` 출력에서 셀 문단별 저장 줄 수를 뽑는다. */
export function storedLineCounts(dumpText) {
  const counts = [];
  let pendingCell = false;
  for (const line of dumpText.split('\n')) {
    if (CELL_LINE.test(line)) {
      pendingCell = true;
      continue;
    }
    if (pendingCell && line.includes(' p[')) {
      const n = (line.match(LINE_SEG) ?? []).length;
      if (n > 0) counts.push(n);
      pendingCell = false;
    }
  }
  return counts;
}

/** render tree JSON 에서 Cell 별 TextLine 개수를 뽑는다. */
export function renderedLineCounts(tree) {
  const counts = [];
  const walk = (node) => {
    if (node.type === 'Cell') {
      counts.push((node.children ?? []).filter((c) => c.type === 'TextLine').length);
    }
    for (const child of node.children ?? []) walk(child);
  };
  walk(tree);
  return counts;
}

/**
 * 저장/렌더 줄 수를 맞대어 집계한다.
 *
 * 글자가 없는 셀(렌더 0줄)은 세지 않는다 — 줄 나눔 판정이 없는 자리다.
 * 개수가 다른 문서는 `skipped` 로 세고 비교하지 않는다.
 */
export function tallyDocument(stored, rendered, totals) {
  if (stored.length !== rendered.length) {
    totals.skipped += 1;
    return;
  }
  for (let i = 0; i < stored.length; i += 1) {
    if (rendered[i] === 0) continue;
    totals.paragraphs += 1;
    const delta = rendered[i] - stored[i];
    if (delta === 0) totals.agree += 1;
    else {
      totals.disagree += 1;
      if (delta > 0) totals.renderedMore += 1;
      else totals.renderedFewer += 1;
    }
  }
}

export function agreementPercent(totals) {
  return totals.paragraphs === 0 ? 0 : (totals.agree / totals.paragraphs) * 100;
}

/** 일치가 줄거나 건너뛴 문서가 늘면 회귀다. */
export function compareAgreement(actual, baseline) {
  const regressions = [];
  const improvements = [];
  const now = agreementPercent(actual);
  const was = agreementPercent(baseline);
  if (now < was - 0.005) regressions.push({ what: '일치율', now: now.toFixed(2), was: was.toFixed(2) });
  else if (now > was + 0.005) improvements.push({ what: '일치율', now: now.toFixed(2), was: was.toFixed(2) });

  // 건너뛴 문서가 늘면 모수가 줄어 일치율이 착시로 오른다.
  if (actual.skipped > baseline.skipped) {
    regressions.push({ what: '짝 못 맞춘 문서', now: actual.skipped, was: baseline.skipped });
  }
  if (actual.paragraphs < baseline.paragraphs) {
    regressions.push({ what: '측정 문단', now: actual.paragraphs, was: baseline.paragraphs });
  }
  return { regressions, improvements };
}

function emptyTotals() {
  return { documents: 0, skipped: 0, paragraphs: 0, agree: 0, disagree: 0, renderedMore: 0, renderedFewer: 0 };
}

function sweep() {
  const exe = path.join(REPO_ROOT, 'target', 'release', 'rhwp');
  if (!existsSync(exe)) throw new Error(`릴리스 바이너리가 없다: ${exe}`);
  const samples = path.join(REPO_ROOT, 'samples');
  const files = readdirSync(samples)
    .filter((f) => f.endsWith('.hwp') || f.endsWith('.hwpx'))
    .sort()
    .map((f) => path.join(samples, f));

  const totals = emptyTotals();
  for (const file of files) {
    const dump = spawnSync(exe, ['dump', file], { encoding: 'utf8', maxBuffer: 512 * 1024 * 1024, timeout: 120_000 });
    if (dump.status !== 0 || !dump.stdout) continue;
    const out = mkdtempSync(path.join(tmpdir(), 'rhwp-cell-'));
    try {
      const tree = spawnSync(exe, ['export-render-tree', file, '-o', out], {
        encoding: 'utf8', maxBuffer: 512 * 1024 * 1024, timeout: 120_000,
      });
      if (tree.status !== 0) continue;
      const rendered = [];
      for (const f of readdirSync(out).filter((f) => f.endsWith('.json')).sort()) {
        rendered.push(...renderedLineCounts(JSON.parse(readFileSync(path.join(out, f), 'utf8'))));
      }
      totals.documents += 1;
      tallyDocument(storedLineCounts(dump.stdout), rendered, totals);
    } finally {
      rmSync(out, { recursive: true, force: true });
    }
  }
  return totals;
}

function report(t) {
  console.log(`  문서 ${t.documents}개 (짝 못 맞춰 건너뜀 ${t.skipped})`);
  console.log(`  셀 문단 ${t.paragraphs}개   일치 ${t.agree} = ${agreementPercent(t).toFixed(2)}%`);
  console.log(`  불일치 ${t.disagree}  (rhwp 가 더 많이 ${t.renderedMore} / 더 적게 ${t.renderedFewer})`);
}

function main() {
  const args = process.argv.slice(2);
  const totals = sweep();
  if (args.includes('--update')) {
    const doc = {
      _comment: [
        '셀 안 문단의 줄 나눔이 한/글 저장 기록과 일치하는 비율.',
        '갱신: node scripts/cell-lineseg-agreement.mjs --update',
        '이 비율은 내려갈 수 없다. 올리는 것이 목표다.',
      ],
      ...totals,
      agreementPercent: Number(agreementPercent(totals).toFixed(2)),
    };
    writeFileSync(BASELINE_PATH, `${JSON.stringify(doc, null, 2)}\n`);
    report(totals);
    console.log(`[기록] ${BASELINE_PATH}`);
    return;
  }
  const baseline = JSON.parse(readFileSync(BASELINE_PATH, 'utf8'));
  report(totals);
  const { regressions, improvements } = compareAgreement(totals, baseline);
  for (const i of improvements) console.log(`  [개선] ${i.what} ${i.was} → ${i.now}`);
  if (regressions.length > 0) {
    for (const r of regressions) console.error(`  [회귀] ${r.what} ${r.was} → ${r.now}`);
    process.exit(1);
  }
  console.log('[통과] 일치율이 기준선 아래로 내려가지 않았다.');
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))) {
  main();
}
