import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const runner = fileURLToPath(new URL('./support/review-runtime-contracts.runner.mjs', import.meta.url));
function run(mode: string, mutation = '') {
  const result = spawnSync(process.execPath,
    ['--experimental-transform-types', '--no-warnings', runner, mode, mutation],
    { encoding: 'utf8', timeout: 30_000 });
  assert.ifError(result.error);
  return result;
}

for (const mode of ['history-100', 'history-50', 'history-fallback', 'font']) {
  test(`제품 경로 동작 계약: ${mode} (#7020/#6600)`, () => {
    const result = run(mode);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stdout, /RUNTIME_CONTRACT_OK/);
  });
}

for (const [mode, mutation, diagnostic] of [
  ['history-50', 'fixed-budget', /CORE_EVICTION|BUDGET_EXCEEDED/],
  ['font', 'drop-engine-chain', /ENGINE_ALIAS_LOST/],
]) {
  test(`기존 결함 음성 대조: ${mutation}`, () => {
    const result = run(mode, mutation);
    assert.notEqual(result.status, 0, '잘못된 구현을 주입했는데 통과했다');
    assert.match(result.stderr, diagnostic);
  });
}
