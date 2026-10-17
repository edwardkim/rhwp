//! #1156: 원래 HWP/HWPX 차트의 두 번째 단 배치와 뒤 문단 비겹침.
//! 독립 Print에서 검증한 단/개체/본문의 포함·순서를 검사한다.
//! 실물의 고정 픽셀 원점이나 영역으로 문단을 선택하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

fn core(rel: &str) -> DocumentCore {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)).expect("원래 차트 문서");
    DocumentCore::from_bytes(&bytes).expect("원본 열기")
}

fn descendants<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        descendants(child, out);
    }
}

fn number(node: roxmltree::Node<'_, '_>, attr: &str) -> f64 {
    node.attribute(attr)
        .expect("차트 크기 속성")
        .parse()
        .expect("수치 속성")
}

fn assert_chart_flow(rel: &str) {
    let core = core(rel);
    let page = core.build_page_render_tree(0).expect("첫 쪽 소유 트리");
    let mut nodes = Vec::new();
    descendants(&page.root, &mut nodes);
    let column = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Column(1)))
        .expect("두 번째 단");
    let owners: Vec<_> = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::RawSvg(raw) => raw.control_ref.as_ref().filter(|r| r.kind == "ole"),
            _ => None,
        })
        .collect();
    assert_eq!(owners.len(), 1, "차트 원본 개체는 한 번만 출력한다");
    let owner = owners[0];
    let svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .expect("최종 Print SVG");
    let xml = roxmltree::Document::parse(&svg).expect("독립 SVG 파서");
    let charts: Vec<_> = xml
        .descendants()
        .filter(|n| {
            n.has_tag_name("g")
                && n.attribute("class")
                    .is_some_and(|s| s.split_whitespace().any(|c| c == "hwp-ole-chart"))
        })
        .collect();
    assert_eq!(charts.len(), 1, "실제 차트 paint도 한 번만 출력한다");
    let chart = charts[0];
    let translate = chart
        .attribute("transform")
        .expect("차트 페이지 원점")
        .strip_prefix("translate(")
        .expect("평행 이동")
        .strip_suffix(')')
        .expect("변환 끝");
    let xy: Vec<f64> = translate
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("변환 좌표"))
        .collect();
    assert_eq!(xy.len(), 2);
    let frame = chart
        .children()
        .find(|n| n.has_tag_name("rect"))
        .expect("차트 외곽");
    let x = xy[0]
        + frame
            .attribute("x")
            .map_or(0.0, |s| s.parse().expect("외곽 x"));
    let y = xy[1]
        + frame
            .attribute("y")
            .map_or(0.0, |s| s.parse().expect("외곽 y"));
    let right = x + number(frame, "width");
    let bottom = y + number(frame, "height");
    assert!(
        x >= column.bbox.x && right <= column.bbox.x + column.bbox.width,
        "{rel}: 차트 실제 외곽이 두 번째 단을 벗어남"
    );
    assert!(
        y >= column.bbox.y && bottom <= column.bbox.y + column.bbox.height,
        "{rel}: 차트 실제 외곽이 단의 본문 영역을 벗어남"
    );
    let mut lines = Vec::new();
    descendants(column, &mut lines);
    let following: Vec<_> = lines
        .iter()
        .filter(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextLine(line) if line.para_index.is_some_and(|pi| pi > owner.para_index))
        })
        .collect();
    assert!(!following.is_empty(), "차트 뒤 문단을 보존한다");
    assert!(
        following.iter().all(|line| line.bbox.y >= bottom),
        "{rel}: 같은 단의 뒤 문단이 차트 실제 영역과 겹침"
    );
}

#[test]
fn chart_moves_to_second_column_and_text_does_not_overlap() {
    for rel in [
        "samples/143E433F503322BD33.hwp",
        "samples/hwpx/143E433F503322BD33.hwpx",
    ] {
        assert_chart_flow(rel);
    }
}

#[test]
fn page_background_watermark_has_opacity() {
    for rel in [
        "samples/143E433F503322BD33.hwp",
        "samples/hwpx/143E433F503322BD33.hwpx",
    ] {
        let svg = core(rel)
            .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
            .expect("실제 Print SVG");
        assert!(
            svg.contains("<g opacity=\""),
            "{rel}: 워터마크 opacity 누락"
        );
    }
}
