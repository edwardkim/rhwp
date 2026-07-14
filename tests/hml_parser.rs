use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{RectangleShape, ShapeObject};
use rhwp::model::style::{border_width_index, BorderLineType, FillType};
use rhwp::parser::hml::{
    parse_hml, parse_hml_with_limits, HmlEncoding, HmlError, HmlLimits, HmlWarningCode,
};
use rhwp::parser::{detect_format, parse_document, FileFormat};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const HML_29: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<HWPML Style="embed" SubVersion="9.0.1.0" Version="2.9">
  <HEAD SecCnt="1" />
  <BODY><SECTION Id="0"><P ParaShape="0" Style="0"><TEXT CharShape="0"><CHAR>안녕 HML 123</CHAR></TEXT></P></SECTION></BODY>
  <TAIL />
</HWPML>"#;

fn utf16le_bom(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xff, 0xfe];
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes
}

fn utf16be_bom(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xfe, 0xff];
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    bytes
}

fn utf8_bom(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xef, 0xbb, 0xbf];
    bytes.extend_from_slice(text.as_bytes());
    bytes
}

fn first_rectangle(document: &rhwp::model::document::Document) -> &RectangleShape {
    document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Rectangle(rectangle) => Some(rectangle),
                _ => None,
            },
            _ => None,
        })
        .expect("fixture should contain a rectangle")
}

fn equations(document: &rhwp::model::document::Document) -> Vec<&rhwp::model::control::Equation> {
    document
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Equation(equation) => Some(equation.as_ref()),
            _ => None,
        })
        .collect()
}

fn find_equation_bbox(node: &RenderNode, para_index: usize) -> Option<BoundingBox> {
    if let RenderNodeType::Equation(equation) = &node.node_type {
        if equation.para_index == Some(para_index) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_equation_bbox(child, para_index))
}

fn find_equation_node(node: &RenderNode) -> Option<&rhwp::renderer::render_tree::EquationNode> {
    if let RenderNodeType::Equation(equation) = &node.node_type {
        return Some(equation);
    }
    node.children.iter().find_map(find_equation_node)
}

fn find_text_bbox(node: &RenderNode, needle: &str) -> Option<BoundingBox> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_text_bbox(child, needle))
}

#[test]
fn parses_repo_equation_fixture_into_shared_ir_without_equation_warnings() {
    let parsed = parse_hml(include_bytes!(
        "fixtures/hml/exambank_math_equations_min.hml"
    ))
    .expect("equation fixture should parse");
    let equations = equations(&parsed.document);

    assert_eq!(
        equations
            .iter()
            .map(|equation| equation.script.as_str())
            .collect::<Vec<_>>(),
        ["x^2 +1", "x^2 +1", "3", "3"]
    );
    assert!(equations.iter().all(|equation| {
        equation.baseline == 65
            && equation.font_size == 1000
            && equation.color == 0
            && equation.version_info == "Equation Version 60"
    }));
    assert!(!parsed
        .warnings
        .iter()
        .any(|warning| warning.xml_path.contains("EQUATION")));

    let paragraph = parsed.document.sections[0]
        .paragraphs
        .iter()
        .find(|paragraph| paragraph.text == "다항식 을 전개하시오.")
        .expect("fixture equation paragraph");
    assert_eq!(
        paragraph.char_offsets,
        [0, 1, 2, 3, 12, 13, 14, 15, 16, 17, 18, 19]
    );
    assert_eq!(paragraph.char_count, 21);
}

#[test]
fn repo_equation_fixture_contract_has_ordered_scripts_and_no_source_identifiers() {
    let fixture = include_str!("fixtures/hml/exambank_math_equations_min.hml");
    let parsed = parse_hml(fixture.as_bytes()).expect("repo equation fixture should parse");

    assert_eq!(
        equations(&parsed.document)
            .iter()
            .map(|equation| equation.script.as_str())
            .collect::<Vec<_>>(),
        ["x^2 +1", "x^2 +1", "3", "3"]
    );
    for disallowed in [
        "ExamBank",
        "exambank",
        "serial_curated",
        "http://",
        "https://",
        "/Users/",
        "\\Users\\",
        "@",
    ] {
        assert!(
            !fixture.contains(disallowed),
            "repo fixture must not retain source identifier {disallowed:?}"
        );
    }
}

