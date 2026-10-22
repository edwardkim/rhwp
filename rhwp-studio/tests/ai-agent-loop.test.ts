import test from 'node:test';
import assert from 'node:assert/strict';

import { INTERRUPTED_MARKERS, runAgent, type AgentLoopOptions } from '../src/ai/agent/agent-loop.ts';
import { ChatSession } from '../src/ai/agent/chat-session.ts';
import {
  AiProviderError,
  type AiMessage,
  type AiProvider,
  type AiToolCall,
  type AiTurnRequest,
  type AiTurnResult,
} from '../src/ai/provider/types.ts';

const usage = { inputTokens: 10, outputTokens: 5, cacheReadTokens: 2, cacheWriteTokens: 1 };

function textTurn(text: string, stop: AiTurnResult['stop'] = 'end'): AiTurnResult {
  return { stop, text, toolCalls: [], usage, raw: [{ type: 'thinking', thinking: '', signature: 's' }, { type: 'text', text }] };
}

function toolTurn(calls: AiToolCall[], stop: AiTurnResult['stop'] = 'tool_use'): AiTurnResult {
  return {
    stop, text: '', toolCalls: calls, usage,
    raw: calls.map((c) => ({ type: 'tool_use', id: c.id, name: c.name, input: c.input })),
  };
}

type Step = AiTurnResult | AiProviderError | ((signal: AbortSignal) => Promise<AiTurnResult>);

function scripted(steps: Step[]) {
  const requests: AiTurnRequest[] = [];
  const snapshots: string[] = [];
  const provider: AiProvider = {
    async streamTurn(request, signal, onEvent) {
      requests.push(request);
      snapshots.push(JSON.stringify(request.messages));
      const step = steps.shift();
      if (!step) throw new Error('script exhausted');
      if (step instanceof AiProviderError) throw step;
      if (typeof step === 'function') return step(signal);
      if (step.text) onEvent({ type: 'text-delta', text: step.text });
      return step;
    },
  };
  return { provider, requests, snapshots };
}

function options(provider: AiProvider, session: ChatSession, extra: Partial<AgentLoopOptions> = {}) {
  const executed: Array<{ call: AiToolCall; boundaryId: string }> = [];
  let boundary = 0;
  const opts: AgentLoopOptions = {
    provider,
    session,
    tools: [{ name: 'get_outline', description: 'd', inputSchema: { type: 'object' } }],
    system: 'SYSTEM',
    model: 'claude-opus-5-5',
    effort: 'medium',
    executeTool: (call, boundaryId) => {
      executed.push({ call, boundaryId });
      return { id: call.id, content: `result:${call.name}`, isError: call.name === 'broken' };
    },
    createBoundaryId: () => `b${++boundary}`,
    ...extra,
  };
  return { opts, executed };
}

function assertAlternates(messages: readonly AiMessage[]): void {
  messages.forEach((m, i) => assert.equal(m.role, i % 2 === 0 ? 'user' : 'assistant', `교대 ${i}`));
}

test('도구 호출 → 모든 결과를 한 user 메시지로 돌려주고 답을 마친다', async () => {
  const { provider, requests } = scripted([
    toolTurn([{ id: 't1', name: 'get_outline', input: {} }, { id: 't2', name: 'broken', input: {} }]),
    textTurn('답'),
  ]);
  const session = new ChatSession();
  const { opts, executed } = options(provider, session);
  const deltas: string[] = [];
  const result = await runAgent(opts, '개요?', new AbortController().signal, { onText: (d) => deltas.push(d) });

  assert.equal(result.status, 'done');
  assert.equal(result.text, '답');
  assert.equal(result.toolRounds, 1);
  assert.deepEqual(deltas, ['답']);
  assert.deepEqual(result.usage, { inputTokens: 20, outputTokens: 10, cacheReadTokens: 4, cacheWriteTokens: 2 });
  assert.equal(executed.length, 2);
  assert.ok(executed.every((e) => e.boundaryId === 'b1'), '한 질문 안에서는 같은 경계 id');

  const kinds = session.messages.map((m) => `${m.role}:${m.kind}`);
  assert.deepEqual(kinds, ['user:text', 'assistant:raw', 'user:tool_results', 'assistant:raw']);
  const results = session.messages[2] as Extract<AiMessage, { kind: 'tool_results' }>;
  assert.deepEqual(results.results.map((r) => [r.id, r.isError]), [['t1', false], ['t2', true]]);
  assert.equal(requests[1].system, 'SYSTEM');
  assert.equal(requests[1].model, 'claude-opus-5-5');
});

