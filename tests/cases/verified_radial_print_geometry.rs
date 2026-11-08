//! 독립 한컴 Print 6쪽으로 원형 채우기의 물리 중심·반경과 셀 내용 보존을 검사한다.
//! 전쪽 Native/fresh WASM 실루엣 모두 100%, 실제 SVG 브라우저 출력도 모두 100%다.
//! 96dpi의 독립 PDF 타원 좌표와 ±2px로 비교하며 포함·소유 관계를 함께 유지한다.
//! 입력·Print SHA, 수정 전 실패와 새 head 증적: task_m100_7642_stage1414.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::RenderProfile;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn document() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gradient/radial-corner-print-contract.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("공식 원형 대조군"))
        .expect("원형 대조군 로드");
    assert_eq!(core.page_count(), 6, "독립 Print의 전체 쪽수");
    core
}

fn near_print(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print {label} ±2 CSS px: actual={actual}, expected={expected}"
    );
}

#[test]
fn verified_radial_print_keeps_physical_circle_at_center_and_edge_origins() {
    let core = document();
    // PDF 타원 path의 bbox를 pt에서 96dpi로 변환했다. 제품 helper의 계산값을
    // 기대값으로 재인용하지 않는다. 정사각형·가로/세로 상자·0/100% 중심 순서다.
    let print_circles = [
        (201.09134, 215.68400, 169.74867),
        (241.07066, 175.72734, 178.86399),
        (161.11333, 255.64067, 178.86400),
        (80.99600, 175.72732, 329.98268),
        (401.14398, 175.72732, 329.98265),
        (241.07001, 95.65535, 226.35801),
    ];
    for (page, (cx, cy, radius)) in print_circles.into_iter().enumerate() {
        let svg = core
            .render_page_svg_layer_with_profile_native(page as u32, RenderProfile::Print)
            .expect("실제 SVG 소비 경로");
        let xml = roxmltree::Document::parse(&svg).expect("유효한 SVG");
        let gradients: Vec<_> = xml
            .descendants()
            .filter(|node| node.has_tag_name("radialGradient"))
            .collect();
        assert_eq!(gradients.len(), 1, "각 원본 쪽의 단일 원형 채우기");
        let gradient = gradients[0];
        assert_eq!(gradient.attribute("gradientUnits"), Some("userSpaceOnUse"));
        for (attribute, expected) in [("cx", cx), ("cy", cy), ("r", radius)] {
            near_print(
                gradient
                    .attribute(attribute)
                    .expect("물리 원 좌표")
                    .parse()
                    .unwrap(),
                expected,
                attribute,
            );
        }
        for (focus, center) in [("fx", "cx"), ("fy", "cy")] {
            assert_eq!(gradient.attribute(focus), gradient.attribute(center));
        }
        let id = gradient.attribute("id").unwrap();
        let reference = format!("url(#{id})");
        let painted: Vec<_> = xml
            .descendants()
            .filter(|node| node.attribute("fill") == Some(reference.as_str()))
            .collect();
        assert_eq!(painted.len(), 1, "실제 셀 면이 해당 원형 자원을 소비한다");
        let expected = format!("RADIAL {}", page + 1);
        let text: String = xml
            .descendants()
            .filter(|node| node.has_tag_name("text"))
            .flat_map(|node| {
                node.descendants()
                    .filter(|child| child.is_text())
                    .filter_map(|child| child.text())
            })
            .collect();
        // SVG는 공백 글리프 대신 다음 text 원점을 전진시킬 수 있다.
        // 실제 원문 공백은 아래 렌더 트리 검사에서 그대로 보호한다.
        let visible = |value: &str| {
            value
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>()
        };
        assert_eq!(
            visible(&text),
            visible(&expected),
            "원본 표식의 순서·누락·중복"
        );
    }
}

fn collect_gradient_cells<'a>(
    node: &'a RenderNode,
    cell: Option<&'a RenderNode>,
    out: &mut Vec<(&'a RenderNode, &'a RenderNode)>,
) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if matches!(&node.node_type, RenderNodeType::Rectangle(rect)
        if rect.gradient.as_ref().is_some_and(|gradient| gradient.gradient_type == 2))
    {
        out.push((node, cell.expect("채우기 면의 셀 소유")));
    }
    for child in &node.children {
        collect_gradient_cells(child, cell, out);
    }
}

fn collect_text<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        out.push(node);
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

fn contains(outer: &RenderNode, inner: &RenderNode) {
    let epsilon = f64::EPSILON * outer.bbox.width.max(outer.bbox.height).max(1.0) * 128.0;
    assert!(
        inner.bbox.x + epsilon >= outer.bbox.x
            && inner.bbox.y + epsilon >= outer.bbox.y
            && inner.bbox.x + inner.bbox.width <= outer.bbox.x + outer.bbox.width + epsilon
            && inner.bbox.y + inner.bbox.height <= outer.bbox.y + outer.bbox.height + epsilon,
        "채우기와 원문은 소유 셀 안에 포함된다"
    );
}

#[test]
fn verified_radial_print_preserves_each_cells_fill_and_owned_text_once() {
    let core = document();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("실제 쪽 배치");
        let mut fills = Vec::new();
        collect_gradient_cells(&tree.root, None, &mut fills);
        assert_eq!(fills.len(), 1, "원본 쪽의 단일 셀 채우기");
        let (fill, cell) = fills[0];
        contains(cell, fill);
        let mut runs = Vec::new();
        collect_text(cell, &mut runs);
        let mut text = String::new();
        let mut prior_end = None;
        for node in runs {
            let RenderNodeType::TextRun(run) = &node.node_type else {
                unreachable!()
            };
            contains(cell, node);
            let context = run.cell_context.as_ref().expect("원문 런의 셀 소유");
            assert_eq!(context.path.len(), 1, "단일 원본 표 경로");
            assert_eq!(context.path[0].cell_index, 0);
            if let Some(end) = prior_end {
                assert!(node.bbox.x >= end, "앞뒤 원문 런 비겹침");
            }
            prior_end = Some(node.bbox.x + node.bbox.width);
            text.push_str(&run.text);
        }
        assert_eq!(text, format!("RADIAL {}", page + 1), "순서·누락·중복");
    }
}
