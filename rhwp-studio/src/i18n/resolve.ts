/**
 * 로케일 결정 — 어디서 값을 얻을지와 우선순위만 담당한다.
 *
 * 우선순위:
 *   1. URL 질의 문자열 `?lang=`      임베드·확장이 넘겨줄 때
 *   2. localStorage 의 저장된 선택    사용자가 UI 에서 고른 값
 *   3. 'ko'                          최종 기본값
 *
 * 브라우저 언어(navigator.language)는 **기본으로 보지 않는다.**
 * 그것을 보면 영어 로케일 기계에서 편집기가 저절로 영어로 바뀐다. 사용자가 고른 적이 없는데
 * 화면이 달라지는 것도 문제지만, 영어 라벨이 한국어보다 넓어 도구 모음이 늘어나면 편집 영역이
 * 줄고 문서 쪽나눔까지 달라진다(실측으로 확인했다). CI 러너는 대개 en-US 라, 자동 감지를 켜 두면
 * 레이아웃에 민감한 e2e 가 이 변경 때문에 흔들린다.
 *
 * 그래서 영어는 **고른 사람에게만** 보인다. 자동 감지가 필요하면 `preferBrowserLanguage` 로 켠다.
 */

import { DEFAULT_LOCALE, normalizeLocale, type Locale } from './core.ts';

/**
 * 사용자가 고른 로케일을 담는 localStorage 키. 기존 설정 키와 분리해 둔다.
 * public/locale-init.js 도 같은 키를 읽는다(번들 밖이라 import 못 함) — tests/i18n-locale-init.test.ts 가 묶는다.
 */
export const LOCALE_STORAGE_KEY = 'rhwp-locale';

/** URL 질의 문자열에서 로케일을 읽을 때 쓰는 이름. */
export const LOCALE_QUERY_PARAM = 'lang';

/** 결정에 필요한 입력. 순수 함수로 시험할 수 있도록 전부 주입받는다. */
export interface LocaleSources {
  /** `location.search` 형태의 질의 문자열. */
  search?: string | null;
  /** 저장된 사용자 선택. */
  stored?: string | null;
  /** `navigator.language` 등 브라우저 선호 언어. 기본으로는 쓰지 않는다. */
  navigator?: string | null;
  /** 브라우저 언어를 따를지. 기본 false — 위 주석의 이유. */
  preferBrowserLanguage?: boolean;
}

/** 우선순위대로 첫 번째로 인식되는 값을 고른다. 없으면 기본값. */
export function resolveLocale(sources: LocaleSources): Locale {
  const fromQuery = normalizeLocale(readQueryLocale(sources.search));
  if (fromQuery) return fromQuery;

  const fromStored = normalizeLocale(sources.stored);
  if (fromStored) return fromStored;

  if (sources.preferBrowserLanguage) {
    const fromNavigator = normalizeLocale(sources.navigator);
    if (fromNavigator) return fromNavigator;
  }

  return DEFAULT_LOCALE;
}

function readQueryLocale(search: string | null | undefined): string | null {
  if (!search) return null;
  try {
    return new URLSearchParams(search).get(LOCALE_QUERY_PARAM);
  } catch {
    return null;
  }
}

/**
 * 브라우저 환경에서 실제 입력을 모은다. 저장소 접근이 막혀 있어도 실패하지 않는다.
 *
 * `navigator` 값도 함께 담지만 `preferBrowserLanguage` 를 켜지 않는 한 쓰이지 않는다.
 * 나중에 언어 선택 UI 에서 "브라우저 설정 따르기" 를 제공하고 싶을 때 쓰라고 남겨 둔다.
 */
export function collectBrowserSources(): LocaleSources {
  return {
    search: typeof location === 'undefined' ? null : location.search,
    stored: readStoredLocale(),
    navigator: typeof navigator === 'undefined' ? null : navigator.language,
  };
}

export function readStoredLocale(): string | null {
  try {
    return localStorage.getItem(LOCALE_STORAGE_KEY);
  } catch {
    return null;
  }
}

/** 사용자 선택을 저장한다. 사생활 모드 등으로 막혀 있으면 조용히 넘어간다. */
export function storeLocale(locale: Locale): void {
  try {
    localStorage.setItem(LOCALE_STORAGE_KEY, locale);
  } catch {
    /* 저장할 수 없는 환경에서도 동작은 계속되어야 한다 */
  }
}
