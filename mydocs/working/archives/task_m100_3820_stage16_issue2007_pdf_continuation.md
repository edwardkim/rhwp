---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-05
---

# Task #3820 Stage 16 — issue2007 HWP/PDF continuation flow reconciliation

## Stage boundary

Stage 14의 프레임 보정은 `d4118aacf`에서 닫았다. 이 Stage는 그 보정과 별개로,
`samples/basic/issue2007_nested_cell_pagination_42065.hwp`의 p10 이후 중첩 1×1
RowBreak table을 **실제 Hancom 기준 PDF**와 직접 대조하고, 다음 구현 Stage가
고쳐야 할 원인 후보를 분리한다.

다른 작업이 만든 `task_m100_3820_stage15_production_hwp_pdf_fidelity.md` 및 그 증적은
이 Stage의 범위가 아니며 수정하지 않는다.

## 기준 및 재현

- HWP: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준 PDF: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- PDF 확인: 17쪽, A4 595×842pt (`pdfinfo`)
- rhwp 확인: scoped release-test target의 현재 head, 17쪽
- 비교 방법: 기준 PDF를 **매 실행마다** Poppler 96dpi PNG(794×1123)로 새 rasterize하고,
  같은 실행의 rhwp SVG raster와 페이지별로 대조한다. 이전 review PNG·contact sheet는
  기준 이미지를 대체하지 않으며 본 Stage의 판정 근거가 아니다.

## 직접 대조 결과

fresh p10–p17 direct pair에서 다음을 확인했다.

- `d4118aacf` 이후 p11/p15의 continuation top frame, p13의 preceding-fragment
  residual, p17의 terminal `4)` source text는 구조적으로 회복됐다.
- p10 본문의 행 투영은 PDF와 거의 같은 16px cadence를 보인다. 처음 보인 큰 ink
  차이만으로 source window 또는 line-height 결함이라고 결론낼 수 없다.
- macOS native rasterizer에는 문서가 요구하는 `휴먼명조`·`Batang`이 없으며
  fontconfig는 해당 family를 fallback으로 해석한다. 따라서 이 환경의 SVG raster
  glyph/ink 차이에는 폰트 대체가 섞인다.

즉 page count 17 및 TextLine overlap 0만으로 합격할 수 없는 것은 그대로지만, 폰트가
다른 raster를 근거로 layout을 다시 보정해서는 안 된다. 이전 review PNG·contact sheet도
동일한 이유로 판정 근거가 될 수 없다.

## 원인 분리

Windows 한컴 설치에는 `HANBatang.ttf` 계열이 존재하지만, 현재 native SVG font-file
후보에서 `휴먼명조`은 `HYMJRE.TTF`와 함초롬 fallback만 찾는다. 따라서 라이선스 폰트를
저장소에 넣지 않은 상태에서 사용자 지정 font path의 한컴 설치 폰트를 발견·임베드하는
경로가 p10–p17의 PDF 직접 대조에 먼저 필요하다. Windows 검증 작업 트리는 사용자
untracked sample로 dirty여서 이 Stage에서는 빌드·수정하지 않았다.

## 수행 순서

1. 다음 Stage에서 `휴먼명조`의 local font-file 후보와 font-path/embedded SVG 경로를
   확인하고, 라이선스 폰트는 임시 검증 경로에만 둔다.
2. 같은 PDF raster와 실제 한컴 폰트를 사용한 fresh p10–p17 SVG pair를 다시 만든다.
3. 그 결과 남는 source-window·line-height·vertical-alignment 차이만 `pi=7, ci=1`
   RowBreak fragment와 안쪽 p84 1×1 table의 stored `LINE_SEG`·compose·unit cut까지
   추적한다.

## Stage 시작 상태

- 직전 commit: `d4118aacf fix: issue2007 continuation 프레임을 보정한다`
- focused regression: `issue_2007_nested_cell_pagination` 8 passed
- WASM build는 사용자가 수동으로 수행한다. 이 Stage에서는 wasm-pack build를 실행하지 않는다.
