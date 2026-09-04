---
title: '#4823 Stage 17 - renderer baseline 문서 웹폰트 준비 순서'
date: 2026-08-15
status: completed
issue: 4823
---

# #4823 Stage 17 - renderer baseline 문서 웹폰트 준비 순서

## CI 결함

PR #4857의 Canvas visual diff에서 `table-004.hwp`는 Canvas2D의 한글 글리프가 누락되고
CanvasKit만 정상 출력됐다. baseline helper가 WASM 문서를 직접 연 뒤 Canvas를 렌더링해,
실제 Studio의 문서별 웹폰트 준비 단계를 우회한 것이 원인이다.

## 보정

- baseline helper가 production `loadWebFonts()`로 `docInfo.fontsUsed`를 준비한다.
- `document.fonts.ready`를 기다린 뒤에만 Canvas를 최초 렌더링한다.
- 순서를 정적 회귀 테스트로 고정한다.

## 범위

E2E baseline의 글꼴 준비 순서만 보정한다. Rust/WASM·renderer 구현과 기준 fixture는 변경하지 않는다.
