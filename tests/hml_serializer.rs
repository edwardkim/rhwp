use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::{BinData, BinDataContent};
use rhwp::model::control::Control;
use rhwp::model::page::BindingMethod;
use rhwp::model::paragraph::{CharShapeRef, ColumnBreakType, Paragraph};
use rhwp::model::shape::ShapeObject;
use rhwp::model::style::BorderLineType;
use rhwp::model::table::TablePageBreak;
use rhwp::parser::hml::PreservedFragment;
use rhwp::parser::{detect_format, parse_document_with_metadata, FileFormat};
use rhwp::serializer::hml::{serialize_hml, HmlExportError};

fn equation_scripts(document: &rhwp::model::document::Document) -> Vec<&str> {
    document
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Equation(equation) => Some(equation.script.as_str()),
            _ => None,
        })
        .collect()
}

fn first_equation_mut(
    document: &mut rhwp::model::document::Document,
) -> &mut rhwp::model::control::Equation {
    document
        .sections
        .iter_mut()
        .flat_map(|section| &mut section.paragraphs)
        .flat_map(|paragraph| &mut paragraph.controls)
        .find_map(|control| match control {
            Control::Equation(equation) => Some(equation.as_mut()),
            _ => None,
        })
        .expect("fixture equation")
}

fn first_equation(document: &rhwp::model::document::Document) -> &rhwp::model::control::Equation {
    document
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| match control {
            Control::Equation(equation) => Some(equation.as_ref()),
            _ => None,
        })
        .expect("fixture equation")
}

#[test]
fn equation_exports_canonically_escapes_and_reparses_edited_and_untouched_scripts() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "fixtures/hml/exambank_math_equations_min.hml"
    ))
    .expect("equation fixture should import");
    core.set_equation_properties_native(
        0,
        2,
        0,
        None,
        None,
        r#"{"script":"a < b & \"c\"","treatAsChar":true}"#,
    )
    .expect("public equation edit should apply");

    let exported = core
        .export_hml_native()
        .expect("equations should export losslessly");
    let xml = std::str::from_utf8(&exported).expect("HML is UTF-8");
    assert!(xml.contains(
        "<EQUATION BaseLine=\"65\" BaseUnit=\"1000\" TextColor=\"0\" Version=\"Equation Version 60\"><SCRIPT>a &lt; b &amp; \"c\"</SCRIPT></EQUATION>"
    ));

    let reparsed = DocumentCore::from_bytes(&exported).expect("exported HML should reparse");
    assert_eq!(
        equation_scripts(reparsed.document()),
        ["a < b & \"c\"", "x^2 +1", "3", "3"]
    );
}

#[test]
fn equation_attributes_preserve_asymmetric_color_and_optional_font() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><EQUATION BaseLine="-4" BaseUnit="1200" TextColor="1122867" Version="v1" Font="Hancom"><SCRIPT>x</SCRIPT></EQUATION></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let core = DocumentCore::from_bytes(xml).expect("equation attributes should import");

    let exported = core.export_hml_native().expect("equation should export");
    let output = std::str::from_utf8(&exported).expect("HML is UTF-8");
    assert!(output.contains(
        "<EQUATION BaseLine=\"-4\" BaseUnit=\"1200\" TextColor=\"1122867\" Version=\"v1\" Font=\"Hancom\">"
    ));

    let reparsed = DocumentCore::from_bytes(&exported).expect("exported HML should reparse");
    let equation = first_equation(reparsed.document());
    assert_eq!(equation.color, 0x0011_2233);
}

#[test]
fn equation_invalid_xml_and_stale_offsets_are_aggregated() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "fixtures/hml/exambank_math_equations_min.hml"
    ))
    .expect("equation fixture should import");
    first_equation_mut(core.document_mut()).script.push('\u{1}');
    let paragraph = core.document_mut().sections[0]
        .paragraphs
        .iter_mut()
        .find(|paragraph| !paragraph.controls.is_empty())
        .expect("equation paragraph");
    paragraph.char_offsets.pop();

    let error = core
        .export_hml_native()
        .expect_err("all invalid equation state should block before writing");
    assert!(error.blockers().iter().any(|blocker| {
        blocker.code == "HML_INVALID_XML_CHARACTER"
            && blocker.xml_path.ends_with("/EQUATION/SCRIPT")
    }));
    assert!(error.blockers().iter().any(|blocker| {
        blocker.code == "HML_UNSUPPORTED_IR" && blocker.message.contains("offset")
    }));
}

#[test]
fn unknown_equation_semantics_blockers_survive_equation_edits() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><EQUATION FutureAttr="1"><SCRIPT>x</SCRIPT><FUTURE/></EQUATION></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let mut core = DocumentCore::from_bytes(xml).expect("unknown equation HML should import");
    first_equation_mut(core.document_mut()).script = "edited".to_string();

    let error = core
        .export_hml_native()
        .expect_err("unknown equation semantics must remain durable blockers");
    let blockers = error
        .blockers()
        .iter()
        .filter(|blocker| blocker.code == "HML_UNSUPPORTED_EQUATION_SEMANTICS")
        .collect::<Vec<_>>();
    assert_eq!(blockers.len(), 2);
    assert!(blockers
        .iter()
        .any(|blocker| blocker.xml_path.ends_with("/@FutureAttr")));
    assert!(blockers
        .iter()
        .any(|blocker| blocker.xml_path.ends_with("/FUTURE")));
}

fn assert_public_ir_equivalent(before: &DocumentCore, after: &DocumentCore) {
    let before = before.document();
    let after = after.document();

    assert_eq!(after.sections.len(), before.sections.len());
    assert_resource_counts(before, after);
    assert_head_resources_equivalent(before, after);
    for (after_section, before_section) in after.sections.iter().zip(&before.sections) {
        assert_section_equivalent(after_section, before_section);
    }
}

fn assert_resource_counts(
    before: &rhwp::model::document::Document,
    after: &rhwp::model::document::Document,
) {
    let counts = |document: &rhwp::model::document::Document| {
        (
            document.doc_info.font_faces.len(),
            document.doc_info.char_shapes.len(),
            document.doc_info.para_shapes.len(),
            document.doc_info.border_fills.len(),
            document.doc_info.styles.len(),
        )
    };
    assert_eq!(counts(after), counts(before));
}

