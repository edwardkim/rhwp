/**
 * SDK 없이 `fetch` 로 부르는 제공자(OpenAI·Gemini)의 공용 부품 — SSE 읽기와 HTTP 오류 갈래.
 *
 * 두 제공자는 브라우저 요청을 CORS 로 받는다. 벤더 SDK 를 들이지 않는 것은 번들을 키우지 않고,
 * 요청 본문을 이 모듈에서 바이트 단위로 고정해 시험하기 위해서다. 키는 헤더로만 보내고
 * URL·로그·오류 메시지에 싣지 않는다.
 */
import { AiProviderError, type AiFailure, type AiFailureReason } from './types.ts';

export type FetchLike = (input: string, init: RequestInit) => Promise<Response>;

export function defaultFetch(): FetchLike {
  return (input, init) => globalThis.fetch(input, init);
}

export interface SseEvent {
  event: string;
  data: string;
}

/**
 * `text/event-stream` 본문을 이벤트 단위로 넘긴다. 빈 줄이 이벤트 경계이고, 한 이벤트의
 * 여러 `data:` 줄은 줄바꿈으로 잇는다(SSE 규격). 청크가 줄 중간에서 끊겨도 이어 붙인다.
 */
export async function readSse(body: ReadableStream<Uint8Array>, onEvent: (event: SseEvent) => void): Promise<void> {
  const reader = body.getReader();
  const decoder = new TextDecoder();
  let buffer = '';
  let event = '';
  let data: string[] = [];
  const dispatch = () => {
    if (data.length > 0) onEvent({ event: event || 'message', data: data.join('\n') });
    event = '';
    data = [];
  };
  const consumeLine = (line: string) => {
    if (line === '') { dispatch(); return; }
    if (line.startsWith(':')) return;
    const colon = line.indexOf(':');
    const field = colon < 0 ? line : line.slice(0, colon);
    let value = colon < 0 ? '' : line.slice(colon + 1);
    if (value.startsWith(' ')) value = value.slice(1);
    if (field === 'event') event = value;
    else if (field === 'data') data.push(value);
  };
  for (;;) {
    const { done, value } = await reader.read();
    buffer += done ? decoder.decode() : decoder.decode(value, { stream: true });
    let newline;
    while ((newline = buffer.search(/\r\n|\r|\n/)) >= 0) {
      const line = buffer.slice(0, newline);
      buffer = buffer.slice(newline + (buffer.startsWith('\r\n', newline) ? 2 : 1));
      consumeLine(line);
    }
    if (done) break;
  }
  if (buffer.length > 0) consumeLine(buffer);
  dispatch();
}

/** 오류 본문 `{"error":{...}}` 를 읽는다. JSON 이 아니면 null. */
export async function readErrorBody(response: Response): Promise<Record<string, unknown> | null> {
  try {
    const body = await response.json() as { error?: unknown };
    return body && typeof body.error === 'object' && body.error !== null ? body.error as Record<string, unknown> : null;
  } catch {
    return null;
  }
}

export function retryAfterSeconds(response: Response): number | undefined {
  const header = response.headers.get('retry-after');
  const seconds = header === null ? NaN : Number(header);
  return Number.isFinite(seconds) ? seconds : undefined;
}

/** 상태 코드만으로 고르는 기본 갈래. 제공자별 세부 갈래(크레딧·키 무효)는 각 adapter 가 먼저 본다. */
export function reasonFromStatus(status: number): AiFailureReason {
  switch (status) {
    case 400: return 'bad-request';
    case 401: return 'auth';
    case 402: return 'billing';
    case 403: return 'permission';
    case 404: return 'model';
    case 429: return 'rate';
    case 503:
    case 529:
      return 'overloaded';
    default: return 'unknown';
  }
}

export function joinDetail(...parts: Array<string | undefined>): string | undefined {
  const kept = parts.filter((part): part is string => typeof part === 'string' && part.length > 0);
  return kept.length > 0 ? kept.join(' · ') : undefined;
}

/** `fetch` 단계의 예외(중단·네트워크)를 갈래로 옮긴다. */
export function fetchFailure(error: unknown, signal: AbortSignal | undefined): AiFailure {
  if (error instanceof AiProviderError) {
    return {
      reason: error.reason,
      ...(error.status !== undefined ? { status: error.status } : {}),
      ...(error.retryAfterSeconds !== undefined ? { retryAfterSeconds: error.retryAfterSeconds } : {}),
      ...(error.detail !== undefined ? { detail: error.detail } : {}),
    };
  }
  if (signal?.aborted) return { reason: 'aborted' };
  if (error instanceof DOMException && error.name === 'AbortError') return { reason: 'aborted' };
  // 브라우저는 CORS·DNS·오프라인 실패를 모두 TypeError 로 던진다.
  if (error instanceof TypeError) return { reason: 'network' };
  return { reason: 'unknown' };
}

export function parseJson(data: string): unknown {
  try { return JSON.parse(data); } catch { return undefined; }
}
