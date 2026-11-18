//! 검사 유형: 관계형 — 원 PUA 네모·원의 프로그램 소유와 리터럴 Unicode를 구분한다.
//! 주석 갱신: 2026-11-18 03:58 KST.
//! 공식 형식문서 70쪽 Native 99.33397%, 최신 fresh WASM 전쪽은 미검증이다.
//! 지정 공식 PDF는 재산출하지 않는다. Wingdings-Regular의 네모와 원은 같은 937/2048 전진·잉크 상자다.
//! 절대 페이지 좌표 대신 원 주소·같은 프로그램의 상대 폭·서로 다른 도형을 검사한다.
//! 리터럴 작은 네모의 글꼴 계약은 바꾸지 않으며 설치된 Wingdings를 요구하지 않는다.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn load(marker: char) -> DocumentCore {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/한글문서파일형식_5.0_revision1.3.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let mut doc = core.document().clone();
    assert_eq!(doc.doc_info.bullets[0].bullet_char, '\u{f0a7}');
    doc.doc_info.bullets[0].bullet_char = marker;
    core.set_document(doc);
    core
}

fn markers(core: &DocumentCore) -> Vec<(String, f64, f64, f64)> {
    fn walk(node: &RenderNode, out: &mut Vec<(String, f64, f64, f64)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.char_start.is_none()
                && run.section_index == Some(4)
                && [2, 15, 21, 25].contains(&run.para_index.unwrap_or(usize::MAX))
            {
                out.push((
                    run.text.clone(),
                    node.bbox.width,
                    run.style.font_size,
                    run.style.ratio,
                ));
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut found = vec![];
    walk(&core.build_page_render_tree(69).unwrap().root, &mut found);
    assert_eq!(found.len(), 4, "같은 원본의 네 변경 이력 마커");
    found
}

#[test]
fn square_marker_keeps_its_original_program_address_and_advance() {
    for (text, width, size, ratio) in markers(&load('\u{f0a7}')) {
        assert_eq!(
            text, "\u{f0a7}",
            "다른 글꼴의 Unicode 작은 네모로 원 주소를 잃지 않는다"
        );
        assert!(
            width > 0.0 && width < size * ratio,
            "원 프로그램의 반각 전진을 표시 face의 전각과 구분한다: {width}"
        );
    }
}

#[test]
fn original_square_and_circle_keep_their_shared_program_advance() {
    let squares = markers(&load('\u{f0a7}'));
    let circles = markers(&load('\u{f09f}'));
    for (square, circle) in squares.iter().zip(&circles) {
        assert!(
            (square.1 - circle.1).abs() <= 96.0 / 7200.0,
            "같은 원 프로그램의 두 도형은 전진도 같다: {} vs {}",
            square.1,
            circle.1
        );
    }
}

#[test]
fn literal_unicode_square_keeps_its_declared_font_contract() {
    for (text, _, _, _) in markers(&load('▪')) {
        assert_eq!(
            text, "▪",
            "리터럴 Unicode에는 암묵적 원 프로그램을 부여하지 않는다"
        );
    }
}

#[test]
fn square_and_circle_paint_distinct_shapes_with_the_same_program_box() {
    use rhwp::renderer::{svg::SvgRenderer, Renderer, TextStyle};
    let style = TextStyle {
        font_family: "HCR Batang".into(),
        font_size: 32.0,
        ratio: 0.75,
        ..Default::default()
    };
    let render = |text: &str| {
        let mut renderer = SvgRenderer::new();
        renderer.draw_text_positioned(text, 0.0, 32.0, &style, None);
        format!("<svg>{}</svg>", renderer.output())
    };
    let square_svg = render("\u{f0a7}");
    let circle_svg = render("\u{f09f}");
    let literal_svg = render("▪");
    let square = roxmltree::Document::parse(&square_svg).unwrap();
    let circle = roxmltree::Document::parse(&circle_svg).unwrap();
    let literal = roxmltree::Document::parse(&literal_svg).unwrap();
    let rectangles: Vec<_> = square
        .descendants()
        .filter(|n| n.has_tag_name("rect"))
        .collect();
    let ellipses: Vec<_> = circle
        .descendants()
        .filter(|n| n.has_tag_name("ellipse"))
        .collect();
    assert_eq!(
        rectangles.len(),
        1,
        "원 네모는 글꼴 대체나 원으로 그리지 않는다"
    );
    assert_eq!(
        ellipses.len(),
        1,
        "기존 원 프로그램의 잉크는 원으로 유지한다"
    );
    assert!(
        !literal.descendants().any(|n| n.has_tag_name("rect")),
        "리터럴 Unicode는 선언 글꼴로 그린다"
    );
    let attr =
        |n: roxmltree::Node<'_, '_>, name| n.attribute(name).unwrap().parse::<f64>().unwrap();
    let r = rectangles[0];
    let e = ellipses[0];
    let rectangle = [
        attr(r, "x"),
        attr(r, "y"),
        attr(r, "width"),
        attr(r, "height"),
    ];
    let ellipse_box = [
        attr(e, "cx") - attr(e, "rx"),
        attr(e, "cy") - attr(e, "ry"),
        2.0 * attr(e, "rx"),
        2.0 * attr(e, "ry"),
    ];
    for (a, b) in rectangle.into_iter().zip(ellipse_box) {
        assert!(
            (a - b).abs() <= 0.001,
            "독립 프로그램의 같은 상자이며 차이는 SVG 소수 직렬화뿐이다: {a} vs {b}"
        );
    }
}