fn assert_section_equivalent(
    after: &rhwp::model::document::Section,
    before: &rhwp::model::document::Section,
) {
    let page_values = |section: &rhwp::model::document::Section| {
        let page = &section.section_def.page_def;
        (
            page.width,
            page.height,
            page.margin_left,
            page.margin_right,
            page.margin_top,
            page.margin_bottom,
            page.margin_header,
            page.margin_footer,
            page.margin_gutter,
            page.landscape,
        )
    };
    assert_eq!(page_values(after), page_values(before));
    assert_eq!(after.paragraphs.len(), before.paragraphs.len());
    for (after, before) in after.paragraphs.iter().zip(&before.paragraphs) {
        assert_paragraph_equivalent(after, before);
    }
}

fn assert_paragraph_equivalent(after: &Paragraph, before: &Paragraph) {
    assert_eq!(after.text, before.text);
    assert_eq!(after.para_shape_id, before.para_shape_id);
    assert_eq!(after.style_id, before.style_id);
    assert_char_shape_refs_equivalent(after, before);
    assert_eq!(after.controls.len(), before.controls.len());
    for (after, before) in after.controls.iter().zip(&before.controls) {
        assert_control_equivalent(after, before);
    }
}

fn assert_char_shape_refs_equivalent(after: &Paragraph, before: &Paragraph) {
    assert_eq!(after.char_shapes.len(), before.char_shapes.len());
    for (after, before) in after.char_shapes.iter().zip(&before.char_shapes) {
        assert_eq!(after.start_pos, before.start_pos);
        assert_eq!(after.char_shape_id, before.char_shape_id);
    }
}

fn assert_control_equivalent(after: &Control, before: &Control) {
    match (after, before) {
        (Control::Table(after), Control::Table(before)) => assert_table_equivalent(after, before),
        (Control::Shape(after), Control::Shape(before)) => {
            let (ShapeObject::Rectangle(after), ShapeObject::Rectangle(before)) =
                (after.as_ref(), before.as_ref())
            else {
                panic!("fixture contains an unexpected shape kind");
            };
            assert_rectangle_equivalent(after, before);
        }
        _ => panic!("fixture control kind changed during HML round-trip"),
    }
}

fn assert_table_equivalent(after: &rhwp::model::table::Table, before: &rhwp::model::table::Table) {
    assert_eq!(after.row_count, before.row_count);
    assert_eq!(after.col_count, before.col_count);
    assert_eq!(after.cell_spacing, before.cell_spacing);
    assert_padding_equivalent(&after.padding, &before.padding);
    assert_eq!(after.border_fill_id, before.border_fill_id);
    assert_eq!(after.cells.len(), before.cells.len());
    assert_common_object_equivalent(&after.common, &before.common);
    for (after, before) in after.cells.iter().zip(&before.cells) {
        assert_cell_equivalent(after, before);
    }
}

fn assert_cell_equivalent(after: &rhwp::model::table::Cell, before: &rhwp::model::table::Cell) {
    assert_eq!(
        (after.col, after.row, after.col_span, after.row_span),
        (before.col, before.row, before.col_span, before.row_span)
    );
    assert_eq!((after.width, after.height), (before.width, before.height));
    assert_padding_equivalent(&after.padding, &before.padding);
    assert_eq!(after.border_fill_id, before.border_fill_id);
    assert_eq!(after.paragraphs.len(), before.paragraphs.len());
}

fn assert_rectangle_equivalent(
    after: &rhwp::model::shape::RectangleShape,
    before: &rhwp::model::shape::RectangleShape,
) {
    assert_eq!(after.x_coords, before.x_coords);
    assert_eq!(after.y_coords, before.y_coords);
    assert_common_object_equivalent(&after.common, &before.common);
    let shape_values = |rectangle: &rhwp::model::shape::RectangleShape| {
        let shape = &rectangle.drawing.shape_attr;
        (
            shape.offset_x,
            shape.offset_y,
            shape.original_width,
            shape.original_height,
            shape.current_width,
            shape.current_height,
            rectangle.drawing.border_line.width,
            rectangle.drawing.border_line.attr,
            rectangle.drawing.fill.alpha,
        )
    };
    assert_eq!(shape_values(after), shape_values(before));
    assert_solid_fill_equivalent(
        after.drawing.fill.solid.as_ref(),
        before.drawing.fill.solid.as_ref(),
    );
    assert_eq!(text_box_texts(after), text_box_texts(before));
}

fn text_box_texts(rectangle: &rhwp::model::shape::RectangleShape) -> Option<Vec<&str>> {
    rectangle.drawing.text_box.as_ref().map(|text_box| {
        text_box
            .paragraphs
            .iter()
            .map(|paragraph| paragraph.text.as_str())
            .collect()
    })
}

fn assert_common_object_equivalent(
    after: &rhwp::model::shape::CommonObjAttr,
    before: &rhwp::model::shape::CommonObjAttr,
) {
    assert_eq!(after.vertical_offset, before.vertical_offset);
    assert_eq!(after.horizontal_offset, before.horizontal_offset);
    assert_eq!(after.width, before.width);
    assert_eq!(after.height, before.height);
    assert_eq!(after.treat_as_char, before.treat_as_char);
    assert_eq!(after.flow_with_text, before.flow_with_text);
    assert_eq!(after.allow_overlap, before.allow_overlap);
    assert_eq!(after.vert_rel_to, before.vert_rel_to);
    assert_eq!(after.vert_align, before.vert_align);
    assert_eq!(after.horz_rel_to, before.horz_rel_to);
    assert_eq!(after.horz_align, before.horz_align);
    assert_eq!(after.text_wrap, before.text_wrap);
}

fn assert_padding_equivalent(after: &rhwp::model::Padding, before: &rhwp::model::Padding) {
    assert_eq!(after.left, before.left);
    assert_eq!(after.right, before.right);
    assert_eq!(after.top, before.top);
    assert_eq!(after.bottom, before.bottom);
}

fn assert_solid_fill_equivalent(
    after: Option<&rhwp::model::style::SolidFill>,
    before: Option<&rhwp::model::style::SolidFill>,
) {
    assert_eq!(after.is_some(), before.is_some());
    if let (Some(after), Some(before)) = (after, before) {
        assert_eq!(after.background_color, before.background_color);
        assert_eq!(after.pattern_color, before.pattern_color);
        assert_eq!(after.pattern_type, before.pattern_type);
    }
}

fn assert_head_resources_equivalent(
    before: &rhwp::model::document::Document,
    after: &rhwp::model::document::Document,
) {
    assert_fonts_equivalent(before, after);
    assert_char_shapes_equivalent(before, after);
    assert_para_shapes_equivalent(before, after);
    assert_border_fills_equivalent(before, after);
    assert_styles_equivalent(before, after);
}

