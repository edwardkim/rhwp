/**
 * 제공자 중립 타입. 벤더 SDK 타입은 각 adapter 안에서만 쓴다 —
 * 다른 제공자(OpenAI·Gemini)를 같은 경계로 붙이기 위해서다.
 *
 * Stage 2 는 연결 테스트까지. 대화 스트림(`streamTurn`)은 Stage 3 에서 더한다.
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

export type AiTestResult =
  | { ok: true; modelName: string }
  | { ok: false; reason: AiFailureReason; status?: number; retryAfterSeconds?: number };
