import test from 'node:test';
import assert from 'node:assert/strict';

import { AI_TOOL_DEFS } from '../src/ai/agent/chat-controller.ts';
import { AI_SYSTEM_PROMPT } from '../src/ai/agent/system-prompt.ts';
import {
  GEMINI_API_BASE,
  GeminiProvider,
  buildGeminiRequest,
  classifyGeminiError,
  geminiStreamUrl,
  mergeGeminiPart,
  testGeminiConnection,
  toGeminiSchema,
  toGeminiTurnResult,
  type GeminiPart,
} from '../src/ai/provider/gemini.ts';
import { AiProviderError, type AiMessage, type AiStreamEvent, type AiTurnRequest } from '../src/ai/provider/types.ts';

const KEY = 'AIzaSy-test-0123456789abcdefghij';

/** Gemini `Schema` 가 받는 키 — 이 밖의 키가 남으면 400 이다. */
const ALLOWED = new Set([
  'type', 'description', 'nullable', 'enum', 'items', 'properties', 'required',
  'minItems', 'maxItems', 'minLength', 'maxLength', 'pattern', 'minimum', 'maximum',
  'anyOf', 'default', 'title',
]);

function assertGeminiSchema(schema: unknown, path: string): void {
  if (!schema || typeof schema !== 'object') return;
  for (const [key, value] of Object.entries(schema as Record<string, unknown>)) {
    assert.ok(ALLOWED.has(key), `${path}.${key} 는 Gemini schema 키가 아니다`);
    if (key === 'properties') {
      for (const [name, child] of Object.entries(value as Record<string, unknown>)) assertGeminiSchema(child, `${path}.${name}`);
    } else if (key === 'items') {
      assertGeminiSchema(value, `${path}[]`);
    }
  }
}

const modelRaw: GeminiPart[] = [
  { text: '개요를 봅니다.' },
  { functionCall: { name: 'get_outline', args: {} }, thoughtSignature: 'sig-A' },
  { functionCall: { id: 'own-2', name: 'read_page_text', args: { page: 1 } } },
];

const history: AiMessage[] = [
  { role: 'user', kind: 'text', text: '개요 알려줘' },
  { role: 'assistant', kind: 'raw', raw: modelRaw },
  { role: 'user', kind: 'tool_results', note: '(알림)', results: [
    { id: 'gemini-call-0', content: '{"outline":[]}', isError: false },
    { id: 'own-2', content: 'Page out of range', isError: true },
  ] },
  { role: 'assistant', kind: 'text', text: '(The user stopped this response.)' },
];

const request = (): AiTurnRequest => ({
  model: 'gemini-3.8-flash', effort: 'low', system: AI_SYSTEM_PROMPT, tools: AI_TOOL_DEFS, messages: history,
});

function sseBody(chunks: unknown[]): ReadableStream<Uint8Array> {
  const bytes = new TextEncoder().encode(chunks.map((chunk) => `data: ${JSON.stringify(chunk)}\r\n\r\n`).join(''));
  return new ReadableStream({
    start(controller) {
      for (let i = 0; i < bytes.length; i += 5) controller.enqueue(bytes.slice(i, i + 5));
      controller.close();
    },
  });
}

