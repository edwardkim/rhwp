import type { DirtyStateChange } from '@/core/document-dirty-state';
import type { EventBus } from '@/core/event-bus';
import {
  createAutosaveDraftId,
  deleteAutosaveDraft,
  saveAutosaveDraft,
  type AutosaveDraft,
} from './autosave-store.ts';

export interface AutosaveDocumentMeta {
  fileName: string;
  sourceFormat: string;
  draftId?: string;
}

export interface AutosaveStoreLike {
  saveDraft(draft: AutosaveDraft): Promise<void>;
  deleteDraft(id: string): Promise<void>;
}

export interface AutosaveScheduleSettings {
  recoveryEnabled: boolean;
  recoveryIntervalMs: number;
  idleEnabled: boolean;
  idleDelayMs: number;
}

export type AutosaveStatus =
  | { state: 'saving'; reason: string }
  | { state: 'saved'; reason: string; byteLength: number }
  | { state: 'error'; reason: string; error: unknown };

export interface AutosaveManagerOptions {
  exportBytes: () => Uint8Array;
  debounceMs?: number;
  minSaveIntervalMs?: number;
  schedule?: Partial<AutosaveScheduleSettings>;
  now?: () => number;
  idFactory?: () => string;
  store?: AutosaveStoreLike;
  logger?: Pick<Console, 'debug' | 'warn'>;
  onStatus?: (status: AutosaveStatus) => void;
}

interface CurrentDocument {
  draftId: string;
  fileName: string;
  sourceFormat: string;
}

const DEFAULT_IDLE_DELAY_MS = 10_000;
const DEFAULT_RECOVERY_INTERVAL_MS = 10 * 60_000;

function reasonText(reason: unknown, fallback: string): string {
  return typeof reason === 'string' && reason.length > 0 ? reason : fallback;
}

export class AutosaveManager {
  private readonly exportBytes: () => Uint8Array;
  private readonly now: () => number;
  private readonly idFactory: () => string;
  private readonly store: AutosaveStoreLike;
  private readonly logger: Pick<Console, 'debug' | 'warn'>;
  private readonly onStatus?: (status: AutosaveStatus) => void;

  private current: CurrentDocument | null = null;
  private idleTimer: ReturnType<typeof setTimeout> | null = null;
  private recoveryTimer: ReturnType<typeof setTimeout> | null = null;
  private lastSavedAt = 0;
  private saving = false;
  private pendingReason: string | null = null;
  private scheduleSettings: AutosaveScheduleSettings;

  constructor(options: AutosaveManagerOptions) {
    this.exportBytes = options.exportBytes;
    this.scheduleSettings = normalizeSchedule({
      recoveryEnabled: true,
      recoveryIntervalMs: options.minSaveIntervalMs ?? DEFAULT_RECOVERY_INTERVAL_MS,
      idleEnabled: true,
      idleDelayMs: options.debounceMs ?? DEFAULT_IDLE_DELAY_MS,
      ...(options.schedule ?? {}),
    });
    this.now = options.now ?? (() => Date.now());
    this.idFactory = options.idFactory ?? createAutosaveDraftId;
    this.store = options.store ?? {
      saveDraft: saveAutosaveDraft,
      deleteDraft: deleteAutosaveDraft,
    };
    this.logger = options.logger ?? console;
    this.onStatus = options.onStatus;
  }

  connect(eventBus: EventBus): () => void {
    const offDirty = eventBus.on('document-dirty-changed', (payload) => {
      const change = payload as Partial<DirtyStateChange> | undefined;
      if (change?.dirty) {
        this.schedule(reasonText(change.reason, 'document-dirty'));
      } else {
        void this.discardCurrentDraft(reasonText(change?.reason, 'document-clean'));
      }
    });
    const offMutated = eventBus.on('document-mutated', (reason) => {
      this.schedule(reasonText(reason, 'document-mutated'));
    });
    const offChanged = eventBus.on('document-changed', (reason) => {
      this.schedule(reasonText(reason, 'document-changed'));
    });

    return () => {
      offDirty();
      offMutated();
      offChanged();
      this.dispose();
    };
  }

  async beginDocument(meta: AutosaveDocumentMeta, options: { discardPreviousDraft?: boolean } = {}): Promise<string> {
    const previousDraftId = this.current?.draftId ?? null;
    this.cancelTimers();
    this.pendingReason = null;
    this.lastSavedAt = 0;
    this.current = {
      draftId: meta.draftId ?? this.idFactory(),
      fileName: meta.fileName,
      sourceFormat: meta.sourceFormat,
    };

    if (options.discardPreviousDraft && previousDraftId && previousDraftId !== this.current.draftId) {
      await this.deleteDraft(previousDraftId, 'document-replaced');
    }
    return this.current.draftId;
  }

