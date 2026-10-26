import test from 'node:test';
import assert from 'node:assert/strict';

import { AI_TOOL_DEFS } from '../src/ai/agent/chat-controller.ts';
import { AI_SYSTEM_PROMPT } from '../src/ai/agent/system-prompt.ts';
import {
  OPENAI_API_BASE,
  OpenAiProvider,
  buildOpenAiRequest,
  classifyOpenAiError,
  testOpenAiConnection,
  toOpenAiTurnResult,
} from '../src/ai/provider/openai.ts';
import { readSse } from '../src/ai/provider/http.ts';
import { AiProviderError, type AiMessage, type AiStreamEvent, type AiTurnRequest } from '../src/ai/provider/types.ts';

const KEY = 'sk-proj-test-0123456789abcdef';

const assistantRaw = [
  { type: 'reasoning', id: 'rs_1', summary: [], encrypted_content: 'enc-1' },
  { type: 'function_call', id: 'fc_1', call_id: 'call_1', name: 'get_outline', arguments: '{}' },
];

const history: AiMessage[] = [
  { role: 'user', kind: 'text', text: '개요 알려줘' },
  { role: 'assistant', kind: 'raw', raw: assistantRaw },
  { role: 'user', kind: 'tool_results', note: '(알림)', results: [
    { id: 'call_1', content: '<document_content id="x">\n{}\n</document_content id="x">', isError: false },
    { id: 'call_2', content: 'Unknown tool', isError: true },
  ] },
  { role: 'assistant', kind: 'text', text: '(The user stopped this response.)' },
];

const request = (): AiTurnRequest => ({
  model: 'gpt-6.1-sol', effort: 'high', system: AI_SYSTEM_PROMPT, tools: AI_TOOL_DEFS, messages: history,
});

function sseBody(events: unknown[]): ReadableStream<Uint8Array> {
  const text = events.map((event) => `event: ${(event as { type: string }).type}\ndata: ${JSON.stringify(event)}\n\n`).join('');
  const bytes = new TextEncoder().encode(text);
  // 줄 중간에서 끊긴 청크도 이어 읽는지 보려고 7바이트씩 흘린다.
  return new ReadableStream({
    start(controller) {
      for (let i = 0; i < bytes.length; i += 7) controller.enqueue(bytes.slice(i, i + 7));
      controller.close();
    },
  });
}

test('요청 본문: 무상태·응답 깊이·시스템·도구(strict 끔)·이력 되돌리기', () => {
  const body = buildOpenAiRequest(request()) as Record<string, any>;
  assert.equal(body.model, 'gpt-6.1-sol');
  assert.equal(body.instructions, AI_SYSTEM_PROMPT);
  assert.deepEqual(body.reasoning, { effort: 'high' });
  assert.equal(body.store, false, '서버에 대화를 남기지 않는다');
  assert.equal(body.stream, true);
  assert.deepEqual(body.include, ['reasoning.encrypted_content']);
  for (const absent of ['temperature', 'tool_choice', 'previous_response_id', 'max_output_tokens']) {
    assert.ok(!(absent in body), `${absent} 를 보내지 않는다`);
  }
  assert.deepEqual(body.tools.map((t: { name: string }) => t.name), AI_TOOL_DEFS.map((t) => t.name), '도구 순서 고정');
  assert.ok(body.tools.every((t: { type: string; strict: boolean }) => t.type === 'function' && t.strict === false));
  const read = body.tools.find((t: { name: string }) => t.name === 'read_page_text');
  assert.equal(read.parameters.properties.page.minimum, 1, 'schema 는 고치지 않고 보낸다');

  assert.deepEqual(body.input, [
    { role: 'user', content: [{ type: 'input_text', text: '개요 알려줘' }] },
    ...assistantRaw,
    { type: 'function_call_output', call_id: 'call_1', output: '<document_content id="x">\n{}\n</document_content id="x">' },
    { type: 'function_call_output', call_id: 'call_2', output: 'Error: Unknown tool' },
    { role: 'user', content: [{ type: 'input_text', text: '(알림)' }] },
    { role: 'assistant', content: '(The user stopped this response.)' },
  ]);
});

test('같은 이력이면 같은 바이트 — 시각·난수를 넣지 않는다', () => {
  assert.equal(JSON.stringify(buildOpenAiRequest(request())), JSON.stringify(buildOpenAiRequest(request())));
});

