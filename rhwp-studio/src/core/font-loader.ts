/**
 * 웹폰트 로더 — web/editor.html의 폰트 로딩 시스템을 TypeScript로 포팅
 *
 * 2계층 로딩:
 *   1. CSS @font-face 규칙 생성 (Canvas 2D 호환)
 *   2. FontFace API로 즉시 로드 + document.fonts.add()
 */

import {
  FONT_RULE_WEBFONT_ENTRIES,
  getProjectedWebFontRuleIds,
  isCanvasKitSfntPlanned,
  normalizeProjectedFontFamily,
  projectedSubstituteTargets,
  resolveProjectedCanvasKitFontPlan,
  type ProjectedWebFontEntry,
} from './font-rule-runtime.ts';

type FontEntry = ProjectedWebFontEntry;

export interface WebFontLoadOptions {
  /** true면 CDN 등 외부 URL 웹폰트 등록/로드를 건너뛴다. */
  disableExternalWebFonts?: boolean;
}

export interface CanvasKitBundledFontSource {
  url: string;
  aliases: string[];
}

export interface CanvasKitFontPlanOptions extends WebFontLoadOptions {
  /** `fonts/` 상대 경로를 이 URL 아래의 확장/앱 자산으로 바꾼다. */
  localFontBaseUrl?: string;
  /** 배포 표면이 실제로 포함한 로컬 파일만 허용한다. 미지정 시 전체 카탈로그를 허용한다. */
  availableLocalFiles?: ReadonlySet<string>;
}

export interface CanvasKitFontPlan {
  sources: CanvasKitBundledFontSource[];
  unavailableFonts: string[];
}

export interface CanvasKitFontPlanDecision extends CanvasKitFontPlan {
  ruleIds: string[];
}

export type WebFontSupplyStatus = 'loaded' | 'registered' | 'catalogued' | 'absent';

export interface WebFontSupplySnapshot {
  status: WebFontSupplyStatus;
  canvasKitSfntPlanned: boolean;
}

export interface WebFontSupplyDecision extends WebFontSupplySnapshot {
  ruleIds: string[];
}

// Canonical registry의 Canvas2D webfont projection. CanvasKit 공급 계획과 공유하지 않는다.
const FONT_LIST: readonly FontEntry[] = FONT_RULE_WEBFONT_ENTRIES;

/** @font-face에 등록된 폰트 이름 Set */
export const REGISTERED_FONTS = new Set(FONT_LIST.map(f => f.name));

/** 초기 렌더링에 필수인 폰트 (대부분의 HWP 문서 기본 서체) */
const CRITICAL_FONTS = new Set(['함초롬바탕', '함초롬돋움']);

/** CSS @font-face에 등록한 글꼴 (문서 요청 단위로 누적) */
const registeredFontFaces: FontEntry[] = [];
const registeredFontFaceKeys = new Set<string>();

/** 한번이라도 요청한 실제 글꼴 파일 (진단용) */
const loadedFiles = new Set<string>();
/** FontFace API로 등록한 이름과 파일 조합 (별칭의 지연 등록 지원) */
const loadedFontFaceKeys = new Set<string>();

function isExternalFontFile(file: string): boolean {
  return /^https?:\/\//i.test(file);
}

function selectableFontList(options?: WebFontLoadOptions): readonly FontEntry[] {
  if (options?.disableExternalWebFonts !== true) return FONT_LIST;
  return FONT_LIST.filter(f => !isExternalFontFile(f.file));
}

function normalizeFontFamily(value: string): string {
  return normalizeProjectedFontFamily(value);
}

/** CanvasKit이 첫 replay 전에 등록해야 하는 실제 font byte source를 계산한다. */
export function resolveCanvasKitFontPlan(
  requiredFontFamilies: readonly string[],
  options: CanvasKitFontPlanOptions = {},
): CanvasKitFontPlan {
  const { sources, unavailableFonts } = resolveCanvasKitFontPlanWithRules(
    requiredFontFamilies,
    options,
  );
  return { sources, unavailableFonts };
}

