# Task #3302 Stage 1 — 수행계획서

## 목표

skia(PNG/PDF) 백엔드에서 1bpp BMP 등 skia 미지원 이미지가 회색 placeholder 로 렌더되는
결함(#3302)을 수정한다. 기준 증상: `SO-SUEOP.hwp` 1쪽 그림(1bpp 흑백 BMP 스캔, 180×602).

## 확정된 진단 (이슈 #3302 코멘트, 2026-07-25)

- 파싱·데이터 적재·이름 매칭 정상. 도형 오인식 아님(Picture 컨트롤).
- SVG 백엔드는 image crate 로 PNG 재인코딩해 정상 렌더 (과거 동일 클래스 해법:
  `troubleshootings/bmp_svg_render.md`).
- 결함 지점: `src/renderer/skia/image_conv.rs` — JPEG(그레이스케일)만 PNG 정규화하고,
  그 외 포맷은 skia `Image::from_encoded` 원시 전달. skia 가 1bpp BMP 를 디코드하지
  못하면 `draw_missing_image_placeholder`(회색 96,96,96 α48 → 흰 배경 합성 225) 폴백.
  실측 225 와 정확히 일치.

## 수정 방안

`Image::from_encoded` 실패 시 즉시 placeholder 로 가지 않고 **image crate 디코드 →
PNG 재인코딩 → skia 재시도** 폴백 단계를 추가한다 (그마저 실패하면 기존 placeholder 유지).

- SVG 경로의 기존 해법과 대칭 — 포맷 특정 분기(1bpp 전용) 대신 "skia 미지원 → image
  crate 폴백" 일반 규칙이라 BMP 변형 전반을 함께 커버.
- 값/포맷 기반이며 HWP3 전용 분기 아님 (렌더링 하드코딩 금지 규칙 준수).
- 성능: 폴백은 skia 디코드 실패 시에만 발동 — 정상 경로 비용 불변.

## 겹침 점검

- 열린 PR #3272(lpaiu-cs, `image_resolver.rs` 변환 메모화)와 **파일 비중첩**
  (본 수정은 `skia/image_conv.rs`). merge 순서 무관. 장기적으로 폴백 결과를 resolver
  메모에 태우는 통합은 #3272 계열 후속으로 분리.
- 4-backend 규칙: svg(기해결)·canvas·paint(json 포함) 경로의 동일 클래스 여부를 점검
  스텝에 포함하되, 수정 자체는 skia 에 국한(다른 백엔드에서 재현되면 별도 이슈).

## 검증 계획

1. 단위: 1bpp BMP 바이트를 skia 폴백 경로에 태워 디코드 성공 검증(픽스처는 SO-SUEOP
   임베디드 이미지 추출 또는 최소 합성 1bpp BMP).
2. 실측: `export-png SO-SUEOP.hwp -p 0` — 이미지 영역이 placeholder(회색)가 아닌
   실 콘텐츠(흑백 스캔)로 렌더. SVG 경로 산출물과 픽셀 상관 비교.
3. 회귀: 전체 release-test + Native Skia 3종. 42·43쪽 diff 0 유지 확인.
4. 시각 판정: 1쪽 before/after + SVG 대조 이미지를 작업지시자 판정에 제출.

## 산출물

- `src/renderer/skia/image_conv.rs` 수정 + 단위 테스트.
- stage2 구현계획서 → 승인 후 구현 → 보고서(`task_m100_3302_report.md`).
