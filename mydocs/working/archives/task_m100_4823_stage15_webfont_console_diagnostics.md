---
title: '#4823 Stage 15 - 웹폰트 브라우저 콘솔 진단'
date: 2026-08-15
status: completed
issue: 4823
---

# #4823 Stage 15 - 웹폰트 브라우저 콘솔 진단

## 목적

Chrome DevTools에서 문서별 조건부 웹폰트 선택 결과를 확인할 수 있게 한다.

## 디버그 로그

`[FontLoader][debug]` 접두어로 다음을 `console.debug`에 출력한다.

- 시스템 글꼴 사용: 시스템에 감지되어 CDN 등록을 건너뛴 패밀리명
- CDN 후보: 문서 요청과 카탈로그 매핑을 통과한 패밀리명
- CDN 로드 시작: 같은 URL을 공유하는 패밀리 별칭과 URL
- CDN 로드 성공: 실제 `FontFace.load()`가 완료된 별칭과 URL
- CDN 로드 실패: 실패한 별칭·URL 및 오류 객체

기존의 시작·완료 요약 `console.log`는 유지한다. 세부 로그는 Chrome Console의
Verbose 수준에서 확인할 수 있다.

## 검증 범위

- 오프라인 로더 테스트가 시스템 글꼴 건너뜀과 KoPubWorld Medium CDN 성공 로그를
  각각 확인한다.
