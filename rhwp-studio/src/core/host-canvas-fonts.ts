import { fontProgramOwnsFamily, intrinsicCanvasSourceFamily } from './canvas-intrinsic-font.mjs';
import { getHostFontState, localFontFaceKey, loadRendererLocalFont,
  onHostFontsChanged, resolveRendererLocalFont, type LocalFontRecord } from './local-fonts.ts';
import { parseCssFontFamilyList, formatCssFontFamilyList } from './font-substitution.ts';
import { ordinaryTextFontFaceData, type OrdinaryTextFontSource } from './ordinary-text-font.ts';

interface CanvasHostFace {
  alias: string;
  font: FontFace;
  bytes: ArrayBuffer;
  intrinsicAlias: string;
  intrinsicFont: FontFace | null;
  ownership: Map<string, boolean>;
}
type CanvasFontContext = Pick<CanvasRenderingContext2D, 'font' | 'measureText'>;
interface CanvasFontSourceOwner extends OrdinaryTextFontSource {
  registerTextFontGlyphCoverage?: (family: string, bytes: Uint8Array, faceIndex: number) => unknown;
  registerHftOutlineFont?: (family: string, bytes: Uint8Array, faceIndex: number) => unknown;
}
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
    for (const { font, intrinsicFont } of this.faces.values()) {
      this.fontSet?.delete(font);
      if (intrinsicFont) this.fontSet?.delete(intrinsicFont);
    }
    this.fontSet = null;
    this.faces.clear(); this.aliases.clear(); this.pending.clear(); this.failures.clear();
    this.generation = -1;
  }

  diagnostics(): { loaded: number; pending: number; failed: number } {
    return { loaded: this.faces.size, pending: this.pending.size, failed: this.failures.size };
  }

  isPrepared(record: LocalFontRecord): boolean {
    return this.generation === getHostFontState().generation && this.faces.has(localFontFaceKey(record));
  }

  async prepare(records: readonly LocalFontRecord[], sourceOwner: CanvasFontSourceOwner | null = null,
    isCurrent: () => boolean = () => true): Promise<void> {
    const generation = getHostFontState().generation;
    if (this.generation !== generation) {
      this.reset(); this.generation = generation;
      this.off = onHostFontsChanged(() => this.reset());
    }
    if (typeof FontFace === 'undefined' || typeof document === 'undefined') return;
    this.fontSet = document.fonts;
    const epoch = this.epoch;
    for (const record of records) {
      if (epoch !== this.epoch || !isCurrent()) return;
      const key = localFontFaceKey(record);
      if (!record.hostReference || this.failures.has(key)) continue;
      const register = (loaded: CanvasHostFace) => {
        if (!isCurrent()) return;
        for (const family of new Set([record.family, ...record.aliases])) {
          sourceOwner?.registerTextFontGlyphCoverage?.(family, new Uint8Array(loaded.bytes), 0);
          if (Number(record.hostReference!.face.weight ?? 400) === 400
            && (record.hostReference!.face.slant ?? 'normal') === 'normal') {
            sourceOwner?.registerHftOutlineFont?.(family, new Uint8Array(loaded.bytes), 0);
          }
        }
      };
      const loaded = this.faces.get(key);
      if (loaded) { register(loaded); continue; }
      let pending = this.pending.get(key);
      if (!pending) {
        const stale = () => epoch !== this.epoch || generation !== getHostFontState().generation;
        pending = (async () => {
          try {
            const data = await loadRendererLocalFont(record);
            if (stale()) return;
            const bytes = data ? ordinaryTextFontFaceData(data.bytes, data.faceIndex ?? 0, sourceOwner) : null;
            if (!bytes) throw new Error('Host font has no usable face');
            const alias = `__rhwp_host_face_${this.id}_${++this.serial}`;
            const face = record.hostReference!.face;
            const font = new FontFace(alias, bytes, { weight: String(face.weight ?? 400), style: face.slant ?? 'normal' });
            await font.load();
            if (stale()) return;
            let intrinsicFont: FontFace | null = null;
            const needsIntrinsic = Number(face.weight ?? 400) < 600
              && [record.family, ...record.aliases].some(name => fontProgramOwnsFamily(bytes, name));
            const intrinsicAlias = needsIntrinsic ? `${alias}__intrinsic` : alias;
            if (intrinsicAlias !== alias) {
              intrinsicFont = new FontFace(intrinsicAlias, bytes, { weight: '700', style: face.slant ?? 'normal' });
              await intrinsicFont.load();
              if (stale()) return;
              this.fontSet!.add(intrinsicFont);
            }
            this.fontSet!.add(font);
            this.faces.set(key, { alias, font, bytes, intrinsicAlias, intrinsicFont, ownership: new Map() });
            this.aliases.add(intrinsicAlias);
            this.aliases.add(alias);
          } catch {
            if (!stale()) this.failures.add(key);
          }
        })().finally(() => { if (this.pending.get(key) === pending) this.pending.delete(key); });
        this.pending.set(key, pending);
      }
      await pending;
      const prepared = this.faces.get(key);
      if (prepared && epoch === this.epoch) register(prepared);
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
    const intrinsic = families[0] ? intrinsicCanvasSourceFamily(families[0]) : null;
    const prepared: string[] = [];
    for (const [index, family] of families.entries()) {
      const requested = index === 0 ? intrinsic ?? family : family;
      const record = resolveRendererLocalFont(requested, { weight: index === 0 && intrinsic ? 400 : weight, slant });
      const loaded = record?.hostReference ? this.faces.get(localFontFaceKey(record)) : null;
      if (!loaded) continue;
      // 실제 프로그램 소유 검증은 원 face에 적용하고 뒤 후보도 세션 별칭으로 공급한다.
      if (index === 0 && intrinsic !== null && !loaded.ownership.has(intrinsic)) {
        loaded.ownership.set(intrinsic, fontProgramOwnsFamily(loaded.bytes, intrinsic));
      }
      const ownsIntrinsic = index === 0 && intrinsic !== null && loaded.ownership.get(intrinsic) === true;
      if (index === 0 && intrinsic !== null && !ownsIntrinsic) return null;
      const selectedAlias = ownsIntrinsic ? loaded.intrinsicAlias : loaded.alias;
      if (!prepared.includes(selectedAlias)) prepared.push(selectedAlias);
    }
    return prepared.length ? `${match[1]}${match[2]} ${formatCssFontFamilyList([...prepared, ...families])}` : null;
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
