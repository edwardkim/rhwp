/**
 * 언어팩 진입점.
 *
 * 앱 시작 시 `initI18n()` 을 한 번 부르면 로케일이 결정되고 정적 마크업이 갱신된다.
 * 이후 문자열은 `t('key')` 로 얻는다.
 *
 * 기본 로케일은 'ko' 이고 ko 카탈로그는 원문을 담는다. 따라서 번역이 하나도 없어도
 * 화면은 도입 전과 같다.
 */

import koCatalog from './locales/ko.ts';
import enCatalog from './locales/en.ts';
import { getLocale, registerCatalog, setLocale, type Locale } from './core.ts';
import { applyI18nToDom } from './dom.ts';
import { collectBrowserSources, resolveLocale, storeLocale } from './resolve.ts';

export {
  DEFAULT_LOCALE,
  formatMessage,
  getEffectiveLocale,
  getLocale,
  hasTranslation,
  isSupportedLocale,
  normalizeLocale,
  t,
  type Catalog,
  type Locale,
  type MessageParams,
} from './core.ts';
export { applyI18nToDom, applyI18nToElement } from './dom.ts';
export { LOCALE_QUERY_PARAM, resolveLocale } from './resolve.ts';

// 번들 카탈로그는 모듈을 읽는 순간 등록한다.
//
// initI18n() 안에서 등록하면, 그 함수를 부르지 않는 경로(단위 테스트가 대화상자
// 모듈만 직접 부르는 경우 등)에서 t() 가 원문 대신 키를 낸다. 원문 폴백은
// 초기화 여부와 무관하게 언제나 살아 있어야 한다.
registerCatalog('ko', koCatalog as Record<string, string>);
registerCatalog('en', enCatalog as Record<string, string>);

// 로케일도 이 시점에 정한다.
//
// 모듈 최상위에서 t() 를 부르는 코드(상수 표 등)는 initI18n() 보다 먼저 평가된다.
// 그때 로케일이 아직 기본값이면 그 표만 영어 화면에서 한국어로 남는다.
// 로케일 결정은 DOM 이 필요 없으므로(질의 문자열·저장값·브라우저 설정) 여기서 해도 된다.
setLocale(resolveLocale(collectBrowserSources()));

/**
 * 정적 마크업을 결정된 로케일로 갱신한다. 두 번 불러도 안전하다.
 *
 * 로케일 자체는 모듈이 읽힐 때 이미 정해졌다. 여기서는 DOM 이 준비된 뒤에
 * 해야 하는 일만 한다.
 * @returns 적용된 로케일
 */
export function initI18n(): Locale {
  // 로케일은 위에서 모듈을 읽을 때 이미 정했다. 한 번 실행하는 동안 바뀌지 않는다(다음 실행부터 적용 —
  // setPreferredLocale). DOM 이 준비된 뒤 해야 하는 일만 한다.
  applyI18nToDom();
  return getLocale();
}

/** 언어 선택의 결과. */
export interface LocalePreferenceResult {
  locale: Locale;
  /** 지금 화면과 다른 언어를 골랐는가 — 참이면 '다음 실행부터 적용됩니다' 를 알린다. */
  appliesOnNextLaunch: boolean;
}

/**
 * 사용자가 표시 언어를 골랐을 때. **선택을 저장만 하고, 적용은 다음 실행부터다.**
 *
 * 지금 화면을 바로 바꾸지 않는 이유(이슈 #5852 검토 ①): 즉시 전환은 이미 열린 대화상자,
 * 모듈을 읽을 때 번역해 둔 상수, 동적 메뉴와 상태 표시줄까지 **현재 앱 상태로** 다시 만들어야
 * 안전하다. 정적 마크업만 다시 칠하면 `구역: 2 / 3` 같은 상태 표시가 초기 문구로 돌아가고,
 * 대화상자에는 이전 언어가 남는다. 새로고침으로 적용하는 길을 연다면 미저장 문서 보호 흐름을
 * 먼저 거쳐야 하므로 그것도 여기서 하지 않는다 — 호출하는 UI 가 결과를 보고 안내한다.
 */
export function setPreferredLocale(locale: Locale): LocalePreferenceResult {
  storeLocale(locale);
  return { locale, appliesOnNextLaunch: locale !== getLocale() };
}
