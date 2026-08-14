#!/usr/bin/env node
/**
 * gym/tools/from_e2e.mjs — studio e2e → gym 과제 어댑터 (온램프 #3, 이슈 #4756)
 *
 * 스튜디오 기여자가 e2e 파일에 `export const gymContract = {...}` 한 조각을 달면,
 * 이 도구가 그 계약에서 CLI 로 채점 가능한 gym 과제를 기계 생성한다:
 *   tasks/<ID>.json + reference/<ID>.json + assets/<ID>-edit.csv
 *
 * 왜 손이 거의 안 가나: 편집 CSV 를 사람이 쓰지 않는다. `rhwp chart-to-csv` 로 실제
 * 차트를 뽑아(계열명·라벨·다른 값이 정확히 맞는 CSV) 계약이 지정한 한 칸만 바꾼다.
 * 그래서 계약은 "무슨 칸을 무슨 값으로" 만 말하면 되고, 형태 맞추기는 어댑터가
 * rhwp 자신에게 시킨다 — gym 의 라이브 오라클과 같은 원리.
 *
 * 설계 함정 둘(이 세션에서 실측으로 확인해 회피함):
 *   ① e2e 파일은 top-level 에서 runTest 를 실행한다 → `import` 하면 브라우저가
 *      기동된다. 그래서 파일을 실행하지 않고 gymContract '객체 리터럴만' 정적
 *      파싱해 평가한다.
 *   ② `chart-to-csv --json` 은 순수 JSON 이다(출처 머리줄 없음) → 그대로 파싱하고
 *      `charts[0].csv` 를 쓴다.
 *
 * 사용:
 *   node gym/tools/from_e2e.mjs \
 *     --e2e rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs \
 *     --pack studio-e2e --id SE01 --bin target/debug/rhwp
 *
 * 생성 뒤 기존 게이트로 스스로 왕복 검증:
 *   python gym/tools/build_baseline.py --agent baseline --pack studio-e2e --bin <bin>
 *   python gym/score.py               --agent baseline --pack studio-e2e --bin <bin>
 *   → studio-e2e 3/3 이면 계약이 CLI 로 충실히 왕복함이 증명된다.
 */
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import path from 'node:path';

function arg(name, def) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 ? process.argv[i + 1] : def;
}

const ROOT = process.cwd();
const e2ePath = arg('e2e');
const packId = arg('pack', 'studio-e2e');
const taskId = arg('id');
const bin = path.resolve(ROOT, arg('bin', 'target/debug/rhwp'));
if (!e2ePath || !taskId) {
  console.error('필수: --e2e <경로> --id <과제ID> [--pack studio-e2e] [--bin target/debug/rhwp]');
  process.exit(2);
}
const packDir = path.join(ROOT, 'gym', 'packs', packId);

