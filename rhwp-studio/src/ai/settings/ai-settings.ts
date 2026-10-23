/**
 * AI 설정 — 비밀이 아닌 값만. 키는 `key-store.ts` 만 다룬다.
 *
 * studio 의 `rhwp-settings` 와 따로 `rhwp-ai-settings` 키에 둔다. AI 청크를 빼고 빌드한
 * studio 가 이 형태를 알 필요가 없게 하기 위해서다.
 */
import {
  AI_EFFORTS,
  AI_PROVIDERS,
  modelsOf,
  type AiEffort,
  type AiProviderId,
} from '../provider/models.ts';

export const AI_SETTINGS_STORAGE_KEY = 'rhwp-ai-settings';

export interface AiSettings {
  provider: AiProviderId;
  model: string;
  effort: AiEffort;
  /** 키를 다음 실행에도 쓰도록 브라우저에 저장하는가 */
  rememberKey: boolean;
  /** AI 가 제안한 문서 변경을 확인 없이 바로 적용하는가. 기본은 끔 — 변경마다 사용자가 고른다. */
  autoApply: boolean;
  /** 문서 내용 전송 동의 — 동의한 시각(ISO)과 그때의 제공자. 제공자가 바뀌면 다시 묻는다. */
  consent: { at: string; provider: AiProviderId } | null;
}

export const DEFAULT_AI_SETTINGS: AiSettings = {
  provider: 'claude',
  model: 'claude-opus-5-5',
  effort: 'medium',
  rememberKey: false,
  autoApply: false,
  consent: null,
};

type StorageLike = Pick<Storage, 'getItem' | 'setItem'>;

function defaultStorage(): StorageLike | null {
  try { return globalThis.localStorage ?? null; } catch { return null; }
}

/** 저장된 값이 어떤 모양이든 유효한 설정으로 돌려준다. 모르는 값은 기본값으로 물러난다. */
export function normalizeAiSettings(raw: unknown): AiSettings {
  const value = (raw && typeof raw === 'object' ? raw : {}) as Record<string, unknown>;
  const provider = AI_PROVIDERS.includes(value.provider as AiProviderId)
    ? value.provider as AiProviderId
    : DEFAULT_AI_SETTINGS.provider;
  const models = modelsOf(provider);
  const model = models.some((m) => m.id === value.model)
    ? value.model as string
    : models[0].id;
  const effort = AI_EFFORTS.includes(value.effort as AiEffort)
    ? value.effort as AiEffort
    : DEFAULT_AI_SETTINGS.effort;
  const rememberKey = value.rememberKey === true;
  const autoApply = value.autoApply === true;
  const rawConsent = value.consent as Record<string, unknown> | null | undefined;
  const consent = rawConsent && typeof rawConsent.at === 'string'
      && !Number.isNaN(Date.parse(rawConsent.at))
      && AI_PROVIDERS.includes(rawConsent.provider as AiProviderId)
    ? { at: rawConsent.at, provider: rawConsent.provider as AiProviderId }
    : null;
  return { provider, model, effort, rememberKey, autoApply, consent };
}

export function loadAiSettings(storage: StorageLike | null = defaultStorage()): AiSettings {
  if (!storage) return { ...DEFAULT_AI_SETTINGS };
  try {
    const raw = storage.getItem(AI_SETTINGS_STORAGE_KEY);
    return normalizeAiSettings(raw === null ? null : JSON.parse(raw));
  } catch {
    return { ...DEFAULT_AI_SETTINGS };
  }
}

/** 저장 실패(사생활 보호 창 등)는 `false` 로 알린다 — 설정은 이번 실행 동안만 유지된다. */
export function saveAiSettings(
  settings: AiSettings,
  storage: StorageLike | null = defaultStorage(),
): boolean {
  if (!storage) return false;
  try {
    storage.setItem(AI_SETTINGS_STORAGE_KEY, JSON.stringify(normalizeAiSettings(settings)));
    return true;
  } catch {
    return false;
  }
}

/** 지금 제공자에 대한 전송 동의가 있는가. */
export function hasConsent(settings: AiSettings): boolean {
  return settings.consent?.provider === settings.provider;
}
