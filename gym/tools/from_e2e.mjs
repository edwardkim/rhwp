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

/**
 * 어댑터 예외. kind 는 ERROR_KINDS 에 있는 값만 쓴다.
 * 메시지는 기존 시험이 정규식으로 고정한 문자열을 유지한다.
 */
export class FromE2eError extends Error {
  constructor(kind, message, extras = {}) {
    super(message);
    this.name = 'FromE2eError';
    this.kind = kind;
    this.extras = extras;
  }
}

/** 예외 kind 카탈로그. 문서·시험이 같은 표를 본다. 여기에 없는 값은 unexpected 로 접는다. */
export const ERROR_KINDS = Object.freeze([
  'missing-arg',
  'missing-file',
  'missing-bin',
  'permission',
  'timeout',
  'decode-error',
  'parse-error',
  'missing-contract',
  'validate-error',
  'studio-only',
  'path-escape',
  'csv-mismatch',
  'csv-missing-row',
  'csv-missing-col',
  'csv-shape',
  'task-id-conflict',
  'task-id-invalid',
  'envelope-error',
  'cli-error',
  'os-error',
  'type-error',
  'json-error',
  'unexpected',
]);

/**
 * 종료 코드. 0 은 성공이다. 분류를 위장하지 않도록 kind 마다 고정한다.
 * 1 은 카탈로그 밖/예기치 않은 실패. 2 인자, 3 계약, 4 CSV, 5 CLI, 6 I/O.
 */
export const EXIT_BY_KIND = Object.freeze({
  'missing-arg': 2,
  'parse-error': 3,
  'missing-contract': 3,
  'validate-error': 3,
  'studio-only': 3,
  'path-escape': 3,
  'task-id-invalid': 3,
  'task-id-conflict': 3,
  'type-error': 3,
  'csv-mismatch': 4,
  'csv-missing-row': 4,
  'csv-missing-col': 4,
  'csv-shape': 4,
  'missing-bin': 5,
  'cli-error': 5,
  'envelope-error': 5,
  'timeout': 5,
  'missing-file': 6,
  'permission': 6,
  'os-error': 6,
  'decode-error': 6,
  'json-error': 6,
  'unexpected': 1,
});

/** 삼키면 안 되는 예외 이름. 사용자가 끊었는데 성공 보고를 내면 거짓말이다. */
export const FATAL_ERROR_NAMES = Object.freeze([
  'SystemExit',
  'GeneratorExit',
]);

/** gymContract 가 가져도 되는 최상위 키. 나머지는 스튜디오 UI 계약을 숨긴 자리로 본다. */
export const ALLOWED_CONTRACT_KEYS = Object.freeze(['sample', 'chart', 'edit']);

/** edit 객체가 가져도 되는 키. */
export const ALLOWED_EDIT_KEYS = Object.freeze(['series', 'point', 'from', 'to']);

/**
 * 스튜디오 전용 표면. CLI 로 재현할 수 없으므로 계약에 있으면 거부한다.
 * 이 목록을 느슨하게 풀어 브라우저 e2e 전체를 과제로 위장하지 않는다.
 */
export const STUDIO_ONLY_KEYS = Object.freeze([
  'ui',
  'menu',
  'click',
  'dblclick',
  'doubleClick',
  'undo',
  'redo',
  'hotkey',
  'shortcut',
  'dialog',
  'pageObject',
  'selector',
  'locator',
  'snapshot',
  'trace',
  'playwright',
  'canvas',
  'pointer',
  'hover',
  'contextMenu',
  'keyboard',
  'mouse',
  'viewport',
  'screenshot',
  'wasm',
  'getChartDataByIndex',
  'setChartDataByIndex',
  'window',
  'document',
  'browser',
  'e2eOnly',
  'studioOnly',
  'ole',
  'noTrace',
]);

/** 이 어댑터가 과제/기준에 넣을 수 있는 CLI 명령. 다른 명령은 다른 pack 의 일이다. */
export const ALLOWED_CLI_COMMANDS = Object.freeze(['chart-to-csv', 'csv-to-chart']);