test('요청 본문: 시스템·생각 수준·도구 선언·이력(서명 보존, 결과는 이름으로 짝)', () => {
  const body = buildGeminiRequest(request()) as Record<string, any>;
  assert.deepEqual(body.systemInstruction, { parts: [{ text: AI_SYSTEM_PROMPT }] });
  assert.deepEqual(body.generationConfig, { thinkingConfig: { thinkingLevel: 'low' } });
  for (const absent of ['toolConfig', 'safetySettings', 'cachedContent']) assert.ok(!(absent in body), `${absent} 를 보내지 않는다`);
  assert.deepEqual(body.contents, [
    { role: 'user', parts: [{ text: '개요 알려줘' }] },
    { role: 'model', parts: modelRaw },
    { role: 'user', parts: [
      { functionResponse: { name: 'get_outline', response: { output: '{"outline":[]}' } } },
      { functionResponse: { id: 'own-2', name: 'read_page_text', response: { error: 'Page out of range' } } },
      { text: '(알림)' },
    ] },
    { role: 'model', parts: [{ text: '(The user stopped this response.)' }] },
  ]);
  assert.equal(body.contents[1].parts[1].thoughtSignature, 'sig-A', '서명은 고치지 않고 되돌린다');
  assert.equal(JSON.stringify(buildGeminiRequest(request())), JSON.stringify(body), '같은 이력이면 같은 바이트');
});

test('도구 선언: 순서 고정, 모든 schema 가 Gemini 부분집합, 매개변수 없는 도구는 parameters 를 뺀다', () => {
  const declarations = (buildGeminiRequest(request()) as Record<string, any>).tools[0].functionDeclarations;
  assert.deepEqual(declarations.map((d: { name: string }) => d.name), AI_TOOL_DEFS.map((t) => t.name));
  for (const [index, declaration] of declarations.entries()) {
    const source = AI_TOOL_DEFS[index].inputSchema as { properties?: Record<string, unknown> };
    if (Object.keys(source.properties ?? {}).length === 0) {
      assert.ok(!('parameters' in declaration), `${declaration.name}: 빈 OBJECT 는 거절된다`);
    } else {
      assertGeminiSchema(declaration.parameters, declaration.name);
      assert.deepEqual(Object.keys(declaration.parameters.properties), Object.keys(source.properties!),
        `${declaration.name}: 매개변수를 빠뜨리지 않는다`);
    }
  }
  const read = declarations.find((d: { name: string }) => d.name === 'read_page_text');
  assert.equal(read.parameters.properties.page.minimum, 1, 'Gemini 가 받는 제약은 남긴다');
});

test('schema 변환: additionalProperties 제거, 타입 배열 → nullable, const → enum', () => {
  assert.deepEqual(toGeminiSchema({
    type: 'object',
    additionalProperties: false,
    properties: {
      a: { type: ['string', 'null'], maxLength: 5, format: 'x-custom' },
      b: { const: 'only' },
      c: { type: 'array', items: { type: 'object', additionalProperties: false, properties: { d: { type: 'integer' } } } },
    },
    required: ['a'],
  }), {
    type: 'object',
    properties: {
      a: { type: 'string', nullable: true, maxLength: 5 },
      b: { enum: ['only'] },
      c: { type: 'array', items: { type: 'object', properties: { d: { type: 'integer' } } } },
    },
    required: ['a'],
  });
});

test('스트림 part 잇기: 글끼리 잇고 서명은 그 part 로, 서명 뒤·도구 호출은 새 part', () => {
  const parts: GeminiPart[] = [];
  mergeGeminiPart(parts, { text: '표를 ' });
  mergeGeminiPart(parts, { text: '봅니다.' });
  mergeGeminiPart(parts, { text: '', thoughtSignature: 'sig-1' });
  mergeGeminiPart(parts, { text: '다음 글' });
  mergeGeminiPart(parts, { functionCall: { name: 'get_outline', args: {} } });
  mergeGeminiPart(parts, { thought: true, text: '생각' });
  assert.deepEqual(parts, [
    { text: '표를 봅니다.', thoughtSignature: 'sig-1' },
    { text: '다음 글' },
    { functionCall: { name: 'get_outline', args: {} } },
    { thought: true, text: '생각' },
  ]);
});

