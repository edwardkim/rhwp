# Task #469 Stage 1 완료 보고서

## 단계 목표

cross-column partial_start 박스의 `rect_y` 가 col_top 위로 침범하는 현상을 검증하는 단위 테스트 추가 (red).

## 산출물

- `src/renderer/layout/integration_tests.rs`: `test_469_partial_start_box_does_not_cross_col_top` 추가

## 테스트 시나리오

`samples/exam_kor.hwp` 페이지 2 SVG 를 렌더링하고, 우측 단 영역(x ∈ [580, 1010]) 의 수직선(y1 ≠ y2 && x1 == x2) 들의 y_top 이 200 이상인지 검증.

## 결과

**FAILED (의도된 red)** — 우측 단 (나) 박스 좌·우 세로선:
- `(x1=994.0, y1=196.55, y2=1020.37)`
- `(x1=593.4, y1=196.55, y2=1020.37)`

두 세로선 모두 y_top=196.55 (헤더선과 동일 위치) 로 측정. 분석 단계의 진단과 정확히 일치.

## 다음 단계

Stage 2 — `src/renderer/layout.rs` L1740 부근에서 `partial_start`/`partial_end` 일 때 `top_pad`/`bot_pad` 미적용으로 `rect_y` 가 col_top 아래로만 시작하도록 수정.
