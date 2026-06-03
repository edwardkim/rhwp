# Task #267 Stage 1 완료 보고서: 코드 조사

> 완료 보고서 | 2026-04-24
> Branch: `local/task267`
> Stage: 1 / 4 — 코드 조사

---

## 1. 조사 결과 요약

right tab 처리 코드를 전체 조사하여 수정 대상 위치를 완전히 파악했다.

---

## 2. 경로 1 — `compute_char_positions` 내 seg_w 위치

### 2.1 EmbeddedTextMeasurer (native 빌드, cargo test 대상)

파일: `src/renderer/layout/text_measurement.rs`

| # | 위치 | 분기 | 내용 | 비고 |
|---|------|------|------|------|
| 1 | L324 | inline_tabs, right tab (tab_type==1) | `seg_w = measure_segment_from(..., i+1, ...)` | 수정 필요 |
| 2 | L345 | tab_stops, right tab (tab_type==1) | `seg_w = measure_segment_from(..., i+1, ...)` | 수정 필요 + debug print 제거 |

**추가 발견 — 잔류 디버그 출력 (L347-348)**:
```rust
if tab_type == 1 {
    eprintln!("[DEBUG_TAB_POS] RIGHT tab: abs_x={:.2}, tab_pos={:.2}, ...");
}
```
→ Stage 2에서 seg_w 수정과 함께 반드시 제거한다.

### 2.2 WasmTextMeasurer (WASM/브라우저 빌드)

파일: `src/renderer/layout/text_measurement.rs`

| # | 위치 | 분기 | 내용 | 비고 |
|---|------|------|------|------|
| 3 | L656 | tab_stops, right tab (tab_type==1) | `seg_w = measure_segment_from(..., i+1, ...)` | 수정 필요 |

**주의**: WasmTextMeasurer의 `compute_char_positions`는 `inline_tabs` 분기가 없음.
`has_custom_tabs`(= `find_next_tab_stop`) 단일 경로만 존재.
→ EmbeddedTextMeasurer의 inline_tabs L324에 대응하는 WASM쪽 코드는 없으므로 L656만 수정.

---

## 3. 경로 2·3 — `pending_right_tab_est/render` 위치

파일: `src/renderer/layout/paragraph_layout.rs`

| # | 위치 | 함수 패스 | 현재 코드 | 수정 방향 |
|---|------|----------|----------|---------|
| 4 | L809-816 | 추정 패스 (est) | `estimate_text_width(&run.text, &ts)` | `run.text.trim_start()` 전달 |
| 5 | L1177-1184 | 렌더 패스 (render) | `estimate_text_width(&run.text, &text_style)` | `run.text.trim_start()` 전달 |

현재 코드 (L809-816):
```rust
if let Some((tab_pos, tab_type)) = pending_right_tab_est.take() {
    ts.line_x_offset = est_x;
    let run_w = estimate_text_width(&run.text, &ts);
    match tab_type {
        1 => est_x = tab_pos - run_w,
        2 => est_x = tab_pos - run_w / 2.0,
        _ => {}
    }
}
```

현재 코드 (L1177-1184):
```rust
if let Some((tab_pos, tab_type)) = pending_right_tab_render.take() {
    text_style.line_x_offset = x - col_area.x;
    let next_w = estimate_text_width(&run.text, &text_style);
    match tab_type {
        1 => x = col_area.x + tab_pos - next_w,
        2 => x = col_area.x + tab_pos - next_w / 2.0,
        _ => {}
    }
}
```

---

## 4. `estimate_text_width` 내 seg_w — 수정 불필요 판단

`estimate_text_width` 자체도 내부에서 tab 처리 시 right tab seg_w를 계산한다 (L221, L241 inline_tabs/tab_stops). 그러나 경로 2/3 수정은 **호출부에서 `run.text.trim_start()`를 전달**하는 방식으로 처리하므로, estimate_text_width 자체는 수정하지 않는다.

---

## 5. 수정 방안 확정

### 경로 1 — `measure_segment_from` 호출부에서 공백 skip

`measure_segment_from`의 시그니처를 변경하지 않고, **호출부에서 leading space를 skip한 시작 인덱스**를 계산하여 전달한다.

```rust
// right tab (tab_type == 1) 분기에서만 적용:
let seg_start = {
    let mut s = i + 1;
    while s < chars.len() && chars[s] == ' ' && cluster_len[s] != 0 {
        s += 1;
    }
    s
};
let seg_w = measure_segment_from(&chars, &cluster_len, seg_start, &char_width);
```

center tab (tab_type == 2) 분기는 현행 `i + 1` 유지 — 공백 포함 폭이 맞을 수 있음.

적용 위치: EmbeddedTextMeasurer L324, L345 / WasmTextMeasurer L656 (총 3곳)

### 경로 2·3 — `run.text.trim_start()` 전달

```rust
// L809: pending_right_tab_est
let run_w = estimate_text_width(run.text.trim_start(), &ts);

// L1177: pending_right_tab_render
let next_w = estimate_text_width(run.text.trim_start(), &text_style);
```

tab_type == 1 분기만 수정 (현재 match arm 내부가 아니라 `let` 바인딩이므로 모든 tab_type에 trim 적용됨 → center tab도 trim됨). 수행계획서 방침에서 center tab은 현재 동작 유지 지시. 따라서 tab_type별로 분기 후 각 arm에서 처리.

**구현계획서에서 정확한 분기 방법 확정 예정.**

---

## 6. 수정 파일 및 위치 요약

| 파일 | 위치 | 내용 |
|------|------|------|
| `text_measurement.rs` | L324 (EmbeddedTextMeasurer, inline_tabs) | seg_start 공백 skip |
| `text_measurement.rs` | L345-350 (EmbeddedTextMeasurer, tab_stops) | seg_start 공백 skip + eprintln 제거 |
| `text_measurement.rs` | L656 (WasmTextMeasurer, tab_stops) | seg_start 공백 skip |
| `paragraph_layout.rs` | L809-816 (pending_right_tab_est) | right tab 분기만 trim_start |
| `paragraph_layout.rs` | L1177-1184 (pending_right_tab_render) | right tab 분기만 trim_start |

총 5개 위치 수정. center tab 분기는 현행 유지.

---

## 7. 테스트 현황

```
cargo test: 967 passed, 0 failed (수정 전 기준)
```

---

## 8. Stage 1 → Stage 2 이행 조건

- [x] 3경로 모든 코드 위치 파악 완료
- [x] `eprintln!("[DEBUG_TAB_POS]")` 위치 확인 (L347-348)
- [x] WasmTextMeasurer inline_tabs 분기 없음 확인
- [x] 수정 방안 (seg_start 공백 skip, trim_start 전달) 확정
- [ ] **작업지시자 승인** → Stage 2 진행 (구현계획서 작성 후 승인 요청)

---
