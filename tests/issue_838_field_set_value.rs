//! Issue #838: ClickHere 필드 값 설정 시 paragraph metadata 보존
//!
//! 재현: field-01.hwp의 빈 ClickHere range에 값을 삽입한 뒤 HWP로 직렬화하면
//! `char_count`, `char_offsets`, `field_ranges`가 함께 갱신되어야 한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::queries::field_query::{FieldInfo, FieldLocation, NestedEntry};
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FieldType};
use rhwp::model::paragraph::Paragraph;

fn load_sample(relative: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {e:?}", path.display()))
}

fn field_named<'a>(core: &'a DocumentCore, name: &str) -> FieldInfo {
    core.collect_all_fields()
        .into_iter()
        .find(|field| field.field.field_name() == Some(name))
        .unwrap_or_else(|| panic!("field {name:?} should exist"))
}

fn paragraph_at_location<'a>(core: &'a DocumentCore, location: &FieldLocation) -> &'a Paragraph {
    let mut paragraph =
        &core.document().sections[location.section_index].paragraphs[location.para_index];
    for entry in &location.nested_path {
        paragraph = match entry {
            NestedEntry::TableCell {
                control_index,
                cell_index,
                para_index,
            } => {
                let Control::Table(table) = &paragraph.controls[*control_index] else {
                    panic!("field path should point at a table")
                };
                &table.cells[*cell_index].paragraphs[*para_index]
            }
            NestedEntry::TextBox {
                control_index,
                para_index,
            } => {
                let Control::Shape(shape) = &paragraph.controls[*control_index] else {
                    panic!("field path should point at a shape textbox")
                };
                &shape
                    .drawing()
                    .and_then(|drawing| drawing.text_box.as_ref())
                    .expect("field shape should own a textbox")
                    .paragraphs[*para_index]
            }
        };
    }
    paragraph
}

fn char_shape_signature(paragraph: &Paragraph) -> Vec<(u32, u32)> {
    paragraph
        .char_shapes
        .iter()
        .map(|shape| (shape.start_pos, shape.char_shape_id))
        .collect()
}

fn line_position_signature(paragraph: &Paragraph) -> Vec<(u32, i32)> {
    paragraph
        .line_segs
        .iter()
        .map(|line| (line.text_start, line.vertical_pos))
        .collect()
}

fn assert_field_layout_roundtrip(
    before_save: &DocumentCore,
    after_load: &DocumentCore,
    name: &str,
) {
    let before_field = field_named(before_save, name);
    let after_field = field_named(after_load, name);
    assert_eq!(
        after_field.value, before_field.value,
        "field value roundtrip"
    );

    let before = paragraph_at_location(before_save, &before_field.location);
    let after = paragraph_at_location(after_load, &after_field.location);
    assert!(
        !before.line_segs.is_empty(),
        "field edit should materialize owner LineSeg before serialization"
    );
    assert_eq!(
        line_position_signature(after),
        line_position_signature(before),
        "field owner LineSeg count/start/vpos should survive roundtrip"
    );
    assert_eq!(
        char_shape_signature(after),
        char_shape_signature(before),
        "field owner char-shape boundaries should survive roundtrip"
    );
}

#[test]
fn set_field_value_updates_empty_click_here_range() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/field-01.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let mut core = DocumentCore::from_bytes(&bytes).expect("parse field-01.hwp");

    let fields = core.collect_all_fields();
    assert!(!fields.is_empty(), "field-01.hwp should contain fields");

    let first_field = &fields[0];
    let field_name = first_field.field.field_name().unwrap_or("");
    assert!(!field_name.is_empty(), "first field should have a name");

    let result = core.set_field_value_by_name(field_name, "테스트회사");
    assert!(
        result.is_ok(),
        "set_field_value_by_name failed: {:?}",
        result.err()
    );

    let fields_after = core.collect_all_fields();
    let updated = fields_after
        .iter()
        .find(|f| f.field.field_name() == Some(field_name))
        .expect("field should still exist after set");

    // 핵심 검증: 빈 ClickHere range가 설정한 값만 포함하도록 확장되어야 한다.
    assert_eq!(
        updated.value, "테스트회사",
        "field value should be exactly the set value; got: '{}'",
        updated.value
    );
}