fn assert_fonts_equivalent(
    before: &rhwp::model::document::Document,
    after: &rhwp::model::document::Document,
) {
    for (after_faces, before_faces) in after
        .doc_info
        .font_faces
        .iter()
        .zip(&before.doc_info.font_faces)
    {
        for (after_font, before_font) in after_faces.iter().zip(before_faces) {
            assert_eq!(after_font.name, before_font.name);
            assert_eq!(after_font.alt_type, before_font.alt_type);
        }
    }
}

fn assert_char_shapes_equivalent(
    before: &rhwp::model::document::Document,
    after: &rhwp::model::document::Document,
) {
    for (after_shape, before_shape) in after
        .doc_info
        .char_shapes
        .iter()
        .zip(&before.doc_info.char_shapes)
    {
        assert_eq!(after_shape.font_ids, before_shape.font_ids);
        assert_eq!(after_shape.ratios, before_shape.ratios);
        assert_eq!(after_shape.spacings, before_shape.spacings);
        assert_eq!(after_shape.relative_sizes, before_shape.relative_sizes);
        assert_eq!(after_shape.char_offsets, before_shape.char_offsets);
        assert_eq!(after_shape.base_size, before_shape.base_size);
        assert_eq!(after_shape.border_fill_id, before_shape.border_fill_id);
        assert_eq!(after_shape.text_color, before_shape.text_color);
        assert_eq!(after_shape.shade_color, before_shape.shade_color);
    }
}

fn assert_para_shapes_equivalent(
    before: &rhwp::model::document::Document,
    after: &rhwp::model::document::Document,
) {
    for (after_shape, before_shape) in after
        .doc_info
        .para_shapes
        .iter()
        .zip(&before.doc_info.para_shapes)
    {
        assert_eq!(after_shape.alignment, before_shape.alignment);
        assert_eq!(after_shape.tab_def_id, before_shape.tab_def_id);
        assert_eq!(after_shape.para_level, before_shape.para_level);
        assert_eq!(after_shape.margin_left, before_shape.margin_left);
        assert_eq!(after_shape.margin_right, before_shape.margin_right);
        assert_eq!(after_shape.indent, before_shape.indent);
        assert_eq!(after_shape.spacing_before, before_shape.spacing_before);
        assert_eq!(after_shape.spacing_after, before_shape.spacing_after);
        assert_eq!(after_shape.line_spacing, before_shape.line_spacing);
        assert_eq!(
            after_shape.line_spacing_type,
            before_shape.line_spacing_type
        );
        assert_eq!(after_shape.border_fill_id, before_shape.border_fill_id);
    }
}

fn assert_border_fills_equivalent(
    before: &rhwp::model::document::Document,
    after: &rhwp::model::document::Document,
) {
    for (after_fill, before_fill) in after
        .doc_info
        .border_fills
        .iter()
        .zip(&before.doc_info.border_fills)
    {
        for (after_line, before_line) in after_fill.borders.iter().zip(before_fill.borders) {
            assert_eq!(after_line.line_type, before_line.line_type);
            assert_eq!(after_line.width, before_line.width);
        }
    }
}

fn assert_styles_equivalent(
    before: &rhwp::model::document::Document,
    after: &rhwp::model::document::Document,
) {
    for (after_style, before_style) in after.doc_info.styles.iter().zip(&before.doc_info.styles) {
        assert_eq!(after_style.local_name, before_style.local_name);
        assert_eq!(after_style.english_name, before_style.english_name);
        assert_eq!(after_style.style_type, before_style.style_type);
        assert_eq!(after_style.next_style_id, before_style.next_style_id);
        assert_eq!(after_style.lang_id, before_style.lang_id);
        assert_eq!(after_style.para_shape_id, before_style.para_shape_id);
        assert_eq!(after_style.char_shape_id, before_style.char_shape_id);
    }
}

fn occurrence_count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|window| *window == needle)
        .count()
}

fn add_redundant_char_shape(paragraph: &mut Paragraph) -> bool {
    let Some(&start_pos) = paragraph.char_offsets.get(1) else {
        return false;
    };
    let Some(char_shape_id) = paragraph
        .char_shapes
        .iter()
        .rev()
        .find(|reference| reference.start_pos <= start_pos)
        .map(|reference| reference.char_shape_id)
    else {
        return false;
    };
    paragraph.char_shapes.push(CharShapeRef {
        start_pos,
        char_shape_id,
    });
    paragraph.char_shapes.sort_by_key(|shape| shape.start_pos);
    true
}

#[test]
fn lawful_hml_fixtures_export_and_reparse_with_equivalent_public_ir() {
    for bytes in [
        include_bytes!("../samples/hml/aligns.hml").as_slice(),
        include_bytes!("../samples/hml/formatting_table.hml").as_slice(),
    ] {
        let before = DocumentCore::from_bytes(bytes).expect("fixture should import");
        let exported = before.export_hml_native().expect("HML should export");

        assert_eq!(detect_format(&exported), FileFormat::Hml);
        let exported_xml = std::str::from_utf8(&exported).expect("HML output must be UTF-8");
        assert!(exported_xml.contains("<HWPML Version=\"2.91\""));
        assert!(exported_xml.contains("SubVersion=\"10.0.0.0\""));
        assert!(exported_xml.contains("Style=\"embed\""));
        assert!(!exported_xml.contains("<DOCSETTING"));
        assert!(!exported_xml.contains("<hp:"));
        let after = DocumentCore::from_bytes(&exported).expect("exported HML should reparse");
        assert_public_ir_equivalent(&before, &after);
        assert_eq!(
            after.hml_metadata().unwrap().warnings,
            before.hml_metadata().unwrap().warnings
        );

        for fragment in &before.hml_metadata().unwrap().preserved_fragments {
            assert_eq!(
                occurrence_count(&exported, fragment.raw_xml.as_bytes()),
                1,
                "preserved fragment must be emitted exactly once: {}",
                fragment.xml_path
            );
        }
    }
}

#[test]
fn non_hml_origin_is_refused_with_typed_source_blocker() {
    let core = DocumentCore::new_empty();

    let error = core
        .export_hml_native()
        .expect_err("non-HML origin must not export as HML");
    let HmlExportError::UnsupportedSourceFormat { actual, blockers } = error else {
        panic!("expected typed source-format refusal");
    };

    assert_eq!(actual, FileFormat::Hwp);
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].code, "HML_SOURCE_REQUIRED");
    assert_eq!(blockers[0].xml_path, "/HWPML");
    assert!(!blockers[0].message.is_empty());
}

