import test from 'node:test';
import assert from 'node:assert/strict';

import {
  DEFAULT_LOCALE,
  formatMessage,
  getEffectiveLocale,
  getLocale,
  hasTranslation,
  isSupportedLocale,
  normalizeLocale,
  registerCatalog,
  resetCatalogs,
  setLocale,
  t,
} from '../src/i18n/core.ts';

function withCatalogs(fn: () => void): void {
  resetCatalogs();
  try {
    fn();
  } finally {
    resetCatalogs();
  }
}

test('기본 로케일은 ko 이고 초기 상태에서도 그대로다', () => {
  withCatalogs(() => {
    assert.equal(DEFAULT_LOCALE, 'ko');
    assert.equal(getLocale(), 'ko');
  });
});

test('카탈로그가 비어 있어도 t 는 예외 없이 키를 돌려준다', () => {
  withCatalogs(() => {
    assert.equal(t('menu.file.open'), 'menu.file.open');
  });
});

test('en 에 번역이 없으면 ko 원문으로 물러난다 — 번역률 0% 무회귀의 근거', () => {
  withCatalogs(() => {
    registerCatalog('ko', { 'menu.file.open': '열기' });
    registerCatalog('en', {});
    setLocale('en');
    assert.equal(t('menu.file.open'), '열기');
  });
});

test('en 에 번역이 있으면 en 을 쓴다', () => {
  withCatalogs(() => {
    registerCatalog('ko', { 'menu.file.open': '열기' });
    registerCatalog('en', { 'menu.file.open': 'Open' });
    setLocale('en');
    assert.equal(t('menu.file.open'), 'Open');
  });
});

test('ko 로케일은 언제나 원문을 그대로 낸다', () => {
  withCatalogs(() => {
    registerCatalog('ko', { 'dialog.charShape.title': '글자 모양' });
    registerCatalog('en', { 'dialog.charShape.title': 'Character' });
    setLocale('ko');
    assert.equal(t('dialog.charShape.title'), '글자 모양');
  });
});

test('이름 자리표시자를 치환하고, 값이 없으면 자리표시자를 남긴다', () => {
  assert.equal(formatMessage('{n}쪽', { n: 12 }), '12쪽');
  assert.equal(formatMessage('{n} pages', { n: 12 }), '12 pages');
  assert.equal(formatMessage('{a}/{b}', { a: 1, b: 2 }), '1/2');
  assert.equal(formatMessage('{missing}', { other: 1 }), '{missing}');
  assert.equal(formatMessage('그대로'), '그대로');
});

test('폴백 경로에서도 자리표시자는 치환된다', () => {
  withCatalogs(() => {
    registerCatalog('ko', { 'msg.pages.count': '{n}쪽' });
    registerCatalog('en', {});
    setLocale('en');
    assert.equal(t('msg.pages.count', { n: 3 }), '3쪽');
  });
});

test('언어 태그를 지원 로케일로 좁힌다', () => {
  assert.equal(normalizeLocale('en-US'), 'en');
  assert.equal(normalizeLocale('ko-KR'), 'ko');
  assert.equal(normalizeLocale('EN'), 'en');
  assert.equal(normalizeLocale('ko_KR'), 'ko');
  assert.equal(normalizeLocale('ja'), null);
  assert.equal(normalizeLocale(''), null);
  assert.equal(normalizeLocale(null), null);
});

test('지원 로케일 판정', () => {
  assert.ok(isSupportedLocale('ko'));
  assert.ok(isSupportedLocale('en'));
  assert.ok(!isSupportedLocale('ja'));
  assert.ok(!isSupportedLocale(42));
});

test('hasTranslation 은 폴백을 세지 않는다', () => {
  withCatalogs(() => {
    registerCatalog('ko', { 'a.b': '가' });
    registerCatalog('en', {});
    assert.ok(hasTranslation('a.b', 'ko'));
    assert.ok(!hasTranslation('a.b', 'en'));
  });
});

test('카탈로그가 비어 있으면 실효 로케일은 ko 다 — lang 속성이 거짓말하지 않게', () => {
  withCatalogs(() => {
    registerCatalog('ko', { 'a.b': '가' });
    registerCatalog('en', {});
    setLocale('en');
    assert.equal(getLocale(), 'en');
    assert.equal(getEffectiveLocale(), 'ko');
  });
});

test('번역이 하나라도 있으면 실효 로케일은 선택한 로케일이다', () => {
  withCatalogs(() => {
    registerCatalog('ko', { 'a.b': '가' });
    registerCatalog('en', { 'a.b': 'A' });
    setLocale('en');
    assert.equal(getEffectiveLocale(), 'en');
  });
});
