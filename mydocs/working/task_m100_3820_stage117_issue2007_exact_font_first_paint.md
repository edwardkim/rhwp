---
kind: investigation
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 117 — issue2007 exact font first paint

## 목적

Stage 111에서 issue2007의 17쪽 구조·페이지 owner는 통과했지만 p10~p15의
`22~25%` 픽셀 차이를 단순 antialiasing으로 종결한 판정은 과도했다. 기준 PDF의
정확 글꼴과 rhwp의 실제 paint font를 식별하고, 두부문자를 재발시키지 않는 경로에서
첫 CanvasKit 렌더부터 exact local face를 사용하게 한다.

## 독립 정답지와 원인

- PDF p10~p15 content stream의 `87.0~91.0%`는 `/F2`다.
- 난독화된 BaseFont 이름을 복원하면 `휴먼명조`다.
- PDF embedded `/F2`와 `/Users/tsjang/Library/Fonts/HMKMM.TTF`는 UPM, head
  bbox, hhea metric이 같고, 표본 한글 12자의 hmtx와 decomposed outline hash도
  모두 같다.
- Stage 111 `--font-style` SVG의 실제 platform font는 `AppleMyungjo`였다.
- 같은 p10을 `export-svg --embed-fonts=full --font-path
  /Users/tsjang/Library/Fonts`로 렌더하면 raw diff가 `24.25% → 11.77%`로 감소했다.

따라서 남은 큰 차이의 상당 부분은 정확한 휴먼명조가 아닌 대체 폰트를 그린 결과다.

## 두부문자와 경로 분리

Blink CSS에서 `local("휴먼명조")`를 직접 우선하면 HMKMM의 EBDT 처리 때문에
두부문자가 생길 수 있다. 그러므로 `src/renderer/svg.rs`의 portable Style alias 순서는
바꾸지 않는다.

- portable 정적 SVG: 현재 안전 fallback 순서를 유지하며 구조 sweep에 사용
- exact 정적 증적: 허가된 로컬 폰트를 `--embed-fonts=full`로 포함
- Studio Canvas: HMKMM raw SFNT bytes를 CanvasKit font manager에 직접 등록

CanvasKit은 같은 HMKMM bytes를 `MakeFreeTypeFaceFromData` 및
`MakeTypefaceFromData`로 정상 로드했고 필요한 한글 glyph ID도 모두 nonzero였다.

## Studio first-paint 결함

`rhwp-studio/src/main.ts`의 현재 순서는 첫 document replay 전에 bundled 대체 폰트만
기다리고, `initializeDocument()`의 첫 `loadDocument()` 뒤에
`prepareCanvasKitLocalFonts()`를 fire-and-forget으로 실행한다. 그 결과 첫 스크린샷과
첫 화면은 Noto/대체 face를 사용하고 나중 rerender에서만 exact local face를 얻는다.

## 최소 변경 계약

`prepareCanvasKitDocument()`에서 첫 replay 전에 다음 순서를 await한다.

1. `loadStoredLocalFonts()`
2. `renderer.prepareLocalFonts(report.requiredFontFamilies)`
3. `renderer.prepareBundledFonts(plan.sources)`

기존 `findPreparedTypeface()`는 local face를 bundled face보다 우선하므로 renderer
내부 우선순위는 바꾸지 않는다. local 접근 또는 등록이 실패해도 bundled fallback은
그대로 남아야 한다.

## 회귀 계획

- main first-replay가 local font 준비를 await한 뒤 bundled fallback을 준비하는지 계약
- local·bundled가 모두 휴먼명조 alias일 때 local exact face 우선
- 저장된 SFNT의 family/full/PostScript 이름이 휴먼명조인 record의 원본 bytes 조회
- portable SVG Style fallback 순서 불변 및 Full mode의 HMKMM bytes 임베드
- Studio focused unit/e2e와 lint/typecheck

## 현재 증적

- [PDF와 full-embed p10 비교](../pr/assets/task_m100_3820_stage117_issue2007_exact_font_first_paint/p10_ref_vs_full_embed.png)
- [font별 확대 비교](../pr/assets/task_m100_3820_stage117_issue2007_exact_font_first_paint/p10_font_crops.png)
- [CanvasKit HMKMM raw-face 검증](../pr/assets/task_m100_3820_stage117_issue2007_exact_font_first_paint/canvaskit_hmkmm.png)

구현 및 focused 검증 진행 중이다.
