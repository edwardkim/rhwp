import { fontProgramOwnsFamily, intrinsicCanvasSourceFamily } from './canvas-intrinsic-font.mjs';
import { getHostFontState, localFontFaceKey, loadRendererLocalFont,
  onHostFontsChanged, resolveRendererLocalFontCandidates, type LocalFontRecord } from './local-fonts.ts';
import { parseCssFontFamilyList, formatCssFontFamilyList } from './font-substitution.ts';
import { ordinaryTextFontFaceData, type OrdinaryTextFontSource } from './ordinary-text-font.ts';
import { fontSymbolDescriptors, type FontSymbolInspector } from './font-symbol-coverage.ts';

interface CanvasHostFace {
  alias: string;
  font: FontFace;
  bytes: ArrayBuffer;
  intrinsicAlias: string;
  intrinsicFont: FontFace | null;
  ownership: Map<string, boolean>;
  source: { bytes: Uint8Array; faceIndex: number };
  unicodeRange: string;
}
type CanvasFontContext = Pick<CanvasRenderingContext2D, 'font' | 'measureText'>;
interface CanvasFontSourceOwner extends OrdinaryTextFontSource {
  clearTextFontFamily?: (family: string) => unknown;
  registerTextFontGlyphCoverage?: (family: string, bytes: Uint8Array, faceIndex: number) => unknown;
  registerHftOutlineFont?: (family: string, bytes: Uint8Array, faceIndex: number) => unknown;
  registerHftOutlineFontCandidates?: (family: string, programs: Uint8Array[]) => unknown;
  registerHftOutlineFontFaces?: (family: string, programs: Array<{ bytes: Uint8Array; faceIndex: number }>) => unknown;
  clearTextFontOutline?: (family: string) => unknown;
  registerPrintFontPrimary?: (family: string, bytes: Uint8Array, faceIndex: number, complete: boolean) => unknown;
  clearPrintFontPrimary?: (family: string) => unknown;
  getFontSymbolUnicodeRange?: (bytes: Uint8Array, faceIndex: number) => string;
}

function isCff(face: CanvasHostFace | null | undefined): boolean {
  return Boolean(face && face.bytes.byteLength >= 4
    && new DataView(face.bytes).getUint32(0) === 0x4f54544f);
}

