/**
 * 제공자 중립 타입. 벤더 SDK 타입은 각 adapter 안에서만 쓴다 —
 * 다른 제공자(OpenAI·Gemini)를 같은 경계로 붙이기 위해서다.
 *
 * 대화 이력은 이 모양으로 들고, adapter 가 요청 직전에 벤더 형식으로 옮긴다.
 */

/** 연결·요청 실패의 갈래. 화면 문구는 이 값으로 고른다(제공자 원문 메시지를 그대로 보이지 않는다). */
export type AiFailureReason =
  | 'auth'
  | 'permission'
  | 'model'
  | 'rate'
  | 'overloaded'
  | 'network'
  | 'aborted'
  | 'bad-request'
  | 'unknown';

/**
 * 실패 정보. `detail` 은 400 처럼 요청 내용이 원인일 때 제공자가 알려 준 사유와 request id 다 —
 * 무엇이 거절됐는지 알아야 고칠 수 있으므로 보인다. 제공자는 키를 이 메시지에 싣지 않는다.
 */
export interface AiFailure {
  reason: AiFailureReason;
  status?: number;
  retryAfterSeconds?: number;
  detail?: string;
}

export type AiTestResult =
  | { ok: true; modelName: string }
  | ({ ok: false } & AiFailure);

/** 토큰 사용량. 회차마다 더해 패널 하단에 보인다. */
export interface AiUsage {
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
}

export const EMPTY_USAGE: AiUsage = Object.freeze({
  inputTokens: 0, outputTokens: 0, cacheReadTokens: 0, cacheWriteTokens: 0,
});

export function addUsage(a: AiUsage, b: AiUsage): AiUsage {
  return {
    inputTokens: a.inputTokens + b.inputTokens,
    outputTokens: a.outputTokens + b.outputTokens,
    cacheReadTokens: a.cacheReadTokens + b.cacheReadTokens,
    cacheWriteTokens: a.cacheWriteTokens + b.cacheWriteTokens,
  };
}

/** 모델에 알리는 도구 하나. `inputSchema` 는 `additionalProperties: false` 인 JSON Schema 객체다. */
export interface AiToolDef {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}

export interface AiToolCall {
  id: string;
  name: string;
  input: unknown;
}

export interface AiToolResult {
  id: string;
  content: string;
  isError: boolean;
}

/**
 * 대화 이력 한 칸 — 제공자 중립.
 *
 * - `user`/`text`: 사용자가 쓴 말
 * - `user`/`tool_results`: 직전 assistant 의 도구 호출 결과 전부(한 메시지에 모은다)
 * - `assistant`/`raw`: 제공자 응답 원형. 다음 요청에 **고치지 않고** 그대로 되돌린다
 *   (thinking 블록이 대화에 묶여 있어 고치면 거절된다).
 * - `assistant`/`text`: 중단·거절·오류로 끝난 회차 자리에 넣는 짧은 표시. 이력의 user/assistant
 *   교대를 지키기 위한 것이다.
 */
export type AiMessage =
  | { role: 'user'; kind: 'text'; text: string }
  | { role: 'user'; kind: 'tool_results'; results: readonly AiToolResult[] }
  | { role: 'assistant'; kind: 'raw'; raw: unknown }
  | { role: 'assistant'; kind: 'text'; text: string };

export type AiStreamEvent =
  | { type: 'text-delta'; text: string }
  | { type: 'tool-start'; id: string; name: string };

export type AiStopReason = 'end' | 'tool_use' | 'max_tokens' | 'refusal' | 'pause' | 'other';

export interface AiTurnResult {
  stop: AiStopReason;
  /** 이 회차의 화면용 텍스트(텍스트 블록을 이은 것) */
  text: string;
  toolCalls: AiToolCall[];
  usage: AiUsage;
  /** 이력에 넣을 assistant 원형 — `AiMessage.raw` 로 그대로 쓴다 */
  raw: unknown;
}

export interface AiTurnRequest {
  model: string;
  effort: 'low' | 'medium' | 'high';
  system: string;
  tools: readonly AiToolDef[];
  messages: readonly AiMessage[];
}

export interface AiProvider {
  /**
   * 한 회차를 보내고 끝날 때까지 스트림을 흘린다. 실패는 `AiProviderError` 로 던진다.
   * `signal` 이 끊기면 `reason: 'aborted'` 로 던진다.
   */
  streamTurn(
    request: AiTurnRequest,
    signal: AbortSignal,
    onEvent: (event: AiStreamEvent) => void,
  ): Promise<AiTurnResult>;
}

export class AiProviderError extends Error {
  readonly reason: AiFailureReason;
  readonly status?: number;
  readonly retryAfterSeconds?: number;
  readonly detail?: string;

  constructor(failure: AiFailure) {
    super(`AI 요청 실패: ${failure.reason}${failure.detail ? ` — ${failure.detail}` : ''}`);
    this.name = 'AiProviderError';
    this.reason = failure.reason;
    if (failure.status !== undefined) this.status = failure.status;
    if (failure.retryAfterSeconds !== undefined) this.retryAfterSeconds = failure.retryAfterSeconds;
    if (failure.detail !== undefined) this.detail = failure.detail;
  }
}
