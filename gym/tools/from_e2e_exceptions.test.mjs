import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  ALLOWED_CHECK_OPS,
  ALLOWED_CLI_COMMANDS,
  ALLOWED_CONTRACT_KEYS,
  ALLOWED_EDIT_KEYS,
  ALLOWED_SAMPLE_SUFFIXES,
  ERROR_KINDS,
  EXIT_BY_KIND,
  FATAL_ERROR_NAMES,
  FORBIDDEN_CLI_COMMANDS,
  FromE2eError,
  STUDIO_ONLY_KEYS,
  TASK_ID_PATTERN,
  USAGE,
  applyCsvEdit,
  assertCliReproducibleContract,
  assertCsvRectangular,
  assertReferenceIsCliReproducible,
  assertSafeSamplePath,
  assertTaskChecksAreCliReproducible,
  assertTaskIdAvailable,
  assertTaskIdFormat,
  buildReference,
  buildTask,
  classifyNodeError,
  collectForbiddenKeys,
  collectUnknownKeys,
  dumpJson,
  exceptionReport,
  exitCodeForKind,
  extractChartCsvFromEnvelope,
  formatExceptionReport,
  honestyPolicy,
  invokeChartToCsv,
  isFatalException,
  joinCsvRows,
  locateGymContract,
  materializeFromContract,
  parseChartToCsvStdout,
  parseContractLiteral,
  parseFromE2eArgv,
  readContract,
  readJsonFileOrThrow,
  runAsCli,
  splitCsvRows,
  truncateHead,
  validateContract,
  wrapNodeError,
} from './from_e2e.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const validEdit = { series: 0, point: 2, from: '4.3', to: '91.7' };

function validContract(overrides = {}) {
  return {
    sample: 'chart/sample.hwp',
    chart: 1,
    edit: { ...validEdit },
    ...overrides,
  };
}

const st01Contract = {
  sample: 'chart/세로막대형/묶은세로막대형.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: 4.3, to: 91.7 },
};
const st01CsvAsset = 'gym/packs/studio-e2e/assets/ST01-edit.csv';
const st01E2e = 'rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs';

function throwsKind(fn, kind) {
  assert.throws(fn, err => err instanceof FromE2eError && err.kind === kind);
}