#[test]
fn set_field_value_roundtrips_two_empty_click_here_fields() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/field-01.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let mut core = DocumentCore::from_bytes(&bytes).expect("parse field-01.hwp");
    let fields = core.collect_all_fields();
    let empty_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere && f.value.is_empty())
        .collect();
    assert!(
        empty_fields.len() >= 2,
        "field-01.hwp should contain at least two empty ClickHere fields"
    );

    let id1 = empty_fields[0].field.field_id;
    let id2 = empty_fields[1].field.field_id;
    core.set_field_value_by_id(id1, "테스트회사")
        .expect("set first field");
    core.set_field_value_by_id(id2, "테스트작성자")
        .expect("set second field");

    for para_index in [7, 8] {
        let para = &core.document().sections[0].paragraphs[para_index];
        assert_eq!(
            para.char_offsets.len(),
            para.text.chars().count(),
            "para {para_index} char_offsets must match text chars"
        );
    }

    let saved = core.export_hwp_native().expect("export hwp");
    if let Ok(output_path) = std::env::var("RHWP_ISSUE838_OUT") {
        if let Some(parent) = Path::new(&output_path).parent() {
            fs::create_dir_all(parent).expect("create output parent");
        }
        fs::write(&output_path, &saved).expect("write issue #838 output hwp");
    }
    let reparsed = DocumentCore::from_bytes(&saved).expect("reparse exported hwp");
    let reparsed_fields = reparsed.collect_all_fields();
    let first = reparsed_fields
        .iter()
        .find(|f| f.field.field_id == id1)
        .expect("first field after reparse");
    let second = reparsed_fields
        .iter()
        .find(|f| f.field.field_id == id2)
        .expect("second field after reparse");

    assert_eq!(first.value, "테스트회사");
    assert_eq!(second.value, "테스트작성자");
    assert!(reparsed.document().sections[0].paragraphs[7]
        .text
        .starts_with("회사명\t\t: "));
    assert!(reparsed.document().sections[0].paragraphs[8]
        .text
        .starts_with("작성자\t\t: "));
    assert_field_layout_roundtrip(&core, &reparsed, "회사명");
    assert_field_layout_roundtrip(&core, &reparsed, "작성자");
}

#[test]
fn set_field_value_by_id_updates_virtual_hwpx_cell_and_roundtrips_layout() {
    const SAMPLE: &str = "samples/issue6102/36444579_traffic_fine_exemption.hwpx";
    const NAME: &str = "기관명";
    let mut core = load_sample(SAMPLE);
    let field = field_named(&core, NAME);
    assert_eq!(
        field.field.ctrl_id, 0,
        "fixture must expose a virtual cell field"
    );
    assert_eq!(field.location.nested_path.len(), 1);

    core.set_field_value_by_id(field.field.field_id, "검증기관")
        .expect("the fieldId returned for a virtual cell must be writable");
    assert_eq!(field_named(&core, NAME).value, "검증기관");

    let saved = core.export_hwpx_native().expect("export edited HWPX");
    let reparsed = DocumentCore::from_bytes(&saved).expect("reparse edited HWPX");
    assert_field_layout_roundtrip(&core, &reparsed, NAME);
}

#[test]
fn set_field_value_by_id_reflows_nested_hwp_clickhere() {
    const SAMPLE: &str = "samples/76076_regulatory_analysis.hwp";
    const NAME: &str = "안건명";
    let mut core = load_sample(SAMPLE);
    let field = field_named(&core, NAME);
    assert_ne!(
        field.field.ctrl_id, 0,
        "fixture must use a real ClickHere control"
    );
    assert_eq!(field.location.nested_path.len(), 1);

    core.set_field_value_by_id(field.field.field_id, "검증 안건명")
        .expect("set nested HWP ClickHere by id");
    let saved = core.export_hwp_native().expect("export edited HWP");
    let reparsed = DocumentCore::from_bytes(&saved).expect("reparse edited HWP");
    assert_field_layout_roundtrip(&core, &reparsed, NAME);
}

#[test]
fn set_field_value_by_name_reflows_textbox_clickhere() {
    const SAMPLE: &str = "samples/basic/BlogForm_BookReview.hwp";
    const NAME: &str = "이곳에 책 표지 그림을 넣으세요.";
    let mut core = load_sample(SAMPLE);
    let field = field_named(&core, NAME);
    assert!(matches!(
        field.location.nested_path.as_slice(),
        [NestedEntry::TableCell { .. }, NestedEntry::TextBox { .. }]
    ));

    core.set_field_value_by_name(NAME, "검증표지")
        .expect("set textbox ClickHere by name");
    let saved = core.export_hwp_native().expect("export edited HWP");
    let reparsed = DocumentCore::from_bytes(&saved).expect("reparse edited HWP");
    assert_field_layout_roundtrip(&core, &reparsed, NAME);
}

#[test]
fn set_field_value_by_name_at_reflows_deep_hwpx_clickhere() {
    const SAMPLE: &str = "samples/issue1893_clickhere_field_roundtrip.hwpx";
    const NAME: &str =
        "./ApplicationArea/cr:ApplicationPoliceArea/cr:Writer/cr:Organization/cr:Organization.Name;nnfield=";
    let mut core = load_sample(SAMPLE);
    let field = field_named(&core, NAME);
    assert_ne!(
        field.field.ctrl_id, 0,
        "fixture must use a real ClickHere control"
    );
    assert!(field.location.nested_path.len() >= 2);

    core.set_field_value_by_name_at(NAME, 0, "검증 관서")
        .expect("set deep HWPX ClickHere by occurrence");
    let saved = core.export_hwpx_native().expect("export edited HWPX");
    let reparsed = DocumentCore::from_bytes(&saved).expect("reparse edited HWPX");
    assert_field_layout_roundtrip(&core, &reparsed, NAME);
}
