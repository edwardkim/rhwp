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

export interface AutosaveManagerOptions {
  exportBytes: () => Uint8Array;
  debounceMs?: number;
  minSaveIntervalMs?: number;
  now?: () => number;
  idFactory?: () => string;
  store?: AutosaveStoreLike;
  logger?: Pick<Console, 'debug' | 'warn'>;
}

interface CurrentDocument {
  draftId: string;
  fileName: string;
  sourceFormat: string;
}

const DEFAULT_DEBOUNCE_MS = 2_000;
const DEFAULT_MIN_SAVE_INTERVAL_MS = 10_000;

function reasonText(reason: unknown, fallback: string): string {
  return typeof reason === 'string' && reason.length > 0 ? reason : fallback;
}

export class AutosaveManager {
  private readonly exportBytes: () => Uint8Array;
  private readonly debounceMs: number;
  private readonly minSaveIntervalMs: number;
  private readonly now: () => number;
  private readonly idFactory: () => string;
  private readonly store: AutosaveStoreLike;
  private readonly logger: Pick<Console, 'debug' | 'warn'>;

  private current: CurrentDocument | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private lastSavedAt = 0;
  private saving = false;
  private pendingReason: string | null = null;

  constructor(options: AutosaveManagerOptions) {
    this.exportBytes = options.exportBytes;
    this.debounceMs = options.debounceMs ?? DEFAULT_DEBOUNCE_MS;
    this.minSaveIntervalMs = options.minSaveIntervalMs ?? DEFAULT_MIN_SAVE_INTERVAL_MS;
    this.now = options.now ?? (() => Date.now());
    this.idFactory = options.idFactory ?? createAutosaveDraftId;
    this.store = options.store ?? {
      saveDraft: saveAutosaveDraft,
      deleteDraft: deleteAutosaveDraft,
    };
    this.logger = options.logger ?? console;
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
    this.cancelTimer();
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

  schedule(reason = 'document-mutated'): void {
    if (!this.current) return;
    this.cancelTimer();
    const elapsed = this.lastSavedAt > 0 ? this.now() - this.lastSavedAt : Number.POSITIVE_INFINITY;
    const intervalDelay = Math.max(0, this.minSaveIntervalMs - elapsed);
    const delay = Math.max(this.debounceMs, intervalDelay);
    this.timer = setTimeout(() => {
      this.timer = null;
      void this.flushNow(reason);
    }, delay);
  }

  async flushNow(reason = 'manual'): Promise<void> {
    const current = this.current;
    if (!current) return;

    if (this.saving) {
      this.pendingReason = reason;
      return;
    }

    this.saving = true;
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
    } catch (error) {
      this.logger.warn('[autosave] draft save failed:', error);
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
    this.cancelTimer();
    this.pendingReason = null;
    this.lastSavedAt = 0;
    const draftId = this.current?.draftId;
    if (!draftId) return;
    await this.deleteDraft(draftId, reason);
  }

  dispose(): void {
    this.cancelTimer();
    this.pendingReason = null;
  }

  private cancelTimer(): void {
    if (!this.timer) return;
    clearTimeout(this.timer);
    this.timer = null;
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
