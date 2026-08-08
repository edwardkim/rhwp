---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 74 — 76076 p33--p36 current PDF revalidation

## 목적

사용자 관찰에서 `76076_regulatory_analysis.hwp`의 p33→p34 연속 표가 다시 잘리고,
p34 표의 문단·우측선 정합이 PDF와 다르게 보였다. Stage 70은 같은 구간을 해결됐다고
기록했지만, 과거 증적이나 기존 회귀가 현재 head의 PDF fidelity를 대신할 수 없다.

이 Stage는 현 `task/3820-production-fidelity` head에서 HWP 2020 기준 PDF와 p33--p36을
다시 직접 비교해 재발 여부와 physical owner를 확정한다. 이 문서는 분석 전용이며, 코드
변경은 원인이 source→layout→paint 하나로 좁혀진 뒤 다음 Stage에서만 수행한다.

## 입력과 정답지

| 항목 | 경로 |
| --- | --- |
| 입력 HWP | `samples/76076_regulatory_analysis.hwp` |
| PDF oracle | `samples/issue1891/76076_regulatory_analysis-2024.pdf` |
| 대상 쪽 | PDF p33--p36 |
| 이전 구현 근거 | Stage 70 `nested_table_width_scale()`의 near-fit projection 제거 |

## 판정 기준

1. p33 하단과 p34 상단의 같은 표 fragment가 PDF처럼 physical page에서 이어지고, 보이는
   문단이 쪽 경계에서 clip되지 않아야 한다.
2. p34 첫 표의 오른쪽 cell text paint가 PDF와 같이 우측 outer border 안에 남아야 한다.
3. p36의 빈 표 영역은 source fragment와 PDF owner를 직접 대조한다. 빈 영역 자체를 page
   count나 pixel proxy만으로 결함 또는 정상으로 단정하지 않는다.
4. 자동 sweep 후보는 triage일 뿐이다. `review_033`--`review_036`, render tree와 PDF panel을
   함께 확인해 실제 결함만 다음 구현 Stage로 이관한다.

## 실행 계획

1. 현 release-test binary로 180 DPI selected-page visual sweep을 실행한다.
2. SVG/render-tree의 table/cell bounds와 PDF review panel을 함께 읽어 문단 clip, border 침범,
   fragment owner를 판정한다.
3. 재발이면 정확한 table/paragraph owner와 마지막 정상/첫 비정상 line을 기록하고, 이 문서를
   먼저 커밋한 뒤 최소 layout 보정을 다음 Stage로 분리한다.
4. 재발이 아니면 새 증적으로 Stage 70의 현재 유효 범위만 갱신하고, #3820의 다른 production
   후보로 이동한다.

## 주의

- 이전 Stage 이미지와 비교하지 않고 PDF oracle을 직접 기준으로 한다.
- HWP와 HWPX의 page map을 같은 원인으로 가정하지 않는다.
- 전체 integration test는 코드 변경 후 범위에 맞게 실행한다. 이 분석 Stage의 visual 재현만으로
  baseline을 갱신하거나 회귀 통과를 주장하지 않는다.

## 현재 head 재현 결과

전용 `release-test` binary와 180 DPI direct sweep으로 PDF p33--p36을 모두 완료했다.
SVG/render tree는 문서 전체 82쪽을 생성했고, 요청한 raster/review는 4/4다.

- [p33--p36 review contact sheet](../pr/assets/task_m100_3820_stage74_76076_current_pdf_revalidation/review_contact_sheet.png)
- [p33 review](../pr/assets/task_m100_3820_stage74_76076_current_pdf_revalidation/review_033.png)
- [p34 review](../pr/assets/task_m100_3820_stage74_76076_current_pdf_revalidation/review_034.png)
- [p35 review](../pr/assets/task_m100_3820_stage74_76076_current_pdf_revalidation/review_035.png)
- [p36 review](../pr/assets/task_m100_3820_stage74_76076_current_pdf_revalidation/review_036.png)
- [summary](../pr/assets/task_m100_3820_stage74_76076_current_pdf_revalidation/summary.json),
  [page metrics](../pr/assets/task_m100_3820_stage74_76076_current_pdf_revalidation/metrics.json)

자동 sweep은 `flagged_page_count=0`을 보고했지만, p33--p36의 rhwp/PDF 패널에는 동일한
표 안 문단의 폭·줄바꿈·세로 밀도 차이가 남아 있다. 특히 p34의 첫 7×2 표는 Stage 70이
고정한 outer/nested table geometry(`x=213.7`, `w=487.6`)를 유지한다. 그러나 우측 nested
cell의 `pi=10` text line은 layout 상 `x=270.8`, `w=439.0`, 우측 외곽선 `x=710.6`으로
**0.8px 여유**만 남긴다. 실제 SVG raster는 PDF보다 넓은 글리프를 써 문단이 우측선에
닿거나 넘어가며, 이후 줄바꿈과 다음 fragment의 physical height도 PDF와 달라진다.

그러므로 Stage 70의 near-fit table-width 보정은 표 box의 선언 폭과 page owner는 회복했지만,
사용자가 관찰한 cell text paint fidelity를 해결하지 못했다. p33→p36의 나머지 표에도
같은 Hanyang Gothic text 경로가 반복돼, p34 한 줄만의 source data 결함으로 보지 않는다.

## 글꼴·paint 폭 경로

입력 HWP의 요청 face에는 `한양중고딕`이 포함된다. 현 renderer는 layout에서
`HanyangJungGothic` 실측 advance를 사용하지만 SVG CSS 체인은
`'한양중고딕' → 'Malgun Gothic' → …`이다. 이 macOS host의 `fc-match '한양중고딕'`는
그 face를 찾지 못한다. 따라서 layout이 계산한 한글 advance와 실제 Malgun glyph advance가
달라져, 순수 한글 cluster는 SVG의 폭 보정을 받지 않는 현 경로에서 누적 오차가 난다.

`svg_text_length_attrs()`는 현재 ASCII 영숫자와 반각 CJK quote에만
`textLength/lengthAdjust=spacingAndGlyphs`를 붙인다. 즉 Hanyang 실측값으로 줄을 나눈
한글 cluster는 actual fallback glyph 폭을 그대로 사용한다. Windows 한컴의 `Hancom Gothic`
font도 후보로 직접 raster 실험했지만 PDF와의 전체 p34 pixel difference가 개선되지 않아,
단순 alias 교체는 이 결함의 안전한 해법이 아니다.

## Stage 75 이관

다음 구현 Stage에서는 Hanyang measured advance와 fallback glyph paint 폭이 갈라지는
**순수 한글 cluster**에 대해서만 SVG glyph-fit contract를 적용할 수 있는지 검증한다. 모든
한글에 무차별 `textLength`를 넣지 않고, `TextStyle::glyph_fit_advance()`가 명시적으로
advance를 제공하는 Hanyang family와 table-cell text case를 focused test로 고정한다.
기준은 p34의 우측선 밖 paint 제거 및 p33--p36 PDF review 개선이며, SVG/Canvas/WASM의
동일 경로와 기존 textLength regression을 함께 확인한다.
