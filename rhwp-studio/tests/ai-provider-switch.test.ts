import test from 'node:test';
import assert from 'node:assert/strict';

import { ChatController, type ChatResetReason } from '../src/ai/agent/chat-controller.ts';
import { AiContext } from '../src/ai/ai-context.ts';
import { AI_PROVIDERS, modelsOf, type AiProviderId } from '../src/ai/provider/models.ts';
import { DEFAULT_AI_SETTINGS, hasConsent, normalizeAiSettings } from '../src/ai/settings/ai-settings.ts';
import { KeyStore } from '../src/ai/settings/key-store.ts';
import type { PluginHost } from '../src/plugin/types.ts';
import type { AiProvider } from '../src/ai/provider/types.ts';

test('세 제공자 모두 모델 목록이 있고, 설정은 제공자·모델 짝을 지킨다', () => {
  assert.deepEqual([...AI_PROVIDERS], ['claude', 'openai', 'gemini']);
  for (const provider of AI_PROVIDERS) {
    const models = modelsOf(provider);
    assert.ok(models.length > 0, provider);
    assert.equal(new Set(models.map((m) => m.id)).size, models.length, `${provider}: 모델 id 중복 없음`);
    assert.equal(normalizeAiSettings({ provider, model: models.at(-1)!.id }).model, models.at(-1)!.id);
    assert.equal(normalizeAiSettings({ provider, model: 'claude-opus-5-5' }).model,
      provider === 'claude' ? 'claude-opus-5-5' : models[0].id, '다른 제공자의 모델 id 는 그 제공자의 기본 모델로 물러난다');
  }
  assert.ok(modelsOf('openai').every((m) => !m.serverFallback) && modelsOf('gemini').every((m) => !m.serverFallback),
    '서버 측 대체는 Claude 전용이다');
});

test('전송 동의는 제공자별 — 제공자를 바꾸면 다시 묻는다', () => {
  const consent = { at: '2026-10-26T00:00:00.000Z', provider: 'claude' as AiProviderId };
  assert.equal(hasConsent({ ...DEFAULT_AI_SETTINGS, consent }), true);
  assert.equal(hasConsent({ ...DEFAULT_AI_SETTINGS, provider: 'openai', model: 'gpt-6-astra', consent }), false);
  assert.deepEqual(normalizeAiSettings({ consent: { ...consent, provider: 'gemini' } }).consent, { ...consent, provider: 'gemini' });
});

test('질문마다 고른 제공자의 adapter 를 받고, 제공자가 바뀌면 이력을 버리고 새 대화를 시작한다', async () => {
  const keys = new KeyStore(null);
  await keys.setKey('claude', 'sk-ant-api03-test-switch-0123456789', { remember: false });
  await keys.setKey('openai', 'sk-proj-test-switch-0123456789', { remember: false });
  const ctx = new AiContext({ ...DEFAULT_AI_SETTINGS }, keys);
  const host = { onDocumentSwap() {} } as unknown as PluginHost;
  const usage = { inputTokens: 1, outputTokens: 1, cacheReadTokens: 0, cacheWriteTokens: 0 };
  const asked: Array<{ provider: AiProviderId; key: string }> = [];
  const factory = async (apiKey: string, provider: AiProviderId): Promise<AiProvider> => {
    asked.push({ provider, key: apiKey });
    return { async streamTurn() { return { stop: 'end', text: '답', toolCalls: [], usage, raw: [{ type: 'text', text: '답' }] }; } };
  };
  const controller = new ChatController(ctx, host, factory);
  const resets: ChatResetReason[] = [];
  controller.onReset((reason) => resets.push(reason));

  await controller.ask('첫 질문');
  assert.equal(controller.session.messages.length, 2);
  ctx.updateSettings({ effort: 'high' });
  assert.deepEqual(resets, [], '제공자가 그대로면 대화를 유지한다');

  ctx.updateSettings({ provider: 'openai', model: 'gpt-6-astra' });
  assert.deepEqual(resets, ['provider']);
  assert.equal(controller.session.messages.length, 0, 'Claude 원형 이력을 OpenAI 로 보내지 않는다');

  await controller.ask('두 번째 질문');
  assert.deepEqual(asked, [
    { provider: 'claude', key: 'sk-ant-api03-test-switch-0123456789' },
    { provider: 'openai', key: 'sk-proj-test-switch-0123456789' },
  ], '키도 제공자별로 꺼낸다');
  assert.equal(controller.session.messages.length, 2);
});
