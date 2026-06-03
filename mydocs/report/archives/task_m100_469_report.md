# Task #469 최종 결과 보고서

## 이슈

- 번호: #469
- 제목: exam_kor 2p 우측 단 (나) cross-column 박스 좌·우 세로선이 헤더선과 맞닿음
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task469`

## 증상

`samples/exam_kor.hwp` 페이지 2 우측 단의 (나) 박스(border_fill_id=7, pi=39..42 의 cross-column 연속 박스) 좌·우 세로선이 페이지 상단 헤더 가로선(`y=196.55`)과 맞닿아 그려짐. PDF 원본은 약 15 px 떨어진 정상 위치.

## 근본 원인

`src/renderer/layout.rs` paragraph border 그룹 렌더링 블록 (L1735–1745):

1. L1707 (Task #445): `y_start` 를 `col_top` (211.65) 으로 클램프 — OK
2. L1740: `rect_y = y_start - top_pad` 적용
   - `top_pad = border_spacing.top` (1133 HU ≈ 15.1 px)
   - 결과: `rect_y = 211.65 − 15.1 = 196.55` (헤더선과 동일 좌표)
3. L1808–1809: partial_start 일 때 윗변 가로선은 `skip_top` 이지만 좌·우 세로선은 `y_top=rect_y` 부터 항상 그려져 **헤더선까지 침범**

원인은 cross-column 으로 이어진 partial 박스의 후속 부분에서 `top_pad`/`bot_pad` 를 그대로 적용한 점. 이전/다음 컬럼에서 이미 inset 이 처리되었으므로 후속 부분은 `col_top` 아래로만 시작해야 함.

## 수정

`src/renderer/layout.rs:1738-1745`:

```rust
// Task #469: cross-column / cross-page 로 이어진 partial 박스의 후속 부분은
// 이전/다음 컬럼에서 이미 inset 이 적용되었으므로 여기서 다시 col_top/col_bot
// 너머로 박스를 확장하면 안 된다 (헤더선/꼬리말선과 충돌).
// y_start/y_end 는 L1707 에서 col_top..col_bot 으로 이미 클램프됨.
let effective_top_pad = if is_partial_start { 0.0 } else { top_pad };
let effective_bot_pad = if is_partial_end { 0.0 } else { bot_pad };
let rect_y = y_start - effective_top_pad;
let rect_h = height + effective_top_pad + effective_bot_pad;
```

대칭적으로 `partial_end` 케이스(꼬리말선 영역 침범) 도 함께 차단.

## 추가된 테스트

`src/renderer/layout/integration_tests.rs::test_469_partial_start_box_does_not_cross_col_top`

- `samples/exam_kor.hwp` 페이지 2 의 SVG 를 렌더링
- 우측 단 영역(x ∈ [580, 1010]) 의 수직선들이 y_top ≥ 200 인지 검증
- 수정 전: FAIL (y1=196.55) → 수정 후: PASS (y1=211.65)

## 검증 결과

| 항목 | 결과 |
|------|------|
| 신규 단위 테스트 | PASS |
| 전체 cargo test (1121건) | 1121 / 1121 PASS |
| 골든 SVG (6건) | 영향 없음, 모두 PASS |
| exam_kor 시각 확인 | 좌·우 세로선 y1 = 211.65 (col_top), 헤더선과 15.1 px 분리 ✓ |
| 좌측 단 (가) 박스 회귀 | 영향 없음 (y1 = 242.41 변동 없음) |

## 영향 범위

- cross-column 또는 cross-page 로 이어지는 paragraph border 박스(border_fill_id > 0) 의 partial_start / partial_end 케이스.
- 단일 paragraph 박스(non-partial)는 `is_partial_start = is_partial_end = false` 라 영향 없음.
- 기존 Task #321 v6 (border_spacing 적용), Task #445 (col_area 클램프), Task #468 (cross-column partial 플래그 보정) 의 정합성 유지하며 후속 정리 역할.

## 산출물

- `src/renderer/layout.rs` (8라인 변경)
- `src/renderer/layout/integration_tests.rs` (테스트 1건 추가, +44라인)
- `mydocs/plans/task_m100_469.md` (수행계획서)
- `mydocs/plans/task_m100_469_impl.md` (구현계획서)
- `mydocs/working/task_m100_469_stage{1,2,3}.md` (단계별 보고서)
- `mydocs/report/task_m100_469_report.md` (본 문서)
