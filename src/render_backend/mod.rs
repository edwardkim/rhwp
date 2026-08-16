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

pub mod backends;
pub mod caps;
pub mod svg_adapter;
pub mod traits;
pub mod util;

pub use backends::{DrawStats, NullBackend, TraceBackend};
pub use caps::{BackendCapabilities, BackendFeature};
pub use svg_adapter::SvgBackend;
pub use traits::{PageSize, RenderBackend, RenderBackendError};
pub use util::{paint_op_kind, replay_page, PageState};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{CacheHint, GroupKind, LayerNode, PageLayerTree, PaintOp};
    use crate::renderer::render_tree::{BoundingBox, LineNode, PageBackgroundNode, RectangleNode};
    use crate::renderer::{LineStyle, ShapeStyle};

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
    }

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

        // 자기모순 선언(래스터 전용인데 벡터 텍스트)은 불변식 위반이다.
        for caps in [
            svg,
            null,
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
