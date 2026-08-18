//! 출력 백엔드 공통 계약 — `RenderBackend`.
//!
//! # 왜 이 모듈이 있나
//!
//! ROADMAP 은 업스트림 책임에 "공통 문서 엔진 … 조판, 편집, 저장과 **여러 출력
//! 방식**"을 적어 두었다(`ROADMAP.md:187`). 그런데 실제 출력 경로들은 공통 계약
//! 없이 각자 자란 상태다.
//!
//! | 백엔드 | 입력 | 산출 | 오류 |
//! | --- | --- | --- | --- |
//! | `SvgRenderer` (`src/renderer/svg.rs:233`) | `&PageRenderTree` | 내부 `String` 버퍼(`output()`) | 없음 |
//! | `SvgLayerRenderer` (`src/renderer/svg_layer.rs:240`) | `&PageLayerTree` | 내부 `String` 버퍼 | `HwpError` |
//! | `HtmlRenderer` (`src/renderer/html.rs:47`) | `&PageRenderTree` | 내부 `String` 버퍼 | 없음 |
//! | `CanvasRenderer` (`src/renderer/canvas.rs:88`) | 둘 다 | `Vec<CanvasCommand>` | 없음 |
//! | `SkiaLayerRenderer` (`src/renderer/skia/renderer.rs:372`) | `&PageLayerTree` | `RasterRenderOutput` | `HwpError` |
//! | PDF (`src/renderer/pdf.rs:948`) | `&[PageLayerTree]` | `Vec<u8>` | `String` |
//!
//! 산출 방식이 셋(내부 버퍼 / 소유 반환 / 부수효과), 오류 타입이 셋(`없음` /
//! `HwpError` / `String`)이다. 새 출력 형식을 붙일 때 따라야 할 형태도, 두
//! 백엔드가 같은 페이지를 같게 그렸는지 물어볼 공통 어휘도 없다.
//!
//! 이 모듈은 그 공통 계약을 **기존 백엔드를 고치지 않고** 신설한다.
//! 어댑터는 기존 코드를 호출만 하며, `src/renderer/**` 는 이 PR 에서 바뀌지 않는다.
//! 구체 어댑터는 `SvgBackend`, `PngBackend`, `SkiaBackend` 이다. PNG 바이트열
//! 어댑터는 `SkiaLayerRenderer::render_png`를 사용하고, native-Skia adapter는
//! 래스터 문서의 치수와 바이트열을 함께 노출한다.
//!
//! # 기존 trait 들과의 관계
//!
//! 이미 세 개의 trait 이 있지만 셋 다 이 계약을 대신하지 못한다.
//!
//! - `Renderer` (`src/renderer/mod.rs:664`) — `begin_page`/`draw_text`/`draw_rect`
//!   … 원시 도형 단위다. `Result` 가 없어 실패를 말할 수 없고, 산출물 타입이
//!   없으며, `PaintOp` 가 아니라 개별 인자를 받는다.
//! - `LayerRenderer` (`src/renderer/layer_renderer.rs:21`) — 페이지 **한 장 통째로**
//!   받는다(`render_page(&mut self, tree: &PageLayerTree)`). 산출물 타입도
//!   능력 선언도 없고, 여러 페이지를 한 문서로 묶는 개념이 없다.
//! - `LayerRasterRenderer` (`src/renderer/layer_renderer.rs:73`) — 래스터 전용이다.
//!
//! `RenderBackend` 는 그 사이를 메운다. `PaintOp` 단위(= 이미 backend 재생용으로
//! 설계된 leaf op)로 받고, 연관 타입 `Output` 으로 산출물을 돌려주며,
//! [`BackendCapabilities`] 로 자기 능력을 밝히고, 여러 페이지를 한 산출물로 묶는다.
//!
//! # 좌표·단위 계약 (요약)
//!
//! - 단위는 **px**, 원점은 **페이지 왼쪽 위**, **y 는 아래로** 증가한다.
//!   이는 `PageLayerTree` 가 이미 선언한 값(`crate::paint::PAGE_LAYER_TREE_UNIT`,
//!   `crate::paint::PAGE_LAYER_TREE_COORDINATE_SYSTEM`)과 같다.
//! - HWPUNIT(1/7200 inch) → px 환산은 이 계층에 **들어오기 전에** 끝난다
//!   (`crate::renderer::hwpunit_to_px`).
//! - px → 형식 고유 단위 환산은 백엔드 **안에서** 한다. 예컨대 직접 PDF 경로는
//!   `CSS_PX_TO_PDF_POINT = 72/96` 을 자기 안에서 곱한다(`src/renderer/pdf.rs:952`).
//! - 좌표는 페이지 절대 좌표다. `PaintOp` 는 평탄화된 leaf op 이므로 조상 변환의
//!   누적이 없다.
//!
//! # 쓰는 법
//!
//! ```no_run
//! use rhwp::paint::PageLayerTree;
//! use rhwp::render_backend::{replay_page, RenderBackend, TraceBackend};
//!
//! fn dump(tree: &PageLayerTree) -> String {
//!     let mut backend = TraceBackend::new();
//!     replay_page(&mut backend, tree).expect("생명주기 위반 없음");
//!     backend.finish().expect("열린 페이지 없음")
//! }
//! ```
//!
//! 설계 배경과 기존 백엔드 채택 시나리오는 `mydocs/tech/render_backend.md`.
//!
//! # 네 번째 어댑터를 붙일 때
//!
//! devel 의 구체 어댑터는 [`SvgBackend`] 하나다. M06-1 `PngBackend`, M06-2
//! `SkiaBackend` 가 이어지고, 그 다음이 어댑터 4 다(후보: 직접 PDF).
//! 작성 절차·능력 정직성·피처 게이트·시험(M06-3/M06-4)의 정본은
//! `mydocs/manual/render_backend_adapter_guide.md` 다.
//!
//! 한 줄 요약: `src/renderer/**` 는 고치지 않는다. [`PageState`] 로 생명주기를
//! 판정한다. [`BackendCapabilities`] 가 광고한 능력만 산출물에 남긴다. 선택
//! 피처가 꺼져도 컴파일·생명주기를 지키고, 광고가 그 사실을 숨기지 않는다.
//! 정직성 대조는 기존 `render_backend` 단위 시험에 접고(새 `#[test]` 금지),
//! M06-4 하네스에 이름을 등재한다.
//!
//! ```ignore
//! impl RenderBackend for PdfBackend {
//!     type Output = Vec<u8>;
//!     type Error = RenderBackendError;
//!     fn capabilities(&self) -> BackendCapabilities { /* 광고 = 실지원 */ }
//!     fn begin_page(&mut self, size: PageSize) -> Result<(), Self::Error> { /* PageState */ }
//!     fn draw(&mut self, op: &PaintOp) -> Result<(), Self::Error> { /* … */ }
//!     fn end_page(&mut self) -> Result<(), Self::Error> { /* 기존 API 호출만 */ }
//!     fn finish(self) -> Result<Self::Output, Self::Error> { /* … */ }
//!     fn finish_boxed(self: Box<Self>) -> Result<Self::Output, Self::Error> {
//!         (*self).finish()
//!     }
//! }
//! ```
//! # M06-f 계약·픽스처
//!
//! 어댑터 본체 위에 **카탈로그·장면·정직성·픽스처·상호 diff** 를 얹는다.
//! `src/renderer/**` 는 여전히 한 줄도 바꾸지 않는다. 새 `#[cfg(test)]`
//! 모듈도 만들지 않는다 — 통합 시험은 `tests/cases/render_backend_m06f_*.rs`
//! 와 `tests/fixtures/render_backend/` 가 맡는다.

