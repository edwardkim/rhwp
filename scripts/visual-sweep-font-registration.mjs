import {installIntrinsicCanvasFont} from '../rhwp-studio/src/core/canvas-intrinsic-font.mjs';

// fontdb 0.23의 style/weight 우선순위로 실제 요청한 메타파일 face만 고른다.
export function selectMetafileFontFaces(fonts, requests) {
  const selected = new Set();
  for (const request of requests) {
    for (const family of request.families) {
      let candidates = fonts.filter(record => record.families.some(
        name => name.toLowerCase() === family.toLowerCase()));
      const styles = request.italic ? ['italic', 'oblique', 'normal'] : ['normal', 'oblique', 'italic'];
      const style = styles.find(value => candidates.some(record => record.style === value));
      candidates = candidates.filter(record => record.style === style);
      if (!candidates.length) continue;
      const weight = Number(request.weight);
      let chosen = candidates.find(record => record.weight === weight);
      if (!chosen && weight >= 400 && weight < 450) chosen = candidates.find(record => record.weight === 500);
      if (!chosen && weight >= 450 && weight <= 500) chosen = candidates.find(record => record.weight === 400);
      if (!chosen) {
        const preferred = candidates.filter(record => weight <= 500 ? record.weight <= weight : record.weight >= weight);
        const ordered = [...(preferred.length ? preferred : candidates)];
        ordered.sort((a, b) => weight <= 500
          ? (preferred.length ? b.weight - a.weight : a.weight - b.weight)
          : (preferred.length ? a.weight - b.weight : b.weight - a.weight));
        chosen = ordered[0];
      }
      selected.add(`${family.toLowerCase()}|${chosen.weight}|${chosen.style}`);
    }
  }
  return selected;
}

export async function installCanvasFont(family, bytes, descriptors, sourceOwner = null, metafileSelected = false, intrinsicRequested = false, onPrepared = null) {
  // 제품과 같은 Rust 함수를 사용한다. 원본 메타파일·명시 컬러 자원은 바꾸지 않는다.
  const prepared = sourceOwner?.ordinaryTextFontProgram?.(new Uint8Array(bytes), 0);
  const paintBytes = prepared ? Uint8Array.from(prepared).buffer : bytes;
  await onPrepared?.(paintBytes);
  const font = new FontFace(family, paintBytes, descriptors);
  await font.load();
  document.fonts.add(font);
  if (intrinsicRequested) await installIntrinsicCanvasFont(family, paintBytes, descriptors);
  // Canvas의 모든 굵기를 paint에 복제하면 미사용 face로 바이트 상한을 소진한다.
  // 같은 catalog에서 요청에 맞게 선택한 실제 프로그램만 별도 세션에 공급한다.
  if (metafileSelected && typeof sourceOwner?.registerMetafileFont === 'function') {
    sourceOwner.registerMetafileFont(family, new Uint8Array(bytes), 0);
  }
  // 브라우저와 같은 프로그램만 paint 세션에 전달한다.
  if (typeof sourceOwner?.registerTextFontGlyphCoverage === 'function') {
    sourceOwner.registerTextFontGlyphCoverage(family, new Uint8Array(paintBytes), 0);
  }
  if (typeof sourceOwner?.registerHftOutlineFont === 'function'
      && String(descriptors.weight || '400') === '400'
      && (descriptors.style || 'normal') === 'normal') {
    sourceOwner.registerHftOutlineFont(family, new Uint8Array(paintBytes), 0);
  }
  return font;
}

// 쪽수/대상 쪽을 고르기 전에 실제 FontFace와 같은 PUA 프로그램을 공급한다.
export async function prepareNativePuaFonts(owner, fonts, loadBytes) {
  if (typeof owner?.getNativePuaFontRequests !== 'function'
      || typeof owner?.registerNativePuaFont !== 'function') return [];
  const requests = JSON.parse(owner.getNativePuaFontRequests());
  const loaded = new Map(), proofs = [];
  for (const request of requests) {
    for (const family of request.paint_families || [request.family]) {
      const selected = selectMetafileFontFaces(fonts,
        [{families: [family], weight: request.bold ? 700 : 400, italic: request.italic}]);
      let accepted = false;
      for (const key of selected) {
        const record = fonts.find(font => font.families.some(name =>
          `${name.toLowerCase()}|${font.weight}|${font.style}` === key));
        if (!record) continue;
        let bytes = loaded.get(key);
        if (!bytes) {
          bytes = await loadBytes(record);
          await installCanvasFont(family, bytes,
            {weight: String(record.weight), style: record.style}, owner);
          loaded.set(key, bytes);
        }
        const register = request.literal_figure_space
          ? owner.registerLiteralFigureSpaceFont : owner.registerNativePuaFont;
        if (typeof register === 'function' && register.call(owner, request.family,
          request.bold, request.italic, new Uint8Array(bytes), 0)) {
          proofs.push({family: request.family, literalFigureSpace: Boolean(request.literal_figure_space), selectedFamily: family, weight: record.weight,
            style: record.style, sha256: record.sha256, sourceSha256: record.sourceSha256});
          accepted = true;
          break;
        }
      }
      if (accepted) break;
    }
  }
  return proofs;
}
