# Task #901 Stage 5 보고서 — paragraph 0 vertical drift fix

**Stage**: 5
**상태**: 정합 ✅ (HWP 영역, 1402 test 회귀 없음)

## 1. 진단

paragraph 0 의 8 line_segs (vpos 0, 4480, 8960, ..., 31360, 각 4480 HU 증가) 가 paragraph_layout 에서 8회 y advance → 한컴 PDF 대비 paragraph 0 vertical 2배 누적, 본문 paragraph (예: SK하이닉스) 가 ~150 px 아래로 밀림.

### 1.1 compose_lines 결과 (HWP)

| idx | cs | sw | vpos | runs.len | text |
|-----|----|----|------|---------|----|
| 0 | 24470 | 2570 | 0 | 1 | " " |
| 1 | 39123 | 3397 | 4480 | 1 | "우" |
| 2 | 24470 | 2570 | 8960 | 0 | "" |
| 3 | 39123 | 3397 | 13440 | 1 | "리" |
| 4 | 24470 | 2570 | 17920 | 0 | "" |
| 5 | 39123 | 3397 | 22400 | 1 | "나" |
| 6 | 24470 | 2570 | 26880 | 0 | "" |
| 7 | 39123 | 3397 | 31360 | 1 | "라" |

cs=24470 (LEFT narrow zone, 의자 그림 사이) 의 line 들은 모두 **runs 가 empty** — 한컴이 사전 wrap zone 계산만 인코딩하고 실제 텍스트는 cs=39123 (RIGHT narrow zone, 작은 의자 우측) 에 배치.

### 1.2 ROOT CAUSE

paragraph_layout 의 line iteration 이 empty runs line 도 y advance → "phantom" line 이 vertical space 차지.

## 2. Fix

`src/renderer/layout/paragraph_layout.rs:2833`:

```rust
// [Task #901 Stage 5] wrap zone paragraph 의 empty-runs line 은 y advance 건너뜀.
let skip_advance_empty_wrap = has_picture_shape_square_wrap
    && comp_line.runs.is_empty();
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else if skip_advance_empty_wrap {
    // no advance
} else {
    let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
    y += line_height + line_spacing_px;
}
```

### 2.1 Why

`has_picture_shape_square_wrap = true` 인 paragraph (자기 Picture/Shape Square wrap 그림 보유) 만 영향. 일반 paragraph 의 empty 빈 줄 (text 없는 빈 paragraph) 은 영향 없음.

### 2.2 결과 (pic2.hwp)

| 항목 | 이전 (Stage 1+2+3) | 이후 (Stage 5) | 한컴 PDF |
|------|---------------|---------|---------|
| 우 y | 224 | 224 | 167 |
| 리 y | 343 | **283** | 242 |
| 나 y | 463 | **343** | 316 |
| 라 y | 582 | **403** | 391 |
| 대한민국 y | 642 | **485** | 465 |
| SK하이닉스 y | 788 | **631** | 538 |
| paragraph 0 line gap | 119 px | **60 px** | 60 px ✅ |

paragraph 0 line spacing 한컴 정합 완료. 본문 paragraph 들이 한컴 PDF 위치로 ~150 px 이동 (drift 대폭 감소).

## 3. 회귀 검증

- ✅ `cargo test --release --all-targets`: **1402 passed, 0 failed**
- ✅ pic2.hwp 페이지 수 유지 (2 페이지)
- ✅ pic2-2018.hwp (한컴오피스 2018 재저장): 동일 결과
- ⚠️ pic2.hwpx / pic2.owpml: pre-existing 별도 parser bug (compose_lines 가 모든 chars 를 line 0 에 packing). Stage 5 와 무관, 변화 없음

## 4. 잔존 차이

- paragraph 22 LAYOUT_OVERFLOW 88.8 px (페이지 2 시작 paragraph 가 페이지 1 끝에 잘못 배치) — 별도 layout drift 이슈
- pic2.hwp 우 y=224 vs 한컴 167 (57 px = 15mm 잔존 offset) — 페이지 head 위치 차이 또는 다른 spacing 처리 차이
- HWPX 의 별도 parser bug — 별도 이슈 #903 으로 분리 제안

## 5. 다음 단계

Stage 6 (회귀 검증 + 최종 보고서 + PR) 진행.
