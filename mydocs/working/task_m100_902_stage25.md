# Task #902 v2 Stage 25 보고서 — WMF SVG inline embed (WASM sandboxed image 우회)

**Stage**: 25 / 25 (v2 점진 포팅 완성)
**상태**: 완료

## 1. ROOT CAUSE — Stage 24 의 한계

Stage 24 에서 `@font-face base64 embed` 를 WMF SVG 내부에 추가했으나, 사용자 화면에서 quality 미개선.

**원인**: WMF SVG 가 outer SVG 의 `<image href="data:image/svg+xml;base64,...">` 로 임베드 → **브라우저가 sandboxed image 로 처리** → SVG 내부의 `@font-face data URI` 가 보안상 로드 안 됨.

## 2. 해결 — inline SVG embed

`<image href=...>` 대신 inline `<svg>` 노드로 임베드:

```rust
// 변환: convert_wmf_to_svg() 의 출력 SVG 를 outer SVG 의 자식으로 직접 inline
pub fn wmf_svg_to_inline(svg_bytes, x, y, width, height) -> Option<String> {
    // 1. <?xml ...?> prolog 제거
    // 2. 루트 <svg> 태그에 x/y/width/height + preserveAspectRatio 삽입
    // 3. SVG 내용 그대로 결합
}
```

WASM 경로 (`#[cfg(target_arch = "wasm32")]`) 에서 적용:
- 기존: `<image href="data:image/svg+xml;base64,...">` (sandboxed)
- 신규: `<svg x="76" y="298" width="608" height="411" viewBox="0 0 6333 4161">...</svg>` (inline)

→ inner WMF SVG 가 outer SVG 의 namespace 에 직접 포함 → Stage 24 의 `<style>@font-face</style>` 가 정상 적용

## 3. 검증

### 3.1 native 회귀

```
cargo test --release --all-targets — 1412 passed / 0 failed
```

### 3.2 WASM 빌드

```
wasm-pack build --release --target web --out-dir pkg
pkg/rhwp_bg.wasm: 4.88 MB (Stage 25 적용)
```

## 4. 부수 효과

| 항목 | 영향 |
|------|------|
| SVG 파일 크기 | WMF 당 ~3.6 MB → inline 으로 outer SVG 에 통합. base64 overhead 없음 (33% 절감) |
| 브라우저 호환 | SVG-in-SVG 표준 — 모든 모던 브라우저 지원 |
| native 경로 | 변경 없음 (raster PNG 우선) |

## 5. 최종 architecture

```
┌─────────────────────────────────────────────────────┐
│ rhwp WMF rendering paths                            │
├─────────────────────────────────────────────────────┤
│                                                     │
│ native (LO opt-in)    : LibreOffice → PNG embed     │
│ native (default)      : WMF→SVG → resvg → PNG embed │
│ native (RasterPlayer) : LO emfio port → PNG (opt-in)│
│ WASM (Stage 25 신규)  : WMF→SVG → inline <svg>      │
│                         + @font-face NanumGothic    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## 6. 산출물

- 소스: `src/renderer/svg.rs` (wmf_svg_to_inline + WASM 경로)
- 본 보고서: `mydocs/working/task_m100_902_stage25.md`
- WASM 번들: `pkg/rhwp_bg.wasm` (Stage 25 적용)

## 7. 다음 단계

PR 생성 (작업지시자 명시 승인).
