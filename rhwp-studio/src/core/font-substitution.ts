/**
 * 폰트 치환 모듈 — web/font_substitution.js를 TypeScript로 포팅
 *
 * webhwp의 g_SubstFonts 치환 테이블 기반.
 * HWP 문서에서 사용하는 폰트 이름을 웹에서 렌더링 가능한 폰트로 변환한다.
 *
 * 3계층 해소:
 *   1. @font-face 등록 폰트 → 그대로 사용
 *   2. g_SubstFonts 치환 체인 → 등록된 폰트까지 체인 추적
 *   3. 최종 fallback → generic serif/sans-serif
 */

import { getDetectedOSFonts, REGISTERED_FONTS } from './font-loader.ts';
import { resolveLocalFont } from './local-fonts.ts';

import {
  FONT_RULE_DISPLAY_CHAIN_POLICY_IDS,
  FONT_RULE_GOVERNMENT_SUCCESSORS,
  FONT_RULE_SUBSTITUTION_TABLES,
  projectedSubstituteTargets,
} from './font-rule-runtime.ts';

interface SubstitutionTarget {
  face: string;
  type: number;
  ruleId: string;
}

// 언어별 치환 해시맵 (생성 projection을 초기화 시 1회 색인)
const SUBST_TABLES: typeof FONT_RULE_SUBSTITUTION_TABLES = FONT_RULE_SUBSTITUTION_TABLES;
const _substMaps = SUBST_TABLES.map(langTable => {
  const map = new Map<string, SubstitutionTarget>();
  for (const [srcName, srcType, dstName, dstType, ruleId] of langTable) {
    const key = srcName + '\0' + srcType;
    if (!map.has(key)) {
      map.set(key, { face: dstName, type: dstType, ruleId });
    }
  }
  return map;
});

export interface FontRuleResolution {
  fontName: string;
  ruleIds: string[];
}

// 해소 결과 캐시
const _resolveCache = new Map<string, FontRuleResolution>();
const GENERIC_FONTS = new Set(['serif', 'sans-serif', 'monospace']);

export interface FontFamilyChainOptions {
  /** 감지 승인 후 확인된 로컬 글꼴 목록. 미지정 시 저장된 감지 결과를 사용한다. */
  confirmedLocalFonts?: readonly string[];
  /** 테스트/레거시 용도: 감지 전 원본 글꼴명을 강제로 포함한다. */
  includeUnconfirmedOriginal?: boolean;
  /** Rust/HWPX가 전달한 문서 선언 substFont. successor 뒤, generic 앞에 둔다. */
  documentFallbackFamilies?: readonly string[];
}

export interface FontFamilyCandidatesDecision {
  candidates: string[];
  ruleIds: string[];
}

const GOVERNMENT_SUCCESSORS_BY_SOURCE = new Map(
  [...new Set(FONT_RULE_GOVERNMENT_SUCCESSORS.map(rule => normalizedFamilyKey(rule.sourceFace)))]
    .map(source => [source, FONT_RULE_GOVERNMENT_SUCCESSORS
      .filter(rule => normalizedFamilyKey(rule.sourceFace) === source)
      .sort((left, right) => left.order - right.order)] as const),
);

let documentFontSubstitutions = new Map<string, string[]>();

function normalizedFamilyKey(fontName: string): string {
  return fontName.normalize('NFC').replace(/\s+/g, ' ').trim().toLocaleLowerCase('en-US');
}

function isGovernmentLegacyFont(fontName: string): boolean {
  return GOVERNMENT_SUCCESSORS_BY_SOURCE.has(normalizedFamilyKey(fontName));
}

/** 새 문서가 열릴 때 이전 문서의 substFont를 남기지 않고 현재 선언으로 교체한다. */
export function setDocumentFontSubstitutions(
  substitutions: ReadonlyArray<readonly [string, string]> | undefined,
): void {
  const next = new Map<string, string[]>();
  for (const entry of substitutions ?? []) {
    if (!Array.isArray(entry) || entry.length !== 2) continue;
    const source = entry[0]?.trim();
    const substitute = entry[1]?.trim();
    if (!source || !substitute) continue;
    const key = normalizedFamilyKey(source);
    const families = next.get(key) ?? [];
    if (!families.some(existing => normalizedFamilyKey(existing) === normalizedFamilyKey(substitute))) {
      families.push(substitute);
    }
    next.set(key, families);
  }
  documentFontSubstitutions = next;
}

