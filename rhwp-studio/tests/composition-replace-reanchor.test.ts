import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import * as nodeModule from 'node:module';

// [#4149 계열 방어] IME 조합 업데이트의 raw replace 는 wasm 의 deferred replace 범위
// 가드에 거부될 수 있다(외부 변이로 앵커·길이가 낡은 경합). 거부가 onInput 밖으로
// 던져지면 핸들러가 죽고 조합 추적(compositionAnchor/Length)이 낡은 값으로 wedge 되어
// 이후 모든 조합 업데이트가 연쇄 실패한다. 조합 분기는 반드시 거부를 잡아 현재 캐럿에
// 재정박해야 한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = readFileSync(join(rootDir, 'src/engine/input-handler-text.ts'), 'utf8');

function compositionBranch(): string {
  const start = source.indexOf('if (this.isComposing && this.compositionAnchor) {');
  assert.notEqual(start, -1, '조합 분기를 찾지 못했다');
  const end = source.indexOf('// iOS 폴백', start);
  return source.slice(start, end === -1 ? start + 3000 : end);
}

test('조합 업데이트의 raw replace 는 try/catch 로 보호된다', () => {
  const branch = compositionBranch();
  const tryAt = branch.indexOf('try {');
  const replaceAt = branch.indexOf('this.replaceTextAtRaw(anchor, this.compositionLength, text)');
  assert.notEqual(replaceAt, -1, '조합 replace 호출이 없다');
  assert.ok(tryAt !== -1 && tryAt < replaceAt,
    '조합 replace 가 try 블록 밖에 있다 — 가드 거부가 onInput 을 죽인다');
});

test('거부 시 조합을 현재 캐럿에 재정박하고 길이를 리셋한다', () => {
  const branch = compositionBranch();
  assert.match(branch, /catch[\s\S]*?this\.compositionAnchor = anchor/,
    '재정박(compositionAnchor 갱신)이 없다');
  assert.match(branch, /catch[\s\S]*?this\.compositionLength = 0/,
    '재정박 시 조합 길이 리셋이 없다');
  assert.match(branch, /catch[\s\S]*?this\.cursor\.getPosition\(\)/,
    '재정박 기준이 현재 캐럿이 아니다');
});

// [#4150 리뷰] 위 두 테스트는 소스 정규식이라 "catch 가 있고 getPosition() 을 쓴다"만
// 확인하고, 머리말/꼬리말·각주 모드에서 getPosition() 이 진입 전 stale 본문 위치를
// 돌려준다는 사실은 잡지 못했다. onInput 을 mock this 로 직접 호출해 재정박이 실제로
// hfCharOffset/fnCharOffset 을 쓰는지 행위로 검증한다.

function transformTypesSupported(): boolean {
  return process.allowedNodeEnvironmentFlags.has('--experimental-transform-types')
    && typeof (nodeModule as { registerHooks?: unknown }).registerHooks === 'function';
}

test('조합 replace 거부 재정박은 머리말/꼬리말·각주 모드에서 stale 본문 offset 대신 hf/fnCharOffset 을 쓴다', (t) => {
  if (!transformTypesSupported()) {
    t.skip('현재 Node 가 --experimental-transform-types / registerHooks 미지원 — 행위 테스트 skip');
    return;
  }
  const runner = join(rootDir, 'tests', 'support', 'composition-hf-fn-reanchor.runner.mjs');
  const res = spawnSync(
    process.execPath,
    ['--experimental-transform-types', '--no-warnings', runner],
    { encoding: 'utf8' },
  );
  assert.equal(res.status, 0,
    `러너가 비정상 종료했습니다.\n--- stdout ---\n${res.stdout}\n--- stderr ---\n${res.stderr}`);
  assert.match(res.stdout, /COMPOSITION_HF_FN_REANCHOR_OK/, '행위 검증 성공 마커가 있어야 함');
});