test('응답 → 중립 결과: 도구 호출·텍스트·사용량·끝 사유', () => {
  const output = [
    { type: 'reasoning', id: 'rs_2', summary: [], encrypted_content: 'enc-2' },
    { type: 'message', id: 'msg_1', role: 'assistant', content: [{ type: 'output_text', text: '표를 ' }, { type: 'output_text', text: '봅니다.' }] },
    { type: 'function_call', id: 'fc_2', call_id: 'call_9', name: 'read_page_text', arguments: '{"page":2}' },
  ];
  const result = toOpenAiTurnResult({
    status: 'completed', output,
    usage: { input_tokens: 1200, output_tokens: 80, input_tokens_details: { cached_tokens: 1000 } },
  });
  assert.equal(result.stop, 'tool_use');
  assert.equal(result.text, '표를 봅니다.');
  assert.deepEqual(result.toolCalls, [{ id: 'call_9', name: 'read_page_text', input: { page: 2 } }]);
  assert.deepEqual(result.raw, output, '출력 항목은 고치지 않고 되돌린다');
  assert.deepEqual(result.usage, { inputTokens: 200, outputTokens: 80, cacheReadTokens: 1000, cacheWriteTokens: 0 });

  assert.equal(toOpenAiTurnResult({ status: 'completed', output: [output[1]] }).stop, 'end');
  assert.equal(toOpenAiTurnResult({ status: 'incomplete', incomplete_details: { reason: 'max_output_tokens' }, output: [] }).stop, 'max_tokens');
  assert.equal(toOpenAiTurnResult({ status: 'incomplete', incomplete_details: { reason: 'content_filter' }, output: [] }).stop, 'refusal');
  assert.equal(toOpenAiTurnResult({
    status: 'completed',
    output: [{ type: 'message', content: [{ type: 'refusal', refusal: 'no' }] }],
  }).stop, 'refusal');
  assert.equal(toOpenAiTurnResult({
    status: 'completed', output: [{ type: 'function_call', call_id: 'c', name: 'x', arguments: '{bad' }],
  }).toolCalls[0].input, '{bad', '깨진 인자는 원문으로 넘겨 클라이언트 검증이 거절하게 한다');
});

test('오류 본문과 상태 코드를 화면 갈래로 옮긴다', () => {
  assert.deepEqual(classifyOpenAiError(401, { message: 'Incorrect API key' }), { reason: 'auth', status: 401 });
  assert.deepEqual(classifyOpenAiError(403, null), { reason: 'permission', status: 403 });
  assert.deepEqual(classifyOpenAiError(404, { code: 'model_not_found', message: 'no' }),
    { reason: 'model', status: 404, detail: 'no' });
  assert.deepEqual(classifyOpenAiError(429, { code: 'rate_limit_exceeded' }, null, 12),
    { reason: 'rate', status: 429, retryAfterSeconds: 12 });
  assert.deepEqual(classifyOpenAiError(429, { type: 'insufficient_quota', message: 'quota' }),
    { reason: 'billing', status: 429, detail: 'quota' });
  assert.equal(classifyOpenAiError(429, { code: 'project_spend_limit_exceeded' }).reason, 'billing');
  assert.equal(classifyOpenAiError(429, { code: 'credit_balance_exhausted' }).reason, 'billing');
  assert.deepEqual(classifyOpenAiError(400, { message: 'bad schema' }, 'req_1'),
    { reason: 'bad-request', status: 400, detail: 'bad schema · request-id req_1' });
  assert.deepEqual(classifyOpenAiError(503, null), { reason: 'overloaded', status: 503 });
});

test('연결 테스트는 모델 조회 한 번 — 키는 Authorization 헤더로만, URL 에 싣지 않는다', async () => {
  const calls: Array<{ url: string; init: RequestInit }> = [];
  const ok = await testOpenAiConnection(KEY, 'gpt-6-astra', undefined, async (url, init) => {
    calls.push({ url, init });
    return new Response(JSON.stringify({ id: 'gpt-6-astra', object: 'model' }), { status: 200 });
  });
  assert.deepEqual(ok, { ok: true, modelName: 'GPT-6 Astra' });
  assert.equal(calls[0].url, `${OPENAI_API_BASE}/models/gpt-6-astra`);
  assert.equal(calls[0].init.method, 'GET');
  assert.equal((calls[0].init.headers as Record<string, string>).authorization, `Bearer ${KEY}`);
  assert.ok(!calls[0].url.includes(KEY));

  const bad = await testOpenAiConnection(KEY, 'gpt-6-astra', undefined, async () =>
    new Response(JSON.stringify({ error: { message: 'Incorrect API key provided' } }), { status: 401 }));
  assert.deepEqual(bad, { ok: false, reason: 'auth', status: 401 });
  const offline = await testOpenAiConnection(KEY, 'gpt-6-astra', undefined, async () => { throw new TypeError('Failed to fetch'); });
  assert.deepEqual(offline, { ok: false, reason: 'network' });
});

