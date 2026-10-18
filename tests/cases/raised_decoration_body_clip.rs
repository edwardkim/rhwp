//! 첫 줄의 위 밑줄·강조점은 글줄 상자 위에 그려진다. 본문 clip 이 그 잉크를
//! 포함해야 SVG/PDF 에서 잘리지 않으며, 글줄·캐럿 기하는 바뀌지 않아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const TEXT: &str = "H 첫째 줄";

fn fixture(props: &str) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, TEXT).unwrap();
    let n = TEXT.chars().count();
    doc.apply_char_format_native(0, 0, 0, n, r#"{"fontSize":3600}"#)
        .unwrap();
    if !props.is_empty() {
        doc.apply_char_format_native(0, 0, 0, n, props).unwrap();
    }
    doc
}

fn body_clip(doc: &HwpDocument) -> BoundingBox {
    fn find(node: &RenderNode) -> Option<BoundingBox> {
        if let RenderNodeType::Body {
            clip_rect: Some(clip),
        } = &node.node_type
        {
            return Some(*clip);
        }
        node.children.iter().find_map(find)
    }
    find(&doc.build_page_render_tree(0).unwrap().root).expect("본문 clip")
}

fn attr(tag: &str, name: &str) -> f64 {
    let rest = tag.split_once(&format!(" {name}=\"")).unwrap().1;
    rest.split_once('"').unwrap().0.parse().unwrap()
}

/// SVG 가 발행한 `<line stroke="#d03050">` 들의 잉크 상단(중심 - 획 반).
fn top_line_ink(svg: &str) -> f64 {
    svg.split("<line ")
        .skip(1)
        .map(|c| format!(" {}", c.split_once('>').unwrap().0))
        .filter(|tag| tag.contains("stroke=\"#d03050\""))
        .map(|tag| attr(&tag, "y1") - attr(&tag, "stroke-width") / 2.0)
        .fold(f64::INFINITY, f64::min)
}

/// SVG 가 발행한 강조점 `<text ...>●</text>` 의 em 상자 상단(기준선 - 글자 크기).
fn top_dot_ink(svg: &str) -> f64 {
    svg.split("<text ")
        .skip(1)
        .map(|c| format!(" {c}"))
        .filter(|c| c.contains(">●</text>"))
        .map(|tag| attr(&tag, "y") - attr(&tag, "font-size"))
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn first_line_top_underline_ink_is_inside_body_clip() {
    let plain = body_clip(&fixture(""));
    for shape in [0, 6, 10] {
        let doc = fixture(&format!(
            r##"{{"underlineType":"Top","underlineShape":{shape},"underlineColor":"#d03050"}}"##
        ));
        let ink_top = top_line_ink(&doc.render_page_svg_native(0).unwrap());
        assert!(ink_top < plain.y, "fixture: 위 밑줄이 본문 위로 나가야 함");
        let clip = body_clip(&doc);
        assert!(
            clip.y <= ink_top + 1e-6,
            "shape {shape}: {clip:?} / {ink_top}"
        );
        assert!(
            (clip.y + clip.height - plain.y - plain.height).abs() < 1e-6,
            "본문 하단은 그대로"
        );
    }
}

#[test]
fn first_line_emphasis_dot_ink_is_inside_body_clip() {
    let plain_doc = fixture("");
    let plain = body_clip(&plain_doc);
    let doc = fixture(r#"{"emphasisDot":1}"#);
    let ink_top = top_dot_ink(&doc.render_page_svg_native(0).unwrap());
    assert!(ink_top < plain.y, "fixture: 강조점이 본문 위로 나가야 함");
    assert!(body_clip(&doc).y <= ink_top + 1e-6);
    // 글자 상자를 키우지 않으므로 캐럿 위치는 그대로다.
    assert_eq!(
        doc.get_cursor_rect_native(0, 0, 1).unwrap(),
        plain_doc.get_cursor_rect_native(0, 0, 1).unwrap()
    );
}
