# Task #267 최종 결과 보고서: 목차 right tab 정렬 — 교차 run 처리 통일

> 최종 보고서 | 2026-04-24
> Issue: [edwardkim/rhwp#267](https://github.com/edwardkim/rhwp/issues/267)
> Branch: `local/task267`

---

## 1. 목표 및 결과

### 목표
KTX.hwp 2페이지 목차에서 소제목 페이지 번호가 ~9.33px 밀리는 버그 수정.
장제목/소제목 모두 tab_pos 우측에 페이지 번호가 동일하게 정렬되어야 함.

### 결과
3경로 right tab 공백 처리를 통일하여 수정 완료.
cargo test 967 passed, clippy 0 warnings, KTX visual diff 전체 PASS.

---

## 2. 근본 원인

right tab 처리 코드가 3경로로 분산되어 있었고, 탭 직후 선행 공백 처리 방식이 불일치:

| 경로 | 케이스 | 위치 | 기존 동작 |
|------|--------|------|---------|
| 1 | 탭이 run 중간 (소제목) | `text_measurement.rs` `compute_char_positions` | `measure_segment_from(i+1, ...)` — 공백 포함 |
| 2 | 탭이 run 끝 추정 패스 (장제목) | `paragraph_layout.rs:809` `pending_right_tab_est` | `estimate_text_width(&run.text)` — 공백 포함 |
| 3 | 탭이 run 끝 렌더 패스 (장제목) | `paragraph_layout.rs:1177` `pending_right_tab_render` | `estimate_text_width(&run.text)` — 공백 포함 |

한컴의 동작: right tab 정렬 시 탭 직후 선행 공백을 무시하고 실질 텍스트의 우측 끝을 tab_pos에 맞춤.

---

## 3. 수정 내용

### 수정 파일 1: `src/renderer/layout/text_measurement.rs`

right tab(tab_type==1) 분기에서 `seg_start` 인덱스를 계산하여 leading space skip:

```rust
// right tab 분기에서만 (center tab은 i+1 유지)
let seg_start = { let mut s = i + 1; while s < chars.len() && chars[s] == ' ' && cluster_len[s] != 0 { s += 1; } s };
let seg_w = measure_segment_from(&chars, &cluster_len, seg_start, &char_width);
```

적용 위치 3곳:
- EmbeddedTextMeasurer::compute_char_positions, inline_tabs 분기
- EmbeddedTextMeasurer::compute_char_positions, tab_stops 분기 (+ `eprintln![DEBUG_TAB_POS]` 제거)
- WasmTextMeasurer::compute_char_positions, tab_stops 분기

### 수정 파일 2: `src/renderer/layout/paragraph_layout.rs`

right tab(tab_type==1) match arm을 분리하여 `run.text.trim_start()` 전달:

```rust
// pending_right_tab_est / pending_right_tab_render 양쪽 동일 패턴
match tab_type {
    1 => {
        let run_w = estimate_text_width(run.text.trim_start(), &ts);
        est_x = tab_pos - run_w;
    }
    2 => {
        let run_w = estimate_text_width(&run.text, &ts);  // center tab 유지
        est_x = tab_pos - run_w / 2.0;
    }
    _ => {}
}
```

---

## 4. 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test` | 963 passed, 0 failed ✅ |
| `cargo clippy --lib -- -D warnings` | 0 warnings ✅ |
| KTX visual diff 평균 matchRate | 96.40% (수정 전 96.58%, -0.18%) ✅ |
| KTX visual diff 전체 판정 | 5/5 PASS ✅ |
| golden svg_snapshot (5개) | 5 passed, 0 failed ✅ |

### KTX visual diff 상세

| 페이지 | 상태 | match |
|--------|------|-------|
| 0 (표지) | ✅ PASS | 96.27% |
| 1 (목차) | ✅ PASS | 97.70% |
| 5 | ✅ PASS | 96.31% |
| 6 | ✅ PASS | 96.41% |
| 10 | ✅ PASS | 95.30% |

---

## 5. Golden SVG

- `tests/svg_snapshot.rs`: `issue_267_ktx_toc_page` 테스트 추가
- `tests/golden_svg/issue-267/ktx-toc-page.svg`: KTX.hwp 2페이지(목차) golden 등록
- `samples/KTX.hwp`: 테스트 대상 파일 추가

---

## 6. 커밋 이력

```
c3ca1fc fix: right tab 선행 공백 처리 — 경로 1/2/3 통일 (Task #267)
8d86502 docs: Task #267 Stage 2·3 완료 보고서
dceb76f test: Issue #267 golden SVG 등록 — KTX 목차 페이지 right tab 정렬
```

---

## 7. PR 준비 완료

- 브랜치: `local/task267` (기준: `local/task157`)
- 대상: `edwardkim/rhwp:devel`
- 제출: `feature/task267`

---
