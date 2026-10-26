/**
 * 제공자 id → adapter. 각 adapter 는 처음 쓸 때 불러온다 — 패널만 열어 보는 사람에게
 * SDK·adapter 코드를 내려보내지 않고, 고른 제공자의 것만 받는다.
 */
import type { AiProviderId } from './models.ts';
import type { AiProvider, AiTestResult } from './types.ts';

export async function createProvider(provider: AiProviderId, apiKey: string): Promise<AiProvider> {
  switch (provider) {
    case 'claude': {
      const { ClaudeProvider } = await import('./claude.ts');
      return new ClaudeProvider(apiKey);
    }
    case 'openai': {
      const { OpenAiProvider } = await import('./openai.ts');
      return new OpenAiProvider(apiKey);
    }
    case 'gemini': {
      const { GeminiProvider } = await import('./gemini.ts');
      return new GeminiProvider(apiKey);
    }
  }
}

/** 키와 모델 접근을 확인한다. 세 제공자 모두 모델 정보 조회라 토큰을 쓰지 않는다. */
export async function testConnection(
  provider: AiProviderId,
  apiKey: string,
  model: string,
  signal?: AbortSignal,
): Promise<AiTestResult> {
  switch (provider) {
    case 'claude': {
      const { testClaudeConnection } = await import('./claude.ts');
      return testClaudeConnection(apiKey, model, signal);
    }
    case 'openai': {
      const { testOpenAiConnection } = await import('./openai.ts');
      return testOpenAiConnection(apiKey, model, signal);
    }
    case 'gemini': {
      const { testGeminiConnection } = await import('./gemini.ts');
      return testGeminiConnection(apiKey, model, signal);
    }
  }
}
