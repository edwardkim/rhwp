/**
 * 제공자별 모델 목록. 설정 대화상자와 provider adapter 가 같은 표를 본다.
 *
 * 모델 id 는 API 에 그대로 보내는 문자열이다 — 날짜 접미사를 붙이지 않는다.
 */
export type AiProviderId = 'claude';

export type AiEffort = 'low' | 'medium' | 'high';

export interface AiModelOption {
  id: string;
  /** 화면 표시용 이름. 제품명이라 번역하지 않는다. */
  label: string;
  /**
   * 안전 분류기가 거절하면 서버가 다른 모델로 다시 돌리는 `fallbacks: "default"` 를 받는가.
   * Haiku 5.5 는 서버 측 대체가 없어 보내면 거절된다.
   */
  serverFallback: boolean;
}

export const AI_PROVIDERS: readonly AiProviderId[] = ['claude'];

export const CLAUDE_MODELS: readonly AiModelOption[] = [
  { id: 'claude-opus-5-5', label: 'Claude Opus 5.5', serverFallback: true },
  { id: 'claude-sonnet-5-5', label: 'Claude Sonnet 5.5', serverFallback: true },
  { id: 'claude-haiku-5-5', label: 'Claude Haiku 5.5', serverFallback: false },
];

export const AI_EFFORTS: readonly AiEffort[] = ['low', 'medium', 'high'];

export function modelsOf(provider: AiProviderId): readonly AiModelOption[] {
  switch (provider) {
    case 'claude': return CLAUDE_MODELS;
  }
}
