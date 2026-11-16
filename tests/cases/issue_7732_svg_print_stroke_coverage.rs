//! 검사 유형: 관계형. SVG Print의 가는 실선 래스터 힌트와 원본 벡터 기하 보존을 구분한다.
//! 주석 갱신: 2026-11-15T22:19:28+09:00.
//! 실제 반례: issue1937_rowbreak_footnote_overpagination.hwp 36쪽 Studio SVG raw 88.253%.
//! 앞선 Native/fresh WASM PNG의 같은 쪽은 각 100.00%; 한컴 2020 Print·96 DPI·2px 기준이다.
//! source: mydocs/pr/assets/issue7445/issue1937_rowbreak_footnote_overpagination.hwp.
//! PDF: pdf/issue7382-regression-review/issue1937_rowbreak_footnote_overpagination-2020.pdf.
//! 합성 검사는 글꼴 없는 선의 원본 속성과 Print 정책의 소비 관계만 검사한다. 문서 px 핀은 없다.

use rhwp::paint::{CacheHint, GroupKind, LayerNode, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::LayerRenderer;
use rhwp::renderer::render_tree::{BoundingBox, GroupNode, LineNode};
use rhwp::renderer::svg_layer::SvgLayerRenderer;
use rhwp::renderer::{ArrowStyle, LineStyle, StrokeDash};

fn line() -> LineNode {
    LineNode::new(
        10.1,
        3.0,
        10.1,
        29.0,
        LineStyle {
            width: 0.48,
            color: 0x00BB_BBBB,
            ..Default::default()
        },
    )
}

fn leaf(line: LineNode) -> LayerNode {
    LayerNode::leaf(
        BoundingBox::new(0.0, 0.0, 32.0, 32.0),
        None,
        vec![PaintOp::line(line.ink_bbox(), line)],
    )
}

fn render(profile: RenderProfile, root: LayerNode) -> String {
    let tree = PageLayerTree::with_profile(32.0, 32.0, root, profile);
    let mut renderer = SvgLayerRenderer::new();
    renderer.render_page(&tree).unwrap();
    renderer.output().to_string()
}

#[test]
fn print_hint_preserves_source_width_color_and_fractional_coordinates() {
    for profile in [RenderProfile::Print, RenderProfile::HighQuality] {
        let svg = render(profile, leaf(line()));
        let parsed = roxmltree::Document::parse(&svg).unwrap();
        let emitted = parsed
            .descendants()
            .find(|n| n.has_tag_name("line"))
            .unwrap();
        assert_eq!(
            emitted.attribute("shape-rendering"),
            Some("crispEdges"),
            "가는 Print 실선은 브라우저 장치 픽셀 처리를 요청한다"
        );
        for (name, expected) in [
            ("x1", "10.1"),
            ("x2", "10.1"),
            ("y1", "3"),
            ("y2", "29"),
            ("stroke-width", "0.48"),
            ("stroke", "#bbbbbb"),
        ] {
            assert_eq!(
                emitted.attribute(name),
                Some(expected),
                "원본 {name}을 바꾸지 않는다"
            );
        }
    }
    let screen = render(RenderProfile::Screen, leaf(line()));
    assert!(
        !screen.contains("crispEdges"),
        "편집 화면의 벡터 표시 계약은 보존한다"
    );
}

#[test]
fn source_fractional_frames_diagonals_and_compound_or_dashed_strokes_keep_antialiasing() {
    let mut cases = Vec::new();
    let mut value = line();
    value.style.preserve_fractional_position = true;
    cases.push(value);
    let mut value = line();
    value.style.compound_component = true;
    cases.push(value);
    let mut value = line();
    value.style.dash = StrokeDash::Dash;
    cases.push(value);
    let mut value = line();
    value.style.dash_pattern = Some(vec![2.0, 1.0]);
    cases.push(value);
    let mut value = line();
    value.style.start_arrow = ArrowStyle::Arrow;
    cases.push(value);
    let mut value = line();
    value.style.width = 1.44;
    cases.push(value);
    let mut value = line();
    value.x2 = 20.1;
    cases.push(value);
    let mut value = line();
    value.transform.rotation = 45.0;
    cases.push(value);
    for source in cases {
        let svg = render(RenderProfile::Print, leaf(source));
        assert!(
            !svg.contains("crispEdges"),
            "독립 선형/분수 위치 계약은 래스터 힌트로 덮지 않는다: {svg}"
        );
    }
}

#[test]
fn a_skewed_group_does_not_apply_axis_hints_or_leak_its_state_to_the_next_line() {
    let bounds = BoundingBox::new(0.0, 0.0, 32.0, 32.0);
    let skewed = LayerNode::group(
        bounds,
        None,
        vec![leaf(line())],
        CacheHint::None,
        GroupKind::Group(GroupNode {
            paint_transform: Some([1.0, 0.2, 0.0, 1.0, 0.0, 0.0]),
            section_index: None,
            para_index: None,
            control_index: None,
        }),
    );
    let root = LayerNode::group(
        bounds,
        None,
        vec![skewed, leaf(line())],
        CacheHint::None,
        GroupKind::Generic,
    );
    let svg = render(RenderProfile::Print, root);
    let parsed = roxmltree::Document::parse(&svg).unwrap();
    let lines: Vec<_> = parsed
        .descendants()
        .filter(|n| n.has_tag_name("line"))
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines[0].attribute("shape-rendering"),
        None,
        "기울어진 실제 장치 선은 제외한다"
    );
    assert_eq!(
        lines[1].attribute("shape-rendering"),
        Some("crispEdges"),
        "그룹 밖의 정상 선은 다시 같은 정책을 소비한다"
    );
}
