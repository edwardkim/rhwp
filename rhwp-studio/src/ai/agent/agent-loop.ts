/**
 * 대화 엔진 — 사용자 말 한 번을 끝까지 처리한다.
 *
 *   사용자 말 덧붙임 → 응답 스트림 → (도구 호출이면) 도구 실행 → 결과를 한 메시지로 덧붙임 → 반복
 *
 * 이력은 항상 유효한 상태로 끝난다. 중단·거절·오류로 회차가 끝나면 그 응답 대신 짧은
 * assistant 표시를 덧붙여 user/assistant 교대를 지킨다 — 그래야 다음 질문을 이어서 할 수 있다.
 */
import {
  AiProviderError,
  EMPTY_USAGE,
  addUsage,
  type AiProvider,
  type AiToolCall,
  type AiToolDef,
  type AiToolResult,
  type AiUsage,
} from '../provider/types.ts';
import type { ChatSession } from './chat-session.ts';

/**
 * 사용자 말 한 번에 허용하는 도구 라운드 수. 문서를 새로 만드는 작업은 변경 하나하나가 라운드라
 * 8회로는 중간에 끊겼다(실측: 회의록 양식 만들기가 표를 다듬던 중 멈춤). 끝없는 반복만 막는 값이다.
 */
export const MAX_TOOL_ROUNDS = 30;

/** 회차가 정상으로 끝나지 않았을 때 이력에 남기는 표시. 모델이 읽는다. */
export const INTERRUPTED_MARKERS = {
  aborted: '(The user stopped this response.)',
  refusal: '(This response was declined.)',
  error: '(This response failed because of an error.)',
  max_tokens: '(This response was cut off because it was too long.)',
  round_limit: '(Stopped: too many tool calls for one question.)',
} as const;

export type AgentRunStatus = 'done' | 'aborted' | 'refusal' | 'error' | 'max_tokens' | 'round_limit';

export interface AgentRunResult {
  status: AgentRunStatus;
  /** 마지막 회차의 응답 텍스트(화면 표시용) */
  text: string;
  usage: AiUsage;
  toolRounds: number;
  error?: AiProviderError;
}

export interface AgentCallbacks {
  onText?(delta: string): void;
  /** 새 응답 회차가 시작될 때(도구 실행 뒤 이어지는 응답 포함) */
  onTurnStart?(round: number): void;
  onToolStart?(call: AiToolCall): void;
  onToolEnd?(call: AiToolCall, result: AiToolResult): void;
}

export interface AgentLoopOptions {
  provider: AiProvider;
  session: ChatSession;
  tools: readonly AiToolDef[];
  system: string;
  model: string;
  effort: 'low' | 'medium' | 'high';
  /**
   * 도구 호출 하나를 실행한다. 던지지 않고 결과(오류 포함)를 돌려줘야 한다.
   * 쓰기 도구는 사용자 승인을 기다리므로 비동기일 수 있다 — `signal` 이 끊기면 기다림을 끝내야 한다.
   */
  executeTool(call: AiToolCall, boundaryId: string, signal: AbortSignal): AiToolResult | Promise<AiToolResult>;
  /** 회차마다 새 문서 경계 id 를 만든다. */
  createBoundaryId(): string;
  maxToolRounds?: number;
}

/**
 * 도구 결과 뒤에 붙일 응답 언어 상기. 결과는 영어 JSON 이라 여러 회차를 지나면 모델이 도구 사이
 * 메모를 영어로 쓴다(실측: 한국어 요청에 「Now formatting: title and headers.」). 시스템 프롬프트는
 * 캐시 때문에 바꿀 수 없으니 새로 붙는 결과 칸에 싣는다. 상기 문구 자체도 한국어로 쓴다 — 영어 상기문은
 * 실측 두 번 중 한 번 영어 메모가 남았다. 한글이 없으면 붙이지 않는다.
 */
export function replyLanguageNote(userText: string): string | undefined {
  return /[\u1100-\u11ff\u3130-\u318f\uac00-\ud7a3]/.test(userText)
    ? '(알림: 도구 사이에 쓰는 짧은 메모와 최종 답 모두 한국어로 쓰세요. 영어로 쓰지 마세요.)'
    : undefined;
}

