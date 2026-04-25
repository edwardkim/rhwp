# Task #324 Stage 2 v2 보고서 — 코드 수정 (정공법)

**기준일**: 2026-04-25
**브랜치**: `local/task324`
**단계**: 코드 수정 — 정공법 (옵션 C)

---

## 1. 메터링 가설 재검증

`compute_cell_line_ranges` 가 line 단위 누적 시 `spacing_before/after` 를 이중 계산하는 가설 → **오답**.

디버그 트레이스(env `RHWP_DBG_T324`):

```
[T324M] pi=2..27  sb=0.00  sa=0.00 ...
```

모든 paragraph 의 spacing_before/after = 0. 이중 계산은 발생하지 않음.

## 2. 진짜 원인 — vpos 컬럼 리셋

`p[15] ls[0] vpos=0` — vpos 가 cell 중간에서 0으로 리셋. vpos 는 **cell 시작부터의 누적 위치가 아님** → vpos 기반 위치 계산 불가능.

따라서 line 누적(`line_height + line_spacing`) 이 cell 내 콘텐츠 위치를 측정하는 **유일한 정답**. 함수 내부 `line_h` 합산은 정확.

## 3. 진짜 결함 — 가시성 결정 시 위치 정보 손실

`compute_cell_line_ranges` 는 다음 두 잔량을 추적:
- `offset_remaining` = content_offset - cum (이전 페이지 영역 잔량)
- `limit_remaining` = content_limit - cum (현재 페이지 잔량)

문제: 두 잔량이 0이 되면 cum 정보를 잃음. 중첩 표 atomic 문단의 가시성을 판단할 때:

- **페이지 1 (limit=443)**: line 누적이 443px 를 초과한 시점에서 `limit_remaining=0`. p[28] (cum~764) 도달 시 `limit_remaining > 0` 분기 false → **차단 누락 → 무조건 표시**.
- **페이지 2 (offset=443)**: 누적이 443px 를 넘은 후 `offset_remaining=0`. p[28] 의 위치는 cum=764 (offset 영역 밖) 인데도, `para_h <= content_offset` heuristic 로 \"이미 지나갔음\" 으로 잘못 판정.

추가 결함: `layout_partial_table` 에서 split-end 행의 (line_count, line_count) 마커 처리 분기 누락 — split-start 케이스만 처리.

## 4. 수정안 (적용)

### 4-1. `src/renderer/layout/table_layout.rs::compute_cell_line_ranges`

**Cumulative position 기반 가시성 결정**으로 재작성:

```rust
let mut cum: f64 = 0.0;
let has_offset = content_offset > 0.0;
let has_limit = content_limit > 0.0;

for each para:
  if line_count == 0 || has_table_in_para {  // atomic
    let para_end_pos = cum + para_h;
    cum = para_end_pos;
    let was_on_prev = has_offset && para_end_pos <= content_offset;
    let exceeds_limit = has_limit && para_end_pos > content_limit;
    push if was_on_prev || exceeds_limit { (n, n) } else { (0, n) };
    continue;
  }
  // 일반 문단: line 단위 누적 + 위치 기반
  for each line:
    let line_end_pos = cum + line_h;
    if has_offset && line_end_pos <= content_offset { skip; continue; }
    if has_limit && line_end_pos > content_limit { break; }
    cum = line_end_pos; para_end = li + 1;
```

핵심 개선:
- 잔량이 아닌 절대 누적 위치(cum)로 판정 → 정보 손실 없음
- 중첩 표 atomic 문단을 명시적으로 \"한쪽 페이지에만\" 배치
- 일반 / atomic 분기를 line_count==0 와 has_table_in_para 통합 처리

### 4-2. `src/renderer/layout/table_partial.rs` (라인 ~589)

split-end 행의 atomic 스킵 분기 추가:

```rust
} else if has_nested_table && is_in_split_row && split_end_content_limit > 0.0 {
    let nested_h: f64 = ...;
    content_y_accum += nested_h;
    continue;
}
```

## 5. 검증 결과

### SVG 출력

| 항목 | Baseline | After | 차이 |
|------|----------|-------|------|
| page 1 size | 269,233 B | 227,903 B | **-41,330 B** (인너 표 제거됨) |
| page 1 `<rect>` 수 | 73 | 70 | -3 (인너 표 border) |
| page 2 size | 266,879 B | 266,879 B | 변화 없음 (인너 표는 이미 page 2 에도 렌더링되어 있었음) |

### 회귀

- `cargo test --release`: **992 unit + 71 integration 통과**, 1 골든 갱신 (form_002_page_0)
- `cargo clippy --release -- -D warnings`: **클린**

골든 변경: `tests/golden_svg/form-002/page-0.svg` 138줄 → 0줄 변경 (실제로는 인너 표 path 제거 = -138줄). 본 fix 의 의도된 변경이므로 골든 갱신.

## 6. 다음 단계

Stage 3 — PDF 와의 시각 비교 및 최종 보고서 작성.
