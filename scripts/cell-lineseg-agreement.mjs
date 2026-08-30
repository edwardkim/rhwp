#!/usr/bin/env node
// 셀 안 문단의 줄 나눔이 한/글이 저장해 둔 것과 같은지 잰다.
//
// HWP 는 문단마다 한/글이 확정한 줄 나눔을 `PARA_LINE_SEG` 로 저장한다. rhwp 는 그것을
// 캐시로 두고, 프레임이 계산한 값과 정확히 같을 때만 받아들인다(`resolve_stored_line_segs_in_frame`).
// 캐시가 거부되면 rhwp 가 자기 폭으로 다시 나눈다. 그 재계산이 한/글과 갈리면 행 높이가 틀어진다.
//
// **왜 셀만 재는가** — 본문 문단은 `composer::lineseg_compare` 가 이미 잰다. 셀 안
// 문단은 코드 경로가 달라 별도 도구가 필요했다.
//
// **짝짓기** — 종전에는 dump 셀 순서와 render tree Cell 순서를 순번으로 맞추고, 개수가
// 다르면 문서를 통째로 건너뛰었다(600 문서 중 231 건너뜀 — 셀 구조가 복잡한, 즉 개선과
// 회귀가 실제로 일어나는 문서가 사각에 몰렸다). 지금은 셀을 (행, 열, 텍스트 접두사)
// 내용 키로 짝짓는다. 못 짝지은 셀만 `unpaired*` 로 세고 문서는 버리지 않는다.
// 같은 키가 여럿이면(빈 셀 격자 등) 순서대로 맞춘다.
//
// **v3 분류 (#6363)** — 기록 없는 문단(저장 줄수 0)은 불일치가 아니라 `noStoredRecord` 다.
// 쪽 나눔 조각은 통짜 저장 셀과 첫 조각만 비교하지 않는다: `hdr=true`(제목 행 반복)는
// 각 조각을 같은 저장 값과 개별 비교하고, `hdr=false` 는 같은 (행,열) 조각을 합산한다.
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

const CELL_LINE = /셀\[\d+\] r=(\d+),c=(\d+) .*?hdr=(true|false) .*?text="([^"]*)"/;
const TABLE_LINE = /(?:내부표|표): /;
const PARA_LINE = /p\[\d+\]/;
const LINE_SEG = /ls\[\d+\] ts=/g;
const PREFIX = /^\s*\[\d+\]( *)/;

/** 짝짓기 키의 텍스트 부분 — 공백·구분 기호를 걷고 앞 12자만 쓴다. */
export function textKey(text) {
  return text.replace(/[\s|]/g, '').slice(0, 12);
}

/**
 * `rhwp dump` 출력에서 셀별 (행, 열, 텍스트, 저장 줄 수)를 뽑는다.
 *
 * 중첩 표가 셀 문단 사이에 끼므로 들여쓰기 스택으로 소유를 판정한다 — `ls` 줄은
 * 자기보다 얕은 가장 가까운 셀의 것이고, 같은 깊이의 새 항목은 그 셀을 닫는다.
 */
export function storedCells(dumpText) {
  const stack = []; // { indent, cell }
  const cells = [];
  for (const line of dumpText.split('\n')) {
    const m = PREFIX.exec(line);
    if (!m) continue;
    const indent = m[1].length;
    const cellMatch = CELL_LINE.exec(line);
    if (cellMatch) {
      while (stack.length > 0 && stack[stack.length - 1].indent >= indent) stack.pop();
      const cell = {
        row: Number(cellMatch[1]),
        col: Number(cellMatch[2]),
        header: cellMatch[3] === 'true',
        text: cellMatch[4],
        lines: 0,
      };
      cells.push(cell);
      stack.push({ indent, cell });
      continue;
    }
    if (LINE_SEG.test(line)) {
      LINE_SEG.lastIndex = 0;
      const owner = [...stack].reverse().find((s) => s.indent < indent);
      if (owner) owner.cell.lines += (line.match(LINE_SEG) ?? []).length;
      continue;
    }
    if (TABLE_LINE.test(line) || PARA_LINE.test(line)) {
      while (stack.length > 0 && stack[stack.length - 1].indent >= indent) stack.pop();
    }
  }
  return cells;
}

