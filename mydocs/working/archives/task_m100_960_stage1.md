# Task #960 Stage 1 — Root cause 정밀 식별

## 1. 증상 (Task #959 Stage 1 발견 재정리)

`samples/3-11월_실전_통합_2022.hwp` page 2 (단 1) 의 문14 (pi=117) 에서:
- f(x) (ci=1) ✓ y=330 (line 0)
- g(x) (ci=2) ✓ y=330 (line 0)
- **cases formula (ci=3) ❌ y=329** (실제 line 1 예상 y~347) — 잘못된 y
- h(x)=lim (ci=4) ✓ y=380 (line 2)

## 2. RHWP_DEBUG_PARA_TAC 추적 결과

```
DEBUG_PARA_TAC cc=64 text_codepoints=64 fffc_at=[] tac_controls=[(5, 1851, 1), (14, 1851, 2), (30, 13339, 3), (34, 13455, 4)]
```

- text 에 **FFFC (object replacement char) 없음** — 모든 chars 가 일반 텍스트/공백/탭/newline
- `find_control_text_positions` 가 **char_offsets 의 gap 분석**으로 control 위치 결정
- Cases formula (ci=3) → **position 30** (= `\n` 문자)

## 3. 데이터 흐름

### text_chars 구조
| codepoint idx | char | utf16 offset |
|---------------|------|--------------|
| 27 | `.` | 51 |
| 28 | `\t` | 52 |
| 29 | `\t` | 60 |
| **30** | **`\n`** | **76** |
| 31 | `함` | 77 |

### char_offsets gap 분석 (control_text_positions in model/paragraph.rs:817-838)

- gap [60, 76] = 76-60-1 = 15
- gap/8 = 1 → 1 control assigned to position (i+1) = 30

### compose_lines 결과
```
ComposedLine[0] char_start=0  chars=23 has_lb=false  → range [0, 23)
ComposedLine[1] char_start=23 chars=7  has_lb=true   → range [23, 30)
ComposedLine[2] char_start=31 chars=8  has_lb=true   → range [31, 39)
ComposedLine[3] char_start=40 chars=24 has_lb=false  → range [40, 64)
```

→ Line 1 chars range = [23, 30) — **position 30 (= \n) excluded** (line break char 제외).  
→ Line 2 starts at 31. **Position 30 is between lines** (no-man's-land).

## 4. Filter 결함 (paragraph_layout.rs:1724-1727)

```rust
let run_tacs: Vec<(usize, f64, usize)> = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= run_char_pos 
        && (*pos < run_char_end 
            || (is_last_run && *pos == run_char_end)))
    .map(|(pos, w, ci)| (pos - run_char_pos, *w, *ci))
    .collect();
```

Cases (pos=30) for line 1 (run_char_pos=23, run_char_end=30):
- pos >= 23 ✓
- pos < 30 ❌ (30 < 30 false)
- `is_last_run && pos == 30` ❌ (line 1 은 paragraph 의 last line 아님)

→ Cases 가 line 1 의 run_tacs 에서 제외 → paragraph_layout 미emit → shape_layout 의 default y (para_y=329.6) 에 emit.

## 5. TAC_LINE 추적 결과

```
TAC_LINE pi=117 line_idx=0 ... run_tacs=[(5, 24.68, 1), (14, 24.68, 2)]
TAC_LINE pi=117 line_idx=1 ... run_tacs=[]                              ← cases 누락
TAC_LINE pi=117 line_idx=2 ... run_tacs=[(3, 179.4, 4)]
TAC_LINE pi=117 line_idx=3 ... run_tacs=[]
```

## 6. Root cause 정리

**Filter 가 has_line_break line 의 end position 의 control 을 누락**.

HWP 인코딩 상 `\n` 직전의 control (예: 본 cases formula at position 30 = '\n' position) 은 logically line 1 의 END 에 속해야 함. 그러나 현재 filter 는 `pos < run_char_end` 또는 `is_last_run && pos == run_char_end` 만 허용.

## 7. Fix 후보

### A. has_line_break line 의 end position 도 포함 (권장)

```rust
let is_last_of_line = is_last_run_of_line(run_idx);
let allow_end = is_last_run 
    || (comp_line.has_line_break && is_last_of_line);
.filter(|(pos, _, _)| *pos >= run_char_pos 
    && (*pos < run_char_end || (allow_end && *pos == run_char_end)))
```

- 위험: **중** — has_line_break line 의 end position control 이 다수 sample 에 미치는 영향
- 정밀: 본 case 해결 + 유사 case (다른 paragraph 의 line break 직전 control) 도 fix

### B. control_text_positions 의 boundary 처리

position 이 `\n` 이면 그 위치를 -1 (line 1 의 마지막 chars 위치) 또는 line 1 의 end-1 로 조정.

- 위험: **고** — 모든 paragraph 의 control 위치 영향

### C. compose_lines 가 line break char 를 line 에 포함

`\n` 을 해당 line 의 마지막 char 로 포함 → chars=8 (정의한다.\t\t\n).

- 위험: **매우 고** — 모든 line 의 char count 영향

## 8. 권장 Fix

**Option A** — paragraph_layout.rs filter 한 줄 수정. 가장 localized, 위험 최소.

## 9. 후속

- Stage 2: 구현 계획 V2 (Fix A 의 안전 구현)
- Stage 3: 구현 + 단위 검증
- Stage 4: 다중 sample 회귀 검증 (특히 has_line_break + 행 끝 control 보유 sample)
