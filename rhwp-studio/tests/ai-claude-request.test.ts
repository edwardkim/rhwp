import test from 'node:test';
import assert from 'node:assert/strict';

import {
  CLAUDE_FALLBACK_BETA,
  CLAUDE_MAX_TOKENS,
  ClaudeProvider,
  buildClaudeRequest,
  echoableContent,
  toTurnResult,
} from '../src/ai/provider/claude.ts';
import { AI_SYSTEM_PROMPT } from '../src/ai/agent/system-prompt.ts';
import { READ_TOOL_DEFS } from '../src/ai/tools/read-tools.ts';
import { AiProviderError, type AiMessage, type AiTurnRequest } from '../src/ai/provider/types.ts';

const assistantRaw = [
  { type: 'thinking', thinking: '', signature: 'sig-1' },
  { type: 'tool_use', id: 'tu_1', name: 'get_outline', input: {} },
];

const history: AiMessage[] = [
  { role: 'user', kind: 'text', text: '개요 알려줘' },
  { role: 'assistant', kind: 'raw', raw: assistantRaw },
  { role: 'user', kind: 'tool_results', results: [
    { id: 'tu_1', content: '<document_content id="x">\n{}\n</document_content id="x">', isError: false },
    { id: 'tu_2', content: 'Unknown tool', isError: true },
  ] },
  { role: 'assistant', kind: 'text', text: '(The user stopped this response.)' },
];

const request = (model = 'claude-opus-5-5'): AiTurnRequest => ({
  model, effort: 'medium', system: AI_SYSTEM_PROMPT, tools: READ_TOOL_DEFS, messages: history,
});

test('요청 본문: 모델·출력 상한·응답 깊이·캐시·시스템·도구·서버 대체', () => {
  const body = buildClaudeRequest(request()) as Record<string, any>;
  assert.equal(body.model, 'claude-opus-5-5');
  assert.equal(body.max_tokens, CLAUDE_MAX_TOKENS);
  assert.deepEqual(body.output_config, { effort: 'medium' });
  assert.deepEqual(body.cache_control, { type: 'ephemeral' });
  assert.deepEqual(body.system, [{ type: 'text', text: AI_SYSTEM_PROMPT }]);
  assert.equal(body.fallbacks, 'default');
  assert.deepEqual(body.betas, [CLAUDE_FALLBACK_BETA]);
  for (const absent of ['thinking', 'tool_choice', 'temperature', 'top_p', 'top_k']) {
    assert.ok(!(absent in body), `${absent} 를 보내지 않는다`);
  }
});

test('도구는 순서대로 strict 로 보내고, strict 가 받지 않는 제약은 설명으로 옮긴다', () => {
  const tools = (buildClaudeRequest(request()) as Record<string, any>).tools;
  assert.deepEqual(tools.map((t: { name: string }) => t.name), READ_TOOL_DEFS.map((t) => t.name));
  assert.ok(tools.every((t: { strict: boolean; eager_input_streaming?: boolean }) =>
    t.strict === true && t.eager_input_streaming === undefined));
  const page = tools.find((t: { name: string }) => t.name === 'read_page_text').input_schema.properties.page;
  assert.equal(page.minimum, undefined);
  assert.match(page.description, /minimum: 1/);
  assert.equal(READ_TOOL_DEFS.find((t) => t.name === 'read_page_text')!.inputSchema.properties
    && (READ_TOOL_DEFS.find((t) => t.name === 'read_page_text')!.inputSchema as any).properties.page.minimum, 1,
  '원래 정의는 바꾸지 않는다');
});

test('Haiku 5.5 에는 서버 대체를 보내지 않는다(받지 않는 모델)', () => {
  const body = buildClaudeRequest(request('claude-haiku-5-5')) as Record<string, any>;
  assert.ok(!('fallbacks' in body) && !('betas' in body));
  const sonnet = buildClaudeRequest(request('claude-sonnet-5-5')) as Record<string, any>;
  assert.equal(sonnet.fallbacks, 'default');
});

test('이력 변환: assistant 원형은 그대로, 도구 결과는 한 메시지, 오류만 is_error', () => {
  const messages = (buildClaudeRequest(request()) as Record<string, any>).messages;
  assert.deepEqual(messages[0], { role: 'user', content: [{ type: 'text', text: '개요 알려줘' }] });
  assert.equal(messages[1].content, assistantRaw, 'thinking 블록을 포함한 원형을 같은 객체로 되돌린다');
  assert.deepEqual(messages[2].content, [
    { type: 'tool_result', tool_use_id: 'tu_1', content: '<document_content id="x">\n{}\n</document_content id="x">' },
    { type: 'tool_result', tool_use_id: 'tu_2', content: 'Unknown tool', is_error: true },
  ]);
  assert.deepEqual(messages[3], { role: 'assistant', content: [{ type: 'text', text: '(The user stopped this response.)' }] });
});

test('같은 이력이면 같은 바이트 — 캐시 접두가 흔들리지 않는다', () => {
  assert.equal(JSON.stringify(buildClaudeRequest(request())), JSON.stringify(buildClaudeRequest(request())));
  const longer: AiTurnRequest = {
    ...request(),
    messages: [...history, { role: 'user', kind: 'text', text: '다음 질문' }],
  };
  const a = JSON.stringify((buildClaudeRequest(request()) as any).messages);
  const b = JSON.stringify((buildClaudeRequest(longer) as any).messages);
  assert.ok(b.startsWith(a.slice(0, -1)), '다음 요청은 앞 요청의 메시지를 접두로 가진다');
});

