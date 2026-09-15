/**
 * "이 화면에 이 문구가 나오는가" 를 소스에 대고 확인하는 도우미.
 *
 * 문자열이 언어팩으로 옮겨지면 소스에는 한국어 대신 키가 남는다. 그때도 단정의
 * 뜻은 그대로여야 한다 — 문구가 사라진 것이 아니라 카탈로그로 옮겨간 것이다.
 *
 * 통과 조건은 두 가지뿐이고 둘 다 '코드' 에 대한 것이다:
 *   (a) 소스의 문자열 리터럴에 그 문구가 있다(주석은 세지 않는다), 또는
 *   (b) 카탈로그에서 그 문구를 **담은** 키를 소스가 실제로 부른다.
 * 주석 일치로는 통과하지 않는다 — 그러면 문구를 지워도 초록이라 검사가 아니다.
 * (b) 가 '담은' 인 이유: 원래 단정이 긴 문장의 일부(`/바이트/`)를 보던 것이 많아, 값 전체 일치로
 * 좁히면 그 단정들을 전부 손으로 고쳐 써야 한다. 대신 '소스가 그 키를 부른다' 를 요구해 무관한
 * 키로 통과하는 일을 막는다.
 */
import assert from 'node:assert/strict';

import koCatalog from '../../src/i18n/locales/ko.ts';

const catalog = koCatalog as Record<string, string>;

/** 주석을 뺀 소스. 한 줄·블록 주석을 지워 주석 속 문구로 통과하는 일을 막는다. */
function withoutComments(source: string): string {
  return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:])\/\/[^\n]*/g, '$1');
}

/** 문구를 값에 담은 카탈로그 키 목록. 값 전체가 같은 키가 있으면 그것만 돌려준다. */
export function keysWithValue(text: string): string[] {
  const wanted = text.trim();
  const exact = Object.keys(catalog).filter((key) => catalog[key].trim() === wanted);
  if (exact.length > 0) return exact;
  return Object.keys(catalog).filter((key) => catalog[key].includes(wanted));
}

/** 소스가 그 문구를 화면에 낸다고 단정한다. 리터럴로 있든 같은 값의 키로 있든 통과한다. */
export function assertShowsText(source: string, text: string, message?: string): void {
  const code = withoutComments(source);
  const literal = new RegExp(`(['"\`])${text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`);
  if (literal.test(code)) return;
  const keys = keysWithValue(text);
  assert.ok(keys.length > 0, message ?? `카탈로그에 '${text}' 를 담은 키가 없다`);
  assert.ok(
    keys.some((key) => code.includes(`'${key}'`)),
    message ?? `소스가 '${text}' 의 키(${keys.slice(0, 3).join(', ')})를 부르지 않는다`,
  );
}

/** 소스가 그 문구를 **내지 않는다**고 단정한다. 리터럴도 없고, 같은 값의 키도 부르지 않아야 한다. */
export function assertDoesNotShowText(source: string, text: string, message?: string): void {
  const code = withoutComments(source);
  const literal = new RegExp(`(['"\`])${text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`);
  assert.ok(!literal.test(code), message ?? `소스에 '${text}' 리터럴이 있다`);
  for (const key of keysWithValue(text)) {
    assert.ok(!code.includes(`'${key}'`), message ?? `소스가 '${text}' 의 키 ${key} 를 부른다`);
  }
}
