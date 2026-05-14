# Task #894 Stage 2 완료 보고서 — paragraph multi-line picture SVG 중복 emit

**Stage**: 2 / 3 (항목 B)
**상태**: ✅ 완료

## 1. 문제

sample16 페이지 18 (한컴 16쪽) 의 paragraph 394 [1] 그림 (WMF 다이어그램, bin_id=3) 이 SVG 에 **3개 `<image>` 로 emit** 됨 (모두 동일 href). 한컴 viewer 는 1개만 표시.

## 2. 진단

### 2.1 paragraph 394 구조

- text: `"￼￼  ￼"` (5 chars, `\u{FFFC}` marker at chars 0, 1, 4)
- controls 3개: [0] 표, [1] 그림 (WMF), [2] 표
- line_segs 3개

### 2.2 SVG image 위치 측정

3 image 모두 width=608.48px (= 161.0mm = picture [1] 의 너비). 위치만 다름. 즉 picture [1] 만 3 번 emit, 표 2 개는 별도 element.

### 2.3 ROOT CAUSE 추적

`src/renderer/layout/paragraph_layout.rs:1555` 의 `run_tacs` 필터:

```rust
let run_tacs: Vec<(usize, f64, usize)> = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= run_char_pos && (*pos < run_char_end || (is_last_run && *pos == run_char_end)))
    .map(|(pos, w, ci)| (pos - run_char_pos, *w, *ci))
    .collect();
```

디버그 결과:
```
[DBG394] tac_offsets_px=[(5, 336.21, 0), (5, 608.48, 1), (5, 177.17, 2)]
[DBG394] composed.tac_controls=[(5, 25216, 0), (5, 45636, 1), (5, 13288, 2)]

line_idx=0 run_char_pos=0 run_char_end=1 → run_tacs=[]
line_idx=1 run_char_pos=1 run_char_end=4 → run_tacs=[]
line_idx=2 run_char_pos=4 run_char_end=5 is_last_run=true → run_tacs=[3 entries]
```

→ **모든 tac control 의 pos 가 5** (paragraph 끝). 실제로는 0, 1, 4 가 정답. line[2] (is_last_run=true) 처리 시 3 control 모두 매치되어 picture 1개가 3번 emit.

### 2.4 ROOT CAUSE 함수 추적

`composer.rs:122` → `find_control_text_positions(para)` → `paragraph.rs:773 control_text_positions()`:

paragraph 394 의 `char_offsets=[0, 1, 2, 3, 4]` — sequential, gap 분석 결과 control 발견 없음. 모든 3 controls 가 fallback 분기 (line 841):

```rust
while positions.len() < total_controls {
    positions.push(chars.len());  // = 5
}
```

으로 paragraph 끝 (5) 으로 push.

### 2.5 HWP3 파서의 char_offsets 생성

`src/parser/hwp3/mod.rs:330~520` — HWP3 파서가 각 char 마다 `char_offsets.push(utf16_len)` + `utf16_len += 1`. **control marker 위치에 +8 gap 추가하지 않음** → HWP5 spec 의 char_offsets 형태와 다름.

## 3. Fix

`src/model/paragraph.rs` 의 `control_text_positions()` 함수의 fallback 분기를 강화:

```rust
// 갭 분석으로 발견되지 않은 컨트롤의 위치를 text 의 `\u{FFFC}` marker
// 위치로 매핑한다.
let mut search_start = positions.last().copied().unwrap_or(0);
while positions.len() < total_controls {
    let next_marker = chars[search_start..].iter()
        .position(|&c| c == '\u{FFFC}')
        .map(|rel| search_start + rel);
    match next_marker {
        Some(abs_pos) => {
            positions.push(abs_pos);
            search_start = abs_pos + 1;
        }
        None => {
            positions.push(chars.len());  // 기존 동작
        }
    }
}
```

**핵심**: 갭 분석으로 채워진 positions 의 마지막 인덱스 이후를 search start 로 사용하여 중복 매핑 방지. HWP5/HWPX 의 정상 갭 분석 결과는 그대로 사용 → 비-HWP3 paragraph 영향 없음.

## 4. 검증

### 4.1 sample16 paragraph 394 image emit

| 항목 | 이전 | 이후 |
|------|------|------|
| `<image>` 개수 | 3 | **1 ✅** |
| 위치 정합 | 잘못된 3 위치 | 정확한 line[1] 위치 |

### 4.2 회귀 점검

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1234 passed (회귀 없음) |
| HWP3 sample 6종 페이지 수 | 모두 동일 (`hwp3-sample` 16, 4, 5, 10, 13, 14) |
| HWPX sample 페이지 수 | 모두 동일 (sample16-hwp5.hwpx 72 유지 — #895 별도) |

## 5. 커밋

- `5c177bd` — Task #894 Stage 2: control_text_positions fallback 강화 — text marker 스캔

## 6. 산출물

- 본 Stage 보고서: `mydocs/working/task_m100_894_stage2.md`
- Fix: `src/model/paragraph.rs` (+23 lines, -2 lines)

## 7. 후속 영향 (참고)

HWP3 파서의 char_offsets 가 HWP5 spec 형태 (control marker +8 gap) 가 아닌 sequential 만 push 하는 것은 별도 정합 task 가치 있음. 본 fix 는 `control_text_positions` 의 fallback 강화로 호환성 보장. 추후 HWP3 파서 자체 수정 시 본 fallback 도 활용 가능 (이중 안전).