test('응답 중간 서버 대체: 경계 앞의 thinking·tool_use 는 빼고 텍스트와 경계 뒤는 남긴다', () => {
  const content = [
    { type: 'thinking', thinking: '', signature: 'a' },
    { type: 'text', text: '부분 ' },
    { type: 'tool_use', id: 'x', name: 'get_outline', input: {} },
    { type: 'fallback', model: 'claude-opus-4-8' },
    { type: 'thinking', thinking: '', signature: 'b' },
    { type: 'text', text: '이어서' },
  ];
  assert.deepEqual(echoableContent(content).map((b) => b.type), ['text', 'fallback', 'thinking', 'text']);
  const plain = [{ type: 'thinking', thinking: '', signature: 'a' }, { type: 'text', text: 't' }];
  assert.deepEqual(echoableContent(plain), plain, '대체가 없으면 그대로');
});

test('완성 응답 → 중립 결과: 멈춘 이유·도구 호출·텍스트·사용량', () => {
  const result = toTurnResult({
    stop_reason: 'tool_use',
    content: [
      { type: 'thinking', thinking: '', signature: 's' },
      { type: 'text', text: '찾아볼게요. ' },
      { type: 'tool_use', id: 'tu_9', name: 'search_text', input: { query: '예산' } },
    ],
    usage: { input_tokens: 100, output_tokens: 20, cache_read_input_tokens: 80, cache_creation_input_tokens: null },
  });
  assert.equal(result.stop, 'tool_use');
  assert.equal(result.text, '찾아볼게요. ');
  assert.deepEqual(result.toolCalls, [{ id: 'tu_9', name: 'search_text', input: { query: '예산' } }]);
  assert.deepEqual(result.usage, { inputTokens: 100, outputTokens: 20, cacheReadTokens: 80, cacheWriteTokens: 0 });
  for (const [reason, stop] of [['end_turn', 'end'], ['max_tokens', 'max_tokens'], ['refusal', 'refusal'],
    ['pause_turn', 'pause'], ['model_context_window_exceeded', 'max_tokens'], ['weird', 'other']]) {
    assert.equal(toTurnResult({ stop_reason: reason, content: [] }).stop, stop, reason);
  }
});

/** SDK 스트림을 흉내 낸다 — on('text'|'streamEvent') 와 finalMessage() 만. */
function fakeClient(script: { events?: Array<[string, unknown]>; final?: unknown; fail?: unknown }) {
  const sent: unknown[] = [];
  return {
    sent,
    client: {
      beta: { messages: { stream(body: unknown, options: { signal: AbortSignal }) {
        sent.push({ body, signal: options.signal });
        const listeners: Record<string, Array<(arg: unknown) => void>> = {};
        return {
          on(name: string, fn: (arg: unknown) => void) { (listeners[name] ??= []).push(fn); return this; },
          async finalMessage() {
            for (const [name, arg] of script.events ?? []) for (const fn of listeners[name] ?? []) fn(arg);
            if (script.fail) throw script.fail;
            return script.final;
          },
        };
      } } },
    } as never,
  };
}

test('스트림: 글자 조각과 도구 시작을 흘리고 완성 응답을 돌려준다', async () => {
  const { client, sent } = fakeClient({
    events: [
      ['text', '안녕'],
      ['streamEvent', { type: 'content_block_start', index: 1, content_block: { type: 'tool_use', id: 'tu_1', name: 'get_outline', input: {} } }],
    ],
    final: { stop_reason: 'end_turn', content: [{ type: 'text', text: '안녕' }], usage: { input_tokens: 1, output_tokens: 1 } },
  });
  const events: unknown[] = [];
  const result = await new ClaudeProvider('k', client).streamTurn(request(), new AbortController().signal, (e) => events.push(e));
  assert.deepEqual(events, [{ type: 'text-delta', text: '안녕' }, { type: 'tool-start', id: 'tu_1', name: 'get_outline' }]);
  assert.equal(result.stop, 'end');
  assert.equal((sent[0] as { body: { model: string } }).body.model, 'claude-opus-5-5');
});

test('스트림 실패는 갈래가 붙은 AiProviderError 로, 중단은 aborted 로 던진다', async () => {
  const { APIError } = await import('@anthropic-ai/sdk');
  const failing = fakeClient({ fail: APIError.generate(401, { type: 'error' }, 'bad', new Headers()) });
  await assert.rejects(
    new ClaudeProvider('k', failing.client).streamTurn(request(), new AbortController().signal, () => {}),
    (error) => error instanceof AiProviderError && error.reason === 'auth',
  );
  const abort = new AbortController();
  abort.abort();
  const aborted = fakeClient({ fail: new Error('socket closed') });
  await assert.rejects(
    new ClaudeProvider('k', aborted.client).streamTurn(request(), abort.signal, () => {}),
    (error) => error instanceof AiProviderError && error.reason === 'aborted',
  );
});
