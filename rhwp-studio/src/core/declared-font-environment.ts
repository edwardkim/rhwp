import { registerLoadedLocalFontFace } from './font-loader.ts';

export interface DocumentFontSelectionPlan {
  environmentId: string | null;
  embeddedFamilies?: string[];
  requests: Array<{ source: string; primary: string[]; substitute: string[] }>;
}

interface LoadedProgram {
  query: string;
  family: string;
  programId: string;
  face: FontFace;
}

let programGeneration = 0;
const loadedPrograms = new Map<string, LoadedProgram>();
const suppliedAliases = new Map<string, string>();
const localSource = (name: string) => `local("${name.replace(/(["\\])/g, '\\$1')}")`;

async function loadLocalProgram(query: string): Promise<LoadedProgram | null> {
  const cached = loadedPrograms.get(query);
  if (cached) return cached;
  try {
    // local() 자체가 성공한 경우만 인정한다. fallback 렌더 가능 여부는 설치 증명이 아니다.
    const face = new FontFace(query, localSource(query));
    await face.load();
    const program = { query, family: query, programId: `browser-local-${++programGeneration}`, face };
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
): Promise<Array<{ query: string; family: string; programId: string }>> {
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
      const alias = new FontFace(requested, localSource(selected.query));
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
  // FontFace local()은 sfnt 형식을 공개하지 않으므로 TrueType임을 선언하지 않는다.
  return [...programs.values()].map(({ query, family, programId }) => ({ query, family, programId }));
}
