import {installIntrinsicCanvasFont, sourceProgramFamilies} from '../rhwp-studio/src/core/canvas-intrinsic-font.mjs';

// 같은 family·style의 실제 프로그램만 TTF → TTC 일치 face → OTF 순서로 공급한다.
// 컬렉션에서 추출한 TTF도 원래 컨테이너의 조회 순서를 보존한다.
export function orderSweepFontPrograms(records) {
  const rank = record => record.openTypeCff ? 2 : record.collectionFace ? 1 : 0;
  return records.toSorted((left, right) => rank(left) - rank(right));
}

// 페이지별 실제 paint 요구를 우선한다. 빈 쪽에 문서 전체 글꼴을 다시 적재하지 않는다.
export function collectSweepPaintFontRequests(owner, pages) {
  if (typeof owner?.getPagePaintFontRequests === 'function') {
    const metafile = [], effects = [];
    for (const page of pages) {
      const requests = JSON.parse(owner.getPagePaintFontRequests(page - 1));
      if (!Array.isArray(requests.metafile) || !Array.isArray(requests.textEffects)) {
        throw Error('페이지 paint 글꼴 요청 형식이 올바르지 않습니다.');
      }
      metafile.push(...requests.metafile);
      effects.push(...requests.textEffects);
    }
    return {metafile, effects};
  }
  // 이전 WASM producer의 비교 실행은 기존 문서 단위 계약을 보존한다.
  return {metafile: JSON.parse(owner.getMetafileFontRequests?.() ?? '[]'),
    effects: JSON.parse(owner.getTextShadowFontRequests?.() ?? '[]')};
}

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

function missingAfterUnicodeRange(owner, bytes, text, missing) {
  if (typeof missing !== 'string' || typeof owner?.getFontSymbolUnicodeRange !== 'function') return missing;
  const ranges = owner.getFontSymbolUnicodeRange(new Uint8Array(bytes), 0);
  if (!ranges) return missing;
  const allowed = ranges.split(',').map(value => {
    const match = /^U\+([0-9A-F]+)(?:-([0-9A-F]+))?$/i.exec(value.trim());
    if (!match) throw Error('글꼴 기호 공급 범위가 유효하지 않습니다.');
    return [parseInt(match[1], 16), parseInt(match[2] ?? match[1], 16)];
  });
  const absent = new Set(missing);
  return [...text].filter(character => absent.has(character) || !allowed.some(([start, end]) => {
    const code = character.codePointAt(0);
    return code >= start && code <= end;
  })).join('');
}

// 실제 글리프를 공급하는 후보만 고른다. 빈 요청은 기존 전체 사다리를 유지한다.
export async function selectTextEffectFontFaces(owner, fonts, requests, loadBytes) {
  const selected = new Set();
  for (const request of requests) {
    const needsSpaceCoverage = typeof request.text === 'string' && /\s/u.test(request.text);
    const hasSpaceCoverage = typeof owner?.fontMissingTextWithSpaces === 'function';
    if (!request.text || (needsSpaceCoverage && !hasSpaceCoverage)
        || (!needsSpaceCoverage && typeof owner?.fontMissingText !== 'function'
          && typeof owner?.fontSupportsText !== 'function')) {
      for (const key of selectMetafileFontFaces(fonts, [request])) selected.add(key);
      continue;
    }
    let remaining = request.text;
    for (const family of request.families) {
      for (const key of selectMetafileFontFaces(fonts, [{...request, families: [family]}])) {
        const group = fonts.filter(face => face.families.some(name =>
          `${name.toLowerCase()}|${face.weight}|${face.style}` === key));
        // 실제 Canvas 공급과 같은 원본 프로그램·순서로 문자 소속을 확인한다.
        const records = orderSweepFontPrograms(group);
        for (const record of records) {
          const bytes = await loadBytes(record);
          if (!bytes) continue;
          // 원 TTC의 faceIndex가 아니라 공급한 개별 SFNT의 0번 face다.
          let missing = needsSpaceCoverage
            ? owner.fontMissingTextWithSpaces(new Uint8Array(bytes), 0, remaining)
            : typeof owner.fontMissingText === 'function'
              ? owner.fontMissingText(new Uint8Array(bytes), 0, remaining)
              : owner.fontSupportsText(new Uint8Array(bytes), 0, remaining) ? '' : remaining;
          missing = missingAfterUnicodeRange(owner, bytes, remaining, missing);
          if (typeof missing !== 'string' || missing === remaining) continue;
          selected.add(key);
          remaining = missing;
          if (!remaining) break;
        }
        if (!remaining) break;
      }
      if (!remaining) break;
    }
  }
  return selected;
}

