import { registerLoadedLocalFontFace } from './font-loader.ts';
import { loadLocalFontBytes, resolveLocalFont } from './local-fonts.ts';
import { normalizeFontFaceData } from './sfnt-face.ts';

export interface DocumentFontSelectionPlan {
  environmentId: string | null;
  embeddedFamilies?: string[];
  requests: Array<{ source: string; primary: string[]; substitute: string[] }>;
}

interface LoadedProgram {
  query: string;
  family: string;
  bytes: ArrayBuffer;
  programId: string;
  face: FontFace;
  trueType: boolean;
}

let programGeneration = 0;
const loadedPrograms = new Map<string, LoadedProgram>();
const suppliedAliases = new Map<string, string>();

async function loadLocalProgram(query: string): Promise<LoadedProgram | null> {
  // local() can report loaded while the platform supplies LastResort glyphs.
  // Only a face from the already authorized catalog may become a declared
  // local program. An unverified name remains on the ordinary fallback path.
  const record = resolveLocalFont(query);
  if (record?.detectionSource !== 'local-font-access') return null;
  const cached = loadedPrograms.get(query);
  if (cached) return cached;
  try {
    // 원본 API face의 바이트를 직접 로드한다. 읽기/형식 실패는 공급 완료가 아니다.
    const original = await loadLocalFontBytes(query);
    // The approved API returns bytes for the matched face. A native local()
    // cascade can silently replace missing glyphs with LastResort. Let the
    // browser consume that same program directly, including its cmap.
    if (!original || (original.byteLength >= 4 && new DataView(original).getUint32(0) === 0x74746366)) return null;
    // FontFace와 CanvasKit의 기존 공통 SFNT 준비를 쓴다. 원본의 glyph/hmtx는
    // 유지하며 디렉터리·cmap 등 브라우저가 거부하는 메타데이터만 같은 방식으로 준비한다.
    const bytes = normalizeFontFaceData(original, 0);
    if (!bytes) return null;
    const face = new FontFace(query, bytes);
    await face.load();
    const signature = bytes.byteLength >= 4 ? new DataView(bytes).getUint32(0) : 0;
    const trueType = signature === 0x00010000 || signature === 0x74727565;
    const program = { query, family: query, bytes, programId: `browser-local-${++programGeneration}`, face, trueType };
    loadedPrograms.set(query, program);
    return program;
  } catch {
    // 부재 결과는 다음 문서까지 캐시하지 않아 새로 설치한 글꼴을 놓치지 않는다.
    return null;
  }
}

/** 공급과 공통 선택에 같은 실제 프로그램 snapshot을 전달한다. */
export async function prepareDeclaredLocalPrograms(
  plan: DocumentFontSelectionPlan,
  isCurrent: () => boolean,
): Promise<Array<{ query: string; family: string; programId: string; trueType: boolean }>> {
  const names = [...new Set(plan.requests.flatMap(request => [...request.primary, ...request.substitute]))];
  if (names.length > 1024 || typeof FontFace === 'undefined') return [];
  const programs = new Map<string, LoadedProgram>();
  const embedded = new Set(plan.embeddedFamilies ?? []);
  for (const name of names) {
    const program = await loadLocalProgram(name);
    if (!isCurrent()) return [];
    if (program) programs.set(name, program);
  }
  for (const request of plan.requests) {
    for (const candidates of [request.primary, request.substitute]) {
      const requested = candidates[0];
      if (!requested) continue;
      const selected = candidates.map(candidate => programs.get(candidate)).find(Boolean);
      if (!selected) continue;
      if (!embedded.has(selected.family)) registerLoadedLocalFontFace(selected.face);
      if (embedded.has(requested) || requested === selected.query || suppliedAliases.get(requested) === selected.programId) continue;
      const alias = new FontFace(requested, selected.bytes);
      try {
        await alias.load();
        if (!isCurrent()) return [];
        registerLoadedLocalFontFace(alias);
        suppliedAliases.set(requested, selected.programId);
      } catch {
        // 프로그램 조회 뒤 설치 상태가 달라지면 alias 공급을 완료로 세지 않는다.
      }
    }
  }
  // 로드한 프로그램 형식만 전달한다. 메트릭 신뢰는 Rust가 검증된 face에 한해 결정한다.
  return [...programs.values()].map(({ query, family, programId, trueType }) => ({ query, family, programId, trueType }));
}
