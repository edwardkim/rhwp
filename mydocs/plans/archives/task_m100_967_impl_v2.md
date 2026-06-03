# 구현 계획서 V2 — Task #967 Stage 2 — Fix B 적용 계획

- 이슈: [#967](https://github.com/edwardkim/rhwp/issues/967)
- Stage 1 결과: `next_will_vpos_reset` 가드의 `next_force_break` 제외 조건이 빈 paragraph + 다음 force_page_break case 를 catch 못함
- 선택: **Fix B** — 별도 분기로 빈 paragraph + 다음 force_page_break case 만 정확 skip

## 1. 변경 위치

`src/renderer/typeset.rs:572-584` (`next_will_vpos_reset` 가드 직전 또는 직후)

## 2. 변경 내용

### Before
```rust
let next_will_vpos_reset = if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
    let next_para = &paragraphs[para_idx + 1];
    let next_force_break = next_para.column_type == ColumnBreakType::Page
        || next_para.column_type == ColumnBreakType::Section;
    if next_force_break {
        false
    } else {
        // vpos check
    }
} else { false };

if next_will_vpos_reset {
    let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
    if is_empty_no_ctrl {
        continue;
    } else {
        st.skip_safety_margin_once = true;
    }
}
```

### After
```rust
let next_will_vpos_reset = if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
    let next_para = &paragraphs[para_idx + 1];
    let next_force_break = next_para.column_type == ColumnBreakType::Page
        || next_para.column_type == ColumnBreakType::Section;
    if next_force_break {
        false
    } else {
        // vpos check
    }
} else { false };

// [Task #967] 빈 paragraph 직후 force page break (쪽나누기) case:
// 빈 paragraph 가 현재 page 잔여 공간 초과 시 별도 page 분기 → +1 page inflate.
// 한컴은 빈 paragraph 를 trailing overflow 로 흡수 + 쪽나누기로 새 page 시작.
// rhwp 도 동일하게 빈 paragraph 를 skip (단독 page 차단).
// 본 가드는 next_will_vpos_reset 의 next_force_break 제외 조건 (hwp-multi-001
// 회귀 차단) 을 유지하면서, 빈 paragraph + 다음 쪽나누기 case 만 추가 catch.
let next_force_break_and_curr_empty = if !st.current_items.is_empty()
    && para_idx + 1 < paragraphs.len()
{
    let next_para = &paragraphs[para_idx + 1];
    let next_force_break = next_para.column_type == ColumnBreakType::Page
        || next_para.column_type == ColumnBreakType::Section;
    let is_curr_empty = para.text.is_empty() && para.controls.is_empty();
    next_force_break && is_curr_empty
} else { false };

if next_will_vpos_reset {
    let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
    if is_empty_no_ctrl {
        continue;
    } else {
        st.skip_safety_margin_once = true;
    }
} else if next_force_break_and_curr_empty {
    // [Task #967] 빈 paragraph + 다음 쪽나누기 → 단독 page 차단
    continue;
}
```

## 3. 영향 분석

### 3.1 변경 직접 영향
- sample18 pi=27 (빈) + pi=28 (쪽나누기) → pi=27 skip → page 2 inflate 제거
- sample18 pi=164 (빈) + pi=165 (쪽나누기) → pi=164 skip → page 14 inflate 제거
- **sample18 페이지 수: 69 → 67** (한컴 정합)

### 3.2 다른 sample 영향
- 빈 paragraph + 다음 force_page_break (column_type::Page/Section) 가진 모든 paragraph 에 영향:
  - 정상화 방향 (단독 page 분기 → 빈 paragraph skip)
  - 한컴 정합 개선 예상

### 3.3 hwp-multi-001 회귀 차단 유지
- 기존 `next_will_vpos_reset` 의 `next_force_break` 제외 조건 그대로 유지 → hwp-multi-001 의 회귀 차단 효과 보존
- 본 fix 의 새 분기는 빈 paragraph 만 영향 → hwp-multi-001 case (비-빈 paragraph) 영향 없음

## 4. 위험 평가

| 위험 | 평가 | 완화 |
|------|------|------|
| 빈 paragraph 가 의도적으로 단독 page 인 case | **낮음** (HWP 인코딩 상 single empty paragraph + page break = trailing) | Stage 4 다중 sample |
| 다른 sample 의 page count 변경 | **중** (정상화 방향 예상) | Stage 4 page count 비교 |
| 회귀 가능성 | **낮음** (빈 paragraph 만 영향, 기존 가드 보존) | cargo test + 시각 회귀 |

## 5. 검증 계획 (Stage 3-4)

### Stage 3 단위 검증
1. cargo build --release
2. sample18 page count: 69 → 67 확인
3. PNG render → 한컴 viewer 정합

### Stage 4 회귀 검증
1. `cargo test --release --lib` 전체 (1288 tests)
2. 다중 sample page count 비교:
   - sample16 (sample 시리즈 — issue #927 영역)
   - sample14, sample10, sample11, sample13
   - exam_kor/math/eng
   - 시험지 4종
   - hwp-multi-001 (회귀 차단 case)
3. golden SVG diff 회귀 0

## 6. Stage 5 (시각 검증 + 최종 보고서 + PR)

## 7. 진행 규칙

- 자동진행 안함
- 각 stage 종료 시 보고서 + 명시 승인
- 회귀 발견 시 **즉시 revert + 보고**
