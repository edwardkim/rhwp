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

import { REGISTERED_FONTS } from './font-loader.ts';
import { resolveLocalFont } from './local-fonts.ts';

// 치환 엔트리: [원본폰트, 원본타입, 대체폰트, 대체타입]
// 타입: 1=TTF, 2=HFT
type SubstEntry = [string, number, string, number];

// 언어별 치환 테이블 (0=한국어, 1=영어, 2=중국어, 3=일본어, 4=기타, 5=기호, 6=사용자)
const SUBST_TABLES: SubstEntry[][] = [
  // === Lang 0: 한국어 ===
  [
    ['휴먼명조',2,'휴먼명조',1],['휴먼명조',1,'HY신명조',1],
    ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
    ['명조',2,'HY견명조',1],['신명 태고딕',2,'HY중고딕',1],
    ['한양견명조',2,'HY견명조',1],['신명 태명조',2,'HY신명조',1],
    ['신명 견고딕',2,'HY견고딕',1],['신명 견명조',2,'HY견명조',1],
    ['신명 태그래픽',2,'HY그래픽',1],['신명 중고딕',2,'HY중고딕',1],
    ['태 가는 헤드라인T',2,'HY헤드라인M',1],['양재 튼튼B',2,'양재튼튼체B',1],
    ['태 가는 헤드라인D',2,'HY헤드라인M',1],['한양견고딕',2,'HY견고딕',1],
    ['Gulim',1,'굴림',1],['HYHeadLine Medium',1,'HY헤드라인M',1],
    ['Malgun Gothic',1,'맑은 고딕',1],
    ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
    ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
    ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
    ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
    ['새궁서',1,'바탕',1],
    ['궁서',1,'새궁서',1],
    ['백묵 굴림',1,'굴림',1],['백묵 돋움',1,'돋움',1],
    ['백묵 바탕',1,'바탕',1],['백묵 헤드라인',1,'돋움',1],
    ['가는안상수체',1,'함초롬돋움',1],['중간안상수체',1,'함초롬돋움',1],
    ['굵은안상수체',1,'함초롬돋움',1],['HY그래픽M',1,'HY그래픽',1],
    ['명조',2,'바탕',1],['고딕',2,'돋움',1],
    ['샘물',2,'고딕',2],['필기',2,'명조',2],['시스템',2,'고딕',2],
    ['HY둥근고딕',2,'시스템',2],['옛한글',2,'명조',2],
    ['가는공한',2,'명조',2],['중간공한',2,'명조',2],['굵은공한',2,'명조',2],
    ['가는한',2,'샘물',2],['중간한',2,'샘물',2],['굵은한',2,'샘물',2],
    ['휴먼명조',2,'옛한글',2],['휴먼고딕',2,'고딕',2],
    ['가는안상수체',2,'가는한',2],['중간안상수체',2,'중간한',2],['굵은안상수체',2,'굵은한',2],
    ['휴먼가는샘체',2,'가는한',2],['휴먼중간샘체',2,'중간한',2],['휴먼굵은샘체',2,'굵은한',2],
    ['휴먼가는팸체',2,'휴먼가는샘체',2],['휴먼중간팸체',2,'휴먼중간샘체',2],['휴먼굵은팸체',2,'휴먼굵은샘체',2],
    ['휴먼옛체',2,'휴먼고딕',2],
    ['한양신명조',2,'휴먼명조',2],['한양견명조',2,'휴먼명조',2],
    ['한양중고딕',2,'휴먼고딕',2],['한양견고딕',2,'휴먼고딕',2],
    ['한양그래픽',2,'굴림',1],['한양궁서',2,'궁서',1],
    ['문화바탕',2,'휴먼명조',2],['문화바탕제목',2,'휴먼명조',2],
    ['문화돋움',2,'휴먼고딕',2],['문화돋움제목',2,'휴먼고딕',2],
    ['문화쓰기',2,'휴먼명조',2],['문화쓰기흘림',2,'휴먼명조',2],
    ['펜흘림',2,'휴먼명조',2],['복숭아',2,'휴먼중간팸체',2],
    ['옥수수',2,'휴먼옛체',2],['오이',2,'필기',2],['가지',2,'필기',2],
    ['강낭콩',2,'한양그래픽',2],['딸기',2,'휴먼옛체',2],['타이프',2,'굵은공한',2],
    ['태 나무',2,'휴먼고딕',2],
    ['태 헤드라인D',2,'신명 견명조',2],['태 가는 헤드라인D',2,'태 헤드라인D',2],
    ['태 헤드라인T',2,'신명 견고딕',2],['태 가는 헤드라인T',2,'태 헤드라인T',2],
    ['양재 다운명조M',2,'휴먼명조',2],['양재 본목각M',2,'옥수수',2],
    ['양재 소슬',2,'태 나무',2],['양재 튼튼B',2,'태 가는 헤드라인T',2],
    ['양재 참숯B',2,'한양견고딕',2],['양재 둘기',2,'가지',2],
    ['양재 매화',2,'옥수수',2],['양재 샤넬',2,'태 나무',2],
    ['양재 와당',2,'양재 참숯B',2],['양재 이니셜',2,'양재 참숯B',2],
    ['신명 세명조',2,'휴먼명조',2],['신명 신명조',2,'휴먼명조',2],
    ['신명 신신명조',2,'휴먼명조',2],['신명 중명조',2,'휴먼명조',2],
    ['신명 태명조',2,'휴먼명조',2],['신명 견명조',2,'휴먼명조',2],
    ['신명 신문명조',2,'휴먼명조',2],['신명 순명조',2,'휴먼명조',2],
    ['신명 세고딕',2,'휴먼고딕',2],['신명 중고딕',2,'휴먼고딕',2],
    ['신명 태고딕',2,'휴먼고딕',2],['신명 견고딕',2,'휴먼고딕',2],
    ['신명 세나루',2,'휴먼고딕',2],['신명 디나루',2,'휴먼고딕',2],
    ['신명 신그래픽',2,'한양그래픽',2],['신명 태그래픽',2,'한양그래픽',2],
    ['신명 궁서',2,'한양궁서',2],['SPOQAHANSANS',1,'SpoqaHanSans',1],
  ],
  // === Lang 1: 영어 ===
  [
    ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
    ['명조',2,'HY견명조',1],['HCI Poppy',2,'Palatino Linotype',1],
    ['신명 태고딕',2,'HY중고딕',1],['산세리프',2,'Calibri',1],
    ['한양견명조',2,'HY견명조',1],['신명 태명조',2,'HY신명조',1],
    ['신명 견고딕',2,'HY견고딕',1],['신명 견명조',2,'HY견명조',1],
    ['신명 태그래픽',2,'HY그래픽',1],['신명 중고딕',2,'HY중고딕',1],
    ['양재 튼튼B',2,'양재튼튼체B',1],['한양견고딕',2,'HY견고딕',1],
    ['Gulim',1,'굴림',1],['HYHeadLine Medium',1,'HY헤드라인M',1],
    ['Malgun Gothic',1,'맑은 고딕',1],
    ['Tahoma',1,'함초롬돋움',1],['MS Sans Serif',1,'함초롬돋움',1],
    ['Times New Roman',1,'함초롬바탕',1],
    ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
    ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
    ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
    ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
    ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
    ['백묵 굴림',1,'굴림',1],['백묵 돋움',1,'돋움',1],
    ['백묵 바탕',1,'바탕',1],['백묵 헤드라인',1,'돋움',1],
    ['HY그래픽M',1,'HY그래픽',1],
    ['명조',2,'바탕',1],['고딕',2,'돋움',1],
    ['산세리프',2,'고딕',2],['필기',2,'명조',2],
    ['한양신명조',2,'명조',2],['한양중고딕',2,'고딕',2],
    ['시스템',2,'한양중고딕',2],['HY둥근고딕',2,'시스템',2],
    ['한양견명조',2,'한양신명조',2],['한양견고딕',2,'한양중고딕',2],
    ['한양그래픽',2,'굴림',1],['한양궁서',2,'궁서',1],
    ['SPOQAHANSANS',1,'SpoqaHanSans',1],
  ],
  // === Lang 2: 중국어 (축약) ===
  [
    ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
    ['명조',2,'HY견명조',1],['신명 태고딕',2,'HY중고딕',1],
    ['Gulim',1,'굴림',1],['Malgun Gothic',1,'맑은 고딕',1],
    ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
    ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
    ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
    ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
    ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
    ['명조',2,'바탕',1],['한양신명조',2,'명조',2],['한양중고딕',2,'돋움',1],
    ['SPOQAHANSANS',1,'SpoqaHanSans',1],
  ],
  // === Lang 3: 일본어 (축약) ===
  [
    ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
    ['명조',2,'HY견명조',1],['신명 태고딕',2,'HY중고딕',1],
    ['Gulim',1,'굴림',1],['Malgun Gothic',1,'맑은 고딕',1],
    ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
    ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
    ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
    ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
    ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
    ['명조',2,'바탕',1],['고딕',2,'돋움',1],
    ['한양신명조',2,'명조',2],['한양중고딕',2,'고딕',2],
    ['시스템',2,'굴림',1],['SPOQAHANSANS',1,'SpoqaHanSans',1],
  ],
  // === Lang 4: 기타 ===
  [
    ['한양신명조',2,'HY신명조',1],['명조',2,'HY견명조',1],
    ['Gulim',1,'굴림',1],['Malgun Gothic',1,'맑은 고딕',1],
    ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
    ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
    ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
    ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
    ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
    ['명조',2,'바탕',1],['한양신명조',2,'명조',2],
    ['SPOQAHANSANS',1,'SpoqaHanSans',1],
  ],
  // === Lang 5: 기호 ===
  [
    ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
    ['명조',2,'HY견명조',1],['신명 견고딕',2,'HY견고딕',1],
    ['신명 견명조',2,'HY견명조',1],['신명 태그래픽',2,'HY그래픽',1],
    ['Gulim',1,'굴림',1],['HYHeadLine Medium',1,'HY헤드라인M',1],
    ['Malgun Gothic',1,'맑은 고딕',1],
    ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
    ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
    ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
    ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
    ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
    ['명조',2,'바탕',1],['시스템',2,'명조',2],
    ['한양신명조',2,'명조',2],['한양중고딕',2,'한양신명조',2],
    ['SPOQAHANSANS',1,'SpoqaHanSans',1],
  ],
  // === Lang 6: 사용자 ===
  [
    ['한양신명조',2,'HY신명조',1],['명조',2,'HY견명조',1],
    ['Gulimche',1,'굴림체',1],['Gulim',1,'굴림',1],
    ['Malgun Gothic',1,'맑은 고딕',1],
    ['함초롬돋움',1,'함초롬바탕',1],
    ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
    ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
    ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
    ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
    ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
    ['명조',2,'바탕',1],['한글 풀어쓰기',2,'명조',2],
    ['SPOQAHANSANS',1,'SpoqaHanSans',1],
  ],
];

