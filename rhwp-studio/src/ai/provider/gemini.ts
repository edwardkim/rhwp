/**
 * Gemini adapter — Gemini API `streamGenerateContent`(SSE)를 브라우저에서 직접 부른다(BYOK).
 *
 * - 키는 `x-goog-api-key` 헤더로만 보낸다. 문서 예제처럼 `?key=` 로 URL 에 싣지 않는다.
 * - 응답 part 의 `thoughtSignature` 는 추론을 다음 회차로 잇는 값이라, 받은 part 를 고치지 않고
 *   다음 요청에 되돌린다(빠뜨리면 도구 호출 뒤 요청이 거절된다). 스트림 조각은 같은 종류의
 *   글 part 끼리만 이어 붙이고, 서명은 그 part 에 그대로 옮긴다.
 * - 도구 schema 는 Gemini 가 받는 OpenAPI 부분집합으로 옮긴다(`toGeminiSchema`).
 */
import { GEMINI_MODELS } from './models.ts';
import {
  defaultFetch,
  fetchFailure,
  joinDetail,
  parseJson,
  readErrorBody,
  readSse,
  reasonFromStatus,
  type FetchLike,
} from './http.ts';
import {
  AiProviderError,
  type AiFailure,
  type AiMessage,
  type AiProvider,
  type AiStopReason,
  type AiStreamEvent,
  type AiTestResult,
  type AiToolCall,
  type AiTurnRequest,
  type AiTurnResult,
} from './types.ts';

export const GEMINI_API_BASE = 'https://generativelanguage.googleapis.com/v1beta';

export interface GeminiPart {
  text?: string;
  thought?: boolean;
  thoughtSignature?: string;
  functionCall?: { id?: string; name: string; args?: Record<string, unknown> };
  functionResponse?: { id?: string; name: string; response: Record<string, unknown> };
  [key: string]: unknown;
}

interface GeminiContent {
  role: 'user' | 'model';
  parts: GeminiPart[];
}

/** Gemini `Schema` 가 받는 키. `additionalProperties`·`$schema` 등은 보내면 400 이다. */
const SCHEMA_KEYS = new Set([
  'type', 'description', 'nullable', 'enum', 'items', 'properties', 'required',
  'minItems', 'maxItems', 'minLength', 'maxLength', 'pattern', 'minimum', 'maximum',
  'anyOf', 'default', 'title',
]);

/**
 * JSON Schema 를 Gemini 부분집합으로 옮긴다. 받지 않는 제약은 버린다 — 그 제약은 실행 전
 * 클라이언트 검증(`schema-validate.ts`)이 지킨다. `["string","null"]` 같은 타입 배열은
 * `nullable` 로, `const` 는 원소 하나짜리 `enum` 으로 바꾼다.
 */
export function toGeminiSchema(schema: unknown): unknown {
  if (Array.isArray(schema)) return schema.map(toGeminiSchema);
  if (!schema || typeof schema !== 'object') return schema;
  const source = schema as Record<string, unknown>;
  const out: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(source)) {
    if (key === 'properties' && value && typeof value === 'object') {
      out.properties = Object.fromEntries(Object.entries(value as Record<string, unknown>)
        .map(([name, child]) => [name, toGeminiSchema(child)]));
    } else if (key === 'type' && Array.isArray(value)) {
      const types = value.filter((type) => type !== 'null');
      if (types.length === 1) out.type = types[0];
      if (types.length !== value.length) out.nullable = true;
    } else if (key === 'const') {
      out.enum = [value];
    } else if (key === 'items' || key === 'anyOf') {
      out[key] = toGeminiSchema(value);
    } else if (SCHEMA_KEYS.has(key)) {
      out[key] = value;
    }
  }
  return out;
}