function confirmedFontName(
  candidates: readonly string[],
  confirmedLocalFonts: readonly string[],
): string | null {
  const confirmed = new Map(
    confirmedLocalFonts.map(fontName => [normalizedFamilyKey(fontName), fontName] as const),
  );
  for (const candidate of candidates) {
    const match = confirmed.get(normalizedFamilyKey(candidate));
    if (match) return match;
  }
  return null;
}

function localRecordCssFamily(
  requestedFontName: string,
  record: NonNullable<ReturnType<typeof resolveLocalFont>>,
): string {
  if (normalizedFamilyKey(requestedFontName) === normalizedFamilyKey(record.family)) {
    return record.family;
  }
  const style = normalizedFamilyKey(record.style);
  if (!style || style === 'regular' || style === 'normal' || style === 'r' || style === 'roman') {
    return record.family;
  }
  return record.fullName || record.family;
}

/** 정부상징 legacy 이름에서만 현재 공식 successor의 설치 face를 찾는다. */
export function resolveGovernmentFontSuccessor(
  fontName: string,
  confirmedLocalFonts?: readonly string[],
): string | null {
  return resolveGovernmentFontSuccessorWithRule(fontName, confirmedLocalFonts).fontName;
}

function resolveGovernmentFontSuccessorWithRule(
  fontName: string,
  confirmedLocalFonts?: readonly string[],
): { fontName: string | null; ruleId: string | null } {
  if (!isGovernmentLegacyFont(fontName)) return { fontName: null, ruleId: null };
  const rules = GOVERNMENT_SUCCESSORS_BY_SOURCE.get(normalizedFamilyKey(fontName)) ?? [];
  if (confirmedLocalFonts !== undefined) {
    const confirmed = new Map(
      confirmedLocalFonts.map(candidate => [normalizedFamilyKey(candidate), candidate] as const),
    );
    for (const rule of rules) {
      const match = confirmed.get(normalizedFamilyKey(rule.targetFace));
      if (match) return { fontName: match, ruleId: rule.ruleId };
    }
    return { fontName: null, ruleId: null };
  }
  for (const rule of rules) {
    const record = resolveLocalFont(rule.targetFace);
    if (record) {
      return {
        fontName: localRecordCssFamily(rule.targetFace, record),
        ruleId: rule.ruleId,
      };
    }
  }
  return { fontName: null, ruleId: null };
}

let detectedOSFontIndexCache: Map<string, string> | null = null;
let detectedOSFontIndexSize = -1;

/** 감지 결과가 늘어날 때만 다시 색인해 paint 경로의 문자열 정규화를 줄인다. */
function detectedOSFontIndex(): ReadonlyMap<string, string> {
  const detected = getDetectedOSFonts();
  if (detectedOSFontIndexCache !== null && detectedOSFontIndexSize === detected.size) {
    return detectedOSFontIndexCache;
  }
  const index = new Map<string, string>();
  for (const name of detected) index.set(normalizedFamilyKey(name), name);
  detectedOSFontIndexCache = index;
  detectedOSFontIndexSize = detected.size;
  return index;
}

/**
 * 이 호스트에 실제 face가 있는 이름인지 판정하고 CSS에 쓸 canonical 이름을 돌려준다.
 *
 * 번들 stand-in 웹폰트 등록(`REGISTERED_FONTS`)은 실제 face 보유로 보지 않는다.
 * legacy 이름은 stand-in만 있고 실제 face는 다른 이름으로 설치돼 있기 때문이다.
 */
function installedFaceName(
  fontName: string,
  confirmedLocalFonts: readonly string[] | undefined,
): string | null {
  if (!fontName) return null;
  if (confirmedLocalFonts === undefined) {
    const record = resolveLocalFont(fontName);
    if (record) return localRecordCssFamily(fontName, record);
  } else {
    const confirmed = confirmedFontName([fontName], confirmedLocalFonts);
    if (confirmed) return confirmed;
    // 호출자가 명시한 목록은 이 호출의 권위 있는 입력이다. 비어 있거나 불일치할 때
    // 이전 문서의 전역 감지 결과를 다시 사용하면 설치되지 않은 face가 선택될 수 있다.
    return null;
  }
  return detectedOSFontIndex().get(normalizedFamilyKey(fontName)) ?? null;
}

/**
 * legacy 이름의 공급이 stand-in 웹폰트뿐일 때, 이 호스트에 설치된 치환 대상 face를 찾는다.
 *
 * `한양중고딕`은 studio 공급 카탈로그에 있으므로 `resolveFont`가 체인을 타지 않고
 * 그대로 돌려준다. 그 공급은 번들 `NotoSansKR-Regular.woff2` stand-in이라 실제로
 * 설치된 `HY중고딕`이 있어도 산세리프로 그려진다. 설치 face를 체인 앞에 둬 이를 막는다.
 */
