// 호스트는 실제 공급만 증명한다. 원 face/대체 face의 선택은 Rust 공통 API가 소유한다.
export async function applySweepFontEnvironment(doc, environmentJson, preparePrograms) {
  if (environmentJson !== null) {
    doc.setFontEnvironment(environmentJson);
    return {source: 'explicit', environment: JSON.parse(environmentJson)};
  }
  if (typeof doc.getDocumentFontSelectionPlan !== 'function'
      || typeof doc.buildFontEnvironmentFromAvailablePrograms !== 'function'
      || typeof doc.setFontEnvironment !== 'function') {
    throw Error('실제 공급 프로그램으로 글꼴 환경을 선택할 수 없는 이전 WASM입니다.');
  }
  const plan = JSON.parse(doc.getDocumentFontSelectionPlan());
  if (plan.environmentId !== null) throw Error('새 문서에 예상하지 못한 글꼴 환경이 있습니다.');
  const prepared = await preparePrograms(plan);
  const environment = JSON.parse(doc.buildFontEnvironmentFromAvailablePrograms(JSON.stringify(prepared.programs)));
  if (environment !== null) doc.setFontEnvironment(JSON.stringify(environment));
  return {source: 'preparedPrograms', environment, programs: prepared.programs, loaded: prepared.loaded || []};
}

export function selectSweepProgramFaces(plan, fonts) {
  const names = [...new Set(plan.requests.flatMap(request => [...request.primary, ...request.substitute]))];
  if (names.length > 1024) throw Error('문서의 글꼴 프로그램 후보가 한도를 넘었습니다.');
  const embedded = new Set((plan.embeddedFamilies || []).map(name => name.toLowerCase()));
  const rank = face => (face.style === 'normal' ? 0 : 1000) + Math.abs(face.weight - 400);
  return names.flatMap(query => {
    if (embedded.has(query.toLowerCase())) return [];
    // Native의 기본 normal/400 조회와 같은 요청이다. 원 face와 대체 중
    // 어느 것을 사용할지는 여기서 결정하지 않고 실제 준비된 후보만 반환한다.
    const candidates = fonts.filter(face => face.families.some(name => name.toLowerCase() === query.toLowerCase())
      && !face.families.some(name => embedded.has(name.toLowerCase())));
    const face = candidates.toSorted((a, b) => rank(a) - rank(b))[0];
    return face ? [{query, face}] : [];
  });
}

export function hasTrueTypeSfntSignature(bytes) {
  if (bytes.byteLength < 4) return false;
  const signature = new DataView(bytes).getUint32(0);
  return signature === 0x00010000 || signature === 0x74727565;
}

// FontTools로 준비한 실제 SFNT bytes의 로드 성공·hash·형식을 증명한 뒤만
// 환경 선택에 제공한다. local() 성공이나 catalogue의 이름만으로 선언하지 않는다.
export async function loadSweepFontPrograms(plan, fonts, host = globalThis) {
  const programs = [], loaded = [], supplied = new Set(), resources = new Map();
  for (const {query, face: record} of selectSweepProgramFaces(plan, fonts)) {
    let resource = resources.get(record.url);
    if (!resource) {
      const response = await host.fetch(record.url);
      if (!response.ok) throw Error('선택한 글꼴 프로그램의 바이트 공급에 실패했습니다.');
      const bytes = await response.arrayBuffer();
      const digest = await host.crypto.subtle.digest('SHA-256', bytes);
      const sha256 = [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, '0')).join('');
      if (sha256 !== record.sha256) throw Error('선택한 글꼴 프로그램의 공급 hash가 다릅니다.');
      resource = {bytes, sha256}; resources.set(record.url, resource);
    }
    const family = record.families[0];
    for (const alias of new Set([query, family])) {
      const key = `${alias.toLowerCase()}|${record.weight}|${record.style}`;
      if (supplied.has(key)) continue;
      const font = new host.FontFace(alias, resource.bytes, {weight: String(record.weight), style: record.style});
      await font.load();
      if (font.status !== 'loaded' || font.family !== alias) throw Error('글꼴 프로그램의 실제 준비 상태가 다릅니다.');
      host.document.fonts.add(font); supplied.add(key);
      loaded.push({family: alias, registeredFamily: font.family, sourceSha256: record.sourceSha256,
        sha256: resource.sha256, faceIndex: record.faceIndex, weight: record.weight, style: record.style,
        repairs: record.repairs, sourceKind: record.sourceKind || 'external'});
    }
    programs.push({query, family, programId: `sweep-${resource.sha256}`,
      trueType: hasTrueTypeSfntSignature(resource.bytes)});
  }
  await host.document.fonts.ready;
  return {programs, loaded};
}
