---
issue: 630
milestone: m100
branch: local/task630
created: 2026-05-06
status: 구현계획 — 승인 대기
---

# Task #630 구현계획서

## 정정 영역 정밀화 (수행계획서 → 구현계획서 보강)

수행계획서의 "본질 결함 2가지" 를 코드 레벨로 확정한다.

### 정정 1: `is_halfwidth_punct` 에서 `U+00B7` 제외

**파일**: `src/renderer/layout/text_measurement.rs:859-862`

**Before**:
```rust
let is_halfwidth_punct = matches!(c,
    '\u{2018}'..='\u{2027}' |
    '\u{00B7}'                 // · MIDDLE DOT
);
```

**After**:
```rust
let is_halfwidth_punct = matches!(c,
    '\u{2018}'..='\u{2027}'    // · (U+00B7) 제외 — Task #630
);
```

영향: 폰트 메트릭의 `·` glyph_w 가 em_size 이상이면 그대로 사용 (전각). 폰트가 자체적으로 narrow 로 저장한 경우 (예: 일부 라틴 폰트) 는 그대로 narrow 폭. 한컴 표준 한글 폰트 (돋움체/바탕 등) 는 전각 인코딩이므로 17.34px 가 됨. SVG `<circle>` 렌더는 cluster_advance 의 중앙에 그려지므로 전각 advance 의 중앙으로 이동.

### 정정 2: native 경로 `tab_type = ext[2]` → `inline_tab_type(ext)` + match 분기 정합

**중요한 추가 발견**: native 경로는 `tab_type = ext[2]` (raw u16) 으로 받으면서 match 분기는 `1 => RIGHT, 2 => CENTER` (enum 0/1/2 가정) — 코드 자체가 모순. raw u16 RIGHT (= 515) 와 enum 1 은 절대 매치 못하므로 사실상 항상 LEFT fallback. WASM 경로는 `inline_tab_type` (high-byte = enum+1) + `2 => RIGHT, 3 => CENTER` 로 정합.

**파일 1**: `src/renderer/layout/text_measurement.rs:244-262` (estimate_text_width)

**Before**:
```rust
if tab_char_idx < style.inline_tabs.len() {
    let ext = &style.inline_tabs[tab_char_idx];
    let tab_width_px = ext[0] as f64 * 96.0 / 7200.0;
    let tab_type = ext[2];                    // ← raw u16
    let tab_target = total + tab_width_px;
    match tab_type {
        1 => { /* RIGHT */ ... }              // ← never matched (raw RIGHT = 515)
        2 => { /* CENTER */ ... }
        _ => { total = tab_target.max(total); }
    }
}
```

**After**:
```rust
if tab_char_idx < style.inline_tabs.len() {
    let ext = &style.inline_tabs[tab_char_idx];
    let tab_width_px = ext[0] as f64 * 96.0 / 7200.0;
    let tab_type = inline_tab_type(ext);      // ← high-byte = enum+1
    let tab_target = total + tab_width_px;
    match tab_type {
        2 => { /* RIGHT */ ... }              // ← enum+1: 2 = RIGHT
        3 => { /* CENTER */ ... }             // ← enum+1: 3 = CENTER
        _ => { total = tab_target.max(total); } // ← LEFT(1)/DECIMAL(4)
    }
}
```

**파일 2**: `src/renderer/layout/text_measurement.rs:358-376` (EmbeddedTextMeasurer::compute_char_positions, inline_tabs branch)

위와 동일한 패턴 적용. `tab_type = ext[2]` → `inline_tab_type(ext)`, `match 1/2` → `match 2/3`.

**코멘트 정리**: line 240–243 의 "네이티브 측 일관성 복원은 별도 이슈로 추적 (Task #296 범위 외)" 코멘트를 본 Task #630 으로 업데이트.

### 정정 영역 외 (변경 없음)

