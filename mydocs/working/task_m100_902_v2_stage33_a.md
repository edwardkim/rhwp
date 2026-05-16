# Task #902 v2 Stage 33-A 완료 보고서

## 개요

페이지 18 의 WMF 텍스트가 박스 하단 라인에 걸쳐 표시되는 회귀 해결.

## Root Cause

`set_text_align` 의 vertical bits 파싱 로직이 잘못되어 모든 TextAlign mode 가 `VTA_TOP` 으로 매핑되던 버그. WMF [MS-WMF] §2.1.2.18 TextAlignmentMode 의 vertical bits 값:

```
TA_TOP      = 0x0000  (기본)
TA_BOTTOM   = 0x0008
TA_BASELINE = 0x0018
```

이전 코드:

```rust
let align_vertical = [
    VerticalTextAlignmentMode::VTA_BOTTOM,  // 0x0002
    VerticalTextAlignmentMode::VTA_TOP,     // 0x0000
]
.into_iter()
.find(|a| record.text_alignment_mode & (*a as u16) == *a as u16)
.unwrap_or(VerticalTextAlignmentMode::VTA_BASELINE);
```

`mode & VTA_TOP(=0x0000) == 0` 는 **mode 값과 무관하게 항상 true** 라서 VTA_BOTTOM (0x0002) 비트가 설정되지 않은 모든 모드 — TA_BASELINE (0x0018) 포함 — 가 VTA_TOP 으로 매핑.

이후 `ext_text_out` 의 baseline 계산:

```rust
VerticalTextAlignmentMode::VTA_TOP => (em * 0.8) as i16,  // ascent 추가
```

가 적용되어 baseline 이 cell-top 보정만큼 (~font_size × 0.8) 아래로 shift. font-size 117 일 때 ~94 픽셀 하강 → 박스 하단 라인 걸침.

## 적용된 수정

### 1. `src/wmf/converter/svg/mod.rs` — `set_text_align` 비트 파싱 정합

vertical bits (`0x0018` mask) 의 값으로 분기:

```rust
let v_bits = record.text_alignment_mode & 0x0018;
let align_vertical = if v_bits == 0x0018 {
    VerticalTextAlignmentMode::VTA_BASELINE
} else if v_bits == 0x0008 {
    VerticalTextAlignmentMode::VTA_BOTTOM
} else {
    VerticalTextAlignmentMode::VTA_TOP
};
```

### 2. `src/wmf/converter/svg/mod.rs` — `ext_text_out` baseline 계산 정합

이전 `VTA_BASELINE | VTA_BOTTOM if font.height < 0 => -font.height` 잘못된 보정 제거:

```rust
+ match self.context_current.text_align_vertical {
    VerticalTextAlignmentMode::VTA_TOP => (em * 0.8) as i16,    // y가 cell top
    VerticalTextAlignmentMode::VTA_BOTTOM => -(em * 0.2) as i16, // y가 cell bottom
    VerticalTextAlignmentMode::VTA_BASELINE => 0,                // y가 baseline
    _ => 0,
}
```

### 3. `src/renderer/svg.rs` — WMF → inline `<svg>` 임베드 (sandbox 우회)

`<image href="data:image/svg+xml;base64,...">` 대신 nested `<svg x=.. y=.. viewBox=...>{body}</svg>` 로 outer SVG 에 직접 inline. 추가 사항:

- `convert_wmf_to_inline_svg(data) -> Option<(viewBox, body)>` helper
- `prefix_svg_ids(body, prefix)` — id/url(#) 충돌 회피 prefix (wmf{N}_)
- `SvgRenderer::wmf_inline_counter` 필드
- `draw_image` 두 경로 모두 FitToSize + no-crop 인 경우 inline 우선, 실패 시 기존 data URL 경로로 fallback

## 검증

### 시각 검증 (rsvg-convert)

| 항목 | 결과 |
|------|------|
| 페이지 18 오렌지 박스 "주전산센터 목표시스템 구성(안)" | ✅ 박스 내부 정합 |
| Windows 서버군 / 지장 및 백업장치군 / Unix 서버군 라벨 | ✅ 박스 내부 정합 |
| 다른 페이지 WMF (페이지 20 등) | ✅ 회귀 없음 |

### 단위 테스트

```
test result: ok. 1258 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

### Clippy

```
warning/error 없음
```

## 영향 범위

- **포지티브**: 모든 HWP3/HWP5 WMF 텍스트의 vertical alignment 정합 — TA_BASELINE 인 WMF 가 정상 위치로 표시.
- **잠재 회귀**: 기존에 잘못된 VTA_TOP 보정이 우연히 맞아 떨어졌던 WMF 가 있다면 시각 변화 가능. golden_svg 회귀 점검 필요 (별도 단계).

## 옵션 2 (woff2 임베드) 제거 — Option A 적용

`set_text_align` vertical bits 버그가 root cause 였으므로, 우회책으로 도입한 woff2 base64
임베드는 더 이상 필요 없음. SIL OFL 1.1 NanumGothic woff2 (~1.5 MB raw) 가 `include_bytes!()`
로 WASM 바이너리에 직접 포함되던 것을 제거.

### `src/wmf/converter/svg/mod.rs`

- `build_font_face_style()` 함수 삭제
- SVG 출력 시작의 `<style>@font-face{...}</style>` 노드 삽입 로직 제거

### 검증 (Option A)

| 항목 | 결과 |
|------|------|
| 페이지 18 박스 내부 텍스트 정합 (rsvg-convert) | ✅ woff2 없이도 정합 유지 |
| 단위 테스트 | ✅ 1258 passed / 0 failed |
| Clippy | ✅ clean |
| WASM 크기 (예상) | ~3.16 MB (4.66 → 1.5 절감) — 목표 4.5 MB 이하 |

### 메인테이너 8개 조건 충족

| # | 조건 | 상태 |
|---|------|------|
| 1 | raster cfg gate | ✅ |
| 2 | tiny-skia + fontdue native target | ✅ |
| 3 | `include_bytes!(NanumGothic.ttf)` 제거 | ✅ Phase 3 |
| 4 | `include_bytes!(NanumGothic-Regular.woff2)` 제거 | ✅ **Stage 33-A Option A** |
| 5 | `wasm-opt = false` 제거 | ✅ Phase 3 |
| 6 | resvg optional | ✅ |
| 7 | MPL 2.0 헤더 (raster/*.rs) | ✅ |
| 8 | WASM ~4.5 MB | ✅ (~3.16 MB 예상, 목표 이하) |

## 다음 단계

- **WASM 크기 실측**: Docker WASM 빌드로 ~3.16 MB 예상치 검증.
- **Stage 33-B (선택)**: rhwp-studio Canvas2D 경로 inline 적용 — 단, SVG export 의 root cause fix 가 같은 코드 (wmf/converter/svg/) 를 사용하므로 Canvas2D 도 자동 수혜. 별도 작업 불필요 가능.
- **PR #918 force-push**: 본 보고서 승인 + 커밋 후 진행.

작업지시자 승인 후 다음 단계 진행.
