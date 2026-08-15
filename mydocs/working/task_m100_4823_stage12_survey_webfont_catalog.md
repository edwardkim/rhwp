---
title: '#4823 Stage 12 - 조사 증적 전체 웹폰트 카탈로그'
date: 2026-08-15
status: completed
issue: 4823
---

# #4823 Stage 12 - 조사 증적 전체 웹폰트 카탈로그

## 목적

`survey_korea_downloads_font_jsdelivr_20260815.tsv`에서 웹폰트 사용 가능으로 판정된
80개 문서 글꼴 전체를 Studio의 조건부 웹폰트 카탈로그에 반영한다.

## 기준

- `status=available`과 `webfont_usable=가능`을 동시에 만족하는 행만 대상이다.
- CDN 응답만 확인되고 원 권리자의 웹 사용 허가가 보류된 `license-review` 행은 제외한다.
- TSV는 jsDelivr 검색, Noonnu CDN, Fontsource npm, jsDelivr GitHub, jsDelivr npm의
검증된 공급 경로를 포함한다.

## 구현

- `scripts/generate_document_webfont_catalog.mjs`가 TSV를 읽어 `font-loader.ts`의
  생성 구역을 갱신한다.
- 이미 수동으로 등록된 동일 패밀리명은 중복 생성하지 않는다.
- WOFF2, WOFF, TTF, OTF의 형식을 URL 확장자로 판별해 `FontFace` 형식을 맞춘다.
- 각 런타임 항목은 기존과 동일하게 문서가 해당 이름을 요청하고 시스템에 없을 때만
  등록되며, 외부 웹폰트 사용 안 함 설정에서는 전부 제외된다.

## 갱신 명령

```sh
node scripts/generate_document_webfont_catalog.mjs \
  --input mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.tsv
```

## 검증 범위

- 오프라인 로더 테스트에 생성 카탈로그 전용 항목 `62570체`를 추가했다.
- 테스트는 해당 문서 선언명이 시스템에 없을 때 `@noonnu/62570che` WOFF를 등록하는지
  확인한다.