/** 매개변수가 없는 도구는 `parameters` 를 뺀다 — 속성이 빈 OBJECT 는 거절된다. */
function functionDeclaration(tool: AiTurnRequest['tools'][number]) {
  const properties = (tool.inputSchema as { properties?: Record<string, unknown> }).properties;
  const hasParams = properties !== undefined && Object.keys(properties).length > 0;
  return {
    name: tool.name,
    description: tool.description,
    ...(hasParams ? { parameters: toGeminiSchema(tool.inputSchema) } : {}),
  };
}

/**
 * 응답 part 의 도구 호출과 화면용 id. Gemini 가 id 를 주지 않으면 순서로 만든다 — 결과를
 * 되돌릴 때 같은 part 에서 다시 계산하므로 이력에 따로 적어 둘 필요가 없다.
 */
export function geminiCalls(parts: readonly GeminiPart[]): Array<{ id: string; name: string; ownId: boolean; args: unknown }> {
  return parts
    .filter((part) => part.functionCall)
    .map((part, index) => ({
      id: part.functionCall!.id ?? `gemini-call-${index}`,
      name: part.functionCall!.name,
      ownId: part.functionCall!.id !== undefined,
      args: part.functionCall!.args ?? {},
    }));
}

function toGeminiContents(messages: readonly AiMessage[]): GeminiContent[] {
  const contents: GeminiContent[] = [];
  let lastModelParts: readonly GeminiPart[] = [];
  for (const message of messages) {
    switch (message.kind) {
      case 'text':
        contents.push({ role: message.role === 'user' ? 'user' : 'model', parts: [{ text: message.text }] });
        break;
      case 'tool_results': {
        // 결과는 이름으로 짝을 짓는다. 직전 응답의 호출 목록에서 id → 이름을 찾는다.
        const calls = geminiCalls(lastModelParts);
        contents.push({
          role: 'user',
          parts: [
            ...message.results.map((result): GeminiPart => {
              const call = calls.find((c) => c.id === result.id);
              return {
                functionResponse: {
                  ...(call?.ownId ? { id: result.id } : {}),
                  name: call?.name ?? 'unknown',
                  response: result.isError ? { error: result.content } : { output: result.content },
                },
              };
            }),
            ...(message.note ? [{ text: message.note }] : []),
          ],
        });
        break;
      }
      case 'raw':
        lastModelParts = message.raw as GeminiPart[];
        contents.push({ role: 'model', parts: [...lastModelParts] });
        break;
    }
  }
  return contents;
}

/** 요청 본문. 같은 이력이면 같은 바이트가 나와 암묵적 캐시가 접두를 다시 읽는다. */
export function buildGeminiRequest(request: AiTurnRequest) {
  return {
    systemInstruction: { parts: [{ text: request.system }] },
    contents: toGeminiContents(request.messages),
    tools: [{ functionDeclarations: request.tools.map(functionDeclaration) }],
    generationConfig: { thinkingConfig: { thinkingLevel: request.effort } },
  };
}

export function geminiStreamUrl(model: string): string {
  return `${GEMINI_API_BASE}/models/${encodeURIComponent(model)}:streamGenerateContent?alt=sse`;
}

/** `RetryInfo.retryDelay`("30s") 를 초로. */
function retryDelaySeconds(error: Record<string, unknown> | null): number | undefined {
  const details = Array.isArray(error?.details) ? error.details as Array<Record<string, unknown>> : [];
  for (const detail of details) {
    const match = typeof detail.retryDelay === 'string' ? /^(\d+(?:\.\d+)?)s$/.exec(detail.retryDelay) : null;
    if (match) return Number(match[1]);
  }
  return undefined;
}

function errorReasons(error: Record<string, unknown> | null): string[] {
  const details = Array.isArray(error?.details) ? error.details as Array<Record<string, unknown>> : [];
  return details.map((detail) => detail.reason).filter((reason): reason is string => typeof reason === 'string');
}

/**
 * 오류 본문 `{"error":{code,message,status,details}}` 를 화면 갈래로. 키가 틀리면 401 이 아니라
 * 400 `API_KEY_INVALID` 로 오는 경우가 있어 사유를 먼저 본다.
 */
