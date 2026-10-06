// 언어팩 FOUC 방지 — 번들이 실행되기 전에 <html lang> 을 고른 로케일로 맞춘다.
//
// 로케일 결정은 src/i18n/resolve.ts 가 하지만 그 코드는 번들(약 1.5MB) 안에 있어 첫 페인트
// 뒤에야 돈다. 그 사이 영어 사용자는 한국어 도구 모음(폭 64px)을 보고, 번들이 로케일을
// 적용하는 순간 글자가 바뀌며 폭이 뛴다. theme-init.js 가 테마에 대해 하는 일과 같은 이유로,
// 같은 방식(동기 외부 스크립트, 확장 CSP 때문에 인라인 불가)으로 lang 만 먼저 찍는다.
//
// 결정 순서는 resolve.ts 와 같아야 한다: ?lang= → localStorage('rhwp-locale') → ko.
// 브라우저 언어는 여기서도 보지 않는다(resolve.ts 의 이유 참고).
(() => {
  const supported = ['ko', 'en'];
  const normalize = (value) => {
    if (!value) return null;
    const base = String(value).trim().toLowerCase().split(/[-_]/)[0];
    return supported.indexOf(base) >= 0 ? base : null;
  };
  let locale = null;
  try {
    locale = normalize(new URLSearchParams(location.search).get('lang'));
  } catch {
    locale = null;
  }
  if (!locale) {
    try {
      locale = normalize(localStorage.getItem('rhwp-locale'));
    } catch {
      locale = null;
    }
  }
  document.documentElement.lang = locale || 'ko';
})();
