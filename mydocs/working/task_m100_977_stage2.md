# 단계 2 완료 보고서 — 정정 구현 (M100 #977)

- 이슈: edwardkim/rhwp#977
- 브랜치: `local/task977`
- 단계: 2/3 (정정 구현)
- 정정 방향: (C) 폴백 로직 공용화 (작업지시자 승인)

## 변경 내용

`src/renderer/layout/text_measurement.rs` 단일 파일.

### 1. 공용 함수 `base_char_width` 신설

미등록 폰트 문자 폭 폴백을 공용 함수로 추출:

```rust
fn base_char_width(c: char, cluster_len_i: u8, style: &TextStyle, font_size: f64) -> f64 {
    if let Some(w) = measure_char_width_embedded(...) { w }
    else if cluster_len_i > 1 || is_cjk_char(c) || is_fullwidth_symbol(c) { font_size }
    else if is_narrow_punctuation(c) { font_size * 0.3 }
    else { font_size * 0.5 }
}
```

### 2. `EmbeddedTextMeasurer` — 순수 리팩터 (동작 불변)

`base_w_raw` 인라인 if-else 3개소를 `base_char_width` 호출로 치환:
- `estimate_text_width`
- `compute_char_positions`
- `estimate_text_width_unrounded`

기존 로직을 그대로 추출한 것이라 네이티브 동작 변화 없음.

### 3. `WasmTextMeasurer` — 동작 정정

`estimate_text_width` / `compute_char_positions`의 단일 문자 폭 산출을
`wasm_internals::measure_char_width_hwp`(브라우저 `measureText` 폴백) →
`base_char_width`(휴리스틱 폴백)로 치환.

합성 클러스터(옛한글) 경로(`hangul_hwp/75`)는 회귀 방지를 위해 종전 유지.

### 4. 사용처 사라진 `measure_char_width_hwp` 제거

`wasm_internals::measure_char_width_hwp` 삭제 (호출부 없음).
`cached_js_measure`·`measure_hangul_width_hwp`는 옛한글 대리 측정에 계속 사용.

## 효과

미등록 폰트의 공백(및 단일 라틴 문자) 폭이 네이티브·WASM 모두 동일한
휴리스틱(`font_size*0.5` 등)으로 산출 → 선두 공백 폰트가 다른 인접
목차 문단의 개요번호 정렬이 WASM 경로에서도 일치한다.
네이티브 휴리스틱은 한컴 PDF 정합 기준으로 튜닝된 값(Task #257 등)이므로
정답지(PDF)에도 부합한다.

## 네이티브 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ 통과 (WASM 측정기 변경 컴파일 확인) |
| `cargo test --release --lib` | ✅ 1297 passed, 0 failed, 2 ignored |
| `text_measurement` 단위 테스트 | ✅ 23 passed |
| `export-svg` 2쪽 회귀 | ✅ 무회귀 — 개요번호 x좌표 종전과 동일 (1=115.59, 2·3=114.59) |
| `cargo clippy --release --lib` | ⚠️ 2 error — **본 타스크 무관 (pre-existing)** |

`clippy` 2건은 `src/diagnostics/hwp5_contract_probe.rs:366,440`의
`panicking_unwrap`/`unnecessary_unwrap` — 본 타스크에서 건드리지 않은
파일이며 `upstream/devel` 기준 기존 이슈. `text_measurement.rs`는 clippy 무경고.

## 다음 단계

단계 3 — Docker WASM 빌드 후 rhwp-studio 시각 검증(개요번호 정렬 회복,
픽셀 측정 SVG 일치) + 장평·다폰트 혼용 샘플 회귀 점검.

## 승인 요청

단계 2(정정 구현) 완료. 단계 3 진행 승인을 요청합니다.
