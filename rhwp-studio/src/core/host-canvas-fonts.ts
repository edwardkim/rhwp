import { getHostFontState, localFontFaceKey, loadRendererLocalFont,
  onHostFontsChanged, resolveRendererLocalFont, type LocalFontRecord } from './local-fonts.ts';
import { parseCssFontFamilyList, formatCssFontFamilyList } from './font-substitution.ts';
import { normalizeFontFaceData } from './sfnt-face.ts';

interface CanvasHostFace { alias: string; font: FontFace }
type CanvasFontContext = Pick<CanvasRenderingContext2D, 'font' | 'measureText'>;
let nextSession = 0;
let currentSession: HostCanvasFontSession | null = null;

/** Synchronous, nested scope: never hold an active document across an await. */
export function withHostCanvasFonts<T>(session: HostCanvasFontSession | null, operation: () => T): T {
  const previous = currentSession;
  currentSession = session;
  try { return operation(); } finally { currentSession = previous; }
}

/** Called only by the existing native Canvas font setter. No document name writes. */
export function scopedHostCanvasFont(cssFont: string): string | null {
  return currentSession?.resolve(cssFont) ?? null;
}

/** One owner per document bridge. FontFace objects are loaded before being added,
 * so document.fonts loadingdone cannot recursively restart their preparation. */
export class HostCanvasFontSession {
  private readonly id = ++nextSession;
  private serial = 0;
  private epoch = 0;
  private generation = -1;
  private off: (() => void) | null = null;
  private readonly faces = new Map<string, CanvasHostFace>();
  private readonly aliases = new Set<string>();
  private readonly pending = new Map<string, Promise<void>>();
  private readonly failures = new Set<string>();
  private fontSet: FontFaceSet | null = null;

  reset(): void {
    this.epoch += 1;
    this.off?.(); this.off = null;
    for (const { font } of this.faces.values()) this.fontSet?.delete(font);
    this.fontSet = null;
    this.faces.clear(); this.aliases.clear(); this.pending.clear(); this.failures.clear();
    this.generation = -1;
  }

  diagnostics(): { loaded: number; pending: number; failed: number } {
    return { loaded: this.faces.size, pending: this.pending.size, failed: this.failures.size };
  }

  async prepare(records: readonly LocalFontRecord[]): Promise<void> {
    const generation = getHostFontState().generation;
    if (this.generation !== generation) {
      this.reset(); this.generation = generation;
      this.off = onHostFontsChanged(() => this.reset());
    }
    if (typeof FontFace === 'undefined' || typeof document === 'undefined') return;
    this.fontSet = document.fonts;
    const epoch = this.epoch;
    for (const record of records) {
      if (epoch !== this.epoch) return;
      const key = localFontFaceKey(record);
      if (!record.hostReference || this.faces.has(key) || this.failures.has(key)) continue;
      let pending = this.pending.get(key);
      if (!pending) {
        const stale = () => epoch !== this.epoch || generation !== getHostFontState().generation;
        pending = (async () => {
          try {
            const data = await loadRendererLocalFont(record);
            if (stale()) return;
            const bytes = data ? normalizeFontFaceData(data.bytes, data.faceIndex ?? 0) : null;
            if (!bytes) throw new Error('Host font has no usable face');
            const alias = `__rhwp_host_face_${this.id}_${++this.serial}`;
            const face = record.hostReference!.face;
            const font = new FontFace(alias, bytes, { weight: String(face.weight ?? 400), style: face.slant ?? 'normal' });
            await font.load();
            if (stale()) return;
            this.fontSet!.add(font);
            this.faces.set(key, { alias, font });
            this.aliases.add(alias);
          } catch {
            if (!stale()) this.failures.add(key);
          }
        })().finally(() => { if (this.pending.get(key) === pending) this.pending.delete(key); });
        this.pending.set(key, pending);
      }
      await pending;
    }
  }

  resolve(cssFont: string): string | null {
    const state = getHostFontState();
    if (this.generation !== state.generation || !state.active) return null;
    // Engine paint and supplemental metric requests both use px descriptors.
    const match = /^(.*?)((?:\d+(?:\.\d+)?|\.\d+)px)\s+(.+)$/.exec(cssFont);
    if (!match) return null;
    const tokens = match[1].trim().split(/\s+/);
    const weightToken = tokens.find(token => /^\d+$/.test(token));
    const weight = weightToken ? Number(weightToken) : tokens.includes('bold') ? 700 : 400;
    const slant = tokens.includes('italic') ? 'italic' : tokens.includes('oblique') ? 'oblique' : 'normal';
    const families = parseCssFontFamilyList(match[3]);
    if (this.aliases.has(families[0])) return cssFont;
    const record = families[0] ? resolveRendererLocalFont(families[0], { weight, slant }) : null;
    const loaded = record?.hostReference ? this.faces.get(localFontFaceKey(record)) : null;
    return loaded ? `${match[1]}${match[2]} ${formatCssFontFamilyList([loaded.alias, ...families])}` : null;
  }

  /** The asynchronous metric session receives a scoped synchronous context. */
  measurementContext(context: CanvasFontContext): CanvasFontContext {
    const session = this;
    return {
      get font() { return context.font; },
      set font(value: string) { withHostCanvasFonts(session, () => { context.font = value; }); },
      measureText(text: string) { return withHostCanvasFonts(session, () => context.measureText(text)); },
    };
  }
}
