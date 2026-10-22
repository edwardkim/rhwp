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

import type { AiFailureReason, AiTestResult } from './types.ts';

export function createClaudeClient(apiKey: string): Anthropic {
  return new Anthropic({
    apiKey,
    dangerouslyAllowBrowser: true,
    // 재시도는 화면이 결정한다 — SDK 가 조용히 다시 보내면 중단 단추가 늦게 먹는다.
    maxRetries: 0,
  });
}

/** SDK 오류를 화면 갈래로 옮긴다. 구체적인 것부터 본다. */
export function classifyClaudeError(error: unknown): {
  reason: AiFailureReason;
  status?: number;
  retryAfterSeconds?: number;
} {
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
  if (error instanceof BadRequestError) return { reason: 'bad-request', status: 400 };
  if (error instanceof APIError) {
    // 529 overloaded 는 전용 클래스가 없다.
    if (error.status === 529) return { reason: 'overloaded', status: 529 };
    return error.status === undefined ? { reason: 'unknown' } : { reason: 'unknown', status: error.status };
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
