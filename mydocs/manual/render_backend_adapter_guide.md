---
kind: guide
status: active
canonical: mydocs/manual/render_backend_adapter_guide.md
last_verified: 2026-08-18
---

# RenderBackend 어댑터 작성 가이드

`src/render_backend/` 에 **네 번째 구체 어댑터**를 붙일 때의 절차다.
계약 자체(생명주기·좌표·능력 필드)의 권위는
[출력 백엔드 공통 계약](../tech/render_backend.md) 이다. 이 문서는
**어떻게 붙이는가** 만 다룬다.

`src/renderer/**` 는 고치지 않는다. 광고한 능력과 실제 지원이 같은지는
M06-3, 있는 어댑터끼리의 상호 diff 는 M06-4 가 판정한다. gym 과
`scripts/visual_sweep.py` 는 이 계층의 범위가 아니다.

## 1. 지금 어디인가

devel 에는 `SvgBackend`(레퍼런스)와 계측 백엔드 `NullBackend` /
`TraceBackend` 만 있다. 이어지는 번호는 이렇다.

| 번호 | 이름 | 파일 | 산출 | 상태 |
| --- | --- | --- | --- | --- |
| 1 | `SvgBackend` | `src/render_backend/svg_adapter.rs` | SVG `String` | devel 에 있음 |
| 2 | `PngBackend` | `src/render_backend/png_adapter.rs` | PNG `Vec<u8>` | M06-1 (#5370, PR #5377) |
| 3 | `SkiaBackend` | `src/render_backend/skia_adapter.rs` | `RasterRenderOutput` | M06-2 (#5378, PR #5383) |
| 4 | 다음 어댑터 | `src/render_backend/<name>_adapter.rs` | 형식 고유 `Output` | 이 가이드의 대상 |

어댑터 4 의 자연스러운 후보는 직접 PDF(`PdfBackend`, `Output = Vec<u8>`) 다.
기존 `layer_trees_to_pdf_with_options` (`src/renderer/pdf.rs`) 를 **호출만**
하면 된다. Canvas·HTML 도 같은 형틀이다. 아래 예는 PDF 를 쓴다.

Png/Skia 원본이 아직 없는 브랜치에서는 M06-3 `build.rs` 가
`rhwp_has_png_backend` / `rhwp_has_skia_backend` cfg 를 켜지 않는다.
없는 어댑터를 있는 것처럼 시험하지 않는다.

## 2. 하지 않는 것

- `src/renderer/**` 수정. 기존 렌더러를 이 trait 으로 이관하는 일은
  [계약 문서 §5](../tech/render_backend.md) 의 별도 PR 이다.
- 새 `#[test]` 를 `src/render_backend/` 에 늘리는 일. 정직성·생명주기
  대조는 **이미 있는** `render_backend::tests` 에 접는다 (M06-3).
- gym / `scripts/visual_sweep.py` / DocumentCore 신설.
- 능력 필드를 꺼 두고 산출물에 그 성질을 남기거나, 켜 두고 산출물에서
  빼는 일. 둘 다 M06-3 실패다.

## 3. 파일과 등록

저장소 루트에서 새 파일 하나는 `src/render_backend/<name>_adapter.rs` 다.
`mod.rs` 에 모듈과 타입을 내보낸다. 파일이 있는데 `mod.rs` 가 타입을
내보내지 않으면 M06-4 는 `skipped_unexported` 로 남긴다 — 침묵하지 않는다.

```rust
// src/render_backend/mod.rs
pub mod pdf_adapter;
pub use pdf_adapter::PdfBackend;
```

선택 어댑터(devel 에 아직 없을 수 있는 파일)는 M06-3 `build.rs` 가
파일 존재로 cfg 를 올린다. 어댑터 4 도 같은 훅을 탄다.

```rust
// build.rs — M06-3 이 신설. 어댑터 4 는 한 쌍을 더한다.
println!("cargo:rerun-if-changed=src/render_backend/pdf_adapter.rs");
println!("cargo:rustc-check-cfg=cfg(rhwp_has_pdf_backend)");
if backend.join("pdf_adapter.rs").is_file() {
    println!("cargo:rustc-cfg=rhwp_has_pdf_backend");
}
```

`mod.rs` 쪽 시험만 cfg 로 가린다. 어댑터 본체는 **항상 컴파일** 하는 것이
Png/Skia 의 결이다. 피처가 꺼져도 생명주기는 지키고, `finish` 가 빈
산출물을 내며, `capabilities()` 가 그 사실을 숨기지 않는다.

## 4. 얇은 어댑터 골격 (예: `PdfBackend`)

기존 공개 API 만 호출한다. `PageState` 로 생명주기를 판정하고,
받은 op 는 `end_page` 에서 `LayerNode::leaf` 하나로 묶어 넘긴다.
이 평탄화가 클립을 버리므로 `clipping` 을 켜지 않는다.

```rust
use crate::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
use crate::renderer::pdf::{layer_trees_to_pdf_with_options, DirectPdfExportOptions};
use crate::renderer::render_tree::BoundingBox;

use super::caps::BackendCapabilities;
use super::traits::{PageSize, RenderBackend, RenderBackendError};
use super::util::PageState;

pub struct PdfBackend {
    state: PageState,
    profile: RenderProfile,
    options: DirectPdfExportOptions,
    pending: Vec<PaintOp>,
    pages: Vec<PageLayerTree>,
}

impl PdfBackend {
    pub fn new() -> Self {
        Self {
            state: PageState::new(),
            profile: RenderProfile::Screen,
            options: DirectPdfExportOptions::default(),
            pending: Vec::new(),
            pages: Vec::new(),
        }
    }
}

impl RenderBackend for PdfBackend {
    type Output = Vec<u8>;
    type Error = RenderBackendError;

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            // 얇은 어댑터는 leaf 로 평탄화하므로 클립을 보존하지 못한다.
            clipping: false,
            // 직접 PDF 는 한 finish 에 여러 페이지를 담는다.
            multi_page: true,
            // 폰트 바이트 내장은 상위(document_core)가 따로 한다.
            embedded_fonts: false,
            ..BackendCapabilities::vector("pdf")
        }
    }

    fn begin_page(&mut self, size: PageSize) -> Result<(), Self::Error> {
        self.state.begin(size)?;
        self.pending.clear();
        Ok(())
    }

    fn draw(&mut self, op: &PaintOp) -> Result<(), Self::Error> {
        self.state.record_draw()?;
        self.pending.push(op.clone());
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), Self::Error> {
        let (size, _) = self.state.end()?;
        let bounds = BoundingBox::new(0.0, 0.0, size.width, size.height);
        let root = LayerNode::leaf(bounds, None, std::mem::take(&mut self.pending));
        self.pages.push(PageLayerTree::with_profile(
            size.width,
            size.height,
            root,
            self.profile,
        ));
        Ok(())
    }

    fn finish(self) -> Result<Self::Output, Self::Error> {
        self.state.assert_finished()?;
        layer_trees_to_pdf_with_options(&self.pages, &self.options)
            .map_err(RenderBackendError::Backend)
    }

    fn finish_boxed(self: Box<Self>) -> Result<Self::Output, Self::Error> {
        (*self).finish()
    }
}
```

좌표는 이 계층을 통과하는 동안 언제나 px 다. PDF pt 환산
(`CSS_PX_TO_PDF_POINT = 72/96`) 은 **감싼 기존 경로 안**에서 일어난다.
어댑터가 한 번 더 곱하지 않는다.

`finish_boxed` 구현은 언제나 `(*self).finish()` 한 줄이다. 빼면
`Box<dyn RenderBackend<..>>` 에서 산출물을 꺼낼 수 없다.

## 5. 능력 정직성 (M06-3)

`BackendCapabilities` 의 각 필드는 **최종 산출물이 그 성질을 보존하는가**다.
중간 단계가 아니다. 소비자는 타입으로 `match` 하지 않고
`caps.supports(BackendFeature::…)` / `caps.covers(&[…])` 로 질의한다.

### 5.1 광고 ↔ 실제

| 필드 | 켜면 산출물에 있어야 하는 것 | 끄면 없어야 하는 것 |
| --- | --- | --- |
| `vector_text` | 선택·검색 가능한 텍스트 (`<text>` 등) | 글리프 윤곽·래스터만 |
| `gradients` | 그라디언트 채우기 (`linearGradient` 등) | 단색으로 무너진 채우기만 |
| `images` | 래스터 이미지 (`<image>`, PNG 시그니처) | 이미지 op 를 버린 산출물 |
| `clipping` | 클립 영역 (`clipPath` 등) | 평탄화된 leaf 만 |
| `embedded_fonts` | 폰트 바이트 (`@font-face`, `data:font`) | 시스템 폰트 이름만 |
| `multi_page` | 두 번째 `begin_page` 가 `Ok` | `MultiplePagesUnsupported` |
| `deterministic` | 같은 op 시퀀스 → 같은 바이트 | 기준선 비교 대상이 아님 |
| `raster_only` | 픽셀만. `vector_text` 와 동시에 켜면 `is_consistent() == false` | — |

얇은 어댑터는 공통으로 `clipping: false` 다. SVG 본체·Skia 본체가 clip 을
지원해도, 이 경로가 `ClipRect` 를 넘기지 않으면 광고할 수 없다.

한 장짜리 형식(SVG 문서, PNG 파일, 래스터 문서)은 `multi_page: false` 이고
두 번째 `begin_page` 를 거절한다. 문서 단위 형식(직접 PDF)만 `true` 다.

### 5.2 피처가 꺼진 빌드

Png/Skia 는 `native-skia` 가 켜진 네이티브 빌드에서만 실제로 래스터한다.
기본 CI·wasm 에서는 어댑터가 컴파일되고 생명주기는 지키지만 `finish` 는
빈 바이트/빈 문서다. 그때 `images`·`gradients` 도 끈다.

```rust
pub const fn raster_available() -> bool {
    cfg!(all(not(target_arch = "wasm32"), feature = "native-skia"))
}

fn capabilities(&self) -> BackendCapabilities {
    let live = Self::raster_available();
    BackendCapabilities {
        gradients: live,
        images: live,
        clipping: false,
        multi_page: false,
        ..BackendCapabilities::raster("png")
    }
}
```

PDF 어댑터 4 가 항상 있는 경로를 감싸면 `live` 분기가 필요 없다.
다만 `src/renderer/pdf.rs` 자체는 `#[cfg(not(target_arch = "wasm32"))]` 이므로
wasm 빌드에서는 모듈을 가리거나, Png 처럼 어댑터는 컴파일하되 `finish` 를
비우고 능력 선언을 맞춘다. 선택 피처 뒤에 숨는 경로를 감싸면 Png/Skia 와
같은 `*_available()` 을 둔다. **광고가 빌드 사실을 숨기지 않는다.**

생성자 매크로 `vector("name")` / `raster("name")` / `none("name")` 의
문자열은 M06-4 가 family 를 읽는 키다. 이름(`"pdf"`)과 파일이 선언하는
family 가 픽스처 `expectedFamilies` 와 같아야 한다.

## 6. 시험 (M06-3 / M06-4)

source-side `#[test]` 는 늘리지 않는다. 기존 `render_backend::tests` 에
정직성 대조를 접고, 선택 어댑터는 cfg 훅으로만 컴파일한다.

### 6.1 M06-3 — 광고 vs 실지원

이미 있는 시험이 하는 일.

- `capabilities_are_queryable_and_consistent` — `is_consistent()` 와
  광고한 능력이 실제 지원과 같다. 여기에 어댑터 4 훅을 더한다.
- `svg_backend_rejects_a_second_page` / `assert_multi_page_matches_advertisement`
  — `multi_page` 광고와 두 번째 `begin_page` 판정이 같다.
- `svg_backend_emits_real_svg_document` — 켠 능력은 산출물 표지로 남고,
  끈 능력은 남지 않는다. 피처가 없으면 빈 산출물.
- `draw_without_begin_page_is_error` — 새 어댑터도 같은 생명주기 오류.

어댑터 4 훅 예 (파일이 있을 때만 컴파일).

```rust
#[cfg(rhwp_has_pdf_backend)]
fn assert_optional_pdf_capabilities_if_present() {
    let caps = PdfBackend::new().capabilities();
    assert_eq!(caps.name, "pdf");
    assert!(caps.is_consistent());
    assert!(caps.supports(BackendFeature::VectorText));
    assert!(!caps.supports(BackendFeature::Clipping));
    assert!(caps.supports(BackendFeature::MultiPage));
    assert_multi_page_matches_advertisement(PdfBackend::new());

    let mut backend = PdfBackend::new();
    replay_page(&mut backend, &sample_tree()).unwrap();
    let pdf = backend.finish().unwrap();
    assert!(pdf.starts_with(b"%PDF"), "vector 광고인데 PDF 시그니처가 없다");
}

#[cfg(not(rhwp_has_pdf_backend))]
fn assert_optional_pdf_capabilities_if_present() {}
```

`assert_advertised_capabilities_match_behavior` 에서 이 함수를 부른다.
Png/Skia 훅과 같은 자리다.

### 6.2 M06-4 — 상호 diff

판정 도구는 `tools/adapter_diff/harness.py` 다. 있는 어댑터끼리
구조·capability family·장면 bbox 를 맞댄다. 없는 파일은
`skipped_missing`, 파일만 있고 미등록이면 `skipped_unexported` 다.
없는 어댑터를 `MATCH` 로 꾸미지 않는다.

어댑터 4 를 등재하는 곳.

1. `tools/adapter_diff/harness.py` 의 `ADAPTERS` 튜플.

```python
AdapterSpec("pdf", "src/render_backend/pdf_adapter.rs", "PdfBackend", False),
```

`required=False` 가 기본이다. devel 에 아직 없을 수 있으면 필수가 아니다.
필수(`True`)는 svg / null / trace 뿐이다.

2. `tools/adapter_diff/fixtures/ci-scene.json` 의 `expectedFamilies`.

```json
"pdf": "vector"
```

family 문자열은 소스의 `BackendCapabilities::vector|raster|none("pdf")` 와
글자 그대로 같다. 생성자를 쓰지 않고 필드를 손으로만 채우면 하네스가
family 를 읽지 못해 `ERROR` 다.

3. `src/render_backend/mod.rs` 가 `PdfBackend` 를 내보내야 `present` 가 된다.

실행 (M06-4 가 들어온 뒤).

```text
python tools/adapter_diff/harness.py --ci
python tools/adapter_diff/harness.py --ci --json
node scripts/run-adapter-diff.mjs --cargo-test
```

`--strict` 는 `FAMILY_MISMATCH` · `ERROR` · 필수 어댑터 부재에서만 실패한다.
skip 은 데이터이지 실패가 아니다.

### 6.3 로컬 게이트

```text
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
cargo test --lib render_backend::
cargo clippy --lib -- -D warnings
```

`src/` 의 `#[cfg(test)]` 줄이 바뀌면
`node scripts/rust-unit-test-tiers.mjs --generate` 후 다시 `--check` 한다.
gym 과 `scripts/visual_sweep.py` 는 돌리지 않는다.

## 7. 체크리스트

어댑터 4 PR 을 열기 전에 이 표를 채운다.

| 항목 | 확인 |
| --- | --- |
| `src/renderer/**` 를 한 줄도 바꾸지 않았다 | |
| `<name>_adapter.rs` 를 추가하고 `mod.rs` 가 타입을 내보낸다 | |
| `PageState` 로 생명주기를 판정한다 | |
| `finish_boxed` 가 `(*self).finish()` 다 | |
| `capabilities().name` 이 안정 식별자다 (`"pdf"` 등) | |
| 켠 능력은 산출물에 표지로 남고, 끈 능력은 없다 | |
| 얇은 평탄화면 `clipping: false` | |
| 한 장짜리 형식이면 `multi_page: false` 이고 두 번째 페이지를 거절 | |
| 선택 피처가 꺼져도 컴파일·생명주기를 지키고 광고가 빈 산출물과 같다 | |
| `vector`/`raster`/`none` 생성자를 써서 M06-4 가 family 를 읽는다 | |
| M06-3 정직성 훅을 기존 시험에 접었다 (새 `#[test]` 없음) | |
| M06-4 `ADAPTERS` + `expectedFamilies` 에 이름을 올렸다 | |
| gym / visual_sweep 를 건드리지 않았다 | |

## 8. 관련 문서

- 계약·채택 시나리오: [출력 백엔드 공통 계약](../tech/render_backend.md)
- 모듈 rustdoc: `src/render_backend/mod.rs`, `src/render_backend/traits.rs`
- 레퍼런스 어댑터: `src/render_backend/svg_adapter.rs`
- 능력 선언: `src/render_backend/caps.rs`
- 정직성 시험: M06-3 (#5384, PR #5391)
- 상호 diff: M06-4 (#5392)
- PNG / native-skia 어댑터: M06-1 (#5370, PR #5377), M06-2 (#5378, PR #5383)
