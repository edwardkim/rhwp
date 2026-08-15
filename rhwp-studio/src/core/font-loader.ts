/**
 * 웹폰트 로더 — web/editor.html의 폰트 로딩 시스템을 TypeScript로 포팅
 *
 * 2계층 로딩:
 *   1. CSS @font-face 규칙 생성 (Canvas 2D 호환)
 *   2. FontFace API로 즉시 로드 + document.fonts.add()
 */

interface FontEntry {
  name: string;
  file: string;
  /** woff2(기본), woff, truetype 또는 opentype — CDN 원본 글꼴 파일용 */
  format?: 'woff2' | 'woff' | 'truetype' | 'opentype';
  /** CSS unicode-range — 지정 시 해당 코드포인트만 매칭, 다운로드도 해당 영역 사용 시에만 발생 */
  unicodeRange?: string;
}

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

// 함초롬체 CDN (눈누 jsdelivr — 비상업적 사용 허용, 한컴 라이선스)
const CDN_HAMCHOB_R = 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff';
const CDN_HAMCHOB_B = 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatangB.woff';
const CDN_HAMCHOD_R = 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff';
const CDN_FONTSOURCE = 'https://cdn.jsdelivr.net/npm/@fontsource';
// 조사 TSV에서 실제 package CSS와 글꼴 파일 응답까지 확인한 face만 추가한다.
const CDN_NANUM_GOTHIC_BOLD = `${CDN_FONTSOURCE}/nanum-gothic@5.3.0/files/nanum-gothic-0-700-normal.woff`;
const CDN_NANUM_GOTHIC_EXTRA_BOLD = `${CDN_FONTSOURCE}/nanum-gothic@5.3.0/files/nanum-gothic-0-800-normal.woff`;
const CDN_NANUM_MYEONGJO_EXTRA_BOLD = `${CDN_FONTSOURCE}/nanum-myeongjo@5.3.0/files/nanum-myeongjo-0-800-normal.woff`;
const CDN_NOTO_SANS_KR_MEDIUM = `${CDN_FONTSOURCE}/noto-sans-kr@5.3.0/files/noto-sans-kr-0-500-normal.woff`;
const CDN_DEJAVU_SERIF_REGULAR = `${CDN_FONTSOURCE}/dejavu-serif@5.3.0/files/dejavu-serif-latin-400-normal.woff2`;
const CDN_ROBOTO_REGULAR = `${CDN_FONTSOURCE}/roboto@5.3.0/files/roboto-latin-400-normal.woff2`;
const CDN_GOVERNMENT_SYMBOL_REGULAR = 'https://cdn.jsdelivr.net/gh/jangster77/korea-government-symbol-font@v1.0.0/fonts/Government_16040911.ttf';
const CDN_KOPUB = 'https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts';
const CDN_KOPUB_WORLD = 'https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts';
const CDN_GYEONGGI_MILLENNIUM = 'https://cdn.jsdelivr.net/gh/projectnoonnu/2410-3@1.0';
const CDN_NOONFONTS_TWO = 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_two@1.0';