export function classifyGeminiError(status: number, error: Record<string, unknown> | null): AiFailure {
  const message = typeof error?.message === 'string' ? error.message : undefined;
  const reasons = errorReasons(error);
  if (reasons.includes('API_KEY_INVALID') || (status === 400 && message && /API key not valid/i.test(message))) {
    return { reason: 'auth', status };
  }
  const reason = reasonFromStatus(status);
  if (reason === 'rate') {
    const seconds = retryDelaySeconds(error);
    return seconds === undefined ? { reason, status } : { reason, status, retryAfterSeconds: seconds };
  }
  const detail = joinDetail(message);
  if ((reason === 'bad-request' || reason === 'unknown') && detail) return { reason, status, detail };
  return { reason, status };
}

function headers(apiKey: string): Record<string, string> {
  return { 'x-goog-api-key': apiKey, 'content-type': 'application/json' };
}

/** 키와 모델 접근을 확인한다. 모델 정보 조회는 토큰을 쓰지 않는다. */
export async function testGeminiConnection(
  apiKey: string,
  model: string,
  signal?: AbortSignal,
  fetchImpl: FetchLike = defaultFetch(),
): Promise<AiTestResult> {
  try {
    const response = await fetchImpl(`${GEMINI_API_BASE}/models/${encodeURIComponent(model)}`, {
      method: 'GET',
      headers: { 'x-goog-api-key': apiKey },
      ...(signal ? { signal } : {}),
    });
    if (!response.ok) return { ok: false, ...classifyGeminiError(response.status, await readErrorBody(response)) };
    const info = await response.json().catch(() => null) as { displayName?: unknown } | null;
    const name = typeof info?.displayName === 'string' && info.displayName
      ? info.displayName
      : GEMINI_MODELS.find((m) => m.id === model)?.label ?? model;
    return { ok: true, modelName: name };
  } catch (error) {
    return { ok: false, ...fetchFailure(error, signal) };
  }
}

/**
 * 스트림 조각의 part 를 앞의 것에 잇는다. 글 part 는 같은 종류(일반·생각)이고 앞 part 에 아직
 * 서명이 없을 때만 잇는다 — 서명이 붙은 part 뒤의 글을 합치면 서명이 다른 글에 묶인다.
 */
export function mergeGeminiPart(parts: GeminiPart[], incoming: GeminiPart): void {
  const last = parts[parts.length - 1];
  const isText = (part: GeminiPart | undefined) => part !== undefined
    && typeof part.text === 'string' && Object.keys(part).every((key) => key === 'text' || key === 'thought' || key === 'thoughtSignature');
  if (isText(incoming) && isText(last) && !last!.thoughtSignature && Boolean(last!.thought) === Boolean(incoming.thought)) {
    last!.text = `${last!.text}${incoming.text}`;
    if (incoming.thoughtSignature) last!.thoughtSignature = incoming.thoughtSignature;
    return;
  }
  parts.push({ ...incoming });
}

const REFUSAL_FINISH = new Set([
  'SAFETY', 'RECITATION', 'LANGUAGE', 'BLOCKLIST', 'PROHIBITED_CONTENT', 'SPII', 'IMAGE_SAFETY',
]);

interface GeminiTurnState {
  parts: GeminiPart[];
  finishReason?: string;
  blocked: boolean;
  usage?: {
    promptTokenCount?: number;
    candidatesTokenCount?: number;
    thoughtsTokenCount?: number;
    cachedContentTokenCount?: number;
  };
}

