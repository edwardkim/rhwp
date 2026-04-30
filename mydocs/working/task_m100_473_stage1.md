# Task #473 Stage 1 완료 보고서

## 단계 목표

그림 crop 변환의 viewBox 과대 회귀를 검증하는 통합 테스트 추가 (red).

## 산출물

`src/renderer/layout/integration_tests.rs::test_473_picture_crop_viewbox_matches_image_px`

## 테스트 시나리오

`samples/21_언어_기출_편집가능본.hwp` 페이지 12 SVG 의 nested `<svg viewBox=...><image .../></svg>` 패턴에서 viewBox 와 image 의 비율이 1.0 ± 10% 안인지 검증.

## 결과

**FAILED (의도된 red)** — viewBox=(13875, 10333.75) vs image=(2220, 1654), ratio=6.25 → 검출.

## 다음 단계

Stage 2 — `compute_image_crop_src` scale 계산을 96-DPI 관행 기반으로 변경.