- WASM `WasmTextMeasurer::compute_char_positions` (line 615–) — 이미 정합, 변경 없음
- `has_custom_tabs` 분기 (TabDef 기반, line 263+/378+) — 본 Task 영향 영역 아님 (HWP TabDef 의 tab_type 은 이미 0/1/2 enum)
- `is_halfwidth_punct` 의 `'\u{2018}'..='\u{2027}'` 범위 (스마트 따옴표 등) — 보존

## 단계 구성 (5 단계, TDD)

### Stage 1 — 회귀 베이스라인 측정 (Before snapshot)

**목표**: 정정 전 상태를 정량 보존하여 Stage 5 의 회귀 검증 기준선 확보.

작업:
1. `samples/aift.hwp` 4페이지 SVG 재생성 → `output/svg/task630_before/aift_004.svg`
2. `(` 위치 분포 추출: `·` 포함/미포함 라인별 표 (현재 8.67px 차이 정량 보존)
3. `samples/` 164 fixture 페이지네이션 sweep — 페이지 수 baseline (회귀 0 확인 기준)
4. `cargo test --lib --release` 베이스라인 (현재 1134 passed 가정, 직전 PR 사이클 결과 참조)
5. issue-147 / issue-267 golden SVG snapshot 보존
6. KTX 목차 / 기타 dot leader 사용 fixture (`·` 가 leader 로 등장하는 케이스) sweep

산출물: `mydocs/working/task_m100_630_stage1.md` + `output/svg/task630_before/`

승인 게이트: 베이스라인 정량 + 회귀 위험 영역 식별 보고 → 승인 후 Stage 2.

### Stage 2 — 단위 테스트 (RED)

**목표**: 정정 전 RED 상태의 회귀 테스트 작성. Stage 3 / Stage 4 정정으로 GREEN 전환 결정적 검증.

테스트 항목 (`tests/integration_tests.rs` 또는 `text_measurement.rs::tests`):

1. **`test_630_middle_dot_full_width`** (text_measurement 단위)
   - 한컴 표준 폰트 (돋움체) 에서 `·` (U+00B7) 측정폭이 `em_size` (전각) 인지 검증
   - 현재 RED: 8.67px. After: 17.34px (font_size=17.33 기준)

2. **`test_630_native_inline_tab_right_align`** (text_measurement 단위)
   - 인라인 탭 type=RIGHT (raw `(2<<8)|3`) + `·` 포함 텍스트 → after-tab segment 가 tab_target 기준 right-aligned
   - 현재 RED: LEFT fallback 으로 tab_target 에 left-aligned. After: tab_target - seg_w 에 left-edge

3. **`test_630_aift_p4_toc_alignment`** (통합 테스트, golden SVG 기반)
   - `samples/aift.hwp` 페이지 4 SVG에서 `(` x 좌표 모든 라인 ±0.5px 정렬
   - 현재 RED: `·` 포함 6 라인이 8.67px 좌측 이탈. After: 모든 라인 정렬

작업: 단위 테스트 추가 → `cargo test test_630` 실행 → RED 확인 → 승인.

산출물: `mydocs/working/task_m100_630_stage2.md` + 회귀 테스트 코드

승인 게이트: RED 확인 → 승인 후 Stage 3.

### Stage 3 — 정정 1 적용 (`·` 측정 통일)

**목표**: `is_halfwidth_punct` 에서 `U+00B7` 제외 → 측정폭 한컴 정합.

