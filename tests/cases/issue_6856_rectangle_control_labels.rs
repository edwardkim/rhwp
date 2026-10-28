//! #6856: 원본 사각형 컨트롤의 소유 영역으로 조판부호를 식별한다.
//! 공개 편집 API로 만든 메모리 입력이다. 한컴 정답지/파일 호환성 시험이 아니다.
use rhwp::document_core::DocumentCore;
use rhwp::paint::builder::LayerBuilder;
use rhwp::paint::layer_tree::LayerOutputOptions;
use rhwp::paint::profile::RenderProfile;
use rhwp::renderer::html::HtmlRenderer;
use rhwp::renderer::svg::SvgRenderer;

fn core_with_rectangle_and_empty_textbox() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    for (kind, x) in [("rectangle", 8504), ("textbox", 31181)] {
        core.create_shape_control_native(
            0,
            0,
            0,
            17008,
            8504,
            x,
            11339,
            false,
            "InFrontOfText",
            kind,
            false,
            false,
            &[],
        )
        .unwrap();
    }
    core
}

fn assert_labels(output: &str) {
    assert_eq!(
        output.matches("[사각형]").count(),
        1,
        "ordinary rectangle marker"
    );
    assert_eq!(
        output.matches("[글상자]").count(),
        1,
        "empty textbox marker, no duplicate"
    );
}

#[test]
fn svg_labels_distinguish_the_source_controls() {
    let tree = core_with_rectangle_and_empty_textbox()
        .build_page_render_tree(0)
        .unwrap();
    let mut renderer = SvgRenderer::new();
    renderer.show_control_codes = true;
    renderer.render_tree(&tree);
    assert_labels(renderer.output());
}

#[test]
fn html_labels_distinguish_the_source_controls() {
    let tree = core_with_rectangle_and_empty_textbox()
        .build_page_render_tree(0)
        .unwrap();
    let mut renderer = HtmlRenderer::new();
    renderer.show_control_codes = true;
    renderer.render_tree(&tree);
    assert_labels(renderer.output());
}

#[test]
fn studio_paint_labels_distinguish_the_source_controls() {
    let tree = core_with_rectangle_and_empty_textbox()
        .build_page_render_tree(0)
        .unwrap();
    let page = LayerBuilder::new(RenderProfile::Screen)
        .with_output_options(LayerOutputOptions {
            show_control_codes: true,
            ..Default::default()
        })
        .build(&tree);
    assert_labels(&page.to_json());
}

#[test]
fn disabled_control_codes_emit_no_labels() {
    let tree = core_with_rectangle_and_empty_textbox()
        .build_page_render_tree(0)
        .unwrap();
    let mut renderer = SvgRenderer::new();
    renderer.render_tree(&tree);
    assert!(!renderer.output().contains("[사각형]"));
    assert!(!renderer.output().contains("[글상자]"));
    let page = LayerBuilder::new(RenderProfile::Print).build(&tree);
    assert!(!page.to_json().contains("\"type\":\"controlLabel\""));
}