test('이력은 덧붙이기만 한다 — 지난 요청의 메시지는 바뀌지 않고 얼어 있다', async () => {
  const { provider, requests, snapshots } = scripted([
    toolTurn([{ id: 't1', name: 'get_outline', input: {} }]),
    textTurn('첫 답'),
    textTurn('둘째 답'),
  ]);
  const session = new ChatSession();
  const { opts } = options(provider, session);
  await runAgent(opts, '하나', new AbortController().signal);
  await runAgent(opts, '둘', new AbortController().signal);

  // 각 요청의 메시지는 바로 앞 요청의 메시지를 접두로 가진다(같은 바이트).
  for (let i = 1; i < snapshots.length; i += 1) {
    const prev = JSON.parse(snapshots[i - 1]) as unknown[];
    const cur = JSON.parse(snapshots[i]) as unknown[];
    assert.deepEqual(cur.slice(0, prev.length), prev, `요청 ${i} 는 요청 ${i - 1} 을 접두로 가진다`);
  }
  // 넘긴 배열 자체도 뒤에서 바뀌지 않는다.
  assert.equal(JSON.stringify(requests[0].messages), snapshots[0]);
  assert.ok(Object.isFrozen(session.messages[1]));
  assert.throws(() => { (session.messages[0] as { text: string }).text = 'x'; });
  assertAlternates(session.messages);
});

test('도구 라운드 상한을 넘기면 멈추고 이력은 유효하게 끝난다', async () => {
  const call = (n: number) => toolTurn([{ id: `t${n}`, name: 'get_outline', input: {} }]);
  const { provider } = scripted([call(1), call(2), call(3)]);
  const session = new ChatSession();
  const { opts, executed } = options(provider, session, { maxToolRounds: 2 });
  const result = await runAgent(opts, '?', new AbortController().signal);
  assert.equal(result.status, 'round_limit');
  assert.equal(executed.length, 2);
  const last = session.messages.at(-1)!;
  assert.deepEqual(last, { role: 'assistant', kind: 'text', text: INTERRUPTED_MARKERS.round_limit });
  assertAlternates(session.messages);
});

test('max_tokens 로 잘린 도구 호출은 실행하지 않고, 결과 없는 tool_use 를 남기지 않는다', async () => {
  const { provider } = scripted([toolTurn([{ id: 't1', name: 'get_outline', input: {} }], 'max_tokens')]);
  const session = new ChatSession();
  const { opts, executed } = options(provider, session);
  const result = await runAgent(opts, '?', new AbortController().signal);
  assert.equal(result.status, 'max_tokens');
  assert.equal(executed.length, 0);
  assert.ok(!JSON.stringify(session.messages).includes('tool_use'));
  assert.equal((session.messages.at(-1) as { text: string }).text, INTERRUPTED_MARKERS.max_tokens);
});

test('도구 없이 max_tokens 로 잘린 답은 원형 그대로 남긴다', async () => {
  const { provider } = scripted([textTurn('긴 답…', 'max_tokens')]);
  const session = new ChatSession();
  const result = await runAgent(options(provider, session).opts, '?', new AbortController().signal);
  assert.equal(result.status, 'max_tokens');
  assert.equal(session.messages.at(-1)?.kind, 'raw');
});