// 언어별 치환 해시맵 (초기화 시 1회 빌드)
const _substMaps = SUBST_TABLES.map(langTable => {
  const map = new Map<string, { face: string; type: number }>();
  for (const [srcName, srcType, dstName, dstType] of langTable) {
    const key = srcName + '\0' + srcType;
    if (!map.has(key)) {
      map.set(key, { face: dstName, type: dstType });
    }
  }
  return map;
});

// 해소 결과 캐시
const _resolveCache = new Map<string, string>();
const GENERIC_FONTS = new Set(['serif', 'sans-serif', 'monospace']);

interface FontFamilyChainOptions {
  /** 감지 승인 후 확인된 로컬 글꼴 목록. 미지정 시 저장된 감지 결과를 사용한다. */
  confirmedLocalFonts?: readonly string[];
  /** 테스트/레거시 용도: 감지 전 원본 글꼴명을 강제로 포함한다. */
  includeUnconfirmedOriginal?: boolean;
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
  if (!fontName) return fontName;
  if (REGISTERED_FONTS.has(fontName)) return fontName;

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
      _resolveCache.set(cacheKey, fontName);
      return fontName;
    }
  }

  // 체인 추적 (최대 15단계)
  const visited = new Set<string>();
  for (let i = 0; i < 15; i++) {
    if (REGISTERED_FONTS.has(name)) break;

    const key = name + '\0' + type;
    if (visited.has(key)) break;
    visited.add(key);

    const subst = substMap.get(key);
    if (!subst) break;

    name = subst.face;
    type = subst.type;
  }

  _resolveCache.set(cacheKey, name);
  return name;
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
  if (!fontName || GENERIC_FONTS.has(fontName)) return fontName;

  const families: string[] = [];
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

  if (localRecord) {
    pushUniqueFontFamily(families, localRecord.family);
  } else if (originalAllowed) {
    pushUniqueFontFamily(families, fontName);
  }

  const resolved = resolveFont(fontName, altType, langId);
  if (resolved && resolved !== fontName) {
    pushUniqueFontFamily(families, resolved);
  }

  const fallbackBase = resolved && resolved !== fontName ? resolved : fontName;
  for (const fallback of systemFallbackFamilies(fallbackBase)) {
    pushUniqueFontFamily(families, fallback);
  }

  return formatCssFontFamilies(families);
}
