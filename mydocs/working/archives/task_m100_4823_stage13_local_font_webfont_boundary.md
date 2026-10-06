---
title: '#4823 Stage 13 - 로컬 글꼴과 웹폰트 카탈로그 경계 보정'
date: 2026-08-15
status: completed
issue: 4823
---

# #4823 Stage 13 - 로컬 글꼴과 웹폰트 카탈로그 경계 보정

## 배경

웹폰트 카탈로그에 KoPub과 정부상징서체를 등록한 뒤 Studio 단위 테스트에서 로컬 face
presence probe가 KoPub을 확인하지 않는 문제가 드러났다. 원인은 `REGISTERED_FONTS`를
웹 공급 가능 여부와 시스템 설치 여부에 함께 사용한 것이었다.

## 보정

- 로컬 presence probe와 미해소 후보 판정에서 `REGISTERED_FONTS` 제외 조건을 제거했다.
- 등록된 웹폰트라도 문서가 요청한 exact face는 Local Font Access 열거 결과에 없을 때
  별도 probe로 시스템 설치 여부를 확인한다.
- 따라서 시스템에 설치된 글꼴은 로컬 face로 유지하고, 설치되지 않은 경우에만 기존
  조건부 CDN 웹폰트 경로로 이어진다.

## 회귀 기대값 갱신

- KoPub바탕체는 정확한 등록 웹폰트 패밀리를 먼저 두고 비례폭 명조 fallback으로 이어진다.
- 정부상징 legacy 이름도 정확한 등록 웹폰트를 먼저 둔 뒤, ROKG successor와 문서
  대체 글꼴을 순서대로 적용한다.

## 검증 범위

- `Local Font Access가 문서 face를 누락하면 ...` 테스트는 등록된 KoPub exact face도
  probe해 로컬 레코드로 보존하는 계약을 계속 검증한다.
- `font-substitution` 테스트는 웹폰트 등록 뒤의 정확한 display chain 순서를 검증한다.
