/**
 * 대화 이력 — **덧붙이기만** 한다.
 *
 * 지난 메시지를 고치거나 지우지 않는다. 응답의 thinking 블록은 그 대화 이력에 묶여 있어,
 * 지난 turn 을 고치면 API 가 요청을 거절한다. 그래서 각 칸을 얼려(freeze) 넣고, 바꿀 수 있는
 * 길은 새 칸을 덧붙이는 `append` 뿐이다. 새 대화는 새 인스턴스로 시작한다.
 */
import type { AiMessage } from '../provider/types.ts';

function deepFreeze<T>(value: T): T {
  if (value && typeof value === 'object' && !Object.isFrozen(value)) {
    Object.freeze(value);
    for (const child of Object.values(value as Record<string, unknown>)) deepFreeze(child);
  }
  return value;
}

export class ChatSession {
  private items: AiMessage[] = [];

  get messages(): readonly AiMessage[] {
    return this.items;
  }

  get lastRole(): 'user' | 'assistant' | null {
    return this.items.at(-1)?.role ?? null;
  }

  append(message: AiMessage): void {
    const last = this.items.at(-1);
    // user/assistant 교대를 지킨다. 어긋난 이력은 다음 요청에서 거절된다.
    if (last ? last.role === message.role : message.role !== 'user') {
      throw new Error(`대화 이력 교대가 어긋납니다: ${last?.role ?? '(처음)'} 뒤에 ${message.role}`);
    }
    // 배열도 새로 만든다 — 이전에 건넨 `messages` 스냅샷이 뒤에서 바뀌지 않게.
    this.items = [...this.items, deepFreeze(structuredClone(message))];
  }
}