/** render tree JSON 에서 Cell 별 (행, 열, 텍스트, TextLine 수)를 뽑는다. */
export function renderedCells(tree) {
  const cells = [];
  const walk = (node) => {
    if (node.type === 'Cell') {
      const lines = (node.children ?? []).filter((c) => c.type === 'TextLine');
      const text = lines
        .map((l) => (l.children ?? []).map((r) => r.text ?? '').join(''))
        .join('|');
      cells.push({
        row: node.row ?? 0,
        col: node.col ?? 0,
        text,
        lines: lines.length,
      });
    }
    for (const child of node.children ?? []) walk(child);
  };
  walk(tree);
  return cells;
}

function cellKey(cell) {
  return `${cell.row}:${cell.col}:${textKey(cell.text)}`;
}

function posKey(cell) {
  return `${cell.row}:${cell.col}`;
}

function judgePair(storedLines, renderedLines, totals) {
  if (renderedLines === 0) return;
  if (storedLines === 0) {
    totals.noStoredRecord += 1;
    return;
  }
  totals.cells += 1;
  const delta = renderedLines - storedLines;
  if (delta === 0) totals.agree += 1;
  else {
    totals.disagree += 1;
    if (delta > 0) totals.renderedMore += 1;
    else totals.renderedFewer += 1;
  }
}

/**
 * 문서에 (행,열)이 유일한 `hdr=false` 저장 셀이면, 쪽 나눔으로 쪼개진 렌더
 * 조각을 합산한다. 첫 조각의 텍스트를 남겨 저장 접두사와 내용 키가 맞는다.
 * (행,열)이 여러 표에 반복되면 합치지 않는다 — #6354 사각을 되돌리지 않기 위함.
 */
export function mergePageSplitFragments(stored, rendered) {
  const storedAtPos = new Map();
  for (const cell of stored) {
    const pos = posKey(cell);
    if (!storedAtPos.has(pos)) storedAtPos.set(pos, []);
    storedAtPos.get(pos).push(cell);
  }
  const mergeable = new Set();
  for (const [pos, list] of storedAtPos) {
    if (list.length === 1 && !list[0].header) mergeable.add(pos);
  }
  const mergedAtPos = new Map();
  const out = [];
  for (const cell of rendered) {
    const pos = posKey(cell);
    if (!mergeable.has(pos)) {
      out.push(cell);
      continue;
    }
    const existing = mergedAtPos.get(pos);
    if (!existing) {
      const merged = { row: cell.row, col: cell.col, text: cell.text, lines: cell.lines };
      mergedAtPos.set(pos, merged);
      out.push(merged);
    } else {
      existing.lines += cell.lines;
    }
  }
  return out;
}

/**
 * 내용 키로 셀을 짝지어 집계한다.
 *
 * 렌더 0줄 셀은 판정하지 않는다(줄 나눔이 없는 자리) — 짝짓기에는 참여시켜
 * 다른 셀의 상대를 빼앗지 않게 한다. 못 짝지은 셀은 양쪽 각각 센다.
 *
 * 저장 줄수 0 은 비교 모수에서 빼고 `noStoredRecord` 로 센다. 같은 키에 저장 1개·
 * 렌더 여러 개이면 `hdr=true` 는 조각마다 비교하고 `hdr=false` 는 줄 수를 합산한다.
 */
