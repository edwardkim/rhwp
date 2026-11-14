import { getHostFontState, hasHostFontProvider, loadRendererLocalFont,
  resolveRendererLocalFontCandidates, localFontFaceKey, type LocalFontRecord } from './local-fonts.ts';
import { parseCssFontFamilyList } from './font-substitution.ts';
import { normalizeFontFaceData } from './sfnt-face.ts';
import { sourceProgramFamilies } from './canvas-intrinsic-font.mjs';

export interface NativeFontProgram { bytes: Uint8Array; faceIndex: number }
export interface NativePuaFontDocument {
  getNativePuaFontRequests?(): string;
  registerNativePuaFont?(family: string, bold: boolean, italic: boolean, bytes: Uint8Array, faceIndex: number): boolean;
  registerLiteralFigureSpaceFont?(family: string, bold: boolean, italic: boolean, bytes: Uint8Array, faceIndex: number): boolean;
  registerNativePuaFontForSource?(family: string, bold: boolean, italic: boolean, sourceTtf: boolean, bytes: Uint8Array, faceIndex: number): boolean;
  registerLiteralFigureSpaceFontForSource?(family: string, bold: boolean, italic: boolean, sourceTtf: boolean, bytes: Uint8Array, faceIndex: number): boolean;
  clearNativePuaFonts?(): void;
  getNativeOtfFontRequests?(): string;
  prepareNativeOtfFonts?(provider: (request: string) => Uint8Array | Uint8Array[] | null): boolean;
  prepareNativeOtfFontFaces?(provider: (request: string) => NativeFontProgram[] | null): boolean;
}
interface Request { literal_figure_space?: boolean; source_ttf?: boolean; family: string; paint_families?: string[]; bold: boolean; italic: boolean }
interface State { generation: string; requests: string; keys: Set<string> }

function facesForRequest(family: string, request: Request): LocalFontRecord[] {
  const slant = request.italic ? 'italic' : 'normal';
  const selected = resolveRendererLocalFontCandidates(family, {weight: request.bold ? 700 : 400, slant});
  return selected.length || !request.bold ? selected
    : resolveRendererLocalFontCandidates(family, {weight: 400, slant});
}

function otfFaces(request: Request): LocalFontRecord[] {
  const records = sourceProgramFamilies(request.family, request.source_ttf).flatMap(family =>
    facesForRequest(family, request.source_ttf ? { ...request, bold: false } : request));
  return [...new Map(records.map(record => [localFontFaceKey(record), record])).values()];
}

/** FontFace의 준비·선택이 확인된 현재 호스트 세대에서만 원 PUA를 허용한다. */
export class NativePuaFontSession {
  private readonly prepared = new WeakMap<NativePuaFontDocument, State>();

  clear(owner: NativePuaFontDocument | null): void {
    if (owner && this.prepared.delete(owner)) owner.clearNativePuaFonts?.();
  }

  records(owner: NativePuaFontDocument | null): LocalFontRecord[] {
    const selected = new Map<string, LocalFontRecord>();
    const requests: Request[] = JSON.parse(owner?.getNativePuaFontRequests?.() ?? '[]');
    for (const request of requests) {
      for (const family of request.paint_families ?? parseCssFontFamilyList(request.family)) {
        const records = facesForRequest(family, request);
        for (const record of records) {
          if (record.hostReference) selected.set(localFontFaceKey(record), record);
        }
      }
    }
    for (const request of JSON.parse(owner?.getNativeOtfFontRequests?.() ?? '[]') as Request[]) {
      for (const record of otfFaces(request)) {
        if (record.hostReference) selected.set(localFontFaceKey(record), record);
      }
    }
    return [...selected.values()];
  }

