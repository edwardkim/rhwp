/** Native resources remain owned by the caller until set() succeeds. */
export class BoundedNativeCache<T> {
  private readonly entries = new Map<string, { value: T; bytes: number }>();
  bytes = 0;
  hits = 0;
  misses = 0;
  evictions = 0;
  readonly limit: number;
  readonly byteLimit: number;
  private readonly disposeValue: (value: T) => void;

  constructor(limit: number, byteLimit: number, disposeValue: (value: T) => void) {
    this.limit = limit;
    this.byteLimit = byteLimit;
    this.disposeValue = disposeValue;
  }

  get size(): number { return this.entries.size; }

  get(key: string): T | undefined {
    const entry = this.entries.get(key);
    if (!entry) { this.misses++; return undefined; }
    this.entries.delete(key);
    this.entries.set(key, entry);
    this.hits++;
    return entry.value;
  }

  set(key: string, value: T, bytes: number): boolean {
    if (!Number.isSafeInteger(bytes) || bytes < 0 || bytes > this.byteLimit || this.limit < 1) return false;
    this.delete(key);
    while (this.size >= this.limit || this.bytes + bytes > this.byteLimit) {
      const oldest = this.entries.keys().next().value;
      if (oldest === undefined) break;
      this.delete(oldest);
      this.evictions++;
    }
    this.entries.set(key, { value, bytes });
    this.bytes += bytes;
    return true;
  }

  delete(key: string): void {
    const entry = this.entries.get(key);
    if (!entry) return;
    this.entries.delete(key);
    this.bytes -= entry.bytes;
    this.disposeValue(entry.value);
  }

  deleteWhere(predicate: (key: string, value: T) => boolean): void {
    for (const [key, entry] of this.entries) if (predicate(key, entry.value)) this.delete(key);
  }

  clear(resetCounters = false): void {
    for (const key of this.entries.keys()) this.delete(key);
    if (resetCounters) { this.hits = 0; this.misses = 0; this.evictions = 0; }
  }
}