/** W2 trace가 실제 generated ruleId와 같은 CanvasKit 계획을 관측하는 내부 상세 경로다. */
export function resolveCanvasKitFontPlanWithRules(
  requiredFontFamilies: readonly string[],
  options: CanvasKitFontPlanOptions = {},
): CanvasKitFontPlanDecision {
  return resolveProjectedCanvasKitFontPlan(requiredFontFamilies, options);
}

function fontFaceKey(entry: FontEntry): string {
  return [normalizeFontFamily(entry.name), entry.file, entry.format ?? 'woff2', entry.unicodeRange ?? ''].join('\u0000');
}

function isDetectedOSFont(name: string): boolean {
  return detectedOSFontFamilies.has(normalizeFontFamily(name));
}

function isRegisteredFontFamily(name: string): boolean {
  const normalized = normalizeFontFamily(name);
  return registeredFontFaces.some(entry => normalizeFontFamily(entry.name) === normalized);
}

function registerFontFaces(entries: readonly FontEntry[], options?: WebFontLoadOptions): void {
  const disableExternal = options?.disableExternalWebFonts === true;
  for (const entry of entries) {
    const key = fontFaceKey(entry);
    if (!registeredFontFaceKeys.has(key)) {
      registeredFontFaceKeys.add(key);
      registeredFontFaces.push(entry);
    }
  }

  const styleId = 'rhwp-web-font-faces';
  let style = document.getElementById(styleId) as HTMLStyleElement | null;
  if (!style) {
    style = document.createElement('style');
    style.id = styleId;
    document.head.appendChild(style);
  }
  style.textContent = registeredFontFaces
    .filter(entry => !(disableExternal && isExternalFontFile(entry.file)))
    .map(f => {
    const fmt = f.format ?? 'woff2';
    const ur = f.unicodeRange ? ` unicode-range: ${f.unicodeRange};` : '';
    return `@font-face { font-family: "${f.name}"; src: url("${f.file}") format("${fmt}"); font-display: swap;${ur} }`;
    }).join('\n');
}

/**
 * OS에 설치된 폰트인지 감지한다 (document.fonts.check 기반).
 * @font-face 등록 전에 호출해야 정확하다.
 */
const OS_FONT_CANDIDATES = [
  // Windows
  '맑은 고딕', 'Malgun Gothic', '바탕', 'Batang', '돋움', 'Dotum',
  '굴림', 'Gulim', '굴림체', 'GulimChe', '바탕체', 'BatangChe', '궁서', 'Gungsuh',
  // macOS / iOS
  'Apple SD Gothic Neo', 'AppleMyungjo', 'AppleGothic',
  // Android
  'Noto Sans KR', 'Noto Serif KR',
];
const detectedOSFonts = new Set<string>();
const detectedOSFontFamilies = new Set<string>();

/**
 * 등록 전 시스템 글꼴을 generic fallback과의 폭 비교로 감지한다.
 * `document.fonts.check()`만 사용하면 일치하는 @font-face가 없을 때도
 * fallback으로 렌더할 수 있어 설치 여부를 확정할 수 없다.
 */
