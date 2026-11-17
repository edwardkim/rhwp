//! 검사 유형: 관계형·고정형. 말미 공백의 줄 예산·장식과 편람 60·63쪽의 원문 입력란 소속.
//! 주석 갱신: 2026-11-17T11:20:46+09:00; 직전 파일 변경: 6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0 2026-11-16T06:38:40+09:00.
//! 측정 source 1da7afbd64bbc59d9338a187ebcd21860ddc0842; 96 DPI·독립 한컴 Print·색상 경계 보정 전 2px raw.
//! nested: 전 17쪽, 전쪽 최저 Native 99.97617% / fresh WASM 99.97643%.
//! 10쪽 Native 100.00000% / fresh WASM 100.00000%; 같은 영역을 확대 판독했다.
//! handbook-hwp: 전 383쪽, 전쪽 최저 Native 99.96504% / fresh WASM 99.96448%.
//! 60쪽 Native 100.00000% / fresh WASM 100.00000%; 같은 영역을 확대 판독했다.
//! 63쪽 Native 100.00000% / fresh WASM 100.00000%; 같은 영역을 확대 판독했다.
//! handbook-hwpx: 전 383쪽, 전쪽 최저 Native 99.96504% / fresh WASM 99.96448%.
//! 60쪽 Native 100.00000% / fresh WASM 100.00000%; 같은 영역을 확대 판독했다.
//! 63쪽 Native 100.00000% / fresh WASM 100.00000%; 같은 영역을 확대 판독했다.
//! 근거: mydocs/working/assets/issue9104/final/visual-prerequisites.json 및 전쪽 TSV·PNG.
//! 고정 대상 쪽은 양쪽 98% 이상이다. 절대 px를 고정하지 않고 원문·입력란 소속·줄 예산·장식을 검사한다.
//! 말미 공백의 장식 해제·동일 모양 run 분할은 글자·저장 줄을 보존한 메모리 계약 변형이며 한컴 저장본으로 주장하지 않는다.
//! Issue #6117의 layer 재생 경로 장식선(밑줄/취소선) 트림 계약을 보완한다.
//!
//! #6028 이 정한 규칙 — soft-wrap 이 소비한 줄-말미 공백은 장식선 길이에서 뺀다 —
//! 은 RenderNode 경로에만 배선돼 있었다. studio 는 layer tree 를 그대로 재생하므로
//! 그 배선이 없어 밑줄이 표 칸 괘선을 넘었다. 이제 LayerBuilder가 최종
//! TextDecoration에 트림을 발행하며, 이 테스트가 그 결정을 고정한다(캔버스 픽셀 검증은
//! `rhwp-studio/e2e/issue-6117-cell-underline-canvas2d.test.mjs`).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::paint::{LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{
    BoundingBox, PageNode, PageRenderTree, RenderNode, RenderNodeType, TextLineNode, TextRunNode,
};
use rhwp::renderer::TextStyle;

fn run(text: &str, para_end: bool) -> TextRunNode {
    TextRunNode {
        text: text.to_string(),
        style: TextStyle {
            underline: rhwp::model::style::UnderlineType::Bottom,
            ..Default::default()
        },
        char_shape_id: None,
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: None,
        cell_context: None,
        is_para_end: para_end,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 12.0,
        field_marker: Default::default(),
        layout_positions: None,
        display_text: None,
    }
}

fn published_trims(runs: Vec<TextRunNode>) -> Vec<usize> {
    let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
    let mut tree = PageRenderTree::new(0, 20.0, 20.0);
    tree.root.node_type = RenderNodeType::Page(PageNode {
        page_index: 0,
        width: 20.0,
        height: 20.0,
        section_index: 0,
    });
    let mut line = RenderNode::new(
        1,
        RenderNodeType::TextLine(TextLineNode::new(10.0, 8.0)),
        bbox,
    );
    for (index, run) in runs.into_iter().enumerate() {
        line.children.push(RenderNode::new(
            10 + index as u32,
            RenderNodeType::TextRun(run),
            bbox,
        ));
    }
    tree.root.children.push(line);
    let layer = LayerBuilder::new(RenderProfile::Screen).build(&tree);
    fn collect(node: &LayerNode, trims: &mut Vec<usize>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    collect(child, trims);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                collect(child, trims)
            }
            LayerNodeKind::Leaf { ops } => trims.extend(ops.iter().filter_map(|op| match op {
                PaintOp::TextDecoration {
                    trim_trailing_spaces,
                    ..
                } => Some(*trim_trailing_spaces),
                _ => None,
            })),
        }
    }
    let mut trims = Vec::new();
    collect(&layer.root, &mut trims);
    trims
}

