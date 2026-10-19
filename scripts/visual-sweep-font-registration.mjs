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

export async function installCanvasFont(family, bytes, descriptors, sourceOwner = null, metafileSelected = false) {
  const font = new FontFace(family, bytes, descriptors);
  await font.load();
  document.fonts.add(font);
  // Canvas의 모든 굵기를 paint에 복제하면 미사용 face로 바이트 상한을 소진한다.
  // 같은 catalog에서 요청에 맞게 선택한 실제 프로그램만 별도 세션에 공급한다.
  if (metafileSelected && typeof sourceOwner?.registerMetafileFont === 'function') {
    sourceOwner.registerMetafileFont(family, new Uint8Array(bytes), 0);
  }
  // 브라우저와 같은 프로그램만 paint 세션에 전달한다.
  if (typeof sourceOwner?.registerHftOutlineFont === 'function'
      && String(descriptors.weight || '400') === '400'
      && (descriptors.style || 'normal') === 'normal') {
    sourceOwner.registerHftOutlineFont(family, new Uint8Array(bytes), 0);
  }
  return font;
}
