import type { DeclaredFontProgramSnapshot, DocumentFontSelectionPlan } from './declared-font-environment.ts';
import { getHostFontState, prepareHostFontCatalog, resolveRendererLocalFontCandidates,
  loadRendererLocalFont } from './local-fonts.ts';
import { normalizeFontFaceData } from './sfnt-face.ts';

/** A catalog entry becomes available only after its selected SFNT face loads. */
export async function prepareDeclaredHostFontPrograms(
  plan: DocumentFontSelectionPlan,
  isCurrent: () => boolean,
): Promise<DeclaredFontProgramSnapshot> {
  const empty = (): DeclaredFontProgramSnapshot => ({ programs: [], publicFallbackBlockedSources: [] });
  const generation = getHostFontState().generation;
  const current = () => isCurrent() && generation === getHostFontState().generation;
  await prepareHostFontCatalog();
  if (!current()) return empty();
  const names = [...new Set(plan.requests.flatMap(request => [...request.primary, ...request.substitute]))];
  if (names.length > 1024) return empty();
  const embedded = new Set(plan.embeddedFamilies ?? []);
  const result = empty();
  for (const query of names) {
    if (embedded.has(query)) continue;
    const records = resolveRendererLocalFontCandidates(query, { weight: 400, slant: 'normal' });
    const prepared: Array<{ order: number; program: DeclaredFontProgramSnapshot['programs'][number] }> = [];
    for (const record of records) {
      if (!record.hostReference) continue;
      const data = await loadRendererLocalFont(record);
      if (!current()) return empty();
      if (!data) continue;
      const faceIndex = data.faceIndex ?? 0;
      const bytes = normalizeFontFaceData(data.bytes, faceIndex);
      if (!bytes || bytes.byteLength < 12) continue;
      const signature = new DataView(bytes).getUint32(0);
      const trueType = signature === 0x00010000 || signature === 0x74727565;
      const openTypeCff = signature === 0x4f54544f;
      const collectionFace = new DataView(data.bytes).getUint32(0) === 0x74746366;
      if (!trueType && !openTypeCff) continue;
      try {
        // No global CSS alias or browser detection snapshot is installed.
        await new FontFace(query, bytes).load();
      } catch { continue; }
      // Rust consumes the same indexed source identity as Native providers.
      const identity = new Uint8Array(data.bytes.byteLength + 4);
      identity.set(new Uint8Array(data.bytes));
      new DataView(identity.buffer).setUint32(data.bytes.byteLength, faceIndex, false);
      const hash = await crypto.subtle.digest('SHA-256', identity);
      if (!current()) return empty();
      const sha256 = [...new Uint8Array(hash)].map(byte => byte.toString(16).padStart(2, '0')).join('');
      prepared.push({ order: trueType ? Number(collectionFace) : 2, program: {
        query, family: query, trueType, openTypeCff,
        programId: sha256,
        ...(collectionFace ? { collectionFace: true } : {}),
      } });
    }
    // The environment has one identity per query. Match the existing declared
    // program order; paint may still use every prepared split face for coverage.
    prepared.sort((left, right) => left.order - right.order);
    if (prepared[0]) result.programs.push(prepared[0].program);
  }
  return result;
}