/** 실제 descriptor 해소와 Print 프로그램 공급이 같은 후보 순서를 사용한다. */
function canvasHostOrder(faces: Array<CanvasHostFace | null | undefined>, sourceTtf: boolean) {
  if (faces.length > 1 && faces.some(face => !face)) return null;
  if (!sourceTtf && faces.length > 1 && !faces.some(isCff)) return null;
  return [...faces].sort((a, b) => sourceTtf
    ? Number(isCff(a)) - Number(isCff(b)) : Number(isCff(b)) - Number(isCff(a)));
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
  private readonly inspectSymbols?: FontSymbolInspector;

  constructor(inspectSymbols?: FontSymbolInspector) { this.inspectSymbols = inspectSymbols; }

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
  private scopedOutlines = false;

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
    this.scopedOutlines = false;
  }

  diagnostics(): { loaded: number; pending: number; failed: number } {
    return { loaded: this.faces.size, pending: this.pending.size, failed: this.failures.size };
  }

  isPrepared(record: LocalFontRecord): boolean {
    return this.generation === getHostFontState().generation && this.faces.has(localFontFaceKey(record));
  }

  async prepare(records: readonly LocalFontRecord[], sourceOwner: CanvasFontSourceOwner | null = null,
    isCurrent: () => boolean = () => true, pageOutlines = false): Promise<void> {
    const generation = getHostFontState().generation;
    if (this.generation !== generation) {
      this.reset(); this.generation = generation;
      this.off = onHostFontsChanged(() => this.reset());
    }
    this.scopedOutlines = pageOutlines && Boolean(sourceOwner?.clearTextFontOutline
      && sourceOwner?.registerHftOutlineFontFaces);
    if (typeof FontFace === 'undefined' || typeof document === 'undefined') return;
    this.fontSet = document.fonts;
    const epoch = this.epoch;
    const clearedSplitFamilies = new Set<string>();
    for (const record of records) {
      const face = record.hostReference?.face;
      if (!face) continue;
      for (const family of new Set([record.family, ...record.aliases])) {
        const candidates = resolveRendererLocalFontCandidates(family,
          {weight: Number(face.weight ?? 400), slant: face.slant ?? 'normal'});
        const key = family.toLowerCase();
        if (candidates.length > 1 && !clearedSplitFamilies.has(key)) {
          sourceOwner?.clearTextFontFamily?.(family);
          clearedSplitFamilies.add(key);
        }
      }
    }
    for (const record of records) {
      if (epoch !== this.epoch || !isCurrent()) return;
      const key = localFontFaceKey(record);
      if (!record.hostReference || this.failures.has(key)) continue;
      const register = (loaded: CanvasHostFace) => {
        if (!isCurrent()) return;
        for (const family of new Set([record.family, ...record.aliases])) {
          const face = record.hostReference!.face;
          const candidates = resolveRendererLocalFontCandidates(family,
            {weight: Number(face.weight ?? 400), slant: face.slant ?? 'normal'});
          // coverage 저장소는 같은 스타일의 여러 프로그램을 확정하지 못한다.
          // 마지막 조각의 cmap을 family 전체의 coverage로 오인하지 않는다.
          if (candidates.length < 2) {
            sourceOwner?.registerTextFontGlyphCoverage?.(family, new Uint8Array(loaded.bytes), 0);
          }
          if (!this.scopedOutlines && candidates.length < 2 && Number(record.hostReference!.face.weight ?? 400) === 400
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
            const descriptors = fontSymbolDescriptors(bytes,
              { weight: String(face.weight ?? 400), style: face.slant ?? 'normal' }, this.inspectSymbols);
            if (stale()) return;
            const font = new FontFace(alias, bytes, descriptors);
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
            this.faces.set(key, { alias, font, bytes, intrinsicAlias, intrinsicFont, ownership: new Map(),
              unicodeRange: descriptors.unicodeRange ?? '',
              source: { bytes: new Uint8Array(data!.bytes), faceIndex: data!.faceIndex ?? 0 } });
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
    // 실제 Canvas와 같은 후보 순서를 공급한다. 혼합 형식은 기존 painter에 맡긴다.
    if (this.scopedOutlines) return;
    for (const family of clearedSplitFamilies) {
      const candidates = resolveRendererLocalFontCandidates(family, {weight: 400, slant: 'normal'});
      if (candidates.length < 2) continue;
      const loaded = candidates.map(record => this.faces.get(localFontFaceKey(record)));
      if (loaded.some(face => !face)) continue;
      if (loaded.every(face => face && face.bytes.byteLength >= 4
        && new DataView(face.bytes).getUint32(0) === 0x4f54544f)) {
        sourceOwner?.registerHftOutlineFontCandidates?.(family,
          loaded.map(face => new Uint8Array(face!.bytes)));
      }
    }
  }

  /** 현재 paint에 필요한 윤곽만 빌린다. 예외·다른 쪽·portable 출력에 남기지 않는다. */
  withPageOutlines<T>(records: readonly LocalFontRecord[], owner: CanvasFontSourceOwner,
    operation: () => T): T {
    if (!this.scopedOutlines || this.generation !== getHostFontState().generation) return operation();
    const families = new Set(records.filter(record => record.hostReference
      && Number(record.hostReference.face.weight ?? 400) === 400
      && (record.hostReference.face.slant ?? 'normal') === 'normal')
      .flatMap(record => [record.family, ...record.aliases]));
    const installed: string[] = [];
    const primaries: string[] = [];
    const hasPrimaryScope = Boolean(owner.registerPrintFontPrimary && owner.clearPrintFontPrimary);
    try {
      for (const family of families) {
        const candidates = resolveRendererLocalFontCandidates(family, { weight: 400, slant: 'normal' });
        const loaded = candidates.map(record => this.faces.get(localFontFaceKey(record)));
        if (!loaded.length || loaded.some(face => !face)) continue;
        const allCff = loaded.every(isCff);
        if (allCff) {
          installed.push(family);
          owner.registerHftOutlineFontFaces!(family, loaded.map(face => face!.source));
        }
        if (!hasPrimaryScope) continue;
        const primary = canvasHostOrder(loaded, false)?.[0];
        const knownRange = (face: CanvasHostFace) => !face.unicodeRange
          || face.unicodeRange === owner.getFontSymbolUnicodeRange?.(new Uint8Array(face.bytes), 0);
        if (primary && (knownRange(primary) || allCff)) {
          primaries.push(family);
          const source = isCff(primary) ? primary.source : { bytes: new Uint8Array(primary.bytes), faceIndex: 0 };
          owner.registerPrintFontPrimary!(family, source.bytes, source.faceIndex, allCff || loaded.length === 1);
        }
        // 실제 descriptor가 원본 TTF용 별칭을 해소할 때와 같은 순서다.
        const ttf = canvasHostOrder(loaded, true)?.[0];
        if (ttf && !isCff(ttf) && knownRange(ttf)) {
          const alias = `__rhwp_ttf__${family}`;
          primaries.push(alias);
          owner.registerPrintFontPrimary!(alias, new Uint8Array(ttf.bytes), 0, loaded.length === 1);
        }
      }
      return operation();
    } finally {
      for (const family of primaries) owner.clearPrintFontPrimary!(family);
      for (const family of installed) owner.clearTextFontOutline!(family);
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
      const ttfPrefix = '__rhwp_ttf__';
      const sourceTtf = family.startsWith(ttfPrefix);
      const requested = sourceTtf ? family.slice(ttfPrefix.length) : index === 0 ? intrinsic ?? family : family;
      let records = resolveRendererLocalFontCandidates(requested,
        { weight: index === 0 && intrinsic ? 400 : weight, slant });
      if (!records.length && weight >= 600) {
        records = resolveRendererLocalFontCandidates(requested, { weight: 400, slant });
      }
      const loaded = records.map(record => record.hostReference ? this.faces.get(localFontFaceKey(record)) : null);
      // 부분 준비 시 표시와 측정의 후보가 달라진다. 다 준비되면 CFF를
      // TTF보다 먼저 조회하고 각 형식 안에서는 호스트 순서를 보존한다.
      const loadedFaces = canvasHostOrder(loaded, sourceTtf);
      if (!loadedFaces) continue;
      for (const loaded of loadedFaces) {
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
