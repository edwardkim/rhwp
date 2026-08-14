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
 * 사용:
 *   node gym/tools/from_e2e.mjs \
 *     --e2e rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs \
 *     --pack studio-e2e --id ST01 --bin target/debug/rhwp
 */
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function arg(name, def) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 ? process.argv[i + 1] : def;
}

/**
 * e2e 파일을 실행하지 않고 gymContract의 제한된 객체 리터럴만 읽는다.
 *
 * 허용 값은 중첩 객체, 문자열, 숫자뿐이다. 식·함수·템플릿·배열·식별자는 거부한다.
 * 외부 PR의 e2e 파일은 검토 환경에서 신뢰할 수 없으므로 Function/eval을 사용하면 안 된다.
 */
export function parseContractLiteral(source) {
  let index = 0;
  const fail = message => {
    throw new Error(`gymContract ${message} (offset ${index})`);
  };
  const skipWhitespace = () => {
    while (index < source.length) {
      while (index < source.length && /\s/.test(source[index])) index += 1;
      if (source.startsWith('//', index)) {
        index = source.indexOf('\n', index + 2);
        if (index < 0) {
          index = source.length;
          return;
        }
        continue;
      }
      if (source.startsWith('/*', index)) {
        const end = source.indexOf('*/', index + 2);
        if (end < 0) fail('블록 주석이 닫히지 않았다');
        index = end + 2;
        continue;
      }
      return;
    }
  };
  const parseString = () => {
    const quote = source[index++];
    let value = '';
    while (index < source.length) {
      const ch = source[index++];
      if (ch === quote) return value;
      if (ch !== '\\') {
        value += ch;
        continue;
      }
      if (index >= source.length) fail('문자열 escape가 끝나지 않았다');
      const escaped = source[index++];
      const simple = {
        '"': '"',
        "'": "'",
        '\\': '\\',
        b: '\b',
        f: '\f',
        n: '\n',
        r: '\r',
        t: '\t',
        v: '\v',
      };
      if (Object.hasOwn(simple, escaped)) {
        value += simple[escaped];
      } else if (escaped === 'u') {
        const hex = source.slice(index, index + 4);
        if (!/^[0-9a-fA-F]{4}$/.test(hex)) fail('유효하지 않은 Unicode escape다');
        value += String.fromCharCode(Number.parseInt(hex, 16));
        index += 4;
      } else {
        fail(`허용되지 않은 문자열 escape \\${escaped}`);
      }
    }
    fail('문자열이 닫히지 않았다');
  };
  const parseKey = () => {
    skipWhitespace();
    if (source[index] === '"' || source[index] === "'") return parseString();
    const match = /^[A-Za-z_$][A-Za-z0-9_$]*/.exec(source.slice(index));
    if (!match) fail('객체 키가 필요하다');
    index += match[0].length;
    return match[0];
  };
  const parseNumber = () => {
    const match = /^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/.exec(source.slice(index));
    if (!match) fail('유효한 숫자가 필요하다');
    index += match[0].length;
    const value = Number(match[0]);
    if (!Number.isFinite(value)) fail('유한한 숫자만 허용한다');
    return value;
  };
  const parseValue = () => {
    skipWhitespace();
    const ch = source[index];
    if (ch === '{') return parseObject();
    if (ch === '"' || ch === "'") return parseString();
    if (ch === '-' || (ch >= '0' && ch <= '9')) return parseNumber();
    fail('객체·문자열·숫자 이외의 식은 허용하지 않는다');
  };
  const parseObject = () => {
    if (source[index] !== '{') fail("'{'가 필요하다");
    index += 1;
    const object = {};
    skipWhitespace();
    if (source[index] === '}') {
      index += 1;
      return object;
    }
    while (true) {
      const key = parseKey();
      if (Object.hasOwn(object, key)) fail(`중복 키 '${key}'는 허용하지 않는다`);
      skipWhitespace();
      if (source[index] !== ':') fail(`키 '${key}' 뒤에 ':'가 필요하다`);
      index += 1;
      object[key] = parseValue();
      skipWhitespace();
      if (source[index] === '}') {
        index += 1;
        return object;
      }
      if (source[index] !== ',') fail(`키 '${key}' 뒤에 ',' 또는 '}'가 필요하다`);
      index += 1;
      skipWhitespace();
      if (source[index] === '}') {
        index += 1;
        return object;
      }
    }
  };

  const value = parseValue();
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    fail('최상위 값은 객체여야 한다');
  }
  return value;
}

