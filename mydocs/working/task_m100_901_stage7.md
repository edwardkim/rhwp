# Task #901 Stage 7 보고서 — pagination format_paragraph whitespace skip 정합

**Stage**: 7
**상태**: 정합 진전 ✅ (paragraph 11 page 1 정합, 1402 test 회귀 없음)

## 1. 진단

Stage 5/6 의 `paragraph_layout` 의 visual skip 은 적용되었으나, `typeset.rs::format_paragraph` 의 height 계산은 별개. pagination engine 이 paragraph 0 의 8 line 전체 height (~478 px) 으로 누적 → page 1 fill 이 시각 렌더링보다 빠르게 발생 → paragraph 11 (올해 확정·지급된 PS...) 이 page 2 로 잘못 분할.

paragraph_layout visual: paragraph 0 = 4 visible lines × ~60 = 240 px
pagination height: paragraph 0 = 8 lines × ~60 = 478 px

**Drift = 238 px** → page 1 가용 space 감소 → 2 개 paragraph (11, 일부 후속) 잘못 page 2 로 push.

## 2. Fix

`src/renderer/typeset.rs::format_paragraph` 에 `paragraph_layout` 와 동일한 skip 로직 mirror:

```rust
let has_picture_shape_square_wrap = para.controls.iter().any(|c| ...);
...
.map(|line| {
    let runs_all_whitespace = line.runs.iter().all(|r| r.text.trim().is_empty());
    if has_picture_shape_square_wrap && runs_all_whitespace {
        return (0.0, 0.0);  // height 0, line_spacing 0
    }
    ...
})
```

## 3. 결과

- ✅ paragraph 11 "올해 확정·지급된 PS..." 이 page 1 로 이동 (한컴 정합)
- ✅ LAYOUT_OVERFLOW: 이전 paragraph 20+21+22 → 현재 paragraph 22 만 (2 paragraph 더 page 1 에 fit)
- ✅ `cargo test --release --all-targets`: **1402 passed, 0 failed**

## 4. 종합 결과 (Stage 1~7)

pic2.hwp page 1 정합 progress:

| 항목 | Baseline (수정 전) | Stage 1+2+3+5+6+7 |
|------|-----------|-------------------|
| paragraph 0 우 위치 | 좌측 (잘못) | 우측 세로 ✅ |
| paragraph 1 대한민국 | 좌측 | 우측 ✅ |
| paragraph 0 line gap | 119 px | 60 px (한컴 정합) ✅ |
| paragraph 7 SK하이닉스 y | 788 | 571 (PDF 538, +33 px) |
| paragraph 11 올해 확정 | page 2 | **page 1** ✅ |
| 1402 test | passed | passed ✅ |

## 5. 잔존 차이

- ~30 px 공통 vertical offset (~8mm) — `col_area.y` 가 body_top 대신 picture vertOffset 에서 시작
- HWPX parser bug (compose_lines packing all chars to line 0) — Stage 5/6/7 의 영향 받지 못함, 별도 이슈
- paragraph 22 overflow 88.8 px (마지막 paragraph, 별도 issue)

## 6. 다음 단계

Stage 8: PR 마무리 결정 또는 잔존 offset 추가 fix.
