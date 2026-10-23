/**
 * Claude adapter.
 *
 * 사용자가 등록한 키로 브라우저에서 직접 Anthropic API 를 부른다(BYOK). SDK 는 이 사용을
 * 기본으로 막으므로 `dangerouslyAllowBrowser` 로 명시한다 — 키 노출 위험과 완화책은
 * `settings/key-store.ts` 머리말과 설정 화면 안내에 있다.
 */
import Anthropic, {
  APIConnectionError,
  APIError,
  APIUserAbortError,
  AuthenticationError,
  BadRequestError,
  NotFoundError,
  PermissionDeniedError,
  RateLimitError,
} from '@anthropic-ai/sdk';
import { transformJSONSchema } from '@anthropic-ai/sdk/lib/transform-json-schema';

import { CLAUDE_MODELS } from './models.ts';
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

/** 요청 하나에 strict 로 보낼 수 있는 도구 수 상한(초과하면 400 「Too many strict tools」). */
export const CLAUDE_MAX_STRICT_TOOLS = 20;
/** strict 도구 전체의 선택(optional) 매개변수 합 상한(초과하면 400 「too many optional parameters」). */
export const CLAUDE_MAX_STRICT_OPTIONAL_PARAMS = 24;

/** 대화 한 회차의 출력 상한. 스트리밍이라 시간 초과 걱정 없이 넉넉히 둔다. */
export const CLAUDE_MAX_TOKENS = 64000;
/** 안전 분류기 거절 시 서버가 권장 모델로 다시 돌리게 하는 beta. `"default"` 형식 전용 헤더다. */
export const CLAUDE_FALLBACK_BETA = 'server-side-fallback-2026-07-01';

type ClaudeBlock = { type: string; [key: string]: unknown };
type ClaudeMessageParam = { role: 'user' | 'assistant'; content: ClaudeBlock[] };

export function createClaudeClient(apiKey: string): Anthropic {
  return new Anthropic({
    apiKey,
    dangerouslyAllowBrowser: true,
    // 재시도는 화면이 결정한다 — SDK 가 조용히 다시 보내면 중단 단추가 늦게 먹는다.
    maxRetries: 0,
  });
}

/** 400·기타 오류의 제공자 사유. 응답 본문 `{"error":{"message"}}` 와 request id. */
function errorDetail(error: APIError): string | undefined {
  const body = error.error as { error?: { message?: unknown } } | undefined;
  const message = typeof body?.error?.message === 'string' ? body.error.message : undefined;
  const parts = [message, error.requestID ? `request-id ${error.requestID}` : undefined].filter(Boolean);
  return parts.length > 0 ? parts.join(' · ') : undefined;
}

/** SDK 오류를 화면 갈래로 옮긴다. 구체적인 것부터 본다. */
export function classifyClaudeError(error: unknown): AiFailure {
  if (error instanceof APIUserAbortError) return { reason: 'aborted' };
  if (error instanceof APIConnectionError) return { reason: 'network' };
  if (error instanceof AuthenticationError) return { reason: 'auth', status: 401 };
  if (error instanceof PermissionDeniedError) return { reason: 'permission', status: 403 };
  if (error instanceof NotFoundError) return { reason: 'model', status: 404 };
  if (error instanceof RateLimitError) {
    const header = error.headers?.get('retry-after');
    const seconds = header === null || header === undefined ? NaN : Number(header);
    return Number.isFinite(seconds)
      ? { reason: 'rate', status: 429, retryAfterSeconds: seconds }
      : { reason: 'rate', status: 429 };
  }
  if (error instanceof BadRequestError) {
    const detail = errorDetail(error);
    // 크레딧 부족은 전용 오류 형식이 없고 400 invalid_request_error 의 문구로만 온다.
    // 문구가 바뀌면 일반 400 으로 물러날 뿐 사유는 그대로 보인다.
    if (detail && /credit balance is too low/i.test(detail)) return { reason: 'billing', status: 400, detail };
    return detail ? { reason: 'bad-request', status: 400, detail } : { reason: 'bad-request', status: 400 };
  }
  if (error instanceof APIError) {
    // 529 overloaded 는 전용 클래스가 없다.
    if (error.status === 529) return { reason: 'overloaded', status: 529 };
    const detail = errorDetail(error);
    return {
      reason: 'unknown',
      ...(error.status !== undefined ? { status: error.status } : {}),
      ...(detail ? { detail } : {}),
    };
  }
  if (error instanceof DOMException && error.name === 'AbortError') return { reason: 'aborted' };
  return { reason: 'unknown' };
}

/**
 * 키와 모델 접근을 확인한다. 모델 정보 조회는 토큰을 쓰지 않는다.
 */
export async function testClaudeConnection(
  apiKey: string,
  model: string,
  signal?: AbortSignal,
  client: Pick<Anthropic, 'models'> = createClaudeClient(apiKey),
): Promise<AiTestResult> {
  try {
    const info = await client.models.retrieve(model, {}, { signal });
    return { ok: true, modelName: info.display_name || model };
  } catch (error) {
    return { ok: false, ...classifyClaudeError(error) };
  }
}

/**
 * 서버 대체(fallback)가 응답 중간에 일어났으면, 되돌릴 때 경계(`fallback` 블록) 앞의
 * thinking·tool_use 등은 빼고 텍스트만 남긴다 — API 계약이다. 대체가 없으면 원형 그대로다.
 * 이 정리는 응답을 받은 그 자리에서 한 번만 한다(이후 이력은 고치지 않는다).
 */
export function echoableContent(content: readonly ClaudeBlock[]): ClaudeBlock[] {
  let boundary = -1;
  content.forEach((block, index) => { if (block.type === 'fallback') boundary = index; });
  if (boundary < 0) return [...content];
  return content.filter((block, index) => index >= boundary || block.type === 'text');
}