export function readContract(root, file) {
  const src = readFileSync(path.resolve(root, file), 'utf8');
  const match = src.match(/export\s+const\s+gymContract\s*=\s*\{/);
  if (!match || match.index === undefined) {
    throw new Error(`${file} 에 'export const gymContract' 가 없다`);
  }
  const objectStart = src.indexOf('{', match.index);
  return parseContractLiteral(src.slice(objectStart));
}

export function validateContract(contract) {
  if (typeof contract.sample !== 'string' || contract.sample.length === 0) {
    throw new Error('gymContract.sample은 비어 있지 않은 문자열이어야 한다');
  }
  if (!Number.isInteger(contract.chart) || contract.chart < 1) {
    throw new Error('gymContract.chart는 1 이상의 정수여야 한다');
  }
  if (contract.edit === null || typeof contract.edit !== 'object' || Array.isArray(contract.edit)) {
    throw new Error('gymContract.edit는 객체여야 한다');
  }
  for (const field of ['series', 'point']) {
    if (!Number.isInteger(contract.edit[field]) || contract.edit[field] < 0) {
      throw new Error(`gymContract.edit.${field}는 0 이상의 정수여야 한다`);
    }
  }
  for (const field of ['from', 'to']) {
    if (typeof contract.edit[field] !== 'string' && typeof contract.edit[field] !== 'number') {
      throw new Error(`gymContract.edit.${field}는 문자열 또는 숫자여야 한다`);
    }
  }
}

export function assertTaskIdAvailable(root, pack, id) {
  const packsDir = path.join(root, 'gym', 'packs');
  if (!existsSync(packsDir)) return;

  const owners = [];
  for (const entry of readdirSync(packsDir, { withFileTypes: true })) {
    if (!entry.isDirectory() || entry.name === pack) continue;
    const tasksDir = path.join(packsDir, entry.name, 'tasks');
    if (!existsSync(tasksDir)) continue;
    for (const name of readdirSync(tasksDir)) {
      if (!name.endsWith('.json')) continue;
      const task = JSON.parse(readFileSync(path.join(tasksDir, name), 'utf8'));
      if (task.id === id) owners.push(`${entry.name}/${name}`);
    }
  }
  if (owners.length > 0) {
    throw new Error(`과제 ID '${id}' 가 다른 pack에 이미 있다: ${owners.join(', ')}`);
  }
}

function main() {
  const root = process.cwd();
  const e2ePath = arg('e2e');
  const packId = arg('pack', 'studio-e2e');
  const taskId = arg('id');
  const bin = path.resolve(root, arg('bin', 'target/debug/rhwp'));
  if (!e2ePath || !taskId) {
    throw new Error('필수: --e2e <경로> --id <과제ID> [--pack studio-e2e] [--bin target/debug/rhwp]');
  }
  const contract = readContract(root, e2ePath);
  validateContract(contract);
  assertTaskIdAvailable(root, packId, taskId);

  const packDir = path.join(root, 'gym', 'packs', packId);
  const sampleRel = path.posix.join('samples', contract.sample);
  const env = JSON.parse(execFileSync(
    bin, ['chart-to-csv', sampleRel, '--chart', String(contract.chart), '--json'],
    { cwd: root, encoding: 'utf8' }));
  const baseCsv = env.charts[0].csv;

  // 생성 자산은 Git text 파일로 보관하므로 입력 CSV의 플랫폼 줄바꿈과 무관하게 LF로 고정한다.
  const eol = '\n';
  const rows = baseCsv.replace(/\r\n/g, '\n').replace(/\n$/, '').split('\n').map(row => row.split(','));
  const dataRow = rows[1 + contract.edit.point];
  const column = 1 + contract.edit.series;
  if (!dataRow) throw new Error(`point ${contract.edit.point} 데이터 행이 없다`);
  if (dataRow[column] !== String(contract.edit.from)) {
    throw new Error(`계약 불일치: (계열 ${contract.edit.series}, 값 ${contract.edit.point}) 현재 `
      + `'${dataRow[column]}' ≠ from '${contract.edit.from}' — 샘플이 바뀌었다`);
  }
  dataRow[column] = String(contract.edit.to);
  const editCsv = rows.map(row => row.join(',')).join(eol) + eol;

  for (const directory of ['assets', 'tasks', 'reference']) {
    mkdirSync(path.join(packDir, directory), { recursive: true });
  }
  const csvAsset = `gym/packs/${packId}/assets/${taskId}-edit.csv`;
  writeFileSync(path.join(root, csvAsset), editCsv, 'utf8');

  const task = {
    id: taskId,
    tier: 3,
    title: `차트 데이터 편집 왕복 (studio ${path.basename(e2ePath)} 파생)`,
    input: sampleRel,
    instructions:
      `차트 ${contract.chart}의 (계열 ${contract.edit.series}, 값 ${contract.edit.point}) 원본 ${contract.edit.from} 을 `
      + `'${contract.edit.to}' 로 바꿔 out.hwp 로 저장하라. 원본 크기(계열 수·값 개수·계열명·`
      + `카테고리 라벨)는 그대로 두어야 한다. 힌트: chart-to-csv 로 뽑아 그 칸만 고치고 `
      + `csv-to-chart 로 되넣어라(-o out.hwp).`,
    submit: { kind: 'artifact', files: ['out.hwp'] },
    checks: [
      { name: '산출물 존재', op: 'file_exists', file: 'out.hwp', minBytes: 1 },
      { name: '원본과 다름 (무편집 복사 거부)', op: 'differs_from_input', file: 'out.hwp' },
      {
        name: `첫 값이 이미 ${contract.edit.to} (센티넬 재적용이 무변경)`,
        op: 'value_eq', path: 'changedCount', value: 0,
        cmd: ['csv-to-chart', '{file:out.hwp}', '--csv', csvAsset,
          '--chart', String(contract.chart), '--dry-run', '--json'],
      },
    ],
  };
  const reference = {
    id: taskId,
    steps: [{ run: ['csv-to-chart', '{input}', '--csv', csvAsset,
      '--chart', String(contract.chart), '-o', '{sub:out.hwp}', '--json'] }],
  };
  const dump = object => JSON.stringify(object, null, 2) + '\n';
  writeFileSync(path.join(packDir, 'tasks', `${taskId}.json`), dump(task), 'utf8');
  writeFileSync(path.join(packDir, 'reference', `${taskId}.json`), dump(reference), 'utf8');

  console.log(`생성: ${taskId} — assets/${taskId}-edit.csv · tasks/${taskId}.json · reference/${taskId}.json`);
  console.log(`검증: python gym/tools/build_baseline.py --agent baseline --pack ${packId} --bin <bin> && python gym/score.py --agent baseline --pack ${packId} --bin <bin>`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
