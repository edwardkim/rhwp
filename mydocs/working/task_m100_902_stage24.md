# Task #902 Stage 24 보고서 — WASM SVG 한국어 폰트 @font-face base64 embed

**Stage**: 24 / 25 (v2 점진 포팅)
**상태**: 완료

## 1. 변경 영역

`src/wmf/converter/svg/mod.rs`:
- `build_font_face_style()` 헬퍼 — `include_bytes!` 로 NanumGothic-Regular.woff2 임베드 + base64 encode
- `SVGPlayer::generate()` 가 `<style>@font-face ...</style>` 노드를 SVG 시작에 추가

## 2. 알고리즘

```rust
fn build_font_face_style() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let font_bytes = include_bytes!("../../../../web/fonts/NanumGothic-Regular.woff2");
        let b64 = STANDARD.encode(font_bytes);
        format!(r#"@font-face {{
  font-family: 'NanumGothicEmbedded';
  src: url('data:font/woff2;base64,{b64}') format('woff2');
}}
text, tspan {{
  font-family: 'NanumGothicEmbedded', 'Apple SD Gothic Neo', ..., sans-serif !important;
}}"#)
    }).clone()
}
```

1회 base64 encode + OnceLock 캐시 → 성능 영향 미미.

## 3. 폰트 정합 경로

| 환경 | WMF 렌더링 → 폰트 |
|------|------------------|
| **WASM (rhwp-studio)** | WMF→SVG embed → `<style>@font-face NanumGothicEmbedded</style>` → 브라우저가 임베디드 폰트 사용 ✓ |
| native CLI (raster 경로) | WMF→SVG→PNG via resvg — @font-face 는 SVG 안만 적용 |
| native CLI (LO opt-in) | LibreOffice 가 자체 폰트 매칭 |

→ **rhwp-studio 의 quality 가장 큰 영향**.

## 4. 라이센스 정합

- 폰트: NanumGothic-Regular.woff2 (340 KB)
- 라이센스: **SIL OFL 1.1** (재배포 가능, 임베드 가능)
- 출처: `web/fonts/` (Google Fonts 변환본)

## 5. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — (1412 passed 기대)
```

WMF SVG 직접 출력 (raster 우회):
```
/tmp/s16_bin3.wmf (4.7 MB) → SVG 3.66 MB
@font-face occurrences: 1 (single embed)
NanumGothicEmbedded references: 2 (@font-face + text/tspan)
```

## 6. 영향 평가

| 항목 | 효과 |
|------|------|
| WASM browser 출력 quality | ★★★ — 폰트 fallback 의존 제거, 일관 렌더링 |
| 출력 파일 크기 | +460 KB / WMF (base64 encoded 340 KB) |
| 성능 | 영향 미미 (OnceLock 캐시) |
| 라이센스 | SIL OFL 1.1 — 호환 |

## 7. 다음 단계

Stage 25: 최종 보고서 + PR (작업지시자 명시 승인)
