# Task #229 Fix — 최종 보고서

## 개요

Task #229 후속 수정. 표 셀 렌더링에서 다음 두 가지 문제를 해결:

1. **narrow glyph 역진 겹침** — 음수 자간(CharShape `letter_spacing` 또는 레이아웃 `extra_char_spacing`)이 적용된 셀에서 콤마·마침표처럼 base advance 가 좁은 글리프가 뒷 글자보다 큰 x 좌표에 배치되어 겹침.
2. **셀 채움 부재** — HWP 편집기는 셀보다 자연 폭이 큰 텍스트를 음수 자간으로 압축하여 저장. 재렌더 시 우리 폰트 메트릭으로 측정하면 측정 폭이 셀 inner 폭보다 좁아져 Center 정렬 시 좌우 여백이 크게 남음.

## 문제 분석

### 재현

- 샘플: `samples/hwpx/table-text.hwpx`
- 셀 예: `"65,063,026,600"` (CharShape `letter_spacing = −2.88 px` @ 12pt)
- 증상: 콤마 base(2.61) + ls(−2.88) = −0.27 → 콤마 다음 숫자가 콤마보다 낮은 x 좌표에 배치. 또한 rhwp 메트릭에서 clamp 적용 후 text_width ≈ 44.8px < 셀 inner ≈ 74px → 좌우 여백.

### 원인 커밋

