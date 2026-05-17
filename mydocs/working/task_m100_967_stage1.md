# Task #967 Stage 1 — Root cause 정밀 식별

## 1. 페이지 비교 (재확인)

| Format | rhwp | 한컴 | 차이 |
|--------|------|------|------|
| HWP3 (.hwp) | **69** | **67** | **+2** |
| HWP5 변환본 (.hwp) | 67 | 67 | 0 ✓ |
| HWPX 변환본 (.hwpx) | 74 | 67 | +7 (별도 issue) |

## 2. 결함 page 의 paragraph 구조

### Page 2 (extra page)

| pi | text | vpos (HU) | lh+ls (HU/px) |
|----|------|-----------|---------------|
| 26 | (빈 문단) — page 1 마지막 | 66556 (888px) | 2800 / 37.3 |
| **27** | **(빈 문단)** | **69356 (925px)** | **1800 / 24.0** |
| 28 | "￼￼" — **[쪽나누기]** | 0 (reset!) | 3296 |

### Page 14 (extra page)

| pi | text | vpos (HU) | lh+ls (HU/px) |
|----|------|-----------|---------------|
| 163 | (빈 문단) | 67916 | 1920 / 25.6 |
| **164** | **(빈 문단)** | **69836 (931px)** | **1920 / 25.6** |
| 165 | "...G2B시스템 서버현황" — **[쪽나누기]** | 0 (reset!) | 1920 |

→ **동일 패턴**: 빈 paragraph (pi=27, pi=164) + 다음 paragraph 에 `[쪽나누기]` 명시.

## 3. Layout 분석

### Page 1 의 잔여 공간

```
body height = 935 px
page 1 used = 934.2 px (pi=0~pi=26)
잔여 = 0.8 px

pi=27 height = 24 px → 잔여 (0.8px) 초과 → 별도 page 분기
```

한컴은 빈 paragraph (24px) 을 trailing overflow 로 허용 → 동일 page 유지. rhwp 는 엄격하게 fit 검사 → 새 page.

## 4. Code path 분석

`src/renderer/typeset.rs:572-584` 의 **단독 빈 페이지 차단** 가드:

```rust
if next_will_vpos_reset {
    let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
    if is_empty_no_ctrl {
        continue;  // skip empty paragraph
    }
}
```

이 가드는 **`next_will_vpos_reset` 가 true 일 때**만 발동. 그러나:

```rust
let next_will_vpos_reset = if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
    let next_para = &paragraphs[para_idx + 1];
    let next_force_break = next_para.column_type == ColumnBreakType::Page
        || next_para.column_type == ColumnBreakType::Section;
    if next_force_break {
        false   // ← pi=28 의 [쪽나누기] = Page → next_will_vpos_reset = false
    } else {
        // vpos check
    }
};
```

pi=28 의 `[쪽나누기]` → `column_type == Page` → `next_force_break = true` → **`next_will_vpos_reset = false`** → 가드 미발동 → pi=27 단독 page 생성.

### 코멘트 참조 (line 553-554)

```
// 가드 제외 조건:
//   - 다음 pi 가 force_page_break (column_type==Page/Section) 인 경우 발동 안 함
//     (정상 쪽나누기 신호 — 단독 페이지 발생 안 함, hwp-multi-001 회귀 차단)
```

본 가드 제외는 `hwp-multi-001` 회귀 차단 목적으로 도입. 그러나 **본 sample18 case 에서는 빈 paragraph + 다음 쪽나누기 = 단독 page 생성** → 제외 조건이 sample18 같은 case 를 못 catch.

## 5. Root cause 정리

`next_will_vpos_reset` 가드의 제외 조건 (next_force_break 시 false) 가 너무 광범위 — 빈 paragraph + 다음 force page break case 를 못 catch.

## 6. Fix 후보

### A. next_force_break 제외 조건에 "현재 빈 paragraph" 예외 추가

```rust
let is_curr_empty = para.text.is_empty() && para.controls.is_empty();
let next_force_break = next_para.column_type == ColumnBreakType::Page
    || next_para.column_type == ColumnBreakType::Section;
if next_force_break && !is_curr_empty {
    false  // 기존 동작
} else if next_force_break && is_curr_empty {
    true   // 빈 paragraph 이면 vpos-reset 가드 발동 → skip
}
```

- 위험: **중** (next_force_break + 빈 paragraph case 가 hwp-multi-001 회귀와 어떻게 다른지 확인 필요)
- 정밀: 빈 paragraph 만 영향

### B. 별도 분기 — next_force_break + 빈 paragraph 직접 skip

`next_will_vpos_reset` 와 별개로 새 가드 추가:

```rust
let next_force_break = ...;
let is_curr_empty = para.text.is_empty() && para.controls.is_empty();
if next_force_break && is_curr_empty && !st.current_items.is_empty() {
    continue;  // 빈 paragraph + 다음 page break = skip
}
```

- 위험: **중** (새 로직 추가)
- 정밀: 본 case 한정

### C. trailing empty paragraph overflow tolerance

빈 paragraph 가 잔여 공간 초과 시 일정 px (예: 30px) 까지 허용.

- 위험: **고** (모든 paragraph 의 fit 검사 영향)
- 정밀: 광범위

## 7. 권장 Fix

**Option B** — 별도 분기, 본 case (빈 paragraph + 다음 force page break) 만 정확 catch. 기존 가드 (next_will_vpos_reset) 동작 보존.

## 8. 후속 (Stage 2)

- Stage 2: 구현 계획 V2 — Fix B 의 안전한 구현 + 위험 평가
- Stage 3: 구현 + sample18 단위 검증
- Stage 4: 다중 sample 회귀 검증 (특히 hwp-multi-001 등 기존 회귀 차단된 case)
