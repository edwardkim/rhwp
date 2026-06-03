# Task #469 Stage 2 완료 보고서

## 단계 목표

`src/renderer/layout.rs` 의 paragraph border 그룹 렌더링에서 `partial_start`/`partial_end` 시 `rect_y` 가 col_top 위(헤더선) 또는 col_bot 아래(꼬리말선) 로 확장되는 것을 차단.

## 변경 내용

`src/renderer/layout.rs:1735-1745` (block 내):

### Before

```rust
let top_pad = if stroke_width > 0.0 && !prev_touches { top_inset.max(DEFAULT_MIN_INSET) } else { top_inset };
let bot_pad = if stroke_width > 0.0 && !next_touches { bottom_inset.max(DEFAULT_MIN_INSET) } else { bottom_inset };
let rect_y = y_start - top_pad;
let rect_h = height + top_pad + bot_pad;
```

### After

```rust
let top_pad = if stroke_width > 0.0 && !prev_touches { top_inset.max(DEFAULT_MIN_INSET) } else { top_inset };
let bot_pad = if stroke_width > 0.0 && !next_touches { bottom_inset.max(DEFAULT_MIN_INSET) } else { bottom_inset };
// Task #469: cross-column / cross-page 로 이어진 partial 박스의 후속 부분은
// 이전/다음 컬럼에서 이미 inset 이 적용되었으므로 여기서 다시 col_top/col_bot
// 너머로 박스를 확장하면 안 된다 (헤더선/꼬리말선과 충돌).
// y_start/y_end 는 L1707 에서 col_top..col_bot 으로 이미 클램프됨.
let effective_top_pad = if is_partial_start { 0.0 } else { top_pad };
let effective_bot_pad = if is_partial_end { 0.0 } else { bot_pad };
let rect_y = y_start - effective_top_pad;
let rect_h = height + effective_top_pad + effective_bot_pad;
```

## 결과

**PASSED** — Stage 1 의 red 테스트가 green 으로 전환:
- 우측 단 (나) 박스 좌·우 세로선: `y1=211.65` (col_top, body top) 으로 변경
- 헤더 가로선 `y=196.55` 와 약 15.1 px 떨어진 정상 위치

## SVG 비교 (관련 라인)

수정 전:
```
<line x1="994.0" y1="196.55" x2="994.0" y2="1020.37" .../>
<line x1="593.4" y1="196.55" x2="593.4" y2="1020.37" .../>
```

수정 후:
```
<line x1="994.0" y1="211.65" x2="994.0" y2="1020.37" .../>
<line x1="593.4" y1="211.65" x2="593.4" y2="1020.37" .../>
```

## 다음 단계

Stage 3 — 전체 회귀 테스트 + 골든 SVG 영향 검토.
