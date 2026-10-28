//! #6127: 사각 안 숫자의 내용과 벡터 프레임을 함께 보존한다.
//! 고정 크기의 SVG 문자열 대신 원본 숫자 런 안의 프레임·숫자 포함 관계를 검사한다.
//! 한컴 Print 대조: stage651/652, source8d8a2c415a, Native/fresh WASM 96.70382%.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn boxed_owner(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.text == "\u{F02B2}\u{F02B0}\u{F02B0}")
    {
        return Some(node);
    }
    node.children.iter().find_map(boxed_owner)
}

#[test]
fn issue_6127_boxed_numbers_render_as_vector_synthesis() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6127/2599643_vessel_pass_application.hwp");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("정식 입력")).expect("문서 열기");
    let page = core.build_page_render_tree(0).expect("번호의 소유 쪽");
    let owner = boxed_owner(&page.root).expect("원본 네모 숫자 200");
    let svg = core.render_page_svg_native(0).expect("같은 쪽 SVG");
    assert_eq!(
        svg.chars()
            .filter(|c| (0xF02B0..=0xF02C4).contains(&(*c as u32)))
            .count(),
        0,
        "글꼴 부재 환경에서 소실될 raw PUA를 남기지 않는다"
    );
    let xml = roxmltree::Document::parse(&svg).expect("SVG XML");
    let mut frames = Vec::new();
    for rect in xml.descendants().filter(|node| node.has_tag_name("rect")) {
        let number = |key| {
            rect.attribute(key)
                .and_then(|value| value.parse::<f64>().ok())
        };
        let (Some(x), Some(y), Some(w), Some(h)) =
            (number("x"), number("y"), number("width"), number("height"))
        else {
            continue;
        };
        if w <= 0.0
            || h <= 0.0
            || x < owner.bbox.x
            || y < owner.bbox.y
            || x + w > owner.bbox.x + owner.bbox.width
            || y + h > owner.bbox.y + owner.bbox.height
        {
            continue;
        }
        let Some(text) = rect
            .next_sibling_element()
            .filter(|node| node.has_tag_name("text"))
        else {
            continue;
        };
        let Some(transform) = text
            .attribute("transform")
            .and_then(|value| value.strip_prefix("matrix("))
            .and_then(|value| value.strip_suffix(')'))
        else {
            continue;
        };
        let matrix: Vec<f64> = transform
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|part| !part.is_empty())
            .map(|part| part.parse().expect("숫자 변환 행렬"))
            .collect();
        assert_eq!(matrix.len(), 6);
        assert!(matrix.iter().all(|value| value.is_finite()));
        assert!(
            matrix[0] > 0.0 && matrix[3] > 0.0,
            "숫자가 접히거나 소실되지 않는다"
        );
        assert!(
            matrix[4] > x && matrix[4] < x + w && matrix[5] > y && matrix[5] < y + h,
            "숫자의 표시 원점은 자신의 프레임 안에 있다"
        );
        assert_eq!(rect.attribute("fill"), Some("none"));
        assert!(rect.attribute("stroke").is_some(), "프레임 윤곽을 보존한다");
        frames.push((x, w, text.text().expect("숫자 내용")));
    }
    assert_eq!(
        frames.len(),
        3,
        "소유 런 안의 숫자 프레임을 빠짐없이 확인한다"
    );
    assert_eq!(
        frames.iter().map(|(_, _, text)| *text).collect::<String>(),
        "200"
    );
    for pair in frames.windows(2) {
        assert!(
            pair[0].0 + pair[0].1 <= pair[1].0,
            "서로 다른 숫자 프레임은 겹치지 않는다"
        );
    }
}
