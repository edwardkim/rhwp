import type { DeclaredFontProgram } from './declared-font-environment.ts';

const STORAGE_KEY = 'rhwp-document-font-preferences';
interface DocumentFontPreference { documentDigest: string; substitutions: Record<string, string> }
function parsePreference(value: unknown): DocumentFontPreference | null {
  if (!value || typeof value !== 'object') return null;
  const data = value as Partial<DocumentFontPreference>;
  if (!/^blake3:[a-f0-9]{64}$/.test(data.documentDigest ?? '')
    || !data.substitutions || typeof data.substitutions !== 'object' || Array.isArray(data.substitutions)) return null;
  const entries = Object.entries(data.substitutions);
  if (entries.length > 64 || !entries.every(([source, target]) => [source, target].every(name =>
    typeof name === 'string' && name.length > 0 && name.length <= 128 && name === name.trim()
      && !/[\x00-\x1f,;"'\\]/.test(name)))) return null;
  return { documentDigest: data.documentDigest!, substitutions: Object.fromEntries(entries) };
}

/** 명시적으로 선택한 설정만 같은 원본 문서에 보존한다. 다른 문서에는 전파하지 않는다. */
export function documentFontPreferences(digest: string | null): Record<string, string> {
  if (!digest) return {};
  try {
    const parameter = new URLSearchParams(globalThis.location?.search).get('font-environment');
    if (parameter && parameter.length <= 16384) {
      const selected = parsePreference(JSON.parse(parameter));
      if (selected?.documentDigest === digest) {
        const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}');
        const preferences = saved && typeof saved === 'object' && !Array.isArray(saved) ? saved : {};
        preferences[digest] = selected;
        localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences));
        return selected.substitutions;
      }
    }
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}');
    return parsePreference(stored?.[digest])?.substitutions ?? {};
  } catch {
    // 저장소가 차단되거나 잘못된 설정이면 원본 프로그램 선택을 유지한다.
    return {};
  }
}

/** 사용자 지정 대체도 실제 로드한 프로그램만 측정과 paint의 공통 환경에 적용한다. */
export function applyDocumentFontPreferences(
  environment: string,
  preferences: Record<string, string>,
  programs: readonly DeclaredFontProgram[],
  id: string,
  supportsSourceMetricOverrides = true,
): string {
  const selected = Object.entries(preferences).flatMap(([source, target]) => {
    const program = programs.find(program => program.query === target);
    return program ? [{ source, program }] : [];
  });
  if (!selected.length) return environment;
  const original = environment === 'null' ? {} : JSON.parse(environment);
  const result = { ...original, id,
    substitutions: { ...original.substitutions }, selectedPrograms: { ...original.selectedPrograms },
    selectionRules: { ...original.selectionRules }, trueTypeFaces: [...(original.trueTypeFaces ?? [])],
    documentSubstitutions: (original.documentSubstitutions ?? [])
      .filter((entry: { source: string }) => !selected.some(item => item.source === entry.source)),
    preserveSourceMetrics: original.preserveSourceMetrics ?? true,
    replaceSourceMetricsFor: [...new Set([...(original.replaceSourceMetricsFor ?? []),
      ...selected.map(({ source }) => source)])],
  };
  for (const { source, program } of selected) {
    result.substitutions[source] = program.family;
    result.selectedPrograms[source] = program;
    delete result.selectionRules[source];
    if (program.trueType && !result.trueTypeFaces.includes(program.family)) result.trueTypeFaces.push(program.family);
  }
  // 이전 WASM의 엄격한 JSON 계약에는 새 필드를 보내지 않는다. 새 런타임에서는
  // 선택한 원본만 변경하고, 이전 런타임은 기존 명시 환경 계약을 유지한다.
  if (!supportsSourceMetricOverrides) {
    result.preserveSourceMetrics = false;
    delete (result as { replaceSourceMetricsFor?: string[] }).replaceSourceMetricsFor;
  }
  return JSON.stringify(result);
}
