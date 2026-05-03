# Task #518 Stage 1 보고서 — line_break_char_idx 다중화

**이슈**: #518
**브랜치**: `local/task518`
**Stage**: 1 / 1 (단일 단계)

## 1. 변경 내용

`src/renderer/layout/paragraph_layout.rs` `layout_inline_table_paragraph`:

1. `line_break_char_idx: Option<usize>` → `line_break_char_indices: Vec<usize>` 일반화 (ls[1..] 모두 사용)
2. ctrl_gap 기반 알고리즘 → 직접 char_offsets 룩업 (단순 + 정확)
3. wrap 조건에 `next_break` 추적 추가
4. `RHWP_LAYOUT_DEBUG=1` 시 `LAYOUT_BREAK_INDICES` 출력

## 2. 알고리즘 비교

### 이전 (버그)
```rust
let mut ctrl_gap = 0u32;
// paragraph 전체 controls 합 (over-subtraction)
if !para.char_offsets.is_empty() {
    ctrl_gap += para.char_offsets[0];
    for i in 1..para.char_offsets.len() { ... ctrl_gap += gap - prev_len; }
}
let text_only_ts = ts.saturating_sub(ctrl_gap);  // saturating 0
let mut char_idx = 0;
let mut u16_accum = 0u32;
for (i, ch) in text_chars.iter().enumerate() {
    if u16_accum >= text_only_ts { char_idx = i; break; }
    ...
}
if char_idx > 0 { Some(char_idx) } else { None }
```

문제: `ctrl_gap` 이 paragraph 전체 controls 합. ts 보다 큰 경우 (controls 가 많은 paragraph) saturating 0 으로 항상 None 반환.

### 신규
```rust
let char_idx = para.char_offsets.iter().position(|&off| off >= ts)
    .unwrap_or(text_chars.len());
```

`char_offsets[i]` 가 이미 controls 포함된 raw UTF-16 위치. `>= ts` 인 첫 i 가 break char index. 단순 + 정확.

## 3. 검증

### 3-1. #496 재현 (exam_science p2 pi=61)

`RHWP_LAYOUT_DEBUG=1` 출력:
```
LAYOUT_INLINE_TABLE_PARA: pi=61 sec=0 col_x=534.8 col_w=422.6 y_start=1176.8 ... ls_count=3 tables=1
  LAYOUT_LS[0]: vpos=74118 lh=2864 ls=460 bl=1432 text_start=0 sw=18939
  LAYOUT_LS[1]: vpos=77442 lh=1150 ls=460 bl=575 text_start=13 sw=18939
  LAYOUT_LS[2]: vpos=79052 lh=1150 ls=460 bl=575 text_start=60 sw=30562
  LAYOUT_INLINE_TBL[0]: ctrl_idx=0 rows=2 cols=1 w=14745 h=2864 vert=Top horz=Left wrap=TopAndBottom
  LAYOUT_BREAK_INDICES: pi=61 indices=[5, 28] (from ls[1..])
```

본문 baseline 분포 (column 1, y=1170~1300):

| | Before | After |
|---|--------|-------|
| 표 row 0 | y=1191.68 | y=1191.68 |
| **본문 라인 1** | **y=1195.85 ("는?...[3점]" 56자 1줄, column 폭 초과)** | **y=1195.85 ("는?" 2자)** |
| 표 row 1 | y=1210.77 | y=1210.77 |
| **본문 라인 2** | — | **y=1227.21 ("(단,는임의의원소기호이고,,," 16자)** |
| **본문 라인 3** | — | **y=1248.68 (",의원자량은각각,,,이다.)[3점]" 19자)** |

→ ls[1] (text_start=13) 과 ls[2] (text_start=60) HWP 인코딩 break 위치에서 정확히 줄바꿈.

### 3-2. 회귀 검증

`scripts/svg_regression_diff.sh diff /tmp/test518_before /tmp/test518_after`:
- 7 샘플 170 페이지: **same=167 / diff=3** (exam_science p2/p3/p4)
- 6 샘플 (exam_kor/exam_eng/exam_math/synam-001/aift/2010-01-06) byte 동일
- exam_science 변경 = 정확도 정정 (이전 algorithm 의 ctrl_gap over-subtraction 버그)

### 3-3. 단위/통합 테스트

- `cargo test --release --lib`: **1103 passed; 0 failed**

## 4. 잔여

본 task 는 #496 의 본질 (B) ls[2..] break 미사용 만 정정. 본질 (A) baseline 정렬 + (C) 인라인 vs 블록 정책은 별도 Phase 3/4 영역.
