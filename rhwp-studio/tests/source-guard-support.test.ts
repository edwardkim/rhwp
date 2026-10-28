import test from 'node:test';
import { codeOnly } from './support/source-guard.ts';
import assert from 'node:assert/strict';

// [#6335] codeOnly — 전문 pin 의 주석 인용 오염 방지 헬퍼의 계약.
// 실전 실패 모드 그대로 고정한다: 주석에 종전 선언을 인용한 채 실값을 바꾸면
// 원문 첫-매치는 통과(무검출)하지만 codeOnly 사본에서는 실패해야 한다.

const FIXTURE = [
  "// UI 조정 이력: 종전에는 const LIMIT = 8; 이었다.",
  "/* 블록 주석 안의 const BLOCK = 1; 도 제거된다 */",
  "const LIMIT = 12;",
  "const url = 'https://example.com/a'; // 문자열 뒤 주석",
  "const label = '주석처럼 보이는 문자열 // 은 보존';",
].join(String.fromCharCode(10));

test('주석 속 선언 인용은 매치되지 않는다 (디코이 차단)', () => {
  const code = codeOnly(FIXTURE);
  assert.doesNotMatch(code, /const LIMIT = 8;/, '줄 주석 인용이 살아 있으면 디코이 오염 재발');
  assert.doesNotMatch(code, /const BLOCK = 1;/, '블록 주석 인용이 살아 있으면 디코이 오염 재발');
  assert.match(FIXTURE, /const LIMIT = 8;/, '전제: 원문 첫-매치는 디코이에 걸린다(무검출 계급)');
});

test('실제 선언과 문자열은 보존된다', () => {
  const code = codeOnly(FIXTURE);
  assert.match(code, /const LIMIT = 12;/, '실선언은 남아야 한다');
  assert.match(code, /'https:..example\.com.a'/, '문자열 내부(//)는 주석으로 오인하지 않는다');
  assert.match(code, /은 보존/, '문자열 속 주석 유사 문자열은 보존');
  assert.doesNotMatch(code, /문자열 뒤 주석/, '문자열 밖 꼬리 주석은 제거');
});

test('줄 구조가 보존된다 — 진단 줄 번호 유지', () => {
  const nl = String.fromCharCode(10);
  assert.equal(codeOnly(FIXTURE).split(nl).length, FIXTURE.split(nl).length);
});