작업:
1. `text_measurement.rs:859-862` 1 줄 제거 + 코멘트 갱신 (Task #630 reason)
2. `cargo test --lib --release` — `test_630_middle_dot_full_width` GREEN 확인
3. `test_630_aift_p4_toc_alignment` 부분 GREEN / 잔존 케이스 측정 (정정 2 필요한 영역)
4. golden SVG 차이 분석 — issue-147/267, KTX 목차, `·` 가 leader 로 사용되는 fixture
5. WASM 빌드 — Docker WASM 사이즈 변동 확인

승인 게이트: 정정 1 효과 + 잔존 영역 보고 → 승인 후 Stage 4.

산출물: `mydocs/working/task_m100_630_stage3.md`

### Stage 4 — 정정 2 적용 (native tab_type 통일)

**목표**: native 경로 `tab_type = ext[2]` → `inline_tab_type(ext)` + match 분기 enum+1 정합.

작업:
1. `text_measurement.rs:247` (estimate_text_width) `let tab_type = ext[2];` → `let tab_type = inline_tab_type(ext);` + match `1/2` → `2/3`
2. `text_measurement.rs:361` (EmbeddedTextMeasurer::compute_char_positions) 동일 변경
3. line 240–243 코멘트 갱신 — "Task #296 범위 외" → "Task #630 정정"
4. `cargo test --lib --release` — `test_630_native_inline_tab_right_align` + `test_630_aift_p4_toc_alignment` 모두 GREEN
5. `cargo test issue_147 issue_267` — 코드 코멘트가 명시한 회귀 위험 영역 검증
6. WASM 빌드 — 사이즈 변동 확인

승인 게이트: 모든 단위 테스트 GREEN + issue-147/267 회귀 0 → 승인 후 Stage 5.

산출물: `mydocs/working/task_m100_630_stage4.md`

### Stage 5 — 광범위 회귀 검증 + 시각 판정 보고

**목표**: 164 fixture / 1614 페이지 sweep + golden SVG diff + 시각 판정 자료 정비.

작업:
1. `samples/` 164 fixture 페이지네이션 sweep — 페이지 수 회귀 0 확인
2. `(` 위치 정량 측정: aift p4 + KTX 목차 등 권위 케이스 Before/After 표
3. SVG byte 차이 분포 분석 — 의도된 차이 vs 회귀
4. 시각 판정 자료 (`output/svg/task630_after/`) 정비 — Before/After 비교
5. WASM 사이즈 / `cargo test --lib --release` 최종 / `clippy 0` / `build --release`
6. 광범위 sweep 의 SVG 차이가 모두 `·` 인접 right-tab 정합 정정인지 분석

승인 게이트: 작업지시자 시각 판정 ★ 통과 → 최종 결과 보고서 작성 → merge 절차.

산출물:
- `mydocs/working/task_m100_630_stage5.md`
- `mydocs/report/task_m100_630_report.md` (최종)
- `output/svg/task630_before/` / `output/svg/task630_after/`

## 회귀 차단 설계 정합성

| 원칙 | 적용 |
|------|------|
| `feedback_rule_not_heuristic` | HWP 표준 룰 (`·` 측정 통일, tab_type 인코딩 정합) 단일 룰 / 분기 추가 0 |
| `feedback_essential_fix_regression_risk` | 5 단계 분리 + Stage 1/2 베이스라인 + 단위 테스트 RED 우선 + 광범위 sweep |
| `feedback_hancom_compat_specific_over_general` | `is_halfwidth_punct` 에서 U+00B7 만 제외 (스마트 따옴표 보존) |
| `feedback_pdf_not_authoritative` | PDF 는 보조 ref, 한컴 2010/2020 환경 차이 별도 점검 |
| 케이스별 명시 가드 | WASM 경로와 동일 코드 패턴 사용 (이미 검증된 정합 동작 재사용) |

## 검증 명령

```bash
# Stage 1 베이스라인
cargo test --lib --release 2>&1 | tail -5
./target/release/rhwp export-svg samples/aift.hwp -p 3 -o output/svg/task630_before/

# Stage 2 RED
cargo test --lib --release test_630 2>&1 | tail -10  # FAIL 예상

# Stage 3/4 GREEN
cargo test --lib --release 2>&1 | tail -5
cargo test --lib --release test_630 2>&1 | tail -10  # GREEN
cargo test --lib --release issue_147 issue_267 2>&1 | tail -10

# Stage 5 광범위
./target/release/rhwp export-svg samples/aift.hwp -o output/svg/task630_after/
# 164 fixture sweep + 페이지 수 회귀 0 확인
# WASM: docker compose --env-file .env.docker run --rm wasm
```

## 승인 요청

본 구현계획서 검토 후 Stage 1 (베이스라인 측정) 진행 승인 부탁드립니다.
