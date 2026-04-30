# Task #471 Stage 1 완료 보고서

## 단계 목표

좌측 단 (가) 박스 하단 가로선 부재를 검증하는 통합 테스트 추가 (red).

## 산출물

`src/renderer/layout/integration_tests.rs::test_471_cross_column_box_no_bottom_line_in_col0`

## 테스트 시나리오

`samples/21_언어_기출_편집가능본.hwp` 페이지 1 SVG 의 좌측 단 영역 (x ∈ [120, 542]) 에서 stroke 가 있는 4면 rect 의 bottom_y 가 1300 보다 큰 항목이 있는지 검사.

## 결과

**FAILED (의도된 red)** — col 0 의 (가) 박스가 4면 stroke rect 로 그려져 bottom_y ≈ 1438 의 위반 검출.

## 다음 단계

Stage 2 — `src/renderer/layout.rs:1670-1699` Task #468 cross-column 검출을 stroke_sig 비교로 변경.