#[test]
fn imported_inline_equation_has_intrinsic_bbox_between_text_and_is_hittable() {
    let core = DocumentCore::from_bytes(include_bytes!(
        "fixtures/hml/exambank_math_equations_min.hml"
    ))
    .expect("equation fixture should open");
    let equation = equations(core.document())[0];
    assert!(equation.common.width > 0 && equation.common.height > 0);

    let tree = core.build_page_render_tree(0).expect("page render tree");
    let before = find_text_bbox(&tree.root, "다항식 ").expect("text before equation");
    let equation = find_equation_bbox(&tree.root, 2).expect("inline equation bbox");
    let after = find_text_bbox(&tree.root, "을 전개하시오.").expect("text after equation");
    assert!(
        equation.width > 0.0 && equation.height > 0.0,
        "{equation:?}"
    );
    assert!(
        before.x + before.width <= equation.x + 0.5,
        "{before:?} vs {equation:?}"
    );
    assert!(
        equation.x + equation.width <= after.x + 0.5,
        "{equation:?} vs {after:?}"
    );

    let hit = core
        .hit_test_native(
            0,
            equation.x + equation.width / 2.0,
            equation.y + equation.height / 2.0,
        )
        .expect("equation center should be hittable");
    assert!(hit.contains("\"paragraphIndex\":2"), "{hit}");
}

#[test]
fn equation_supports_font_color_entities_and_cdata() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT CharShape="0"><EQUATION BaseLine="-4" BaseUnit="1200" TextColor="1122867" Version="v&amp;1" Font="Hancom"><SCRIPT><![CDATA[a < b]]>&amp;c</SCRIPT></EQUATION></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let parsed = parse_hml(xml).expect("equation entities and CDATA should parse");
    let equation = equations(&parsed.document)[0];

    assert_eq!(equation.script, "a < b&c");
    assert_eq!(equation.baseline, -4);
    assert_eq!(equation.font_size, 1200);
    assert_eq!(equation.color, 0x0011_2233);
    assert_eq!(equation.version_info, "v&1");
    assert_eq!(equation.font_name, "Hancom");

    let core = DocumentCore::from_bytes(xml).expect("equation should open for rendering");
    let tree = core.build_page_render_tree(0).expect("page render tree");
    let rendered = find_equation_node(&tree.root).expect("rendered equation");
    assert_eq!(rendered.color, 0x0011_2233);
    assert_eq!(rendered.color_str, "#332211");
    assert!(rendered.font_size > 0.0);
}

#[test]
fn unknown_equation_semantics_emit_durable_exact_path_warnings() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><EQUATION FutureAttr="1"><SCRIPT>x</SCRIPT>outside &amp; text<FUTURE Mode="matrix&amp;inline">secret &lt; value</FUTURE></EQUATION></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let parsed = parse_hml(xml).expect("unknown equation semantics should not block import");

    assert_eq!(equations(&parsed.document).len(), 1);
    for path in [
        "/HWPML/BODY/SECTION/P/TEXT/EQUATION/@FutureAttr",
        "/HWPML/BODY/SECTION/P/TEXT/EQUATION/FUTURE",
        "/HWPML/BODY/SECTION/P/TEXT/EQUATION/FUTURE/@Mode",
        "/HWPML/BODY/SECTION/P/TEXT/EQUATION/FUTURE/#text",
        "/HWPML/BODY/SECTION/P/TEXT/EQUATION/#text",
    ] {
        assert!(
            parsed.warnings.iter().any(|warning| {
                warning.code == HmlWarningCode::UnsupportedEquationSemantics
                    && warning.xml_path == path
                    && !warning.preserved
            }),
            "missing durable warning for {path}"
        );
    }
    let diagnostic_messages = parsed
        .warnings
        .iter()
        .map(|warning| warning.message.as_str())
        .collect::<Vec<_>>();
    assert!(diagnostic_messages
        .iter()
        .any(|message| message.contains("Mode=matrix&inline")));
    assert!(diagnostic_messages
        .iter()
        .any(|message| message.contains("#text=secret < value")));
    assert!(diagnostic_messages
        .iter()
        .any(|message| message.contains("#text=outside & text")));
}

#[test]
fn unknown_equation_diagnostic_values_are_bounded_on_unicode_boundaries() {
    let long_value = "한".repeat(400);
    let xml = format!(
        r#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><EQUATION><SCRIPT>x</SCRIPT><FUTURE Value="{long_value}"/></EQUATION></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#
    );
    let parsed = parse_hml(xml.as_bytes()).expect("long diagnostic value remains importable");
    let warning = parsed
        .warnings
        .iter()
        .find(|warning| warning.xml_path.ends_with("/FUTURE/@Value"))
        .expect("unknown child attribute warning");
    let semantics = warning
        .message
        .split_once(": ")
        .map(|(_, value)| value)
        .expect("diagnostic message prefix");

    assert_eq!(semantics.chars().count(), 256);
    assert!(semantics.ends_with('…'));
}

