/**
 * 제공자별 모델 목록. 설정 대화상자와 provider adapter 가 같은 표를 본다.
 *
 * 모델 id 는 API 에 그대로 보내는 문자열이다 — 날짜 접미사를 붙이지 않는다.
 */
export type AiProviderId = 'claude' | 'openai' | 'gemini';

export type AiEffort = 'low' | 'medium' | 'high';

export interface AiModelOption {
  id: string;
  /** 화면 표시용 이름. 제품명이라 번역하지 않는다. */
  label: string;
  /**
   * 안전 분류기가 거절하면 서버가 다른 모델로 다시 돌리는 `fallbacks: "default"` 를 받는가(Claude 전용).
   * Haiku 5.5 는 서버 측 대체가 없어 보내면 거절된다. 다른 제공자는 쓰지 않는다.
   */
  serverFallback: boolean;
}

export const AI_PROVIDERS: readonly AiProviderId[] = ['claude', 'openai', 'gemini'];

export const CLAUDE_MODELS: readonly AiModelOption[] = [
  { id: 'claude-opus-5-5', label: 'Claude Opus 5.5', serverFallback: true },
  { id: 'claude-sonnet-5-5', label: 'Claude Sonnet 5.5', serverFallback: true },
  { id: 'claude-haiku-5-5', label: 'Claude Haiku 5.5', serverFallback: false },
];

/**
 * OpenAI Responses API 모델. 세 모델 모두 `reasoning.effort` low·medium·high 를 받는다
 * (developers.openai.com/api/docs/models, 2026-10 확인). 맨 앞이 제공자를 고를 때의 기본값이다.
 */
export const OPENAI_MODELS: readonly AiModelOption[] = [
  { id: 'gpt-6-astra', label: 'GPT-6 Astra', serverFallback: false },
  { id: 'gpt-6.1-sol', label: 'GPT-6.1 Sol', serverFallback: false },
  { id: 'gpt-6-luna', label: 'GPT-6 Luna', serverFallback: false },
];

/**
 * Gemini API 안정판 모델. 세 모델 모두 `thinkingLevel` low·medium·high 를 받는다
 * (ai.google.dev/gemini-api/docs/models·thinking, 2026-10 확인).
 */
export const GEMINI_MODELS: readonly AiModelOption[] = [
  { id: 'gemini-4-argon', label: 'Gemini 4 Argon', serverFallback: false },
  { id: 'gemini-3.8-flash', label: 'Gemini 3.8 Flash', serverFallback: false },
  { id: 'gemini-3.5-flash-lite', label: 'Gemini 3.5 Flash-Lite', serverFallback: false },
];

export const AI_EFFORTS: readonly AiEffort[] = ['low', 'medium', 'high'];

export function modelsOf(provider: AiProviderId): readonly AiModelOption[] {
  switch (provider) {
    case 'claude': return CLAUDE_MODELS;
    case 'openai': return OPENAI_MODELS;
    case 'gemini': return GEMINI_MODELS;
  }
}
