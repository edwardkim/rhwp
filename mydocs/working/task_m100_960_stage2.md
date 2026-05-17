# Stage 2 보고서 — Task #960 디버그 추적 + Root cause 확정

- 이슈: [#960](https://github.com/edwardkim/rhwp/issues/960)
- 구현 V2: [task_m100_960_impl_v2.md](../plans/task_m100_960_impl_v2.md)

## 1. 디버그 instrument

`RHWP_DEBUG_PARA_TAC` env-gated:
- `src/renderer/composer.rs:compose_paragraph` — tac_controls + char_offsets + line_segs + ComposedLine dump
- `src/renderer/layout/paragraph_layout.rs:1724~` — per line/run 의 run_char_pos, run_char_end, run_tacs

Build: `cargo build --release` (성공). 디버그 instrument 는 env 미설정 시 zero overhead (single `std::env::var` 호출).

## 2. 실측 핵심 결과

### pi=117 text + char_offsets (요약)

```
text 64 codepoints, FFFC 없음
char_offsets gap 분석:
  idx 4→5: 12→21 (gap=8)  → ci=1 (f)    at pos 5
  idx 13→14: 29→38 (gap=8) → ci=2 (g)    at pos 14
  idx 29→30: 60→76 (gap=15)→ ci=3 (cases) at pos 30 ← '\n' 와 동일 codepoint
  idx 33→34: 79→88 (gap=8) → ci=4 (h=lim) at pos 34
```

### ComposedLine + layout 결과

| line_idx | char_start | run chars | has_lb | run_tacs (실측) | 정상 여부 |
|----------|-----------|-----------|--------|----------------|----------|
| 0 | 0  | 23 | false | [(5, ci=1), (14, ci=2)] | ✓ f, g |
| 1 | 23 | 7  | **true** | **[]** | **⚠️ cases 누락** |
| 2 | 31 | 8  | true | [(3, ci=4)] | ✓ h=lim |
| 3 | 40 | 24 | false | [] | ✓ |

## 3. Root cause

[src/renderer/layout/paragraph_layout.rs:1724-1727](src/renderer/layout/paragraph_layout.rs#L1724-L1727) filter:

```rust
.filter(|(pos, _, _)| *pos >= run_char_pos
    && (*pos < run_char_end || (is_last_run && *pos == run_char_end)))
```

- ls[1]: pos=30 == run_char_end=30, `is_last_run=false` → **cases 제외**
- ls[2]: pos=30 < run_char_pos=31 → **cases 제외**
- 어느 라인에서도 cases emit 안됨 → **완전 누락**

`run_char_end` 가 30 인 이유: `compose_lines` 가 ls[1] line_text 끝의 `\n` 을 strip 하여 run.text 가 7 chars (정의한다.\t\t) 가 됨. `run_char_end = 23 + 7 = 30`. cases 의 absolute pos = 30 (= \n 의 codepoint 위치) 와 정확히 boundary collide.

## 4. Issue 본문 vs 실제

Issue 본문: "cases y=380 / h=lim y=329 swap"
실제: **cases 누락 + h=lim ls[2] 정상 emit**. 사용자가 ls[2] 의 h=lim 을 cases 로 오인.

→ 결함의 **본질은 swap 이 아닌 cases 누락**.

## 5. Fix 안

Stage 2 구현 계획 V2 ([task_m100_960_impl_v2.md](../plans/task_m100_960_impl_v2.md)) 의 **Fix A** 권장:

```rust
let claim_boundary = is_last_run
    || (is_last_run_of_line(run_idx) && comp_line.has_line_break);
.filter(|(pos, _, _)| *pos >= run_char_pos
    && (*pos < run_char_end || (claim_boundary && *pos == run_char_end)))
```

적용 위치 3곳:
- line 1724 (run_tacs — 본 결함)
- line 1166 (estimation pass)
- line 1811 (각주/미주 markers)

## 6. 검증 자료

`RHWP_DEBUG_PARA_TAC=1 ./target/release/rhwp export-svg samples/3-11월_실전_통합_2022.hwp -p 1`

전체 라인별 emit 데이터:
```
TAC_LINE pi=117 line_idx=0 run_char_pos=0  run_char_end=23 y=329.6 run_tacs=[(5, 24.68, 1), (14, 24.68, 2)]
TAC_LINE pi=117 line_idx=1 run_char_pos=23 run_char_end=30 y=347.6 run_tacs=[]
TAC_LINE pi=117 line_idx=2 run_char_pos=31 run_char_end=39 y=379.4 run_tacs=[(3, 179.4, 4)]
TAC_LINE pi=117 line_idx=3 run_char_pos=40 run_char_end=64 y=407.5 run_tacs=[]
```

## 7. 다음 단계 (Stage 3)

Fix A 적용 → 단위 검증 → Stage 4 회귀 검증.
**Stage 3 진행 전 작업지시자 명시 승인 필요**.