#[test]
fn equation_accepts_only_the_first_direct_script_and_warns_for_duplicates_and_nested_scripts() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><EQUATION><SCRIPT FutureMode="matrix&amp;inline">first</SCRIPT><SCRIPT>second</SCRIPT><SCRIPT>outer<SCRIPT>nested</SCRIPT></SCRIPT></EQUATION></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let parsed = parse_hml(xml).expect("duplicate scripts should remain importable");

    assert_eq!(equations(&parsed.document)[0].script, "first");
    for path in [
        "/HWPML/BODY/SECTION/P/TEXT/EQUATION/SCRIPT/@FutureMode",
        "/HWPML/BODY/SECTION/P/TEXT/EQUATION/SCRIPT[2]",
        "/HWPML/BODY/SECTION/P/TEXT/EQUATION/SCRIPT[3]",
        "/HWPML/BODY/SECTION/P/TEXT/EQUATION/SCRIPT/SCRIPT",
    ] {
        assert!(
            parsed.warnings.iter().any(|warning| {
                warning.code == HmlWarningCode::UnsupportedEquationSemantics
                    && warning.xml_path == path
                    && !warning.preserved
            }),
            "missing duplicate/nested SCRIPT warning for {path}"
        );
    }
    let script_attribute = parsed
        .warnings
        .iter()
        .find(|warning| warning.xml_path.ends_with("/SCRIPT/@FutureMode"))
        .expect("SCRIPT attribute warning");
    assert!(
        script_attribute
            .message
            .contains("FutureMode=matrix&inline"),
        "unsupported SCRIPT attribute diagnostics must retain name and value: {}",
        script_attribute.message
    );
}

#[test]
fn rejects_unclosed_equation_script_as_invalid_xml() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><EQUATION><SCRIPT>x</EQUATION></TEXT></P></SECTION></BODY></HWPML>"#;

    assert!(matches!(parse_hml(xml), Err(HmlError::InvalidXml(_))));
}

#[test]
fn detects_utf16le_hwpml_29_by_root_signature() {
    assert_eq!(detect_format(&utf16le_bom(HML_29)), FileFormat::Hml);
}

#[test]
fn detects_utf16be_hwpml_29_by_root_signature() {
    assert_eq!(detect_format(&utf16be_bom(HML_29)), FileFormat::Hml);
}

#[test]
fn detects_utf8_bom_hwpml_29_by_root_signature() {
    assert_eq!(detect_format(&utf8_bom(HML_29)), FileFormat::Hml);
}

#[test]
fn does_not_detect_ordinary_xml_or_html_as_hml() {
    let samples: [&[u8]; 2] = [
        br#"<?xml version="1.0"?><catalog><item>HWPML</item></catalog>"#,
        br#"<?xml version="1.0"?><html><body>server error</body></html>"#,
    ];

    for sample in samples {
        assert_ne!(detect_format(sample), FileFormat::Hml);
    }
}

#[test]
fn does_not_detect_hwpml_named_xml_without_a_version_signature() {
    assert_ne!(
        detect_format(br#"<?xml version="1.0"?><HWPML/>"#),
        FileFormat::Hml
    );
}

#[test]
fn inline_controls_preserve_text_offsets_and_unsupported_paths() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT CharShape="0"><CHAR>a</CHAR><EQUATION/><CHAR>b</CHAR><PICTURE/></TEXT></P></SECTION></BODY><TAIL><BINDATA/></TAIL></HWPML>"#;
    let parsed = parse_hml(xml).expect("unsupported controls should not abort readable text");
    let paragraph = &parsed.document.sections[0].paragraphs[0];

    assert_eq!(paragraph.text, "ab");
    assert_eq!(paragraph.char_offsets, [0, 9]);
    assert!(matches!(paragraph.controls[0], Control::Equation(_)));
    assert_eq!(parsed.metadata.resource_count, 1);
    for path in ["/HWPML/BODY/SECTION/P/TEXT/PICTURE", "/HWPML/TAIL/BINDATA"] {
        assert!(
            parsed.warnings.iter().any(|warning| {
                warning.code == HmlWarningCode::UnsupportedElement && warning.xml_path == path
            }),
            "missing structured warning for {path}"
        );
    }
}

