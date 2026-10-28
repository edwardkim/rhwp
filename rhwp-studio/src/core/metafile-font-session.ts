import { getHostFontState, hasHostFontProvider, loadRendererLocalFont, loadStoredLocalFonts,
  prepareHostFontCatalog, resolveRendererLocalFont, localFontFaceKey } from './local-fonts';

export interface MetafileFontDocument {
  getMetafileFontRequests?(): string;
  registerMetafileFont?(family: string, bytes: Uint8Array, faceIndex: number): boolean;
  clearMetafileFonts?(): void;
}

interface FontRequest { families: string[]; weight: number; italic: boolean; text: string }
interface Prepared { generation: string; requests: string; keys: Set<string> }

/** 이미지 내부 글꼴도 실제 host/local SFNT를 받는다. CSS 존재를 bytes 공급으로 간주하지 않는다. */
export class MetafileFontSession {
  private readonly prepared = new WeakMap<MetafileFontDocument, Prepared>();

  async prepare(owner: MetafileFontDocument | null, fontGeneration: number,
    current: () => boolean): Promise<boolean> {
    if (!owner?.getMetafileFontRequests || !owner.registerMetafileFont) return true;
    const raw = owner.getMetafileFontRequests();
    const requests: FontRequest[] = JSON.parse(raw);
    if (!requests.length) return true;
    if (hasHostFontProvider()) await prepareHostFontCatalog();
    else {
      try { await loadStoredLocalFonts(); }
      catch { /* 만료된 로컬 권한으로 bytes를 못 받으면 기존 SVG 재생을 유지한다. */ }
    }
    if (!current()) return false;
    const generation = `${fontGeneration}:${getHostFontState().generation}`;
    let state = this.prepared.get(owner);
    if (state?.generation === generation && state.requests === raw) return true;
    if (!state || state.generation !== generation) {
      owner.clearMetafileFonts?.();
      state = { generation, requests: '', keys: new Set() };
      this.prepared.set(owner, state);
    }
    for (const request of requests) {
      for (const family of request.families) {
        const style = { weight: request.weight, slant: request.italic ? 'italic' as const : 'normal' as const };
        const record = resolveRendererLocalFont(family, style) ?? resolveRendererLocalFont(family);
        if (!record) continue;
        const key = `${family.toLowerCase()}|${localFontFaceKey(record)}`;
        if (state.keys.has(key)) continue;
        const data = await loadRendererLocalFont(record);
        if (!current() || generation !== `${fontGeneration}:${getHostFontState().generation}`) return false;
        if (!data) continue;
        // 실패를 빈 glyph 프로그램으로 대체하지 않는다. 미공급 face는 기존 SVG 경로가 담당한다.
        owner.registerMetafileFont(family, new Uint8Array(data.bytes), data.faceIndex ?? 0);
        state.keys.add(key);
      }
    }
    state.requests = raw;
    return current();
  }
}