test('거절은 부분 응답을 넣지 않고 표시로 끝낸다', async () => {
  const { provider } = scripted([textTurn('부분', 'refusal')]);
  const session = new ChatSession();
  const result = await runAgent(options(provider, session).opts, '?', new AbortController().signal);
  assert.equal(result.status, 'refusal');
  assert.ok(!JSON.stringify(session.messages).includes('부분'));
  assert.equal((session.messages.at(-1) as { text: string }).text, INTERRUPTED_MARKERS.refusal);
});

test('중단하면 표시로 끝내고, 이어서 다음 질문을 할 수 있다', async () => {
  const abort = new AbortController();
  const { provider } = scripted([
    (signal) => new Promise((_resolve, reject) => {
      signal.addEventListener('abort', () => reject(new AiProviderError({ reason: 'aborted' })));
      queueMicrotask(() => abort.abort());
    }),
    textTurn('다음 답'),
  ]);
  const session = new ChatSession();
  const { opts } = options(provider, session);
  const stopped = await runAgent(opts, '첫 질문', abort.signal);
  assert.equal(stopped.status, 'aborted');
  assert.equal((session.messages.at(-1) as { text: string }).text, INTERRUPTED_MARKERS.aborted);

  const next = await runAgent(opts, '둘째 질문', new AbortController().signal);
  assert.equal(next.status, 'done');
  assertAlternates(session.messages);
});

test('도구 실행 중 중단돼도 모든 tool_use 에 결과를 채운 뒤 끝낸다', async () => {
  const abort = new AbortController();
  const { provider } = scripted([
    toolTurn([{ id: 't1', name: 'get_outline', input: {} }, { id: 't2', name: 'get_outline', input: {} }]),
  ]);
  const session = new ChatSession();
  const { opts } = options(provider, session, {
    executeTool: (call) => { abort.abort(); return { id: call.id, content: 'r', isError: false }; },
  });
  const result = await runAgent(opts, '?', abort.signal);
  assert.equal(result.status, 'aborted');
  const results = session.messages[2] as Extract<AiMessage, { kind: 'tool_results' }>;
  assert.deepEqual(results.results.map((r) => r.id), ['t1', 't2']);
  assertAlternates(session.messages);
});

test('제공자 오류는 갈래를 그대로 돌려주고 표시로 끝낸다', async () => {
  const { provider } = scripted([new AiProviderError({ reason: 'rate', status: 429, retryAfterSeconds: 3 })]);
  const session = new ChatSession();
  const result = await runAgent(options(provider, session).opts, '?', new AbortController().signal);
  assert.equal(result.status, 'error');
  assert.equal(result.error?.reason, 'rate');
  assert.equal(result.error?.retryAfterSeconds, 3);
  assert.equal((session.messages.at(-1) as { text: string }).text, INTERRUPTED_MARKERS.error);
});

test('질문마다 새 경계 id 를 쓴다', async () => {
  const { provider } = scripted([
    toolTurn([{ id: 't1', name: 'get_outline', input: {} }]), textTurn('a'),
    toolTurn([{ id: 't2', name: 'get_outline', input: {} }]), textTurn('b'),
  ]);
  const session = new ChatSession();
  const { opts, executed } = options(provider, session);
  await runAgent(opts, '1', new AbortController().signal);
  await runAgent(opts, '2', new AbortController().signal);
  assert.deepEqual(executed.map((e) => e.boundaryId), ['b1', 'b2']);
});

test('세션은 user/assistant 교대가 어긋난 덧붙이기를 거절한다', () => {
  const session = new ChatSession();
  assert.throws(() => session.append({ role: 'assistant', kind: 'text', text: 'x' }), /교대/);
  session.append({ role: 'user', kind: 'text', text: 'q' });
  assert.throws(() => session.append({ role: 'user', kind: 'text', text: 'q2' }), /교대/);
});