#[test]
fn export_canonicalizes_version_and_retains_stored_root_attributes() {
    let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<HWPML Version="2.9" SubVersion="8.0.0.0" Style="custom-style">
  <HEAD><MAPPINGTABLE/></HEAD>
  <BODY><SECTION><P><TEXT CharShape="0"><CHAR>root metadata</CHAR></TEXT></P></SECTION></BODY>
  <TAIL/>
</HWPML>"#;
    let core = DocumentCore::from_bytes(bytes).expect("HML 2.9 should import");
    let metadata = core.hml_metadata().expect("HML metadata should exist");
    assert_eq!(metadata.sub_version.as_deref(), Some("8.0.0.0"));
    assert_eq!(metadata.style.as_deref(), Some("custom-style"));

    let exported = core.export_hml_native().expect("HML should export");
    let xml = std::str::from_utf8(&exported).expect("HML output should be UTF-8");
    assert!(xml.contains("Version=\"2.91\""));
    assert!(xml.contains("SubVersion=\"8.0.0.0\""));
    assert!(xml.contains("Style=\"custom-style\""));
}

#[test]
fn edited_hml_text_is_present_after_export_and_reparse() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture should import");
    let paragraph = &mut core.document_mut().sections[0].paragraphs[0];
    paragraph.text = paragraph.text.replacen("123", "ABC", 1);

    let exported = core.export_hml_native().expect("edited HML should export");
    let reparsed = DocumentCore::from_bytes(&exported).expect("edited HML should reparse");

    assert!(reparsed.document().sections[0].paragraphs[0]
        .text
        .contains("ABC"));
    assert_public_ir_equivalent(&core, &reparsed);
}

#[test]
fn stale_offsets_after_unequal_direct_text_mutation_block_export() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture should import");
    let (section_index, paragraph_index) = core
        .document()
        .sections
        .iter()
        .enumerate()
        .find_map(|(section_index, section)| {
            section
                .paragraphs
                .iter()
                .position(|paragraph| !paragraph.controls.is_empty())
                .map(|paragraph_index| (section_index, paragraph_index))
        })
        .expect("fixture should contain an inline control");
    core.document_mut().sections[section_index].paragraphs[paragraph_index]
        .text
        .push('X');

    let HmlExportError::UnsupportedIr { blockers } = core
        .export_hml_native()
        .expect_err("stale offsets must not move an inline control")
    else {
        panic!("expected typed unsupported-IR refusal");
    };

    assert!(blockers.iter().any(|blocker| {
        blocker.xml_path == format!("/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]")
    }));
}

#[test]
fn legitimate_document_core_text_edit_keeps_control_offsets_savable() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture should import");
    let (section_index, paragraph_index) = core
        .document()
        .sections
        .iter()
        .enumerate()
        .find_map(|(section_index, section)| {
            section
                .paragraphs
                .iter()
                .position(|paragraph| !paragraph.controls.is_empty())
                .map(|paragraph_index| (section_index, paragraph_index))
        })
        .expect("fixture should contain an inline control");

    core.insert_text_native(section_index, paragraph_index, 0, "X")
        .expect("public edit API should update paragraph bookkeeping");
    let exported = core
        .export_hml_native()
        .expect("bookkeeping-preserving edit should remain savable");
    let reparsed = DocumentCore::from_bytes(&exported).expect("edited HML should reparse");
    assert!(
        reparsed.document().sections[section_index].paragraphs[paragraph_index]
            .text
            .starts_with('X')
    );
}

#[test]
fn non_reconstructable_char_shape_runs_are_blocked_recursively() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture should import");
    let mut expected_paths = Vec::new();
    add_redundant_char_shapes_to_fixture(core.document_mut(), &mut expected_paths);

    let HmlExportError::UnsupportedIr { blockers } = core
        .export_hml_native()
        .expect_err("char-shape runs the reader canonicalizes must block export")
    else {
        panic!("expected typed unsupported-IR refusal");
    };
    assert_eq!(expected_paths.len(), 3, "fixture should cover all owners");
    for path in expected_paths {
        assert!(
            blockers.iter().any(|blocker| blocker.xml_path == path),
            "missing char-shape blocker: {path}"
        );
    }
}

fn add_redundant_char_shapes_to_fixture(
    document: &mut rhwp::model::document::Document,
    paths: &mut Vec<String>,
) {
    for (section_index, section) in document.sections.iter_mut().enumerate() {
        for (paragraph_index, paragraph) in section.paragraphs.iter_mut().enumerate() {
            let path = format!("/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]");
            if paths.is_empty() && add_redundant_char_shape(paragraph) {
                paths.push(path.clone());
            }
            add_nested_redundant_char_shapes(paragraph, &path, paths);
        }
    }
}

