# 2단계 완료보고서 — Task M100 #1313

## 단계 목표
적분 상·하한(범위)을 확대된 적분 글리프의 상·하단부에 밀착시켜, 위아래로 벌어지는 현상을 제거한다.

## 핵심 발견 — 실제 코드 경로
파서(`parser.rs` 315·1784행)에서 **적분은 항상 `MathSymbol(∫) + SubSup`** 으로 파싱된다
(nolimits 스타일). `BigOp` 은 ∑/∏ 전용이다. 따라서:

- 실제 상·하한 배치 경로 = `layout.rs::layout_subsup()` 의 `is_integral` 분기 (538~583행)
- `layout_integral()`(BigOp 헬퍼)은 적분에 대해 **도달 불가능한 dead code**

1단계에서 잠정 수정했던 `layout_integral` 의 `op_fs` 는 dead code 이므로 원복하여
무관한 diff 를 제거했다(글리프 확대는 공통 `layout_math_symbol` 에서 이미 적용됨).

## 변경 내용 (`layout.rs::layout_subsup` 적분 분기)

| 항목 | 변경 전 | 변경 후 | 효과 |
|------|---------|---------|------|
| 상한 배치 | `sup_box.y = 0` (박스 최상단) | `sup_box.y = base_box.y + fs*0.03` | 상한이 글리프 상단부에 밀착 |
| 하한 배치 | `base_box.y + height - fs*0.25` | `base_box.y + height - fs*0.95` | 하한이 글리프 하단부로 상승·밀착 |

`base_y`(글리프의 줄 내 수직 위치)는 유지 → 적분기호 자체의 줄 정합은 변동 없음.
렌더 3경로(svg/canvas/skia)는 `LayoutKind::SubSup` 자식 box 좌표를 그대로 사용하므로
레이아웃 수정만으로 모두 반영된다.

## 검증 (96dpi 픽셀, p.9 첫 적분)

| 항목 | 1단계 후 | 2단계 후 | 정답 목표 |
|------|---------|---------|----------|
| 상한 baseline (group y) | 6.72 (글리프 위로 뜸) | **13.92** | ~13.4 |
| 하한 baseline (group y) | 36.36 (글리프 아래로 떨어짐) | **27.96** | ~27.7 |

- p.9 적분 3개, p.11 적분 모두 정답 PDF와 시각 정합 (상·하한이 기호에 밀착).
- `cargo test --release --lib equation`: **151 passed, 0 failed**.
- ∑/∏ 경로 무영향 (SubSup 적분 분기만 수정, BIG_OP_SCALE 유지).

## 비고
3단계에서 전체 `cargo test` 와 추가 페이지/샘플 회귀 점검, 최종 보고서를 진행한다.

> **후속 (PDF 정합 튜닝)**: merge 후 작업지시자 피드백으로 정답 PDF(300dpi 측정)에 맞춰
> 글리프 높이(INTEGRAL_SCALE 2.15→2.5)와 상·하한 오프셋(sup 0.21·sub 0.55)을 재조정했다.
> 본 단계의 13.92/27.96 수치는 최종 보고서의 PDF 정합 값으로 대체됨.
