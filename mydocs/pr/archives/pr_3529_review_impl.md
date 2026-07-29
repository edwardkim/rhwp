---
kind: pr_review_impl
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-29
---

# PR #3529 구현 보정 기록

## 보정 내역

- HWP3 rectangle의 `0x10000000` no-fill marker와 line color의 동일 no-line marker를 IR 기본값과
  구분해, 채움·테두리를 임의로 그리지 않게 했다.
- 암호 HWP3 floating object의 reference position 1만 column origin으로 매핑하고, 일반 HWP3는
  기존 paragraph origin을 유지했다. Square-wrap line box의 문단 inset·기본 gap도 암호 원본 계약으로
  한정했다.
- `PageBackgroundImage`의 raw legacy brightness/contrast를 화면 표현용 순서로 투영하고,
  일반 `RealPic` 배경을 watermark opacity로 낮추지 않도록 SVG·Canvas·Skia를 일치시켰다.
- `scaled_canvas_extent()`는 truncation 대신 ceil을 써 A4 fractional CSS pixel의 우·하단 clipping을
  방지하고 Studio E2E로 1191×1684 bitmap을 확인했다.
- CanvasKit `PageRenderer`와 결과 비교 창도 같은 ceil bitmap 경계를 적용했다. 이는 Canvas2D의
  `794×1123`와 CanvasKit의 `793×1122` 불일치로 readiness corpus 전체가 비교 오류가 된 CI 결과를
  직접 보정한 것이며, 7개 representative corpus의 Canvas2D↔CanvasKit readiness 재현으로 확인했다.
- CI의 HWP3 drawing-group round-trip 회귀를 보정했다. 일반 HWP3의 가시 개체 control은 파싱 시
  원본 `LineInfo`·`CharShape`와 같은 marker 1칸으로 누적하고, 실제 암호 HWP3만 HWP5 변환본의
  8-unit control 계약을 사용한다. 문단 앞뒤 간격·음수 들여쓰기·추가 정보 #6 배경 이미지도 이 암호
  원본 계약에만 한정해 일반 HWP3 저장 왕복을 바꾸지 않는다.
- HWPX `HwpUnitChar`의 앞·뒤 문단 간격은 다른 margin과 같은 공통 2배 IR 스케일을 유지한다. 이를
  HWP3 암호 문서의 spacing 계약으로 전역 반감하면 SO-SUEOP HWP3/HWPX 기준 위치가 함께 회귀하므로,
  암호 HWP3 전용 보정은 HWP3 복호화 parser와 레이아웃 profile로 분리했다.

## baseline 갱신 사유

`samples/HWP5-nopassword-123456.hwp`가 field-sweep corpus에 들어온 뒤 baseline에 없던 HWP5
round-trip divergence 경로를 등록했다. 이어 HWPX `HwpUnitChar` spacing을 공통 2배 IR 스케일로
정규화하면서 `spacing_*` 발산 129건은 사라지고 `raw_header_extra` 20개 집계가 달라졌다. 현재
683 divergence path(684 TSV 행)를 덤프한 뒤 `ir_field_sweep_baseline` 2건을 다시 실행해 baseline
초과가 없음을 확인했다.

## 수용하지 않은 변경

PDF p1의 한컴 glyph와 공개 old-Hangul font의 외관 차이는 parser 원문 오류로 증명되지 않았다. 전역
`ᄒᆞᆫ → 한` 치환, font asset 교체, p3 표의 추정 보정은 이 PR에 넣지 않는다.
