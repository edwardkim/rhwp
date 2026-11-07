// 관계형: 사용자가 지정한 문서와 실제 공급 프로그램만 글꼴 대체를 소유한다.
// 주석 갱신: 2026-11-07 20:35 KST. 관계형이며 설치된 사설 글꼴에 의존하지 않는다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { documentFontPreferences, applyDocumentFontPreferences } from '../src/core/document-font-preferences.ts';

test('글꼴 설정은 선택한 원본에만 저장되고 다른 문서로 전파되지 않는다', () => {
  const keys = ['localStorage', 'location'] as const;
  const previous = new Map(keys.map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  const values = new Map<string, string>();
  const digest = 'blake3:' + 'a'.repeat(64);
  const location = { search: '?' + new URLSearchParams({ 'font-environment': JSON.stringify({ documentDigest: digest, substitutions: { '나눔고딕': 'Haansoft Batang' } }) }) };
  Object.defineProperty(globalThis, 'location', { configurable: true, value: location });
  Object.defineProperty(globalThis, 'localStorage', { configurable: true, value: {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
  } });
  try {
    assert.deepEqual(documentFontPreferences('blake3:' + 'b'.repeat(64)), {});
    assert.equal(values.size, 0);
    assert.deepEqual(documentFontPreferences(digest), { '나눔고딕': 'Haansoft Batang' });
    location.search = '';
    assert.deepEqual(documentFontPreferences(digest), { '나눔고딕': 'Haansoft Batang' });
  } finally {
    for (const key of keys) {
      const descriptor = previous.get(key);
      if (descriptor) Object.defineProperty(globalThis, key, descriptor); else Reflect.deleteProperty(globalThis, key);
    }
  }
});

test('사용자 지정도 실제 프로그램 없이 성공 처리하지 않고 다른 원본 선택을 유지한다', () => {
  const corn = { query: '옥수수', family: '옥수수', programId: 'original-corn', trueType: false, openTypeCff: true };
  const automatic = JSON.stringify({ id: 'automatic', substitutions: { '옥수수': '옥수수' }, selectedPrograms: { '옥수수': corn }, preserveSourceMetrics: true });
  const preferences = { '나눔고딕': 'Haansoft Batang' };
  assert.equal(applyDocumentFontPreferences(automatic, preferences, [], 'user'), automatic);
  const target = { query: 'Haansoft Batang', family: 'Haansoft Batang', programId: 'loaded-batang', trueType: true, openTypeCff: false };
  const result = JSON.parse(applyDocumentFontPreferences(automatic, preferences, [target], 'user'));
  assert.equal(result.substitutions['나눔고딕'], 'Haansoft Batang');
  assert.deepEqual(result.selectedPrograms['옥수수'], corn);
  assert.deepEqual(result.selectedPrograms['나눔고딕'], target);
  assert.equal(result.preserveSourceMetrics, true, '자동 공급 환경의 다른 원본 메트릭은 유지한다');
  assert.deepEqual(result.replaceSourceMetricsFor, ['나눔고딕'], '사용자가 선택한 원본만 자연 프로그램 전진을 사용한다');
  const legacy = JSON.parse(applyDocumentFontPreferences(automatic, preferences, [target], 'user', false));
  assert.equal(legacy.preserveSourceMetrics, false);
  assert.equal('replaceSourceMetricsFor' in legacy, false, '이전 WASM에는 지원하지 않는 새 JSON 필드를 보내지 않는다');
});
