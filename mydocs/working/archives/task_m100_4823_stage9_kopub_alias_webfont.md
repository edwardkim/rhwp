---
title: '#4823 Stage 9 - KoPub 영문 패밀리 별칭 조건부 웹폰트'
date: 2026-08-15
status: completed
issue: 4823
---

# #4823 Stage 9 - KoPub 영문 패밀리 별칭 조건부 웹폰트

## 목적

HWP/HWPX 문서가 같은 KoPub 서체를 한글 패밀리명 대신 `KoPubDotum` 또는
`KoPubBatang` 영문 패밀리명으로 선언해도, 시스템에 해당 이름의 글꼴이 없을 때
동일한 `font-kopub@1.0.2` 웹폰트를 연결한다.

## 구현

- `KoPubDotum`과 `KoPubBatang`의 Light, Medium, Bold 공백 표기 별칭을 등록했다.
- 공백이 없는 `KoPubDotumMedium`, `KoPubBatangMedium` 형식도 같은 자산에 등록했다.
- 각 별칭은 한글 패밀리명과 같은 WOFF URL을 공유하므로 URL 단위 로드 묶음에서
  같은 파일을 중복 요청하지 않는다.
- 기존 시스템 글꼴 감지를 그대로 사용한다. 문서가 요청한 정확한 패밀리명이 이미
  시스템에 있으면 CSS와 `FontFace` 등록 대상에서 제외된다.

## 검증 범위

- 오프라인 로더 테스트에 `KoPubBatangMedium` 문서 요청을 추가했다.
- 테스트는 한글 이름의 KoPub돋움체와 공백 없는 영문 KoPub바탕체 별칭 모두가
  정확한 WOFF 자산을 사용하도록 등록되는지를 확인한다.

## 제외 범위

- KoPubWorld는 사용 등록 조건이 있으므로 자동 웹폰트 카탈로그에 포함하지 않는다.
- 시스템에 다른 별칭으로만 설치된 동일 글꼴을 추론해 외부 로드를 생략하는 기능은
  글꼴 메타데이터의 신뢰 가능한 동치 판정이 필요하므로 다음 단계의 별도 과제로 둔다.
