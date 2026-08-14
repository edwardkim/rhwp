import assert from 'node:assert/strict';
import test from 'node:test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { assertTaskIdAvailable, parseContractLiteral, validateContract } from './from_e2e.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');

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
