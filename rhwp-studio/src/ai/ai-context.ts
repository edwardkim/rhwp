/**
 * AI 도우미의 공유 상태 — 설정·키 보관과 변경 알림.
 *
 * 패널과 대화상자가 같은 인스턴스를 본다. 키 원문은 `KeyStore` 밖으로 꺼내 보관하지 않는다.
 */
import {
  hasConsent,
  loadAiSettings,
  saveAiSettings,
  type AiSettings,
} from './settings/ai-settings.ts';
import { KeyStore, openIndexedDbKeyBackend, type KeyStatus } from './settings/key-store.ts';

export class AiContext {
  private current: AiSettings;
  private readonly listeners = new Set<() => void>();
  readonly keyStore: KeyStore;
  /** 설정을 저장소에 남기지 못하는 환경인가 (이번 실행 동안만 유지) */
  settingsVolatile = false;

  constructor(settings: AiSettings, keyStore: KeyStore) {
    this.current = settings;
    this.keyStore = keyStore;
  }

  static async create(): Promise<AiContext> {
    const backend = await openIndexedDbKeyBackend();
    return new AiContext(loadAiSettings(), new KeyStore(backend));
  }

  get settings(): AiSettings {
    return this.current;
  }

  updateSettings(patch: Partial<AiSettings>): void {
    this.current = { ...this.current, ...patch };
    this.settingsVolatile = !saveAiSettings(this.current);
    this.emit();
  }

  keyStatus(): Promise<KeyStatus> {
    return this.keyStore.status(this.current.provider);
  }

  hasConsent(): boolean {
    return hasConsent(this.current);
  }

  /** 키를 바꾸거나 지운 뒤 화면을 다시 그리게 한다. */
  emit(): void {
    for (const listener of [...this.listeners]) {
      try { listener(); } catch (error) { console.error('[AiContext] 변경 알림 실패', error); }
    }
  }

  onChange(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => { this.listeners.delete(listener); };
  }
}
