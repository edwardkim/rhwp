/** Host-owned metadata and bytes. No paths, browser permissions or persistence. */
export interface HostFontFace {
  id: string;
  family: string;
  fullName: string;
  postscriptName: string;
  style: string;
  aliases?: readonly string[];
  weight?: number;
  slant?: 'normal' | 'italic' | 'oblique';
}

export interface HostFontSnapshot {
  /** Change this token whenever metadata, selected faces or bytes change. */
  revision: string;
  faces: readonly HostFontFace[];
}

export interface HostFontData {
  bytes: ArrayBuffer;
  /** Zero for standalone SFNT; required to select a nonzero TTC face. */
  faceIndex?: number;
}

export interface HostFontProvider {
  getSnapshot(signal: AbortSignal): Promise<HostFontSnapshot>;
  readFace(id: string, revision: string, signal: AbortSignal): Promise<HostFontData>;
  /** Notify after publishing the new snapshot. Return a subscription disposer. */
  subscribe(onChange: () => void): () => void;
}

export interface HostFontReference {
  readonly key: string;
  readonly face: Readonly<HostFontFace>;
  readonly revision: string;
  readonly generation: number;
}

const MAX_FACES = 25_000;
const MAX_FONT_BYTES = 64 * 1024 * 1024;

function validName(value: unknown, allowEmpty = false): value is string {
  return typeof value === 'string' && value.length <= 1024 && (allowEmpty || value.trim().length > 0);
}

function copySnapshot(value: HostFontSnapshot): HostFontSnapshot {
  if (!value || !validName(value.revision) || !Array.isArray(value.faces) || value.faces.length > MAX_FACES) {
    throw new Error('Invalid host font snapshot');
  }
  const ids = new Set<string>();
  const faces = value.faces.map(face => {
    if (!face || !validName(face.id) || ids.has(face.id)
      || !validName(face.family) || !validName(face.fullName) || !validName(face.postscriptName, true)
      || !validName(face.style, true)
      || (face.weight !== undefined && (!Number.isFinite(face.weight) || face.weight < 1 || face.weight > 1000))
      || (face.slant !== undefined && !['normal', 'italic', 'oblique'].includes(face.slant))
      || (face.aliases !== undefined && (!Array.isArray(face.aliases) || face.aliases.length > 32
        || !face.aliases.every((alias: unknown) => validName(alias))))) {
      throw new Error('Invalid host font face');
    }
    ids.add(face.id);
    // Copy only the public contract; never keep accidental host paths or authority tokens.
    return Object.freeze({
      id: face.id, family: face.family, fullName: face.fullName,
      postscriptName: face.postscriptName, style: face.style,
      aliases: Object.freeze([...(face.aliases ?? [])]), weight: face.weight, slant: face.slant,
    });
  });
  return Object.freeze({ revision: value.revision, faces: Object.freeze(faces) });
}

/** One catalog per Studio realm; renderer objects are owned by their renderer. */
export class HostFontSource {
  private provider: HostFontProvider | null = null;
  private off: (() => void) | null = null;
  private controller = new AbortController();
  private epoch = 0;
  private connection = 0;
  private snapshot: HostFontSnapshot | null = null;
  private refs: readonly HostFontReference[] = [];
  private pendingSnapshot: Promise<void> | null = null;
  private readonly pendingBytes = new Map<string, Promise<HostFontData | null>>();
  private readonly listeners = new Set<() => void>();
  private error: string | null = null;

  get active(): boolean { return this.provider !== null; }
  get generation(): number { return this.epoch; }
  get lastError(): string | null { return this.error; }

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => { this.listeners.delete(listener); };
  }

  private notify(): void {
    for (const listener of this.listeners) listener();
  }

  private invalidate(): void {
    this.controller.abort();
    this.controller = new AbortController();
    this.epoch += 1;
    this.snapshot = null;
    this.refs = [];
    this.pendingSnapshot = null;
    this.pendingBytes.clear();
    this.error = null;
  }

  async setProvider(provider: HostFontProvider | null): Promise<void> {
    const connection = ++this.connection;
    const off = this.off;
    this.off = null;
    this.provider = provider;
    this.invalidate();
    // An unsubscribe callback cannot invalidate the replacement provider.
    try { off?.(); } catch (error) { console.warn('[HostFonts] Unsubscribe failed:', error); }
    if (provider) {
      try {
        const unsubscribe = provider.subscribe(() => {
          if (this.connection !== connection) return;
          this.invalidate();
          this.notify();
          void this.ready();
        });
        if (typeof unsubscribe !== 'function') throw new Error('Host font subscribe must return a disposer');
        this.off = unsubscribe;
      } catch (error) {
        this.error = String(error);
      }
    }
    this.notify();
    await this.ready();
  }

  ready(): Promise<void> {
    if (!this.provider || this.snapshot || this.error !== null) return Promise.resolve();
    if (this.pendingSnapshot) return this.pendingSnapshot;
    const provider = this.provider;
    const generation = this.epoch;
    const signal = this.controller.signal;
    const pending = Promise.resolve().then(() => provider.getSnapshot(signal)).then(value => {
      if (signal.aborted || generation !== this.epoch) return;
      this.snapshot = copySnapshot(value);
      this.refs = Object.freeze(this.snapshot.faces.map(face => Object.freeze({
        key: JSON.stringify(['host', this.epoch, this.snapshot!.revision, face.id]),
        face, revision: this.snapshot!.revision, generation: this.epoch,
      })));
      this.error = null;
    }).catch(error => {
      if (!signal.aborted && generation === this.epoch) this.error = String(error);
    }).finally(() => {
      if (this.pendingSnapshot === pending) {
        this.pendingSnapshot = null;
      }
    });
    this.pendingSnapshot = pending;
    return pending;
  }

  references(): readonly HostFontReference[] {
    return this.refs;
  }

  read(reference: HostFontReference): Promise<HostFontData | null> {
    const provider = this.provider;
    const signal = this.controller.signal;
    if (!provider || reference.generation !== this.epoch || reference.revision !== this.snapshot?.revision
      || !this.snapshot.faces.some(face => face === reference.face)) return Promise.resolve(null);
    const existing = this.pendingBytes.get(reference.key);
    if (existing) return existing;
    const pending = Promise.resolve().then(() => provider.readFace(reference.face.id, reference.revision, signal))
      .then(data => {
        if (signal.aborted || reference.generation !== this.epoch) return null;
        if (!(data?.bytes instanceof ArrayBuffer) || data.bytes.byteLength === 0
          || data.bytes.byteLength > MAX_FONT_BYTES || !Number.isSafeInteger(data.faceIndex ?? 0)
          || (data.faceIndex ?? 0) < 0) return null;
        // Consumers may transfer/detach buffers. Keep ownership at the API boundary explicit.
        return { bytes: data.bytes.slice(0), faceIndex: data.faceIndex ?? 0 };
      }, () => null).finally(() => {
        if (this.pendingBytes.get(reference.key) === pending) this.pendingBytes.delete(reference.key);
      });
    this.pendingBytes.set(reference.key, pending);
    return pending;
  }
}
