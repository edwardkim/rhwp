# Task #969 Stage 3 — D6 패치 적용 + 검증

- 이슈: [#969](https://github.com/edwardkim/rhwp/issues/969)
- 선행: [Stage 2 구현 계획서](../plans/task_m100_969_impl.md)
- 브랜치: `local/task969`

## 1. D6 패치

[src/renderer/typeset.rs:1190-1228](src/renderer/typeset.rs#L1190-L1228) — `format_paragraph` 의 composed branch:

```rust
let recompute_lh = max_fs > 0.0 && raw_lh < max_fs;
let lh = if recompute_lh {
    // Percent/Fixed/SpaceOnly/Minimum 으로 lh 재계산
    ...
} else {
    raw_lh
};
// [Task #969] lh 재계산이 ParaShape ls_type 기반으로 일어났다면
// preset 의 line_spacing 은 이미 재계산된 lh 에 흡수된 값 (e.g. HWPX
// 의 linesegArray: lh=font, ls=extra) → 별도 가산 시 double-count
// (160% 가 lh+ls 양쪽). HWPX +8 페이지 inflate (sample16-hwp5) 원인.
let line_spacing_px = if recompute_lh {
    0.0
} else {
    hwpunit_to_px(line.line_spacing, self.dpi)
};
(lh, line_spacing_px)
```

## 2. 검증 결과

### 페이지 수
| 변종 | Pre-D6 | Post-D6 | 변동 |
|------|--------|---------|------|
| HWPX sample16-hwp5 | 72 | **71** | -1 |
| HWP5 sample16-hwp5 | 62 | 62 | 0 |
| HWP3 sample16 | 64 | 64 | 0 |
| PDF (정답) | 64 | 64 | — |

→ HWPX 1 페이지 감소. **목표 (64 페이지) 대비 +7 페이지 잔존**.

### fmt_total 정합
| | Pre-D6 | Post-D6 |
|---|--------|---------|
| HWPX | 25395.5 px | 36919.6 px (실측 변동) |
| HWP5 | 24482.8 px | 37002.0 px |
| Diff | +912.7 px | **-82.4 px** (거의 정합) |

→ format 차원에서는 정합. **잔존 drift 는 typeset_paragraph 의 page break 결정 / table path 영역**.

### pi=395 비교 (대표 케이스)
| | Pre-D6 | Post-D6 | HWP5 |
|---|--------|---------|------|
| lh_sum | 27.7 | 27.7 | 27.7 |
| ls_sum | 10.4 | **0.0** | 0.0 |
| fmt_total | 53.2 | **42.8** | 42.8 |

→ 정확히 HWP5 와 정합.

## 3. D8 시도 (tolerance) 결과

`raw_lh < max_fs * 0.95` 으로 recompute 조건 완화 → 효과 0 페이지. 제거.

(이유: HWPX preset lh=1299 vs max_fs=17.33 → recompute 발동. tolerance 적용해도 cur_h 변동 미미.)

## 4. 잔존 drift 의 root cause 후보

D6 적용 후에도 +7 페이지 차이 남음. format_paragraph 영역 밖:

- **typeset_paragraph 의 page break 결정 로직**: cur_h overflow 판단, safety margin, vpos_overflow, next_substantial 등 5중 AND 조건
- **table path (typeset_table_paragraph)**: pi=394 가 HWPX 에서 drift log 누락 (has_table=true 분기, 별도 path)
- **wrap_around / vpos-reset 상태 머신**: HWPX vs HWP5 의 wrap zone 처리 미세 차이
- **HWPX vs HWP5 의 IR 구조 차이**: paragraph 0 의 `secd` 컨트롤 (HWP5 +1 개), 7건 pos 차이

→ **별도 task 분리 권장**.

## 5. 결론

D6 는 진정한 bug fix (line_spacing double-count when `ls_type=Percent`) 로 commit 가치 있음. +8 페이지 inflate 의 1/8 해소. 잔존 +7 페이지는 별도 깊은 분석 필요.