/** 모은 part 를 중립 결과로. 스트림 없이 시험할 수 있게 따로 둔다. */
export function toGeminiTurnResult(state: GeminiTurnState): AiTurnResult {
  // 빈 글 part(서명도 없는 것)는 되돌릴 이유가 없다.
  const raw = state.parts.filter((part) => !(Object.keys(part).length === 1 && part.text === ''));
  const toolCalls: AiToolCall[] = geminiCalls(raw).map((call) => ({ id: call.id, name: call.name, input: call.args }));
  const text = raw
    .filter((part) => typeof part.text === 'string' && !part.thought)
    .map((part) => part.text)
    .join('');

  let stop: AiStopReason;
  if (state.blocked || (state.finishReason && REFUSAL_FINISH.has(state.finishReason))) stop = 'refusal';
  else if (state.finishReason === 'MAX_TOKENS') stop = 'max_tokens';
  else if (toolCalls.length > 0) stop = 'tool_use';
  else if (state.finishReason === undefined || state.finishReason === 'STOP') stop = 'end';
  else stop = 'other';

  const usage = state.usage ?? {};
  const cached = usage.cachedContentTokenCount ?? 0;
  return {
    stop,
    text,
    toolCalls,
    raw,
    usage: {
      // promptTokenCount 는 캐시에서 읽은 부분을 포함한다 — Claude 와 같은 뜻이 되도록 뺀다.
      inputTokens: Math.max(0, (usage.promptTokenCount ?? 0) - cached),
      outputTokens: (usage.candidatesTokenCount ?? 0) + (usage.thoughtsTokenCount ?? 0),
      cacheReadTokens: cached,
      cacheWriteTokens: 0,
    },
  };
}

/** 사용자 키로 만든 Gemini provider. 키는 생성 시점에 받아 이 인스턴스 안에만 둔다. */
export class GeminiProvider implements AiProvider {
  private readonly apiKey: string;
  private readonly fetchImpl: FetchLike;

  constructor(apiKey: string, fetchImpl: FetchLike = defaultFetch()) {
    this.apiKey = apiKey;
    this.fetchImpl = fetchImpl;
  }

  async streamTurn(
    request: AiTurnRequest,
    signal: AbortSignal,
    onEvent: (event: AiStreamEvent) => void,
  ): Promise<AiTurnResult> {
    try {
      const response = await this.fetchImpl(geminiStreamUrl(request.model), {
        method: 'POST',
        headers: headers(this.apiKey),
        body: JSON.stringify(buildGeminiRequest(request)),
        signal,
      });
      if (!response.ok) throw new AiProviderError(classifyGeminiError(response.status, await readErrorBody(response)));
      if (!response.body) throw new AiProviderError({ reason: 'unknown' });

      const state: GeminiTurnState & { error: AiFailure | null; calls: number } = {
        parts: [], blocked: false, error: null, calls: 0,
      };
      await readSse(response.body, ({ data }) => {
        const chunk = parseJson(data) as Record<string, unknown> | undefined;
        if (!chunk) return;
        if (chunk.error && typeof chunk.error === 'object') {
          const error = chunk.error as Record<string, unknown>;
          state.error = classifyGeminiError(typeof error.code === 'number' ? error.code : 500, error);
          return;
        }
        if ((chunk.promptFeedback as { blockReason?: unknown } | undefined)?.blockReason) state.blocked = true;
        if (chunk.usageMetadata) state.usage = chunk.usageMetadata as GeminiTurnState['usage'];
        const candidate = (chunk.candidates as Array<Record<string, unknown>> | undefined)?.[0];
        if (!candidate) return;
        if (typeof candidate.finishReason === 'string') state.finishReason = candidate.finishReason;
        const parts = (candidate.content as { parts?: GeminiPart[] } | undefined)?.parts ?? [];
        for (const part of parts) {
          if (typeof part.text === 'string' && part.text && !part.thought) onEvent({ type: 'text-delta', text: part.text });
          if (part.functionCall) {
            onEvent({
              type: 'tool-start',
              id: part.functionCall.id ?? `gemini-call-${state.calls}`,
              name: part.functionCall.name,
            });
            state.calls += 1;
          }
          mergeGeminiPart(state.parts, part);
        }
      });
      if (state.error) throw new AiProviderError(state.error);
      return toGeminiTurnResult(state);
    } catch (error) {
      if (signal.aborted) throw new AiProviderError({ reason: 'aborted' });
      throw new AiProviderError(fetchFailure(error, signal));
    }
  }
}
