import test from 'node:test';
import assert from 'node:assert/strict';

import { LOCALE_QUERY_PARAM, resolveLocale } from '../src/i18n/resolve.ts';

test('질의 문자열이 저장값과 브라우저 설정보다 앞선다', () => {
  assert.equal(
    resolveLocale({ search: `?${LOCALE_QUERY_PARAM}=en`, stored: 'ko', navigator: 'ko-KR' }),
    'en',
  );
});

test('질의 문자열이 없으면 저장된 사용자 선택을 쓴다', () => {
  assert.equal(resolveLocale({ search: '', stored: 'en', navigator: 'ko-KR' }), 'en');
});

test('브라우저 설정은 기본으로 따르지 않는다 — 고른 적 없는데 화면이 바뀌면 안 된다', () => {
  assert.equal(resolveLocale({ search: null, stored: null, navigator: 'en-GB' }), 'ko');
});

test('preferBrowserLanguage 를 켜면 그때 브라우저 설정을 쓴다', () => {
  assert.equal(
    resolveLocale({ search: null, stored: null, navigator: 'en-GB', preferBrowserLanguage: true }),
    'en',
  );
});

test('아무 단서가 없으면 ko 로 떨어진다', () => {
  assert.equal(resolveLocale({}), 'ko');
  assert.equal(resolveLocale({ search: '', stored: null, navigator: null }), 'ko');
});

test('지원하지 않는 값은 무시하고 다음 단계로 넘어간다', () => {
  assert.equal(resolveLocale({ search: '?lang=ja', stored: 'en', navigator: 'ko' }), 'en');
  assert.equal(
    resolveLocale({ search: '?lang=ja', stored: 'zz', navigator: 'ja-JP', preferBrowserLanguage: true }),
    'ko',
  );
});

test('망가진 질의 문자열이 예외를 만들지 않는다', () => {
  assert.equal(
    resolveLocale({ search: '?%%%', stored: null, navigator: 'en', preferBrowserLanguage: true }),
    'en',
  );
});
