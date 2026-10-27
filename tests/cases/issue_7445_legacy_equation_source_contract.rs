//! Synthetic model/edit/paint contracts. The completed baseline67 page also
//! protects independent Print rule endpoints (Native/fresh WASM raw100%).
//! The current generator serialization and direct-reading evidence are in stage8 C427.
//! Baseline-only edits are qualified against exact same-input independent Print
//! in stage8 C429: original/current cell states and inline states are raw100% in both backends.
//! The public document consumer exposes the actual EquationNode layout. No new
//! test-only API or source-side unit-test baseline is needed.
use rhwp::model::control::{Control, Equation};
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::render_tree::{BoundingBox, EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn equation() -> Equation {
    let mut equation = Equation {
        script: "가 over 나".into(),
        font_name: "HYhwpEQ".into(),
        font_size: 1200,
        baseline: 67,
        ..Default::default()
    };
    equation.common.width = 10_000;
    equation.common.height = 2808;
    equation.common.treat_as_char = true;
    equation.common.attr = 1;
    equation
}

fn document(equation: Equation) -> DocumentCore {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public document supplies valid section/style defaults");
    let mut doc = core.document().clone();
    doc.sections.truncate(1);
    let mut paragraph = Paragraph::new_empty();
    paragraph.text = "\u{fffc}".into();
    paragraph.char_count = 2;
    paragraph.char_offsets = vec![0];
    paragraph.line_segs.clear();
    paragraph.controls = vec![Control::Equation(Box::new(equation))];
    doc.sections[0].paragraphs = vec![paragraph];
    core.set_document(doc);
    core
}

fn rendered(core: &DocumentCore) -> (BoundingBox, EquationNode) {
    fn collect(node: &RenderNode, out: &mut Vec<(BoundingBox, EquationNode)>) {
        if let RenderNodeType::Equation(equation) = &node.node_type {
            out.push((node.bbox.clone(), equation.clone()));
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let tree = core.build_page_render_tree(0).expect("public page paint");
    let mut equations = Vec::new();
    collect(&tree.root, &mut equations);
    assert_eq!(
        equations.len(),
        1,
        "one source equation, no loss or duplicate"
    );
    equations.remove(0)
}

// Inspect the actual public SVG paint origins, rather than the outer flow slot.
fn cjk_paint_origins(bounds: &BoundingBox, paint: &EquationNode) -> Vec<(String, f64, f64)> {
    let xml = format!("<svg>{}</svg>", paint.svg_content);
    let svg = roxmltree::Document::parse(&xml).expect("public equation SVG");
    svg.descendants()
        .filter(|n| n.has_tag_name("text"))
        .map(|n| {
            assert!(
                n.attribute("transform").is_none(),
                "these CJK controls use direct origins"
            );
            (
                n.text().unwrap().to_owned(),
                bounds.x + n.attribute("x").unwrap().parse::<f64>().unwrap(),
                bounds.y + n.attribute("y").unwrap().parse::<f64>().unwrap(),
            )
        })
        .collect()
}

fn assert_same_origins(before: &[(String, f64, f64)], after: &[(String, f64, f64)]) {
    assert_eq!(
        before.len(),
        2,
        "one numerator and denominator, no duplicate"
    );
    assert_eq!(before.len(), after.len());
    for (before, after) in before.iter().zip(after) {
        assert_eq!(before.0, after.0, "source content and order are preserved");
        assert!(
            (before.1 - after.1).abs() <= 1.0 / 75.0,
            "baseline-only edit keeps the inner horizontal origin"
        );
        assert!((before.2 - after.2).abs() <= 1.0 / 75.0, "baseline-only edit keeps the HFT program's inner vertical origin: {before:?} -> {after:?}");
    }
}

#[test]
fn legacy_operator_spacing_survives_glyph_projection() {
    for script in ["가=나", "가+나", "가-나", "+가"] {
        let mut equation = equation();
        equation.script = script.into();
        let projected = rendered(&document(equation.clone())).1.layout_box;
        equation.common.size_protect = true;
        let logical = rendered(&document(equation)).1.layout_box;
        assert!(
            (projected.width - logical.width).abs() < 1e-9,
            "glyph selection must preserve {script}'s logical operator slot and gaps"
        );
    }
}

fn completed_control(relative: &str) -> DocumentCore {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap()
}

fn paint_characters(
    layout: &rhwp::renderer::equation::layout::LayoutBox,
    x: f64,
    out: &mut Vec<(char, f64, f64)>,
) {
    use rhwp::renderer::equation::layout::LayoutKind;
    let x = x + layout.x;
    match &layout.kind {
        LayoutKind::Text(text) => {
            assert_eq!(text.chars().count(), 1, "독립 한글 대조군의 낱글자 소유");
            out.push((text.chars().next().unwrap(), x, layout.font_size));
        }
        LayoutKind::HancomGlyphs(run) => {
            assert_eq!(run.glyphs.chars().count(), run.positions.len());
            for (character, dx) in run.glyphs.chars().zip(&run.positions) {
                out.push((
                    if character == '\u{e047}' {
                        '='
                    } else {
                        character
                    },
                    x + dx,
                    layout.font_size,
                ));
            }
        }
        LayoutKind::Row(children) => {
            for child in children {
                paint_characters(child, x, out)
            }
        }
        LayoutKind::FontStyle { body, .. } | LayoutKind::Color { body, .. } => {
            paint_characters(body, x, out)
        }
        other => panic!("완료 대조군의 예상하지 않은 표시 경로: {other:?}"),
    }
}

#[test]
fn modern_color_boundary_preserves_print_owned_operator_gaps() {
    let mut actual = Vec::new();
    for name in ["plain", "color-grouped", "color-ungrouped"] {
        let core = completed_control(&format!("mydocs/pr/assets/legacy_literal_operator_plane_20261025/inputs/modern-group-{name}.hwpx"));
        assert_eq!(core.page_count(), 1, "독립 Print와 같은 쪽 소속");
        let paint = rendered(&core).1;
        let mut glyphs = Vec::new();
        paint_characters(&paint.layout_box, 0.0, &mut glyphs);
        assert_eq!(
            glyphs.iter().map(|x| x.0).collect::<String>(),
            "가=나",
            "내용·순서·누락·중복"
        );
        if name != "plain" {
            assert!(
                paint.svg_content.contains("#ff0000"),
                "COLOR의 실제 빨간 기호 보존"
            );
        }
        actual.push(glyphs);
    }
    for colored in &actual[1..] {
        let fs = actual[0][1].2;
        // 독립 Print에서 = 원점 차이/11.9902pt=0.210057em,
        // 나 원점 차이/11.9902pt=0.420115em. 절대 좌표나 구현 폭을 답으로 쓰지 않는다.
        assert!(
            ((actual[0][1].1 - colored[1].1) / fs - 0.21).abs() < 0.002,
            "COLOR 경계 앞 자동 여백을 중복하지 않는다"
        );
        assert!(
            ((actual[0][2].1 - colored[2].1) / fs - 0.42).abs() < 0.002,
            "COLOR 경계 양쪽 자동 여백을 중복하지 않는다"
        );
        assert_eq!(actual[0][0].1, colored[0].1, "앞 한글의 원점 보존");
    }
    assert_eq!(
        actual[1], actual[2],
        "두 유효 COLOR 구문의 독립 Print 원점이 같다"
    );
}

#[test]
fn percent_paint_preserves_independent_legacy_and_modern_font_ownership() {
    use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
    fn find(layout: &LayoutBox, modern: bool) -> bool {
        match &layout.kind {
            LayoutKind::HancomGlyphs(run) if !modern => {
                assert_eq!(run.font_family, "RHWP Hancom Legacy Equation");
                assert_eq!(run.glyphs, "%", "독립 구형 Type3의 원문 문자");
                assert_eq!(run.positions.len(), 1);
                assert_eq!(run.shears, vec![0.0], "정체 원본 프로그램");
                true
            }
            LayoutKind::FontGlyph {
                text, font_family, ..
            } if modern => {
                assert_eq!(text, "\u{e042}", "독립 현대 HyhwpEQ의 수학 PUA");
                assert!(font_family.starts_with("HYhwpEQ"));
                true
            }
            LayoutKind::Row(children) => children.iter().any(|child| find(child, modern)),
            _ => false,
        }
    }
    for (name, modern) in [("legacy", false), ("modern", true)] {
        let core=completed_control(&format!("mydocs/pr/assets/legacy_literal_operator_plane_20261025/inputs/equation-cc9-{name}-percent.hwpx"));
        assert_eq!(core.page_count(), 1);
        let (bounds, paint) = rendered(&core);
        assert_eq!(paint.script, "%", "표시용 PUA로 원문을 치환하지 않는다");
        assert!(core.extract_page_text_native(0).unwrap().contains('%'));
        assert!(
            find(&paint.layout_box, modern),
            "같은 공통 표시 결과를 최종 paint가 소비한다"
        );
        assert!(
            (bounds.width - 10000.0 / 75.0).abs() < 1.0 / 75.0,
            "선택한 glyph와 별개인 원본 선언 프레임 보존"
        );
    }
}

#[test]
fn legacy_cjk_fallback_does_not_replace_the_stored_flow_frame() {
    let mut equation = equation();
    let unprotected = rendered(&document(equation.clone())).0;
    equation.common.size_protect = true;
    let protected = rendered(&document(equation.clone())).0;
    assert_eq!(
        (unprotected.width, unprotected.height),
        (protected.width, protected.height)
    );
    assert!((unprotected.height * 75.0 - f64::from(equation.common.height)).abs() < 1e-8);
}

#[test]
fn legacy_cjk_baseline_edit_preserves_the_inner_program_frame() {
    let mut equation = equation();
    equation.baseline = 50;
    let mut core = document(equation);
    core.set_equation_properties_native(0, 0, 0, None, None, r#"{"baseline":67}"#)
        .expect("public baseline edit to source percentage");
    let first = rendered(&core);
    assert_eq!(
        core.document().sections[0].paragraphs[0].line_segs[0].baseline_distance,
        (2808.0_f64 * 0.67).round() as i32
    );
    core.set_equation_properties_native(0, 0, 0, None, None, r#"{"baseline":80}"#)
        .expect("second public baseline edit");
    let second = rendered(&core);
    assert_eq!(
        core.document().sections[0].paragraphs[0].line_segs[0].baseline_distance,
        (2808.0_f64 * 0.80).round() as i32
    );
    assert_eq!(
        (first.0.width, first.0.height),
        (second.0.width, second.0.height)
    );
    assert_eq!(first.1.para_index, Some(0));
    assert_eq!(second.1.para_index, Some(0));
    assert_eq!(first.1.script, "가 over 나");
    assert_eq!(first.1.script, second.1.script);
    let before = cjk_paint_origins(&first.0, &first.1);
    let after = cjk_paint_origins(&second.0, &second.1);
    assert_eq!(
        before.iter().map(|p| p.0.as_str()).collect::<Vec<_>>(),
        ["가", "나"]
    );
    // Exact independent Print streams for the 67/80 edit states are identical.
    // The stored percentage belongs to the slot's host alignment, not a second
    // translation of the glyph program inside that slot.
    assert_same_origins(&before, &after);
}

#[test]
fn modern_cjk_reflow_keeps_its_existing_intrinsic_path() {
    let mut equation = equation();
    equation.version_info = "Equation Version 60".into();
    let reflowed = rendered(&document(equation.clone())).0;
    equation.common.size_protect = true;
    let protected = rendered(&document(equation)).0;
    assert_ne!(
        (reflowed.width, reflowed.height),
        (protected.width, protected.height)
    );
}

#[test]
fn legacy_script_edit_refreshes_the_frame_in_its_paint_context() {
    let mut core = document(equation());
    let before = core.document().sections[0].paragraphs[0].controls[0].clone();
    core.set_equation_properties_native(0, 0, 0, None, None, r#"{"script":"가가가 over 나"}"#)
        .expect("public equation edit");
    let Control::Equation(edited) = &core.document().sections[0].paragraphs[0].controls[0] else {
        panic!("edited equation");
    };
    let Control::Equation(before) = before else {
        unreachable!()
    };
    assert_ne!(
        (edited.common.width, edited.common.height),
        (before.common.width, before.common.height)
    );
    let paint = rendered(&core).1.layout_box;
    assert!((f64::from(edited.common.width) / 75.0 - paint.width).abs() <= 1.0 / 75.0);
    assert!((f64::from(edited.common.height) / 75.0 - paint.height).abs() <= 1.0 / 75.0);
    core.set_equation_properties_native(
        0,
        0,
        0,
        None,
        None,
        r#"{"script":"가가가가 over 나","width":7777}"#,
    )
    .expect("public edit with explicitly owned width");
    let Control::Equation(edited) = &core.document().sections[0].paragraphs[0].controls[0] else {
        unreachable!();
    };
    assert_eq!(edited.common.width, 7777);
}

#[test]
fn cell_baseline_edit_reflows_the_cell_owner_and_preserves_equation_membership() {
    use rhwp::model::table::{Cell, Table};
    let mut source = equation();
    source.baseline = 50;
    let mut core = document(source);
    // Generate a valid new frame through the public edit producer. Reusing the
    // compact 2808-HU frame with a different fraction can legitimately hit the
    // cell's top clamp, so a paint-motion expectation on that input is invalid.
    core.set_equation_properties_native(0, 0, 0, None, None, r#"{"script":"가가 over 나"}"#)
        .unwrap();
    let Control::Equation(authored) = &core.document().sections[0].paragraphs[0].controls[0] else {
        unreachable!()
    };
    let authored_height = f64::from(authored.common.height);
    let mut doc = core.document().clone();
    let content = doc.sections[0].paragraphs[0].clone();
    let mut table = Table::default();
    table.row_count = 1;
    table.col_count = 1;
    table.row_sizes = vec![1];
    table.cell_grid = vec![Some(0)];
    table.common.width = 30_000;
    table.common.height = 10_000;
    table.common.treat_as_char = true;
    table.common.attr = 1;
    table.cells = vec![Cell {
        row_span: 1,
        col_span: 1,
        width: 30_000,
        height: 10_000,
        paragraphs: vec![content],
        ..Default::default()
    }];
    doc.sections[0].paragraphs[0].controls = vec![Control::Table(Box::new(table))];
    doc.sections[0].paragraphs[0].line_segs.clear();
    core.set_document(doc);
    fn cell_bounds(node: &RenderNode, out: &mut Vec<BoundingBox>) {
        if matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.model_cell_index == Some(0))
        {
            out.push(node.bbox.clone());
        }
        for child in &node.children {
            cell_bounds(child, out);
        }
    }
    let mut previous: Option<(Vec<(String, f64, f64)>, i32)> = None;
    // Keep both percentages below the degenerate baseline == text_height
    // boundary. The common paragraph consumer intentionally reserves a
    // descender at that boundary; it is not a linear baseline edit contract.
    for percentage in [90, 95] {
        core.set_equation_properties_native(
            0,
            0,
            0,
            Some(0),
            Some(0),
            &format!(r#"{{"baseline":{percentage}}}"#),
        )
        .unwrap();
        let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[0] else {
            panic!("table owner");
        };
        let paragraph = &table.cells[0].paragraphs[0];
        assert_eq!(paragraph.line_segs.len(), 1);
        assert_eq!(
            paragraph.line_segs[0].baseline_distance,
            (authored_height * f64::from(percentage) / 100.0).round() as i32
        );
        let (bounds, paint) = rendered(&core);
        assert_eq!(paint.cell_index, Some(0));
        assert_eq!(paint.cell_para_index, Some(0));
        let tree = core.build_page_render_tree(0).unwrap();
        let mut cells = Vec::new();
        cell_bounds(&tree.root, &mut cells);
        assert_eq!(cells.len(), 1, "one cell owner");
        let cell = &cells[0];
        let cell_end = cell.y + cell.height;
        let equation_end = bounds.y + bounds.height;
        let roundoff = 8.0 * f64::EPSILON * cell_end.abs().max(equation_end.abs()).max(1.0);
        assert!(bounds.y + roundoff >= cell.y && equation_end <= cell_end + roundoff);
        let relative = BoundingBox::new(
            bounds.x - cell.x,
            bounds.y - cell.y,
            bounds.width,
            bounds.height,
        );
        let origins = cjk_paint_origins(&relative, &paint);
        assert_eq!(
            origins.iter().map(|p| p.0.as_str()).collect::<Vec<_>>(),
            ["가가", "나"]
        );
        let source_baseline = paragraph.line_segs[0].baseline_distance;
        if let Some((old_origins, old_source_baseline)) = previous {
            assert!(
                source_baseline > old_source_baseline,
                "the edited cell line still consumes its source baseline"
            );
            // Independent Print keeps both operands at the same cell-relative
            // origin for these baseline-only edits. Do not add the source
            // baseline delta to the equation's own display program again.
            assert_same_origins(&old_origins, &origins);
        }
        previous = Some((origins, source_baseline));
    }
}

#[test]
fn completed_legacy_fraction_rule_uses_content_bounds_instead_of_registration_space() {
    let mut core = document(equation());
    core.set_hangul2024_compat(true);
    let (bounds, paint) = rendered(&core);
    assert_eq!(paint.para_index, Some(0));
    assert_eq!(paint.script, "가 over 나");
    let fragment = rhwp::renderer::equation::svg_render::render_equation_svg(
        &paint.layout_box,
        "#000",
        paint.font_size,
    );
    let xml = format!("<svg>{fragment}</svg>");
    let svg = roxmltree::Document::parse(&xml).unwrap();
    let rules: Vec<_> = svg
        .descendants()
        .filter(|n| n.has_tag_name("line"))
        .collect();
    assert_eq!(rules.len(), 1, "one fraction rule, no duplicate");
    let value = |n: roxmltree::Node<'_, '_>, key| n.attribute(key).unwrap().parse::<f64>().unwrap();
    let rule = rules[0];
    let left = bounds.x + value(rule, "x1");
    let right = bounds.x + value(rule, "x2");
    // Independent Hancom2024 Print of the exact current model-test generator:
    // p1 rule x=79.957333..101.865333 CSSpx, measured from PDF paths.
    // Native/fresh WASM 192DPI raw2px are both100%; compare in96DPI CSSpx.
    assert!(
        (left - 79.95733333333334).abs() <= 2.0,
        "the rule must not consume the CJK registration space: left={left}"
    );
    assert!(
        (right - 101.86533333333334).abs() <= 2.0,
        "independent rule end: right={right}"
    );
    let text: Vec<_> = svg
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .collect();
    assert_eq!(text.len(), 2, "both source operands, no loss or duplicate");
    assert_eq!(text[0].text(), Some("가"));
    assert_eq!(text[1].text(), Some("나"));
    let rule_y = value(rule, "y1");
    assert_eq!(rule_y, value(rule, "y2"));
    assert!(
        value(text[0], "y") < rule_y && rule_y < value(text[1], "y"),
        "the rule stays between the owned numerator and denominator"
    );
    for operand in text {
        let x = bounds.x + value(operand, "x");
        assert!(
            left <= x && x + paint.font_size <= right,
            "rule contains the full-em CJK operand, without moving its glyph"
        );
    }
}

#[test]
fn compound_fraction_preserves_a_declared_frame_that_contains_its_natural_paint() {
    let mut source = equation();
    source.script = "가 TIMES LEFT ( {나} over {다} RIGHT )".into();
    source.common.width = 20_000;
    source.common.height = 6_000;
    let (bounds, paint) = rendered(&document(source.clone()));
    assert!(paint.layout_box.width < f64::from(source.common.width) / 75.0);
    assert!(paint.layout_box.height < f64::from(source.common.height) / 75.0);
    assert!(
        (bounds.width * 75.0 - f64::from(source.common.width)).abs() < 1e-8,
        "a containing source reservation is not replaced by the smaller compound paint"
    );
    assert!(
        (bounds.height * 75.0 - f64::from(source.common.height)).abs() < 1e-8,
        "source height keeps the same frame owner"
    );
    source.common.size_protect = true;
    let protected = rendered(&document(source)).0;
    assert_eq!(
        (bounds.width, bounds.height),
        (protected.width, protected.height),
        "size protection does not choose another reservation when the source frame already fits"
    );
}

#[test]
fn legacy_color_spelling_keeps_the_independent_print_empty_paint() {
    // 빈 버전 COLOR는 현대 RGB 래퍼가 아니다. 같은 원본의 독립 Print는 비어 있다.
    // 현대 COLOR의 표시·간격은 modern_color_boundary_preserves_print_owned_operator_gaps가 보호한다.
    let core = completed_control("mydocs/pr/assets/legacy_literal_operator_plane_20261025/inputs/equation-cc9-legacy-color.hwpx");
    assert_eq!(core.page_count(), 1, "독립 Print의 한 쪽 소속");
    let (_, paint) = rendered(&core);
    assert_eq!(
        paint.script, "가 COLOR{255,0,0}{=} 나",
        "원본의 앞뒤 글자를 포함한 구형 스크립트 보존"
    );
    assert!(paint.svg_content.is_empty(), "독립 Print와 같은 빈 표시");
    assert_eq!(
        paint.layout_box.width, 0.0,
        "독립 Print에 없는 수식을 만들지 않음"
    );
    assert_eq!(
        paint.layout_box.height, 0.0,
        "현대 색상 프레임을 잘못 적용하지 않음"
    );
}
