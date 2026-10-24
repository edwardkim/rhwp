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

#[test]
fn legacy_transparent_color_keeps_operator_spacing() {
    let mut equation = equation();
    equation.script = "가=나".into();
    let plain = rendered(&document(equation.clone())).1.layout_box;
    equation.script = "가 COLOR{255,0,0}{=} 나".into();
    let colored = rendered(&document(equation)).1.layout_box;
    assert!(
        (plain.width - colored.width).abs() < 1e-9,
        "a paint color wrapper must preserve logical operator gaps"
    );
}

#[test]
fn legacy_percent_selects_equation_pua() {
    let mut equation = equation();
    equation.script = "%".into();
    let legacy = rendered(&document(equation.clone())).1;
    assert!(serde_json::to_string(&legacy.layout_box)
        .unwrap()
        .contains('\u{e042}'));
    fn percent_width(layout: &rhwp::renderer::equation::layout::LayoutBox) -> Option<f64> {
        use rhwp::renderer::equation::layout::LayoutKind;
        match &layout.kind {
            LayoutKind::HancomGlyphs(run) if run.glyphs == "\u{e042}" => Some(run.width),
            LayoutKind::Row(children) => children.iter().find_map(percent_width),
            _ => None,
        }
    }
    assert_eq!(
        percent_width(&legacy.layout_box).expect("percent paint glyph") / legacy.font_size,
        1.0,
        "independent HFT Print has a full-em percent glyph slot; outer frame inset is separate"
    );
    equation.version_info = "Equation Version 60".into();
    let modern = rendered(&document(equation)).1;
    assert!(!serde_json::to_string(&modern.layout_box)
        .unwrap()
        .contains('\u{e042}'));
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
fn compound_fraction_color_keeps_the_same_frame_and_advance() {
    let mut source = equation();
    source.script = "가 TIMES LEFT ( {나} over {다} RIGHT )".into();
    let plain = rendered(&document(source.clone())).1.layout_box;
    source.script = format!("COLOR{{255,0,0}}{{{}}}", source.script);
    let colored = rendered(&document(source)).1.layout_box;
    assert_eq!(
        (plain.width, plain.height, plain.baseline),
        (colored.width, colored.height, colored.baseline),
        "a transparent paint wrapper does not choose another compound frame"
    );
}