/// soft-wrap 줄의 마지막 가시 run 의 말미 공백이 트림 대상이다.
#[test]
fn issue_6117_trims_trailing_spaces_of_the_last_visible_run() {
    assert_eq!(
        published_trims(vec![run("가나", false), run("다라  ", false)]),
        vec![0, 2]
    );
}

/// 문단 끝 줄의 말미 공백은 저자 콘텐츠(밑줄 서명란 등)라 트림하지 않는다.
#[test]
fn issue_6117_keeps_author_spaces_on_a_paragraph_end_run() {
    assert_eq!(published_trims(vec![run("서명란   ", true)]), vec![0]);
}

/// 같은 soft-wrap 공백이 별도 run으로 나뉘어도 장식에 남지 않는다.
#[test]
fn issue_6117_trims_space_only_runs_after_soft_wrapped_text() {
    assert_eq!(
        published_trims(vec![run("본문  ", false), run("   ", false)]),
        vec![2, 3]
    );
}

#[test]
fn issue_6117_keeps_split_author_spaces_at_paragraph_end() {
    assert_eq!(
        published_trims(vec![run("서명란  ", false), run("   ", true)]),
        vec![0, 0]
    );
}

#[test]
fn issue_6117_keeps_split_author_spaces_before_explicit_break() {
    let mut tail = run("   ", false);
    tail.is_line_break_end = true;
    assert_eq!(
        published_trims(vec![run("서명란  ", false), tail]),
        vec![0, 0]
    );
}

#[test]
fn issue_6117_keeps_space_only_form_lines() {
    assert_eq!(
        published_trims(vec![run("  ", false), run("   ", false)]),
        vec![0, 0]
    );
}

#[test]
fn issue_6117_trims_every_split_tail_run_of_the_same_soft_wrap() {
    assert_eq!(
        published_trims(vec![
            run("본문  ", false),
            run(" ", false),
            run("  ", false)
        ]),
        vec![2, 1, 2]
    );
}
fn collect_tail_render_nodes(node: &RenderNode, nodes: &mut Vec<RenderNode>) {
    nodes.push(node.clone());
    for child in &node.children {
        collect_tail_render_nodes(child, nodes);
    }
}

fn terminal_decorated_paragraph(
    paragraphs: &mut [rhwp::model::paragraph::Paragraph],
) -> Option<&mut rhwp::model::paragraph::Paragraph> {
    for paragraph in paragraphs {
        if paragraph
            .text
            .ends_with("그 특정점포의 장은 이를 거부하지 못한다. ")
            && paragraph
                .text
                .contains("특정점포의 장에게 금융거래정보의 제출을 요구할 수 있으며")
        {
            return Some(paragraph);
        }
        for control in &mut paragraph.controls {
            if let rhwp::model::control::Control::Table(table) = control {
                for cell in &mut table.cells {
                    if let Some(found) = terminal_decorated_paragraph(&mut cell.paragraphs) {
                        return Some(found);
                    }
                }
            }
        }
    }
    None
}

fn terminal_decorated_line(core: &DocumentCore) -> RenderNode {
    let mut matches = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut nodes = Vec::new();
        collect_tail_render_nodes(&tree.root, &mut nodes);
        for node in nodes {
            if matches!(&node.node_type, RenderNodeType::TextLine(_)) {
                let text: String = node
                    .children
                    .iter()
                    .filter_map(|child| match &child.node_type {
                        RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                        _ => None,
                    })
                    .collect();
                if text.contains("특정점포의 장에게 금융거래정보의 제출을 요구할 수 있으며")
                {
                    assert!(
                        text.ends_with("못한다. "),
                        "real final U+0020 is preserved: {text:?}"
                    );
                    matches.push(node);
                }
            }
        }
    }
    assert_eq!(
        matches.len(),
        1,
        "the source line is neither lost nor duplicated"
    );
    matches.pop().unwrap()
}

