# Task #967 — 최종 보고서

- 이슈: [#967](https://github.com/edwardkim/rhwp/issues/967)
- 마일스톤: M100 / v1.0.0
- 브랜치: `local/task967`
- 기간: 2026-05-18 (1일)

## 1. 작업 범위

HWP3 `hwp3-sample18.hwp` 페이지 수 +2 inflate (rhwp 69 vs 한컴 67) 해결.

## 2. Root cause

### 2.1 증상

| Format | rhwp | 한컴 | 차이 |
|--------|------|------|------|
| HWP3 | 69 | 67 | +2 |
| HWP5 변환본 | 67 | 67 | 0 |
| HWPX 변환본 | 74 | 67 | +7 (별도 issue) |

### 2.2 결함 page 의 paragraph 구조

| Page | pi | text | vpos | lh+ls |
|------|----|----|------|-------|
| Page 2 | pi=27 | (빈 문단) | 69356 HU (925 px) | 1800 / 24 px |
| Page 14 | pi=164 | (빈 문단) | 69836 HU (931 px) | 1920 / 25.6 px |

→ 두 빈 paragraph 가 각각 별도 page 차지.

### 2.3 다음 paragraph 의 [쪽나누기]

- pi=28 = "￼￼" + **[쪽나누기]** control
- pi=165 = "    G2B시스템 서버현황" + **[쪽나누기]**

→ 빈 paragraph (pi=27, pi=164) 직후 강제 page break.

### 2.4 typeset 의 가드 logic gap

`src/renderer/typeset.rs:555-584`:
```rust
let next_will_vpos_reset = if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
    let next_para = &paragraphs[para_idx + 1];
    let next_force_break = next_para.column_type == ColumnBreakType::Page
        || next_para.column_type == ColumnBreakType::Section;
    if next_force_break {
        false   // ← 다음 force_break 시 false (hwp-multi-001 회귀 차단)
    } else {
        // vpos check
    }
};
if next_will_vpos_reset {
    if is_empty_no_ctrl { continue; }
    ...
}
```

pi=28 의 `[쪽나누기]` → `column_type == Page` → `next_force_break = true` → **`next_will_vpos_reset = false`** → 단독 빈 페이지 차단 가드 미발동 → pi=27 별도 page 생성.

## 3. Fix

`src/renderer/typeset.rs:584-604`: 기존 next_will_vpos_reset 가드 직후 별도 분기 추가.

```rust
} else if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
    // [Task #967] 빈 paragraph 직후 force page break (쪽나누기) case 가드:
    // 빈 paragraph 가 현재 page 잔여 공간 초과 시 별도 page 분기 →
    // +1 page inflate 회귀 (sample18.hwp 의 pi=27, pi=164).
    let next_para = &paragraphs[para_idx + 1];
    let next_force_break = next_para.column_type == ColumnBreakType::Page
        || next_para.column_type == ColumnBreakType::Section;
    let is_curr_empty = para.text.is_empty() && para.controls.is_empty();
    if next_force_break && is_curr_empty {
        continue;  // 빈 paragraph skip — 단독 page 차단
    }
}
```

기존 next_will_vpos_reset 의 next_force_break 제외 조건 (hwp-multi-001 회귀 차단) **보존**.

## 4. 검증

### 4.1 cargo test
- `cargo test --release --lib`: **1288 passed, 0 failed, 2 ignored**

### 4.2 sample18 단위 검증
- 페이지 수: **69 → 67 ✓** (한컴 정합)

### 4.3 다중 sample 회귀 검증
- 16 sample (HWP3 시리즈 + table + multi-table + exam): **모두 회귀 0**
- hwp-multi-001 (가드의 회귀 차단 case): **변경 없음 ✓** (회귀 차단 보존)

## 5. 영향 평가

| 영역 | 영향 |
|------|------|
| 빈 paragraph + 다음 [쪽나누기] case | skip → +1 page inflate 제거 (회귀 fix) |
| 비-빈 paragraph + 다음 [쪽나누기] | 영향 없음 (기존 동작) |
| 빈 paragraph + 다음 일반 paragraph (vpos-reset 가드) | 영향 없음 (기존 가드 보존) |
| hwp-multi-001 (회귀 차단) | 영향 없음 |

## 6. 관련

- 닫힌 issue [#927](https://github.com/edwardkim/rhwp/issues/927) — 페이지 수 inflate (sample16 case, 본 fix 와 무관)
- archive/task936 의 page 2 inflate fix 시도 history
- `44145ab4` (Task #877 Stage 3 v3) — TAC ShapeObject treat_as_char 확장 (다른 inflate fix)

## 7. 후속

- 작업지시자가 PR 머지
- HWPX +7 inflate (sample18-hwp5.hwpx) — 별도 issue (HWPX-specific pagination)
- 다른 sample 의 page count mismatch — 개별 분석 (sample16 case 등)