/** 이 어댑터가 과제 검사에 넣을 수 있는 op. */
export const ALLOWED_CHECK_OPS = Object.freeze([
  'file_exists',
  'differs_from_input',
  'value_eq',
]);

/** 이 어댑터가 위장하면 안 되는 CLI. 표·필드·렌더는 다른 온램프의 축이다. */
export const FORBIDDEN_CLI_COMMANDS = Object.freeze([
  'table-to-csv',
  'csv-to-table',
  'fill-fields',
  'export-pdf',
  'export-png',
  'export-svg',
  'render',
  'inspect',
  'thumbnail',
  'mcp-serve',
]);

/** 샘플 경로가 가져야 하는 확장자. 스튜디오 차트 왕복은 HWP/HWPX 만 연다. */
export const ALLOWED_SAMPLE_SUFFIXES = Object.freeze(['.hwp', '.hwpx']);

/** 과제 ID 형식. 기존 pack 은 두세 글자 + 숫자 두 자리다. */
export const TASK_ID_PATTERN = /^[A-Z]{1,3}[0-9]{2}$/;

export const USAGE =
  '필수: --e2e <경로> --id <과제ID> [--pack studio-e2e] [--bin target/debug/rhwp]';

export const ERROR_HEAD_LIMIT = 160;

function argFrom(argv, name, def) {
  const i = argv.indexOf(`--${name}`);
  return i >= 0 ? argv[i + 1] : def;
}

function arg(name, def) {
  return argFrom(process.argv, name, def);
}

/**
 * e2e 파일을 실행하지 않고 gymContract의 제한된 객체 리터럴만 읽는다.
 *
 * 허용 값은 중첩 객체, 문자열, 숫자뿐이다. 식·함수·템플릿·배열·식별자는 거부한다.
 * 외부 PR의 e2e 파일은 검토 환경에서 신뢰할 수 없으므로 Function/eval을 사용하면 안 된다.
 */