#[test]
fn inherited_trailing_decoration_does_not_compress_visible_body() {
    let mut original = DocumentCore::from_bytes(include_bytes!(
        "../../samples/basic/issue2007_nested_cell_pagination_42065.hwp"
    ))
    .unwrap();
    // Only nominal metric selection; no system glyph program is required.
    original
        .set_font_environment(Some(
            rhwp::renderer::font_environment::FontEnvironment::from_json(
                r#"{"id":"stored-tail-metric-contract","substitutions":{"굴림":"Gulim"}}"#,
            )
            .unwrap(),
        ))
        .unwrap();
    let source_document = original.document().clone();
    let environment = original.font_environment().cloned();
    let source_line = terminal_decorated_line(&original);
    let source_runs: Vec<_> = source_line
        .children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some((node.bbox, run)),
            _ => None,
        })
        .collect();
    let source_text: String = source_runs
        .iter()
        .map(|(_, run)| run.text.as_str())
        .collect();
    for keep_decoration in [false, true] {
        let mut changed = source_document.clone();
        let (tail_pos, old_shape_id) = {
            let paragraph =
                terminal_decorated_paragraph(&mut changed.sections[0].paragraphs).unwrap();
            assert!(paragraph.text.ends_with(' '));
            let tail = *paragraph.char_offsets.last().unwrap();
            let id = paragraph
                .char_shapes
                .iter()
                .rev()
                .find(|shape| shape.start_pos <= tail)
                .unwrap()
                .char_shape_id;
            (tail, id)
        };
        let mut tail_shape = changed.doc_info.char_shapes[old_shape_id as usize].clone();
        assert_ne!(
            tail_shape.underline_type,
            rhwp::model::style::UnderlineType::None
        );
        if !keep_decoration {
            tail_shape.underline_type = rhwp::model::style::UnderlineType::None;
        }
        let new_id = changed.doc_info.char_shapes.len() as u32;
        changed.doc_info.char_shapes.push(tail_shape);
        let paragraph = terminal_decorated_paragraph(&mut changed.sections[0].paragraphs).unwrap();
        let text = paragraph.text.clone();
        let count = paragraph.char_count;
        let lines = paragraph.line_segs.clone();
        paragraph
            .char_shapes
            .push(rhwp::model::paragraph::CharShapeRef {
                start_pos: tail_pos,
                char_shape_id: new_id,
            });
        original.set_document(changed);
        original.set_font_environment(environment.clone()).unwrap();
        let changed_line = terminal_decorated_line(&original);
        let changed_runs: Vec<_> = changed_line
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) => Some((node.bbox, run)),
                _ => None,
            })
            .collect();
        let changed_text: String = changed_runs
            .iter()
            .map(|(_, run)| run.text.as_str())
            .collect();
        assert_eq!(source_text, changed_text, "logical content and order");
        // The source already emits punctuation and its final space separately.
        // Splitting only the tail style must preserve every visible run's frame.
        // No installed font program or private position helper is required.
        let frames = |runs: &[(BoundingBox, &rhwp::renderer::render_tree::TextRunNode)]| {
            runs.iter()
                .filter(|(_, run)| run.text.chars().any(|ch| !ch.is_whitespace()))
                .map(|(bbox, run)| {
                    (
                        run.text.clone(),
                        *bbox,
                        run.style.extra_word_spacing,
                        run.style.extra_char_spacing,
                    )
                })
                .collect::<Vec<_>>()
        };
        let expected = frames(&source_runs);
        let actual = frames(&changed_runs);
        assert_eq!(expected.len(), actual.len());
        for ((a, frame, word, character), (b, other, other_word, other_character)) in
            expected.into_iter().zip(actual)
        {
            assert_eq!(a, b);
            for (before, after) in [
                (frame.x, other.x),
                (frame.y, other.y),
                (frame.width, other.width),
                (frame.height, other.height),
                (word, other_word),
                (character, other_character),
            ] {
                assert!((before-after).abs()<1e-6,"inherited decoration must preserve visible body frame {a}: {frame:?} -> {other:?}");
            }
        }
        let saved = original.document();
        let mut cloned = saved.clone();
        let paragraph = terminal_decorated_paragraph(&mut cloned.sections[0].paragraphs).unwrap();
        assert_eq!(paragraph.text, text);
        assert_eq!(paragraph.char_count, count);
        assert_eq!(paragraph.line_segs, lines);
    }
}

fn collect_tail_decorations(node: &LayerNode, output: &mut Vec<(String, usize)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect_tail_decorations(child, output);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            collect_tail_decorations(child, output)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::TextDecoration {
                    run,
                    trim_trailing_spaces,
                    ..
                } = op
                {
                    output.push((run.text.clone(), *trim_trailing_spaces));
                }
            }
        }
    }
}