test('예외 kind 카탈로그는 문서와 같은 표를 쓴다', () => {
  assert.deepEqual([...ERROR_KINDS], [
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
  assert.equal(new Set(ERROR_KINDS).size, ERROR_KINDS.length);
  for (const kind of ERROR_KINDS) {
    assert.equal(typeof EXIT_BY_KIND[kind], 'number');
    assert.ok(EXIT_BY_KIND[kind] >= 1 && EXIT_BY_KIND[kind] <= 6);
  }
});

test('FromE2eError는 kind와 extras를 싣는다', () => {
  const err = new FromE2eError('parse-error', 'gymContract 시험', { offset: 4 });
  assert.equal(err.name, 'FromE2eError');
  assert.equal(err.kind, 'parse-error');
  assert.equal(err.message, 'gymContract 시험');
  assert.equal(err.extras.offset, 4);
  assert.ok(err instanceof Error);
});

test('isFatalException은 SystemExit·GeneratorExit·fatal 표지만 참이다', () => {
  assert.equal(isFatalException(null), false);
  assert.equal(isFatalException('x'), false);
  assert.equal(isFatalException(new Error('no')), false);
  assert.equal(isFatalException(new FromE2eError('cli-error', 'x')), false);
  const systemExit = new Error('exit');
  systemExit.name = 'SystemExit';
  assert.equal(isFatalException(systemExit), true);
  const gen = new Error('gen');
  gen.name = 'GeneratorExit';
  assert.equal(isFatalException(gen), true);
  const oom = new Error('oom');
  oom.code = 'ERR_WORKER_OUT_OF_MEMORY';
  assert.equal(isFatalException(oom), true);
  const marked = new Error('marked');
  marked.fatal = true;
  assert.equal(isFatalException(marked), true);
  assert.ok(FATAL_ERROR_NAMES.includes('SystemExit'));
  assert.ok(FATAL_ERROR_NAMES.includes('GeneratorExit'));
});

test('exitCodeForKind는 카탈로그 밖을 unexpected(1)로 접는다', () => {
  assert.equal(exitCodeForKind('missing-arg'), 2);
  assert.equal(exitCodeForKind('parse-error'), 3);
  assert.equal(exitCodeForKind('csv-mismatch'), 4);
  assert.equal(exitCodeForKind('missing-bin'), 5);
  assert.equal(exitCodeForKind('missing-file'), 6);
  assert.equal(exitCodeForKind('unexpected'), 1);
  assert.equal(exitCodeForKind('not-a-kind'), 1);
  assert.equal(exitCodeForKind(undefined), 1);
});

test('truncateHead는 한도와 null을 자른다', () => {
  assert.equal(truncateHead(null), '');
  assert.equal(truncateHead(undefined), '');
  assert.equal(truncateHead('abcd', 2), 'ab');
  assert.equal(truncateHead('abcd', 4), 'abcd');
  assert.equal(truncateHead('abcd', 0), '');
  assert.equal(truncateHead(12), '12');
});

const nodeErrorCases = [
  { name: 'ENOENT io', code: 'ENOENT', context: 'io', kind: 'missing-file' },
  { name: 'ENOENT cli', code: 'ENOENT', context: 'cli', kind: 'missing-bin' },
  { name: 'ENOENT bin', code: 'ENOENT', context: 'bin', kind: 'missing-bin' },
  { name: 'EACCES', code: 'EACCES', context: 'io', kind: 'permission' },
  { name: 'EPERM', code: 'EPERM', context: 'io', kind: 'permission' },
  { name: 'ETIMEDOUT', code: 'ETIMEDOUT', context: 'cli', kind: 'timeout' },
  { name: 'EAGAIN', code: 'EAGAIN', context: 'io', kind: 'unexpected' },
  { name: 'ENOSPC', code: 'ENOSPC', context: 'io', kind: 'unexpected' },
  { name: 'ERR_FS', code: 'ERR_FS_FILE_TOO_LARGE', context: 'io', kind: 'os-error' },
];

for (const row of nodeErrorCases) {
  test(`classifyNodeError ${row.name} → ${row.kind}`, () => {
    const err = Object.assign(new Error(row.name), { code: row.code });
    assert.equal(classifyNodeError(err, row.context), row.kind);
  });
}

test('classifyNodeError status·타입·JSON·유니코드', () => {
  assert.equal(classifyNodeError(Object.assign(new Error('fail'), { status: 2 }), 'cli'), 'cli-error');
  assert.equal(classifyNodeError(new TypeError('t'), 'io'), 'type-error');
  assert.equal(classifyNodeError(new RangeError('r'), 'io'), 'type-error');
  assert.equal(classifyNodeError(new SyntaxError('s'), 'json'), 'json-error');
  assert.equal(classifyNodeError(new SyntaxError('s'), 'io'), 'parse-error');
  assert.equal(classifyNodeError(new URIError('u'), 'io'), 'decode-error');
  assert.equal(classifyNodeError(Object.assign(new Error('os'), { errno: -2 }), 'io'), 'os-error');
  assert.equal(classifyNodeError(new Error('mystery'), 'io'), 'unexpected');
  assert.equal(classifyNodeError(null, 'io'), 'unexpected');
  assert.equal(classifyNodeError(new Error('operation timeout'), 'cli'), 'timeout');
  assert.equal(classifyNodeError(new FromE2eError('csv-mismatch', 'x'), 'io'), 'csv-mismatch');
});

test('wrapNodeError는 FromE2eError를 다시 감싸지 않는다', () => {
  const inner = new FromE2eError('parse-error', '그대로');
  assert.equal(wrapNodeError(inner, 'io', 'x'), inner);
  const wrapped = wrapNodeError(Object.assign(new Error('nope'), { code: 'ENOENT' }), 'io', 'ghost.mjs');
  assert.ok(wrapped instanceof FromE2eError);
  assert.equal(wrapped.kind, 'missing-file');
  assert.match(wrapped.message, /ghost\.mjs/);
});

test('exceptionReport는 치명 예외를 fatal로 표지한다', () => {
  const fatal = new Error('stop');
  fatal.name = 'SystemExit';
  const report = exceptionReport(fatal, 'main');
  assert.equal(report.fatal, true);
  assert.equal(report.error, 'SystemExit');
  const normal = exceptionReport(new FromE2eError('missing-arg', USAGE), 'main');
  assert.equal(normal.fatal, false);
  assert.equal(normal.kind, 'missing-arg');
  assert.equal(normal.exit, 2);
  assert.match(formatExceptionReport(normal), /from_e2e missing-arg/);
  const extras = exceptionReport(new FromE2eError('csv-mismatch', 'x', { series: 1 }), 'csv');
  assert.equal(extras.extras.series, 1);
  const other = exceptionReport(new Error('plain'), 'io');
  assert.deepEqual(other.extras, {});
});

test('parseFromE2eArgv는 필수 인자를 검사한다', () => {
  throwsKind(() => parseFromE2eArgv(['node', 'from_e2e.mjs']), 'missing-arg');
  throwsKind(() => parseFromE2eArgv(['node', 'from_e2e.mjs', '--e2e', 'a.mjs']), 'missing-arg');
  throwsKind(() => parseFromE2eArgv(['node', 'from_e2e.mjs', '--id', 'ST01']), 'missing-arg');
  throwsKind(() => parseFromE2eArgv(['node', 'from_e2e.mjs', '--e2e', '--id', 'ST01']), 'missing-arg');
  throwsKind(() => parseFromE2eArgv(null), 'type-error');
  const parsed = parseFromE2eArgv([
    'node', 'from_e2e.mjs', '--e2e', 'a.mjs', '--id', 'ST09', '--pack', 'studio-e2e', '--bin', 'rhwp',
  ]);
  assert.deepEqual(parsed, { e2e: 'a.mjs', pack: 'studio-e2e', id: 'ST09', bin: 'rhwp' });
  const defaults = parseFromE2eArgv(['--e2e', 'a.mjs', '--id', 'ST09']);
  assert.equal(defaults.pack, 'studio-e2e');
  assert.equal(defaults.bin, 'target/debug/rhwp');
  assert.match(USAGE, /--e2e/);
});

test('assertTaskIdFormat은 ST01·AU13을 허용하고 소문자를 거부한다', () => {
  assert.doesNotThrow(() => assertTaskIdFormat('ST01'));
  assert.doesNotThrow(() => assertTaskIdFormat('AU13'));
  assert.doesNotThrow(() => assertTaskIdFormat('T01'));
  assert.doesNotThrow(() => assertTaskIdFormat('SE01'));
  throwsKind(() => assertTaskIdFormat('st01'), 'task-id-invalid');
  throwsKind(() => assertTaskIdFormat('ST1'), 'task-id-invalid');
  throwsKind(() => assertTaskIdFormat('ST001'), 'task-id-invalid');
  throwsKind(() => assertTaskIdFormat('TOOLONG01'), 'task-id-invalid');
  throwsKind(() => assertTaskIdFormat(''), 'task-id-invalid');
  assert.equal(TASK_ID_PATTERN.test('ST01'), true);
  assert.equal(TASK_ID_PATTERN.test('SE01'), true);
});

const samplePathCases = [
  { sample: 'chart/sample.hwp', ok: true },
  { sample: 'chart/세로막대형/묶은세로막대형.hwp', ok: true },
  { sample: 'a.hwpx', ok: true },
  { sample: 'A.HWP', ok: true },
  { sample: '../secret.hwp', ok: false, kind: 'path-escape' },
  { sample: 'chart/../../etc/passwd.hwp', ok: false, kind: 'path-escape' },
  { sample: '/tmp/x.hwp', ok: false, kind: 'path-escape' },
  { sample: 'chart/x.txt', ok: false, kind: 'validate-error' },
  { sample: 'chart/x.hwp\0', ok: false, kind: 'path-escape' },
  { sample: '', ok: false, kind: 'validate-error' },
];

for (const row of samplePathCases) {
  test(`assertSafeSamplePath ${JSON.stringify(row.sample)}`, () => {
    if (row.ok) assert.doesNotThrow(() => assertSafeSamplePath(row.sample));
    else throwsKind(() => assertSafeSamplePath(row.sample), row.kind);
  });
}

test('Windows 절대 샘플 경로는 path-escape 다', () => {
  throwsKind(() => assertSafeSamplePath('C:/abs/x.hwp'), 'path-escape');
});

test('assertCliReproducibleContract는 sample/chart/edit만 남긴다', () => {
  assert.doesNotThrow(() => assertCliReproducibleContract(validContract()));
  throwsKind(() => assertCliReproducibleContract(validContract({ menu: '차트' })), 'studio-only');
  throwsKind(() => assertCliReproducibleContract(validContract({ meta: { note: 'x' } })), 'studio-only');
  throwsKind(() => assertCliReproducibleContract(validContract({
    edit: { ...validEdit, locator: '#cell' },
  })), 'studio-only');
  throwsKind(() => assertCliReproducibleContract(validContract({
    edit: { series: 0, point: 0, from: '4.3', to: '4.3' },
  })), 'validate-error');
});

for (const key of STUDIO_ONLY_KEYS) {
  test(`assertCliReproducibleContract는 스튜디오 키 ${key} 를 거부한다`, () => {
    throwsKind(() => assertCliReproducibleContract(validContract({ [key]: 1 })), 'studio-only');
  });
}

test('collectForbiddenKeys는 중첩 경로를 모은다', () => {
  assert.deepEqual(collectForbiddenKeys({ edit: { undo: true } }, STUDIO_ONLY_KEYS), ['edit.undo']);
  assert.deepEqual(collectForbiddenKeys(null, STUDIO_ONLY_KEYS), []);
  assert.deepEqual(collectUnknownKeys({ sample: 'a', extra: 1 }, ALLOWED_CONTRACT_KEYS), ['extra']);
});

test('허용 키·명령 카탈로그는 차트 왕복만 담는다', () => {
  assert.deepEqual([...ALLOWED_CONTRACT_KEYS], ['sample', 'chart', 'edit']);
  assert.deepEqual([...ALLOWED_EDIT_KEYS], ['series', 'point', 'from', 'to']);
  assert.deepEqual([...ALLOWED_CLI_COMMANDS], ['chart-to-csv', 'csv-to-chart']);
  assert.deepEqual([...ALLOWED_CHECK_OPS], ['file_exists', 'differs_from_input', 'value_eq']);
  assert.ok(ALLOWED_SAMPLE_SUFFIXES.includes('.hwp'));
  assert.ok(ALLOWED_SAMPLE_SUFFIXES.includes('.hwpx'));
  for (const command of FORBIDDEN_CLI_COMMANDS) {
    assert.ok(!ALLOWED_CLI_COMMANDS.includes(command), command);
  }
});

test('honestyPolicy는 eval·e2e 실행을 하지 않는다고 선언한다', () => {
  const policy = honestyPolicy();
  assert.equal(policy.executesE2e, false);
  assert.equal(policy.usesEval, false);
  assert.equal(policy.usesFunction, false);
  assert.equal(policy.liveOracle, true);
  assert.deepEqual(policy.allowedCliCommands, ['chart-to-csv', 'csv-to-chart']);
  assert.ok(policy.studioOnlyKeys.includes('undo'));
  assert.ok(policy.errorKinds.includes('studio-only'));
});

const envelopeCases = [
  { name: '정상', env: { ok: true, charts: [{ csv: ',s1\nr,1\n' }] }, ok: true, csv: ',s1\nr,1\n' },
  { name: 'ok 생략', env: { charts: [{ csv: ',s1\nr,1\n' }] }, ok: true, csv: ',s1\nr,1\n' },
  { name: 'ok false', env: { ok: false, charts: [{ csv: ',s1\nr,1\n' }] }, kind: 'cli-error' },
  { name: 'null', env: null, kind: 'envelope-error' },
  { name: '배열', env: [], kind: 'envelope-error' },
  { name: 'charts 없음', env: { ok: true }, kind: 'envelope-error' },
  { name: 'charts 빈값', env: { ok: true, charts: [] }, kind: 'envelope-error' },
  { name: 'charts[0] 문자열', env: { ok: true, charts: ['x'] }, kind: 'envelope-error' },
  { name: 'csv 없음', env: { ok: true, charts: [{}] }, kind: 'envelope-error' },
  { name: 'csv 빈값', env: { ok: true, charts: [{ csv: '' }] }, kind: 'envelope-error' },
];

for (const row of envelopeCases) {
  test(`extractChartCsvFromEnvelope ${row.name}`, () => {
    if (row.ok) assert.equal(extractChartCsvFromEnvelope(row.env), row.csv);
    else throwsKind(() => extractChartCsvFromEnvelope(row.env), row.kind);
  });
}

test('parseChartToCsvStdout은 BOM·공백을 허용하고 빈 출력은 거부한다', () => {
  const env = parseChartToCsvStdout('\uFEFF{"ok":true,"charts":[{"csv":"a"}]}\n');
  assert.equal(env.charts[0].csv, 'a');
  throwsKind(() => parseChartToCsvStdout(''), 'envelope-error');
  throwsKind(() => parseChartToCsvStdout('not-json'), 'envelope-error');
  throwsKind(() => parseChartToCsvStdout(12), 'type-error');
});

for (const row of [
  { name: 'HTML', stdout: '<html>no</html>' },
  { name: '부분 JSON', stdout: '{"ok":true' },
  { name: '숫자', stdout: '12' },
  { name: '문자열 JSON', stdout: '"hello"' },
  { name: 'true', stdout: 'true' },
]) {
  test(`parseChartToCsvStdout ${row.name} 은 봉투가 아니다`, () => {
    assert.throws(() => {
      const env = parseChartToCsvStdout(row.stdout);
      extractChartCsvFromEnvelope(env);
    }, err => err instanceof FromE2eError);
  });
}

test('invokeChartToCsv는 목 exec로 봉투를 읽는다', () => {
  const csv = invokeChartToCsv('rhwp', 'samples/a.hwp', 1, {
    execFileSync: () => JSON.stringify({ ok: true, charts: [{ csv: ',s1\nr,4.3\n' }] }),
  });
  assert.equal(csv, ',s1\nr,4.3\n');
});

test('invokeChartToCsv는 없는 바이너리·권한·비정상 종료를 접는다', () => {
  throwsKind(() => invokeChartToCsv('', 'samples/a.hwp', 1, { execFileSync: () => '' }), 'missing-bin');
  throwsKind(() => invokeChartToCsv('rhwp', 'samples/a.hwp', 1, {
    execFileSync: () => { const e = new Error('no'); e.code = 'ENOENT'; throw e; },
  }), 'missing-bin');
  throwsKind(() => invokeChartToCsv('rhwp', 'samples/a.hwp', 1, {
    execFileSync: () => { const e = new Error('fail'); e.status = 2; throw e; },
  }), 'cli-error');
  throwsKind(() => invokeChartToCsv('rhwp', 'samples/a.hwp', 1, {
    execFileSync: () => { const e = new Error('denied'); e.code = 'EACCES'; throw e; },
  }), 'permission');
});

test('locateGymContract는 export const 표지만 인정한다', () => {
  const src = "export const gymContract = {\n  sample: 'a.hwp'\n};\n";
  const loc = locateGymContract(src, 'x.mjs');
  assert.equal(src[loc.objectStart], '{');
  throwsKind(() => locateGymContract('const gymContract = { sample: 1 }', 'x.mjs'), 'missing-contract');
  throwsKind(() => locateGymContract('export let gymContract = { sample: 1 }', 'x.mjs'), 'missing-contract');
  throwsKind(() => locateGymContract(1, 'x.mjs'), 'type-error');
});

test('parseContractLiteral은 원문이 문자열이 아니면 type-error 다', () => {
  throwsKind(() => parseContractLiteral(null), 'type-error');
  throwsKind(() => parseContractLiteral({ sample: 'a' }), 'type-error');
});

const parseRejectCases = [
  { name: '미닫힘 객체', src: '{ sample: "a.hwp"', re: /키가 필요하다|',' 또는 '}'가 필요하다/ },
  { name: '콜론 없음', src: '{ sample "a.hwp" }', re: /뒤에 ':'가 필요하다/ },
  { name: '값 없음', src: '{ sample: }', re: /객체·문자열·숫자 이외/ },
  { name: '선행 점 숫자', src: '{ n: .5 }', re: /객체·문자열·숫자 이외/ },
  { name: '16진수', src: '{ n: 0x10 }', re: /키 'n' 뒤에 ',' 또는 '}'가 필요하다/ },
  { name: 'Infinity', src: '{ n: Infinity }', re: /객체·문자열·숫자 이외/ },
  { name: 'NaN', src: '{ n: NaN }', re: /객체·문자열·숫자 이외/ },
  { name: 'undefined', src: '{ n: undefined }', re: /객체·문자열·숫자 이외/ },
  { name: '정규식', src: '{ n: /x/ }', re: /객체·문자열·숫자 이외/ },
  { name: '괄호 식', src: '{ n: (1) }', re: /객체·문자열·숫자 이외/ },
  { name: '스프레드', src: '{ ...x }', re: /객체 키가 필요하다/ },
  { name: '계산된 키', src: '{ ["sample"]: "a" }', re: /객체 키가 필요하다/ },
  { name: '함수', src: '{ f() {} }', re: /뒤에 ':'가 필요하다/ },
  { name: '화살표', src: '{ n: () => 1 }', re: /객체·문자열·숫자 이외/ },
  { name: 'new', src: '{ n: new Date() }', re: /객체·문자열·숫자 이외/ },
  { name: '삼항', src: '{ n: 1 ? 2 : 3 }', re: /',' 또는 '}'가 필요하다/ },
  { name: '곱셈', src: '{ n: 1*2 }', re: /',' 또는 '}'가 필요하다/ },
  { name: '비트', src: '{ n: 1|2 }', re: /',' 또는 '}'가 필요하다/ },
  { name: 'await', src: '{ sample: await 1 }', re: /객체·문자열·숫자 이외/ },
  { name: 'typeof', src: '{ n: typeof 1 }', re: /객체·문자열·숫자 이외/ },
  { name: 'void', src: '{ n: void 0 }', re: /객체·문자열·숫자 이외/ },
  { name: 'class', src: '{ sample: class X {} }', re: /객체·문자열·숫자 이외/ },
  { name: '배열 중첩', src: '{ edit: { xs: [1] } }', re: /객체·문자열·숫자 이외/ },
  { name: '세미콜론 키', src: '{ sample: "a"; chart: 1 }', re: /',' 또는 '}'가 필요하다/ },
  { name: '허용 안 된 escape', src: '{ sample: "\\0" }', re: /허용되지 않은 문자열 escape/ },
];

for (const row of parseRejectCases) {
  test(`파서는 ${row.name} 을 거부한다`, () => {
    assert.throws(() => parseContractLiteral(row.src), row.re);
  });
}

const parseAcceptCases = [
  { name: '과학적 표기', src: '{ n: 1e3 }', check: c => c.n === 1000 },
  { name: '음수 지수', src: '{ n: -2.5e-1 }', check: c => c.n === -0.25 },
  { name: '음수 영', src: '{ n: -0 }', check: c => Object.is(c.n, -0) || c.n === 0 },
  { name: '빈 문자열', src: '{ sample: "" }', check: c => c.sample === '' },
  { name: '작은따옴표 포함', src: '{ sample: "it\'s" }', check: c => c.sample === "it's" },
  { name: '개행 escape', src: '{ sample: "a\\nb" }', check: c => c.sample === 'a\nb' },
  { name: '탭 escape', src: '{ sample: "a\\tb" }', check: c => c.sample === 'a\tb' },
  { name: '후행 쉼표만', src: '{ sample: "a.hwp", }', check: c => c.sample === 'a.hwp' },
  { name: '중첩 빈 객체', src: '{ extra: {} }', check: c => c.extra && Object.keys(c.extra).length === 0 },
  { name: '식별자 키 $', src: '{ $id: 1 }', check: c => c.$id === 1 },
  { name: '_ 키', src: '{ _x: 2 }', check: c => c._x === 2 },
];

for (const row of parseAcceptCases) {
  test(`파서는 ${row.name} 을 읽는다`, () => {
    const value = parseContractLiteral(row.src);
    assert.ok(row.check(value), JSON.stringify(value));
  });
}

const unicodeSyllables = [
  ['AC00', '가'],
  ['B098', '나'],
  ['B2E4', '다'],
  ['B77C', '라'],
  ['B9C8', '마'],
  ['BC14', '바'],
  ['C0AC', '사'],
  ['C544', '아'],
  ['C790', '자'],
  ['CC28', '차'],
];

for (const [hex, ch] of unicodeSyllables) {
  test(`Unicode \\u${hex} 는 ${ch} 로 읽힌다`, () => {
    const contract = parseContractLiteral(
      `{ sample: "\\u${hex}/x.hwp", chart: 1, edit: { series: 0, point: 0, from: "1", to: "2" } }`,
    );
    assert.equal(contract.sample, `${ch}/x.hwp`);
    validateContract(contract);
  });
}

test('validateContract는 계약이 객체가 아니면 거부한다', () => {
  throwsKind(() => validateContract(null), 'validate-error');
  throwsKind(() => validateContract([]), 'validate-error');
  throwsKind(() => validateContract('x'), 'validate-error');
});

test('validateContract는 chart 문자열·NaN을 거부한다', () => {
  throwsKind(() => validateContract(validContract({ chart: '1' })), 'validate-error');
  throwsKind(() => validateContract(validContract({ chart: NaN })), 'validate-error');
  throwsKind(() => validateContract(validContract({ chart: Infinity })), 'validate-error');
});

test('validateContract는 series 소수·from 객체를 거부한다', () => {
  throwsKind(() => validateContract(validContract({ edit: { ...validEdit, series: 1.2 } })), 'validate-error');
  throwsKind(() => validateContract(validContract({ edit: { ...validEdit, from: { n: 1 } } })), 'validate-error');
});

test('applyCsvEdit는 형·모양 오류를 종류별로 낸다', () => {
  throwsKind(() => applyCsvEdit(null, validEdit), 'type-error');
  throwsKind(() => applyCsvEdit(',s1\nr,1\n', null), 'type-error');
  throwsKind(() => applyCsvEdit(',s1\nr,1\n', []), 'type-error');
  throwsKind(() => applyCsvEdit('', validEdit), 'csv-shape');
  throwsKind(() => applyCsvEdit(',s1\nr,1\n', { series: 5, point: 0, from: '1', to: '2' }), 'csv-missing-col');
  throwsKind(() => applyCsvEdit(',s1\na,1\nb,1,2\n', validEdit), 'csv-shape');
});

test('splitCsvRows/joinCsvRows는 LF로 왕복한다', () => {
  const text = ',s1,s2\nr,1,2\n';
  const rows = splitCsvRows(text);
  assert.deepEqual(rows, [['', 's1', 's2'], ['r', '1', '2']]);
  assert.equal(joinCsvRows(rows), text);
  assertCsvRectangular(rows);
});

test('assertCsvRectangular는 폭 1 헤더를 거부한다', () => {
  throwsKind(() => assertCsvRectangular([['only']]), 'csv-shape');
  throwsKind(() => assertCsvRectangular([]), 'csv-shape');
});

test('applyCsvEdit는 두 번째 계열 칸만 바꾼다', () => {
  const out = applyCsvEdit(',s1,s2\na,1,2\nb,3,4\n', { series: 1, point: 1, from: '4', to: '9' });
  assert.equal(out, ',s1,s2\na,1,2\nb,3,9\n');
});

test('applyCsvEdit는 숫자 0 from을 문자열 0과 대조한다', () => {
  const out = applyCsvEdit(',s1\nr,0\n', { series: 0, point: 0, from: 0, to: 1 });
  assert.equal(out, ',s1\nr,1\n');
});

const csvGrid = [
  ['10', '20', '30'],
  ['11', '21', '31'],
  ['12', '22', '32'],
  ['13', '23', '33'],
];

for (let series = 0; series < 3; series += 1) {
  for (let point = 0; point < 4; point += 1) {
    test(`applyCsvEdit 계열 ${series} 값 ${point} 칸만 바꾼다`, () => {
      const header = ',s1,s2,s3';
      const body = csvGrid.map((row, idx) => `p${idx},${row.join(',')}`);
      const base = `${[header, ...body].join('\n')}\n`;
      const from = csvGrid[point][series];
      const out = applyCsvEdit(base, { series, point, from, to: '99' });
      const rows = splitCsvRows(out);
      assert.equal(rows[1 + point][1 + series], '99');
      for (let p = 0; p < 4; p += 1) {
        for (let s = 0; s < 3; s += 1) {
          if (p === point && s === series) continue;
          assert.equal(rows[1 + p][1 + s], csvGrid[p][s]);
        }
      }
    });
  }
}

test('materializeFromContract는 ST01 형태 산출을 조립한다', () => {
  const chartCsv = [',계열 1,계열 2,계열 3', '항목 1,4.3,2.4,2', '항목 2,2.5,4.4,2', ''].join('\n');
  const out = materializeFromContract({
    taskId: 'ST09',
    e2ePath: 'rhwp-studio/e2e/demo.test.mjs',
    contract: {
      sample: 'chart/세로막대형/묶은세로막대형.hwp',
      chart: 1,
      edit: { series: 0, point: 0, from: '4.3', to: '91.7' },
    },
    csvAsset: 'gym/packs/studio-e2e/assets/ST09-edit.csv',
    chartCsv,
  });
  assert.match(out.editCsv, /^,계열 1,계열 2,계열 3\n항목 1,91\.7,2\.4,2\n/);
  assert.equal(out.task.id, 'ST09');
  assert.equal(out.task.checks[2].cmd[0], 'csv-to-chart');
  assert.equal(out.reference.steps[0].run[0], 'csv-to-chart');
  assertTaskChecksAreCliReproducible(out.task);
  assertReferenceIsCliReproducible(out.reference);
});

test('materializeFromContract는 스튜디오 전용·경로 탈출·잘못된 ID를 거부한다', () => {
  const chartCsv = ',s1\nr,4.3\n';
  throwsKind(() => materializeFromContract({
    taskId: 'ST09', e2ePath: 'x.mjs', contract: validContract({ undo: true }),
    csvAsset: 'gym/packs/studio-e2e/assets/ST09-edit.csv', chartCsv,
  }), 'studio-only');
  throwsKind(() => materializeFromContract({
    taskId: 'ST09', e2ePath: 'x.mjs', contract: validContract({ sample: '../x.hwp' }),
    csvAsset: 'gym/packs/studio-e2e/assets/ST09-edit.csv', chartCsv,
  }), 'path-escape');
  throwsKind(() => materializeFromContract({
    taskId: 'bad', e2ePath: 'x.mjs', contract: validContract(),
    csvAsset: 'gym/packs/studio-e2e/assets/bad-edit.csv', chartCsv,
  }), 'task-id-invalid');
});

test('assertTaskChecksAreCliReproducible는 금지 명령을 거부한다', () => {
  const task = buildTask({
    taskId: 'ST01', e2ePath: st01E2e, contract: st01Contract, csvAsset: st01CsvAsset,
  });
  assert.doesNotThrow(() => assertTaskChecksAreCliReproducible(task));
  throwsKind(() => assertTaskChecksAreCliReproducible({
    ...task, checks: [{ ...task.checks[0], op: 'screenshot_eq' }],
  }), 'studio-only');
  throwsKind(() => assertTaskChecksAreCliReproducible({
    ...task,
    checks: [{ name: 'x', op: 'value_eq', path: 'a', value: 1, cmd: ['fill-fields', '{file:out.hwp}'] }],
  }), 'studio-only');
  throwsKind(() => assertTaskChecksAreCliReproducible({
    ...task,
    checks: [{ name: 'x', op: 'value_eq', path: 'a', value: 1, cmd: ['export-pdf', '{file:out.hwp}'] }],
  }), 'studio-only');
});

test('assertReferenceIsCliReproducible는 한 스텝 csv-to-chart만 받는다', () => {
  const reference = buildReference({
    taskId: 'ST01', contract: st01Contract, csvAsset: st01CsvAsset,
  });
  assert.doesNotThrow(() => assertReferenceIsCliReproducible(reference));
  throwsKind(() => assertReferenceIsCliReproducible({ id: 'ST01', steps: [] }), 'validate-error');
  throwsKind(() => assertReferenceIsCliReproducible({
    id: 'ST01', steps: [{ run: ['table-to-csv', '{input}'] }],
  }), 'studio-only');
});

test('dumpJson은 마지막 개행과 2칸 들여쓰기를 쓴다', () => {
  const text = dumpJson({ a: 1 });
  assert.equal(text, '{\n  "a": 1\n}\n');
  assert.doesNotMatch(text, /\r/);
});

test('없는 e2e 파일은 missing-file 이다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-nofile-'));
  try {
    throwsKind(() => readContract(dir, 'ghost.test.mjs'), 'missing-file');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('readJsonFileOrThrow는 깨진 JSON을 json-error로 접는다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-json-'));
  const file = path.join(dir, 'broken.json');
  writeFileSync(file, '{ not json', 'utf8');
  try {
    throwsKind(() => readJsonFileOrThrow(file, 'demo/broken.json'), 'json-error');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('assertTaskIdAvailable는 깨진 과제 JSON을 json-error로 접는다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-packjson-'));
  const tasks = path.join(dir, 'gym', 'packs', 'other', 'tasks');
  mkdirSync(tasks, { recursive: true });
  writeFileSync(path.join(tasks, 'XX01.json'), '{', 'utf8');
  try {
    throwsKind(() => assertTaskIdAvailable(dir, 'studio-e2e', 'XX01'), 'json-error');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('assertTaskIdAvailable는 id 없는 JSON을 소유자로 치지 않는다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-noid-'));
  const tasks = path.join(dir, 'gym', 'packs', 'other', 'tasks');
  mkdirSync(tasks, { recursive: true });
  writeFileSync(path.join(tasks, 'XX01.json'), '{"title":"no-id"}\n', 'utf8');
  try {
    assert.doesNotThrow(() => assertTaskIdAvailable(dir, 'studio-e2e', 'XX01'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('runAsCli는 인자 오류를 exit 2로 접고 치명 예외는 다시 올린다', () => {
  const errs = [];
  const result = runAsCli(['node', 'from_e2e.mjs'], {
    log() {},
    err: (...args) => errs.push(args.join(' ')),
  });
  assert.equal(result.ok, false);
  assert.equal(result.exit, 2);
  assert.match(errs.join('\n'), /missing-arg/);
  const fatal = new Error('stop');
  fatal.name = 'SystemExit';
  fatal.fatal = true;
  assert.throws(() => wrapNodeError(fatal, 'cli', 'rhwp'), /stop/);
  assert.throws(() => invokeChartToCsv('rhwp', 'samples/a.hwp', 1, {
    execFileSync() { throw fatal; },
  }), /stop/);
});

test('runAsCli dryRun은 목 chart-to-csv로 과제를 조립한다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-cli-'));
  writeFileSync(path.join(dir, 'demo.test.mjs'), `
export const gymContract = {
  sample: 'chart/sample.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: '4.3', to: '91.7' },
};
`);
  try {
    const result = runAsCli(
      ['node', 'from_e2e.mjs', '--e2e', 'demo.test.mjs', '--id', 'ST09', '--pack', 'studio-e2e'],
      { log() {}, err() {} },
      {
        cwd: dir,
        dryRun: true,
        execFileSync: () => JSON.stringify({ ok: true, charts: [{ csv: ',s1\nrow,4.3\n' }] }),
      },
    );
    assert.equal(result.ok, true);
    assert.equal(result.exit, 0);
    assert.equal(result.task.id, 'ST09');
    assert.equal(result.task.input, 'samples/chart/sample.hwp');
    assert.equal(result.reference.steps[0].run[0], 'csv-to-chart');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('runAsCli는 ok=false 봉투를 성공으로 위장하지 않는다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-okfalse-'));
  writeFileSync(path.join(dir, 'demo.test.mjs'), `
export const gymContract = {
  sample: 'chart/sample.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: '4.3', to: '91.7' },
};
`);
  try {
    const result = runAsCli(['--e2e', 'demo.test.mjs', '--id', 'ST09'], { log() {}, err() {} }, {
      cwd: dir,
      dryRun: true,
      execFileSync: () => JSON.stringify({ ok: false, charts: [{ csv: ',s1\nrow,4.3\n' }] }),
    });
    assert.equal(result.ok, false);
    assert.equal(result.report.kind, 'cli-error');
    assert.equal(result.exit, 5);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('runAsCli는 gymContract 없는 파일을 missing-contract 로 접는다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-nocon-'));
  writeFileSync(path.join(dir, 'empty.test.mjs'), 'export const other = 1;\n');
  try {
    const result = runAsCli(['--e2e', 'empty.test.mjs', '--id', 'ST09'], { log() {}, err() {} }, {
      cwd: dir, dryRun: true,
    });
    assert.equal(result.ok, false);
    assert.equal(result.report.kind, 'missing-contract');
    assert.equal(result.exit, 3);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('runAsCli는 스튜디오 키 계약을 studio-only 로 접는다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-studio-'));
  writeFileSync(path.join(dir, 'ui.test.mjs'), `
export const gymContract = {
  sample: 'chart/sample.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: '4.3', to: '91.7' },
  undo: 1,
};
`);
  try {
    const result = runAsCli(['--e2e', 'ui.test.mjs', '--id', 'ST09'], { log() {}, err() {} }, {
      cwd: dir, dryRun: true,
    });
    assert.equal(result.ok, false);
    assert.equal(result.report.kind, 'studio-only');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('runAsCli는 샘플 경로 탈출을 path-escape 로 접는다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-esc-'));
  writeFileSync(path.join(dir, 'esc.test.mjs'), `
export const gymContract = {
  sample: '../secret.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: '4.3', to: '91.7' },
};
`);
  try {
    const result = runAsCli(['--e2e', 'esc.test.mjs', '--id', 'ST09'], { log() {}, err() {} }, {
      cwd: dir, dryRun: true,
    });
    assert.equal(result.ok, false);
    assert.equal(result.report.kind, 'path-escape');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('readContract는 let/var gymContract를 계약으로 인정하지 않는다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-let-'));
  writeFileSync(path.join(dir, 'let.test.mjs'), "let gymContract = { sample: 'a.hwp', chart: 1 };\n");
  try {
    throwsKind(() => readContract(dir, 'let.test.mjs'), 'missing-contract');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('readContract는 빈 파일을 missing-contract 로 거부한다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-emptyfile-'));
  writeFileSync(path.join(dir, 'empty.test.mjs'), '');
  try {
    throwsKind(() => readContract(dir, 'empty.test.mjs'), 'missing-contract');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('buildTask 검사는 file_exists → differs_from_input → value_eq 순서다', () => {
  const task = buildTask({
    taskId: 'ST02',
    e2ePath: 'rhwp-studio/e2e/other.test.mjs',
    contract: { sample: 'chart/a.hwp', chart: 2, edit: { series: 1, point: 3, from: 8, to: 9 } },
    csvAsset: 'gym/packs/studio-e2e/assets/ST02-edit.csv',
  });
  assert.deepEqual(task.checks.map(item => item.op), ['file_exists', 'differs_from_input', 'value_eq']);
  assert.equal(task.input, 'samples/chart/a.hwp');
  assert.match(task.instructions, /차트 2의 \(계열 1, 값 3\) 원본 8/);
  assert.equal(task.checks[2].cmd[5], '2');
});

test('buildReference run 인자는 고정 순서다', () => {
  const reference = buildReference({
    taskId: 'ST03',
    contract: { sample: 'a.hwp', chart: 4, edit: { series: 0, point: 0, from: 1, to: 2 } },
    csvAsset: 'gym/packs/studio-e2e/assets/ST03-edit.csv',
  });
  assert.deepEqual(reference.steps[0].run, [
    'csv-to-chart', '{input}', '--csv', 'gym/packs/studio-e2e/assets/ST03-edit.csv',
    '--chart', '4', '-o', '{sub:out.hwp}', '--json',
  ]);
});

test('기존 파서 오류도 FromE2eError parse-error 다', () => {
  try {
    parseContractLiteral('{ sample: someVar }');
    assert.fail('should throw');
  } catch (err) {
    assert.ok(err instanceof FromE2eError);
    assert.equal(err.kind, 'parse-error');
    assert.match(err.message, /객체·문자열·숫자 이외의 식은 허용하지 않는다/);
  }
});

test('파서는 extra 키를 읽지만 정직 게이트가 막는다', () => {
  const contract = parseContractLiteral(`{
    sample: 'chart/sample.hwp',
    chart: 1,
    edit: { series: 0, point: 0, from: '1', to: '2' },
    hint: '파서는 읽는다',
  }`);
  validateContract(contract);
  assert.equal(contract.hint, '파서는 읽는다');
  throwsKind(() => assertCliReproducibleContract(contract), 'studio-only');
});

test('assertTaskIdAvailable는 저장소의 SE01 충돌을 계속 거부한다', () => {
  assert.throws(
    () => assertTaskIdAvailable(repoRoot, 'studio-e2e', 'SE01'),
    /security\/SE01\.json/,
  );
});