test('스트림: 글 조각·도구 시작을 흘리고 response.completed 로 끝낸다', async () => {
  const final = {
    status: 'completed',
    output: [
      { type: 'message', id: 'msg_1', role: 'assistant', content: [{ type: 'output_text', text: '읽어 봅니다.' }] },
      { type: 'function_call', id: 'fc_1', call_id: 'call_1', name: 'get_outline', arguments: '{}' },
    ],
    usage: { input_tokens: 10, output_tokens: 5 },
  };
  const sent: Array<{ url: string; body: any; headers: Record<string, string> }> = [];
  const provider = new OpenAiProvider(KEY, async (url, init) => {
    sent.push({ url, body: JSON.parse(String(init.body)), headers: init.headers as Record<string, string> });
    return new Response(sseBody([
      { type: 'response.created', response: { status: 'in_progress' } },
      { type: 'response.output_text.delta', delta: '읽어 ' },
      { type: 'response.output_text.delta', delta: '봅니다.' },
      { type: 'response.output_item.added', item: { type: 'function_call', call_id: 'call_1', name: 'get_outline' } },
      { type: 'response.completed', response: final },
    ]), { status: 200, headers: { 'content-type': 'text/event-stream' } });
  });
  const events: AiStreamEvent[] = [];
  const result = await provider.streamTurn(request(), new AbortController().signal, (event) => events.push(event));
  assert.equal(sent[0].url, `${OPENAI_API_BASE}/responses`);
  assert.equal(sent[0].headers.authorization, `Bearer ${KEY}`);
  assert.deepEqual(events, [
    { type: 'text-delta', text: '읽어 ' },
    { type: 'text-delta', text: '봅니다.' },
    { type: 'tool-start', id: 'call_1', name: 'get_outline' },
  ]);
  assert.equal(result.stop, 'tool_use');
  assert.deepEqual(result.raw, final.output);
});

test('스트림 오류·HTTP 오류·중단은 AiProviderError 갈래로 던진다', async () => {
  const failing = new OpenAiProvider(KEY, async () => new Response(sseBody([
    { type: 'response.failed', response: { status: 'failed', error: { code: 'server_error', message: 'boom' } } },
  ]), { status: 200 }));
  await assert.rejects(failing.streamTurn(request(), new AbortController().signal, () => {}),
    (error: unknown) => error instanceof AiProviderError && error.reason === 'unknown' && error.detail === 'boom');

  const limited = new OpenAiProvider(KEY, async () => new Response(
    JSON.stringify({ error: { code: 'rate_limit_exceeded', message: 'slow' } }),
    { status: 429, headers: { 'retry-after': '3' } },
  ));
  await assert.rejects(limited.streamTurn(request(), new AbortController().signal, () => {}),
    (error: unknown) => error instanceof AiProviderError && error.reason === 'rate' && error.retryAfterSeconds === 3);

  const abort = new AbortController();
  const hanging = new OpenAiProvider(KEY, async (_url, init) => new Promise((_resolve, reject) => {
    init.signal?.addEventListener('abort', () => reject(new DOMException('aborted', 'AbortError')));
  }));
  const pending = hanging.streamTurn(request(), abort.signal, () => {});
  abort.abort();
  await assert.rejects(pending, (error: unknown) => error instanceof AiProviderError && error.reason === 'aborted');
});

test('SSE 읽기: 여러 data 줄·주석·CRLF·끝 빈 줄 없는 마지막 이벤트', async () => {
  const text = ': keep-alive\r\nevent: a\r\ndata: 1\r\ndata: 2\r\n\r\ndata: last';
  const body = new ReadableStream<Uint8Array>({
    start(controller) { controller.enqueue(new TextEncoder().encode(text)); controller.close(); },
  });
  const seen: Array<{ event: string; data: string }> = [];
  await readSse(body, (event) => seen.push(event));
  assert.deepEqual(seen, [{ event: 'a', data: '1\n2' }, { event: 'message', data: 'last' }]);
});
