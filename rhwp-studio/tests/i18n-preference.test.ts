import test from 'node:test';
import assert from 'node:assert/strict';

import { getLocale, setPreferredLocale } from '../src/i18n/index.ts';

// 언어 변경의 적용 시점 계약 (이슈 #5852 검토 ①·③): **다음 실행부터 적용**.
//
// 즉시 전환은 이미 열린 대화상자, 모듈 상수에 담긴 문구, 동적 메뉴·상태 표시줄까지 현재 상태로
// 다시 만들어야 안전하다. 그 전까지 언어 선택은 선택을 저장만 하고 지금 화면과 로케일은 건드리지 않는다.
// 새로고침으로 적용하는 경로를 연다면 미저장 문서 보호 흐름을 거쳐야 한다 — 이 함수는 그 일을 하지 않는다.

test('언어를 골라도 현재 로케일은 그대로다 — 다음 실행부터 적용', () => {
  const before = getLocale();
  const other = before === 'ko' ? 'en' : 'ko';
  const result = setPreferredLocale(other);
  assert.equal(getLocale(), before);
  assert.deepEqual(result, { locale: other, appliesOnNextLaunch: true });
});

test('지금과 같은 언어를 고르면 다시 시작할 필요가 없다', () => {
  assert.deepEqual(setPreferredLocale(getLocale()), { locale: getLocale(), appliesOnNextLaunch: false });
});

test('언어를 골라도 화면(DOM)을 다시 쓰지 않는다 — document 가 없는 환경에서도 예외가 없다', () => {
  assert.equal(typeof (globalThis as { document?: unknown }).document, 'undefined');
  assert.doesNotThrow(() => setPreferredLocale('en'));
});