#[test]
fn does_not_detect_malformed_utf8_as_hml() {
    let mut bytes = HML_29.as_bytes().to_vec();
    bytes.push(0xff);

    assert_ne!(detect_format(&bytes), FileFormat::Hml);
}

#[test]
fn detects_real_hwpml_291_fixture_by_root_signature() {
    let bytes = std::fs::read("samples/hml/aligns.hml").expect("real HML fixture should exist");

    assert_eq!(detect_format(&bytes), FileFormat::Hml);
}

#[test]
fn rejects_hml_with_doctype() {
    let xml = br#"<?xml version="1.0"?>
<!DOCTYPE HWPML [<!ENTITY secret "expanded">]>
<HWPML Style="embed" SubVersion="9.0.1.0" Version="2.9">
  <HEAD SecCnt="1"/><BODY><SECTION Id="0"/></BODY><TAIL/>
</HWPML>"#;

    assert!(matches!(parse_hml(xml), Err(HmlError::InvalidXml(_))));
}

#[test]
fn rejects_malformed_hml_xml() {
    let xml = br#"<HWPML Version="2.9"><HEAD/><BODY><SECTION><P></SECTION></BODY></HWPML>"#;

    assert!(matches!(parse_hml(xml), Err(HmlError::InvalidXml(_))));
}