fn add_nested_redundant_char_shapes(
    paragraph: &mut Paragraph,
    path: &str,
    paths: &mut Vec<String>,
) {
    for (control_index, control) in paragraph.controls.iter_mut().enumerate() {
        let control_path = format!("{path}/CONTROL[{control_index}]");
        match control {
            Control::Table(table) if !paths.iter().any(|path| path.contains("/CELL[")) => {
                let nested = &mut table.cells[0].paragraphs[0];
                if add_redundant_char_shape(nested) {
                    paths.push(format!("{control_path}/TABLE/CELL[0]/P[0]"));
                }
            }
            Control::Shape(shape) if !paths.iter().any(|path| path.contains("/DRAWTEXT/")) => {
                let ShapeObject::Rectangle(rectangle) = shape.as_mut() else {
                    continue;
                };
                let Some(text_box) = rectangle.drawing.text_box.as_mut() else {
                    continue;
                };
                if add_redundant_char_shape(&mut text_box.paragraphs[0]) {
                    paths.push(format!("{control_path}/RECTANGLE/DRAWTEXT/P[0]"));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn stale_offsets_in_cell_and_textbox_paragraphs_are_aggregated() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture should import");
    let mut expected_paths = Vec::new();
    for (section_index, section) in core.document_mut().sections.iter_mut().enumerate() {
        for (paragraph_index, paragraph) in section.paragraphs.iter_mut().enumerate() {
            for (control_index, control) in paragraph.controls.iter_mut().enumerate() {
                match control {
                    Control::Table(table)
                        if expected_paths
                            .iter()
                            .all(|path: &String| !path.contains("/CELL[")) =>
                    {
                        table.cells[0].paragraphs[0].text.push('X');
                        expected_paths.push(format!(
                            "/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]/CONTROL[{control_index}]/TABLE/CELL[0]/P[0]"
                        ));
                    }
                    Control::Shape(shape) => {
                        let ShapeObject::Rectangle(rectangle) = shape.as_mut() else {
                            continue;
                        };
                        if expected_paths
                            .iter()
                            .any(|path| path.contains("/DRAWTEXT/"))
                        {
                            continue;
                        }
                        let Some(text_box) = rectangle.drawing.text_box.as_mut() else {
                            continue;
                        };
                        text_box.paragraphs[0].text.push('X');
                        expected_paths.push(format!(
                            "/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]/CONTROL[{control_index}]/RECTANGLE/DRAWTEXT/P[0]"
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
    assert_eq!(
        expected_paths.len(),
        2,
        "fixture should cover both nested owners"
    );

    let HmlExportError::UnsupportedIr { blockers } = core
        .export_hml_native()
        .expect_err("nested stale offsets must block together")
    else {
        panic!("expected typed unsupported-IR refusal");
    };
    let actual_paths = blockers
        .iter()
        .map(|blocker| blocker.xml_path.clone())
        .collect::<Vec<_>>();
    for path in expected_paths {
        assert!(
            actual_paths.contains(&path),
            "missing nested blocker: {path}"
        );
    }
}

#[test]
fn non_preserved_warning_blocks_hml_export_with_structured_path() {
    let fixture = std::str::from_utf8(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture is UTF-8");
    let bytes = fixture.replacen("Type=\"None\"", "Type=\"Dash\"", 1);
    let core = DocumentCore::from_bytes(bytes.as_bytes())
        .expect("synthetic HML should import with warning");

    let error = core
        .export_hml_native()
        .expect_err("non-preserved warning must block HML export");
    let HmlExportError::LossyImport { blockers } = error else {
        panic!("expected typed lossy-import refusal");
    };

    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].code, "UNSUPPORTED_ATTRIBUTE");
    assert_eq!(
        blockers[0].xml_path,
        "/HWPML/HEAD/MAPPINGTABLE/BORDERFILLLIST/BORDERFILL/LEFTBORDER"
    );
    assert!(!blockers[0].message.is_empty());
}

#[test]
fn public_preflight_matches_every_export_error_variant_and_blocker() {
    let non_hml = DocumentCore::from_bytes(include_bytes!("../samples/re-align-center-hancom.hwp"))
        .expect("HWP fixture should import");

    let fixture = std::str::from_utf8(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture is UTF-8");
    let lossy_bytes = fixture.replacen("Type=\"None\"", "Type=\"Dash\"", 1);
    let lossy =
        DocumentCore::from_bytes(lossy_bytes.as_bytes()).expect("lossy HML should still import");

    let mut unsupported_ir =
        DocumentCore::from_bytes(fixture.as_bytes()).expect("lawful HML should import");
    unsupported_ir.document_mut().sections[0].paragraphs[0].column_type = ColumnBreakType::Section;

    let mut mixed =
        DocumentCore::from_bytes(lossy_bytes.as_bytes()).expect("lossy HML should import");
    mixed.document_mut().sections[0].paragraphs[0].column_type = ColumnBreakType::Section;

    for core in [&non_hml, &lossy, &unsupported_ir, &mixed] {
        let preflight = core
            .hml_export_preflight()
            .expect_err("each case should be blocked");
        let export = core
            .export_hml_native()
            .expect_err("export must return the same refusal");
        assert_eq!(preflight, export);
        assert!(preflight.blockers().iter().all(|blocker| {
            !blocker.code.is_empty() && !blocker.xml_path.is_empty() && !blocker.message.is_empty()
        }));
    }

    assert!(matches!(
        non_hml.hml_export_preflight(),
        Err(HmlExportError::UnsupportedSourceFormat { .. })
    ));
    assert!(matches!(
        lossy.hml_export_preflight(),
        Err(HmlExportError::LossyImport { .. })
    ));
    assert!(matches!(
        unsupported_ir.hml_export_preflight(),
        Err(HmlExportError::UnsupportedIr { .. })
    ));
    assert!(matches!(
        mixed.hml_export_preflight(),
        Err(HmlExportError::LossyImportAndUnsupportedIr { .. })
    ));
}

#[test]
fn import_warnings_and_current_ir_failures_are_reported_together() {
    let fixture = std::str::from_utf8(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture is UTF-8");
    let bytes = fixture.replacen("<CHAR>", "<PICTURE/><CHAR>", 1);
    let mut core = DocumentCore::from_bytes(bytes.as_bytes()).expect("synthetic HML should import");
    core.document_mut().sections[0].paragraphs[0].column_type = ColumnBreakType::Section;

    let HmlExportError::LossyImportAndUnsupportedIr { blockers } = core
        .export_hml_native()
        .expect_err("import and edited-IR loss must be aggregated")
    else {
        panic!("expected typed mixed-loss refusal");
    };
    assert_eq!(blockers.len(), 3);
    assert_eq!(blockers[0].code, "UNSUPPORTED_ELEMENT");
    assert_eq!(blockers[1].code, "HML_UNSUPPORTED_IR");
    assert_eq!(blockers[2].code, "HML_UNSUPPORTED_IR");
    assert_eq!(blockers[0].xml_path, "/HWPML/BODY/SECTION/P/TEXT/PICTURE");
    assert_eq!(blockers[1].xml_path, "/HWPML/BODY/SECTION[0]/P[0]");
    assert_eq!(blockers[2].xml_path, "/HWPML/BODY/SECTION[0]/P[0]");
}

#[test]
fn reader_unsupported_border_value_blocks_instead_of_becoming_solid() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/aligns.hml"))
        .expect("fixture should import");
    core.document_mut().doc_info.border_fills[0].borders[0].line_type = BorderLineType::Dash;

    let error = core
        .export_hml_native()
        .expect_err("a value the HML reader cannot reconstruct must block export");
    let HmlExportError::UnsupportedIr { blockers } = error else {
        panic!("expected typed unsupported-IR refusal");
    };

    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].code, "HML_UNSUPPORTED_IR");
    assert_eq!(
        blockers[0].xml_path,
        "/HWPML/HEAD/MAPPINGTABLE/BORDERFILLLIST/BORDERFILL[0]/LEFTBORDER"
    );
    assert!(!blockers[0].message.is_empty());
}

#[test]
fn section_break_blocks_instead_of_becoming_no_break() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/aligns.hml"))
        .expect("fixture should import");
    core.document_mut().sections[0].paragraphs[0].column_type = ColumnBreakType::Section;

    let error = core
        .export_hml_native()
        .expect_err("a section break the HML reader cannot reconstruct must block export");
    let HmlExportError::UnsupportedIr { blockers } = error else {
        panic!("expected typed unsupported-IR refusal");
    };

    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].code, "HML_UNSUPPORTED_IR");
    assert_eq!(blockers[0].xml_path, "/HWPML/BODY/SECTION[0]/P[0]");
    assert!(!blockers[0].message.is_empty());
}

#[test]
fn multi_column_break_blocks_instead_of_becoming_no_break() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/aligns.hml"))
        .expect("fixture should import");
    core.document_mut().sections[0].paragraphs[0].column_type = ColumnBreakType::MultiColumn;

    let error = core
        .export_hml_native()
        .expect_err("a multi-column break the HML reader cannot reconstruct must block export");
    let HmlExportError::UnsupportedIr { blockers } = error else {
        panic!("expected typed unsupported-IR refusal");
    };

    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].code, "HML_UNSUPPORTED_IR");
    assert_eq!(blockers[0].xml_path, "/HWPML/BODY/SECTION[0]/P[0]");
    assert!(!blockers[0].message.is_empty());
}

#[test]
fn omitted_head_resource_fields_are_aggregated() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/aligns.hml"))
        .expect("fixture should import");
    let info = &mut core.document_mut().doc_info;
    info.char_shapes[0].bold = true;
    info.tab_defs[0].auto_tab_left = true;
    info.para_shapes[0].numbering_id = 1;
    info.border_fills[0].borders[0].color = 0x00ff_0000;

    let HmlExportError::UnsupportedIr { blockers } = core
        .export_hml_native()
        .expect_err("all omitted resource semantics must block together")
    else {
        panic!("expected typed unsupported-IR refusal");
    };

    assert_eq!(blockers.len(), 4);
    assert_eq!(
        blockers[0].xml_path,
        "/HWPML/HEAD/MAPPINGTABLE/CHARSHAPELIST/CHARSHAPE[0]"
    );
    assert_eq!(
        blockers[1].xml_path,
        "/HWPML/HEAD/MAPPINGTABLE/TABDEFLIST/TABDEF[0]"
    );
    assert_eq!(
        blockers[2].xml_path,
        "/HWPML/HEAD/MAPPINGTABLE/PARASHAPELIST/PARASHAPE[0]"
    );
    assert_eq!(
        blockers[3].xml_path,
        "/HWPML/HEAD/MAPPINGTABLE/BORDERFILLLIST/BORDERFILL[0]/LEFTBORDER"
    );
}

#[test]
fn omitted_document_resource_section_and_page_fields_are_aggregated() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/aligns.hml"))
        .expect("fixture should import");
    let document = core.document_mut();
    document.doc_info.font_faces.pop();
    document.doc_info.font_faces[0][0].raw_data = Some(vec![1]);
    document.doc_info.styles[0].raw_data = Some(vec![2]);
    document.doc_info.styles[0].style_type = 2;
    document.sections[0].section_def.flags = 1;
    document.sections[0].section_def.page_def.binding = BindingMethod::DuplexSided;
    document.doc_info.bin_data_list.push(BinData::default());
    document.bin_data_content.push(BinDataContent {
        id: 1,
        data: vec![0xff].into(),
        extension: "bin".to_string(),
    });

    let HmlExportError::UnsupportedIr { blockers } = core
        .export_hml_native()
        .expect_err("all reader-omitted document fields must block together")
    else {
        panic!("expected typed unsupported-IR refusal");
    };

    let paths = blockers
        .iter()
        .map(|blocker| blocker.xml_path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec![
            "/HWPML/HEAD/MAPPINGTABLE/FACENAMELIST",
            "/HWPML/HEAD/MAPPINGTABLE/FACENAMELIST/FONTFACE[0]/FONT[0]",
            "/HWPML/HEAD/MAPPINGTABLE/STYLELIST/STYLE[0]",
            "/HWPML/HEAD/MAPPINGTABLE/BINDATALIST",
            "/HWPML/BINDATA",
            "/HWPML/BODY/SECTION[0]",
            "/HWPML/BODY/SECTION[0]/P[0]/TEXT/SECDEF/PAGEDEF",
        ]
    );
}

#[test]
fn omitted_rectangle_semantics_block_export() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture should import");
    let (expected_path, rectangle) =
        first_rectangle_mut(core.document_mut()).expect("fixture should contain a rectangle");
    rectangle.round_rate = 20;

    let HmlExportError::UnsupportedIr { blockers } = core
        .export_hml_native()
        .expect_err("an omitted rectangle field must block export")
    else {
        panic!("expected typed unsupported-IR refusal");
    };

    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].xml_path, expected_path);
}

