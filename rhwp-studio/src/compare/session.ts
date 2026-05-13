import type { EventBus } from '@/core/event-bus';
import type { CompareSession, DiffItem } from './types';

export class CompareSessionStore {
  private session: CompareSession | null = null;

  constructor(private readonly eventBus: EventBus) {}

  set(session: CompareSession): void {
    this.session = session;
    this.eventBus.emit('compare:session-changed', session);
  }

  clear(): void {
    this.session = null;
    this.eventBus.emit('compare:session-cleared');
  }

  get(): CompareSession | null {
    return this.session;
  }

  nextDiff(): DiffItem | null {
    if (!this.session || this.session.diffItems.length === 0) return null;
    const next = Math.min(this.session.currentDiffIndex + 1, this.session.diffItems.length - 1);
    this.session.currentDiffIndex = next;
    const item = this.session.diffItems[next];
    this.eventBus.emit('compare:navigate-diff', item, next, this.session.diffItems.length);
    return item;
  }

  prevDiff(): DiffItem | null {
    if (!this.session || this.session.diffItems.length === 0) return null;
    const prev = Math.max(this.session.currentDiffIndex - 1, 0);
    this.session.currentDiffIndex = prev;
    const item = this.session.diffItems[prev];
    this.eventBus.emit('compare:navigate-diff', item, prev, this.session.diffItems.length);
    return item;
  }

  gotoDiff(index: number): DiffItem | null {
    if (!this.session || this.session.diffItems.length === 0) return null;
    if (index < 0 || index >= this.session.diffItems.length) return null;
    this.session.currentDiffIndex = index;
    const item = this.session.diffItems[index];
    this.eventBus.emit('compare:navigate-diff', item, index, this.session.diffItems.length);
    return item;
  }
}