#[test]
fn parses_real_hwpml_291_alignment_fixture_into_shared_ir() {
    let bytes = std::fs::read("samples/hml/aligns.hml").expect("real HML fixture should exist");
    let parsed = parse_hml(&bytes).expect("HWPML 2.91 fixture should parse");

    assert_eq!(parsed.metadata.hwpml_version.as_deref(), Some("2.91"));
    assert_eq!(parsed.metadata.encoding, HmlEncoding::Utf8);
    assert_eq!(parsed.metadata.resource_count, 0);
    assert_eq!(parsed.document.sections.len(), 1);
    assert_eq!(parsed.document.sections[0].paragraphs.len(), 17);
    assert_eq!(parsed.document.doc_info.font_faces.len(), 7);
    assert!(parsed
        .document
        .doc_info
        .font_faces
        .iter()
        .all(|fonts| fonts.len() == 2));
    assert_eq!(parsed.document.doc_info.char_shapes.len(), 5);
    assert_eq!(parsed.document.doc_info.para_shapes.len(), 12);
    assert_eq!(parsed.document.doc_info.styles.len(), 14);
    assert_eq!(
        parsed.document.sections[0]
            .paragraphs
            .iter()
            .filter(|paragraph| {
                paragraph.column_type == rhwp::model::paragraph::ColumnBreakType::Page
            })
            .count(),
        15,
        "fixture has one page-break paragraph before each page after the first"
    );

    let shape_texts: Vec<&str> = parsed.document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .filter_map(|control| match control {
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Rectangle(rectangle) => rectangle.drawing.text_box.as_ref(),
                _ => None,
            },
            _ => None,
        })
        .flat_map(|text_box| text_box.paragraphs.iter())
        .map(|paragraph| paragraph.text.as_str())
        .collect();
    assert_eq!(
        shape_texts,
        [
            "left 0",
            "left 10",
            "center 0",
            "center -10",
            "right 0",
            "right 10",
            "inside 0",
            "inside 0",
            "outside 0",
            "outside 10",
            "top 0",
            "top 10",
            "middle 0",
            "middle -10",
            "bottom 0",
            "bottom 10",
        ]
    );
    let rectangles: Vec<_> = parsed.document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .filter_map(|control| match control {
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Rectangle(rectangle) => Some(rectangle),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(
        rectangles[0].common.horz_rel_to,
        rhwp::model::shape::HorzRelTo::Page
    );
    assert_eq!(
        rectangles[0].common.vert_rel_to,
        rhwp::model::shape::VertRelTo::Page
    );
    assert_eq!(
        rectangles[2].common.horz_align,
        rhwp::model::shape::HorzAlign::Center
    );
    assert_eq!(rectangles[3].common.horizontal_offset as i32, -2835);
    assert_eq!(
        rectangles[13].common.vert_align,
        rhwp::model::shape::VertAlign::Center
    );
    assert_eq!(rectangles[13].common.vertical_offset as i32, -2835);
    assert_eq!(
        rectangles[14].common.vert_align,
        rhwp::model::shape::VertAlign::Bottom
    );
    assert_eq!(
        rectangles[0].common.text_wrap,
        rhwp::model::shape::TextWrap::InFrontOfText
    );
    assert_eq!(rectangles[0].drawing.shape_attr.offset_x, 0);
    assert_eq!(rectangles[0].drawing.shape_attr.offset_y, 0);
    assert_eq!(rectangles[0].drawing.shape_attr.current_width, 8504);
    assert_eq!(rectangles[0].drawing.shape_attr.current_height, 5669);
    assert_eq!(rectangles[0].drawing.shape_attr.original_width, 11235);
    assert_eq!(rectangles[0].drawing.shape_attr.original_height, 4345);
    assert!(parsed.warnings.iter().any(|warning| {
        warning.code == HmlWarningCode::UnsupportedElement
            && warning.xml_path == "/HWPML/TAIL/SCRIPTCODE"
    }));
}

#[test]
fn preserves_tail_scriptcode_subtree_byte_verbatim_and_flags_warning_preserved() {
    let bytes = std::fs::read("samples/hml/aligns.hml").expect("real HML fixture should exist");

    // Independently derive the expected SCRIPTCODE span without touching the parser's
    // own decode path, so this assertion is not tautological with the implementation.
    let bom = [0xef, 0xbb, 0xbf];
    let without_bom = if bytes.starts_with(&bom) {
        &bytes[bom.len()..]
    } else {
        &bytes[..]
    };
    let text = std::str::from_utf8(without_bom).expect("fixture should be valid UTF-8");
    let start = text
        .find("<SCRIPTCODE")
        .expect("fixture should contain SCRIPTCODE");
    let end_tag = "</SCRIPTCODE>";
    let end = text[start..]
        .find(end_tag)
        .map(|offset| start + offset + end_tag.len())
        .expect("fixture should close SCRIPTCODE");
    let expected = &text[start..end];

    let parsed = parse_hml(&bytes).expect("HWPML 2.91 fixture should parse");

    let warning = parsed
        .warnings
        .iter()
        .find(|warning| warning.xml_path == "/HWPML/TAIL/SCRIPTCODE")
        .expect("SCRIPTCODE warning should be emitted");
    assert!(
        warning.preserved,
        "TAIL-parented skip should be envelope-preserved"
    );

    let fragment = parsed
        .preserved_fragments
        .iter()
        .find(|fragment| fragment.xml_path == "/HWPML/TAIL/SCRIPTCODE")
        .expect("SCRIPTCODE subtree should be captured verbatim");
    assert_eq!(fragment.raw_xml, expected);
    assert_eq!(fragment.parent, "TAIL");
}

#[test]
fn body_inline_unsupported_elements_are_not_envelope_preserved() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT CharShape="0"><CHAR>a</CHAR><EQUATION/><CHAR>b</CHAR><PICTURE/></TEXT></P></SECTION></BODY><TAIL><BINDATA/></TAIL></HWPML>"#;
    let parsed = parse_hml(xml).expect("unsupported controls should not abort readable text");

    assert!(!parsed
        .warnings
        .iter()
        .any(|warning| warning.xml_path.ends_with("/EQUATION")));
    let path = "/HWPML/BODY/SECTION/P/TEXT/PICTURE";
    let warning = parsed
        .warnings
        .iter()
        .find(|warning| warning.xml_path == path)
        .unwrap_or_else(|| panic!("missing structured warning for {path}"));
    assert!(
        !warning.preserved,
        "body-inline skip must not be envelope-preserved: {path}"
    );
    assert!(
        !parsed
            .preserved_fragments
            .iter()
            .any(|fragment| fragment.xml_path == path),
        "body-inline skip must not produce a preserved fragment: {path}"
    );

    let tail_warning = parsed
        .warnings
        .iter()
        .find(|warning| warning.xml_path == "/HWPML/TAIL/BINDATA")
        .expect("missing structured warning for /HWPML/TAIL/BINDATA");
    assert!(
        tail_warning.preserved,
        "TAIL-parented self-closing skip should be envelope-preserved"
    );
    let fragment = parsed
        .preserved_fragments
        .iter()
        .find(|fragment| fragment.xml_path == "/HWPML/TAIL/BINDATA")
        .expect("BINDATA subtree should be captured verbatim");
    assert_eq!(fragment.raw_xml, "<BINDATA/>");
}