// 1) 계약을 '정적 파싱' 으로 읽는다 — 파일을 import/실행하지 않는다(함정 ①).
function readContract(file) {
  const src = readFileSync(path.resolve(ROOT, file), 'utf8');
  const m = src.match(/export\s+const\s+gymContract\s*=\s*\{/);
  if (!m) throw new Error(`${file} 에 'export const gymContract' 가 없다`);
  const open = src.indexOf('{', m.index);
  let depth = 0, end = -1;
  for (let j = open; j < src.length; j++) {
    if (src[j] === '{') depth++;
    else if (src[j] === '}' && --depth === 0) { end = j; break; }
  }
  if (end < 0) throw new Error('gymContract 객체 리터럴의 닫는 괄호를 못 찾음');
  const literal = src.slice(open, end + 1);
  // 리터럴 하나만 평가한다 — 파일 본문(runTest)은 실행되지 않는다.
  return Function(`"use strict"; return (${literal});`)();
}
const c = readContract(e2ePath);
for (const k of ['sample', 'chart', 'edit']) {
  if (c[k] === undefined) throw new Error(`gymContract.${k} 가 없다`);
}

// 2) 실제 차트를 CSV 로 뽑는다 — chart-to-csv --json 은 순수 JSON(함정 ②).
const sampleRel = path.posix.join('samples', c.sample);
const env = JSON.parse(execFileSync(
  bin, ['chart-to-csv', sampleRel, '--chart', String(c.chart), '--json'],
  { cwd: ROOT, encoding: 'utf8' }));
const baseCsv = env.charts[0].csv;

// 3) 계약이 지정한 한 칸만 교체. chart-to-csv 규약: 행=카테고리, 열=계열.
//    (values 는 숫자, 이름은 콤마 없음 → 단순 split 안전.)
const eol = baseCsv.includes('\r\n') ? '\r\n' : '\n';
const rows = baseCsv.replace(/\r\n/g, '\n').replace(/\n$/, '').split('\n').map(r => r.split(','));
const dataRow = rows[1 + c.edit.point];        // 0행 = 헤더(계열명)
const col = 1 + c.edit.series;                 // 0열 = 카테고리 라벨
if (!dataRow) throw new Error(`point ${c.edit.point} 데이터 행이 없다`);
if (dataRow[col] !== String(c.edit.from)) {
  throw new Error(`계약 불일치: (계열 ${c.edit.series}, 값 ${c.edit.point}) 현재 `
    + `'${dataRow[col]}' ≠ from '${c.edit.from}' — 샘플이 바뀌었다`);
}
dataRow[col] = String(c.edit.to);
const editCsv = rows.map(r => r.join(',')).join(eol) + eol;

// 4) 산출: 자산 CSV + 과제 + 기준풀이. 과제·기준은 순수 템플릿(로직 0).
for (const d of ['assets', 'tasks', 'reference']) mkdirSync(path.join(packDir, d), { recursive: true });
const csvAsset = `gym/packs/${packId}/assets/${taskId}-edit.csv`;
writeFileSync(path.join(ROOT, csvAsset), editCsv, 'utf8');

const task = {
  id: taskId,
  tier: 3,
  title: `차트 데이터 편집 왕복 (studio ${path.basename(e2ePath)} 파생)`,
  input: sampleRel,
  instructions:
    `차트 ${c.chart}의 (계열 ${c.edit.series}, 값 ${c.edit.point}) 원본 ${c.edit.from} 을 `
    + `'${c.edit.to}' 로 바꿔 out.hwp 로 저장하라. 원본 크기(계열 수·값 개수·계열명·`
    + `카테고리 라벨)는 그대로 두어야 한다. 힌트: chart-to-csv 로 뽑아 그 칸만 고치고 `
    + `csv-to-chart 로 되넣어라(-o out.hwp).`,
  submit: { kind: 'artifact', files: ['out.hwp'] },
  checks: [
    { name: '산출물 존재', op: 'file_exists', file: 'out.hwp', minBytes: 1 },
    { name: '원본과 다름 (무편집 복사 거부)', op: 'differs_from_input', file: 'out.hwp' },
    {
      name: `첫 값이 이미 ${c.edit.to} (센티넬 재적용이 무변경)`,
      op: 'value_eq', path: 'changedCount', value: 0,
      cmd: ['csv-to-chart', '{file:out.hwp}', '--csv', csvAsset,
            '--chart', String(c.chart), '--dry-run', '--json'],
    },
  ],
};
const reference = {
  id: taskId,
  steps: [{ run: ['csv-to-chart', '{input}', '--csv', csvAsset,
                  '--chart', String(c.chart), '-o', '{sub:out.hwp}', '--json'] }],
};
const dump = (obj) => JSON.stringify(obj, null, 2) + '\n';
writeFileSync(path.join(packDir, 'tasks', `${taskId}.json`), dump(task), 'utf8');
writeFileSync(path.join(packDir, 'reference', `${taskId}.json`), dump(reference), 'utf8');

console.log(`생성: ${taskId} — assets/${taskId}-edit.csv · tasks/${taskId}.json · reference/${taskId}.json`);
console.log(`검증: python gym/tools/build_baseline.py --agent baseline --pack ${packId} --bin <bin> && python gym/score.py --agent baseline --pack ${packId} --bin <bin>`);