fn first_rectangle_mut(
    document: &mut rhwp::model::document::Document,
) -> Option<(String, &mut rhwp::model::shape::RectangleShape)> {
    for (section_index, section) in document.sections.iter_mut().enumerate() {
        for (paragraph_index, paragraph) in section.paragraphs.iter_mut().enumerate() {
            for (control_index, control) in paragraph.controls.iter_mut().enumerate() {
                let Control::Shape(shape) = control else {
                    continue;
                };
                let ShapeObject::Rectangle(rectangle) = shape.as_mut() else {
                    continue;
                };
                let path = format!(
                    "/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]/CONTROL[{control_index}]/RECTANGLE"
                );
                return Some((path, rectangle));
            }
        }
    }
    None
}

fn first_table_mut(
    document: &mut rhwp::model::document::Document,
) -> Option<(String, &mut rhwp::model::table::Table)> {
    for (section_index, section) in document.sections.iter_mut().enumerate() {
        for (paragraph_index, paragraph) in section.paragraphs.iter_mut().enumerate() {
            for (control_index, control) in paragraph.controls.iter_mut().enumerate() {
                let Control::Table(table) = control else {
                    continue;
                };
                let path = format!(
                    "/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]/CONTROL[{control_index}]/TABLE"
                );
                return Some((path, table));
            }
        }
    }
    None
}