  async prepare(owner: NativePuaFontDocument | null, fontGeneration: number,
    isPrepared: (record: LocalFontRecord) => boolean, current: () => boolean): Promise<boolean> {
    if (!owner?.getNativePuaFontRequests || !owner.registerNativePuaFont) return true;
    const generation = `${fontGeneration}:${getHostFontState().generation}`;
    let state = this.prepared.get(owner);
    if (!state || state.generation !== generation) {
      if (state) owner.clearNativePuaFonts?.();
      state = { generation, requests: '', keys: new Set() };
      this.prepared.set(owner, state);
    }
    if (!hasHostFontProvider()) return true;
    const puaRaw = owner.getNativePuaFontRequests();
    const otfRaw = owner.getNativeOtfFontRequests?.() ?? '[]';
    const raw = JSON.stringify([puaRaw, otfRaw]);
    if (state.requests === raw) return true;
    const requests: Request[] = JSON.parse(puaRaw);
    let settled = true;
    for (const request of requests) {
      const key = JSON.stringify([request.family, request.bold, request.italic, Boolean(request.literal_figure_space), Boolean(request.source_ttf)]);
      if (state.keys.has(key)) continue;
      families: for (const family of request.paint_families ?? parseCssFontFamilyList(request.family)) {
        const records = facesForRequest(family, request);
        const programs: Uint8Array[] = [];
        for (const record of records) {
          if (!isPrepared(record)) { settled = false; break families; }
          const data = await loadRendererLocalFont(record);
          if (!current() || this.prepared.get(owner) !== state
            || generation !== `${fontGeneration}:${getHostFontState().generation}`) return false;
          if (!data) continue;
          const bytes = normalizeFontFaceData(data.bytes, data.faceIndex ?? 0);
          if (!bytes) continue;
          programs.push(new Uint8Array(bytes));
        }
        if (!request.source_ttf && records.length > 1 && !programs.some(bytes => bytes.byteLength >= 4
          && new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint32(0) === 0x4f54544f)) continue;
        programs.sort((a, b) => {
          const cff = (bytes: Uint8Array) => bytes.byteLength >= 4
            && new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint32(0) === 0x4f54544f;
          return request.source_ttf ? Number(cff(a)) - Number(cff(b)) : Number(cff(b)) - Number(cff(a));
        });
        for (const bytes of programs) {
          const registerSource = request.literal_figure_space
            ? owner.registerLiteralFigureSpaceFontForSource : owner.registerNativePuaFontForSource;
          const register: NativePuaFontDocument['registerNativePuaFont'] = request.literal_figure_space
            ? owner.registerLiteralFigureSpaceFont : owner.registerNativePuaFont;
          if (request.source_ttf && !registerSource) { settled = false; break families; }
          const registered = registerSource
            ? registerSource.call(owner, request.family, request.bold, request.italic,
              Boolean(request.source_ttf), bytes, 0)
            : register?.call(owner, request.family, request.bold, request.italic, bytes, 0);
          if (registered) {
            state.keys.add(key);
            break families;
          }
        }
      }
    }
    if (owner.prepareNativeOtfFonts) {
      const programs = new Map<string, NativeFontProgram[]>();
      for (const request of JSON.parse(otfRaw) as Request[]) {
        const selected: NativeFontProgram[] = [];
        const records = otfFaces(request);
        for (const record of records) {
          if (!record.hostReference) continue;
          if (!isPrepared(record)) { settled = false; continue; }
          const data = await loadRendererLocalFont(record);
          if (!current() || this.prepared.get(owner) !== state
            || generation !== `${fontGeneration}:${getHostFontState().generation}`) return false;
          const bytes = data && normalizeFontFaceData(data.bytes, data.faceIndex ?? 0);
          if (!bytes || bytes.byteLength < 4) continue;
          const signature = new DataView(bytes).getUint32(0);
          if (request.source_ttf ? ![0x00010000, 0x74727565].includes(signature) : signature !== 0x4f54544f) continue;
          selected.push(owner.prepareNativeOtfFontFaces
            ? { bytes: new Uint8Array(data!.bytes), faceIndex: data!.faceIndex ?? 0 }
            : { bytes: new Uint8Array(bytes), faceIndex: 0 });
        }
        if (selected.length) {
          programs.set(JSON.stringify([request.family, request.bold, request.italic, Boolean(request.source_ttf)]), selected);
        }
      }
      if (!settled) return false;
      const selectedPrograms = (rawRequest: string) => {
        const request: Request = JSON.parse(rawRequest);
        return programs.get(JSON.stringify([request.family, request.bold, request.italic, Boolean(request.source_ttf)])) ?? null;
      };
      if (owner.prepareNativeOtfFontFaces) owner.prepareNativeOtfFontFaces(selectedPrograms);
      else owner.prepareNativeOtfFonts(rawRequest => {
        const selected = selectedPrograms(rawRequest)?.map(program => program.bytes);
        return selected?.length === 1 ? selected[0] : selected ?? null;
      });
    }
    if (settled) state.requests = raw;
    return current();
  }
}
