---
title: '#4823 Stage 11 - KoPubWorld 조건부 웹폰트'
date: 2026-08-15
status: completed
issue: 4823
---

# #4823 Stage 11 - KoPubWorld 조건부 웹폰트

## 목적

공개 제공되는 KoPubWorld 글꼴을 문서가 요청하고 시스템에 설치되어 있지 않은 경우에만
웹폰트로 연결한다.

## 근거

- npm 패키지 `font-kopubworld@1.0.3`은 KOPUS 배포 KoPubWorld 글꼴을 제공한다.
- 패키지의 jsDelivr CSS와 파일 목록에서 Dotum·Batang의 Light, Medium, Bold WOFF2
  자산을 확인했다.
- `KoPubWorld-Dotum-Medium.woff2`의 jsDelivr 응답은 HTTP 200 및 `font/woff2`
  콘텐츠 형식을 반환한다.

## 구현

- 한글 문서 선언명 `KoPubWorld돋움체`, `KoPubWorld바탕체`의 Light, Medium, Bold를
  각 WOFF2 자산에 매핑했다.
- 패키지 CSS의 영문 기본 이름 `KoPubWorld Dotum`, `KoPubWorld Batang`과 공백 없는
  별칭도 Medium 자산에 매핑했다.
- 기존 시스템 글꼴 감지와 외부 웹폰트 사용 안 함 설정을 그대로 사용한다. 따라서
  시스템에 설치되어 있으면 CDN을 요청하지 않으며, 사용 안 함 설정에서는 등록하지 않는다.

## 검증 범위

- 오프라인 로더 테스트에 시스템에 없는 `KoPubWorld돋움체 Medium` 요청을 추가했다.
- 테스트는 해당 패밀리의 `FontFace` 등록과 정확한 Medium WOFF2 URL 선택을 확인한다.
