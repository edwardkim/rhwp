import { getHostFontState, hasHostFontProvider, loadRendererLocalFont,
  resolveRendererLocalFont, localFontFaceKey, type LocalFontRecord } from './local-fonts.ts';
import { parseCssFontFamilyList } from './font-substitution.ts';
import { normalizeFontFaceData } from './sfnt-face.ts';

export interface NativePuaFontDocument {
  getNativePuaFontRequests?(): string;
  registerNativePuaFont?(family: string, bold: boolean, italic: boolean, bytes: Uint8Array, faceIndex: number): boolean;
  clearNativePuaFonts?(): void;
}
interface Request { family: string; paint_families?: string[]; bold: boolean; italic: boolean }
interface State { generation: string; requests: string; keys: Set<string> }

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
        const record = resolveRendererLocalFont(family,
          { weight: request.bold ? 700 : 400, slant: request.italic ? 'italic' : 'normal' });
        if (record?.hostReference) selected.set(localFontFaceKey(record), record);
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
    const raw = owner.getNativePuaFontRequests();
    if (state.requests === raw) return true;
    const requests: Request[] = JSON.parse(raw);
    let settled = true;
    for (const request of requests) {
      const key = JSON.stringify([request.family, request.bold, request.italic]);
      if (state.keys.has(key)) continue;
      for (const family of request.paint_families ?? parseCssFontFamilyList(request.family)) {
        const record = resolveRendererLocalFont(family,
          { weight: request.bold ? 700 : 400, slant: request.italic ? 'italic' : 'normal' });
        if (!record) continue;
        if (!isPrepared(record)) { settled = false; break; }
        const data = await loadRendererLocalFont(record);
        if (!current() || generation !== `${fontGeneration}:${getHostFontState().generation}`) return false;
        if (!data) continue;
        const bytes = normalizeFontFaceData(data.bytes, data.faceIndex ?? 0);
        if (!bytes) continue;
        if (owner.registerNativePuaFont(request.family, request.bold, request.italic,
          new Uint8Array(bytes), 0)) {
          state.keys.add(key);
          break;
        }
      }
    }
    if (settled) state.requests = raw;
    return current();
  }
}