#[test]
fn generic_document_children_are_warned_preserved_and_anchored() {
    let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<HWPML Version="2.91">
  <HEAD>
    <UNKNOWN_HEAD_BEFORE><SECTION/></UNKNOWN_HEAD_BEFORE>
    <DOCSETTING><BEGINNUMBER Page="1"/></DOCSETTING>
    <MAPPINGTABLE/>
    <UNKNOWN_HEAD_AFTER/>
  </HEAD>
  <BODY>
    <UNKNOWN_BODY_BEFORE><P/></UNKNOWN_BODY_BEFORE>
    <SECTION><P><TEXT CharShape="0"><CHAR>one</CHAR></TEXT></P></SECTION>
    <UNKNOWN_BODY_MIDDLE/>
    <SECTION><P><TEXT CharShape="0"><CHAR>two</CHAR></TEXT></P></SECTION>
    <UNKNOWN_BODY_AFTER/>
  </BODY>
  <TAIL><UNKNOWN_TAIL><P/></UNKNOWN_TAIL></TAIL>
</HWPML>"#;

    let result = rhwp::parser::hml::parse_hml(bytes).expect("synthetic HML should parse");
    assert_eq!(
        result.document.sections.len(),
        2,
        "captured descendants stay opaque"
    );
    assert_eq!(result.warnings.len(), 6);
    assert!(result.warnings.iter().all(|warning| warning.preserved));
    let placements = result
        .preserved_fragments
        .iter()
        .map(|fragment| {
            (
                fragment.parent.as_str(),
                fragment.modeled_siblings_before,
                fragment.xml_path.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        placements,
        vec![
            ("HEAD", 0, "/HWPML/HEAD/UNKNOWN_HEAD_BEFORE"),
            ("HEAD", 1, "/HWPML/HEAD/UNKNOWN_HEAD_AFTER"),
            ("BODY", 0, "/HWPML/BODY/UNKNOWN_BODY_BEFORE"),
            ("BODY", 1, "/HWPML/BODY/UNKNOWN_BODY_MIDDLE"),
            ("BODY", 2, "/HWPML/BODY/UNKNOWN_BODY_AFTER"),
            ("TAIL", 0, "/HWPML/TAIL/UNKNOWN_TAIL"),
        ]
    );
    assert!(result
        .preserved_fragments
        .iter()
        .all(|fragment| !fragment.raw_xml.contains("DOCSETTING")));
}

#[test]
fn maps_real_rectangle_line_shape_into_shared_ir() {
    let bytes = std::fs::read("samples/hml/aligns.hml").expect("real HML fixture should exist");
    let parsed = parse_hml(&bytes).expect("HWPML 2.91 fixture should parse");
    let line = &first_rectangle(&parsed.document).drawing.border_line;

    assert_eq!(line.width, 33);
    assert_eq!(line.attr & 0x3f, 1, "Style=Solid");
    assert_eq!((line.attr >> 6) & 0x0f, 1, "EndCap=Flat");
}

#[test]
fn maps_real_rectangle_text_margin_into_shared_ir() {
    let bytes = std::fs::read("samples/hml/aligns.hml").expect("real HML fixture should exist");
    let parsed = parse_hml(&bytes).expect("HWPML 2.91 fixture should parse");
    let text_box = first_rectangle(&parsed.document)
        .drawing
        .text_box
        .as_ref()
        .expect("fixture rectangle should have a text box");

    assert_eq!(text_box.margin_left, 283);
    assert_eq!(text_box.margin_right, 283);
    assert_eq!(text_box.margin_top, 283);
    assert_eq!(text_box.margin_bottom, 283);
}

#[test]
fn maps_real_rectangle_window_brush_into_shared_ir() {
    let bytes =
        std::fs::read("samples/hml/formatting_table.hml").expect("real HML fixture should exist");
    let parsed = parse_hml(&bytes).expect("HWPML 2.91 fixture should parse");
    let fill = &first_rectangle(&parsed.document).drawing.fill;

    assert_eq!(fill.fill_type, FillType::Solid);
    let solid = fill.solid.expect("WINDOWBRUSH should create a solid fill");
    assert_eq!(solid.background_color, 16_777_215);
    assert_eq!(solid.pattern_color, 0);
    assert_eq!(solid.pattern_type, -1);
    assert_eq!(fill.alpha, 0);
}

#[test]
fn maps_real_hwpml_291_formatting_table_fixture_without_losing_inline_order() {
    let bytes =
        std::fs::read("samples/hml/formatting_table.hml").expect("real HML fixture should exist");
    let parsed = parse_hml(&bytes).expect("formatting/table fixture should parse");
    let paragraphs = &parsed.document.sections[0].paragraphs;

    assert_eq!(paragraphs.len(), 2);
    assert_eq!(paragraphs[0].text, "123456");
    assert_eq!(paragraphs[0].char_offsets, [0, 1, 2, 11, 12, 13]);
    assert_eq!(paragraphs[1].text, "abcefg");
    assert_eq!(paragraphs[1].char_offsets, [0, 1, 2, 11, 12, 13]);
    assert_eq!(parsed.document.doc_info.char_shapes[5].base_size, 1600);
    assert_eq!(
        parsed.document.doc_info.para_shapes[16].alignment,
        rhwp::model::style::Alignment::Left
    );
    assert_eq!(parsed.document.doc_info.styles[17].local_name, "차례 3");

    let Control::Shape(shape) = &paragraphs[0].controls[0] else {
        panic!("first inline control should be a shape");
    };
    let ShapeObject::Rectangle(rectangle) = shape.as_ref() else {
        panic!("fixture shape should be a rectangle");
    };
    assert_eq!(
        rectangle.drawing.text_box.as_ref().unwrap().paragraphs[0].text,
        "textbox"
    );

    let Control::Table(table) = &paragraphs[1].controls[0] else {
        panic!("second inline control should be a table");
    };
    assert_eq!((table.row_count, table.col_count), (1, 1));
    assert_eq!((table.common.width, table.common.height), (41956, 1282));
    assert!(table.common.treat_as_char);
    assert_eq!(table.attr & 0x01, 0x01);
    assert!(table.common.flow_with_text);
    assert!(!table.common.allow_overlap);
    assert_eq!(
        table.common.horz_rel_to,
        rhwp::model::shape::HorzRelTo::Para
    );
    assert_eq!(table.common.horz_align, rhwp::model::shape::HorzAlign::Left);
    assert_eq!(
        table.common.vert_rel_to,
        rhwp::model::shape::VertRelTo::Para
    );
    assert_eq!(table.common.vert_align, rhwp::model::shape::VertAlign::Top);
    assert_eq!(table.cells.len(), 1);
    assert_eq!(table.cells[0].paragraphs[0].text, "table");
    assert_eq!(table.cell_grid, [Some(0)]);
}

#[test]
fn maps_real_border_fill_edges_into_shared_ir() {
    let aligns = std::fs::read("samples/hml/aligns.hml").expect("real HML fixture should exist");
    let aligns = parse_hml(&aligns).expect("alignment fixture should parse");
    let paragraph_border_id = aligns
        .document
        .doc_info
        .para_shapes
        .iter()
        .find_map(|shape| (shape.border_fill_id == 2).then_some(shape.border_fill_id))
        .expect("fixture paragraph shapes should reference border fill 2");
    let paragraph_border =
        &aligns.document.doc_info.border_fills[usize::from(paragraph_border_id - 1)];

    assert!(paragraph_border
        .borders
        .iter()
        .all(|border| border.line_type == BorderLineType::None));
    assert!(paragraph_border
        .borders
        .iter()
        .all(|border| border.width == border_width_index(0.1)));

    let formatting =
        std::fs::read("samples/hml/formatting_table.hml").expect("real HML fixture should exist");
    let formatting = parse_hml(&formatting).expect("formatting/table fixture should parse");
    let Control::Table(table) = &formatting.document.sections[0].paragraphs[1].controls[0] else {
        panic!("second inline control should be a table");
    };
    assert_eq!(table.border_fill_id, 3);
    assert_eq!(table.cells[0].border_fill_id, 3);
    let table_border = &formatting.document.doc_info.border_fills[2];

    assert!(table_border
        .borders
        .iter()
        .all(|border| border.line_type == BorderLineType::Solid));
    assert!(table_border
        .borders
        .iter()
        .all(|border| border.width == border_width_index(0.12)));
}

#[test]
fn nested_table_layout_does_not_overwrite_enclosing_rectangle() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><RECTANGLE>
        <SHAPEOBJECT><SIZE Width="1000" Height="500"/><POSITION TreatAsChar="true" FlowWithText="false" AllowOverlap="true" HorzOffset="11" VertOffset="22" HorzRelTo="Page" VertRelTo="Page" HorzAlign="Right" VertAlign="Bottom"/></SHAPEOBJECT>
        <DRAWINGOBJECT><DRAWTEXT><PARALIST><P><TEXT><TABLE RowCount="1" ColCount="1">
          <SHAPEOBJECT><SIZE Width="300" Height="200"/><POSITION TreatAsChar="false" FlowWithText="true" AllowOverlap="false" HorzOffset="-33" VertOffset="-44" HorzRelTo="Para" VertRelTo="Para" HorzAlign="Left" VertAlign="Top"/></SHAPEOBJECT>
          <ROW><CELL ColAddr="0" RowAddr="0"><PARALIST><P><TEXT><CHAR>cell</CHAR></TEXT></P></PARALIST></CELL></ROW>
        </TABLE></TEXT></P></PARALIST></DRAWTEXT></DRAWINGOBJECT>
      </RECTANGLE></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let parsed = parse_hml(xml).expect("nested table HML should parse");
    let rectangle = first_rectangle(&parsed.document);

    assert_eq!(
        (rectangle.common.width, rectangle.common.height),
        (1000, 500)
    );
    assert_eq!(rectangle.common.horizontal_offset as i32, 11);
    assert_eq!(rectangle.common.vertical_offset as i32, 22);
    assert!(rectangle.common.treat_as_char);
    assert!(rectangle.common.allow_overlap);

    let text_box = rectangle
        .drawing
        .text_box
        .as_ref()
        .expect("rectangle should contain a text box");
    let Control::Table(table) = &text_box.paragraphs[0].controls[0] else {
        panic!("text box should contain the nested table");
    };
    assert_eq!((table.common.width, table.common.height), (300, 200));
    assert_eq!(table.common.horizontal_offset as i32, -33);
    assert_eq!(table.common.vertical_offset as i32, -44);
    assert!(!table.common.treat_as_char);
    assert!(table.common.flow_with_text);
    assert!(!table.common.allow_overlap);
}

#[test]
fn missing_shape_current_size_materializes_from_original_size() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><RECTANGLE>
        <SHAPEOBJECT><SIZE Width="600" Height="400"/></SHAPEOBJECT>
        <DRAWINGOBJECT><SHAPECOMPONENT XPos="-12" YPos="-34" OriWidth="600" OriHeight="400"/></DRAWINGOBJECT>
      </RECTANGLE></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let parsed = parse_hml(xml).expect("shape with original size should parse");
    let shape_attr = &first_rectangle(&parsed.document).drawing.shape_attr;

    assert_eq!((shape_attr.offset_x, shape_attr.offset_y), (-12, -34));
    assert_eq!(
        (shape_attr.original_width, shape_attr.original_height),
        (600, 400)
    );
    assert_eq!(
        (shape_attr.current_width, shape_attr.current_height),
        (600, 400)
    );
}