// 메타파일과 그림자 모두 실제 문자가 필요한 공급자만 paint에 등록한다.
export async function selectSweepPaintFontFaces(owner, fonts, metafileRequests, effectRequests, loadBytes) {
  const selected = await selectTextEffectFontFaces(owner, fonts, metafileRequests, loadBytes);
  const effects = await selectTextEffectFontFaces(owner, fonts, effectRequests, loadBytes);
  for (const key of effects) selected.add(key);
  return selected;
}

export async function installCanvasFont(family, bytes, descriptors, sourceOwner = null, metafileSelected = false, intrinsicRequested = false, onPrepared = null, registerOutline = true, registerCoverage = true) {
  // 제품과 같은 Rust 함수를 사용한다. 원본 메타파일·명시 컬러 자원은 바꾸지 않는다.
  const prepared = sourceOwner?.ordinaryTextFontProgram?.(new Uint8Array(bytes), 0);
  const paintBytes = prepared ? Uint8Array.from(prepared).buffer : bytes;
  await onPrepared?.(paintBytes);
  const unicodeRange = sourceOwner?.getFontSymbolUnicodeRange?.(new Uint8Array(bytes), 0);
  if (unicodeRange) descriptors = {...descriptors, unicodeRange};
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
  if (registerCoverage && typeof sourceOwner?.registerTextFontGlyphCoverage === 'function') {
    sourceOwner.registerTextFontGlyphCoverage(family, new Uint8Array(paintBytes), 0);
  }
  if (registerOutline && typeof sourceOwner?.registerHftOutlineFont === 'function'
      && String(descriptors.weight || '400') === '400'
      && (descriptors.style || 'normal') === 'normal') {
    sourceOwner.registerHftOutlineFont(family, new Uint8Array(paintBytes), 0);
  }
  return font;
}

// Canvas에서 선택한 원본 TrueType과 HFT 대체 OTF를 출처별로 조판에 공급한다.
export async function prepareHftOtfFonts(owner, fonts, loadBytes) {
  if (typeof owner?.getNativeOtfFontRequests !== 'function'
      || typeof owner?.prepareNativeOtfFonts !== 'function') {
    throw Error('HFT OpenType 폭 공급 API가 없는 WASM입니다.');
  }
  const requests = JSON.parse(owner.getNativeOtfFontRequests());
  const programs = new Map(), loaded = new Map(), proofs = [];
  for (const request of requests) {
    const selectedFaces = selectMetafileFontFaces(fonts, [{families: sourceProgramFamilies(request.family, request.source_ttf),
      weight: request.source_ttf ? 400 : (request.bold ? 700 : 400), italic: request.italic}]);
    const selected = [], seen = new Set();
    let selectedBytes = 0;
    for (const key of selectedFaces) {
      const matches = fonts.filter(record => record.families.some(name =>
        `${name.toLowerCase()}|${record.weight}|${record.style}` === key));
      const records = request.source_ttf
        ? orderSweepFontPrograms(matches).filter(record => !record.openTypeCff)
        : orderSweepFontPrograms(matches);
      for (const record of records) {
        if (seen.has(record.sha256)) continue;
        seen.add(record.sha256);
        let bytes = loaded.get(record.sha256);
        if (!bytes) { bytes = await loadBytes(record); loaded.set(record.sha256, bytes); }
        if (bytes.byteLength < 4 || bytes.byteLength > 32 * 1024 * 1024
            || selectedBytes + bytes.byteLength > 64 * 1024 * 1024) continue;
        const signature = new DataView(bytes).getUint32(0);
        if (request.source_ttf ? ![0x00010000, 0x74727565].includes(signature)
          : ![0x00010000, 0x74727565, 0x4f54544f].includes(signature)) continue;
        selected.push(new Uint8Array(bytes));
        selectedBytes += bytes.byteLength;
        proofs.push({family: request.family, bold: request.bold, italic: request.italic, sourceTtf: Boolean(request.source_ttf),
          weight: record.weight, style: record.style, sha256: record.sha256});
      }
    }
    if (selected.length) programs.set(JSON.stringify([request.family, request.bold, request.italic, Boolean(request.source_ttf)]), selected);
  }
  owner.prepareNativeOtfFonts(raw => {
    const request = JSON.parse(raw);
    const selected = programs.get(JSON.stringify([request.family, request.bold, request.italic, Boolean(request.source_ttf)]));
    return selected?.length === 1 ? selected[0] : selected ?? null;
  });
  return proofs;
}

