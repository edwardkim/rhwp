import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  applyCsvEdit,
  assertTaskIdAvailable,
  buildReference,
  buildTask,
  parseContractLiteral,
  readContract,
  validateContract,
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

test('허용된 gymContract 객체 리터럴을 실행하지 않고 읽는다', () => {
  const contract = parseContractLiteral(`{
    sample: 'chart/sample.hwp',
    chart: 1,
    edit: { series: 0, point: 2, from: '4.3', to: '91.7' }, // e2e 설명
  }`);
  validateContract(contract);
  assert.deepEqual(contract.edit, { series: 0, point: 2, from: '4.3', to: '91.7' });
});

test('실행 식을 gymContract 값으로 허용하지 않는다', () => {
  assert.throws(
    () => parseContractLiteral(`{
      sample: globalThis.process.exit(1),
      chart: 1,
      edit: { series: 0, point: 0, from: '4.3', to: '91.7' },
    }`),
    /객체·문자열·숫자 이외의 식은 허용하지 않는다/,
  );
});

test('다른 pack에 이미 있는 과제 ID는 생성 전에 거부한다', () => {
  assert.throws(
    () => assertTaskIdAvailable(repoRoot, 'studio-e2e', 'SE01'),
    /security\/SE01\.json/,
  );
  assert.doesNotThrow(() => assertTaskIdAvailable(repoRoot, 'studio-e2e', 'ST01'));
});

test('Unicode escape \\uAC00을 한글 음절로 읽는다', () => {
  const contract = parseContractLiteral(`{
    sample: "\\uAC00/file.hwp",
    chart: 1,
    edit: { series: 0, point: 0, from: "1", to: "2" }
  }`);
  assert.equal(contract.sample, '가/file.hwp');
  validateContract(contract);
});

test('블록 주석과 줄 주석을 값으로 취급하지 않는다', () => {
  const contract = parseContractLiteral(`{
    /* 샘플 경로 */
    sample: 'chart/sample.hwp', // 한 줄
    chart: 1,
    /* 중첩 객체 앞 */
    edit: {
      series: 0, // 첫 계열
      point: 0,
      from: '4.3',
      to: '91.7',
    },
  }`);
  validateContract(contract);
  assert.equal(contract.sample, 'chart/sample.hwp');
  assert.deepEqual(contract.edit, { series: 0, point: 0, from: '4.3', to: '91.7' });
});

test('따옴표 키와 중첩 객체를 읽는다', () => {
  const contract = parseContractLiteral(`{
    "sample": 'chart/sample.hwp',
    'chart': 2,
    "edit": { 'series': 1, "point": 3, from: 4.3, to: 91.7 },
    meta: { note: "nested", n: 2 },
  }`);
  validateContract(contract);
  assert.equal(contract.chart, 2);
  assert.equal(contract.edit.series, 1);
  assert.equal(contract.edit.point, 3);
  assert.equal(contract.edit.from, 4.3);
  assert.equal(contract.meta.note, 'nested');
  assert.equal(contract.meta.n, 2);
});

test('빈 객체는 파싱되나 validateContract에서 거부한다', () => {
  const contract = parseContractLiteral('{}');
  assert.deepEqual(contract, {});
  assert.throws(() => validateContract(contract), /gymContract\.sample은 비어 있지 않은 문자열이어야 한다/);
});

test('배열 값은 거부한다', () => {
  assert.throws(
    () => parseContractLiteral(`{ sample: ['chart/sample.hwp'] }`),
    /객체·문자열·숫자 이외의 식은 허용하지 않는다/,
  );
});

test('식별자 값은 거부한다', () => {
  assert.throws(
    () => parseContractLiteral(`{ sample: someVar, chart: 1 }`),
    /객체·문자열·숫자 이외의 식은 허용하지 않는다/,
  );
});

test('템플릿 리터럴은 거부한다', () => {
  assert.throws(
    () => parseContractLiteral('{ sample: `chart/sample.hwp`, chart: 1 }'),
    /객체·문자열·숫자 이외의 식은 허용하지 않는다/,
  );
});

