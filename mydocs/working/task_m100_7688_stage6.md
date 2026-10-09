---
kind: working
status: completed
last_verified: 2026-10-09
---

# #7688 6단계 — CanvasKit 문서 글꼴 준비와 style 소비

화살촉 공통 구현 `0ec7eace9`를 커밋한 뒤 승인된 순서로 진행한다.
실제 Studio explicit CanvasKit의 group-drawing-02 원문에서 `함초롬바탕`이
`Noto Sans KR / unregisteredDefault`로 대체되고 bundledTypefaceCount=0이었다.
같은 원문의 Canvas2D·SVG와 한컴 Print PDF는 그 serif 원문 요청을 보존한다.
단순 font family 문자열/readiness가 실제 font bytes 등록을 입증하지 않는 반례다.

## 생산과 소비

WASM의 실제 paint 소비에서 수집한 requiredFontFamilies → canonical font 공급 계획 →
local/host/bundled Typeface 준비 → 실제 renderText/charOverlap의 face 선택과 glyph paint.
기존 main은 auto만 report를 전달하고 explicit에는 `if (!report) return`으로 공급을 건너뛴다.
선택 정책과 자원 준비를 구분하여 explicit에서도 같은 글꼴 요구 결과로 bytes를 준비한다.
없는 글꼴을 묵시적 기본 글꼴로 그리지 않고 기존 session의 Canvas2D fallback과 진단을 사용한다.
auto의 eligibility gate나 canonical 글꼴 치환 규칙은 변경하지 않는다.

style 선택은 실제 regular/bold/italic face와 합성 스타일의 소비를 별도로 확인한다.
다른 face를 준비하지 않았는데 준비한 것으로 주장하지 않으며,
번들 family의 다른 style 일치는 독립 실제 공급·출력 없이 추측으로 바꾸지 않는다.

## style의 실제 선택과 검출 증거

bundled 별칭 Map이 한 face만 보유하여 마지막 URL이 이전 face를 덮었다.
여러 face가 같은 별칭을 공급하면 실제 bytes를 TypefaceFontProvider에 묶고 Skia의
style matcher가 선택한 face를 renderText/charOverlap이 소비한다.
요청 weight/slant별 native 객체를 캐시하고 문서 변경·dispose에서 해제한다.
실제 Bold가 regular 선택과 다른 face이면 같은 윤곽선에 합성 bold를 다시 적용하지 않는다.
italic face가 없는 대조군에서는 기존 합성 italic을 보존한다. glyph fallback에는 원 face의
스타일 판정을 복제하지 않고 해당 fallback의 합성 정책을 적용한다.
하나의 face만 공급하는 경로의 기존 합성 정책은 이번 선택 대조군으로 변경하지 않는다.

독립 FontTools 측정에서 NotoSerifKR-Regular/Bold.woff2의 OS/2 weight는 400/700이다.
실제 CanvasKit binding의 matchFamilyStyle은 숫자 SkFontStyle 값을 소비한다.
같은 family에 enum 객체를 전달하면 regular로 선택되고, 숫자 400/700에서는
서로 다른 윤곽선을 선택한 것을 actual glyph width / renderPage로 확인했다.
[Skia binding 원문](https://skia.googlesource.com/skia/+/refs/heads/main/modules/canvaskit/canvaskit_bindings.cpp)과
설치된 CanvasKit API를 대조했다. 다른 layer의 paragraph/shaping 선택을 이 비교로 대체하지 않는다.

정식 `e2e/issue-7688-canvaskit-font-preparation.test.mjs`는 원문의 실제 explicit CanvasKit
준비·대체 진단과 실제 renderPage를 검사한다. 원문 paint tree의 text face/style만 지정한
자원 소비 대조군을 screen/print에서 실행한다. 실제 Regular/Bold 단독 공급의 서로 다른
출력이 독립 기대값이며 두 공급 순서의 family 선택 출력이 각각 그 대조군과 일치한다.
절대 위치·이미지 golden을 새로 고정하지 않는다. 이것은 자원 소비 계약 검사이며,
이 합성 style tree의 모든 조판이 한컴과 일치한다는 주장으로 승격하지 않는다.

같은 검사에서 수정 전에는 원문 bundledTypefaceCount=0 / unregisteredFallback=1과
`screen/pair: Regular did not consume its supplied outline`로 실패했다. 수정 후 준비 및
두 공급 순서의 screen/print outline 소비가 통과했다. 원문 font URL만 차단한 경우 기존
session이 `canvaskitResourcePreparationFailed`를 기록하고 같은 1쪽을 Canvas2D로 연다.
portable catalog에 없는 실제 준비 face를 availability에서 배제하지 않는다. 기존
host/local의 준비된 face도 실제 소비 후보로 확인하고, 다른 family가 있다고 readiness를
대신하지 않는다. auto eligibility 정책 자체는 변경하지 않았다.

## 선행 시각 범위와 실행 결과

6개 원문 전체 8쪽 Native/fresh WASM SVG sweep의 최저는 95.89322%로 모두 90% 이상이다.
실제 Native PNG screen/print도 전체 8쪽 최저 96.18681%다.
최종 실제 Studio의 두 backend는 6개 문서 첫쪽에서 전부 90% 이상이며 최저는
Canvas2D 95.53281% / CanvasKit software 94.58234%다. 원문 group, crop, equation
PNG를 직접 확인했다. 표 문서의 Studio 2·3쪽까지 이번 첫쪽 캡처로 완료 판정하지 않는다.
JS/WASM 실제 초기화 응답의 source hash를 확인했으며 WASM cc5e9c9d…는 Docker 산출물이다.
TS만 변경한 이 단계에서는 같은 WASM을 재빌드하지 않는다.

TS noEmit 및 Studio 전체 1825 PASS / 0 FAIL / 2 skipped를 확인했다.
기존 초기화 순서 검사에서 바뀐 요구 목록 변수명을 갱신하고 순서 조건은 유지했다.
최종 확대된 style/fallback 검사도 통과했다. 같은 최종 검사는 수정 전 explicit 준비 누락과
실제 Regular outline 불일치로 FAIL / 수정 후 PASS다. 실제 Italic face 대신 합성 italic을
쓰는 경계도 두 공급 순서의 screen/print에서 단독 face 대조군과 일치했다.
최종 재캡처는 main/renderer/style module 응답의 source map hash까지 실제 source와 대조한다.
Rust 필수 전체 검증 결과는 [7단계 기록](task_m100_7688_stage7.md)에서 이어 확인한다. ignored `canvaskit-font-fix/`의 e2e-before/after/final 로그와
`final/`의 Native/fresh WASM TSV·실제 PNG·Studio provenance/score를 연결한다.

실제 GPU/Windows/macOS, 모든 italic 원 face, 단일 bold-only payload의 합성 정책,
다른 equation/glyph/form 변형 전체는 이번 대표 입력으로 완결하지 않았다.
#7688은 승인된 로컬 보정의 결과와 남은 범위를 구분하며 아직 종료하거나 게시하지 않는다.
