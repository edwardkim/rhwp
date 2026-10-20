//! 같은 원본의 한컴 저장본·Print와 Native/fresh WASM 전1쪽 100%를 확인한 계약.
//! 도형의 PAPER 참조는 헤더 안에서 헤더 프레임 원점을 사용한다.
use rhwp::model::control::Control;
use rhwp::model::shape::{HorzRelTo, ShapeObject, VertRelTo};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn collect<'a>(
    node: &'a RenderNode,
    accept: fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if accept(node) {
        out.push(node);
    }
    for child in &node.children {
        collect(child, accept, out);
    }
}

#[test]
fn issue7642_paper_relative_rectangle_keeps_header_origin_and_precedes_body() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue_7642/header-origin/header-paper-shape-hancom-2020.hwp");
    let doc = HwpDocument::from_bytes(&std::fs::read(path).expect("공식 한컴 저장본"))
        .expect("원본 파싱");
    assert_eq!(doc.page_count(), 1, "독립 Print의 전체 쪽수");
    let mut sources = Vec::new();
    for section in &doc.document().sections {
        for paragraph in &section.paragraphs {
            for control in &paragraph.controls {
                if let Control::Header(header) = control {
                    for paragraph in &header.paragraphs {
                        for control in &paragraph.controls {
                            if let Control::Shape(shape) = control {
                                if let ShapeObject::Rectangle(rect) = &**shape {
                                    sources.push((&rect.common, &section.section_def.page_def));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        sources.len(),
        1,
        "실제 Header의 원본 사각형 소유가 유일하다"
    );
    let (source, page) = sources[0];
    assert!(matches!(source.horz_rel_to, HorzRelTo::Paper));
    assert!(matches!(source.vert_rel_to, VertRelTo::Paper));
    let tree = doc.build_page_render_tree(0).expect("실제 첫 쪽");
    let mut headers = Vec::new();
    collect(
        &tree.root,
        |n| matches!(n.node_type, RenderNodeType::Header),
        &mut headers,
    );
    assert_eq!(headers.len(), 1);
    let header = headers[0];
    let mut shapes = Vec::new();
    collect(
        header,
        |n| matches!(n.node_type, RenderNodeType::Rectangle(_)),
        &mut shapes,
    );
    assert_eq!(shapes.len(), 1, "그림을 도형 대조의 대용으로 쓰지 않는다");
    let rect = shapes[0];
    let width_hu = f64::from(page.width - page.margin_left - page.margin_right);
    let height_hu = f64::from(page.margin_header);
    assert!(width_hu > 0.0 && height_hu > 0.0);
    // 독립 Print가 확인한 소속 관계를 원본 단위와 프레임 비율로 검사한다.
    let expected = [
        f64::from(source.horizontal_offset) / width_hu,
        f64::from(source.vertical_offset) / height_hu,
        f64::from(source.width) / width_hu,
        f64::from(source.height) / height_hu,
    ];
    let actual = [
        (rect.bbox.x - header.bbox.x) / header.bbox.width,
        (rect.bbox.y - header.bbox.y) / header.bbox.height,
        rect.bbox.width / header.bbox.width,
        rect.bbox.height / header.bbox.height,
    ];
    for (have, want) in actual.into_iter().zip(expected) {
        assert!(
            (have - want).abs() < 0.00001,
            "도형의 헤더 참조·크기 관계: {have} != {want}"
        );
    }
    let mut bodies = Vec::new();
    collect(
        &tree.root,
        |n| matches!(n.node_type, RenderNodeType::Body { .. }),
        &mut bodies,
    );
    assert_eq!(bodies.len(), 1);
    assert!(
        rect.bbox.y + rect.bbox.height < bodies[0].bbox.y,
        "Print처럼 사각형은 본문 전의 헤더에 남는다"
    );
    let mut runs = Vec::new();
    collect(
        bodies[0],
        |n| matches!(n.node_type, RenderNodeType::TextRun(_)),
        &mut runs,
    );
    let text: String = runs
        .iter()
        .map(|n| match &n.node_type {
            RenderNodeType::TextRun(t) => t.display_or_text(),
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(text.trim(), "HEADER ORIGIN", "본문 누락·중복 없음");
}
