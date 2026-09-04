---
kind: work-stage
status: active
issue: 4764
stage: 1
last_verified: 2026-08-16
---

# Stage 1 - KoPub CanvasKit SFNT 원본 연결

## 관찰

- [#4764](https://github.com/edwardkim/rhwp/issues/4764)는 #3820에서 고정한 383/82/17 페이지 수를 바꾸지
  않고 HWP 2020 PDF와의 잔여 raster fidelity를 개선한다.
- 2025 행정업무운영 편람 화면 비교에서 문서 선언 `KoPub돋움체 Light`가 CDN으로 로드됐어도 PDF와 글리프
  형태·폭이 다르게 보였다.
- CSS `FontFace`는 브라우저 Canvas2D에는 적용되지만 CanvasKit은 별도 font byte source를 fetch해 native
  `Typeface`/`FontMgr`로 준비한다. 기존 CanvasKit plan은 CSS와 같은 KoPub WOFF/KoPubWorld WOFF2를
  사용했다.

## 원인과 보정

- jsDelivr `font-kopub@1.0.2`는 같은 KoPub face의 WOFF와 TTF를 함께 제공한다. `font-kopubworld@1.0.3`는
  WOFF2와 OTF를 함께 제공한다.
- CSS는 네트워크 비용이 작은 기존 WOFF/WOFF2를 계속 사용한다. CanvasKit plan에는 `canvasKitFile`을 추가해
  FreeType/Skia가 직접 해석할 TTF/OTF SFNT 원본을 선택한다.
- 문서 글꼴명과 동의어는 같은 CanvasKit source URL로 묶되, source 비교도 CSS URL이 아닌 CanvasKit URL로
  수행한다.

## 회귀 계약

- `canvaskit-font-plan.test.ts`는 KoPub돋움·KoPub바탕·KoPubWorld돋움·KoPubWorld바탕의 CanvasKit URL이
  각각 TTF/OTF임을 고정한다.
- 기존 CSS loader 계약은 WOFF/WOFF2 URL을 계속 요청해 웹 표면의 다운로드 형식을 바꾸지 않는다.
- 이 stage는 페이지 수·Rust layout·fixture를 변경하지 않는다. 후속 browser/PDF 비교는 같은 383/82/17
  페이지 수와 #4764의 HWP 2020 PDF 대응표를 유지한다.