// 한컴 webhwp CSS(@font-face) 매핑 기준 + HWP 문서에서 사용하는 별칭
const FONT_LIST: FontEntry[] = [
  // === 함초롬/함초롱/한컴 폰트 (CDN 참조) ===
  { name: '함초롬돋움', file: CDN_HAMCHOD_R, format: 'woff' },
  { name: '함초롬바탕', file: CDN_HAMCHOB_R, format: 'woff' },
  { name: '함초롱돋움', file: CDN_HAMCHOD_R, format: 'woff' },
  { name: '함초롱바탕', file: CDN_HAMCHOB_R, format: 'woff' },
  { name: '한컴돋움', file: CDN_HAMCHOD_R, format: 'woff' },
  { name: '한컴바탕', file: CDN_HAMCHOB_R, format: 'woff' },
  { name: '한컴산뜻돋움', file: CDN_HAMCHOD_R, format: 'woff' },
  { name: '새돋움', file: CDN_HAMCHOD_R, format: 'woff' },
  { name: '새바탕', file: CDN_HAMCHOB_R, format: 'woff' },
  // === 한컴 HY 폰트 → 오픈소스 대체 ===
  { name: 'HY헤드라인M', file: 'fonts/NotoSansKR-Bold.woff2' },
  { name: 'HYHeadLine M', file: 'fonts/NotoSansKR-Bold.woff2' },
  { name: 'HYHeadLine Medium', file: 'fonts/NotoSansKR-Bold.woff2' },
  { name: 'HY견고딕', file: 'fonts/NotoSansKR-Bold.woff2' },
  { name: 'HYGothic-Extra', file: 'fonts/NotoSansKR-Bold.woff2' },
  { name: 'HY그래픽', file: 'fonts/NotoSansKR-Regular.woff2' },
  { name: 'HYGraphic-Medium', file: 'fonts/NotoSansKR-Regular.woff2' },
  { name: 'HY그래픽M', file: 'fonts/NotoSansKR-Regular.woff2' },
  { name: 'HY견명조', file: 'fonts/NotoSerifKR-Bold.woff2' },
  { name: 'HYMyeongJo-Extra', file: 'fonts/NotoSerifKR-Bold.woff2' },
  { name: 'HY신명조', file: 'fonts/NotoSerifKR-Regular.woff2' },
  { name: 'HY중고딕', file: 'fonts/NotoSansKR-Regular.woff2' },
  { name: '양재튼튼체B', file: 'fonts/NotoSansKR-Bold.woff2' },
  // === 한글 시스템 폰트 → 오픈소스 대체 (OS 폰트 없을 때 폴백) ===
  { name: 'Malgun Gothic', file: 'fonts/Pretendard-Regular.woff2' },
  { name: '맑은 고딕', file: 'fonts/Pretendard-Regular.woff2' },
  // Task #1224: 한컴 돋움/MS 돋움·굴림 계열은 한컴 돋움(획 두께 페이지밀도 0.265)에
  // 근접한 Noto Sans KR ExtraLight 로 대체. 기존 NotoSansKR-Regular(밀도 0.378)는
  // 획이 +43% 두꺼워 PDF 대비 과도하게 굵게 보였다(네이티브 generic_fallback 와 정합).
  { name: '돋움', file: 'fonts/NotoSansKR-ExtraLight.woff2' },
  { name: '돋움체', file: 'fonts/NotoSansKR-ExtraLight.woff2' },
  { name: '굴림', file: 'fonts/NotoSansKR-ExtraLight.woff2' },
  { name: '굴림체', file: 'fonts/D2Coding-Regular.woff2' },
  { name: '새굴림', file: 'fonts/NotoSansKR-ExtraLight.woff2' },
  // Haansoft Dotum: HWP 문서가 직접 지정하는 한컴 돋움 영문명(예: 수능 모의고사 본문).
  // 기존 미등록 → 체인의 'Malgun Gothic'(Pretendard) 가 먼저 매칭되어 굵게 렌더됐다.
  { name: 'Haansoft Dotum', file: 'fonts/NotoSansKR-ExtraLight.woff2' },
  { name: '바탕', file: 'fonts/NotoSerifKR-Regular.woff2' },
  { name: '바탕체', file: 'fonts/D2Coding-Regular.woff2' },
  { name: '궁서', file: 'fonts/GowunBatang-Regular.woff2' },
  { name: '궁서체', file: 'fonts/GowunBatang-Regular.woff2' },
  { name: '새궁서', file: 'fonts/GowunBatang-Regular.woff2' },
  // === 나눔 폰트 (OFL, 로컬) ===
  { name: '나눔고딕', file: 'fonts/NanumGothic-Regular.woff2' },
  { name: '나눔고딕 Bold', file: CDN_NANUM_GOTHIC_BOLD, format: 'woff' },
  { name: '나눔고딕 ExtraBold', file: CDN_NANUM_GOTHIC_EXTRA_BOLD, format: 'woff' },
  { name: '나눔명조', file: 'fonts/NanumMyeongjo-Regular.woff2' },
  { name: '나눔명조 ExtraBold', file: CDN_NANUM_MYEONGJO_EXTRA_BOLD, format: 'woff' },
  { name: '나눔고딕코딩', file: 'fonts/NanumGothicCoding-Regular.woff2' },
  { name: '나눔고딕_코딩', file: 'fonts/NanumGothicCoding-Regular.woff2' },
  { name: 'NanumGothic', file: 'fonts/NanumGothic-Regular.woff2' },
  // === 영문 폰트 → OS 폴백 (번들 제거) ===
  { name: 'Palatino Linotype', file: 'fonts/NotoSerifKR-Regular.woff2' },
  // === Noto (OFL, 로컬) ===
  { name: 'Noto Sans KR', file: 'fonts/NotoSansKR-Regular.woff2' },
  { name: 'Noto Sans KR Medium', file: CDN_NOTO_SANS_KR_MEDIUM, format: 'woff' },
  // Task #1224: generic_fallback sans 체인 말단의 'Noto Sans KR ExtraLight' 해석용.
  // 미등록 고딕 문서폰트가 체인을 따라 내려올 때 무거운 Noto 직전에 ExtraLight 매칭.
  { name: 'Noto Sans KR ExtraLight', file: 'fonts/NotoSansKR-ExtraLight.woff2' },
  { name: 'Noto Serif KR', file: 'fonts/NotoSerifKR-Regular.woff2' },
  // === Pretendard ===
  { name: 'Pretendard', file: 'fonts/Pretendard-Regular.woff2' },
  { name: 'Pretendard Thin', file: 'fonts/Pretendard-Thin.woff2' },
  { name: 'Pretendard ExtraLight', file: 'fonts/Pretendard-ExtraLight.woff2' },
  { name: 'Pretendard Light', file: 'fonts/Pretendard-Light.woff2' },
  { name: 'Pretendard Medium', file: 'fonts/Pretendard-Medium.woff2' },
  { name: 'Pretendard SemiBold', file: 'fonts/Pretendard-SemiBold.woff2' },
  { name: 'Pretendard Bold', file: 'fonts/Pretendard-Bold.woff2' },
  { name: 'Pretendard ExtraBold', file: 'fonts/Pretendard-ExtraBold.woff2' },
  { name: 'Pretendard Black', file: 'fonts/Pretendard-Black.woff2' },
  // === 조사 기반 영문 글꼴 (OFL, 문서 요청 시에만 CDN 로드) ===
  { name: 'DejaVu Serif', file: CDN_DEJAVU_SERIF_REGULAR },
  { name: 'Roboto', file: CDN_ROBOTO_REGULAR },
  // === 대한민국정부상징서체 (공공누리 제4유형, 문서 요청 시에만 원본 TTF CDN 로드) ===
  { name: 'Government_16040911', file: CDN_GOVERNMENT_SYMBOL_REGULAR, format: 'truetype' },
  { name: '정부상징 부처명_16040911', file: CDN_GOVERNMENT_SYMBOL_REGULAR, format: 'truetype' },
  // === KoPub (KOPUS 공개 패키지, 문서 요청 시에만 CDN 로드) ===
  { name: 'KoPub돋움체 Light', file: `${CDN_KOPUB}/KoPubDotum-Light.woff`, format: 'woff' },
  { name: 'KoPub돋움체 Medium', file: `${CDN_KOPUB}/KoPubDotum-Medium.woff`, format: 'woff' },
  { name: 'KoPub돋움체 Bold', file: `${CDN_KOPUB}/KoPubDotum-Bold.woff`, format: 'woff' },
  { name: 'KoPub바탕체 Light', file: `${CDN_KOPUB}/KoPubBatang-Light.woff`, format: 'woff' },
  { name: 'KoPub바탕체 Medium', file: `${CDN_KOPUB}/KoPubBatang-Medium.woff`, format: 'woff' },
  { name: 'KoPub바탕체 Bold', file: `${CDN_KOPUB}/KoPubBatang-Bold.woff`, format: 'woff' },
  { name: 'KoPubDotum Light', file: `${CDN_KOPUB}/KoPubDotum-Light.woff`, format: 'woff' },
  { name: 'KoPubDotum Medium', file: `${CDN_KOPUB}/KoPubDotum-Medium.woff`, format: 'woff' },
  { name: 'KoPubDotum Bold', file: `${CDN_KOPUB}/KoPubDotum-Bold.woff`, format: 'woff' },
  { name: 'KoPubBatang Light', file: `${CDN_KOPUB}/KoPubBatang-Light.woff`, format: 'woff' },
  { name: 'KoPubBatang Medium', file: `${CDN_KOPUB}/KoPubBatang-Medium.woff`, format: 'woff' },
  { name: 'KoPubBatang Bold', file: `${CDN_KOPUB}/KoPubBatang-Bold.woff`, format: 'woff' },
  { name: 'KoPubDotumLight', file: `${CDN_KOPUB}/KoPubDotum-Light.woff`, format: 'woff' },
  { name: 'KoPubDotumMedium', file: `${CDN_KOPUB}/KoPubDotum-Medium.woff`, format: 'woff' },
  { name: 'KoPubDotumBold', file: `${CDN_KOPUB}/KoPubDotum-Bold.woff`, format: 'woff' },
  { name: 'KoPubBatangLight', file: `${CDN_KOPUB}/KoPubBatang-Light.woff`, format: 'woff' },
  { name: 'KoPubBatangMedium', file: `${CDN_KOPUB}/KoPubBatang-Medium.woff`, format: 'woff' },
  { name: 'KoPubBatangBold', file: `${CDN_KOPUB}/KoPubBatang-Bold.woff`, format: 'woff' },
  // === KoPubWorld (KOPUS 공개 글꼴, 문서 요청 시에만 CDN 로드) ===
  { name: 'KoPubWorld돋움체 Light', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Dotum-Light.woff2` },
  { name: 'KoPubWorld돋움체 Medium', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Dotum-Medium.woff2` },
  { name: 'KoPubWorld돋움체 Bold', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Dotum-Bold.woff2` },
  { name: 'KoPubWorld바탕체 Light', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Batang-Light.woff2` },
  { name: 'KoPubWorld바탕체 Medium', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Batang-Medium.woff2` },
  { name: 'KoPubWorld바탕체 Bold', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Batang-Bold.woff2` },
  { name: 'KoPubWorld Dotum', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Dotum-Medium.woff2` },
  { name: 'KoPubWorld Batang', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Batang-Medium.woff2` },
  { name: 'KoPubWorldDotum', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Dotum-Medium.woff2` },
  { name: 'KoPubWorldBatang', file: `${CDN_KOPUB_WORLD}/KoPubWorld-Batang-Medium.woff2` },
  // === Noonnu에서 웹사이트 사용 가능으로 확인된 문서 사용 글꼴 ===
  { name: '경기천년바탕 Bold', file: `${CDN_GYEONGGI_MILLENNIUM}/Batang_Regular.woff`, format: 'woff' },
  { name: '경기천년바탕 Regular', file: `${CDN_GYEONGGI_MILLENNIUM}/Batang_Regular.woff`, format: 'woff' },
  { name: '경기천년제목 Bold', file: `${CDN_GYEONGGI_MILLENNIUM}/Title_Medium.woff`, format: 'woff' },
  { name: '경기천년제목 Light', file: `${CDN_GYEONGGI_MILLENNIUM}/Title_Medium.woff`, format: 'woff' },
  { name: '경기천년제목 Medium', file: `${CDN_GYEONGGI_MILLENNIUM}/Title_Medium.woff`, format: 'woff' },
  { name: '나눔바른펜', file: `${CDN_NOONFONTS_TWO}/NanumBarunpen.woff`, format: 'woff' },
  { name: '나눔스퀘어라운드 Bold', file: `${CDN_NOONFONTS_TWO}/NanumSquareRound.woff`, format: 'woff' },
  { name: '나눔스퀘어라운드 ExtraBold', file: `${CDN_NOONFONTS_TWO}/NanumSquareRound.woff`, format: 'woff' },
  { name: '나눔스퀘어라운드 Regular', file: `${CDN_NOONFONTS_TWO}/NanumSquareRound.woff`, format: 'woff' },
  // === D2 Coding (OFL, 로컬) ===
  { name: 'D2Coding', file: 'fonts/D2Coding-Regular.woff2' },
  // === Happiness Sans ===
  { name: '해피니스 산스 레귤러', file: 'fonts/Happiness-Sans-Regular.woff2' },
  { name: 'Happiness Sans Regular', file: 'fonts/Happiness-Sans-Regular.woff2' },
  { name: '해피니스 산스 볼드', file: 'fonts/Happiness-Sans-Bold.woff2' },
  { name: 'Happiness Sans Bold', file: 'fonts/Happiness-Sans-Bold.woff2' },
  { name: '해피니스 산스 타이틀', file: 'fonts/Happiness-Sans-Title.woff2' },
  { name: 'Happiness Sans Title', file: 'fonts/Happiness-Sans-Title.woff2' },
  { name: '해피니스 산스 VF', file: 'fonts/HappinessSansVF.woff2' },
  { name: 'Happiness Sans VF', file: 'fonts/HappinessSansVF.woff2' },
  // === Cafe24 ===
  { name: 'Cafe24 Ssurround Bold', file: 'fonts/Cafe24Ssurround-v2.0.woff2' },
  { name: '카페24 슈퍼매직', file: 'fonts/Cafe24Supermagic-Regular-v1.0.woff2' },
  { name: 'Cafe24 Supermagic', file: 'fonts/Cafe24Supermagic-Regular-v1.0.woff2' },
  // === 수식 전용 폰트 (OFL/GUST, 로컬) ===
  { name: 'Latin Modern Math', file: 'fonts/LatinModernMath-Regular.woff2' },
  // === 기타 ===
  { name: 'SpoqaHanSans', file: 'fonts/SpoqaHanSans-Regular.woff2' },
  // === Gowun (OFL, 로컬) ===
  { name: '고운바탕', file: 'fonts/GowunBatang-Regular.woff2' },
  { name: '고운돋움', file: 'fonts/GowunDodum-Regular.woff2' },
  // === Source Han Serif K Old Hangul (Task #528, OFL, 로컬, 옛한글 자모 한정 subset) ===
  // PUA 옛한글 (HanCom 자체 인코딩) 을 KS X 1026-1:2007 자모 시퀀스로 변환 후
  // 합자 렌더링용. unicode-range 로 옛한글 영역에서만 매칭 → 일반 한글 영향 0.
  {
    name: 'Source Han Serif K Old Hangul',
    file: 'fonts/SourceHanSerifK-OldHangul-subset.woff2',
    unicodeRange: 'U+1100-11FF, U+A960-A97F, U+D7B0-D7FF',
  },
];

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

function selectableFontList(options?: WebFontLoadOptions): FontEntry[] {
  if (options?.disableExternalWebFonts !== true) return FONT_LIST;
  return FONT_LIST.filter(f => !isExternalFontFile(f.file));
}

function normalizeFontFamily(value: string): string {
  return value
    .replace(/\u0000/g, '')
    .normalize('NFC')
    .replace(/\s+/g, ' ')
    .trim()
    .toLocaleLowerCase('en-US');
}

function canvasKitFontUrl(file: string, localFontBaseUrl?: string): string {
  if (isExternalFontFile(file) || !localFontBaseUrl) return file;
  const base = localFontBaseUrl.replace(/\/+$/, '');
  return `${base}/${file.replace(/^fonts\//, '')}`;
}

/** CanvasKit이 첫 replay 전에 등록해야 하는 실제 font byte source를 계산한다. */
export function resolveCanvasKitFontPlan(
  requiredFontFamilies: readonly string[],
  options: CanvasKitFontPlanOptions = {},
): CanvasKitFontPlan {
  const canvasKitSubstitutes = new Map([
    [normalizeFontFamily('휴먼명조'), normalizeFontFamily('HY신명조')],
    [normalizeFontFamily('한양중고딕'), normalizeFontFamily('HY중고딕')],
    [normalizeFontFamily('한컴 윤고딕 230'), normalizeFontFamily('Noto Sans KR ExtraLight')],
  ]);
  const entriesByFamily = new Map<string, FontEntry>();
  for (const entry of FONT_LIST) {
    entriesByFamily.set(normalizeFontFamily(entry.name), entry);
  }

  const sourcesByUrl = new Map<string, Set<string>>();
  const unavailableFonts = new Map<string, string>();
  const requiredEntries: Array<{ entry: FontEntry; requested: string }> = [];
  for (const requested of requiredFontFamilies) {
    const normalized = normalizeFontFamily(requested);
    if (!normalized) continue;
    const entry = entriesByFamily.get(normalized)
      ?? entriesByFamily.get(canvasKitSubstitutes.get(normalized) ?? '');
    if (!entry) {
      unavailableFonts.set(normalized, requested.trim());
      continue;
    }
    const localFile = entry.file.startsWith('fonts/')
      ? entry.file.slice('fonts/'.length)
      : null;
    const unavailable = (options.disableExternalWebFonts === true && isExternalFontFile(entry.file))
      || (localFile !== null
        && options.availableLocalFiles !== undefined
        && !options.availableLocalFiles.has(localFile));
    if (unavailable) {
      unavailableFonts.set(normalized, requested.trim());
      continue;
    }
    requiredEntries.push({ entry, requested: requested.trim() });
  }

  for (const { entry, requested } of requiredEntries) {
    const url = canvasKitFontUrl(entry.file, options.localFontBaseUrl);
    const aliases = sourcesByUrl.get(url) ?? new Set<string>();
    aliases.add(requested);
    for (const candidate of FONT_LIST) {
      if (candidate.file === entry.file) aliases.add(candidate.name);
    }
    sourcesByUrl.set(url, aliases);
  }

  return {
    sources: [...sourcesByUrl.entries()].map(([url, aliases]) => ({
      url,
      aliases: [...aliases].sort((left, right) => left.localeCompare(right, 'ko')),
    })),
    unavailableFonts: [...unavailableFonts.values()]
      .sort((left, right) => left.localeCompare(right, 'ko')),
  };
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
  detectOSFonts(targetNames);

  // 1) 문서에서 요청한 글꼴과 초기 필수 글꼴만 대상으로 삼는다.
  // 시스템 글꼴이 감지되면 CSS 규칙도 추가하지 않아 원격 face가 이를 덮어쓰지 않는다.
  const targetSet = new Set(targetNames.map(normalizeFontFamily));
  const requestedEntries = selectableFontList(options).filter(entry => (
    targetSet.has(normalizeFontFamily(entry.name)) && !isDetectedOSFont(entry.name)
  ));
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
      try {
        const entries = entriesByFile.get(f.file) ?? [f];
        for (const entry of entries) {
          const fmt = entry.format ?? 'woff2';
          const face = new FontFace(entry.name, `url(${entry.file}) format('${fmt}')`);
          const result = await face.load();
          document.fonts.add(result);
          loadedFontFaceKeys.add(fontFaceKey(entry));
        }
        loadedFiles.add(f.file);
        loaded++;
      } catch {
        failed++;
      }
      onProgress?.(loaded + failed, total);
    }));
    if (i + BATCH < uniqueToLoad.length) {
      await new Promise(r => setTimeout(r, 0));
    }
  }

  console.log(`[FontLoader] 폰트 로드 완료: ${loaded}개 성공, ${failed}개 실패 (총 ${loadedFiles.size}개 파일 요청)`);
}
