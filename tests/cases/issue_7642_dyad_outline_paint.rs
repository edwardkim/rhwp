//! 같은 공개 원본의 Native/fresh WASM 전체 1쪽 98.50470%와 확대 판독을 근거로 한다.
//! SVG 요소 이름 대신 소유·문자 순서와 최종 표시 윤곽의 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::equation::symbols::FontStyleKind;
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};

const ORIGINAL: &[u8] = include_bytes!("../../samples/issue7593/repro-equation-issue7593.hwpx");

fn descendants<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        descendants(child, out);
    }
}

fn equation<'a>(nodes: &[&'a RenderNode], script: &str) -> (&'a RenderNode, &'a EquationNode) {
    let matches: Vec<_> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Equation(eq) if eq.script == script => Some((*node, eq)),
            _ => None,
        })
        .collect();
    assert_eq!(matches.len(), 1, "원본 수식의 단일 소유와 중복 없는 방출");
    matches[0]
}

fn roman_letters(layout: &LayoutBox, parent_x: f64, roman: bool, out: &mut Vec<(String, f64)>) {
    let x = parent_x + layout.x;
    match &layout.kind {
        LayoutKind::HancomGlyphs(run) => {
            assert!(roman, "독립 Print의 rm AB는 정체다");
            assert_eq!(run.glyphs, "AB", "원본 두 문자의 표시 보존");
            assert_eq!(run.positions.len(), 2);
            assert_eq!(run.shears, [0.0, 0.0], "정체를 기울임으로 바꾸면 안 된다");
            for (letter, offset) in run.glyphs.chars().zip(&run.positions) {
                out.push((letter.to_string(), x + offset));
            }
        }
        LayoutKind::Row(children) => {
            for child in children {
                roman_letters(child, x, roman, out);
            }
        }
        LayoutKind::Decoration { body, .. } => roman_letters(body, x, roman, out),
        LayoutKind::FontStyle { style, body } => {
            roman_letters(body, x, *style == FontStyleKind::Roman, out);
        }
        LayoutKind::FontGlyph { text, outline, .. } => {
            assert!(roman, "독립 Print의 rm AB는 정체다");
            assert!(outline.is_some(), "표시 글자의 실제 윤곽 보존");
            out.push((text.clone(), x));
        }
        other => panic!("원본 AB 장식에 예상하지 않은 내용: {other:?}"),
    }
}

fn assert_ab_order(eq: &EquationNode) {
    let mut letters = Vec::new();
    roman_letters(&eq.layout_box, 0.0, false, &mut letters);
    assert_eq!(
        letters
            .iter()
            .map(|letter| letter.0.as_str())
            .collect::<String>(),
        "AB"
    );
    assert!(
        letters.windows(2).all(|pair| pair[0].1 < pair[1].1),
        "A 다음 B의 내용 순서"
    );
}

fn paint_paths(svg: &usvg::Tree, owner: &RenderNode) -> Vec<usvg::Rect> {
    fn collect(group: &usvg::Group, canvas: usvg::Transform, out: &mut Vec<usvg::Rect>) {
        for node in group.children() {
            match node {
                usvg::Node::Group(group) => collect(group, canvas, out),
                // text 로컬 윤곽에 실제 SVG 변환을 적용해 장식과 같은 좌표로 검사한다.
                usvg::Node::Text(text) => collect(
                    text.flattened(),
                    canvas.pre_concat(text.abs_transform()),
                    out,
                ),
                usvg::Node::Path(path) => out.push(
                    path.abs_bounding_box()
                        .transform(canvas)
                        .expect("최종 표시 윤곽의 유효한 좌표 변환"),
                ),
                _ => {}
            }
        }
    }
    let mut paths = Vec::new();
    collect(svg.root(), usvg::Transform::default(), &mut paths);
    paths.retain(|path| {
        let x = f64::from(path.x() + path.width() / 2.0);
        let y = f64::from(path.y() + path.height() / 2.0);
        // 실제 수식 소유 상자로 선택한다. 고정 픽셀 영역이나 화면 위치를 쓰지 않는다.
        x >= owner.bbox.x
            && x <= owner.bbox.x + owner.bbox.width
            && y >= owner.bbox.y
            && y <= owner.bbox.y + owner.bbox.height
    });
    assert!(
        paths.len() >= 3,
        "A/B 글리프와 양방향 장식의 최종 표시 보존"
    );
    paths
}