function isSystemFontAvailable(name: string): boolean {
  const body = document.body;
  if (body) {
    try {
      const probe = document.createElement('span');
      probe.textContent = 'mmmmmmmmmwwwwwwwWMWMWM한글글꼴측정0123456789';
      probe.style.position = 'absolute';
      probe.style.visibility = 'hidden';
      probe.style.whiteSpace = 'nowrap';
      probe.style.fontSize = '72px';
      probe.style.fontStyle = 'normal';
      probe.style.fontWeight = 'normal';
      body.appendChild(probe);

      try {
        const genericFamilies = ['monospace', 'serif', 'sans-serif'];
        const fallbackWidths = genericFamilies.map(family => {
          probe.style.fontFamily = family;
          return probe.offsetWidth;
        });
        const escapedName = name.replace(/(["\\])/g, '\\$1');
        return genericFamilies.some((family, index) => {
          probe.style.fontFamily = `"${escapedName}", ${family}`;
          return probe.offsetWidth !== fallbackWidths[index];
        });
      } finally {
        body.removeChild(probe);
      }
    } catch {
      // body가 아직 없거나 레이아웃 측정을 지원하지 않는 surface는 기존 API로 보수적으로 처리한다.
    }
  }

  try {
    return document.fonts.check(`16px "${name}"`);
  } catch {
    return false;
  }
}

/** OS 폰트 감지 실행 (@font-face 등록 전에 호출) */
function detectOSFonts(fontNames: readonly string[]): void {
  const candidates = new Set([...OS_FONT_CANDIDATES, ...fontNames]);
  for (const name of candidates) {
    const normalized = normalizeFontFamily(name);
    if (!normalized || detectedOSFontFamilies.has(normalized) || isRegisteredFontFamily(name)) continue;
    try {
      if (isSystemFontAvailable(name)) {
        detectedOSFonts.add(name);
        detectedOSFontFamilies.add(normalized);
      }
    } catch { /* 무시 */ }
  }
  if (detectedOSFonts.size > 0) {
    console.log(`[FontLoader] OS 폰트 감지: ${Array.from(detectedOSFonts).join(', ')}`);
  }
}

/** 감지된 OS 폰트 목록 (외부 참조용) */
export function getDetectedOSFonts(): ReadonlySet<string> {
  return detectedOSFonts;
}

/** 네트워크 요청 없이 현재 웹폰트 등록/적재 상태와 SFNT 계획 가능 여부만 읽는다. */
export function getWebFontSupplySnapshot(fontName: string): WebFontSupplySnapshot {
  const { status, canvasKitSfntPlanned } = getWebFontSupplySnapshotWithRules(fontName);
  return { status, canvasKitSfntPlanned };
}

/** W2 trace용 상세 경로. 공개 snapshot의 기존 두 필드는 그대로 유지한다. */
export function getWebFontSupplySnapshotWithRules(fontName: string): WebFontSupplyDecision {
  const normalized = normalizeFontFamily(fontName);
  const entries = FONT_LIST.filter(entry => normalizeFontFamily(entry.name) === normalized);
  const loaded = entries.some(entry => loadedFontFaceKeys.has(fontFaceKey(entry)));
  const registered = entries.some(entry => registeredFontFaceKeys.has(fontFaceKey(entry)));
  return {
    status: loaded ? 'loaded' : registered ? 'registered' : entries.length > 0 ? 'catalogued' : 'absent',
    canvasKitSfntPlanned: isCanvasKitSfntPlanned(fontName),
    ruleIds: getProjectedWebFontRuleIds(fontName),
  };
}

/**
 * 웹폰트를 선별 로드한다.
 *   1단계(동기): CSS @font-face 등록
 *   2단계: 대상 폰트 로드 (이미 로드된 파일은 건너뜀)
 *
 * @param docFonts 문서에서 사용하는 폰트 이름 목록 (있으면 해당 폰트 + CRITICAL만 로드, 없으면 전체)
 * @param onProgress 폰트 로드 진행률 콜백 (loaded, total)
 * @param options 외부 웹폰트 사용 여부 등 로드 옵션
 */
export async function loadWebFonts(
  docFonts?: string[],
  onProgress?: (loaded: number, total: number) => void,
  options?: WebFontLoadOptions,
): Promise<void> {
  // 0) 문서 요청명과 필수 글꼴을 @font-face 등록 전에 감지한다.
  // 이미 등록한 face가 있으면 브라우저가 그 face를 시스템 글꼴처럼 보고할 수 있으므로
  // 해당 이름은 재감지하지 않는다.
  const targetNames = [...(docFonts ?? []), ...CRITICAL_FONTS];
  // legacy 이름(`한양중고딕`)은 stand-in 웹폰트로만 공급되므로 치환 대상(`HY중고딕`)까지
  // 감지 후보에 넣어야 설치 face를 표시 체인 앞에 둘 수 있다.
  const detectionNames = [
    ...targetNames,
    ...targetNames.flatMap(name => projectedSubstituteTargets(name).map(target => target.face)),
  ];
  detectOSFonts(detectionNames);
  const systemFontNames = [...new Set(targetNames.filter(isDetectedOSFont))];
  if (systemFontNames.length > 0) {
    console.debug(`[FontLoader][debug] 시스템 글꼴 사용: ${systemFontNames.join(', ')}`);
  }

  // 1) 문서에서 요청한 글꼴과 초기 필수 글꼴만 대상으로 삼는다.
  // 시스템 글꼴이 감지되면 CSS 규칙도 추가하지 않아 원격 face가 이를 덮어쓰지 않는다.
  const targetSet = new Set(targetNames.map(normalizeFontFamily));
  const requestedEntries = selectableFontList(options).filter(entry => (
    targetSet.has(normalizeFontFamily(entry.name)) && !isDetectedOSFont(entry.name)
  ));
  if (requestedEntries.length > 0) {
    console.debug(
      `[FontLoader][debug] CDN 후보: ${requestedEntries.map(entry => entry.name).join(', ')}`,
    );
  }
  registerFontFaces(requestedEntries, options);

  // 2) 같은 파일을 쓰더라도 새 별칭은 FontFace에 별도로 등록한다.
  // 이미 받아 둔 URL은 브라우저 캐시를 사용하므로 추가 네트워크 전송은 발생하지 않는다.
  const toLoad = requestedEntries.filter(entry => !loadedFontFaceKeys.has(fontFaceKey(entry)));
  const entriesByFile = new Map<string, FontEntry[]>();
  for (const entry of toLoad) {
    const entries = entriesByFile.get(entry.file) ?? [];
    entries.push(entry);
    entriesByFile.set(entry.file, entries);
  }
  const uniqueToLoad = [...entriesByFile.values()].map(entries => entries[0]);

  if (uniqueToLoad.length === 0) return;

  const total = uniqueToLoad.length;
  console.log(`[FontLoader] 웹폰트 로드 시작: ${total}개 파일 (이미 요청함: ${loadedFiles.size}개)`);

  let loaded = 0;
  let failed = 0;
  const BATCH = 4;

  for (let i = 0; i < uniqueToLoad.length; i += BATCH) {
    const batch = uniqueToLoad.slice(i, i + BATCH);
    await Promise.all(batch.map(async (f) => {
      const entries = entriesByFile.get(f.file) ?? [f];
      const fontNames = entries.map(entry => entry.name).join(', ');
      console.debug(`[FontLoader][debug] CDN 로드 시작: ${fontNames} <- ${f.file}`);
      try {
        for (const entry of entries) {
          const fmt = entry.format ?? 'woff2';
          const face = new FontFace(entry.name, `url(${entry.file}) format('${fmt}')`);
          const result = await face.load();
          document.fonts.add(result);
          loadedFontFaceKeys.add(fontFaceKey(entry));
        }
        loadedFiles.add(f.file);
        loaded++;
        console.debug(`[FontLoader][debug] CDN 로드 성공: ${fontNames} <- ${f.file}`);
      } catch (error) {
        failed++;
        console.debug(`[FontLoader][debug] CDN 로드 실패: ${fontNames} <- ${f.file}`, error);
      }
      onProgress?.(loaded + failed, total);
    }));
    if (i + BATCH < uniqueToLoad.length) {
      await new Promise(r => setTimeout(r, 0));
    }
  }

  console.log(`[FontLoader] 폰트 로드 완료: ${loaded}개 성공, ${failed}개 실패 (총 ${loadedFiles.size}개 파일 요청)`);
}