`21a02ec` (Task #229 후속) 에서 다음을 제거:
- `text_measurement.rs` per-char 50% min_advance 클램프 5곳
- `paragraph_layout.rs` 셀 underflow 양수 자간 확장 분기

## 수정 내용

### (A) per-char 최소 advance 클램프 조건부 복원

`src/renderer/layout/text_measurement.rs` 의 `char_width` 클로저 5곳에 가드 추가:

```rust
if style.letter_spacing + style.extra_char_spacing < 0.0 {
    let min_w = base_w * ratio * 0.5;     // 또는 char_px * ratio * 0.5 (WASM)
    w = w.max(min_w);
}
```

- `EmbeddedTextMeasurer::estimate_text_width`
- `EmbeddedTextMeasurer::compute_char_positions`
- `WasmTextMeasurer::estimate_text_width`
- `WasmTextMeasurer::compute_char_positions`
- `estimate_text_width_unrounded`

**가드 기준**: CharShape 의 음수 자간(`letter_spacing`)과 레이아웃의 음수 추가 자간(`extra_char_spacing`) 합이 음수일 때만 clamp. 양수/0 케이스는 `21a02ec` 동작 그대로 유지.

### (B) 셀 underflow 자간 확장 (조건부 복원)

`src/renderer/layout/paragraph_layout.rs`:

```rust
} else if cell_ctx.is_some()
    && total_char_count > 1
    && !has_tabs
    && alignment != Alignment::Left
    && total_text_width < available_width
    && total_text_width > 0.0
    && 런_중_하나_이상이_letter_spacing_< -0.01
    && 자연_폭(ls=0) > available_width         // ← 핵심 가드
{
    // 3회 수렴 반복으로 extra_char_spacing 결정
}
```

**자연 폭 가드의 의미**: 음수 자간이 "셀에 맞추기 위한 HWP 편집기 보정" 인 경우(자연 폭 > 셀 폭)에만 fill 적용. 음수 자간이 "장식적 tightening" 인 경우(자연 폭 ≤ 셀 폭)는 기존 동작 유지 → `form-002.hwpx` 등 일반 문서의 기존 레이아웃 보존.

**수렴 반복**: narrow glyph per-char clamp 가 개입할 때 선형 1회 분배와 실제 렌더 폭이 어긋나므로 최대 3회 반복으로 `available_width` 에 수렴.

### (C) effective_text_width 복원

Center/Right/Distribute 정렬의 x 시작점 계산에서 자간 확장 후 실제 렌더 폭을 반영:

```rust
let effective_text_width = if extra_char_sp > 0.0 && cell_ctx.is_some() && ... {
    total_text_width + extra_char_sp * total_char_count as f64
} else {
    total_text_width
};
```

### (D) 회귀 테스트

`src/renderer/layout/text_measurement.rs` 의 test 모듈에 3개 단위 테스트 추가:

- `test_overflow_compression_positions_monotonic_comma` — `extra_char_spacing=-2.88` 콤마 단조성
- `test_overflow_compression_positions_monotonic_period` — 마침표 단조성
- `test_charshape_negative_letter_spacing_no_reverse` — 실제 문서 재현(`letter_spacing=-2.88`, `extra_char_spacing=0`) 단조성
- `test_non_compression_width_unchanged_by_fix` — 비-압축 경로 비회귀 가드

### (E) SVG 스냅샷 회귀 하네스

- `tests/svg_snapshot.rs` — `table_text_page_0` 테스트 엔트리 추가
- `tests/golden_svg/table-text/page-0.svg` — 수정 후 골든
- `samples/hwpx/table-text.hwpx` — 재현 샘플

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | **939 passed**; 0 failed; 1 ignored |
| `cargo test --release --test svg_snapshot table_text_page_0` | passed |
| `cargo check --target wasm32-unknown-unknown --lib` | OK |
| `export-svg` 단조성 (`table-text.hwpx`) | regressions = 0 |

### 시각 검증 (`samples/hwpx/table-text.hwpx`)

| 셀 | 텍스트 | text_span (px) | 셀 inner (px) | 상태 |
|----|--------|----------------|---------------|------|
| (r=2,c=0) | `65,063,026,600` | 67.8 | ≈74 | 채움 ✓ |
| (r=2,c=1) | `87,804,677,338` | 67.8 | ≈74 | 채움 ✓ |
| (r=2,c=2) | `151,459,074,040` | 68.6 | ≈74 | 채움 ✓ |
| (r=2,c=3) | `72.5` | 15.8 | ≈74 | 자연 폭 중앙 정렬 (비-압축) |
| (r=2,c=4) | `526.278` | 35.6 | ≈74 | 자연 폭 중앙 정렬 |
| (r=2,c=5) | `772,712` | 35.6 | ≈74 | 자연 폭 중앙 정렬 |
| (r=2,c=6) | `1,391,874` | 44.8 | ≈74 | 자연 폭 중앙 정렬 |
| (r=2,c=7) | `80.0` | 15.8 | ≈74 | 자연 폭 중앙 정렬 |

### 시각 검증 (`samples/hwpx/form-002.hwpx`)

자연 폭(ls=0)이 셀 inner 폭을 넘지 않는 셀은 underflow 확장 분기에 진입하지 않음 → 기존 레이아웃 그대로 보존됨을 10 페이지 export-svg 로 확인.

## 영향 분석

| 영역 | 영향 |
|------|------|
| 비-압축 경로 (`ls + ecs ≥ 0`) | **없음** — 가드 미진입. |
| 표 외부 Justify/Distribute 압축 | 기존 평균-기반 letter_spacing 클램프 + 신규 per-char 보조 클램프의 이중 보호. |
| 비정렬 overflow 압축 | 기존 정책 유지 + narrow glyph 역진 해소. |
| 표 셀 압축 + 셀 채움 필요 (자연 폭 > 셀 폭) | **채움 적용** — 음수 자간을 양수로 확장하여 셀 폭에 맞춤. |
| 표 셀 압축 + 자연 폭 ≤ 셀 폭 (장식적) | **기존 동작 유지** — 확장 없음. |
| WASM | 네이티브와 동일 정책. |

## 리스크

낮음. 변경 국소성(2 파일). 가드 조건이 기존 음수 자간 분기와 정확히 일치하며, 자연 폭 조건으로 의도하지 않은 텍스트 확장을 방지.

## 남은 과제 (본 타스크 범위 밖)

- `form_002_page_0` SVG 스냅샷은 본 수정 이전부터 기존 골든과 `actual.svg` 가 바이트 불일치. 본 수정 적용/미적용 두 상태에서 `actual.svg` MD5 동일(`d34c2c8...`) → 본 수정과 무관. 별도 이슈로 원인 조사 필요.

## 결론

수행계획서의 수용 기준 5개 모두 충족 + 사용자 시각 검증 피드백("이제 된 것 같음", "form-002 제대로 나옴") 반영 완료.