export function tallyDocument(stored, rendered, totals) {
  const renderedForPairing = mergePageSplitFragments(stored, rendered);
  const buckets = new Map();
  for (const cell of stored) {
    const key = cellKey(cell);
    if (!buckets.has(key)) buckets.set(key, []);
    buckets.get(key).push(cell);
  }
  const renderedBuckets = new Map();
  for (const cell of renderedForPairing) {
    const key = cellKey(cell);
    if (!renderedBuckets.has(key)) renderedBuckets.set(key, []);
    renderedBuckets.get(key).push(cell);
  }

  const keys = new Set([...buckets.keys(), ...renderedBuckets.keys()]);
  for (const key of keys) {
    const storedQueue = buckets.get(key) ?? [];
    const renderedQueue = renderedBuckets.get(key) ?? [];
    if (storedQueue.length === 1 && renderedQueue.length > 1) {
      const storedCell = storedQueue[0];
      if (storedCell.header) {
        for (const renderedCell of renderedQueue) {
          judgePair(storedCell.lines, renderedCell.lines, totals);
        }
      } else if (renderedQueue.some((cell) => cell.lines > 0)) {
        const summed = renderedQueue.reduce((n, cell) => n + cell.lines, 0);
        judgePair(storedCell.lines, summed, totals);
      }
      storedQueue.length = 0;
      continue;
    }
    while (storedQueue.length > 0 && renderedQueue.length > 0) {
      const storedCell = storedQueue.shift();
      const renderedCell = renderedQueue.shift();
      judgePair(storedCell.lines, renderedCell.lines, totals);
    }
    totals.unpairedRendered += renderedQueue.length;
  }
  for (const queue of buckets.values()) totals.unpairedStored += queue.length;
}

export function agreementPercent(totals) {
  return totals.cells === 0 ? 0 : (totals.agree / totals.cells) * 100;
}

/** 일치가 줄거나, 못 짝지은 셀이 늘거나, 측정 모수가 줄거나, 기록 없음이 늘면 회귀다. */
export function compareAgreement(actual, baseline) {
  const regressions = [];
  const improvements = [];
  const now = agreementPercent(actual);
  const was = agreementPercent(baseline);
  if (now < was - 0.005) regressions.push({ what: '일치율', now: now.toFixed(2), was: was.toFixed(2) });
  else if (now > was + 0.005) improvements.push({ what: '일치율', now: now.toFixed(2), was: was.toFixed(2) });

  const unpairedNow = actual.unpairedStored + actual.unpairedRendered;
  const unpairedWas = baseline.unpairedStored + baseline.unpairedRendered;
  if (unpairedNow > unpairedWas) {
    regressions.push({ what: '못 짝지은 셀', now: unpairedNow, was: unpairedWas });
  }
  // 모수가 줄면 일치율이 착시로 오를 수 있다 — "안 재서 통과"를 막는다.
  if (actual.cells < baseline.cells) {
    regressions.push({ what: '측정 셀', now: actual.cells, was: baseline.cells });
  }
  const noStoredNow = actual.noStoredRecord ?? 0;
  const noStoredWas = baseline.noStoredRecord ?? 0;
  if (noStoredNow > noStoredWas) {
    regressions.push({ what: '기록 없는 셀', now: noStoredNow, was: noStoredWas });
  }
  return { regressions, improvements };
}

function emptyTotals() {
  return {
    documents: 0,
    unpairedStored: 0,
    unpairedRendered: 0,
    noStoredRecord: 0,
    cells: 0,
    agree: 0,
    disagree: 0,
    renderedMore: 0,
    renderedFewer: 0,
  };
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
        rendered.push(...renderedCells(JSON.parse(readFileSync(path.join(out, f), 'utf8'))));
      }
      totals.documents += 1;
      tallyDocument(storedCells(dump.stdout), rendered, totals);
    } finally {
      rmSync(out, { recursive: true, force: true });
    }
  }
  return totals;
}

function report(t) {
  console.log(`  문서 ${t.documents}개 (못 짝지은 셀: 저장 ${t.unpairedStored} / 렌더 ${t.unpairedRendered})`);
  console.log(`  기록 없음 ${t.noStoredRecord}개 (비교 모수에서 제외)`);
  console.log(`  측정 셀 ${t.cells}개   일치 ${t.agree} = ${agreementPercent(t).toFixed(2)}%`);
  console.log(`  불일치 ${t.disagree}  (rhwp 가 더 많이 ${t.renderedMore} / 더 적게 ${t.renderedFewer})`);
}

function main() {
  const args = process.argv.slice(2);
  const totals = sweep();
  if (args.includes('--update')) {
    const doc = {
      _comment: [
        '셀 안 문단의 줄 나눔이 한/글 저장 기록과 일치하는 비율 (내용 키 짝짓기, v3).',
        '비교 모수는 저장 줄수가 있는 셀만. 기록 없음은 noStoredRecord.',
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