  getCurrentDraftId(): string | null {
    return this.current?.draftId ?? null;
  }

  updateSchedule(settings: Partial<AutosaveScheduleSettings>): void {
    const hadScheduledSave = Boolean(this.idleTimer || this.recoveryTimer || this.pendingReason);
    this.scheduleSettings = normalizeSchedule({
      ...this.scheduleSettings,
      ...settings,
    });
    if (!this.current) return;
    this.cancelTimers();
    if (hadScheduledSave && (this.scheduleSettings.recoveryEnabled || this.scheduleSettings.idleEnabled)) {
      this.schedule('autosave-settings-changed');
    }
  }

  schedule(reason = 'document-mutated'): void {
    if (!this.current) return;
    const settings = this.scheduleSettings;
    if (!settings.recoveryEnabled && !settings.idleEnabled) return;

    if (settings.idleEnabled) {
      this.cancelIdleTimer();
      this.idleTimer = setTimeout(() => {
        this.idleTimer = null;
        void this.flushNow(reason);
      }, settings.idleDelayMs);
    }

    if (settings.recoveryEnabled && !this.recoveryTimer) {
      const elapsed = this.lastSavedAt > 0 ? this.now() - this.lastSavedAt : 0;
      const delay = Math.max(0, settings.recoveryIntervalMs - elapsed);
      this.recoveryTimer = setTimeout(() => {
        this.recoveryTimer = null;
        void this.flushNow('recovery-interval');
      }, delay);
    }
  }

  async flushNow(reason = 'manual'): Promise<void> {
    const current = this.current;
    if (!current) return;

    if (this.saving) {
      this.pendingReason = reason;
      return;
    }

    this.saving = true;
    this.cancelTimers();
    this.onStatus?.({ state: 'saving', reason });
    try {
      const bytes = this.exportBytes();
      const savedAt = this.now();
      await this.store.saveDraft({
        id: current.draftId,
        fileName: current.fileName,
        sourceFormat: current.sourceFormat,
        savedAt,
        byteLength: bytes.byteLength,
        data: new Uint8Array(bytes),
        dirtyReason: reason,
      });
      this.lastSavedAt = savedAt;
      this.logger.debug?.(`[autosave] draft saved: ${current.fileName} (${bytes.byteLength} bytes)`);
      this.onStatus?.({ state: 'saved', reason, byteLength: bytes.byteLength });
    } catch (error) {
      this.logger.warn('[autosave] draft save failed:', error);
      this.onStatus?.({ state: 'error', reason, error });
    } finally {
      this.saving = false;
      const pending = this.pendingReason;
      this.pendingReason = null;
      if (pending && this.current) {
        this.schedule(pending);
      }
    }
  }

  async discardCurrentDraft(reason = 'discard'): Promise<void> {
    this.cancelTimers();
    this.pendingReason = null;
    this.lastSavedAt = 0;
    const draftId = this.current?.draftId;
    if (!draftId) return;
    await this.deleteDraft(draftId, reason);
  }

  dispose(): void {
    this.cancelTimers();
    this.pendingReason = null;
  }

  private cancelIdleTimer(): void {
    if (!this.idleTimer) return;
    clearTimeout(this.idleTimer);
    this.idleTimer = null;
  }

  private cancelRecoveryTimer(): void {
    if (!this.recoveryTimer) return;
    clearTimeout(this.recoveryTimer);
    this.recoveryTimer = null;
  }

  private cancelTimers(): void {
    this.cancelIdleTimer();
    this.cancelRecoveryTimer();
  }

  private async deleteDraft(id: string, reason: string): Promise<void> {
    try {
      await this.store.deleteDraft(id);
      this.logger.debug?.(`[autosave] draft deleted: ${id} (${reason})`);
    } catch (error) {
      this.logger.warn('[autosave] draft delete failed:', error);
    }
  }
}

function normalizeSchedule(settings: AutosaveScheduleSettings): AutosaveScheduleSettings {
  return {
    recoveryEnabled: settings.recoveryEnabled,
    recoveryIntervalMs: normalizeMs(settings.recoveryIntervalMs, DEFAULT_RECOVERY_INTERVAL_MS, 0),
    idleEnabled: settings.idleEnabled,
    idleDelayMs: normalizeMs(settings.idleDelayMs, DEFAULT_IDLE_DELAY_MS, 0),
  };
}

function normalizeMs(value: unknown, fallback: number, min: number): number {
  const number = typeof value === 'number' ? value : Number(value);
  if (!Number.isFinite(number)) return fallback;
  return Math.max(min, Math.round(number));
}
