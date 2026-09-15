/**
 * 언어팩 조회 코어 — DOM 과 무관한 순수 로직.
 *
 * 규칙은 세 가지뿐이다.
 *  1. 기본 로케일은 'ko' 이고, ko 카탈로그는 원문을 그대로 담는다.
 *  2. 폴백은 현재 로케일 → 'ko' → 키 문자열 순이다.
 *     ko 가 원문을 담으므로 두 번째 단계에서 항상 원래 한국어가 나온다.
 *  3. 번역이 하나도 없어도 화면은 원문 그대로여야 한다(무회귀).
 */

/** 평면 키→문자열 표. 중첩하지 않는다(검색·중복검사·병합이 쉬워진다). */
export type Catalog = Readonly<Record<string, string>>;

/** 지원 로케일. 늘어나면 여기에 추가한다. */
export type Locale = 'ko' | 'en';

/** 최종 기본값. 원문 로케일이기도 하다. */
export const DEFAULT_LOCALE: Locale = 'ko';

const SUPPORTED: readonly Locale[] = ['ko', 'en'];

/**
 * 문자열 파라미터. 이름 자리표시자 `{name}` 으로 치환한다.
 *
 * `undefined` 도 받는다. 원래 코드가 템플릿 리터럴이었으므로 값이 없으면
 * 화면에 'undefined' 가 찍혔고, 언어팩이 그 동작을 바꾸면 그것도 회귀다.
 */
export type MessageParams = Readonly<Record<string, string | number | undefined>>;

const catalogs = new Map<Locale, Catalog>();

let currentLocale: Locale = DEFAULT_LOCALE;

/** 로케일 카탈로그를 등록한다. 같은 로케일을 다시 등록하면 교체한다. */
export function registerCatalog(locale: Locale, catalog: Catalog): void {
  catalogs.set(locale, catalog);
}

/** 테스트·재초기화용. 등록된 카탈로그와 현재 로케일을 초기 상태로 되돌린다. */
export function resetCatalogs(): void {
  catalogs.clear();
  currentLocale = DEFAULT_LOCALE;
}

/** 지원하는 로케일인지 판정한다. */
export function isSupportedLocale(value: unknown): value is Locale {
  return typeof value === 'string' && (SUPPORTED as readonly string[]).includes(value);
}

/**
 * 임의의 언어 태그를 지원 로케일로 좁힌다.
 * 'en-US' → 'en', 'ko-KR' → 'ko', 그 외 → null.
 */
export function normalizeLocale(tag: string | null | undefined): Locale | null {
  if (!tag) return null;
  const base = tag.trim().toLowerCase().split(/[-_]/)[0];
  return isSupportedLocale(base) ? base : null;
}

export function getLocale(): Locale {
  return currentLocale;
}

export function setLocale(locale: Locale): void {
  currentLocale = locale;
}

/** `{name}` 자리표시자를 값으로 바꾼다. 대응하는 값이 없으면 자리표시자를 그대로 둔다. */
export function formatMessage(template: string, params?: MessageParams): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (whole, name: string) => {
    // 값이 undefined 인 것과 아예 넘어오지 않은 것은 다르다.
    // 앞은 원래 코드의 `${x}` 와 같게 'undefined' 를 찍고, 뒤는 자리표시자를 남긴다.
    if (!Object.prototype.hasOwnProperty.call(params, name)) return whole;
    return String(params[name]);
  });
}

/**
 * 키를 현재 로케일 문자열로 바꾼다.
 *
 * 어떤 경우에도 예외를 던지지 않는다. 최악의 경우 키 문자열이 나오고,
 * 그 상태는 도구(check-parity)가 잡는다.
 */
export function t(key: string, params?: MessageParams): string {
  const primary = catalogs.get(currentLocale)?.[key];
  if (primary !== undefined) return formatMessage(primary, params);

  const original = catalogs.get(DEFAULT_LOCALE)?.[key];
  if (original !== undefined) return formatMessage(original, params);

  return key;
}

/**
 * 키의 원문(ko 카탈로그 값). 정적 마크업이 처음 보이는 글자와 같다. 없으면 undefined.
 * 로케일 적용이 '요소가 아직 원문을 보이고 있는지' 판단할 때 쓴다(dom.ts).
 */
export function originalText(key: string): string | undefined {
  return catalogs.get(DEFAULT_LOCALE)?.[key];
}

/** 현재 로케일에 해당 키의 번역이 실제로 있는지. 도구·디버깅용. */
export function hasTranslation(key: string, locale: Locale = currentLocale): boolean {
  return catalogs.get(locale)?.[key] !== undefined;
}

/**
 * 화면에 실제로 나타나는 언어.
 *
 * 현재 로케일의 카탈로그가 비어 있으면 화면은 폴백 때문에 원문(ko)이다.
 * `<html lang>` 같은 표시는 이 값을 따라야 한다 — 번역이 없는데 lang="en" 이라고
 * 적으면 한국어 화면을 영어라고 거짓말하는 셈이고, 글꼴 선택과 보조기술이 그 말을 믿는다.
 */
export function getEffectiveLocale(): Locale {
  const catalog = catalogs.get(currentLocale);
  if (catalog && Object.keys(catalog).length > 0) return currentLocale;
  return DEFAULT_LOCALE;
}
