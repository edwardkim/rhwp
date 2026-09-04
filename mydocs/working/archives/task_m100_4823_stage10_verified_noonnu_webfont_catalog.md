---
title: '#4823 Stage 10 - 검증된 Noonnu 문서 글꼴 조건부 로드'
date: 2026-08-15
status: completed
issue: 4823
---

# #4823 Stage 10 - 검증된 Noonnu 문서 글꼴 조건부 로드

## 목적

문서 글꼴 조사 증적에서 웹사이트 사용 가능과 CDN 응답이 함께 확인된 글꼴을
조건부 웹폰트 카탈로그에 추가한다. 시스템에 설치된 경우에는 기존 감지 로직에 따라
네트워크 요청 없이 시스템 글꼴을 사용한다.

## 포함 글꼴

- 경기천년바탕 Bold, Regular
- 경기천년제목 Bold, Light, Medium
- 나눔바른펜
- 나눔스퀘어라운드 Bold, ExtraBold, Regular

위 이름과 URL은 `survey_korea_downloads_font_jsdelivr_20260815.tsv`에서 Noonnu의
웹사이트 사용 가능 요약 및 jsDelivr CDN 응답이 확인된 행만 사용했다.

## 구현

- 각 문서 선언명은 해당 Noonnu WOFF URL로 등록했다.
- 원본 제공이 같은 변형명은 조사 증적에 기록된 동일 URL을 공유한다.
- 외부 웹폰트 사용 안 함 설정은 기존과 동일하게 모든 Noonnu URL의 CSS 등록과
  `FontFace` 로드를 차단한다.

## 검증 범위

- 오프라인 로더 테스트에 시스템에 없는 `경기천년제목 Medium` 요청을 추가했다.
- 테스트는 해당 패밀리가 등록되고 `Title_Medium.woff`가 선택되는지를 확인한다.

## 제외 범위

- CDN 응답만 있고 원 권리자의 웹 사용 허가가 확인되지 않은 OnlineWebFonts 항목은
  자동 카탈로그에 추가하지 않는다.
