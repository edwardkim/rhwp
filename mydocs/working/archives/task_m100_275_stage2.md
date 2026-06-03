# Task #275 단계 2 완료보고서 — RawSvg `<image>` 단일 경로 (A)

**이슈**: [#275](https://github.com/edwardkim/rhwp/issues/275)
**브랜치**: `local/task275`
**계획서**: [`task_m100_275_impl.md`](../plans/task_m100_275_impl.md) §3 단계 2

## 1. 변경 내용

### 1.1 `src/renderer/svg_fragment.rs` (신규)

SVG 조각 파서 유틸리티 모듈. 네이티브/WASM 양쪽에서 사용 가능 (web_canvas 는 wasm32 전용이라 여기 두면 네이티브 단위 테스트 가능).

노출 API (`pub(crate)`):

1. `find_svg_attr_value(s, attr) -> Option<&str>`
   - 단어 경계 인식 속성 추출기 (`href` 검색이 `xlink:href` 를 잘못 매칭하지 않음)
2. `try_parse_single_image_data_url(svg) -> Option<&str>`
   - `<image ... href="data:..." .../>` 단일 요소 조각에서 data URL 추출
   - 복합 SVG (`<g>`, 다중 요소) 는 None
3. `decode_base64_data_url(data_url) -> Option<(String, Vec<u8>)>`
   - `data:MIME;base64,PAYLOAD` → `(mime, bytes)`
   - 비-base64 data URL, 콤마 부재, base64 디코드 실패 → None

총 **164 줄** (테스트 포함).

### 1.2 `src/renderer/mod.rs`

```rust
pub mod svg_fragment;
```
추가 (wasm32 gate 없음 — 네이티브 테스트 가능).

### 1.3 `src/renderer/web_canvas.rs`

`render_node` match 에 `RenderNodeType::RawSvg` arm 추가 (Placeholder arm 바로 앞):

```rust
RenderNodeType::RawSvg(raw) => {
    use super::svg_fragment::{try_parse_single_image_data_url, decode_base64_data_url};
    if let Some(data_url) = try_parse_single_image_data_url(&raw.svg) {
        if let Some((_mime, bytes)) = decode_base64_data_url(data_url) {
            self.draw_image(
                &bytes,
                node.bbox.x, node.bbox.y,
                node.bbox.width, node.bbox.height,
            );
        }
    }
    // else: 단계 3 대기. 현재는 암묵 skip (단계 1 이전 동작과 동일).
}
```

`draw_image` (web_canvas.rs:1529) 의 기존 IMAGE_CACHE 해시 기반 비동기 로드 패턴을 그대로 재사용. A 경로는 base64 데이터를 한 번 디코드 후 `draw_image` 내부에서 재인코딩되는 작은 비효율이 있으나, 캐시 히트 이후엔 문제없음.

추가 라인: **+16 줄** (arm 본문)

## 2. 검증

### 2.1 단위 테스트 (신규 12건)

`cargo test --lib svg_fragment`:

```
running 12 tests
test renderer::svg_fragment::tests::decode_data_url_png ... ok
test renderer::svg_fragment::tests::decode_data_url_rejects_malformed ... ok
test renderer::svg_fragment::tests::decode_data_url_rejects_non_base64 ... ok
test renderer::svg_fragment::tests::find_attr_basic ... ok
test renderer::svg_fragment::tests::find_attr_missing ... ok
test renderer::svg_fragment::tests::find_attr_word_boundary ... ok
test renderer::svg_fragment::tests::parse_single_image_href_only ... ok
test renderer::svg_fragment::tests::parse_single_image_leading_whitespace ... ok
test renderer::svg_fragment::tests::parse_single_image_rejects_group ... ok
test renderer::svg_fragment::tests::parse_single_image_rejects_missing_href ... ok
test renderer::svg_fragment::tests::parse_single_image_rejects_non_data_href ... ok
test renderer::svg_fragment::tests::parse_single_image_xlink_href ... ok

test result: ok. 12 passed; 0 failed
```

### 2.2 전체 lib 테스트

`cargo test --lib`:
- **961 passed / 14 failed / 1 ignored**
- passed: 949 (단계 1 baseline) + 12 (신규) = 961 ✓
- 14 failed: baseline 기존 실패 (cfb_writer/wasm_api 직렬화), 변화 없음

### 2.3 WASM 빌드

- `cargo check --lib --target wasm32-unknown-unknown`: clean
- `wasm-pack build --target web`: 성공 (18.21s compile + 26.69s wasm-opt)
- `pkg/rhwp_bg.wasm` 크기: 4,043,989 → **4,055,962 bytes (+11,973 bytes)** (~ +12KB, RawSvg arm + svg_fragment 모듈)

### 2.4 E2E 재현 샘플

`rhwp-studio/e2e/debug-load-bug.test.mjs` (puppeteer, 4 파일 테스트):

| 파일 | 변경 전 | 변경 후 |
|------|---------|---------|
| `samples/bitmap.hwp` | 빈 페이지 | **비트맵 이미지 정상 렌더** (손글씨 형태) |
| `samples/한셀OLE.hwp` | 빈 페이지 | **스프레드시트 이미지 정상 렌더** (노란 배경 + "1 2 3 4 5" 반복) |
| `samples/biz_plan.hwp` | 정상 | 정상 (회귀 없음) |
| `samples/form-002.hwpx` | 정상 | 정상 (회귀 없음) |

스크린샷: `rhwp-studio/e2e/screenshots/debug-_bitmap.png`, `debug-_hancell_ole.png`, `debug-biz_plan.png`, `debug-form-002x.png`

### 2.5 Clippy

- 신규 모듈 `svg_fragment.rs`: clean
- `web_canvas.rs` 변경 라인: clean
- (baseline 16 error 는 변경 외 라인에 사전 존재)

## 3. 이슈 #275 핵심 재현 케이스 해결

이슈에 기재된 두 재현 파일 (`bitmap.hwp`, `한셀OLE.hwp`) 은 **단계 2 시점에서 해결**. 사용자가 즉시 혜택 볼 수 있는 핵심 시나리오 완료.

범위 외 (단계 3):
- EMF 프리뷰 OLE (복합 SVG)
- OOXML 차트 OLE (복합 SVG)
- Placeholder 폴백 (시각 검증)

## 4. 다음 단계

**단계 3 — RawSvg 일반 경로 (B/C) + async 로드**
- `wrap_svg_fragment` — 조각을 `<svg>` 루트로 래핑
- `draw_raw_svg_async` — SVG→Image 비동기 로드 (IMAGE_CACHE 공유)
- Placeholder 시각 검증 (강제 재현)

## 5. 커밋 예정 파일

```
src/renderer/mod.rs                          (수정: svg_fragment mod 등록)
src/renderer/svg_fragment.rs                 (신규, 164줄)
src/renderer/web_canvas.rs                   (수정: RawSvg arm 추가, 2280줄에 있던 헬퍼는 svg_fragment로 이전)
mydocs/working/task_m100_275_stage2.md       (신규, 본 보고서)
```

임시 파일은 단계 4 에서 정리.

## 6. 승인 요청

단계 2 승인 후 커밋 + 단계 3 착수 진행.
