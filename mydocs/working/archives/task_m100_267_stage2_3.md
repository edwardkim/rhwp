# Task #267 Stage 2·3 완료 보고서: right tab 공백 처리 수정

> 완료 보고서 | 2026-04-24
> Branch: `local/task267`
> Stage: 2·3 / 4

---

## 1. 수정 내용

### Stage 2 — `text_measurement.rs` (경로 1, 3곳)

**수정 방법**: right tab(tab_type==1) 분기에서 `i+1` 대신 `seg_start` 사용.
leading space를 skip한 시작 인덱스를 호출부에서 계산하여 `measure_segment_from`에 전달.

```rust
let seg_start = { let mut s = i + 1; while s < chars.len() && chars[s] == ' ' && cluster_len[s] != 0 { s += 1; } s };
let seg_w = measure_segment_from(&chars, &cluster_len, seg_start, &char_width);
```

| 위치 | 내용 |
|------|------|
| EmbeddedTextMeasurer::compute_char_positions, inline_tabs L324 | seg_start 적용 |
| EmbeddedTextMeasurer::compute_char_positions, tab_stops L345 | seg_start 적용 + `eprintln![DEBUG_TAB_POS]` 3행 제거 |
| WasmTextMeasurer::compute_char_positions, tab_stops L656 | seg_start 적용 |

center tab(tab_type==2) 분기는 현행 `i + 1` 유지.

### Stage 3 — `paragraph_layout.rs` (경로 2·3, 2곳)

**수정 방법**: tab_type==1 match arm을 분리하여 right tab에서만 `run.text.trim_start()` 전달.

| 위치 | 내용 |
|------|------|
| `pending_right_tab_est` L809 | right tab arm: `estimate_text_width(run.text.trim_start(), &ts)` |
| `pending_right_tab_render` L1177 | right tab arm: `estimate_text_width(run.text.trim_start(), &text_style)` |

center tab(tab_type==2)은 `&run.text` 그대로 유지.

---

## 2. 검증 결과

### cargo test
```
963 passed; 0 failed (lib)
4 passed; 0 failed (svg_snapshot)
```

### cargo clippy --lib -- -D warnings
```
0 warnings
```

### KTX visual diff

| 페이지 | 상태 | match | 수정 전 |
|--------|------|-------|---------|
| 0  | ✅ PASS | 96.27% | 96.64% |
| 1  | ✅ PASS | 97.70% | 97.77% |
| 5  | ✅ PASS | 96.31% | 96.69% |
| 6  | ✅ PASS | 96.41% | 96.41% |
| 10 | ✅ PASS | 95.30% | 95.39% |
| **평균** | **전체 PASS** | **96.40%** | 96.58% |

전체 PASS 유지. 평균 -0.18%는 right tab 정렬 이동에 의한 정상 변화 (목차 페이지 번호 위치 이동).

---

## 3. 커밋

```
c3ca1fc fix: right tab 선행 공백 처리 — 경로 1/2/3 통일 (Task #267)
```

---

## 4. Stage 4 이행 조건

- [x] text_measurement.rs 3곳 수정 완료
- [x] eprintln![DEBUG_TAB_POS] 제거 완료
- [x] paragraph_layout.rs 2곳 수정 완료
- [x] cargo test 967 passed, 0 failed
- [x] clippy 0 warnings
- [x] KTX visual diff 전체 PASS (96.40%)
- [ ] **작업지시자 승인** → Stage 4 진행 (golden SVG 등록 + KTX 2페이지 육안 확인)

---