test('결과 → 중립: 도구 호출 id·텍스트(생각 제외)·사용량·끝 사유', () => {
  const result = toGeminiTurnResult({
    parts: [
      { thought: true, text: '속생각' },
      { text: '읽습니다.' },
      { functionCall: { name: 'get_outline', args: {} }, thoughtSignature: 's' },
      { functionCall: { id: 'x9', name: 'read_page_text', args: { page: 3 } } },
    ],
    finishReason: 'STOP',
    blocked: false,
    usage: { promptTokenCount: 900, cachedContentTokenCount: 600, candidatesTokenCount: 40, thoughtsTokenCount: 60 },
  });
  assert.equal(result.stop, 'tool_use');
  assert.equal(result.text, '읽습니다.');
  assert.deepEqual(result.toolCalls, [
    { id: 'gemini-call-0', name: 'get_outline', input: {} },
    { id: 'x9', name: 'read_page_text', input: { page: 3 } },
  ]);
  assert.deepEqual(result.usage, { inputTokens: 300, outputTokens: 100, cacheReadTokens: 600, cacheWriteTokens: 0 });

  const stop = (finishReason?: string, blocked = false, parts: GeminiPart[] = [{ text: 'a' }]) =>
    toGeminiTurnResult({ parts, finishReason, blocked }).stop;
  assert.equal(stop('STOP'), 'end');
  assert.equal(stop(undefined), 'end');
  assert.equal(stop('MAX_TOKENS'), 'max_tokens');
  assert.equal(stop('SAFETY'), 'refusal');
  assert.equal(stop('PROHIBITED_CONTENT'), 'refusal');
  assert.equal(stop(undefined, true, []), 'refusal', '질문이 막히면(promptFeedback) 거절');
  assert.equal(stop('MALFORMED_FUNCTION_CALL', false, []), 'other');
  assert.deepEqual(toGeminiTurnResult({ parts: [{ text: '' }], blocked: false }).raw, [], '서명 없는 빈 글은 되돌리지 않는다');
});

test('오류 갈래: 400 API_KEY_INVALID 는 키 오류, 429 는 retryDelay, 그 밖은 상태 코드', () => {
  assert.deepEqual(classifyGeminiError(400, {
    code: 400, message: 'API key not valid. Please pass a valid API key.', status: 'INVALID_ARGUMENT',
    details: [{ '@type': 'type.googleapis.com/google.rpc.ErrorInfo', reason: 'API_KEY_INVALID' }],
  }), { reason: 'auth', status: 400 });
  assert.deepEqual(classifyGeminiError(429, {
    code: 429, message: 'quota', status: 'RESOURCE_EXHAUSTED',
    details: [{ '@type': 'type.googleapis.com/google.rpc.RetryInfo', retryDelay: '27s' }],
  }), { reason: 'rate', status: 429, retryAfterSeconds: 27 });
  assert.deepEqual(classifyGeminiError(429, null), { reason: 'rate', status: 429 });
  assert.deepEqual(classifyGeminiError(403, { message: 'denied' }), { reason: 'permission', status: 403 });
  assert.deepEqual(classifyGeminiError(404, { message: 'models/x is not found' }), { reason: 'model', status: 404 });
  assert.deepEqual(classifyGeminiError(400, { message: 'Invalid JSON payload' }),
    { reason: 'bad-request', status: 400, detail: 'Invalid JSON payload' });
  assert.deepEqual(classifyGeminiError(503, { message: 'The model is overloaded.' }), { reason: 'overloaded', status: 503 });
});

test('연결 테스트·스트림 URL — 키는 x-goog-api-key 헤더로만, URL 에 싣지 않는다', async () => {
  assert.equal(geminiStreamUrl('gemini-3.8-flash'),
    `${GEMINI_API_BASE}/models/gemini-3.8-flash:streamGenerateContent?alt=sse`);
  const calls: Array<{ url: string; init: RequestInit }> = [];
  const ok = await testGeminiConnection(KEY, 'gemini-4-argon', undefined, async (url, init) => {
    calls.push({ url, init });
    return new Response(JSON.stringify({ name: 'models/gemini-4-argon', displayName: 'Gemini 4 Argon' }), { status: 200 });
  });
  assert.deepEqual(ok, { ok: true, modelName: 'Gemini 4 Argon' });
  assert.equal(calls[0].url, `${GEMINI_API_BASE}/models/gemini-4-argon`);
  assert.equal((calls[0].init.headers as Record<string, string>)['x-goog-api-key'], KEY);
  assert.ok(!calls[0].url.includes(KEY));
});

