//! 관계형: 빈 글자 겹침의 캐리어는 한 프레임을 차지하지만 문자를 그리지 않는다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 59쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 59쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 한컴 2024 Print와 원본 HWP/HWPX의 59쪽 96dpi Native/fresh WASM raw는 98% 이상이다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode},
    DocumentCore,
};
fn nodes<'a>(n: &'a RenderNode, out: &mut Vec<(&'a RenderNode, &'a TextRunNode)>) {
    if let RenderNodeType::TextRun(r) = &n.node_type {
        out.push((n, r))
    }
    for c in &n.children {
        nodes(c, out)
    }
}
fn paints<'a>(n: &'a LayerNode, out: &mut Vec<&'a TextRunNode>) {
    match &n.kind {
        LayerNodeKind::Group { children, .. } => {
            for c in children {
                paints(c, out)
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            paints(child, out)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::CharOverlap { run, .. } = op {
                    out.push(run)
                }
            }
        }
    }
}
fn check(ext: &str) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join(format!("samples/2025 행정업무운영 편람(최종).{ext}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let before = core.export_hwp_native().unwrap();
    let tree = core.build_page_render_tree(58).unwrap();
    let mut all = Vec::new();
    nodes(&tree.root, &mut all);
    let (prefix_node, prefix) = all
        .iter()
        .copied()
        .find(|(_, r)| r.text.starts_with("거., 거), (거), "))
        .expect("원본 셀 앞 내용");
    let cell = format!("{:?}", prefix.cell_context);
    assert!(prefix.cell_context.is_some());
    let siblings: Vec<_> = all
        .iter()
        .copied()
        .filter(|(_, r)| {
            r.para_index == prefix.para_index && format!("{:?}", r.cell_context) == cell
        })
        .collect();
    let (frame, _) = siblings
        .iter()
        .copied()
        .find(|(_, r)| r.char_overlap.is_some() && r.text == "　")
        .expect("빈 겹침의 원문 캐리어");
    let (comma, _) = siblings
        .iter()
        .copied()
        .find(|(_, r)| r.text == ",")
        .expect("후속 쉼표");
    assert!(
        frame.bbox.width > 0.0
            && frame.bbox.x >= prefix_node.bbox.x + prefix_node.bbox.width - 1.0 / 75.0
    );
    assert!(
        comma.bbox.x >= frame.bbox.x + frame.bbox.width - 1.0 / 75.0,
        "쉼표가 점유 프레임 뒤에 위치"
    );
    let page = core
        .build_page_layer_tree_with_profile(58, RenderProfile::Print)
        .unwrap();
    let mut overlap = Vec::new();
    paints(&page.root, &mut overlap);
    let target: Vec<_> = overlap
        .into_iter()
        .filter(|r| r.para_index == prefix.para_index && format!("{:?}", r.cell_context) == cell)
        .collect();
    assert_eq!(target.len(), 1, "빈 겹침 소유와 출력 누락·중복");
    assert_eq!(
        target[0].display_or_text(),
        "",
        "독립 Print는 빈 캐리어의 글리프를 출력하지 않는다"
    );
    let json = page.to_json();
    assert!(
        json.split("\"type\":\"charOverlap\"").skip(1).any(|op| {
            op.split("\"type\":")
                .next()
                .is_some_and(|body| body.contains("\"displayText\":\"\""))
        }),
        "브라우저 출력 계약에도 빈 표시가 전달된다"
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        before,
        "표시 처리로 원문이 바뀌지 않는다"
    );
}
#[test]
fn hwp_empty_overlap_occupies_space_without_ink() {
    check("hwp")
}
#[test]
fn hwpx_empty_overlap_occupies_space_without_ink() {
    check("hwpx")
}

#[test]
fn empty_and_explicit_spaces_keep_their_format_semantics() {
    use rhwp::model::{
        control::{CharOverlap, Control},
        paragraph::{LineSeg, Paragraph},
    };
    use rhwp::renderer::composer::compose_paragraph;

    // 한컴 저장 대조에서 HWP 배열은 프레임 문자를 포함하고 HWPX 구성은 포함하지 않는다.
    let cases: &[(bool, u8, &[char], bool)] = &[
        (false, 0, &[], true),
        (false, 0, &['\u{3000}'], false),
        (false, 0, &[' '], false),
        (true, 0, &['\u{3000}'], true),
        (true, 0, &['\u{3000}', '\u{3000}'], false),
        (true, 0, &['\u{3000}', ' '], false),
        (true, 1, &['\u{25ef}'], false),
        (false, 1, &[], false),
    ];
    for &(hwp_wire_chars, border_type, chars, empty_ink) in cases {
        let para = Paragraph {
            line_segs: vec![LineSeg {
                line_height: 400,
                ..Default::default()
            }],
            controls: vec![Control::CharOverlap(CharOverlap {
                chars: chars.to_vec(),
                border_type,
                hwp_wire_chars,
                ..Default::default()
            })],
            ..Default::default()
        };
        let composed = compose_paragraph(&para);
        let run = composed
            .lines
            .iter()
            .flat_map(|line| &line.runs)
            .find(|run| run.char_overlap.is_some())
            .expect("겹침 프레임");
        assert_eq!(
            run.char_overlap.as_ref().unwrap().empty_ink,
            empty_ink,
            "형식={hwp_wire_chars}, 테두리={border_type}, 원문={chars:?}"
        );
        let Control::CharOverlap(source) = &para.controls[0] else {
            unreachable!()
        };
        assert_eq!(source.chars, chars, "원문 문자 배열을 변경하지 않는다");
    }
}

// 합성 렌더 트리는 공개 SVG 경로에서 겹침 출력 소비 계약을 검사한다.
#[test]
fn empty_char_overlap_retains_frame_without_svg_ink() {
    use rhwp::renderer::{
        composer::CharOverlapInfo,
        render_tree::{BoundingBox, PageRenderTree},
        svg::SvgRenderer,
        Renderer, TextStyle,
    };

    fn render(text: &str, empty_ink: bool, include_run: bool) -> String {
        let mut page = PageRenderTree::new(0, 100.0, 100.0);
        let run = TextRunNode {
            text: text.into(),
            style: TextStyle {
                font_size: 16.0,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: Some(CharOverlapInfo {
                border_type: 0,
                inner_char_size: 0,
                empty_ink,
                composition: None,
            }),
            border_fill_id: 0,
            baseline: 16.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        };
        if include_run {
            page.root.children.push(RenderNode::new(
                1,
                RenderNodeType::TextRun(run),
                BoundingBox::new(10.0, 10.0, 16.0, 16.0),
            ));
        }
        let mut renderer = SvgRenderer::new();
        renderer.begin_page(100.0, 100.0);
        renderer.render_tree(&page);
        renderer.end_page();
        renderer.output().to_owned()
    }
    let blank = render("\u{3000}", true, true);
    assert_eq!(
        blank,
        render("", true, false),
        "빈 구성은 SVG 잉크를 출력하지 않는다"
    );
    assert!(
        render("가", false, true).contains("가"),
        "실제 구성 글자는 출력한다"
    );
}