// Native의 원 face 부재 관측과 같은 프로그램을 공급할 때만 공통 측정 face를 확정한다.
// 부분적인 브라우저 글꼴 목록이나 document.fonts.check로 설치 부재를 추측하지 않는다.
export async function prepareObservedFontFallbacks(owner, fonts, records, loadBytes) {
  if (!records.length) return [];
  if (typeof owner?.getDeclaredFontFallbackRequests !== 'function'
      || typeof owner?.setObservedFontFallbacks !== 'function') {
    throw Error('실제 대체 글꼴 관측 API가 없는 WASM입니다.');
  }
  const requests = JSON.parse(owner.getDeclaredFontFallbackRequests());
  const requestKey = r => JSON.stringify([r.family, r.fallback, r.bold, r.italic]);
  const known = new Set(requests.map(requestKey)), seen = new Set(), prepared = [];
  for (const {request, source} of records) {
    const key = requestKey(request);
    if (!known.has(key) || seen.has(key)) throw Error('원본에 없는 대체 글꼴 관측 또는 중복 관측입니다.');
    seen.add(key);
    const hasFamily = (record, family) => record.families.some(name => name.toLowerCase() === family.toLowerCase());
    if (fonts.some(record => hasFamily(record, request.family))) {
      throw Error('WASM 공급 목록에 원 face가 있어 Native의 부재 관측을 적용할 수 없습니다.');
    }
    const keys = selectMetafileFontFaces(fonts,
      [{families: [source.family], weight: request.bold ? 700 : 400, italic: request.italic}]);
    const record = fonts.find(font => hasFamily(font, source.family)
      && keys.has(`${source.family.toLowerCase()}|${font.weight}|${font.style}`));
    if (!record || record.sourceSha256 !== source.sha256 || record.faceIndex !== source.faceIndex) {
      throw Error('Canvas가 선택할 글꼴 프로그램이 실제 공급자의 관측과 다릅니다.');
    }
    const bytes = await loadBytes(record);
    const digest = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)),
      byte => byte.toString(16).padStart(2, '0')).join('');
    if (digest !== record.sha256) throw Error('관측 글꼴의 실제 공급 바이트가 변경됐습니다.');
    prepared.push({request, source, record, bytes});
  }
  for (const {source, record, bytes} of prepared) {
    await installCanvasFont(source.family, bytes,
      {weight: String(record.weight), style: record.style}, owner);
  }
  owner.setObservedFontFallbacks(JSON.stringify(records));
  return prepared.map(({request, source, record}) => ({request, source,
    suppliedSha256: record.sha256, weight: record.weight, style: record.style, repairs: record.repairs}));
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
        const matches = fonts.filter(font => font.families.some(name =>
          `${name.toLowerCase()}|${font.weight}|${font.style}` === key));
        const records = orderSweepFontPrograms(matches);
        const seen = new Set();
        for (const record of records) {
          if (seen.has(record.sha256)) continue;
          seen.add(record.sha256);
          const programKey = `${key}|${record.sha256}`;
          let bytes = loaded.get(programKey);
          if (!bytes) {
            bytes = await loadBytes(record);
            await installCanvasFont(family, bytes,
              {weight: String(record.weight), style: record.style}, owner,
              false, false, null, records.length < 2, records.length < 2);
            loaded.set(programKey, bytes);
          }
          const registerSource = request.literal_figure_space
            ? owner.registerLiteralFigureSpaceFontForSource : owner.registerNativePuaFontForSource;
          const registerLegacy = request.literal_figure_space
            ? owner.registerLiteralFigureSpaceFont : owner.registerNativePuaFont;
          if (request.source_ttf && typeof registerSource !== 'function') {
            throw Error('원본 TTF별 PUA 등록 API가 없는 WASM입니다.');
          }
          const registered = typeof registerSource === 'function'
            ? registerSource.call(owner, request.family, request.bold, request.italic,
              Boolean(request.source_ttf), new Uint8Array(bytes), 0)
            : typeof registerLegacy === 'function' && registerLegacy.call(owner, request.family,
              request.bold, request.italic, new Uint8Array(bytes), 0);
          if (registered) {
            proofs.push({family: request.family, literalFigureSpace: Boolean(request.literal_figure_space), selectedFamily: family, weight: record.weight,
              style: record.style, sha256: record.sha256, sourceSha256: record.sourceSha256});
            accepted = true;
            break;
          }
        }
        if (accepted) break;
      }
      if (accepted) break;
    }
  }
  return proofs;
}
