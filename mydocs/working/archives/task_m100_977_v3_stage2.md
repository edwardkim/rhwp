# Stage 2 보고서 — Task #977 v3: 구현 + 검증

- 브랜치: `local/task977-v2`
- 수정: `src/renderer/layout/text_measurement.rs` (1 곳, +8 lines)

## 변경

`measure_char_width_hwp` (WASM, line 819~) 의 narrow_punctuation 분기 직후, JS Canvas 폴백
직전에 공백 분기 추가:

```rust
if super::is_narrow_punctuation(c) {
    return font_size * 0.3;
}

// [Task #977] 미등록 폰트의 공백(U+0020) 폭을 native EmbeddedTextMeasurer 폴백
// (font_size * 0.5, 한컴 반각 정합) 과 동기화. JS Canvas 폴백은 미등록 폰트의
// 공백 폭이 폰트별로 달라(예: 나눔바른고딕 ≠ 맑은 고딕) 목차 페이지의 선두
// 공백 CharShape 가 인접 문단과 다를 때 개요번호 시작 x 가 ~9~10px 어긋남.
// PR #1026 (narrow_punct 분기 + native/WASM 동기화, 5/21 머지) 흡수 후 잔여 본질.
if c == ' ' {
    return font_size * 0.5;
}

// 3차: JS 폴백 (미등록 폰트)
let raw_px = cached_js_measure(measure_font, c);
...
```

PR #1026 의 `is_narrow_punctuation` 분기 + `cached_js_measure` 폴백 구조는 **모두 보존** —
신규 분기는 두 사이에 삽입만.

## 검증

- **KTX 회귀 가드** (`tests/issue_874_ktx_toc_page_number_right_align.rs`, PR #1026 영역): **1/1 통과**.
- **golden SVG 8 종** (`svg_snapshot`): **8/8 통과**.
- `cargo test --release --lib`: **1336 passed, 0 failed**.
- `cargo check --lib --target wasm32-unknown-unknown`: **OK**.
- clippy clean, fmt clean.

## 비회귀 분석 (페이퍼)
- 등록 폰트 공백: 1차 `measure_char_width_embedded` (line 1394 `c == ' '` em/2) 처리 → 새 분기
  미진입 (불변).
- 한글 음절: 2차 분기 (불변).
- narrow_punct 글자 (`, . : ; ' " \`` U+00B7 U+2018 U+2019 U+2027): 3차 분기 (불변).
- 미등록 폰트의 한자/일본어 가나 등 (비-공백·비-구두점·비-한글): 4차 JS 폴백 (불변).
- 미등록 폰트의 **공백 (U+0020) 만** 신규 5차 분기 진입 → 0.5em (native 동기화).

## 다음 (Stage 3)
WASM Docker 빌드 + rhwp-studio 시각 재현 확인 (작업지시자) → 최종 보고서.
