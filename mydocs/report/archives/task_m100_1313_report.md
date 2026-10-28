# 최종 결과보고서 — Task M100 #1313

## 제목
적분기호(∫) 상·하한이 위아래로 벌어지고 기호 높이가 정답보다 작음

- 이슈: edwardkim/rhwp#1313 (M100, bug)
- 브랜치: `local/task1313`
- 관련 문서: 수행계획서 `plans/task_m100_1313.md`, 구현계획서 `plans/task_m100_1313_impl.md`,
  단계보고 `working/task_m100_1313_stage1.md`·`_stage2.md`

## 1. 증상
`samples/3-10월_교육_통합_2022.hwp` 9페이지의 적분식에서 적분 상·하한(범위)이 적분기호로부터
위아래로 벌어져 보였다. 다른 페이지 적분식도 동일.

## 2. 원인
1. **글리프 크기**: 적분기호가 `BIG_OP_SCALE`(1.5)로 그려져 정답(한글 2022, ≈1.9×fs)보다 작았다.
2. **상·하한 배치**: `layout_subsup()` 의 적분 분기에서 상한은 박스 최상단(`sup_box.y=0`),
   하한은 글리프 하단에서 `fs*0.25` 만 올린 위치에 두어, 글리프 대비 상·하한이 떠 보였다.
3. 적분은 파서상 항상 `MathSymbol(∫) + SubSup`(nolimits)로 파싱되므로 실제 배치 경로는
   `layout_subsup()` 의 `is_integral` 분기다 (`BigOp`/`layout_integral` 은 ∑/∏ 전용·적분 dead code).

## 3. 수정 내용

### 글리프 높이 (1단계)
- `layout.rs` 에 적분 전용 상수 `INTEGRAL_SCALE` 신설 (PDF 정합 튜닝 후 **2.5**).
- 적분 글리프 `op_fs` 를 `BIG_OP_SCALE` → `INTEGRAL_SCALE` 로 분리:
  - `layout_math_symbol()` (적분 base 글리프)
  - 렌더 3경로 `svg_render` / `canvas_render` / `skia::equation_conv` 의 BigOp 분기를
    `is_integral ? INTEGRAL_SCALE : BIG_OP_SCALE` 로 조건부 적용.
- ∑/∏ 등은 `BIG_OP_SCALE` 유지.

### 상·하한 밀착 (2단계 + PDF 정합 튜닝)
- `layout.rs::layout_subsup()` 적분 분기에서 상한을 글리프 상단부(`sup_box.y = base_box.y + fs*0.21`),
  하한을 글리프 하단 바로 아래(`base_box.y + height - fs*0.55`)에 배치.
- 렌더 3경로는 `SubSup` 자식 box 좌표를 그대로 사용 → 레이아웃 수정만으로 모두 반영.
- (정리) 1단계에서 잠정 수정한 dead code `layout_integral` op_fs 는 원복.

## 4. 검증

### 시각 정합 (정답 `pdf/3-10월_교육_통합_2022.pdf`, 작업지시자 PDF 정합 판정)
정답 PDF를 300dpi로 정밀 측정해 글리프 높이·상·하한 위치를 맞췄다 (96dpi 절대 px).

| 항목 | 수정 전 | 최종(PDF 정합) | 정답 |
|------|--------|--------------|------|
| ∫ 글리프 세로 높이 (p.9) | ~16px | **~28px** | ~28px (92–120) |
| 상한 "4" baseline (abs) | 91 부근(위로 뜸) | **~98** | ~99 |
| 하한 "0" baseline (abs) | 글리프 아래로 떨어짐 | **~119** | ~120 (글리프 하단) |

- p.9 적분 3개, p.11 적분 모두 정답 PDF와 시각 정합 (글리프 높이·상·하한 위치 일치, 줄 충돌 없음).
- 초기 수정(글리프 24px, 상·하한 13.92/27.96)은 작업지시자 피드백("범위가 좁다·PDF 기준으로")에 따라
  글리프 높이(2.15→2.5)와 상·하한 오프셋을 PDF 정밀 측정값에 맞춰 재튜닝했다.

### 테스트
- 전체 `cargo test --release`: **2085 passed, 0 failed** (117개 스위트 전부 ok).
- `cargo build --release` 성공.
- ∑/∏ 경로 무영향 (조건 분기 게이트 + SubSup 적분 분기 한정 수정).

## 5. 영향 파일
- `src/renderer/equation/layout.rs`
- `src/renderer/equation/svg_render.rs`
- `src/renderer/equation/canvas_render.rs`
- `src/renderer/skia/equation_conv.rs`

## 6. 결론
적분기호 글리프를 정답 비례로 확대하고 상·하한을 글리프 상·하단부에 밀착시켜, 보고된
"범위가 위아래로 벌어짐" 현상을 해소했다. 전 테스트 통과, ∑/∏ 등 다른 큰 연산자 무영향.
