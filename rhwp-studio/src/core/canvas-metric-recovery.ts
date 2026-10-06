// Rust's descriptor guard is deliberately narrower than a general render failure.
const DESCRIPTOR_MISMATCH = 'Canvas metric descriptor changed; prepare font metrics again';

export function isCanvasMetricDescriptorMismatch(error: unknown): boolean {
  const message = error instanceof Error ? error.message : error;
  return message === DESCRIPTOR_MISMATCH || message === `렌더링 오류: ${DESCRIPTOR_MISMATCH}`;
}

interface RecoveryAttempt {
  /** Document + externally changed font generation; recovery itself must not advance it. */
  key: string;
  isCurrent(): boolean;
  invalidate(): void;
  prepare(isCurrent: () => boolean): Promise<unknown>;
  repaint(): void;
  report(error: unknown): void;
}

/** One asynchronous attempt per document/font generation, shared by every page.
 * Keep the consumed key after success or failure: a repaint must not refill its own budget.
 */
export class CanvasMetricRecovery {
  private attemptedKey: string | null = null;

  async recover(error: unknown, attempt: RecoveryAttempt): Promise<boolean> {
    if (!isCanvasMetricDescriptorMismatch(error)
      || !attempt.isCurrent() || this.attemptedKey === attempt.key) return false;
    this.attemptedKey = attempt.key;
    // Let the failed paint release its page surfaces before touching layout or painting again.
    await Promise.resolve();
    if (!attempt.isCurrent()) return false;
    try {
      attempt.invalidate();
      await attempt.prepare(attempt.isCurrent);
      if (!attempt.isCurrent()) return false;
      attempt.repaint();
      return true;
    } catch (error) {
      if (attempt.isCurrent()) attempt.report(error);
      return false;
    }
  }
}
