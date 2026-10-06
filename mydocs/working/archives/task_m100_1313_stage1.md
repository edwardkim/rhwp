# 1단계 완료보고서 — Task M100 #1313

## 단계 목표
적분 전용 스케일 상수(`INTEGRAL_SCALE`)를 분리하고, 적분기호(∫) 글리프 높이를 정답(한글 2022)에
맞게 확대한다. ∑/∏ 등 다른 큰 연산자는 영향받지 않도록 한다.

## 변경 내용

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/layout.rs` | `INTEGRAL_SCALE = 2.15` 상수 신설; `layout_math_symbol`·`layout_integral` 의 `op_fs` 를 `INTEGRAL_SCALE` 적용 |
| `src/renderer/equation/svg_render.rs` | BigOp 분기에서 `op_fs` 를 `is_integral ? INTEGRAL_SCALE : BIG_OP_SCALE` 로 조건부 계산 |
| `src/renderer/equation/canvas_render.rs` | 동일 조건부 `op_fs` (SVG 동기화) |
| `src/renderer/skia/equation_conv.rs` | `INTEGRAL_SCALE` import + 동일 조건부 `op_fs` |

## 검증

샘플: `samples/3-10월_교육_통합_2022.hwp` p.9, 정답 `pdf/3-10월_교육_통합_2022.pdf` p.9 (96dpi).

| 항목 | 1단계 전 | 1단계 후 | 정답 |
|------|---------|---------|------|
| ∫ 글리프 세로 높이 | ~16px (≈1.33×fs) | **~24px (≈2.0×fs)** | ~23px (≈1.9×fs) |

- ∫ 글리프 높이가 정답에 근접하도록 확대됨 (시각 비교 `/tmp/cmp1.png`).
- ∑/∏ 경로는 `BIG_OP_SCALE` 유지 — 조건 분기로 불변.
- `cargo build --release` 성공, 수식 관련 테스트 통과(0 failed).

## 비고 (다음 단계로 이월)
글리프 확대로 `op_h` 가 커지면서 기존 비율식(`sup_shift=op_h*0.1`, `sub_shift=op_h*0.55`)의
상·하한이 오히려 더 벌어진 상태이다. **2단계에서 상·하한을 글리프 실제 상·하단 끝에 밀착**하도록
오프셋을 재조정한다(계획대로).
