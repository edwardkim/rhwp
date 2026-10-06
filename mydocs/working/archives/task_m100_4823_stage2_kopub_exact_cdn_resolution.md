# #4823 Stage 2: KoPub 정확 CDN face 판정

## 문제

`survey_korea_downloads_font_jsdelivr_20260815.tsv`의 KoPub 기본 돋움·바탕 글꼴은 `not-found`로 남았다. 당시의 조사 산출물은 npm 전문 검색을 생략했고, 현재의 jsDelivr 탐색어만으로는 한글 family와 영문 패키지명 `font-kopub`을 안정적으로 연결하지 못한다.

## 근거

- jsDelivr `font-kopub@1.0.2` 메타데이터는 `KOPUS-Custom` 라이선스를 표기한다.
- 패키지에는 다음 WOFF가 있다.
  - `fonts/KoPubDotum-{Light,Medium,Bold}.woff`
  - `fonts/KoPubBatang-{Light,Medium,Bold}.woff`
- 패키지에는 `KoPubWorld` 또는 `_Pro` 계열의 face가 없다.

## 변경

- 기본 `KoPub돋움체`·`KoPub바탕체`와 영문 축약형 `KoPubDotum`·`KoPubBatang`의 Light/Medium/Bold만 정확한 `font-kopub` WOFF 경로로 판정한다.
- 요청 글꼴명에서 family와 굵기를 모두 추출해 해당하는 파일명 하나만 선택한다.
- `KoPubWorld`·`_Pro` 계열은 이 패키지로 발견 처리하지 않고 별도 조사 대상으로 유지한다.

## 후속 검증

스크립트를 npm 전문 검색을 활성화한 상태로 재실행해 TSV의 기본 KoPub 8개 행이 `available`과 정확한 파일 경로로 갱신되는지 확인한다.
