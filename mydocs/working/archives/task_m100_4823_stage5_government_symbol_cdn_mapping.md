---
title: "#4823 Stage 5 - 대한민국정부상징서체 CDN 원본 매핑"
date: 2026-08-15
issue: 4823
stage: 5
status: completed
---

# 대한민국정부상징서체 CDN 원본 매핑

## 목적

`Government_16040911` 및 `정부상징 부처명_16040911` 요청을 검증된 원본 TTF CDN으로 직접
판정한다. 이름의 공백, 밑줄, 하이픈 및 확장자 차이는 동일한 원본 글꼴 별칭으로 처리한다.

## 원본 및 배포 근거

- 원본 글꼴: `Government_16040911` Regular (`정부상징 부처명_16040911`)
- 원본 SHA-256: `9ff914274d89c97abe3c22934c1f5f049d5c82de3cf0a3bc6053ac139b8a111a`
- 저장소: <https://github.com/jangster77/korea-government-symbol-font>
- 고정 CDN: <https://cdn.jsdelivr.net/gh/jangster77/korea-government-symbol-font@v1.0.0/fonts/Government_16040911.ttf>
- 이용 조건: 공공누리 제4유형, 출처표시 + 상업적 이용금지 + 변경금지

## 변경 범위

- `scripts/survey_korea_downloads_font_jsdelivr.mjs`에 정확 별칭 전용 GitHub/jsDelivr 매핑을 추가한다.
- 원본 TTF를 `woff` 또는 `woff2`로 변환하지 않는다.
- CDN 가용성은 2026-08-15에 HTTP 200 및 `font/ttf` 응답으로 확인했다.

## 후속 검증

2026-08-15 전수 조사를 다시 실행했다. 입력 문서 10,000건의 선언 글꼴에는
`Government_16040911` 또는 `정부상징 부처명_16040911`이 없어 TSV 행은 생성되지 않았다.

대신 스크립트의 실제 `knownGovernmentSymbolCdn` 함수를 다음 입력으로 직접 호출했다.

- `Government_16040911`
- `정부상징 부처명_16040911`
- `Government_16040911.ttf`

세 입력 모두 `jangster77/korea-government-symbol-font@v1.0.0`의
`fonts/Government_16040911.ttf`, `jsDelivr GitHub`, 공공누리 제4유형 고지를 동일하게 반환했다.
고정 CDN URL은 HTTP 200 및 `font/ttf` 응답도 확인했다.