fn tail_line_trims(node: &LayerNode, output: &mut Vec<Vec<(String, usize)>>) {
    match &node.kind {
        LayerNodeKind::Group {
            children,
            group_kind,
            ..
        } => {
            if matches!(group_kind, rhwp::paint::layer_tree::GroupKind::TextLine(_)) {
                let mut decorations = Vec::new();
                collect_tail_decorations(node, &mut decorations);
                if decorations.iter().any(|(text, _)| {
                    text.contains("특정점포의 장에게 금융거래정보의 제출을 요구할 수 있으며")
                }) {
                    output.push(decorations);
                }
            }
            for child in children {
                tail_line_trims(child, output);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            tail_line_trims(child, output)
        }
        LayerNodeKind::Leaf { .. } => {}
    }
}

#[test]
fn inherited_tail_preserves_authored_decoration_in_the_line_budget() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/basic/issue2007_nested_cell_pagination_42065.hwp"
    ))
    .unwrap();
    core.set_font_environment(Some(
        rhwp::renderer::font_environment::FontEnvironment::from_json(
            r#"{"id":"stored-tail-metric-contract","substitutions":{"굴림":"Gulim"}}"#,
        )
        .unwrap(),
    ))
    .unwrap();
    let mut found = Vec::new();
    for page in 0..core.page_count() {
        let tree = core
            .build_page_layer_tree_with_profile(page, RenderProfile::Print)
            .unwrap();
        tail_line_trims(&tree.root, &mut found);
    }
    assert_eq!(
        found.len(),
        1,
        "one source line owns this trailing decoration"
    );
    let line = found.pop().unwrap();
    let text: String = line.iter().map(|(text, _)| text.as_str()).collect();
    assert!(
        text.ends_with("못한다. "),
        "authored U+0020 stays in the paint source: {text:?}"
    );
    assert_eq!(
        line.iter().map(|(_, trim)| trim).sum::<usize>(),
        0,
        "authored trailing spaces keep their underline in the formatted line budget"
    );
}

fn find_title_field_trims(node: &LayerNode, output: &mut Vec<Vec<(String, usize)>>) {
    match &node.kind {
        LayerNodeKind::Group {
            children,
            group_kind,
            ..
        } => {
            if matches!(group_kind, rhwp::paint::layer_tree::GroupKind::TextLine(_)) {
                let mut decorations = Vec::new();
                collect_tail_decorations(node, &mut decorations);
                let text: String = decorations.iter().map(|(text, _)| text.as_str()).collect();
                if text.trim() == "제목" && text.ends_with(' ') {
                    output.push(decorations);
                }
            }
            for child in children {
                find_title_field_trims(child, output);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            find_title_field_trims(child, output)
        }
        LayerNodeKind::Leaf { .. } => {}
    }
}

#[test]
fn independently_styled_blank_field_keeps_its_occupancy_and_underline() {
    for (format, bytes) in [
        (
            "HWP",
            include_bytes!("../../samples/2025 행정업무운영 편람(최종).hwp").as_slice(),
        ),
        (
            "HWPX",
            include_bytes!("../../samples/2025 행정업무운영 편람(최종).hwpx").as_slice(),
        ),
    ] {
        let mut core = DocumentCore::from_bytes(bytes).unwrap();
        core.set_hangul2024_compat(true);
        // Independent Print p63: the title label and its authored blank field
        // have distinct character shapes. No font program or absolute px pin.
        let tree = core.build_page_render_tree(62).unwrap();
        let mut nodes = Vec::new();
        collect_tail_render_nodes(&tree.root, &mut nodes);
        let lines: Vec<_> = nodes
            .iter()
            .filter(|node| {
                if !matches!(node.node_type, RenderNodeType::TextLine(_)) {
                    return false;
                }
                let text: String = node
                    .children
                    .iter()
                    .filter_map(|child| match &child.node_type {
                        RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                        _ => None,
                    })
                    .collect();
                text.trim() == "제목" && text.ends_with(' ')
            })
            .collect();
        assert_eq!(
            lines.len(),
            1,
            "{format}: original form field is owned once"
        );
        let line = lines[0];
        let runs: Vec<_> = line
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) => Some((node.bbox, run)),
                _ => None,
            })
            .collect();
        let label = runs.iter().find(|(_, run)| run.text == "제목").unwrap();
        let tail = runs.last().unwrap();
        assert_eq!(
            tail.1.text,
            " ".repeat(43),
            "{format}: authored field content"
        );
        assert_ne!(
            label.1.style.font_size, tail.1.style.font_size,
            "the source assigns a separate character shape to the field"
        );
        assert!(tail.0.x >= label.0.x + label.0.width - 1e-6);
        // One original HWPUNIT (1/7200 inch) covers source-unit quantization.
        assert!(
            tail.0.x + tail.0.width <= line.bbox.x + line.bbox.width + 96.0 / 7200.0,
            "{format}: the field must consume the line budget, not hang outside: {:?} in {:?}",
            tail.0,
            line.bbox
        );
        let layer = core
            .build_page_layer_tree_with_profile(62, RenderProfile::Print)
            .unwrap();
        let mut fields = Vec::new();
        find_title_field_trims(&layer.root, &mut fields);
        assert_eq!(
            fields.len(),
            1,
            "{format}: the source field decoration is unique"
        );
        let field = fields.pop().unwrap();
        assert_eq!(
            field.iter().map(|(_, trim)| trim).sum::<usize>(),
            0,
            "{format}: the independent field keeps all authored underline advances"
        );
        assert_eq!(
            field
                .iter()
                .map(|(text, _)| text.as_str())
                .collect::<String>(),
            format!("제목{}", " ".repeat(43))
        );
    }
}