test('중복 키는 거부한다', () => {
  assert.throws(
    () => parseContractLiteral(`{ sample: 'a.hwp', sample: 'b.hwp', chart: 1 }`),
    /중복 키 'sample'는 허용하지 않는다/,
  );
});

test('닫히지 않은 문자열은 거부한다', () => {
  assert.throws(
    () => parseContractLiteral(`{ sample: 'oops, chart: 1 }`),
    /문자열이 닫히지 않았다/,
  );
});

test('잘못된 Unicode escape는 거부한다', () => {
  assert.throws(
    () => parseContractLiteral(`{ sample: "\\uZZZZ.hwp" }`),
    /유효하지 않은 Unicode escape다/,
  );
  assert.throws(
    () => parseContractLiteral(`{ sample: "\\u12" }`),
    /유효하지 않은 Unicode escape다/,
  );
});

test('객체 뒤 추가 식은 거부한다', () => {
  assert.throws(
    () => parseContractLiteral(`{ sample: 'x.hwp' } + 1`),
    /뒤에 추가 식이 있다/,
  );
  assert.throws(
    () => parseContractLiteral(`{ sample: 'x.hwp' }; globalThis.process.exit(1)`),
    /뒤에 추가 식이 있다/,
  );
});

test('객체 뒤 주석만 있으면 허용한다', () => {
  const contract = parseContractLiteral(`{ sample: 'x.hwp', chart: 1, edit: { series: 0, point: 0, from: '1', to: '2' } } // tail`);
  validateContract(contract);
  assert.equal(contract.sample, 'x.hwp');
});

test('boolean·null·단항 플러스는 식이므로 거부한다', () => {
  assert.throws(() => parseContractLiteral('{ flag: true }'), /객체·문자열·숫자 이외의 식은 허용하지 않는다/);
  assert.throws(() => parseContractLiteral('{ flag: false }'), /객체·문자열·숫자 이외의 식은 허용하지 않는다/);
  assert.throws(() => parseContractLiteral('{ flag: null }'), /객체·문자열·숫자 이외의 식은 허용하지 않는다/);
  assert.throws(() => parseContractLiteral('{ n: +1 }'), /객체·문자열·숫자 이외의 식은 허용하지 않는다/);
});

test('최상위 문자열·숫자는 객체가 아니다', () => {
  assert.throws(() => parseContractLiteral(`'hello'`), /최상위 값은 객체여야 한다/);
  assert.throws(() => parseContractLiteral('12'), /최상위 값은 객체여야 한다/);
});

test('닫히지 않은 블록 주석은 거부한다', () => {
  assert.throws(() => parseContractLiteral('{ /* 미닫힘 sample: 1 }'), /블록 주석이 닫히지 않았다/);
});

test('허용되지 않은 문자열 escape는 거부한다', () => {
  assert.throws(() => parseContractLiteral(`{ sample: "\\x41" }`), /허용되지 않은 문자열 escape/);
});

test('validateContract는 sample이 없거나 빈 문자열이면 거부한다', () => {
  assert.throws(() => validateContract(validContract({ sample: '' })), /gymContract\.sample은 비어 있지 않은 문자열이어야 한다/);
  assert.throws(() => validateContract(validContract({ sample: 1 })), /gymContract\.sample은 비어 있지 않은 문자열이어야 한다/);
  const missing = validContract();
  delete missing.sample;
  assert.throws(() => validateContract(missing), /gymContract\.sample은 비어 있지 않은 문자열이어야 한다/);
});

test('validateContract는 chart가 1 미만이면 거부한다', () => {
  assert.throws(() => validateContract(validContract({ chart: 0 })), /gymContract\.chart는 1 이상의 정수여야 한다/);
  assert.throws(() => validateContract(validContract({ chart: -1 })), /gymContract\.chart는 1 이상의 정수여야 한다/);
  assert.throws(() => validateContract(validContract({ chart: 1.5 })), /gymContract\.chart는 1 이상의 정수여야 한다/);
});

test('validateContract는 edit가 객체가 아니면 거부한다', () => {
  assert.throws(() => validateContract(validContract({ edit: null })), /gymContract\.edit는 객체여야 한다/);
  assert.throws(() => validateContract(validContract({ edit: [] })), /gymContract\.edit는 객체여야 한다/);
  assert.throws(() => validateContract(validContract({ edit: 'nope' })), /gymContract\.edit는 객체여야 한다/);
});