function installedSubstituteFace(
  fontName: string,
  altType: number,
  langId: number,
  confirmedLocalFonts: readonly string[] | undefined,
): { fontName: string; ruleId: string } | null {
  for (const target of projectedSubstituteTargets(fontName, altType, langId)) {
    const installed = installedFaceName(target.face, confirmedLocalFonts);
    if (installed) return { fontName: installed, ruleId: target.ruleId };
  }
  return null;
}

function quoteCssFontFamily(fontName: string): string {
  return `"${fontName.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
}

function formatCssFontFamilies(families: string[]): string {
  return families
    .map(name => GENERIC_FONTS.has(name) ? name : quoteCssFontFamily(name))
    .join(', ');
}

function pushUniqueFontFamily(families: string[], fontName: string): void {
  const name = fontName.trim();
  if (!name) return;
  const key = name.toLocaleLowerCase('en-US');
  if (families.some(existing => existing.toLocaleLowerCase('en-US') === key)) return;
  families.push(name);
}

function systemFallbackFamilies(fontName: string): string[] {
  if (GENERIC_FONTS.has(fontName)) return [fontName];
  // KoPub바탕체는 이름에 "바탕체"가 있지만 Windows BatangChe와 달리 비례폭 출판 명조다.
  if (/KoPub\s*바탕(?:체)?|KoPub\s*Batang/i.test(fontName)) {
    return ['Batang', 'AppleMyungjo', 'Noto Serif KR', 'serif'];
  }
  // Monospace 판별
  if (/굴림체|바탕체|gulimche|batangche|coding|courier/i.test(fontName)) {
    return ['GulimChe', 'D2Coding', 'Noto Sans Mono', 'monospace'];
  }
  // Serif 판별
  if (/[바탕명조궁서]|hymjre|times|palatino|georgia|batang|gungsuh/i.test(fontName)) {
    return ['Batang', 'AppleMyungjo', 'Noto Serif KR', 'serif'];
  }
  // Sans-serif (기본)
  return ['Malgun Gothic', 'Apple SD Gothic Neo', 'Noto Sans KR', 'Pretendard', 'sans-serif'];
}

/**
 * 폰트 이름을 웹에서 렌더링 가능한 폰트로 치환한다.
 *
 * @param fontName HWP 문서의 폰트 이름
 * @param altType 폰트 타입 (0=알수없음, 1=TTF, 2=HFT)
 * @param langId 언어 카테고리 (0=한국어, 1=영어, ..., 6=사용자)
 * @returns 치환된 폰트 이름
 */
export function resolveFont(fontName: string, altType: number, langId: number): string {
  return resolveFontWithRules(fontName, altType, langId).fontName;
}

export function resolveFontWithRules(
  fontName: string,
  altType: number,
  langId: number,
): FontRuleResolution {
  if (!fontName || REGISTERED_FONTS.has(fontName)) return { fontName, ruleIds: [] };

  const cacheKey = langId + '\0' + fontName + '\0' + altType;
  const cached = _resolveCache.get(cacheKey);
  if (cached !== undefined) return cached;

  const langIdx = (langId >= 0 && langId <= 6) ? langId : 0;
  const substMap = _substMaps[langIdx];

  let name = fontName;
  let type = altType || 0;

  // altType=0이면 TTF(1) 시도, 실패하면 HFT(2) 시도
  if (type === 0) {
    if (substMap.has(name + '\x001')) {
      type = 1;
    } else if (substMap.has(name + '\x002')) {
      type = 2;
    } else {
      const result = { fontName, ruleIds: [] };
      _resolveCache.set(cacheKey, result);
      return result;
    }
  }

  // 체인 추적 (최대 15단계)
  const visited = new Set<string>();
  const ruleIds: string[] = [];
  for (let i = 0; i < 15; i++) {
    if (REGISTERED_FONTS.has(name)) break;

    const key = name + '\0' + type;
    if (visited.has(key)) break;
    visited.add(key);

    const subst = substMap.get(key);
    if (!subst) break;

    name = subst.face;
    type = subst.type;
    ruleIds.push(subst.ruleId);
  }

  const result = { fontName: name, ruleIds };
  _resolveCache.set(cacheKey, result);
  return result;
}

/**
 * CSS font-family 문자열에 전 플랫폼 fallback 체인을 추가한다.
 * Windows → macOS/iOS → Android → 오픈소스 → generic
 */
export function fontFamilyWithFallback(fontName: string): string {
  if (GENERIC_FONTS.has(fontName)) {
    return fontName;
  }
  return formatCssFontFamilies([fontName, ...systemFallbackFamilies(fontName)]);
}

/**
 * 문서 원본 글꼴명을 보존하면서 표시/측정용 CSS font-family chain을 만든다.
 *
 * 순서:
 *   1. rhwp 웹폰트 또는 감지 승인 후 확인된 로컬 글꼴의 canonical CSS family
 *   2. rhwp 웹 대체 글꼴명(resolveFont 결과)
 *   3. OS/system fallback
 *   4. generic fallback
 */
export function fontFamilyChainForDisplay(
  fontName: string,
  altType = 0,
  langId = 0,
  options: FontFamilyChainOptions = {},
): string {
  return formatCssFontFamilies(fontFamilyCandidatesForDisplay(fontName, altType, langId, options));
}

/** Canvas2D 실제 설정과 trace가 공유하는 ordered CSS family 후보다. */
export function fontFamilyCandidatesForDisplay(
  fontName: string,
  altType = 0,
  langId = 0,
  options: FontFamilyChainOptions = {},
): string[] {
  return fontFamilyCandidatesForDisplayWithRules(fontName, altType, langId, options).candidates;
}

export function fontFamilyCandidatesForDisplayWithRules(
  fontName: string,
  altType = 0,
  langId = 0,
  options: FontFamilyChainOptions = {},
): FontFamilyCandidatesDecision {
  if (!fontName) return { candidates: [], ruleIds: [] };
  if (GENERIC_FONTS.has(fontName)) {
    return {
      candidates: [fontName],
      ruleIds: [FONT_RULE_DISPLAY_CHAIN_POLICY_IDS['generic-fallback']],
    };
  }

  const families: string[] = [];
  const ruleIds: string[] = [];
  const confirmedLocalFonts = options.confirmedLocalFonts ?? [];
  const confirmedLocalFontSet = new Set(
    confirmedLocalFonts.map(name => name.toLocaleLowerCase('en-US')),
  );
  const localRecord = options.confirmedLocalFonts === undefined
    ? resolveLocalFont(fontName)
    : null;
  const originalAllowed =
    options.includeUnconfirmedOriginal === true ||
    REGISTERED_FONTS.has(fontName) ||
    confirmedLocalFontSet.has(fontName.toLocaleLowerCase('en-US'));

  if (!localRecord) {
    const substitute = installedFaceName(fontName, options.confirmedLocalFonts) === null
      ? installedSubstituteFace(fontName, altType, langId, options.confirmedLocalFonts)
      : null;
    if (substitute) {
      pushUniqueFontFamily(families, substitute.fontName);
      ruleIds.push(substitute.ruleId);
      ruleIds.push(FONT_RULE_DISPLAY_CHAIN_POLICY_IDS['paint-substitute']);
    }
  }

  if (localRecord) {
    pushUniqueFontFamily(families, localRecordCssFamily(fontName, localRecord));
    ruleIds.push(FONT_RULE_DISPLAY_CHAIN_POLICY_IDS['style-fallback']);
  } else if (originalAllowed) {
    pushUniqueFontFamily(families, fontName);
    ruleIds.push(FONT_RULE_DISPLAY_CHAIN_POLICY_IDS['style-fallback']);
  }

  const governmentSuccessor = resolveGovernmentFontSuccessorWithRule(
    fontName,
    options.confirmedLocalFonts,
  );
  if (governmentSuccessor.fontName) {
    pushUniqueFontFamily(families, governmentSuccessor.fontName);
    if (governmentSuccessor.ruleId) ruleIds.push(governmentSuccessor.ruleId);
    ruleIds.push(FONT_RULE_DISPLAY_CHAIN_POLICY_IDS['official-successor']);
  }

  const resolved = resolveFontWithRules(fontName, altType, langId);
  if (resolved.fontName && resolved.fontName !== fontName) {
    pushUniqueFontFamily(families, resolved.fontName);
    ruleIds.push(...resolved.ruleIds);
    ruleIds.push(FONT_RULE_DISPLAY_CHAIN_POLICY_IDS['paint-substitute']);
  }

  const documentFallbackFamilies = options.documentFallbackFamilies
    ?? documentFontSubstitutions.get(normalizedFamilyKey(fontName))
    ?? [];
  let documentFallbackAccepted = false;
  for (const documentFallback of documentFallbackFamilies) {
    const localFallback = options.confirmedLocalFonts === undefined
      ? resolveLocalFont(documentFallback)
      : null;
    const confirmedFallback = confirmedFontName([documentFallback], confirmedLocalFonts);
    if (localFallback) {
      pushUniqueFontFamily(
        families,
        localRecordCssFamily(documentFallback, localFallback),
      );
      documentFallbackAccepted = true;
    } else if (confirmedFallback || REGISTERED_FONTS.has(documentFallback)) {
      pushUniqueFontFamily(families, confirmedFallback ?? documentFallback);
      documentFallbackAccepted = true;
    }
  }
  if (documentFallbackAccepted) {
    ruleIds.push(FONT_RULE_DISPLAY_CHAIN_POLICY_IDS['document-substitution']);
  }

  const fallbackBase = documentFallbackFamilies.at(-1)
    ?? (resolved.fontName && resolved.fontName !== fontName ? resolved.fontName : fontName);
  for (const fallback of systemFallbackFamilies(fallbackBase)) {
    pushUniqueFontFamily(families, fallback);
  }
  ruleIds.push(FONT_RULE_DISPLAY_CHAIN_POLICY_IDS['generic-fallback']);

  return {
    candidates: families,
    ruleIds: [...new Set(ruleIds.filter(Boolean))],
  };
}

/** CSS family 목록에서 큰따옴표로 감싼 이름만 순서대로 뽑는다. */
function quotedFamilyNames(familyList: string): string[] {
  return [...familyList.matchAll(/"([^"]*)"/g)].map(match => match[1]);
}

/**
 * studio 체인에 **코어가 준 face 이름들**을 합친다.
 *
 * [#6171] studio 는 코어 체인에서 primary 하나만 뽑아 자기 표로 다시 만들었다.
 * 그래서 `installed_render_font_aliases`(Rust) 가 아는 설치 face 이름이 화면에
 * 닿지 못했다 — 3146683 1쪽 `『별표 7』` 의 `『` 가 Malgun 으로 떨어져 뒤 글자와의
 * 틈이 8.00pt(한글 2020 오라클 2.50pt, rhwp PDF 2.38pt)가 되던 원인이다.
 * 코어 체인을 그대로 흘려보내는 통제 렌더에서 잉크 시작이 65.50pt 로 PDF 실측과
 * 일치하고 틈이 2.25pt 로 닫히는 것을 확인했다.
 *
 * 코어 체인(`canvas_font_family_chain`)은 face 이름을 모두 `"..."` 로 내보내고
 * 마지막에 generic fallback 을 덧붙인다. 그래서 큰따옴표 이름 중 **첫 번째를 뺀
 * 나머지**가 코어가 아는 alias/base/문서 선언 대체다. 그것을 studio 가 고른 첫
 * 이름 바로 뒤에 끼워 넣는다 — studio 가 설치 여부를 보고 앞세운 face 는 그대로
 * 1순위로 두고, 코어 지식만 보태는 순서다.
 */
function mergeCoreFaceAliases(studioChain: string, coreFamilyList: string): string {
  const coreAliases = quotedFamilyNames(coreFamilyList).slice(1);
  if (coreAliases.length === 0) return studioChain;

  const seen = new Set(quotedFamilyNames(studioChain).map(normalizedFamilyKey));
  const missing: string[] = [];
  for (const name of coreAliases) {
    const key = normalizedFamilyKey(name);
    if (!key || seen.has(key)) continue;
    seen.add(key);
    missing.push(name);
  }
  if (missing.length === 0) return studioChain;

  const inserted = missing.map(name => `"${name}"`).join(', ');
  const first = /"[^"]*"/.exec(studioChain);
  if (!first) return `${inserted}, ${studioChain}`;
  const at = first.index + first[0].length;
  return `${studioChain.slice(0, at)}, ${inserted}${studioChain.slice(at)}`;
}

/**
 * CSS font 문자열에서 font-family를 추출하여 폰트 치환을 적용한다.
 *
 * 입력: 'bold 14.5px "안상수2006가는", sans-serif'
 * 출력: 'bold 14.5px "돋움", sans-serif'
 */
export function substituteCssFontFamily(cssFont: string): string {
  const pxIdx = cssFont.indexOf('px ');
  if (pxIdx < 0) return cssFont;

  const prefix = cssFont.substring(0, pxIdx + 3);
  const familyPart = cssFont.substring(pxIdx + 3);

  const match = familyPart.match(/^"([^"]+)"/);
  if (!match) return cssFont;

  const fontName = match[1];
  const studioChain = fontFamilyChainForDisplay(fontName, 0, 0);
  return prefix + mergeCoreFaceAliases(studioChain, familyPart);
}
