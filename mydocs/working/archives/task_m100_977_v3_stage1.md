# Stage 1 보고서 — Task #977 v3: 조사 + 수정 설계 확정

- 브랜치: `local/task977-v2` (소스 무변경)

## 폴백 체인 비교 (native vs WASM)

| 단계 | native `EmbeddedTextMeasurer` (text_measurement.rs:230~) | WASM `measure_char_width_hwp` (text_measurement.rs:799~) |
|---|---|---|
| 1. 등록 폰트 메트릭 | `measure_char_width_embedded` (Some) — 공백은 em/2 (line 1394) | 동일 (Some → 반환) |
| 2. 한글 음절 처리 | (1 에서 처리 — 한글 폰트 등록 시) | U+AC00..U+D7A3 → '가' 대리 측정값 (hangul_width_hwp) |
| 3. cluster>1 / CJK / fullwidth | `font_size` (전각) | (없음 — 모두 JS 폴백으로 진입) |
| 4. `is_narrow_punctuation` | `font_size * 0.3` | `font_size * 0.3` (PR #1026 동기화) |
| 5. **그 외 (공백 포함)** | **`font_size * 0.5`** (한컴 반각 정합) | **`cached_js_measure` (JS Canvas — 폰트별 다름)** ⚠️ |

→ 미등록 폰트의 **공백(U+0020)** 에서 native = 0.5em, WASM = JS 폰트별 측정 → 정합 어긋남.
이것이 #977 의 잔여 본질 (PR #1026 narrow_punct 분기 흡수 후 잔여).

## `is_narrow_punctuation` 대상 (text_measurement.rs:1568)
`, . : ; ' " \` U+00B7 U+2018 U+2019 U+2027` — 모두 구두점/기호. **공백 미포함**.

## 회귀 가드 양립 검증 (페이퍼)

| 가드 | 의존 분기 | 공백 분기와 충돌? |
|---|---|---|
| `tests/issue_874_ktx_toc_page_number_right_align.rs` (PR #1026 회귀) | narrow_punctuation (0.3em) | 무관 — 공백 ∉ narrow_punct |
| golden SVG 8 종 (svg_snapshot) | native EmbeddedTextMeasurer (WASM 미사용) | 무관 — WASM 폴백 변경 무영향 |

→ 공백 분기 추가는 PR #1026 영역과 **정면 충돌 없음**.

## 수정 설계 (Stage 2)

`measure_char_width_hwp` 의 narrow_punctuation 분기 직후, JS 폴백 직전에 분기 추가:

```rust
if super::is_narrow_punctuation(c) {
    return font_size * 0.3;
}
// [Task #977 v3] 미등록 폰트의 공백 폭을 native EmbeddedTextMeasurer 폴백
// (font_size * 0.5, 한컴 반각 정합) 과 동기화. JS Canvas 폴백은 폰트별로
// 폭이 달라 목차 개요번호 정렬이 어긋남 (#977 잔여 본질, PR #1026 의
// narrow_punct 분기 흡수 후 잔여).
if c == ' ' {
    return font_size * 0.5;
}
// 3차: JS 폴백 (미등록 폰트의 비-공백·비-구두점·비-한글 — 한자/일본어 가나 등)
```

**범위 한정**: 공백 (U+0020) **만**. cluster_len/CJK/fullwidth 분기는 추가하지 않음 — WASM 한글
분기는 이미 존재(2단계), 그 외 CJK/fullwidth 는 JS 폴백 유지 (회귀 위험 최소화). #977 본 타깃은
공백 폭 정합이므로 그것만 정정.

## 비회귀 케이스
- 등록 폰트 공백: 1차에서 처리, 새 분기 미진입 (불변).
- 한글 음절: 2차 분기 (불변).
- narrow_punct: 3차 분기 (불변).
- 미등록 폰트의 한자/일본어 등: 5차 JS 폴백 (불변).
- 미등록 폰트의 **공백 (U+0020) 만** 신규 5차 분기 진입 → 0.5em.

## 다음 (Stage 2)
구현 + 검증 (cargo test, golden, KTX 가드, fmt/clippy, wasm32 check).
