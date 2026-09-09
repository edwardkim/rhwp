---
title: '#4823 Stage 14 - 등록 웹폰트 로컬 face 테스트 계약'
date: 2026-08-15
status: completed
issue: 4823
---

# #4823 Stage 14 - 등록 웹폰트 로컬 face 테스트 계약

## 배경

Stage 13에서 등록 웹폰트도 시스템 설치 여부를 probe하도록 보정한 뒤, 기존 테스트의 두 번째
`detectLocalFonts()` 조회가 등록된 KoPub face를 기본 목록에서 숨겼다.

## 정정

- 기본 `detectLocalFonts()` 반환은 기존 UI 호환을 위해 `REGISTERED_FONTS`에 있는 face를
  계속 숨긴다.
- exact local face의 저장·재사용을 검증하는 테스트 조회에는 `includeRegistered: true`를
  명시해 snapshot 전체 계약을 검사한다.
- 이는 웹폰트 등록 여부와 로컬 설치 여부를 분리한 Stage 13의 동작을 바꾸지 않는다.
