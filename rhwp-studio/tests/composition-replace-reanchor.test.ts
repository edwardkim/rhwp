import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

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
