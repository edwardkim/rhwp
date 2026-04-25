# Task #321 PR #323 v3 정밀화 — Option A 적용 보고서

**기준일**: 2026-04-25
**브랜치**: `task321`
**관련 이슈**: #326

## 적용 내역

작업지시자 피드백(`mydocs/feedback/task_321_pr323_review_col1_start.md`) 의 Option A 권고에 따라, v3 의 Paper 도형 일률 제외 가드를 \"본문과 겹치지 않을 때만 제외\" 로 정밀화.

### `src/renderer/typeset.rs::compute_body_wide_top_reserve_for_para`

```rust
if matches!(common.vert_rel_to, VertRelTo::Paper) {
    let shape_top_abs = hwpunit_to_px(common.vertical_offset as i32, dpi);
    let shape_bottom_abs = shape_top_abs + hwpunit_to_px(common.height as i32, dpi);
    if shape_bottom_abs <= body_top {
        continue;
    }
}
```

### `src/renderer/layout/shape_layout.rs::calculate_body_wide_shape_reserved`

동일 로직 추가 (양쪽 동기화).

## 검증

### 21_언어 page 1

- `compute_body_wide_top_reserve_for_para` 트레이스: `pi=0 reserve=329.95 body_top=209.76 cols=2`
- `calculate_body_wide_shape_reserved` 트레이스: `pi=0 ci=2 vert_rel=Paper bottom_y=329.95 shape_y=131.63 thresh=618.56`
- col 1 첫 본문 SVG y 위치: **342.4px** (4×5 표 끝점 314.8 보다 아래) ✓

### 4 샘플 페이지 수

| 샘플 | v3 적용 | v3 revert | Option A |
|------|---------|-----------|----------|
| 21_언어 | 15 | 16 | **16** |
| exam_math | 20 | 20 | 20 |
| exam_kor | 24 | 24 | 24 |
| exam_eng | 9 | 10 | **10** |

Option A 는 v3 의 \"15쪽/9쪽\" 페이지 수 감소 효과를 회복하지 못함. 이는 본 fix 가 reserve 를 유지(시각적 정확성 우선)하기 때문이며, v3 의 페이지 감소는 reserve 를 부적절히 제거한 부산물(과실)이었음.

### 회귀

- `cargo test --release`: 992 + 71 통과
- `cargo clippy --release -- -D warnings`: 클린

## 알려진 차이 (작업지시자 보고 데이터 vs 본 fix)

- 작업지시자 \"devel(정상)\" 데이터: `단 1 used=1223.1 hwp_used=38.9 diff=+1184.3`
- Option A 적용 후: `단 1 used=1226.4 hwp_used=925.5 diff=+300.9`

`hwp_used` 는 col 1 에 포함된 PageItem 들의 vpos 분포에서 계산되며 col 1 콘텐츠에 따라 달라짐. devel(=v2/v3 미적용) 은 reserve 가 typeset 단계에 반영되지 않아 col 1 에 적은 콘텐츠만 포함되었을 가능성. Option A 는 typeset 의 reserve 인지로 인해 더 많은 콘텐츠를 col 1 에 분배함.

**시각적 정확성**(col 1 이 표 아래에서 시작) 은 달성됨. 페이지 수 16 은 v3 가 15 로 만들었던 것이 부적절한 reserve 제거의 부산물이었음을 시사.

## 다음 단계

작업지시자 재검토 → 페이지 수 16 vs 15 의 시각적 정확성 우선 여부 확인.
