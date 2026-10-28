import test from 'node:test';
import assert from 'node:assert/strict';

import { ChatController } from '../src/ai/agent/chat-controller.ts';
import { AiContext } from '../src/ai/ai-context.ts';
import { DEFAULT_AI_SETTINGS } from '../src/ai/settings/ai-settings.ts';
import { KeyStore } from '../src/ai/settings/key-store.ts';
import { StudioCommandError, type CommandErrorV1 } from '../src/core/command-error.ts';
import type { PluginHost } from '../src/plugin/types.ts';
import type { AiProvider, AiTurnResult } from '../src/ai/provider/types.ts';

test('AI 쓰기는 명령 복구 보고를 기다리고 원래 실패 사유를 보존한 뒤 다음 변경을 적용한다', async () => {
  const keys = new KeyStore(null);
  await keys.setKey('claude', 'sk-ant-api03-test-recovery-0123456789', { remember: false });
  const ctx = new AiContext({ ...DEFAULT_AI_SETTINGS, autoApply: true }, keys);
  let finishRecovery!: (value: CommandErrorV1) => void;
  const report = new Promise<CommandErrorV1>((resolve) => { finishRecovery = resolve; });
  let enteredFailure!: () => void;
  const failed = new Promise<void>((resolve) => { enteredFailure = resolve; });
  let recovering = true;
  let transactions = 0;
  let text = '원문';
  const document = {
    searchAllText: () => JSON.stringify([{ sec: 0, para: 0, charOffset: 0, length: 2 }]),
    getParagraphLength: () => text.length,
    getTextRange: (_s: number, _p: number, start: number, end: number) => text.slice(start, end),
    replaceText: (_s: number, _p: number, _at: number, _count: number, value: string) => {
      text = value;
      return '{"ok":true}';
    },
  };
  const host = {
    onDocumentSwap() {},
    read(fn: (doc: typeof document) => unknown) { return fn(document); },
    transaction(_label: string, fn: (tx: { doc(): typeof document }) => unknown) {
      transactions += 1;
      if (transactions === 1) {
        enteredFailure();
        throw new StudioCommandError(new Error('지원하지 않는 첫 변경'), report);
      }
      assert.equal(recovering, false, '복구 중에는 다음 편집 명령을 호출하지 않는다');
      return fn({ doc: () => document });
    },
  } as unknown as PluginHost;
  const usage = { inputTokens: 1, outputTokens: 1, cacheReadTokens: 0, cacheWriteTokens: 0 };
  const turns: AiTurnResult[] = [{
    stop: 'tool_use', text: '', usage, raw: [],
    toolCalls: [
      { id: 'first', name: 'replace_text', input: { find: '원문', replace: '첫 변경' } },
      { id: 'next', name: 'replace_text', input: { find: '원문', replace: '다음 변경' } },
    ],
  }, { stop: 'end', text: '완료', toolCalls: [], usage, raw: [] }];
  const provider: AiProvider = { async streamTurn() { return turns.shift()!; } };
  const controller = new ChatController(ctx, host, async () => provider);
  const settled: Array<{ outcome: string; message?: string }> = [];
  controller.setPresenter({
    present: () => ({ decision: Promise.resolve(true), settle: (outcome, message) => { settled.push({ outcome, message }); } }),
  });
  const answer = controller.ask('두 변경을 순서대로 적용');
  await failed;
  await new Promise((resolve) => setImmediate(resolve));
  // 구현이 보고를 기다리지 않으면 이 시점에 두 번째 트랜잭션까지 실행된다.
  const callsBeforeRecovery = transactions;
  recovering = false;
  finishRecovery({
    schemaVersion: 1, capability: 'command-error-v1', eventId: 'test:1', documentSessionId: 'test',
    operation: 'plugin:ai:first', code: 'COMMAND_ROLLED_BACK', recovery: 'succeeded',
    checks: { content: true, editing: true, history: true, dirty: true, ready: true },
  });
  await answer;
  assert.equal(callsBeforeRecovery, 1);
  assert.equal(text, '다음 변경');
  assert.deepEqual(settled.map((s) => s.outcome), ['failed', 'applied']);
  assert.match(settled[0].message!, /지원하지 않는 첫 변경/);
  const results = controller.session.messages.find((message) => message.kind === 'tool_results');
  assert.ok(results && results.kind === 'tool_results');
  assert.deepEqual(results.results.map((r) => r.isError), [true, false]);
  assert.match(results.results[0].content, /지원하지 않는 첫 변경/);
});
