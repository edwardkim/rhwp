import { getHostFontState, hasHostFontProvider, loadRendererLocalFont,
  resolveRendererLocalFont, localFontFaceKey, type LocalFontRecord } from './local-fonts.ts';
import { parseCssFontFamilyList } from './font-substitution.ts';

export interface NativePuaFontDocument {
  getNativePuaFontRequests?(): string;
  registerNativePuaFont?(family: string, bold: boolean, italic: boolean, bytes: Uint8Array, faceIndex: number): boolean;
  clearNativePuaFonts?(): void;
}
interface Request { family: string; bold: boolean; italic: boolean }
interface State { generation: string; requests: string; keys: Set<string> }

/** FontFace의 준비·선택이 확인된 현재 호스트 세대에서만 원 PUA를 허용한다. */
export class NativePuaFontSession {
  private readonly prepared = new WeakMap<NativePuaFontDocument, State>();

  clear(owner: NativePuaFontDocument | null): void {
    if (owner && this.prepared.delete(owner)) owner.clearNativePuaFonts?.();
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
      const family = parseCssFontFamilyList(request.family)[0];
      if (!family) continue;
      const record = resolveRendererLocalFont(family,
        { weight: request.bold ? 700 : 400, slant: request.italic ? 'italic' : 'normal' });
      if (!record) continue;
      if (!isPrepared(record)) { settled = false; continue; }
      const key = JSON.stringify([request.family, request.bold, request.italic, localFontFaceKey(record)]);
      if (state.keys.has(key)) continue;
      const data = await loadRendererLocalFont(record);
      if (!current() || generation !== `${fontGeneration}:${getHostFontState().generation}`) return false;
      if (!data) continue;
      owner.registerNativePuaFont(request.family, request.bold, request.italic,
        new Uint8Array(data.bytes), data.faceIndex ?? 0);
      state.keys.add(key);
    }
    if (settled) state.requests = raw;
    return current();
  }
}