export async function runAgent(
  options: AgentLoopOptions,
  userText: string,
  signal: AbortSignal,
  callbacks: AgentCallbacks = {},
): Promise<AgentRunResult> {
  const { provider, session } = options;
  const maxRounds = options.maxToolRounds ?? MAX_TOOL_ROUNDS;
  const boundaryId = options.createBoundaryId();
  const note = replyLanguageNote(userText);
  let usage = EMPTY_USAGE;
  let toolRounds = 0;
  let lastText = '';

  session.append({ role: 'user', kind: 'text', text: userText });

  const finish = (status: Exclude<AgentRunStatus, 'done'>, error?: AiProviderError): AgentRunResult => {
    const marker = status === 'error' ? INTERRUPTED_MARKERS.error : INTERRUPTED_MARKERS[status];
    if (session.lastRole === 'user') session.append({ role: 'assistant', kind: 'text', text: marker });
    return { status, text: lastText, usage, toolRounds, ...(error ? { error } : {}) };
  };

  for (let round = 0; ; round += 1) {
    if (signal.aborted) return finish('aborted');
    callbacks.onTurnStart?.(round);

    let turn;
    try {
      turn = await options.provider.streamTurn(
        {
          model: options.model,
          effort: options.effort,
          system: options.system,
          tools: options.tools,
          messages: session.messages,
        },
        signal,
        (event) => {
          if (event.type === 'text-delta') callbacks.onText?.(event.text);
        },
      );
    } catch (error) {
      const failure = error instanceof AiProviderError
        ? error
        : new AiProviderError({ reason: signal.aborted ? 'aborted' : 'unknown' });
      return failure.reason === 'aborted' ? finish('aborted') : finish('error', failure);
    }
    usage = addUsage(usage, turn.usage);
    lastText = turn.text;

    switch (turn.stop) {
      case 'refusal':
        // 거절 직전까지 흘린 부분 응답은 이력에 넣지 않는다(완성된 답이 아니다).
        return finish('refusal');
      case 'max_tokens':
        // 잘린 응답의 도구 호출은 입력이 잘렸을 수 있어 실행하지 않는다. 결과 없는
        // tool_use 를 이력에 남기면 다음 요청이 거절되므로 그때는 표시로 대신한다.
        if (turn.toolCalls.length === 0 && Array.isArray(turn.raw) && turn.raw.length > 0) {
          session.append({ role: 'assistant', kind: 'raw', raw: turn.raw });
        }
        return finish('max_tokens');
      case 'pause':
        // pause_turn 은 서버 도구가 반복 상한에 걸렸을 때만 온다. P0 는 서버 도구를 쓰지 않으므로
        // 오면 예상 밖 상태로 보고 끝낸다(부분 응답은 넣지 않는다).
        return finish('error', new AiProviderError({ reason: 'unknown' }));
      case 'tool_use':
        break;
      default: {
        if (Array.isArray(turn.raw) && turn.raw.length > 0) {
          session.append({ role: 'assistant', kind: 'raw', raw: turn.raw });
          return { status: 'done', text: lastText, usage, toolRounds };
        }
        return finish('error', new AiProviderError({ reason: 'unknown' }));
      }
    }

    if (turn.toolCalls.length === 0) {
      return finish('error', new AiProviderError({ reason: 'unknown' }));
    }
    if (toolRounds >= maxRounds) return finish('round_limit');

    session.append({ role: 'assistant', kind: 'raw', raw: turn.raw });
    toolRounds += 1;
    // tool_use 마다 결과가 하나씩 있어야 한다 — 중단 신호가 와도 모두 채운 뒤 끝낸다.
    // 차례대로 실행한다: 쓰기 제안은 사용자가 하나씩 보고 고른다.
    const results: AiToolResult[] = [];
    for (const call of turn.toolCalls) {
      if (signal.aborted) {
        results.push({ id: call.id, content: 'Not run: the user stopped the response.', isError: true });
        continue;
      }
      callbacks.onToolStart?.(call);
      const result = await options.executeTool(call, boundaryId, signal);
      callbacks.onToolEnd?.(call, result);
      results.push(result);
    }
    session.append({ role: 'user', kind: 'tool_results', results, ...(note ? { note } : {}) });
  }
}
