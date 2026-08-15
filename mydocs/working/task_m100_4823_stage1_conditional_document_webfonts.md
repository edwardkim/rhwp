# #4823 Stage 1: 문서별 조건부 웹폰트 로드

## 목표

시스템 글꼴이 없는 경우에만 문서가 실제로 요청한 글꼴의 검증된 웹폰트를 로드한다. 시작 시 전 글꼴 카탈로그를 등록하거나 조사 결과 전체를 다운로드하지 않는다.

## 근거

- 조사 입력: `mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.tsv`
- 조사 결과 1,414개 중 `available`은 19개다.
- 한컴 계열 6개와 나눔·Noto·Pretendard의 기본 face는 기존 로더가 이미 보유하거나 로컬 자산으로 제공한다.
- 조사 TSV의 일부 URL은 첫 응답 파일만 기록해 문서명에 적힌 굵기와 다르다. `@fontsource` CSS를 재확인해 700/800/500 face가 존재하는 항목만 새 URL로 추가했다.

## 변경

- `loadWebFonts()`가 `docFonts + CRITICAL_FONTS`만 먼저 OS 글꼴로 감지한다. 감지는 `document.fonts.check()`의 fallback 오인을 피하기 위해 세 generic fallback과 실제 텍스트 폭을 비교한다.
- OS 글꼴이 감지된 항목은 `@font-face` 규칙과 `FontFace.load()` 모두에서 제외한다.
- CSS 규칙은 전체 `FONT_LIST`가 아니라 실제 요청·미감지 항목만 누적 등록한다.
- 같은 파일을 공유하는 서로 다른 글꼴명은 각 이름의 `FontFace`를 지연 등록해 문서 전환 뒤에도 별칭이 누락되지 않게 한다.
- TSV 기반 신규 매핑은 다음으로 한정한다.
  - `나눔고딕 Bold` 700, `나눔고딕 ExtraBold` 800
  - `나눔명조 ExtraBold` 800, `Noto Sans KR Medium` 500
  - `DejaVu Serif` 400, `Roboto` 400
  - `나눔고딕_코딩`, `NanumGothic`은 기존 로컬 자산의 별칭이다.
- `나눔고딕 Light`는 조사 공급자에 300 face가 없어 등록하지 않는다. 유사하거나 다른 굵기의 파일로 대체하지 않고 기존 폴백을 유지한다.

## 회귀 방지

`rhwp-studio/tests/font-loader-offline-mode.test.ts`에 시스템에 존재하는 `DejaVu Serif`는 원격 등록하지 않고, 없는 `Roboto`만 조건부 등록하는 단위 검증을 추가했다.

## 검증 계획

- `npm test -- --test-name-pattern '외부 웹폰트|문서 글꼴'`
- `npm run lint`
- Studio에서 시스템 글꼴 보유·미보유 환경 각각으로 문서 로드 후 네트워크 요청을 확인한다.
