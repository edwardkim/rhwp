import { registerLoadedLocalFontFace } from './font-loader.ts';
import { FONT_RULE_WEBFONT_ENTRIES } from './font-rule-runtime.ts';
import { loadLocalFontBytes, resolveLocalFont } from './local-fonts.ts';
import { normalizeFontFaceData } from './sfnt-face.ts';

export interface DocumentFontSelectionPlan {
  environmentId: string | null;
  embeddedFamilies?: string[];
  requests: Array<{ source: string; primary: string[]; substitute: string[]; publicFallback?: string[] }>;
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

export interface DeclaredFontProgram {
  query: string;
  family: string;
  programId: string;
  trueType: boolean;
}

export interface DeclaredFontProgramSnapshot {
  programs: DeclaredFontProgram[];
  /** 원본/문서 지정 이름은 존재하나 byte 조회가 승인되지 않은 경우. 공급 증명이 아니다. */
  publicFallbackBlockedSources: string[];
}

const publicPrograms = new Map<string, LoadedProgram>();

async function probeLocalName(query: string): Promise<FontFace | null> {
  // 존재 확인은 대체를 막을 뿐 glyph coverage/TrueType을 주장하지 않는다.
  try {
    const escaped = query.replaceAll('\\', '\\\\').replaceAll('"', '\\"');
    const probe = new FontFace(query, `local("${escaped}")`);
    await probe.load();
    return probe.status === 'loaded' ? probe : null;
  } catch {
    return null;
  }
}

async function loadPublicProgram(query: string): Promise<LoadedProgram | null> {
  const entries = FONT_RULE_WEBFONT_ENTRIES.filter(entry => entry.name === query
    && entry.format === 'woff2' && !entry.unicodeRange && entry.file.startsWith('fonts/'));
  if (entries.length !== 1) return null;
  const cached = publicPrograms.get(query);
  if (cached) return cached;
  try {
    const url = new URL(entries[0].file, document.baseURI);
    if (url.origin !== new URL(document.baseURI).origin) return null;
    const response = await fetch(url);
    if (!response.ok) return null;
    const bytes = await response.arrayBuffer();
    // WOFF2를 일반 SFNT 정규화기에 전달하지 않는다. 컨테이너와 실제 flavor는 별개다.
    if (bytes.byteLength < 48 || bytes.byteLength > 64 * 1024 * 1024) return null;
    const view = new DataView(bytes);
    if (view.getUint32(0) !== 0x774f4632) return null;
    const flavor = view.getUint32(4);
    const face = new FontFace(query, bytes);
    await face.load();
    const digest = await crypto.subtle.digest('SHA-256', bytes);
    const programId = `browser-canonical-${[...new Uint8Array(digest)]
      .map(byte => byte.toString(16).padStart(2, '0')).join('')}`;
    const program = { query, family: query, bytes, programId, face,
      trueType: flavor === 0x00010000 || flavor === 0x74727565 };
    publicPrograms.set(query, program);
    return program;
  } catch {
    // 공급 실패를 설치 성공이나 영구 부재로 캐시하지 않는다.
    return null;
  }
}

/** 공급과 공통 선택에 같은 실제 프로그램/존재 snapshot을 전달한다. */
export async function prepareDeclaredFontPrograms(
  plan: DocumentFontSelectionPlan,
  isCurrent: () => boolean,
): Promise<DeclaredFontProgramSnapshot> {
  const empty = (): DeclaredFontProgramSnapshot => ({ programs: [], publicFallbackBlockedSources: [] });
  const names = [...new Set(plan.requests.flatMap(request => [...request.primary, ...request.substitute]))];
  if (names.length > 1024 || typeof FontFace === 'undefined') return empty();
  const programs = new Map<string, LoadedProgram>();
  const embedded = new Set(plan.embeddedFamilies ?? []);
  for (const name of names) {
    const program = await loadLocalProgram(name);
    if (!isCurrent()) return empty();
    if (program) programs.set(name, program);
  }
  const blocked = new Set<string>();
  const presence = new Map<string, boolean>();
  for (const request of plan.requests) {
    if (!request.publicFallback?.length || embedded.has(request.source)) continue;
    const preferred = [...request.primary, ...request.substitute];
    if (preferred.some(name => programs.has(name))) continue;
    let found = false;
    for (const name of preferred) {
      let present = presence.get(name);
      if (present === undefined) {
        const localFace = embedded.has(name) ? null : await probeLocalName(name);
        if (!isCurrent()) return empty();
        present = embedded.has(name) || localFace !== null;
        // 실제 local face를 CSS에도 보존한다. 이것을 byte/TrueType snapshot에 넣지 않는다.
        if (localFace) registerLoadedLocalFontFace(localFace);
        presence.set(name, present);
      }
      if (present) { found = true; break; }
    }
    if (found) { blocked.add(request.source); continue; }
    for (const name of request.publicFallback) {
      if (embedded.has(name)) continue;
      const program = await loadPublicProgram(name);
      if (!isCurrent()) return empty();
      if (!program) continue;
      programs.set(name, program);
      // 공개 대체는 최종 family로 출력한다. 원본 이름의 CSS alias를 만들지 않아
      // 다음 문서의 실제 원본 프로그램을 이전 공개 alias가 가리지 않는다.
      registerLoadedLocalFontFace(program.face);
      break;
    }
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
        if (!isCurrent()) return empty();
        registerLoadedLocalFontFace(alias);
        suppliedAliases.set(requested, selected.programId);
      } catch {
        // 프로그램 조회 뒤 설치 상태가 달라지면 alias 공급을 완료로 세지 않는다.
      }
    }
  }
  return {
    programs: [...programs.values()].map(({ query, family, programId, trueType }) => ({ query, family, programId, trueType })),
    publicFallbackBlockedSources: [...blocked],
  };
}

/** 기존 local-only 공급 호출은 프로그램 배열 계약을 유지한다. */
export async function prepareDeclaredLocalPrograms(
  plan: DocumentFontSelectionPlan,
  isCurrent: () => boolean,
): Promise<DeclaredFontProgram[]> {
  return (await prepareDeclaredFontPrograms(plan, isCurrent)).programs;
}