test('스트림: 조각을 모아 도구 호출·서명을 보존하고, 글 조각과 도구 시작을 흘린다', async () => {
  const sent: Array<{ url: string; body: any; headers: Record<string, string> }> = [];
  const provider = new GeminiProvider(KEY, async (url, init) => {
    sent.push({ url, body: JSON.parse(String(init.body)), headers: init.headers as Record<string, string> });
    return new Response(sseBody([
      { candidates: [{ content: { role: 'model', parts: [{ text: '읽어 ' }] } }] },
      { candidates: [{ content: { role: 'model', parts: [{ text: '봅니다.' }] } }] },
      { candidates: [{ content: { role: 'model', parts: [{ functionCall: { name: 'get_outline', args: {} }, thoughtSignature: 'sig-Z' }] }, finishReason: 'STOP' }],
        usageMetadata: { promptTokenCount: 50, candidatesTokenCount: 7 } },
    ]), { status: 200, headers: { 'content-type': 'text/event-stream' } });
  });
  const events: AiStreamEvent[] = [];
  const result = await provider.streamTurn(request(), new AbortController().signal, (event) => events.push(event));
  assert.equal(sent[0].url, geminiStreamUrl('gemini-3.8-flash'));
  assert.equal(sent[0].headers['x-goog-api-key'], KEY);
  assert.deepEqual(events, [
    { type: 'text-delta', text: '읽어 ' },
    { type: 'text-delta', text: '봅니다.' },
    { type: 'tool-start', id: 'gemini-call-0', name: 'get_outline' },
  ]);
  assert.equal(result.stop, 'tool_use');
  assert.deepEqual(result.raw, [
    { text: '읽어 봅니다.' },
    { functionCall: { name: 'get_outline', args: {} }, thoughtSignature: 'sig-Z' },
  ]);
  assert.deepEqual(result.usage, { inputTokens: 50, outputTokens: 7, cacheReadTokens: 0, cacheWriteTokens: 0 });
});

test('HTTP 오류·스트림 안 오류·중단은 AiProviderError 갈래로 던진다', async () => {
  const badKey = new GeminiProvider(KEY, async () => new Response(JSON.stringify({ error: {
    code: 400, message: 'API key not valid.', status: 'INVALID_ARGUMENT',
    details: [{ reason: 'API_KEY_INVALID' }],
  } }), { status: 400 }));
  await assert.rejects(badKey.streamTurn(request(), new AbortController().signal, () => {}),
    (error: unknown) => error instanceof AiProviderError && error.reason === 'auth');

  const midStream = new GeminiProvider(KEY, async () => new Response(sseBody([
    { candidates: [{ content: { parts: [{ text: '앞' }] } }] },
    { error: { code: 503, message: 'overloaded', status: 'UNAVAILABLE' } },
  ]), { status: 200 }));
  await assert.rejects(midStream.streamTurn(request(), new AbortController().signal, () => {}),
    (error: unknown) => error instanceof AiProviderError && error.reason === 'overloaded');

  const abort = new AbortController();
  const hanging = new GeminiProvider(KEY, async (_url, init) => new Promise((_resolve, reject) => {
    init.signal?.addEventListener('abort', () => reject(new DOMException('aborted', 'AbortError')));
  }));
  const pending = hanging.streamTurn(request(), abort.signal, () => {});
  abort.abort();
  await assert.rejects(pending, (error: unknown) => error instanceof AiProviderError && error.reason === 'aborted');
});