function toClaudeMessage(message: AiMessage): ClaudeMessageParam {
  switch (message.kind) {
    case 'text':
      return { role: message.role, content: [{ type: 'text', text: message.text }] };
    case 'tool_results':
      return {
        role: 'user',
        content: [
          ...message.results.map((result) => (result.isError
            ? { type: 'tool_result', tool_use_id: result.id, content: result.content, is_error: true }
            : { type: 'tool_result', tool_use_id: result.id, content: result.content })),
          ...(message.note ? [{ type: 'text', text: message.note }] : []),
        ],
      };
    case 'raw':
      return { role: 'assistant', content: message.raw as ClaudeBlock[] };
  }
}

/**
 * 요청 본문. 같은 이력이면 같은 바이트가 나와야 prompt cache 가 유지된다 —
 * 도구 순서·시스템 문구는 호출자가 고정하고, 여기서는 시각·난수를 넣지 않는다.
 */
export function buildClaudeRequest(request: AiTurnRequest) {
  const model = CLAUDE_MODELS.find((m) => m.id === request.model);
  const params: Record<string, unknown> = {
    model: request.model,
    max_tokens: CLAUDE_MAX_TOKENS,
    // thinking 은 생략한다 — 세 모델 모두 adaptive 로 돈다(Opus 5.5 는 끌 수도 없다).
    output_config: { effort: request.effort },
    // 최상위 자동 캐시: 도구 → 시스템 → 지난 대화까지 접두가 같으면 다시 읽는다.
    cache_control: { type: 'ephemeral' },
    system: [{ type: 'text', text: request.system }],
    // tool_choice 는 기본(auto). 이 모델들은 강제 호출(any/tool)을 거절한다.
    // eager_input_streaming 은 끈다 — P0 도구 입력은 작아 이득이 없고, 끄면 서버가 strict
    // schema 로 입력을 검증한다. strict 가 받지 않는 제약(minimum·maxLength 등)은 SDK 변환기가
    // description 으로 옮기므로, 그 제약은 실행 전 클라이언트 검증(`schema-validate.ts`)이 지킨다.
    tools: request.tools.map((tool) => (tool.strict === false
      ? { name: tool.name, description: tool.description, input_schema: tool.inputSchema }
      : {
        name: tool.name,
        description: tool.description,
        input_schema: transformJSONSchema(tool.inputSchema),
        strict: true,
      })),
    messages: request.messages.map(toClaudeMessage),
  };
  const betas: string[] = [];
  if (model?.serverFallback) {
    params.fallbacks = 'default';
    betas.push(CLAUDE_FALLBACK_BETA);
  }
  if (betas.length > 0) params.betas = betas;
  return params;
}

function toStopReason(reason: string | null): AiStopReason {
  switch (reason) {
    case 'end_turn':
    case 'stop_sequence':
      return 'end';
    case 'tool_use': return 'tool_use';
    case 'max_tokens':
    case 'model_context_window_exceeded':
      return 'max_tokens';
    case 'refusal': return 'refusal';
    case 'pause_turn': return 'pause';
    default: return 'other';
  }
}

interface ClaudeFinalMessage {
  content: ClaudeBlock[];
  stop_reason: string | null;
  usage?: {
    input_tokens?: number | null;
    output_tokens?: number | null;
    cache_read_input_tokens?: number | null;
    cache_creation_input_tokens?: number | null;
  };
}

/** 완성된 응답을 중립 결과로. 스트림 없이 시험할 수 있게 따로 둔다. */
export function toTurnResult(message: ClaudeFinalMessage): AiTurnResult {
  const raw = echoableContent(message.content);
  const toolCalls: AiToolCall[] = raw
    .filter((block) => block.type === 'tool_use')
    .map((block) => ({ id: String(block.id), name: String(block.name), input: block.input }));
  const text = raw
    .filter((block) => block.type === 'text')
    .map((block) => String(block.text ?? ''))
    .join('');
  const usage = message.usage ?? {};
  return {
    stop: toStopReason(message.stop_reason),
    text,
    toolCalls,
    raw,
    usage: {
      inputTokens: usage.input_tokens ?? 0,
      outputTokens: usage.output_tokens ?? 0,
      cacheReadTokens: usage.cache_read_input_tokens ?? 0,
      cacheWriteTokens: usage.cache_creation_input_tokens ?? 0,
    },
  };
}

/** 사용자 키로 만든 Claude provider. 키는 생성 시점에 받아 SDK 클라이언트 안에만 둔다. */
export class ClaudeProvider implements AiProvider {
  private readonly client: Pick<Anthropic, 'beta'>;

  constructor(apiKey: string, client: Pick<Anthropic, 'beta'> = createClaudeClient(apiKey)) {
    this.client = client;
  }

  async streamTurn(
    request: AiTurnRequest,
    signal: AbortSignal,
    onEvent: (event: AiStreamEvent) => void,
  ): Promise<AiTurnResult> {
    try {
      const stream = this.client.beta.messages.stream(
        buildClaudeRequest(request) as unknown as Parameters<Anthropic['beta']['messages']['stream']>[0],
        { signal },
      );
      stream.on('text', (delta) => onEvent({ type: 'text-delta', text: delta }));
      stream.on('streamEvent', (event) => {
        if (event.type === 'content_block_start' && event.content_block.type === 'tool_use') {
          onEvent({ type: 'tool-start', id: event.content_block.id, name: event.content_block.name });
        }
      });
      const message = await stream.finalMessage();
      return toTurnResult(message as unknown as ClaudeFinalMessage);
    } catch (error) {
      if (signal.aborted) throw new AiProviderError({ reason: 'aborted' });
      throw new AiProviderError(classifyClaudeError(error));
    }
  }
}
