/** Reuse a settled font policy only within one synchronous paint transaction.
 * A new paint or nested host-font scope gets a new map. No cached resolution
 * survives the synchronous callback or an await; later paints read fresh policy. */
export class CanvasFontResolutionScope {
  private values: Map<string, string> | null = null;
  private readonly resolveFont: (font: string) => string;

  constructor(resolveFont: (font: string) => string) { this.resolveFont = resolveFont; }

  resolve(font: string): string {
    const cached = this.values?.get(font);
    if (cached !== undefined) return cached;
    const resolved = this.resolveFont(font);
    this.values?.set(font, resolved);
    return resolved;
  }

  run<T>(operation: () => T): T {
    const previous = this.values;
    this.values = new Map();
    try { return operation(); }
    finally { this.values = previous; }
  }
}