test('validateContract는 series가 음수이면 거부한다', () => {
  assert.throws(
    () => validateContract(validContract({ edit: { ...validEdit, series: -1 } })),
    /gymContract\.edit\.series는 0 이상의 정수여야 한다/,
  );
});

test('validateContract는 from이 없으면 거부한다', () => {
  const edit = { series: 0, point: 0, to: '91.7' };
  assert.throws(
    () => validateContract(validContract({ edit })),
    /gymContract\.edit\.from는 문자열 또는 숫자여야 한다/,
  );
});

test('validateContract는 to·point 형식도 검사한다', () => {
  assert.throws(
    () => validateContract(validContract({ edit: { ...validEdit, to: undefined } })),
    /gymContract\.edit\.to는 문자열 또는 숫자여야 한다/,
  );
  assert.throws(
    () => validateContract(validContract({ edit: { ...validEdit, point: -3 } })),
    /gymContract\.edit\.point는 0 이상의 정수여야 한다/,
  );
});

test('applyCsvEdit는 지정 칸만 바꾸고 LF로 끝낸다', () => {
  const base = ',계열 1,계열 2,계열 3\n항목 1,4.3,2.4,2\n항목 2,2.5,4.4,2\n';
  const out = applyCsvEdit(base, { series: 0, point: 0, from: '4.3', to: '91.7' });
  assert.equal(out, ',계열 1,계열 2,계열 3\n항목 1,91.7,2.4,2\n항목 2,2.5,4.4,2\n');
  assert.match(out, /\n$/);
  assert.doesNotMatch(out, /\r/);
});

test('applyCsvEdit는 CRLF 입력을 LF로 정규화한다', () => {
  const base = ',s1,s2\r\nitem,1,2\r\n';
  const out = applyCsvEdit(base, { series: 1, point: 0, from: '2', to: '9' });
  assert.equal(out, ',s1,s2\nitem,1,9\n');
});

test('applyCsvEdit는 숫자 from/to를 문자열 칸과 대조한다', () => {
  const out = applyCsvEdit(',s1\nrow,4.3\n', { series: 0, point: 0, from: 4.3, to: 91.7 });
  assert.equal(out, ',s1\nrow,91.7\n');
});

test('applyCsvEdit는 from 불일치면 계약 오류다', () => {
  assert.throws(
    () => applyCsvEdit(',s1\nrow,4.3\n', { series: 0, point: 0, from: '9', to: '1' }),
    /계약 불일치: \(계열 0, 값 0\) 현재 '4\.3' ≠ from '9'/,
  );
});

test('applyCsvEdit는 없는 point 행을 거부한다', () => {
  assert.throws(
    () => applyCsvEdit(',s1\nrow,4.3\n', { series: 0, point: 5, from: '4.3', to: '1' }),
    /point 5 데이터 행이 없다/,
  );
});

test('applyCsvEdit는 ST01 원본 한 칸만 91.7로 바꾼다', () => {
  const base = [
    ',계열 1,계열 2,계열 3',
    '항목 1,4.3,2.4,2',
    '항목 2,2.5,4.4,2',
    '항목 3,3.5,1.8,3',
    '항목 4,4.5,2.8,5',
    '',
  ].join('\n');
  const out = applyCsvEdit(base, { series: 0, point: 0, from: '4.3', to: '91.7' });
  assert.equal(out, [
    ',계열 1,계열 2,계열 3',
    '항목 1,91.7,2.4,2',
    '항목 2,2.5,4.4,2',
    '항목 3,3.5,1.8,3',
    '항목 4,4.5,2.8,5',
    '',
  ].join('\n'));
});

const st01Contract = {
  sample: 'chart/세로막대형/묶은세로막대형.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: 4.3, to: 91.7 },
};
const st01CsvAsset = 'gym/packs/studio-e2e/assets/ST01-edit.csv';
const st01E2e = 'rhwp-studio/e2e/issue-4694-chart-data-edit.test.mjs';