#[test]
fn omitted_table_and_cell_semantics_are_aggregated() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture should import");
    let mut expected_path = None;
    'outer: for (section_index, section) in core.document_mut().sections.iter_mut().enumerate() {
        for (paragraph_index, paragraph) in section.paragraphs.iter_mut().enumerate() {
            for (control_index, control) in paragraph.controls.iter_mut().enumerate() {
                if let Control::Table(table) = control {
                    table.page_break = TablePageBreak::None;
                    table.cells[0].text_direction = 1;
                    expected_path = Some(format!(
                        "/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]/CONTROL[{control_index}]/TABLE"
                    ));
                    break 'outer;
                }
            }
        }
    }
    let expected_path = expected_path.expect("fixture should contain a table");

    let HmlExportError::UnsupportedIr { blockers } = core
        .export_hml_native()
        .expect_err("omitted table and cell fields must block together")
    else {
        panic!("expected typed unsupported-IR refusal");
    };

    assert_eq!(blockers.len(), 2);
    assert_eq!(blockers[0].xml_path, expected_path);
    assert_eq!(blockers[1].xml_path, format!("{expected_path}/CELL[0]"));
}

#[test]
fn table_attr_treat_as_char_mirror_is_not_a_false_preflight_blocker() {
    let mut mirrored =
        DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
            .expect("fixture should import");
    let (_, table) = first_table_mut(mirrored.document_mut()).expect("fixture table");
    assert!(table.common.treat_as_char);
    assert_eq!(table.attr, 0x01);
    mirrored
        .export_hml_native()
        .expect("the modeled treat-as-char mirror bit is representable in HML");

    let mut common_only =
        DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
            .expect("fixture should import");
    let (_, table) = first_table_mut(common_only.document_mut()).expect("fixture table");
    table.attr = 0;
    common_only
        .export_hml_native()
        .expect("common.treat_as_char remains the HML source of truth");
}

#[test]
fn contradictory_or_unknown_table_attr_bits_still_block_export() {
    for attr in [0x01, 0x02] {
        let mut core =
            DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
                .expect("fixture should import");
        let (expected_path, table) =
            first_table_mut(core.document_mut()).expect("fixture should contain a table");
        table.attr = attr;
        if attr == 0x01 {
            table.common.treat_as_char = false;
        }

        let HmlExportError::UnsupportedIr { blockers } = core
            .export_hml_native()
            .expect_err("contradictory or unknown table attr bits must block export")
        else {
            panic!("expected typed unsupported-IR refusal");
        };
        assert!(blockers
            .iter()
            .any(|blocker| blocker.xml_path == expected_path));
    }
}

#[test]
fn preserved_head_fragments_keep_their_modeled_sibling_anchor() {
    let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<HWPML Version="2.91" SubVersion="10.0.0.0" Style="embed">
  <HEAD>
    <PICTURE Id="before"><PAYLOAD Value="1"/></PICTURE>
    <MAPPINGTABLE/>
    <EQUATION Id="after"><PAYLOAD Value="2"/></EQUATION>
  </HEAD>
  <BODY><SECTION><P><TEXT CharShape="0"><CHAR>safe</CHAR></TEXT></P></SECTION></BODY>
  <TAIL/>
</HWPML>"#;
    let core = DocumentCore::from_bytes(bytes).expect("synthetic HML should import");
    let exported = core
        .export_hml_native()
        .expect("preserved HEAD fragments should export");
    let xml = std::str::from_utf8(&exported).expect("exported HML must be UTF-8");

    let before = xml
        .find("<PICTURE Id=\"before\">")
        .expect("before fragment");
    let mapping = xml.find("<MAPPINGTABLE>").expect("modeled mapping table");
    let after = xml.find("<EQUATION Id=\"after\">").expect("after fragment");
    assert!(before < mapping && mapping < after);

    let reparsed = DocumentCore::from_bytes(&exported).expect("exported HML should reparse");
    let anchors = reparsed
        .hml_metadata()
        .unwrap()
        .preserved_fragments
        .iter()
        .filter(|fragment| fragment.parent == "HEAD")
        .map(|fragment| fragment.modeled_siblings_before)
        .collect::<Vec<_>>();
    assert_eq!(anchors, vec![0, 1]);
}

#[test]
fn generic_head_body_and_tail_fragments_reinsert_at_modeled_anchors() {
    let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<HWPML Version="2.91">
  <HEAD><HEAD_BEFORE/><MAPPINGTABLE/><HEAD_AFTER/></HEAD>
  <BODY>
    <BODY_BEFORE/>
    <SECTION><P><TEXT CharShape="0"><CHAR>one</CHAR></TEXT></P></SECTION>
    <BODY_MIDDLE/>
    <SECTION><P><TEXT CharShape="0"><CHAR>two</CHAR></TEXT></P></SECTION>
    <BODY_AFTER/>
  </BODY>
  <TAIL><TAIL_ONLY/></TAIL>
</HWPML>"#;
    let core = DocumentCore::from_bytes(bytes).expect("synthetic HML should import");
    let exported = core
        .export_hml_native()
        .expect("preserved fragments should export");
    let xml = std::str::from_utf8(&exported).expect("exported HML must be UTF-8");

    let head_before = xml.find("<HEAD_BEFORE/>").unwrap();
    let mapping = xml.find("<MAPPINGTABLE>").unwrap();
    let head_after = xml.find("<HEAD_AFTER/>").unwrap();
    assert!(head_before < mapping && mapping < head_after);

    let body_before = xml.find("<BODY_BEFORE/>").unwrap();
    let first = xml.find("<SECTION Id=\"0\">").unwrap();
    let middle = xml.find("<BODY_MIDDLE/>").unwrap();
    let second = xml.find("<SECTION Id=\"1\">").unwrap();
    let body_after = xml.find("<BODY_AFTER/>").unwrap();
    assert!(body_before < first && first < middle && middle < second && second < body_after);
    assert!(xml.contains("<TAIL><TAIL_ONLY/></TAIL>"));

    let reparsed = DocumentCore::from_bytes(&exported).expect("exported HML should reparse");
    assert_eq!(reparsed.document().sections.len(), 2);
    assert_eq!(reparsed.hml_metadata().unwrap().warnings.len(), 6);
}