function consumeContractLiteral(source) {
  let index = 0;
  const fail = message => {
    throw new FromE2eError('parse-error', `gymContract ${message} (offset ${index})`, { offset: index });
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
  skipWhitespace();
  return { value, index };
}

export function parseContractLiteral(source) {
  if (typeof source !== 'string') {
    throw new FromE2eError('type-error', 'gymContract 원문은 문자열이어야 한다');
  }
  const { value, index } = consumeContractLiteral(source);
  if (index < source.length) {
    throw new FromE2eError(
      'parse-error',
      `gymContract 뒤에 추가 식이 있다 (offset ${index})`,
      { offset: index },
    );
  }
  return value;
}

export function locateGymContract(source, file = 'e2e') {
  if (typeof source !== 'string') {
    throw new FromE2eError('type-error', 'e2e 원문은 문자열이어야 한다');
  }
  const match = source.match(/export\s+const\s+gymContract\s*=\s*\{/);
  if (!match || match.index === undefined) {
    throw new FromE2eError(
      'missing-contract',
      `${file} 에 'export const gymContract' 가 없다`,
      { file },
    );
  }
  const objectStart = source.indexOf('{', match.index);
  if (objectStart < 0) {
    throw new FromE2eError('missing-contract', `${file} 에 'export const gymContract' 가 없다`, { file });
  }
  return { matchIndex: match.index, objectStart, marker: match[0] };
}

export function readContract(root, file) {
  const abs = path.resolve(root, file);
  const src = readTextFileOrThrow(abs, file);
  const { objectStart } = locateGymContract(src, file);
  // 파일 나머지 코드는 실행하지 않고, 계약 객체만 소비한다.
  return consumeContractLiteral(src.slice(objectStart)).value;
}

export function validateContract(contract) {
  if (contract === null || typeof contract !== 'object' || Array.isArray(contract)) {
    throw new FromE2eError('validate-error', 'gymContract는 객체여야 한다');
  }
  if (typeof contract.sample !== 'string' || contract.sample.length === 0) {
    throw new FromE2eError('validate-error', 'gymContract.sample은 비어 있지 않은 문자열이어야 한다');
  }
  if (!Number.isInteger(contract.chart) || contract.chart < 1) {
    throw new FromE2eError('validate-error', 'gymContract.chart는 1 이상의 정수여야 한다');
  }
  if (contract.edit === null || typeof contract.edit !== 'object' || Array.isArray(contract.edit)) {
    throw new FromE2eError('validate-error', 'gymContract.edit는 객체여야 한다');
  }
  for (const field of ['series', 'point']) {
    if (!Number.isInteger(contract.edit[field]) || contract.edit[field] < 0) {
      throw new FromE2eError('validate-error', `gymContract.edit.${field}는 0 이상의 정수여야 한다`);
    }
  }
  for (const field of ['from', 'to']) {
    if (typeof contract.edit[field] !== 'string' && typeof contract.edit[field] !== 'number') {
      throw new FromE2eError('validate-error', `gymContract.edit.${field}는 문자열 또는 숫자여야 한다`);
    }
  }
}

export function assertTaskIdAvailable(root, pack, id) {
  const packsDir = path.join(root, 'gym', 'packs');
  if (!existsSync(packsDir)) return;

  let entries;
  try {
    entries = readdirSync(packsDir, { withFileTypes: true });
  } catch (err) {
    throw wrapNodeError(err, 'task-id', packsDir);
  }

  const owners = [];
  for (const entry of entries) {
    if (!entry.isDirectory() || entry.name === pack) continue;
    const tasksDir = path.join(packsDir, entry.name, 'tasks');
    if (!existsSync(tasksDir)) continue;
    let names;
    try {
      names = readdirSync(tasksDir);
    } catch (err) {
      throw wrapNodeError(err, 'task-id', tasksDir);
    }
    for (const name of names) {
      if (!name.endsWith('.json')) continue;
      const taskPath = path.join(tasksDir, name);
      let task;
      try {
        task = readJsonFileOrThrow(taskPath, `${entry.name}/${name}`);
      } catch (err) {
        if (err instanceof FromE2eError) throw err;
        throw wrapNodeError(err, 'task-id', taskPath);
      }
      if (task && typeof task === 'object' && !Array.isArray(task) && task.id === id) {
        owners.push(`${entry.name}/${name}`);
      }
    }
  }
  if (owners.length > 0) {
    throw new FromE2eError(
      'task-id-conflict',
      `과제 ID '${id}' 가 다른 pack에 이미 있다: ${owners.join(', ')}`,
      { id, owners },
    );
  }
}

/**
 * chart-to-csv 산출 CSV에서 계약이 지정한 한 칸만 바꾼다.
 * 입력 줄바꿈과 무관하게 LF로 정규화하고, 결과는 항상 LF로 끝난다.
 */
export function applyCsvEdit(baseCsv, edit) {
  if (typeof baseCsv !== 'string') {
    throw new FromE2eError('type-error', 'applyCsvEdit의 CSV는 문자열이어야 한다');
  }
  if (edit === null || typeof edit !== 'object' || Array.isArray(edit)) {
    throw new FromE2eError('type-error', 'applyCsvEdit의 edit는 객체여야 한다');
  }
  const rows = splitCsvRows(baseCsv);
  assertCsvRectangular(rows);
  const dataRow = rows[1 + edit.point];
  const column = 1 + edit.series;
  if (!dataRow) {
    throw new FromE2eError('csv-missing-row', `point ${edit.point} 데이터 행이 없다`, {
      point: edit.point,
    });
  }
  if (column < 0 || column >= dataRow.length) {
    throw new FromE2eError('csv-missing-col', `series ${edit.series} 열이 없다`, {
      series: edit.series,
    });
  }
  if (dataRow[column] !== String(edit.from)) {
    throw new FromE2eError(
      'csv-mismatch',
      `계약 불일치: (계열 ${edit.series}, 값 ${edit.point}) 현재 `
        + `'${dataRow[column]}' ≠ from '${edit.from}' — 샘플이 바뀌었다`,
      { series: edit.series, point: edit.point, actual: dataRow[column], from: edit.from },
    );
  }
  dataRow[column] = String(edit.to);
  return joinCsvRows(rows);
}

export function buildTask({ taskId, e2ePath, contract, csvAsset }) {
  return {
    id: taskId,
    tier: 3,
    title: `차트 데이터 편집 왕복 (studio ${path.basename(e2ePath)} 파생)`,
    input: path.posix.join('samples', contract.sample),
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
}

export function buildReference({ taskId, contract, csvAsset }) {
  return {
    id: taskId,
    steps: [{ run: ['csv-to-chart', '{input}', '--csv', csvAsset,
      '--chart', String(contract.chart), '-o', '{sub:out.hwp}', '--json'] }],
  };
}

export function dumpJson(object) {
  return `${JSON.stringify(object, null, 2)}\n`;
}

export function splitCsvRows(text) {
  return text.replace(/\r\n/g, '\n').replace(/\n$/, '').split('\n').map(row => row.split(','));
}

export function joinCsvRows(rows) {
  return `${rows.map(row => row.join(',')).join('\n')}\n`;
}

export function assertCsvRectangular(rows) {
  if (!Array.isArray(rows) || rows.length === 0) {
    throw new FromE2eError('csv-shape', 'CSV 행이 없다');
  }
  const width = rows[0].length;
  if (width < 2) {
    throw new FromE2eError('csv-shape', 'CSV 헤더에 계열 열이 없다');
  }
  for (let i = 0; i < rows.length; i += 1) {
    if (!Array.isArray(rows[i]) || rows[i].length !== width) {
      throw new FromE2eError('csv-shape', `CSV ${i}행 열 수가 ${width}가 아니다`);
    }
  }
}

export function isFatalException(exc) {
  if (exc == null || typeof exc !== 'object') return false;
  if (exc.fatal === true) return true;
  if (FATAL_ERROR_NAMES.includes(exc.name)) return true;
  if (exc.code === 'ERR_WORKER_OUT_OF_MEMORY') return true;
  return false;
}

export function exitCodeForKind(kind) {
  if (Object.hasOwn(EXIT_BY_KIND, kind)) return EXIT_BY_KIND[kind];
  return EXIT_BY_KIND.unexpected;
}

export function truncateHead(text, limit = ERROR_HEAD_LIMIT) {
  if (text == null) return '';
  const value = typeof text === 'string' ? text : String(text);
  if (limit <= 0) return '';
  return value.length <= limit ? value : value.slice(0, limit);
}

export function classifyNodeError(err, context = 'io') {
  if (err instanceof FromE2eError) return err.kind;
  if (err == null) return 'unexpected';
  const code = err.code;
  if (code === 'ENOENT') return context === 'cli' || context === 'bin' ? 'missing-bin' : 'missing-file';
  if (code === 'EACCES' || code === 'EPERM') return 'permission';
  if (code === 'ETIMEDOUT' || code === 'ERR_SCRIPT_EXECUTION_TIMEOUT') return 'timeout';
  if (typeof err.status === 'number') return 'cli-error';
  if (err instanceof SyntaxError) return context === 'json' || context === 'envelope' ? 'json-error' : 'parse-error';
  if (err instanceof TypeError) return 'type-error';
  if (err instanceof RangeError) return 'type-error';
  if (isUnicodeError(err)) return 'decode-error';
  if (typeof err.message === 'string' && /timeout|ETIMEDOUT/i.test(err.message)) return 'timeout';
  if (code && String(code).startsWith('ERR_')) return 'os-error';
  if (typeof err.errno === 'number') return 'os-error';
  return 'unexpected';
}

function isUnicodeError(err) {
  return Boolean(err && (err.name === 'URIError' || /decode|encode|utf/i.test(err.message || '')));
}

export function wrapNodeError(err, context, pathHint) {
  if (isFatalException(err)) throw err;
  if (err instanceof FromE2eError) return err;
  const kind = classifyNodeError(err, context);
  const where = pathHint ? ` (${pathHint})` : '';
  const head = truncateHead(err && err.message ? err.message : String(err));
  return new FromE2eError(kind, `${kind}${where}: ${head}`, {
    context,
    path: pathHint,
    causeName: err && err.name,
    causeCode: err && err.code,
  });
}

export function exceptionReport(err, context = 'main') {
  if (isFatalException(err)) {
    return {
      kind: 'unexpected',
      fatal: true,
      context,
      error: err && err.name ? err.name : 'Fatal',
      message: truncateHead(err && err.message),
    };
  }
  const kind = err instanceof FromE2eError ? err.kind : classifyNodeError(err, context);
  return {
    kind: ERROR_KINDS.includes(kind) ? kind : 'unexpected',
    fatal: false,
    context,
    error: err && err.name ? err.name : 'Error',
    message: truncateHead(err && err.message),
    extras: err instanceof FromE2eError ? err.extras : {},
    exit: exitCodeForKind(ERROR_KINDS.includes(kind) ? kind : 'unexpected'),
  };
}

export function formatExceptionReport(report) {
  const extras = report.extras && Object.keys(report.extras).length > 0
    ? ` ${JSON.stringify(report.extras)}`
    : '';
  return `from_e2e ${report.kind} [${report.context}] ${report.message}${extras}`;
}

export function readTextFileOrThrow(absPath, label) {
  try {
    return readFileSync(absPath, 'utf8');
  } catch (err) {
    throw wrapNodeError(err, 'io', label || absPath);
  }
}

export function readJsonFileOrThrow(absPath, label) {
  const text = readTextFileOrThrow(absPath, label);
  try {
    return JSON.parse(text);
  } catch (err) {
    throw new FromE2eError(
      'json-error',
      `과제 파일 파싱 실패: ${label || absPath}: ${err.message}`,
      { path: absPath },
    );
  }
}

export function parseFromE2eArgv(argv) {
  if (!Array.isArray(argv)) {
    throw new FromE2eError('type-error', 'argv는 배열이어야 한다');
  }
  const e2e = argFrom(argv, 'e2e');
  const pack = argFrom(argv, 'pack', 'studio-e2e');
  const id = argFrom(argv, 'id');
  const bin = argFrom(argv, 'bin', 'target/debug/rhwp');
  if (!e2e || !id) {
    throw new FromE2eError('missing-arg', USAGE);
  }
  if (e2e.startsWith('--') || id.startsWith('--')) {
    throw new FromE2eError('missing-arg', USAGE);
  }
  return { e2e, pack, id, bin };
}

export function assertTaskIdFormat(id) {
  if (typeof id !== 'string' || !TASK_ID_PATTERN.test(id)) {
    throw new FromE2eError(
      'task-id-invalid',
      `과제 ID '${id}' 는 대문자 1~3자 + 숫자 두 자리여야 한다`,
      { id },
    );
  }
}

export function assertSafeSamplePath(sample) {
  if (typeof sample !== 'string' || sample.length === 0) {
    throw new FromE2eError('validate-error', 'gymContract.sample은 비어 있지 않은 문자열이어야 한다');
  }
  if (sample.includes('\0')) {
    throw new FromE2eError('path-escape', 'gymContract.sample에 NUL 이 있다');
  }
  if (path.isAbsolute(sample) || /^[A-Za-z]:[\\/]/.test(sample)) {
    throw new FromE2eError('path-escape', 'gymContract.sample은 samples/ 아래 상대 경로여야 한다');
  }
  const parts = sample.split(/[\\/]/);
  if (parts.some(part => part === '..')) {
    throw new FromE2eError('path-escape', 'gymContract.sample은 상위 경로를 포함할 수 없다');
  }
  const suffix = ALLOWED_SAMPLE_SUFFIXES.find(item => sample.toLowerCase().endsWith(item));
  if (!suffix) {
    throw new FromE2eError('validate-error', 'gymContract.sample은 .hwp 또는 .hwpx 여야 한다');
  }
}

export function collectForbiddenKeys(object, forbidden, prefix = '') {
  const found = [];
  if (object === null || typeof object !== 'object' || Array.isArray(object)) return found;
  for (const key of Object.keys(object)) {
    const pathKey = prefix ? `${prefix}.${key}` : key;
    if (forbidden.includes(key)) found.push(pathKey);
    found.push(...collectForbiddenKeys(object[key], forbidden, pathKey));
  }
  return found;
}

export function collectUnknownKeys(object, allowed) {
  if (object === null || typeof object !== 'object' || Array.isArray(object)) return [];
  return Object.keys(object).filter(key => !allowed.includes(key));
}

/**
 * CLI 로 재현 가능한 스튜디오 계약만 남긴다.
 * sample/chart/edit 이외의 키, 스튜디오 UI 키, from===to 는 거부한다.
 */
export function assertCliReproducibleContract(contract) {
  validateContract(contract);
  assertSafeSamplePath(contract.sample);
  const studio = collectForbiddenKeys(contract, STUDIO_ONLY_KEYS);
  if (studio.length > 0) {
    throw new FromE2eError(
      'studio-only',
      `CLI 로 재현할 수 없는 스튜디오 키: ${studio.join(', ')}`,
      { keys: studio },
    );
  }
  const extra = collectUnknownKeys(contract, ALLOWED_CONTRACT_KEYS);
  if (extra.length > 0) {
    throw new FromE2eError(
      'studio-only',
      `허용되지 않은 gymContract 키: ${extra.join(', ')} — CLI 차트 왕복만 받는다`,
      { keys: extra },
    );
  }
  const extraEdit = collectUnknownKeys(contract.edit, ALLOWED_EDIT_KEYS);
  if (extraEdit.length > 0) {
    throw new FromE2eError(
      'studio-only',
      `허용되지 않은 edit 키: ${extraEdit.join(', ')}`,
      { keys: extraEdit },
    );
  }
  if (String(contract.edit.from) === String(contract.edit.to)) {
    throw new FromE2eError(
      'validate-error',
      'gymContract.edit.from 과 to 가 같다 — 무편집을 왕복으로 위장하지 않는다',
    );
  }
}

export function assertTaskChecksAreCliReproducible(task) {
  if (task === null || typeof task !== 'object' || Array.isArray(task)) {
    throw new FromE2eError('type-error', '과제는 객체여야 한다');
  }
  if (!Array.isArray(task.checks)) {
    throw new FromE2eError('validate-error', '과제 checks 는 배열이어야 한다');
  }
  for (const check of task.checks) {
    if (!ALLOWED_CHECK_OPS.includes(check.op)) {
      throw new FromE2eError(
        'studio-only',
        `허용되지 않은 검사 op '${check.op}' — CLI 차트 왕복만 받는다`,
        { op: check.op },
      );
    }
    if (Array.isArray(check.cmd)) {
      const command = check.cmd[0];
      if (FORBIDDEN_CLI_COMMANDS.includes(command)) {
        throw new FromE2eError(
          'studio-only',
          `금지된 CLI 명령 '${command}' — 이 어댑터의 축이 아니다`,
          { command },
        );
      }
      if (!ALLOWED_CLI_COMMANDS.includes(command)) {
        throw new FromE2eError(
          'studio-only',
          `허용되지 않은 CLI 명령 '${command}' — chart-to-csv/csv-to-chart 만 쓴다`,
          { command },
        );
      }
    }
  }
}

export function assertReferenceIsCliReproducible(reference) {
  if (reference === null || typeof reference !== 'object' || Array.isArray(reference)) {
    throw new FromE2eError('type-error', '기준풀이는 객체여야 한다');
  }
  const steps = reference.steps;
  if (!Array.isArray(steps) || steps.length !== 1) {
    throw new FromE2eError('validate-error', '기준풀이는 csv-to-chart 한 스텝이어야 한다');
  }
  const run = steps[0] && steps[0].run;
  if (!Array.isArray(run) || run[0] !== 'csv-to-chart') {
    throw new FromE2eError(
      'studio-only',
      '기준풀이 run 은 csv-to-chart 여야 한다',
      { run },
    );
  }
  if (FORBIDDEN_CLI_COMMANDS.some(command => run.includes(command))) {
    throw new FromE2eError('studio-only', '기준풀이에 금지된 CLI 명령이 있다');
  }
}

/**
 * chart-to-csv --json 봉투에서 CSV 를 꺼낸다. 바이너리를 부르지 않는다.
 * ok=false 를 성공으로 위장하지 않는다.
 */
export function extractChartCsvFromEnvelope(envelope, chartIndex = 1) {
  if (envelope === null || typeof envelope !== 'object' || Array.isArray(envelope)) {
    throw new FromE2eError('envelope-error', 'chart-to-csv 봉투는 객체여야 한다');
  }
  if (envelope.ok === false) {
    throw new FromE2eError(
      'cli-error',
      'chart-to-csv 봉투 ok=false — CLI 실패를 성공으로 위장하지 않는다',
    );
  }
  if (!Array.isArray(envelope.charts)) {
    throw new FromE2eError('envelope-error', 'chart-to-csv 봉투에 charts 배열이 없다');
  }
  if (envelope.charts.length === 0) {
    throw new FromE2eError('envelope-error', 'chart-to-csv 봉투 charts 가 비어 있다');
  }
  const slot = envelope.charts[0];
  if (slot === null || typeof slot !== 'object' || Array.isArray(slot)) {
    throw new FromE2eError('envelope-error', 'chart-to-csv 봉투 charts[0] 은 객체여야 한다');
  }
  if (typeof slot.csv !== 'string' || slot.csv.length === 0) {
    throw new FromE2eError('envelope-error', 'chart-to-csv 봉투 charts[0].csv 가 없다');
  }
  if (!Number.isInteger(chartIndex) || chartIndex < 1) {
    throw new FromE2eError('validate-error', 'gymContract.chart는 1 이상의 정수여야 한다');
  }
  return slot.csv;
}

export function parseChartToCsvStdout(stdout) {
  if (typeof stdout !== 'string') {
    throw new FromE2eError('type-error', 'chart-to-csv 출력은 문자열이어야 한다');
  }
  const trimmed = stdout.replace(/^\uFEFF/, '').trim();
  if (trimmed.length === 0) {
    throw new FromE2eError('envelope-error', 'chart-to-csv 출력이 비어 있다');
  }
  let envelope;
  try {
    envelope = JSON.parse(trimmed);
  } catch (err) {
    throw new FromE2eError(
      'envelope-error',
      `chart-to-csv 출력이 JSON 이 아니다: ${truncateHead(err.message)}`,
    );
  }
  return envelope;
}

export function invokeChartToCsv(bin, sampleRel, chart, options = {}) {
  const execFn = options.execFileSync || execFileSync;
  const cwd = options.cwd || process.cwd();
  if (typeof bin !== 'string' || bin.length === 0) {
    throw new FromE2eError('missing-bin', 'rhwp 바이너리 경로가 없다');
  }
  let stdout;
  try {
    stdout = execFn(
      bin,
      ['chart-to-csv', sampleRel, '--chart', String(chart), '--json'],
      { cwd, encoding: 'utf8' },
    );
  } catch (err) {
    throw wrapNodeError(err, 'cli', bin);
  }
  const envelope = parseChartToCsvStdout(stdout);
  return extractChartCsvFromEnvelope(envelope, chart);
}

/**
 * 이미 뽑아 둔 CLI CSV 와 계약을 과제/기준/편집 CSV 로 조립한다.
 * 바이너리를 부르지 않으므로 순수 시험이 고정한다.
 */
export function materializeFromContract({ taskId, e2ePath, contract, csvAsset, chartCsv }) {
  assertTaskIdFormat(taskId);
  assertCliReproducibleContract(contract);
  const editCsv = applyCsvEdit(chartCsv, contract.edit);
  const task = buildTask({ taskId, e2ePath, contract, csvAsset });
  const reference = buildReference({ taskId, contract, csvAsset });
  assertTaskChecksAreCliReproducible(task);
  assertReferenceIsCliReproducible(reference);
  return { csvAsset, editCsv, task, reference };
}

export function honestyPolicy() {
  return {
    adapter: 'from_e2e',
    liveOracle: true,
    executesE2e: false,
    usesEval: false,
    usesFunction: false,
    allowedContractKeys: [...ALLOWED_CONTRACT_KEYS],
    allowedEditKeys: [...ALLOWED_EDIT_KEYS],
    allowedCliCommands: [...ALLOWED_CLI_COMMANDS],
    allowedCheckOps: [...ALLOWED_CHECK_OPS],
    forbiddenCliCommands: [...FORBIDDEN_CLI_COMMANDS],
    studioOnlyKeys: [...STUDIO_ONLY_KEYS],
    errorKinds: [...ERROR_KINDS],
    exitByKind: { ...EXIT_BY_KIND },
    note: '문서 데이터 계약만 CLI 로 파생한다. UI·undo·메뉴·OLE 음성계약은 e2e 에 남긴다.',
  };
}

function writeGeneratedArtifacts(root, packId, taskId, materialized) {
  const packDir = path.join(root, 'gym', 'packs', packId);
  for (const directory of ['assets', 'tasks', 'reference']) {
    try {
      mkdirSync(path.join(packDir, directory), { recursive: true });
    } catch (err) {
      throw wrapNodeError(err, 'io', path.join(packDir, directory));
    }
  }
  const csvAbs = path.join(root, materialized.csvAsset);
  try {
    writeFileSync(csvAbs, materialized.editCsv, 'utf8');
    writeFileSync(path.join(packDir, 'tasks', `${taskId}.json`), dumpJson(materialized.task), 'utf8');
    writeFileSync(path.join(packDir, 'reference', `${taskId}.json`), dumpJson(materialized.reference), 'utf8');
  } catch (err) {
    throw wrapNodeError(err, 'io', csvAbs);
  }
}

export function mainWith(argv = process.argv, options = {}) {
  const root = options.cwd || process.cwd();
  const parsed = parseFromE2eArgv(argv);
  assertTaskIdFormat(parsed.id);
  const contract = readContract(root, parsed.e2e);
  assertCliReproducibleContract(contract);
  assertTaskIdAvailable(root, parsed.pack, parsed.id);

  const bin = path.resolve(root, parsed.bin);
  const sampleRel = path.posix.join('samples', contract.sample);
  const baseCsv = invokeChartToCsv(bin, sampleRel, contract.chart, {
    cwd: root,
    execFileSync: options.execFileSync,
  });
  const csvAsset = `gym/packs/${parsed.pack}/assets/${parsed.id}-edit.csv`;
  const materialized = materializeFromContract({
    taskId: parsed.id,
    e2ePath: parsed.e2e,
    contract,
    csvAsset,
    chartCsv: baseCsv,
  });
  if (!options.dryRun) {
    writeGeneratedArtifacts(root, parsed.pack, parsed.id, materialized);
  }
  return {
    ok: true,
    exit: 0,
    taskId: parsed.id,
    pack: parsed.pack,
    csvAsset,
    task: materialized.task,
    reference: materialized.reference,
  };
}

export function runAsCli(argv = process.argv, io = {
  log: (...args) => console.log(...args),
  err: (...args) => console.error(...args),
}, options = {}) {
  try {
    const result = mainWith(argv, options);
    if (io && typeof io.log === 'function') {
      io.log(`생성: ${result.taskId} — assets/${result.taskId}-edit.csv · tasks/${result.taskId}.json · reference/${result.taskId}.json`);
      io.log(`검증: python gym/tools/build_baseline.py --agent baseline --pack ${result.pack} --bin <bin> && python gym/score.py --agent baseline --pack ${result.pack} --bin <bin>`);
    }
    return result;
  } catch (err) {
    if (isFatalException(err)) throw err;
    const report = exceptionReport(err, 'main');
    if (io && typeof io.err === 'function') {
      io.err(formatExceptionReport(report));
    }
    return { ok: false, exit: report.exit, report };
  }
}

function main() {
  const result = runAsCli(process.argv);
  if (!result.ok) {
    process.exitCode = result.exit;
    throw new FromE2eError(result.report.kind, result.report.message, result.report.extras);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = runAsCli(process.argv);
  if (!result.ok) process.exit(result.exit);
}