fn find_same_style_fields(node: &LayerNode, output: &mut Vec<Vec<(String, usize)>>) {
    match &node.kind {
        LayerNodeKind::Group {
            children,
            group_kind,
            ..
        } => {
            if matches!(group_kind, rhwp::paint::layer_tree::GroupKind::TextLine(_)) {
                let mut decorations = Vec::new();
                collect_tail_decorations(node, &mut decorations);
                let text: String = decorations.iter().map(|(text, _)| text.as_str()).collect();
                if text.trim_start().starts_with("제목vv○○○○○") {
                    output.push(decorations);
                }
            }
            for child in children {
                find_same_style_fields(child, output);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            find_same_style_fields(child, output)
        }
        LayerNodeKind::Leaf { .. } => {}
    }
}

#[test]
fn same_style_authored_fields_keep_all_spaces_and_their_underline() {
    for (format, bytes) in [
        (
            "HWP",
            include_bytes!("../../samples/2025 행정업무운영 편람(최종).hwp").as_slice(),
        ),
        (
            "HWPX",
            include_bytes!("../../samples/2025 행정업무운영 편람(최종).hwpx").as_slice(),
        ),
    ] {
        let mut core = DocumentCore::from_bytes(bytes).unwrap();
        core.set_hangul2024_compat(true);
        // Independent Print p60 has three title fields. The last field reaches
        // the line's right edge; a gray background must not hide missing ink.
        let tree = core.build_page_render_tree(59).unwrap();
        let mut nodes = Vec::new();
        collect_tail_render_nodes(&tree.root, &mut nodes);
        let mut texts = Vec::new();
        for line in nodes
            .iter()
            .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        {
            let runs: Vec<_> = line
                .children
                .iter()
                .filter_map(|child| {
                    if let RenderNodeType::TextRun(run) = &child.node_type {
                        Some((child.bbox, run))
                    } else {
                        None
                    }
                })
                .collect();
            let text: String = runs.iter().map(|(_, run)| run.text.as_str()).collect();
            if !text.trim_start().starts_with("제목vv○○○○○") {
                continue;
            }
            let label = runs.iter().find(|(_, run)| run.text == "제목").unwrap();
            let tail = runs.last().unwrap();
            assert!(
                tail.1.text.ends_with(' '),
                "{format}: authored spaces remain in the field"
            );
            assert_eq!(
                label.1.char_shape_id, tail.1.char_shape_id,
                "same source character shape owns the field"
            );
            assert!(tail.0.x >= label.0.x + label.0.width - 1e-6);
            assert!(
                tail.0.x + tail.0.width <= line.bbox.x + line.bbox.width + 96.0 / 7200.0,
                "{format}: trailing field occupies the remaining line budget: {:?} in {:?}",
                tail.0,
                line.bbox
            );
            texts.push(text);
        }
        assert_eq!(
            texts.len(),
            3,
            "{format}: all three source fields are present exactly once"
        );
        let layer = core
            .build_page_layer_tree_with_profile(59, RenderProfile::Print)
            .unwrap();
        let mut fields = Vec::new();
        find_same_style_fields(&layer.root, &mut fields);
        assert_eq!(
            fields.len(),
            texts.len(),
            "{format}: no field decoration is lost"
        );
        for field in fields {
            assert!(
                field.iter().all(|(_, trim)| *trim == 0),
                "{format}: authored SP ink cannot be removed"
            );
            let text: String = field.iter().map(|(text, _)| text.as_str()).collect();
            assert!(text.ends_with(' '));
            assert!(
                texts
                    .iter()
                    .any(|source| source.trim_start() == text.trim_start()),
                "{format}: all field text and spaces are preserved"
            );
        }
    }
}