#[test]
fn xml_1_0_illegal_characters_in_emitted_attributes_and_text_are_aggregated() {
    let mut parsed =
        parse_document_with_metadata(include_bytes!("../samples/hml/formatting_table.hml"))
            .expect("fixture should import");
    parsed.document.doc_info.font_faces[0][0].name.push('\u{1}');
    let (section_index, paragraph_index, paragraph) = parsed
        .document
        .sections
        .iter_mut()
        .enumerate()
        .find_map(|(section_index, section)| {
            section
                .paragraphs
                .iter_mut()
                .enumerate()
                .find(|(_, paragraph)| !paragraph.text.is_empty())
                .map(|(paragraph_index, paragraph)| (section_index, paragraph_index, paragraph))
        })
        .expect("fixture should contain text");
    let first_len = paragraph.text.chars().next().unwrap().len_utf8();
    paragraph.text.replace_range(..first_len, "\u{b}");
    let metadata = parsed.hml_metadata.as_mut().expect("HML metadata");
    metadata.style = Some("bad\u{c}style".to_string());

    let HmlExportError::UnsupportedIr { blockers } = serialize_hml(&parsed.document, metadata)
        .expect_err("XML 1.0-illegal scalars must not reach the writer")
    else {
        panic!("expected typed unsupported-IR refusal");
    };
    let paths = blockers
        .iter()
        .map(|blocker| blocker.xml_path.as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"/HWPML"));
    assert!(paths.contains(&"/HWPML/HEAD/MAPPINGTABLE/FACENAMELIST/FONTFACE[0]/FONT[0]"));
    assert!(paths
        .contains(&format!("/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]").as_str()));
    assert!(blockers
        .iter()
        .all(|blocker| blocker.code == "HML_INVALID_XML_CHARACTER"));
}

#[test]
fn invalid_public_preserved_fragments_are_aggregated_before_raw_emission() {
    let mut parsed = parse_document_with_metadata(include_bytes!("../samples/hml/aligns.hml"))
        .expect("fixture should import");
    let metadata = parsed.hml_metadata.as_mut().expect("HML metadata");
    metadata.preserved_fragments.clear();
    let mut add = |parent: &str, anchor: usize, path: &str, raw_xml: String| {
        let order = metadata.preserved_fragments.len();
        metadata.preserved_fragments.push(PreservedFragment {
            parent: parent.to_string(),
            order,
            modeled_siblings_before: anchor,
            xml_path: path.to_string(),
            raw_xml,
        });
    };
    add("OTHER", 0, "/HWPML/OTHER/X", "<X/>".to_string());
    add("HEAD", 0, "/HWPML/HEAD/EXPECTED", "<MISMATCH/>".to_string());
    add("HEAD", 0, "/HWPML/HEAD/TWO", "<TWO/><SECOND/>".to_string());
    add(
        "HEAD",
        0,
        "/HWPML/HEAD/DOCTYPE",
        "<!DOCTYPE DOCTYPE><DOCTYPE/>".to_string(),
    );
    add(
        "HEAD",
        0,
        "/HWPML/HEAD/ENTITY",
        "<ENTITY>&unknown;</ENTITY>".to_string(),
    );
    add(
        "HEAD",
        0,
        "/HWPML/HEAD/NUMERIC",
        "<NUMERIC>&#1;</NUMERIC>".to_string(),
    );
    add(
        "HEAD",
        0,
        "/HWPML/HEAD/DEEP",
        format!("{}{}", "<DEEP>".repeat(257), "</DEEP>".repeat(257)),
    );
    let attributes = (0..257)
        .map(|index| format!(" a{index}=\"x\""))
        .collect::<String>();
    add(
        "HEAD",
        0,
        "/HWPML/HEAD/ATTRS",
        format!("<ATTRS{attributes}/>"),
    );
    add(
        "HEAD",
        0,
        "/HWPML/HEAD/LARGE",
        format!("<LARGE>{}</LARGE>", "x".repeat(8 * 1024 * 1024 + 1)),
    );
    add("BODY", 99, "/HWPML/BODY/LATE", "<LATE/>".to_string());

    let HmlExportError::UnsupportedIr { blockers } = serialize_hml(&parsed.document, metadata)
        .expect_err("untrusted raw fragments must be validated as a batch")
    else {
        panic!("expected typed unsupported-IR refusal");
    };
    assert_eq!(blockers.len(), 10);
    assert!(blockers
        .iter()
        .all(|blocker| blocker.code == "HML_INVALID_RAW_FRAGMENT"));
    let paths = blockers
        .iter()
        .map(|blocker| blocker.xml_path.as_str())
        .collect::<Vec<_>>();
    for expected in [
        "/HWPML/OTHER/X",
        "/HWPML/HEAD/EXPECTED",
        "/HWPML/HEAD/TWO",
        "/HWPML/HEAD/DOCTYPE",
        "/HWPML/HEAD/ENTITY",
        "/HWPML/HEAD/NUMERIC",
        "/HWPML/HEAD/DEEP",
        "/HWPML/HEAD/ATTRS",
        "/HWPML/HEAD/LARGE",
        "/HWPML/BODY/LATE",
    ] {
        assert!(
            paths.contains(&expected),
            "missing raw-fragment blocker: {expected}"
        );
    }
}

#[test]
fn raw_fragment_depth_boundary_matches_secure_reparse() {
    let mut parsed = parse_document_with_metadata(include_bytes!("../samples/hml/aligns.hml"))
        .expect("fixture should import");
    let metadata = parsed.hml_metadata.as_mut().expect("HML metadata");
    metadata.preserved_fragments.clear();
    metadata.preserved_fragments.push(PreservedFragment {
        parent: "HEAD".to_string(),
        order: 0,
        modeled_siblings_before: 0,
        xml_path: "/HWPML/HEAD/X".to_string(),
        raw_xml: format!("<X>{}<EMPTY/>{}</X>", "<A>".repeat(252), "</A>".repeat(252)),
    });

    let bytes = serialize_hml(&parsed.document, metadata)
        .expect("the deepest secure-reader-compatible empty element should serialize");
    DocumentCore::from_bytes(&bytes).expect("serializer depth boundary must securely reparse");

    metadata.preserved_fragments[0].raw_xml =
        format!("<X>{}<EMPTY/>{}</X>", "<A>".repeat(253), "</A>".repeat(253));
    let HmlExportError::UnsupportedIr { blockers } = serialize_hml(&parsed.document, metadata)
        .expect_err("a fragment exceeding the secure reader depth must be rejected")
    else {
        panic!("expected typed unsupported-IR refusal");
    };
    assert_eq!(blockers[0].code, "HML_INVALID_RAW_FRAGMENT");
    assert_eq!(blockers[0].xml_path, "/HWPML/HEAD/X");
}