fn ink_width(paths: &[usvg::Rect]) -> f64 {
    let left = paths
        .iter()
        .map(|path| path.left())
        .fold(f32::INFINITY, f32::min);
    let right = paths
        .iter()
        .map(|path| path.right())
        .fold(f32::NEG_INFINITY, f32::max);
    f64::from(right - left)
}

fn print_svg(core: &DocumentCore) -> usvg::Tree {
    let svg = core
        .render_page_svg_layer_with_profile_native(0, rhwp::paint::RenderProfile::Print)
        .unwrap();
    // AB의 공개 수식 글꼴을 명시해 CI와 로컬에서 같은 윤곽을 검사한다.
    // 시스템 글꼴이 없어서 누락된 문자를 장식 폭의 회귀로 오판하지 않는다.
    let mut options = usvg::Options::default();
    options
        .fontdb_mut()
        .load_font_file(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/fonts/LatinModernMath-VariableOutline-subset.otf"),
        )
        .expect("저장소의 공개 수식 글꼴");
    options.font_family = "Latin Modern Math".into();
    options.fontdb_mut().set_serif_family("Latin Modern Math");
    usvg::Tree::from_str(&svg, &options).expect("실제 최종 Print SVG")
}

#[test]
fn equal_base_units_keep_equal_final_dyad_ink_with_different_reservations() {
    let core = DocumentCore::from_bytes(ORIGINAL).unwrap();
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    descendants(&tree.root, &mut nodes);
    let first = equation(&nodes, "rm dyadAB");
    let small = equation(&nodes, "rm dyad AB");
    let tall = equation(&nodes, "dyad{rm AB}");
    assert!(
        first.0.bbox.width > small.0.bbox.width,
        "원본은 서로 다른 예약 폭이다"
    );
    assert!(
        tall.0.bbox.height > small.0.bbox.height,
        "원본은 서로 다른 예약 높이다"
    );
    let svg = print_svg(&core);
    let mut widths = Vec::new();
    for (node, eq) in [first, small, tall] {
        assert_eq!(eq.font_size, first.1.font_size);
        assert_ab_order(eq);
        widths.push(ink_width(&paint_paths(&svg, node)));
    }
    for width in &widths[1..] {
        // usvg의 f32 기하 연산 오차만 흡수한다. 문서 위치·폭의 px 허용치가 아니다.
        assert!(
            (widths[0] / width - 1.0).abs() <= f64::from(f32::EPSILON) * 128.0,
            "같은 글자 크기의 실제 장식·글리프 폭이 예약 폭에 따라 달라지면 안 된다: {widths:?}"
        );
    }
}

#[test]
fn tall_dyad_final_ink_stays_above_the_following_owned_cell_text() {
    let core = DocumentCore::from_bytes(ORIGINAL).unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    descendants(&tree.root, &mut nodes);
    let (node, eq) = equation(&nodes, "dyad{rm AB}");
    assert_ab_order(eq);
    let line = nodes
        .iter()
        .find(|line| line.children.iter().any(|child| child.id == node.id))
        .expect("수식이 소속된 실제 줄");
    let (text_node, text) = line
        .children
        .iter()
        .find_map(|child| match &child.node_type {
            RenderNodeType::TextRun(text) if text.text.contains("이다") => Some((child, text)),
            _ => None,
        })
        .expect("수식 뒤 본문");
    let context = text.cell_context.as_ref().unwrap();
    let cell = context.path.first().unwrap();
    assert_eq!(eq.para_index, Some(context.parent_para_index));
    assert_eq!(eq.control_index, Some(cell.control_index));
    assert_eq!(eq.cell_index, Some(cell.cell_index));
    assert_eq!(eq.cell_para_index, Some(cell.cell_para_index));
    assert!(
        node.bbox.x + node.bbox.width <= text_node.bbox.x,
        "내용 순서와 비겹침"
    );
    let svg = print_svg(&core);
    let paths = paint_paths(&svg, node);
    let bottom = paths
        .iter()
        .map(|path| f64::from(path.bottom()))
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        bottom < text_node.bbox.y + text.baseline,
        "독립 Print처럼 보기 ㄴ의 실제 표시 잉크는 뒤 본문 기준선보다 위에 남는다"
    );
}