#[test]
fn enforces_configured_xml_size_limit() {
    let limits = HmlLimits {
        max_xml_bytes: HML_29.len() - 1,
        ..HmlLimits::default()
    };

    assert!(matches!(
        parse_hml_with_limits(HML_29.as_bytes(), &limits),
        Err(HmlError::LimitExceeded(_))
    ));
}

#[test]
fn enforces_configured_xml_depth_limit() {
    let xml = br#"<HWPML Version="2.9"><HEAD><A><B></B></A></HEAD><BODY/></HWPML>"#;
    let limits = HmlLimits {
        max_depth: 3,
        ..HmlLimits::default()
    };

    assert!(matches!(
        parse_hml_with_limits(xml, &limits),
        Err(HmlError::LimitExceeded(_))
    ));
}

#[test]
fn counts_self_closing_elements_toward_depth_limit() {
    let xml = br#"<HWPML Version="2.9"><HEAD/><BODY/></HWPML>"#;
    let limits = HmlLimits {
        max_depth: 1,
        ..HmlLimits::default()
    };

    assert!(matches!(
        parse_hml_with_limits(xml, &limits),
        Err(HmlError::LimitExceeded(_))
    ));
}

#[test]
fn maps_minimal_hwpml_body_text_into_document_ir() {
    let parsed = parse_hml(HML_29.as_bytes()).expect("minimal HWPML should parse");
    let paragraph = &parsed.document.sections[0].paragraphs[0];

    assert_eq!(parsed.metadata.hwpml_version.as_deref(), Some("2.9"));
    assert_eq!(parsed.metadata.encoding, HmlEncoding::Utf8);
    assert_eq!(parsed.document.sections.len(), 1);
    assert_eq!(paragraph.text, "안녕 HML 123");
    assert_eq!(paragraph.char_shapes[0].char_shape_id, 0);
    assert_eq!(parsed.document.doc_info.char_shapes[0].base_size, 1000);
}

#[test]
fn parse_document_dispatches_hml_into_document_ir() {
    let document = parse_document(HML_29.as_bytes()).expect("HML dispatch should parse");

    assert_eq!(document.sections[0].paragraphs[0].text, "안녕 HML 123");
}
