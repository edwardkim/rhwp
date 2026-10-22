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
}

export const AI_PROVIDERS: readonly AiProviderId[] = ['claude'];

export const CLAUDE_MODELS: readonly AiModelOption[] = [
  { id: 'claude-opus-5-5', label: 'Claude Opus 5.5' },
  { id: 'claude-sonnet-5-5', label: 'Claude Sonnet 5.5' },
  { id: 'claude-haiku-5-5', label: 'Claude Haiku 5.5' },
];

export const AI_EFFORTS: readonly AiEffort[] = ['low', 'medium', 'high'];

export function modelsOf(provider: AiProviderId): readonly AiModelOption[] {
  switch (provider) {
    case 'claude': return CLAUDE_MODELS;
  }
}
