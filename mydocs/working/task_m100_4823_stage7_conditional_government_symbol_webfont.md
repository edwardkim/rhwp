---
title: "#4823 Stage 7 - 문서 요청 정부상징서체 조건부 웹폰트"
date: 2026-08-15
issue: 4823
stage: 7
status: completed
---

# 문서 요청 정부상징서체 조건부 웹폰트

## 목적

Studio가 문서를 읽은 뒤 `docInfo.fontsUsed`에 포함된 글꼴 중 시스템에서 발견되지 않은 항목만
웹폰트로 보강한다. 이번 단계는 검증된 대한민국정부상징서체 원본 TTF를 이 경로에 연결한다.

## 구현

- 문서 초기화는 이미 `loadWebFonts(docInfo.fontsUsed, ..., extensionViewerSettings)`를 호출한다.
- `loadWebFonts`는 `document.fonts` 기반 시스템 글꼴 감지를 먼저 수행한다.
- 감지된 시스템 글꼴은 CSS 등록과 `FontFace.load()`를 모두 건너뛴다.
- 미설치 `Government_16040911` 또는 `정부상징 부처명_16040911`만 아래 고정 URL을 `truetype`으로 로드한다.

```text
https://cdn.jsdelivr.net/gh/jangster77/korea-government-symbol-font@v1.0.0/fonts/Government_16040911.ttf
```

## 제약

- 외부 웹폰트 비활성화 설정은 기존처럼 외부 CDN 등록과 요청 전체를 차단한다.
- 원본 파일을 WOFF/WOFF2로 변환하지 않는다.
- 공공누리 제4유형의 출처표시, 상업적 이용금지, 변경금지 조건은 CDN 로드 후에도 유지된다.

## 회귀 범위

`font-loader-offline-mode.test.ts`는 시스템에 있는 `DejaVu Serif`를 건너뛰고, 문서에서 요청한
미설치 정부상징서체만 원본 TTF `FontFace`로 요청하는지를 검증한다.
