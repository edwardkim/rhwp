---
issue: 630
milestone: m100
branch: local/task630
stage: 3 — 정정 1 적용 (`·` 측정 통일)
created: 2026-05-06
status: 완료 — 승인 대기
---

# Task #630 Stage 3 완료 보고서 — 정정 1 적용 (`·` 측정 통일)

## 1. 정정 내용

### 1-1. 변경 파일 / 위치

`src/renderer/layout/text_measurement.rs:859-862`

### 1-2. Diff

```rust
// Before
let is_halfwidth_punct = matches!(c,
    '\u{2018}'..='\u{2027}' | // ''‚‛""„‟†‡•‣․‥…‧ 구두점/기호
    '\u{00B7}'                 // · MIDDLE DOT
);

// After
// [Issue #630] U+00B7 (가운뎃점) 은 본 분기에서 제외 — 한컴 저장본의
// tab_extended 가 전각 측정 기반으로 산출되므로 반각 강제 시 right-tab
// 정렬이 8.67px 좌측 이탈. 폰트 메트릭 그대로 사용 (전각).
let is_halfwidth_punct = matches!(c,
    '\u{2018}'..='\u{2027}' // ''‚‛""„‟†‡•‣․‥…‧ 구두점/기호
);
```

영향:
- `·` 가 등록 폰트 메트릭에서 전각 (em_size) 으로 기록된 경우 그대로 전각 사용.
- 폰트 메트릭이 narrow 로 저장한 경우 (예: 일부 라틴 폰트) 는 그 폭 사용.
- 미등록 폰트 (UNREGISTERED_FONT) 는 별도 `is_narrow_punctuation` 분기 (font_size * 0.3 heuristic) 그대로 적용 — 본 정정 영향 없음.
- SVG `<circle>` 렌더 (Task #257) 는 cluster_advance 기반이라 자동으로 전각 advance 의 중앙에 그려짐 → 시각적으로 한컴 PDF 정합 향상.

## 2. 단위 테스트 결과

```
$ cargo test --lib --release test_630
test test_630_middle_dot_full_width_in_registered_font ... ok          # ← Stage 3 GREEN
test test_630_native_inline_tab_right_align ... FAILED                  # ← Stage 4 영역
```

`test_630_middle_dot_full_width_in_registered_font` GREEN 전환 — `·` 측정 통일 확인.

## 3. 통합 테스트 결과 (부분 효과)

```
$ cargo test --release --test issue_630
test test_630_aift_p4_toc_paren_alignment ... FAILED
  lines=23 min_x=600.25 max_x=601.39 spread=1.13px (예상 ≤1.0)
```

| 메트릭 | Before (Stage 1) | After Stage 3 | 변화 |
|--------|------------------|---------------|-----|
| `(페이지 표기)` 정렬 그룹 수 | 4 (592 / 600 / 601 / 293) | 1 (≈600~601 ±0.6) | -3 |
| 8.67px 이탈 라인 수 | 6 (1-1, 3-1, 3-4, 4-1, 7-2, 8-1) | **0** | -6 |
| spread | 308.49px (9.08 본 라인만) | **1.13px** | **−7.95px** |

aift p4 paren_x 분포 (Stage 3 후):
- 모든 24 라인 (정상 22 + 6-1 + 7-1 wrap) 이 `paren_x ≈ 600.25 ~ 601.39` 안에 들어옴
- `·` 포함 6 라인이 정확히 `601.00 ~ 601.08` 으로 정렬 — Before 의 ≈592 대비 정합 회복
- 6-1 라인 (`paren_x=600.25`) 만 다른 라인 대비 ≈0.7~1.1px 차이 — Stage 4 (RIGHT 탭 정확 매치) 후 흡수 예상

통합 테스트 허용 오차 1.0px 와 잔여 0.13px 차이. Stage 4 적용 후 GREEN 전환 예상.

## 4. 회귀 점검

### 4-1. 단위 테스트 (cargo test --lib --release)

```
test result: FAILED. 1135 passed; 1 failed; 2 ignored; 0 measured
```

- baseline 1134 → **1135 passed** (test_630_middle_dot_full_width_in_registered_font 추가 GREEN)
- 1 failed: test_630_native_inline_tab_right_align (Stage 4 영역, 의도된 RED)
- 본질 회귀 **0** — Stage 3 정정이 기존 1134 테스트에 영향 없음

### 4-2. svg_snapshot 회귀

```
test issue_267_ktx_toc_page ... ok                    # ← KTX (`·` 없음) 영향 없음
test issue_147_aift_page3 ... FAILED                   # ← 권위 fixture (예상)
test form_002_page_0 ... ok
test table_text_page_0 ... ok
test issue_157_page_1 ... ok
test render_is_deterministic_within_process ... ok
```

5/6 passed. issue_147 (aift p3) 는 본 정정의 권위 fixture — Stage 5 에서 시각 판정 후 `UPDATE_GOLDEN=1` 으로 갱신 예정. issue_267 (KTX TOC) 는 통과 — `·` 미출현 fixture.

### 4-3. WASM / 광범위 sweep / clippy

Stage 4 와 함께 묶어 Stage 5 에서 측정.

## 5. 산출물

- `src/renderer/layout/text_measurement.rs` (정정 1)
- `output/svg/task630_stage3/aift_004.svg` (시각 검증용)
- 본 단계별 보고서

## 6. 다음 단계 (Stage 4)

`text_measurement.rs:247` (estimate_text_width) + `:361` (compute_char_positions) — `tab_type = ext[2]` → `inline_tab_type(ext)` + match 분기 `1/2 → 2/3`. line 240-243 코멘트 갱신.

기대 효과:
- `test_630_native_inline_tab_right_align` GREEN
- `test_630_aift_p4_toc_paren_alignment` GREEN (잔여 0.13px 흡수)
- aift p4 paren_x 단일 그룹 (≈601, ±0.5) 으로 완전 정합

## 7. 승인 요청

Stage 3 정정 1 적용 완료, 단위 테스트 1번 GREEN + 통합 테스트 부분 GREEN (spread 9.08→1.13). Stage 4 (정정 2 — native tab_type 통일) 진행 승인 부탁드립니다.
