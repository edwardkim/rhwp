# Task #901 Stage 6 보고서 — whitespace-only line skip 확장

**Stage**: 6
**상태**: 정합 진전 ✅ (HWP, 1402 test 회귀 없음)

## 1. 진단

Stage 5 (empty-runs skip) 적용 후 paragraph 0 의 idx=0 (cs=24470 LEFT narrow wrap zone) 의 line 은 공백 한 글자 " " 만 가지므로 `runs.is_empty()` 조건에 걸리지 않아 여전히 y advance 됨.

paragraph 0 의 첫 line_seg (idx=0) 는 한컴 viewer 의 wrap zone 좌측 영역 표시 — 텍스트 미배치, vertical 공간 미차지.

## 2. Fix

`src/renderer/layout/paragraph_layout.rs:2837`:

```rust
// runs 전체가 공백/empty 이면 skip
let runs_all_whitespace = comp_line.runs.iter().all(|r| r.text.trim().is_empty());
let skip_advance_empty_wrap = has_picture_shape_square_wrap
    && runs_all_whitespace;
```

`has_picture_shape_square_wrap` gate 로 wrap zone host paragraph 만 영향.

## 3. 결과

paragraph y 위치 변화 (pic2.hwp page 1, 96 DPI):

| paragraph | Stage 5 | Stage 6 | PDF 환산 (96 DPI) | Δ vs PDF |
|-----------|---------|---------|--------|------|
| 0 우 | 224 | **164** | 133 | +31 px |
| 1 대 | 485 | **425** | 372 | +53 px |
| 2 서 | 524 | **465** | 431 | +34 px |
| 3 성 | 546 | **486** | 452 | +34 px |
| 7 SK하 | 631 | **571** | 538 | +33 px |
| 9 글 | 746 | **686** | 643 | +43 px |
| 11 올해 | 810 | **750** | 793 | -43 px (overshoot) |

paragraph 0~9 의 vertical 위치 60 px 추가 상향. paragraph 11 (올해) 가 PDF 보다 위로 이동 — paragraph 10 (empty, controls=0) 의 height 가 PDF 대비 부족한 별도 이슈 추정.

잔존 +30~50 px 오프셋 (~10mm) — `col_area.y` 가 body_top (75.6 px) 대신 paragraph 0 picture 의 vertOffset (132.3 px) 에서 시작하는 별도 이슈로 추정 (이 부분은 별도 layout drift).

## 4. 회귀 검증

- ✅ `cargo test --release --all-targets`: **1402 passed, 0 failed**
- ✅ pic2.hwp 페이지 수 유지 (2 페이지)
- ⚠️ pic2.hwpx: 사전 별도 parser bug 영향 (compose_lines packing) — Stage 6 와 무관

## 5. 다음 단계

Stage 7+: 잔존 +30 px (col_area.y offset) + paragraph 11 overshoot 분석. 또는 현 상태로 PR 마무리 후 별도 이슈 분리.
