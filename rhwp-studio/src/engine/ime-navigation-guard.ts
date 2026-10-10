import type { NavigationKeyInput } from './navigation-keymap';

type KeyInput = NavigationKeyInput & { isComposing?: boolean; keyCode?: number; repeat?: boolean };

/** One physical IME navigation key can be forwarded again after compositionend. */
export class ImeNavigationGuard {
  private held: NavigationKeyInput | null = null;
  private replayed = false;

  queue(input: NavigationKeyInput): void {
    this.held = { ...input };
    this.replayed = false;
  }

  markReplayed(): void {
    // A keyup before compositionend must not suppress the next physical press.
    this.replayed = this.held !== null;
  }

  consume(input: KeyInput): boolean {
    if (!this.replayed || !this.held || input.isComposing || input.keyCode === 229) return false;
    const held = this.held;
    this.reset();
    return !input.repeat
      && (input.code || input.key) === (held.code || held.key)
      && input.shiftKey === held.shiftKey
      && input.ctrlKey === held.ctrlKey
      && input.metaKey === held.metaKey
      && input.altKey === held.altKey;
  }

  release(input: Pick<NavigationKeyInput, 'code' | 'key'>): void {
    if (this.held && (input.code || input.key) === (this.held.code || this.held.key)) this.reset();
  }

  reset(): void {
    this.held = null;
    this.replayed = false;
  }
}