test('buildTask는 file_exists·differs_from_input·value_eq changedCount 0 dry-run을 넣는다', () => {
  const task = buildTask({
    taskId: 'ST01',
    e2ePath: st01E2e,
    contract: st01Contract,
    csvAsset: st01CsvAsset,
  });
  assert.equal(task.id, 'ST01');
  assert.equal(task.tier, 3);
  assert.equal(task.title, '차트 데이터 편집 왕복 (studio issue-4694-chart-data-edit.test.mjs 파생)');
  assert.equal(task.input, 'samples/chart/세로막대형/묶은세로막대형.hwp');
  assert.equal(task.submit.kind, 'artifact');
  assert.deepEqual(task.submit.files, ['out.hwp']);
  assert.equal(task.checks.length, 3);
  assert.deepEqual(task.checks[0], { name: '산출물 존재', op: 'file_exists', file: 'out.hwp', minBytes: 1 });
  assert.deepEqual(task.checks[1], { name: '원본과 다름 (무편집 복사 거부)', op: 'differs_from_input', file: 'out.hwp' });
  const sentinel = task.checks[2];
  assert.equal(sentinel.op, 'value_eq');
  assert.equal(sentinel.path, 'changedCount');
  assert.equal(sentinel.value, 0);
  assert.match(sentinel.name, /91\.7/);
  assert.deepEqual(sentinel.cmd, [
    'csv-to-chart',
    '{file:out.hwp}',
    '--csv',
    st01CsvAsset,
    '--chart',
    '1',
    '--dry-run',
    '--json',
  ]);
  assert.match(task.instructions, /차트 1의 \(계열 0, 값 0\) 원본 4\.3/);
  assert.match(task.instructions, /csv-to-chart 로 되넣어라\(-o out\.hwp\)/);
});

test('buildReference run은 csv-to-chart {input} --csv ... -o {sub:out.hwp} 이다', () => {
  const reference = buildReference({
    taskId: 'ST01',
    contract: st01Contract,
    csvAsset: st01CsvAsset,
  });
  assert.deepEqual(reference, {
    id: 'ST01',
    steps: [{
      run: [
        'csv-to-chart',
        '{input}',
        '--csv',
        st01CsvAsset,
        '--chart',
        '1',
        '-o',
        '{sub:out.hwp}',
        '--json',
      ],
    }],
  });
});

test('assertTaskIdAvailable는 security가 가진 SE01을 계속 거부한다', () => {
  assert.throws(
    () => assertTaskIdAvailable(repoRoot, 'studio-e2e', 'SE01'),
    /과제 ID 'SE01' 가 다른 pack에 이미 있다: security\/SE01\.json/,
  );
});

test('assertTaskIdAvailable는 pack studio-e2e의 ST01을 허용한다', () => {
  assert.doesNotThrow(() => assertTaskIdAvailable(repoRoot, 'studio-e2e', 'ST01'));
});

test('assertTaskIdAvailable는 다른 pack의 ST01은 거부한다', () => {
  assert.throws(
    () => assertTaskIdAvailable(repoRoot, 'security', 'ST01'),
    /studio-e2e\/ST01\.json/,
  );
});

test('assertTaskIdAvailable는 gym/packs가 없으면 통과한다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-nopack-'));
  try {
    assert.doesNotThrow(() => assertTaskIdAvailable(dir, 'studio-e2e', 'SE01'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('readContract는 파일의 gymContract만 읽고 뒤 코드를 실행하지 않는다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-read-'));
  const rel = 'sample.test.mjs';
  writeFileSync(path.join(dir, rel), `
export const gymContract = {
  sample: 'chart/sample.hwp',
  chart: 1,
  edit: { series: 0, point: 0, from: '1', to: '2' },
};
throw new Error('이 파일은 실행되면 안 된다');
`);
  try {
    const contract = readContract(dir, rel);
    validateContract(contract);
    assert.equal(contract.sample, 'chart/sample.hwp');
    assert.equal(contract.edit.to, '2');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('readContract는 gymContract가 없으면 거부한다', () => {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'from-e2e-missing-'));
  const rel = 'no-contract.test.mjs';
  writeFileSync(path.join(dir, rel), 'export const other = { sample: "x" };\n');
  try {
    assert.throws(() => readContract(dir, rel), /export const gymContract/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
