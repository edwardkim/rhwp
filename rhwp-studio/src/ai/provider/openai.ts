/**
 * OpenAI adapter — Responses API(`POST /v1/responses`)를 브라우저에서 직접 부른다(BYOK).
 *
 * - 서버에 대화를 남기지 않는다(`store: false`). 대신 응답의 출력 항목(reasoning·message·
 *   function_call)을 고치지 않고 다음 요청의 `input` 에 되돌린다 — 무상태 모드의 reasoning 항목은
 *   `encrypted_content` 를 싣고 와서, 그대로 돌려주면 도구 사이에서도 추론이 이어진다.
 * - 도구는 strict 를 끈다. strict 는 모든 속성을 required 로 요구해 선택 매개변수가 많은
 *   쓰기 도구와 맞지 않는다 — 입력은 실행 전 클라이언트 검증(`schema-validate.ts`)이 지킨다.
 * - prompt cache 는 자동이다. 같은 이력이면 같은 바이트가 나오도록 시각·난수를 넣지 않는다.
 */
import { OPENAI_MODELS } from './models.ts';
import {
  defaultFetch,
  fetchFailure,
  joinDetail,
  parseJson,
  readErrorBody,
  readSse,
  reasonFromStatus,
  retryAfterSeconds,
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

export const OPENAI_API_BASE = 'https://api.openai.com/v1';

type OpenAiItem = { type?: string; [key: string]: unknown };

/** 크레딧·지출 한도 소진 — 키는 맞지만 결제 쪽을 고쳐야 한다(다시 시도해도 풀리지 않는다). */
const BILLING_CODES = /^(insufficient_quota|credit_balance_exhausted|billing_hard_limit_reached)$|_(spend|usage)_limit_exceeded$/;

/** 오류 본문 `{"error":{message,type,code}}` 와 상태 코드를 화면 갈래로 옮긴다. */
export function classifyOpenAiError(
  status: number,
  error: Record<string, unknown> | null,
  requestId?: string | null,
  retryAfter?: number,
): AiFailure {
  const message = typeof error?.message === 'string' ? error.message : undefined;
  const code = typeof error?.code === 'string' ? error.code : '';
  const type = typeof error?.type === 'string' ? error.type : '';
  const detail = joinDetail(message, requestId ? `request-id ${requestId}` : undefined);
  const withDetail = (failure: AiFailure): AiFailure => (detail ? { ...failure, detail } : failure);
  if (BILLING_CODES.test(code) || BILLING_CODES.test(type)) return withDetail({ reason: 'billing', status });
  if (code === 'model_not_found') return withDetail({ reason: 'model', status });
  const reason = reasonFromStatus(status);
  if (reason === 'rate') {
    return retryAfter === undefined ? { reason, status } : { reason, status, retryAfterSeconds: retryAfter };
  }
  if (reason === 'bad-request' || reason === 'unknown') return withDetail({ reason, status });
  return { reason, status };
}

async function failureFromResponse(response: Response): Promise<AiFailure> {
  return classifyOpenAiError(
    response.status,
    await readErrorBody(response),
    response.headers.get('x-request-id'),
    retryAfterSeconds(response),
  );
}

function headers(apiKey: string): Record<string, string> {
  return { authorization: `Bearer ${apiKey}`, 'content-type': 'application/json' };
}

/** 키와 모델 접근을 확인한다. 모델 정보 조회는 토큰을 쓰지 않는다. */
export async function testOpenAiConnection(
  apiKey: string,
  model: string,
  signal?: AbortSignal,
  fetchImpl: FetchLike = defaultFetch(),
): Promise<AiTestResult> {
  try {
    const response = await fetchImpl(`${OPENAI_API_BASE}/models/${encodeURIComponent(model)}`, {
      method: 'GET',
      headers: { authorization: `Bearer ${apiKey}` },
      ...(signal ? { signal } : {}),
    });
    if (!response.ok) return { ok: false, ...await failureFromResponse(response) };
    const label = OPENAI_MODELS.find((m) => m.id === model)?.label;
    return { ok: true, modelName: label ?? model };
  } catch (error) {
    return { ok: false, ...fetchFailure(error, signal) };
  }
}

function toOpenAiInput(messages: readonly AiMessage[]): OpenAiItem[] {
  const input: OpenAiItem[] = [];
  for (const message of messages) {
    switch (message.kind) {
      case 'text':
        input.push(message.role === 'user'
          ? { role: 'user', content: [{ type: 'input_text', text: message.text }] }
          : { role: 'assistant', content: message.text });
        break;
      case 'tool_results':
        // 결과마다 항목 하나. 오류 표시 칸이 없어 문구 앞에 붙인다.
        for (const result of message.results) {
          input.push({
            type: 'function_call_output',
            call_id: result.id,
            output: result.isError ? `Error: ${result.content}` : result.content,
          });
        }
        if (message.note) input.push({ role: 'user', content: [{ type: 'input_text', text: message.note }] });
        break;
      case 'raw':
        input.push(...message.raw as OpenAiItem[]);
        break;
    }
  }
  return input;
}

/** 요청 본문. 같은 이력이면 같은 바이트가 나온다. */
export function buildOpenAiRequest(request: AiTurnRequest) {
  return {
    model: request.model,
    instructions: request.system,
    input: toOpenAiInput(request.messages),
    tools: request.tools.map((tool) => ({
      type: 'function',
      name: tool.name,
      description: tool.description,
      parameters: tool.inputSchema,
      strict: false,
    })),
    reasoning: { effort: request.effort },
    store: false,
    // 무상태 모드에서는 기본으로 실리지만, 이전 계약의 값도 받으므로 명시해 둔다.
    include: ['reasoning.encrypted_content'],
    stream: true,
  };
}

interface OpenAiResponse {
  status?: string;
  output?: OpenAiItem[];
  incomplete_details?: { reason?: string } | null;
  error?: { code?: string; message?: string } | null;
  usage?: {
    input_tokens?: number;
    output_tokens?: number;
    input_tokens_details?: { cached_tokens?: number };
  };
}

function parseArguments(raw: unknown): unknown {
  if (typeof raw !== 'string') return raw;
  if (raw.trim() === '') return {};
  // 깨진 JSON 은 원문 그대로 넘긴다 — 클라이언트 검증이 거절하고 모델이 다시 보낸다.
  const parsed = parseJson(raw);
  return parsed === undefined ? raw : parsed;
}

/** 완성된 응답을 중립 결과로. 스트림 없이 시험할 수 있게 따로 둔다. */
export function toOpenAiTurnResult(response: OpenAiResponse): AiTurnResult {
  const raw = [...response.output ?? []];
  const toolCalls: AiToolCall[] = raw
    .filter((item) => item.type === 'function_call')
    .map((item) => ({ id: String(item.call_id), name: String(item.name), input: parseArguments(item.arguments) }));
  const parts = raw
    .filter((item) => item.type === 'message')
    .flatMap((item) => (Array.isArray(item.content) ? item.content as OpenAiItem[] : []));
  const text = parts
    .filter((part) => part.type === 'output_text')
    .map((part) => String(part.text ?? ''))
    .join('');
  const refused = parts.some((part) => part.type === 'refusal');

  let stop: AiStopReason;
  if (response.status === 'incomplete') {
    const reason = response.incomplete_details?.reason;
    stop = reason === 'max_output_tokens' ? 'max_tokens' : reason === 'content_filter' ? 'refusal' : 'other';
  } else if (refused && toolCalls.length === 0) {
    stop = 'refusal';
  } else {
    stop = toolCalls.length > 0 ? 'tool_use' : 'end';
  }

  const usage = response.usage ?? {};
  const cached = usage.input_tokens_details?.cached_tokens ?? 0;
  return {
    stop,
    text,
    toolCalls,
    raw,
    usage: {
      // input_tokens 는 캐시에서 읽은 부분을 포함한다 — Claude 와 같은 뜻이 되도록 뺀다.
      inputTokens: Math.max(0, (usage.input_tokens ?? 0) - cached),
      outputTokens: usage.output_tokens ?? 0,
      cacheReadTokens: cached,
      cacheWriteTokens: 0,
    },
  };
}

/** 사용자 키로 만든 OpenAI provider. 키는 생성 시점에 받아 이 인스턴스 안에만 둔다. */
export class OpenAiProvider implements AiProvider {
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
      const response = await this.fetchImpl(`${OPENAI_API_BASE}/responses`, {
        method: 'POST',
        headers: headers(this.apiKey),
        body: JSON.stringify(buildOpenAiRequest(request)),
        signal,
      });
      if (!response.ok) throw new AiProviderError(await failureFromResponse(response));
      if (!response.body) throw new AiProviderError({ reason: 'unknown' });
      const requestId = response.headers.get('x-request-id');

      // 콜백 안에서 채운다 — 지역 변수로 두면 TS 가 null 로 좁혀 버린다.
      const state: { final: OpenAiResponse | null; error: AiFailure | null } = { final: null, error: null };
      await readSse(response.body, ({ data }) => {
        const event = parseJson(data) as { type?: string; [key: string]: unknown } | undefined;
        if (!event) return;
        switch (event.type) {
          case 'response.output_text.delta':
            if (typeof event.delta === 'string') onEvent({ type: 'text-delta', text: event.delta });
            break;
          case 'response.output_item.added': {
            const item = event.item as OpenAiItem | undefined;
            if (item?.type === 'function_call') {
              onEvent({ type: 'tool-start', id: String(item.call_id), name: String(item.name) });
            }
            break;
          }
          case 'response.completed':
          case 'response.incomplete':
            state.final = event.response as OpenAiResponse;
            break;
          case 'response.failed': {
            const failed = (event.response as OpenAiResponse | undefined)?.error ?? null;
            state.error = classifyOpenAiError(500, failed as Record<string, unknown> | null, requestId);
            break;
          }
          case 'error':
            // 스트림 중 오류는 상태 코드 없이 code·message 만 온다.
            state.error = classifyOpenAiError(
              event.code === 'rate_limit_exceeded' ? 429 : 500,
              event as Record<string, unknown>,
              requestId,
            );
            break;
        }
      });
      if (state.error) throw new AiProviderError(state.error);
      if (!state.final) throw new AiProviderError({ reason: signal.aborted ? 'aborted' : 'unknown' });
      return toOpenAiTurnResult(state.final);
    } catch (error) {
      if (signal.aborted) throw new AiProviderError({ reason: 'aborted' });
      throw new AiProviderError(fetchFailure(error, signal));
    }
  }
}
