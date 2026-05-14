import type { EventBus } from './event-bus';

export interface DirtyStateChange {
  dirty: boolean;
  reason?: string;
}

/**
 * 저장되지 않은 문서 변경 상태를 관리한다.
 *
 * 브라우저는 beforeunload에서 앱 커스텀 모달을 허용하지 않으므로,
 * dirty 상태일 때만 브라우저 기본 이탈 확인창이 뜨도록 한다.
 */
export class DocumentDirtyState {
  private dirty = false;
  private beforeUnloadWindow: Window | null = null;
  private readonly eventBus: EventBus;
  private readonly beforeUnloadHandler = (event: BeforeUnloadEvent): string | void => {
    if (!this.dirty) return;
    event.preventDefault();
    event.returnValue = '';
    return '';
  };

  constructor(eventBus: EventBus) {
    this.eventBus = eventBus;
  }

  isDirty(): boolean {
    return this.dirty;
  }

  markDirty(reason?: string): void {
    this.setDirty(true, reason);
  }

  markClean(reason?: string): void {
    this.setDirty(false, reason);
  }

  installBeforeUnload(windowLike: Window): () => void {
    if (this.beforeUnloadWindow === windowLike) {
      return () => this.uninstallBeforeUnload(windowLike);
    }
    this.beforeUnloadWindow?.removeEventListener('beforeunload', this.beforeUnloadHandler);
    this.beforeUnloadWindow = windowLike;
    windowLike.addEventListener('beforeunload', this.beforeUnloadHandler);
    return () => this.uninstallBeforeUnload(windowLike);
  }

  private uninstallBeforeUnload(windowLike: Window): void {
    if (this.beforeUnloadWindow !== windowLike) return;
    windowLike.removeEventListener('beforeunload', this.beforeUnloadHandler);
    this.beforeUnloadWindow = null;
  }

  private setDirty(next: boolean, reason?: string): void {
    if (this.dirty === next) return;
    this.dirty = next;
    this.eventBus.emit('document-dirty-changed', { dirty: next, reason } satisfies DirtyStateChange);
  }
}