pub mod backends;
pub mod caps;
pub mod catalog;
pub mod contract;
pub mod diff;
pub mod fixture;
pub mod honesty;
pub mod png_adapter;
pub mod scenes;
pub mod skia_adapter;
pub mod svg_adapter;
pub mod traits;
pub mod util;

pub use backends::{DrawStats, NullBackend, TraceBackend};
pub use caps::{BackendCapabilities, BackendFeature};
pub use catalog::{
    catalog_invariants_hold, classify_op, spec_for_kind, ClassifiedOp, OpBounds, PaintOpKindSpec,
    PAINT_OP_KIND_COUNT, PAINT_OP_KIND_SPECS,
};
pub use contract::{
    error_display_holds, page_size_cases_hold, run_lifecycle, standard_lifecycle_scripts,
    LifecycleExpect, LifecycleScript, LifecycleStep, PageSizeCase, PAGE_SIZE_CASES,
};
pub use diff::{
    all_families_share_trace, compare_shots, kind_set, shot_from_tree, svg_is_deterministic,
    BackendFamily, BackendShot, OutputFamily, PairVerdict,
};
pub use fixture::{
    fixture_root, load_manifest, load_scene_fixtures, parse_fixture_json, FixtureManifest,
    FixtureScene,
};
pub use honesty::{
    expected_honesty_table, honesty_table_holds, observe_svg, HonestyRow, SvgObservation,
    ALL_FEATURES, PNG_SIGNATURE,
};
pub use png_adapter::PngBackend;
pub use scenes::{
    builtin_scene, builtin_scenes, materializable_kinds, materialize_scene_op, SceneOp, SceneSpec,
    HONESTY_TEXT, TINY_PNG,
};
pub use skia_adapter::SkiaBackend;
pub use svg_adapter::SvgBackend;
pub use traits::{PageSize, RenderBackend, RenderBackendError};
pub use util::{paint_op_kind, replay_page, PageState};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{CacheHint, ClipKind, GroupKind, LayerNode, PageLayerTree, PaintOp};
    use crate::renderer::render_tree::{
        BoundingBox, ImageNode, LineNode, PageBackgroundNode, RectangleNode, TextRunNode,
    };
    use crate::renderer::{GradientFillInfo, LineStyle, ShapeStyle, TextStyle};

    fn bbox(x: f64, y: f64, w: f64, h: f64) -> BoundingBox {
        BoundingBox::new(x, y, w, h)
    }

    fn rect_op(x: f64, y: f64) -> PaintOp {
        PaintOp::rectangle(
            bbox(x, y, 10.0, 10.0),
            RectangleNode::new(0.0, ShapeStyle::default(), None),
        )
    }

    fn line_op() -> PaintOp {
        PaintOp::line(
            bbox(0.0, 0.0, 50.0, 0.0),
            LineNode::new(0.0, 0.0, 50.0, 0.0, LineStyle::default()),
        )
    }

    fn page_background_op(width: f64, height: f64) -> PaintOp {
        PaintOp::page_background(
            bbox(0.0, 0.0, width, height),
            PageBackgroundNode {
                background_color: None,
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: None,
            },
        )
    }

    /// 광고 vs 실지원을 가리기 위해 산출물에 심는 고유 문자열.
    const HONESTY_TEXT: &str = "M06-3-CAP";

    /// 1×1 투명 PNG. 이미지 capability 가 산출물에 남는지 보는 데 쓴다.
    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    fn text_op(text: &str) -> PaintOp {
        PaintOp::text_run(
            bbox(10.0, 20.0, 120.0, 16.0),
            TextRunNode {
                text: text.to_string(),
                style: TextStyle {
                    font_family: "sans-serif".to_string(),
                    font_size: 16.0,
                    ..TextStyle::default()
                },
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: false,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: 12.0,
                field_marker: Default::default(),
                display_text: None,
            },
        )
    }

    fn gradient_rect_op() -> PaintOp {
        let gradient = GradientFillInfo {
            gradient_type: 1,
            angle: 0,
            center_x: 50,
            center_y: 50,
            colors: vec![0x00FF0000, 0x000000FF],
            positions: vec![0.0, 1.0],
        };
        PaintOp::rectangle(
            bbox(0.0, 0.0, 80.0, 40.0),
            RectangleNode::new(0.0, ShapeStyle::default(), Some(Box::new(gradient))),
        )
    }

    fn image_op() -> PaintOp {
        PaintOp::image(
            bbox(0.0, 0.0, 8.0, 8.0),
            ImageNode::new(1, Some(TINY_PNG.to_vec())),
            None,
        )
    }

    fn leaf_tree(ops: Vec<PaintOp>) -> PageLayerTree {
        let bounds = bbox(0.0, 0.0, 400.0, 300.0);
        let leaf = LayerNode::leaf(bounds, None, ops);
        let root = LayerNode::group(
            bounds,
            None,
            vec![leaf],
            CacheHint::default(),
            GroupKind::Body,
        );
        PageLayerTree::new(400.0, 300.0, root)
    }

    fn render_svg(tree: &PageLayerTree) -> String {
        let mut backend = SvgBackend::new();
        replay_page(&mut backend, tree).unwrap();
        backend.finish().unwrap()
    }

    /// SVG 태그 사이의 글자만 이어 붙인다. 글리프 단위 `<text>` 산출물에서
    /// 선택·검색 가능한 텍스트가 남았는지 보는 데 쓴다.
    fn svg_visible_text(svg: &str) -> String {
        let mut out = String::new();
        let mut in_tag = false;
        for ch in svg.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag && !ch.is_whitespace() => out.push(ch),
                _ => {}
            }
        }
        out
    }

    /// `multi_page` 광고가 두 번째 `begin_page` 실제 판정과 같은지 본다.
    fn assert_multi_page_matches_advertisement<B>(mut backend: B)
    where
        B: RenderBackend<Error = RenderBackendError>,
    {
        let caps = backend.capabilities();
        let name = caps.name;
        backend.begin_page(PageSize::new(40.0, 30.0)).unwrap();
        backend.end_page().unwrap();
        let second = backend.begin_page(PageSize::new(40.0, 30.0));
        if caps.supports(BackendFeature::MultiPage) {
            second.unwrap_or_else(|err| {
                panic!("{name} advertised multi_page but rejected page 2: {err}")
            });
            backend.end_page().unwrap();
            backend.finish().unwrap();
        } else {
            assert_eq!(
                second.unwrap_err(),
                RenderBackendError::MultiplePagesUnsupported { backend: name }
            );
        }
    }

    /// SVG 가 선언한 능력이 산출물에 실제로 남는지(또는 빠지는지) 본다.
    fn assert_svg_advertised_capabilities_match_output() {
        let caps = SvgBackend::new().capabilities();

        let text_svg = render_svg(&leaf_tree(vec![text_op(HONESTY_TEXT)]));
        // 글리프마다 `<text>` 를 내므로 태그 사이 글자를 이어 붙여 선택 가능 여부를 본다.
        let visible = svg_visible_text(&text_svg);
        let has_vector_text = text_svg.contains("<text") && visible.contains(HONESTY_TEXT);
        assert_eq!(
            has_vector_text,
            caps.supports(BackendFeature::VectorText),
            "vector_text advertisement vs output (visible={visible})\n{text_svg}"
        );

        let gradient_svg = render_svg(&leaf_tree(vec![gradient_rect_op()]));
        let has_gradient = gradient_svg.contains("linearGradient")
            || gradient_svg.contains("radialGradient")
            || gradient_svg.contains("<gradient");
        assert_eq!(
            has_gradient,
            caps.supports(BackendFeature::Gradients),
            "gradients advertisement vs output\n{gradient_svg}"
        );

        let image_svg = render_svg(&leaf_tree(vec![image_op()]));
        let has_image = image_svg.contains("<image") || image_svg.contains("data:image");
        assert_eq!(
            has_image,
            caps.supports(BackendFeature::Images),
            "images advertisement vs output\n{image_svg}"
        );

        // replay_page 는 ClipRect 를 벗기고 leaf 만 넘긴다. clipping:false 면
        // 산출물에 clipPath 가 없어야 하고, 클립 안 사각형은 그대로 남아야 한다.
        let bounds = bbox(0.0, 0.0, 400.0, 300.0);
        let clipped = LayerNode::clip_rect(
            bounds,
            None,
            bbox(0.0, 0.0, 5.0, 5.0),
            LayerNode::leaf(bounds, None, vec![rect_op(20.0, 20.0)]),
            ClipKind::Body,
        );
        let clip_svg = render_svg(&PageLayerTree::new(400.0, 300.0, clipped));
        let has_clip = clip_svg.contains("clipPath") || clip_svg.contains("clip-path");
        assert_eq!(
            has_clip,
            caps.supports(BackendFeature::Clipping),
            "clipping advertisement vs output\n{clip_svg}"
        );
        assert!(
            clip_svg.contains("<rect") || clip_svg.contains("rectangle"),
            "clipped tree must still emit the leaf op\n{clip_svg}"
        );

        let has_embedded_font = text_svg.contains("@font-face") || text_svg.contains("data:font");
        assert_eq!(
            has_embedded_font,
            caps.supports(BackendFeature::EmbeddedFonts),
            "embedded_fonts advertisement vs output\n{text_svg}"
        );
    }

    /// Null·Trace 는 그림을 안 그리므로 시각 capability 를 켜면 안 되고,
    /// 켜 둔 multi_page·deterministic 은 실제로 지켜야 한다.
    fn assert_instrument_advertised_capabilities_match_behavior() {
        for caps in [
            NullBackend::new().capabilities(),
            TraceBackend::new().capabilities(),
        ] {
            assert!(caps.is_consistent(), "{}", caps.name);
            assert!(!caps.supports(BackendFeature::VectorText), "{}", caps.name);
            assert!(!caps.supports(BackendFeature::Gradients), "{}", caps.name);
            assert!(!caps.supports(BackendFeature::Images), "{}", caps.name);
            assert!(!caps.supports(BackendFeature::Clipping), "{}", caps.name);
            assert!(
                !caps.supports(BackendFeature::EmbeddedFonts),
                "{}",
                caps.name
            );
            assert!(caps.supports(BackendFeature::MultiPage), "{}", caps.name);
            assert!(
                caps.supports(BackendFeature::Deterministic),
                "{}",
                caps.name
            );
        }

        assert_multi_page_matches_advertisement(NullBackend::new());
        assert_multi_page_matches_advertisement(TraceBackend::new());

        fn trace_once() -> String {
            let mut backend = TraceBackend::new();
            replay_page(&mut backend, &sample_tree()).unwrap();
            backend.finish().unwrap()
        }
        assert_eq!(trace_once(), trace_once());

        let mut first = NullBackend::new();
        let mut second = NullBackend::new();
        replay_page(&mut first, &sample_tree()).unwrap();
        replay_page(&mut second, &sample_tree()).unwrap();
        assert_eq!(first.finish().unwrap(), second.finish().unwrap());
    }

    /// 210b3ee37 후속 공백: 광고한 capability 가 실제 지원과 같아야 한다.
    fn assert_advertised_capabilities_match_behavior() {
        assert_multi_page_matches_advertisement(SvgBackend::new());
        assert_svg_advertised_capabilities_match_output();
        assert_instrument_advertised_capabilities_match_behavior();
        assert_optional_png_capabilities_if_present();
        assert_optional_skia_capabilities_if_present();
    }

    #[cfg(rhwp_has_png_backend)]
    fn assert_optional_png_capabilities_if_present() {
        let caps = PngBackend::new().capabilities();
        assert_eq!(caps.name, "png");
        assert!(caps.raster_only);
        assert!(caps.is_consistent());
        assert!(!caps.supports(BackendFeature::VectorText));
        assert!(!caps.supports(BackendFeature::EmbeddedFonts));
        assert!(!caps.supports(BackendFeature::Clipping));
        assert!(!caps.supports(BackendFeature::MultiPage));
        assert!(!caps.supports(BackendFeature::Deterministic));
        let live = PngBackend::raster_available();
        assert_eq!(caps.supports(BackendFeature::Images), live);
        assert_eq!(caps.supports(BackendFeature::Gradients), live);
        assert_eq!(
            live,
            cfg!(all(not(target_arch = "wasm32"), feature = "native-skia"))
        );

        assert_multi_page_matches_advertisement(PngBackend::new());

        let mut backend = PngBackend::new();
        replay_page(&mut backend, &sample_tree()).unwrap();
        let png = backend.finish().unwrap();
        if live {
            assert!(
                png.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
                "images/gradients advertised but PNG signature missing ({} bytes)",
                png.len()
            );
        } else {
            assert!(
                png.is_empty(),
                "raster unavailable so images/gradients are off; finish must be empty"
            );
        }
    }

    #[cfg(not(rhwp_has_png_backend))]
    fn assert_optional_png_capabilities_if_present() {}

    #[cfg(rhwp_has_skia_backend)]
    fn assert_optional_skia_capabilities_if_present() {
        let caps = SkiaBackend::new().capabilities();
        assert_eq!(caps.name, "skia");
        assert!(caps.raster_only);
        assert!(caps.is_consistent());
        assert!(!caps.supports(BackendFeature::VectorText));
        assert!(!caps.supports(BackendFeature::EmbeddedFonts));
        assert!(!caps.supports(BackendFeature::Clipping));
        assert!(!caps.supports(BackendFeature::MultiPage));
        assert!(!caps.supports(BackendFeature::Deterministic));
        let live = SkiaBackend::raster_available();
        assert_eq!(caps.supports(BackendFeature::Images), live);
        assert_eq!(caps.supports(BackendFeature::Gradients), live);
        assert_eq!(
            live,
            cfg!(all(not(target_arch = "wasm32"), feature = "native-skia"))
        );

        assert_multi_page_matches_advertisement(SkiaBackend::new());

        let mut backend = SkiaBackend::new();
        replay_page(&mut backend, &sample_tree()).unwrap();
        let out = backend.finish().unwrap();
        if live {
            assert!(
                out.width > 0 && out.height > 0 && !out.bytes.is_empty(),
                "images/gradients advertised but raster document is empty {}x{} ({} bytes)",
                out.width,
                out.height,
                out.bytes.len()
            );
        } else {
            assert_eq!(out.width, 0);
            assert_eq!(out.height, 0);
            assert!(
                out.bytes.is_empty(),
                "raster unavailable so images/gradients are off; finish must be empty"
            );
        }
    }

    #[cfg(not(rhwp_has_skia_backend))]
    fn assert_optional_skia_capabilities_if_present() {}

    /// 페이지 배경을 **일부러 마지막에** 넣은 트리.
    /// `replay_page` 가 plane 순서로 재정렬하는지 보는 데 쓴다.
    fn sample_tree() -> PageLayerTree {
        let bounds = bbox(0.0, 0.0, 400.0, 300.0);
        let leaf = LayerNode::leaf(
            bounds,
            None,
            vec![
                rect_op(20.0, 20.0),
                line_op(),
                page_background_op(400.0, 300.0),
            ],
        );
        let root = LayerNode::group(
            bounds,
            None,
            vec![leaf],
            CacheHint::default(),
            GroupKind::Body,
        );
        PageLayerTree::new(400.0, 300.0, root)
    }

    // 1. 정상 생명주기 — 계측 백엔드가 op 를 종류별로 센다.
    #[test]
    fn null_backend_counts_ops_by_kind() {
        let mut backend = NullBackend::new();
        backend.begin_page(PageSize::new(400.0, 300.0)).unwrap();
        backend.draw(&rect_op(0.0, 0.0)).unwrap();
        backend.draw(&rect_op(20.0, 20.0)).unwrap();
        backend.draw(&line_op()).unwrap();
        backend.end_page().unwrap();

        let stats = backend.finish().unwrap();
        assert_eq!(stats.pages, 1);
        assert_eq!(stats.ops, 3);
        assert_eq!(stats.count_of("rectangle"), 2);
        assert_eq!(stats.count_of("line"), 1);
        assert_eq!(stats.count_of("image"), 0);
    }

    // 2. begin_page 없이 draw 하면 오류다.
    #[test]
    fn draw_without_begin_page_is_error() {
        let mut backend = NullBackend::new();
        let err = backend.draw(&rect_op(0.0, 0.0)).unwrap_err();
        assert_eq!(err, RenderBackendError::NoOpenPage { call: "draw" });

        // TraceBackend·SvgBackend 도 같은 자리에서 같은 오류를 낸다.
        let mut trace = TraceBackend::new();
        assert_eq!(
            trace.draw(&rect_op(0.0, 0.0)).unwrap_err(),
            RenderBackendError::NoOpenPage { call: "draw" }
        );
        let mut svg = SvgBackend::new();
        assert_eq!(
            svg.draw(&rect_op(0.0, 0.0)).unwrap_err(),
            RenderBackendError::NoOpenPage { call: "draw" }
        );
        assert_optional_png_draw_without_begin_page();
        assert_optional_skia_draw_without_begin_page();
    }

    #[cfg(rhwp_has_png_backend)]
    fn assert_optional_png_draw_without_begin_page() {
        let mut png = PngBackend::new();
        assert_eq!(
            png.draw(&rect_op(0.0, 0.0)).unwrap_err(),
            RenderBackendError::NoOpenPage { call: "draw" }
        );
    }

    #[cfg(not(rhwp_has_png_backend))]
    fn assert_optional_png_draw_without_begin_page() {}

    #[cfg(rhwp_has_skia_backend)]
    fn assert_optional_skia_draw_without_begin_page() {
        let mut skia = SkiaBackend::new();
        assert_eq!(
            skia.draw(&rect_op(0.0, 0.0)).unwrap_err(),
            RenderBackendError::NoOpenPage { call: "draw" }
        );
    }

    #[cfg(not(rhwp_has_skia_backend))]
    fn assert_optional_skia_draw_without_begin_page() {}

    // 3. 페이지 경계 위반 — 중복 열기, 안 연 채 닫기, 안 닫고 끝내기.
    #[test]
    fn page_boundary_violations_are_rejected() {
        let mut backend = NullBackend::new();
        assert_eq!(
            backend.end_page().unwrap_err(),
            RenderBackendError::NoOpenPage { call: "end_page" }
        );

        backend.begin_page(PageSize::new(100.0, 100.0)).unwrap();
        assert_eq!(
            backend.begin_page(PageSize::new(100.0, 100.0)).unwrap_err(),
            RenderBackendError::PageAlreadyOpen
        );

        assert_eq!(
            backend.finish().unwrap_err(),
            RenderBackendError::UnclosedPage { pages_completed: 0 }
        );
    }

    // 4. 페이지 치수 유효성.
    #[test]
    fn invalid_page_size_is_rejected() {
        let mut backend = NullBackend::new();
        assert_eq!(
            backend.begin_page(PageSize::new(0.0, 300.0)).unwrap_err(),
            RenderBackendError::InvalidPageSize {
                width: 0.0,
                height: 300.0
            }
        );
        assert!(backend.begin_page(PageSize::new(-1.0, 300.0)).is_err());
        assert!(backend.begin_page(PageSize::new(f64::NAN, 300.0)).is_err());
        assert!(!PageSize::new(f64::INFINITY, 1.0).is_valid());
        assert!(PageSize::new(1.0, 1.0).is_valid());
        // 실패한 begin_page 는 페이지를 열지 않는다 — 이어서 정상 열기가 된다.
        assert!(backend.begin_page(PageSize::new(10.0, 10.0)).is_ok());
    }

    // 5. TraceBackend 결정성 — 같은 입력이면 같은 문자열.
    #[test]
    fn trace_backend_is_deterministic() {
        fn run() -> String {
            let mut backend = TraceBackend::new();
            replay_page(&mut backend, &sample_tree()).unwrap();
            backend.finish().unwrap()
        }

        let first = run();
        let second = run();
        assert_eq!(first, second);
        assert!(!first.is_empty());
        // 자릿수 흔들림 없이 소수 2자리로 고정된다.
        assert!(first.starts_with("begin_page 400.00x300.00"), "{first}");
    }

    // 6. 여러 페이지 경계가 추적 출력에 그대로 남는다.
    #[test]
    fn trace_backend_records_multiple_pages() {
        let mut backend = TraceBackend::new();
        backend.begin_page(PageSize::new(10.0, 10.0)).unwrap();
        backend.draw(&rect_op(0.0, 0.0)).unwrap();
        backend.end_page().unwrap();
        backend.begin_page(PageSize::new(20.0, 20.0)).unwrap();
        backend.end_page().unwrap();

        let trace = backend.finish().unwrap();
        let lines: Vec<&str> = trace.lines().collect();
        assert_eq!(
            lines,
            vec![
                "begin_page 10.00x10.00",
                "  rectangle bbox=0.00,0.00,10.00,10.00",
                "end_page ops=1",
                "begin_page 20.00x20.00",
                "end_page ops=0",
            ]
        );
    }

    // 7. 능력 질의 — 소비자가 백엔드 종류로 분기하지 않는다.
    #[test]
    fn capabilities_are_queryable_and_consistent() {
        let svg = SvgBackend::new().capabilities();
        assert_eq!(svg.name, "svg");
        assert!(svg.supports(BackendFeature::VectorText));
        assert!(svg.supports(BackendFeature::Gradients));
        assert!(!svg.supports(BackendFeature::EmbeddedFonts));
        assert!(!svg.supports(BackendFeature::Clipping));
        assert!(!svg.supports(BackendFeature::MultiPage));
        assert!(svg.covers(&[BackendFeature::VectorText, BackendFeature::Images]));
        assert!(!svg.covers(&[BackendFeature::EmbeddedFonts]));

        let null = NullBackend::new().capabilities();
        assert_eq!(null.name, "null");
        assert!(!null.supports(BackendFeature::VectorText));
        assert!(null.supports(BackendFeature::Deterministic));

        // PNG adapter는 native-Skia 가용성만 능력으로 광고한다.
        let png = PngBackend::new().capabilities();
        assert_eq!(png.name, "png");
        assert!(png.raster_only);
        assert!(!png.supports(BackendFeature::VectorText));
        assert!(!png.supports(BackendFeature::EmbeddedFonts));
        assert!(!png.supports(BackendFeature::Clipping));
        assert!(!png.supports(BackendFeature::MultiPage));
        assert!(!png.supports(BackendFeature::Deterministic));
        let live = PngBackend::raster_available();
        assert_eq!(png.supports(BackendFeature::Images), live);
        assert_eq!(png.supports(BackendFeature::Gradients), live);

        // Skia adapter도 같은 런타임 가용성만 능력으로 광고한다.
        let skia = SkiaBackend::new().capabilities();
        assert_eq!(skia.name, "skia");
        assert!(skia.raster_only);
        assert!(!skia.supports(BackendFeature::VectorText));
        assert!(!skia.supports(BackendFeature::EmbeddedFonts));
        assert!(!skia.supports(BackendFeature::Clipping));
        assert!(!skia.supports(BackendFeature::MultiPage));
        assert!(!skia.supports(BackendFeature::Deterministic));
        let skia_live = SkiaBackend::raster_available();
        assert_eq!(skia.supports(BackendFeature::Images), skia_live);
        assert_eq!(skia.supports(BackendFeature::Gradients), skia_live);
        assert_eq!(live, skia_live);
        assert_eq!(
            live,
            cfg!(all(not(target_arch = "wasm32"), feature = "native-skia"))
        );

        // 자기모순 선언(래스터 전용인데 벡터 텍스트)은 불변식 위반이다.
        for caps in [
            svg,
            null,
            png,
            skia,
            TraceBackend::new().capabilities(),
            BackendCapabilities::raster("skia"),
        ] {
            assert!(caps.is_consistent(), "{}", caps.name);
        }
        let bogus = BackendCapabilities {
            vector_text: true,
            ..BackendCapabilities::raster("bogus")
        };
        assert!(!bogus.is_consistent());

        // 210b3ee37 은 광고 플래그만 고쳤다. 광고가 실제 지원과 같은지는 여기서 닫는다.
        assert_advertised_capabilities_match_behavior();
    }

    // 8. trait 객체로 다형 호출 — 계측 백엔드와 실제 SVG 백엔드를 같은 통로로 몬다.
    #[test]
    fn backends_are_callable_through_trait_objects() {
        let tree = sample_tree();
        let mut backends: Vec<Box<dyn RenderBackend<Output = String, Error = RenderBackendError>>> =
            vec![Box::new(TraceBackend::new()), Box::new(SvgBackend::new())];

        let mut names = Vec::new();
        let mut outputs = Vec::new();
        for backend in backends.iter_mut() {
            names.push(backend.capabilities().name);
            replay_page(backend.as_mut(), &tree).unwrap();
        }
        for backend in backends {
            outputs.push(backend.finish_boxed().unwrap());
        }

        assert_eq!(names, vec!["trace", "svg"]);
        assert!(outputs[0].starts_with("begin_page"));
        assert!(outputs[1].starts_with("<svg"));
    }

    // 9. 레퍼런스 어댑터가 진짜 SVG 문서를 낸다.
    #[test]
    fn svg_backend_emits_real_svg_document() {
        let mut backend = SvgBackend::new();
        replay_page(&mut backend, &sample_tree()).unwrap();
        assert_eq!(backend.pages().len(), 1);

        let svg = backend.finish().unwrap();
        assert!(svg.starts_with("<svg"), "{svg}");
        assert!(svg.contains("viewBox=\"0 0 400 300\""), "{svg}");
        assert!(svg.trim_end().ends_with("</svg>"), "{svg}");

        assert_svg_advertised_capabilities_match_output();
        assert_optional_png_capabilities_if_present();
        assert_optional_skia_capabilities_if_present();
    }

    // 10. 페이지별 SVG 문서를 이어 붙여 유효하지 않은 단일 SVG를 만들지 않는다.
    #[test]
    fn svg_backend_rejects_a_second_page() {
        let mut backend = SvgBackend::new();
        backend.begin_page(PageSize::new(400.0, 300.0)).unwrap();
        backend.end_page().unwrap();

        assert_eq!(
            backend.begin_page(PageSize::new(400.0, 300.0)).unwrap_err(),
            RenderBackendError::MultiplePagesUnsupported { backend: "svg" }
        );

        assert_multi_page_matches_advertisement(SvgBackend::new());
        assert_multi_page_matches_advertisement(NullBackend::new());
        assert_multi_page_matches_advertisement(TraceBackend::new());
        assert_optional_png_capabilities_if_present();
        assert_optional_skia_capabilities_if_present();
    }

    // 11. 어댑터도 결정적이다 — 같은 페이지를 두 번 그리면 같은 바이트열이다.
    #[test]
    fn svg_backend_is_deterministic() {
        fn run() -> String {
            let mut backend = SvgBackend::new();
            replay_page(&mut backend, &sample_tree()).unwrap();
            backend.finish().unwrap()
        }
        assert_eq!(run(), run());
        assert!(SvgBackend::new()
            .capabilities()
            .supports(BackendFeature::Deterministic));
    }

    // 11. replay_page 가 plane 순서(배경 → … → 글 앞)로 재정렬한다.
    #[test]
    fn replay_page_reorders_ops_into_replay_planes() {
        let mut backend = TraceBackend::new();
        replay_page(&mut backend, &sample_tree()).unwrap();
        let trace = backend.finish().unwrap();
        let lines: Vec<&str> = trace.lines().collect();

        // 트리에는 rectangle, line, pageBackground 순으로 넣었지만
        // 재생은 pageBackground(Background plane)부터 나가야 한다.
        assert_eq!(lines[0], "begin_page 400.00x300.00");
        assert!(lines[1].contains("pageBackground"), "{trace}");
        assert!(lines[2].contains("rectangle"), "{trace}");
        assert!(lines[3].contains("line"), "{trace}");
        assert_eq!(lines[4], "end_page ops=3");
    }

    // 12. op 이름표가 기존 LayerTree JSON 의 "type" 값과 같은 어휘를 쓴다.
    #[test]
    fn paint_op_kind_uses_layer_json_type_names() {
        assert_eq!(paint_op_kind(&rect_op(0.0, 0.0)), "rectangle");
        assert_eq!(paint_op_kind(&line_op()), "line");
        assert_eq!(
            paint_op_kind(&page_background_op(10.0, 10.0)),
            "pageBackground"
        );
    }

    // 13. 생명주기 상태기 자체의 계수 — 백엔드 밖에서도 같은 판정을 재사용한다.
    #[test]
    fn page_state_tracks_counters() {
        let mut state = PageState::new();
        assert_eq!(state.current_page(), None);
        assert!(state.assert_finished().is_ok());

        state.begin(PageSize::new(5.0, 5.0)).unwrap();
        assert_eq!(state.current_page(), Some(PageSize::new(5.0, 5.0)));
        state.record_draw().unwrap();
        state.record_draw().unwrap();
        assert_eq!(state.ops_on_page(), 2);

        let (size, ops) = state.end().unwrap();
        assert_eq!(size, PageSize::new(5.0, 5.0));
        assert_eq!(ops, 2);
        assert_eq!(state.pages_completed(), 1);
        assert_eq!(state.ops_total(), 2);
        assert!(state.assert_finished().is_ok());
    }

    // 14. 오류가 기존 HwpError 로 손실 없이 건너간다.
    #[test]
    fn errors_convert_to_hwp_error() {
        let err = RenderBackendError::NoOpenPage { call: "draw" };
        let hwp: crate::error::HwpError = err.into();
        assert!(matches!(hwp, crate::error::HwpError::RenderError(_)));
        assert!(hwp.to_string().contains("begin_page"));
    }
}
