---
title: "#4823 Stage 8 - 문서 요청 KoPub 조건부 웹폰트"
date: 2026-08-15
issue: 4823
stage: 8
status: completed
---

# 문서 요청 KoPub 조건부 웹폰트

## 목적

문서가 KoPub 돋움·바탕 face를 선언했지만 해당 시스템 글꼴이 없을 때, 조사 도구가 실제 파일 응답까지
검증한 `font-kopub@1.0.2` WOFF만 선별적으로 연결한다.

## 구현

- `KoPub돋움체` 및 `KoPub바탕체`의 Light, Medium, Bold 6개를 런타임 카탈로그에 등록한다.
- 기존 `loadWebFonts`의 시스템 감지 후 필터를 재사용하므로, 설치된 KoPub face는 CSS 등록과 네트워크
  요청을 모두 건너뛴다.
- 문서가 `KoPub돋움체 Medium`을 요청하고 시스템에서 찾지 못하면
  `font-kopub@1.0.2/fonts/KoPubDotum-Medium.woff`만 `FontFace`로 로드한다.
- 외부 웹폰트 사용 안 함 설정은 이 face도 계속 차단한다.

## 범위 제외

KoPubWorld는 사용 등록 조건을 애플리케이션이 사용자 대신 충족할 수 없으므로 자동 런타임 로드 대상에서
제외한다. 조사 도구의 가용성 정보와 Studio가 자동으로 요청할 수 있는 카탈로그를 분리한다.

## 회귀 범위

기존 조건부 로더 테스트에 KoPub Medium 요청을 추가해, 시스템에 있는 DejaVu는 건너뛰고 누락된
정부상징서체·KoPub만 각각의 검증된 CDN URL로 요청하는지를 확인한다.
