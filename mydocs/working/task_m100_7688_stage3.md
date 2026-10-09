---
kind: working
status: active
last_verified: 2026-10-09
---

# #7688 3단계 — Native PNG 수식 글꼴 소비 보정

독립 기준은 `samples/eq-01.hwp`와 1단계에서 검증한 동일 원문의 한컴 2020 Print PDF다.
사용자가 Native 수식의 글꼴 보정 진행을 승인했다. source 기준은 `6f66932a73ee6fd05a74a83bdc438c9499a89a12`다.

## 구현 계약과 실제 소비

본문 `text_typeface_candidates`의 custom → system → bundled → legacy 후보 수집 본체를
`typeface_candidates_for_families`로 공유했다. 본문 후보·우선순위·문자 선택은 유지한다.
수식은 기존 math/CJK family 순서와 CJK 정체 규칙을 유지하며 web의 명조 대체 family를 포함한다.
문자별 glyph coverage는 본문과 같은 `select_typeface_for_character`를 사용한다.

`SkiaLayerRenderer::draw_layer`의 `PaintOp::Equation` 두 scale 분기에서 같은 `EquationFonts`를
전달한다. 재귀 `render_box`와 bracket의 text 분기도 이를 전달한다. 실제 `draw_text`가
선택한 동일 face의 연속 run을 만들고 그 font와 폭을 가운데 정렬·draw_str 양쪽이 사용한다.
LayoutBox의 줄 소속·원점·저장 scale과 뒤쪽 페이지 배치는 바꾸지 않았다.
전체 renderer 이관이나 sample ID/좌표 clamp 예외를 추가하지 않았다.

## 선행 실행

공용 `target/pr-review`의 수정 전 Native 바이너리를 ignored 증적에 별도로 보존했다.
첫 Native PNG에서 수식 한글 네모가 사라진 것을 직접 판독했다. 동일 PDF의 2px 실루엣은
system screen/print 96.29090%, custom Batang/Noto screen/print 96.53925%다.
저장소 Noto Sans KR Regular의 custom 공급 대조군은 95.63343%다.
이는 선행 결과이며 최종 source SHA·fresh WASM·필수 검사·정식 회귀는 아래에 이어 기록한다.

증적 루트: `output/pr-review/renderer-backend-audit-20261009/equation-font-fix/`.
공통 crop·화살촉과 다른 backend 글꼴 등록은 남은 #7688 범위다.
