import test from 'node:test';
import assert from 'node:assert/strict';

import {
  AI_SETTINGS_STORAGE_KEY,
  DEFAULT_AI_SETTINGS,
  hasConsent,
  loadAiSettings,
  normalizeAiSettings,
  saveAiSettings,
} from '../src/ai/settings/ai-settings.ts';

function memoryStorage(initial?: string) {
  const values = new Map<string, string>();
  if (initial !== undefined) values.set(AI_SETTINGS_STORAGE_KEY, initial);
  return {
    values,
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => { values.set(key, value); },
  };
}

test('기본값은 Claude Opus 5.5, 응답 깊이 보통, 키 기억 안 함, 동의 없음', () => {
  assert.deepEqual(loadAiSettings(memoryStorage()), DEFAULT_AI_SETTINGS);
  assert.equal(DEFAULT_AI_SETTINGS.model, 'claude-opus-5-5');
  assert.equal(DEFAULT_AI_SETTINGS.effort, 'medium');
  assert.equal(DEFAULT_AI_SETTINGS.rememberKey, false);
  assert.equal(DEFAULT_AI_SETTINGS.consent, null);
});

test('모르는 제공자·모델·깊이·깨진 동의는 기본값으로 물러난다', () => {
  assert.deepEqual(normalizeAiSettings({
    provider: 'nope', model: 'gpt-x', effort: 'max', rememberKey: 'yes',
    consent: { at: 'not-a-date', provider: 'claude' },
  }), DEFAULT_AI_SETTINGS);
  assert.equal(normalizeAiSettings({ model: 'claude-opus-5-5-20260401' }).model, 'claude-opus-5-5',
    '목록에 없는 날짜 접미사 id 는 받지 않는다');
});

test('깨진 JSON 과 접근이 막힌 저장소에서도 기본값으로 동작한다', () => {
  assert.deepEqual(loadAiSettings(memoryStorage('{oops')), DEFAULT_AI_SETTINGS);
  const throwing = {
    getItem: () => { throw new Error('denied'); },
    setItem: () => { throw new Error('denied'); },
  };
  assert.deepEqual(loadAiSettings(throwing), DEFAULT_AI_SETTINGS);
  assert.equal(saveAiSettings(DEFAULT_AI_SETTINGS, throwing), false);
  assert.equal(saveAiSettings(DEFAULT_AI_SETTINGS, null), false);
});

test('저장 → 읽기 왕복과 동의 판정', () => {
  const storage = memoryStorage();
  const settings = {
    ...DEFAULT_AI_SETTINGS,
    model: 'claude-haiku-5-5',
    effort: 'high' as const,
    rememberKey: true,
    consent: { at: '2026-10-23T00:00:00.000Z', provider: 'claude' as const },
  };
  assert.equal(saveAiSettings(settings, storage), true);
  const loaded = loadAiSettings(storage);
  assert.deepEqual(loaded, settings);
  assert.equal(hasConsent(loaded), true);
  assert.equal(hasConsent(DEFAULT_AI_SETTINGS), false);
  assert.ok(!storage.values.get(AI_SETTINGS_STORAGE_KEY)!.includes('sk-'), '설정에는 키가 없다');
});
