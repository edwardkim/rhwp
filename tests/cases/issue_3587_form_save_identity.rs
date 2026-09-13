//! #3587 A4: explicit identities survive saving; only missing Form IDs are synthesized.
use rhwp::document_core::DocumentCore;
use rhwp::model::{
    control::{Control, FieldType},
    document::Document,
};
use std::collections::BTreeSet;

fn open(path: &str) -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn field_ids(core: &DocumentCore) -> Vec<u32> {
    core.collect_all_fields()
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .map(|f| f.field.field_id)
        .collect()
}

fn field_values(core: &DocumentCore) -> Vec<String> {
    core.collect_all_fields()
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .map(|f| f.value.clone())
        .collect()
}

fn form_ids(doc: &Document) -> Vec<u32> {
    doc.sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            Control::Form(f) => Some(f.common.instance_id),
            _ => None,
        })
        .collect()
}

#[test]
fn repeated_clickhere_paste_after_forms_preserves_field_identities_in_both_saves() {
    for path in [
        "samples/form-01.hwp",
        "samples/hwpx/form-01.hwpx",
        "samples/form-02.hwp",
        "samples/hwpx/form-02.hwpx",
    ] {
        let mut core = open(path);
        let (si, pi, start, end) = {
            let fields = core.collect_all_fields();
            let field = fields
                .iter()
                .find(|f| f.field.field_type == FieldType::ClickHere)
                .unwrap();
            let si = field.location.section_index;
            let pi = field.location.para_index;
            let range =
                &core.document().sections[si].paragraphs[pi].field_ranges[field.field_range_index];
            (si, pi, range.start_char_idx, range.end_char_idx)
        };
        let original = field_ids(&core);
        core.copy_selection_native(si, pi, start, pi, end).unwrap();
        for _ in 0..2 {
            let dst = core.document().sections[si].paragraphs.len();
            let last_len = core.document().sections[si].paragraphs[dst - 1]
                .text
                .chars()
                .count();
            core.split_paragraph_native(si, dst - 1, last_len, None)
                .unwrap();
            core.paste_internal_native(si, dst, 0).unwrap();
        }
        let expected = field_ids(&core);
        let expected_values = field_values(&core);
        assert_eq!(expected.len(), original.len() + 2, "{path}");
        assert_eq!(&expected[..original.len()], &original);
        assert_eq!(
            expected.iter().copied().collect::<BTreeSet<_>>().len(),
            expected.len()
        );
        for output in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&output).unwrap();
            assert_eq!(field_ids(&reopened), expected, "{path}");
            assert_eq!(field_values(&reopened), expected_values, "{path}");
        }
        assert_eq!(field_ids(&core), expected, "saving must not mutate input");
    }
}

#[test]
fn explicit_form_ids_survive_even_without_hwp_header_provenance() {
    let mut core = open("samples/hwpx/form-01.hwpx");
    let mut next = 900001;
    for para in &mut core.document_mut().sections[0].paragraphs {
        for ctrl in &mut para.controls {
            if let Control::Form(form) = ctrl {
                form.common.attr = 0;
                form.common.instance_id = next;
                next += 1;
            }
        }
    }
    let expected = form_ids(core.document());
    assert_eq!(expected.len(), 5);
    let reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert_eq!(form_ids(reopened.document()), expected);
    assert_eq!(form_ids(core.document()), expected);
}

#[test]
fn missing_form_ids_reserve_other_sections_and_do_not_mutate_the_input() {
    let mut core = open("samples/hwpx/form-01.hwpx");
    let mut extra = core.document().sections[0].clone();
    // An existing source identity blocks the old seed, including in a later section.
    let mut count = 0;
    for para in &mut extra.paragraphs {
        for ctrl in &mut para.controls {
            if let Control::Form(f) = ctrl {
                f.common.attr = 0x002a6211;
                f.common.instance_id = 0x7dcd59d6 + count;
                count += 1;
            }
        }
    }
    core.document_mut().sections.push(extra);
    let original = form_ids(core.document());
    let bytes = core.export_hwp_native().unwrap();
    let reopened = DocumentCore::from_bytes(&bytes).unwrap();
    let saved = form_ids(reopened.document());
    assert_eq!(saved.len(), 10);
    assert_eq!(saved.iter().copied().collect::<BTreeSet<_>>().len(), 10);
    assert_eq!(&saved[5..], &original[5..]);
    assert_eq!(form_ids(core.document()), original);
    assert_eq!(core.export_hwp_native().unwrap(), bytes);
}

#[test]
fn original_hwp_zero_form_identity_is_not_treated_as_missing() {
    let mut core = open("samples/form-01.hwp");
    for para in &mut core.document_mut().sections[0].paragraphs {
        for ctrl in &mut para.controls {
            if let Control::Form(f) = ctrl {
                assert_ne!(f.common.attr, 0);
                f.common.instance_id = 0;
            }
        }
    }
    let saved = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert_eq!(form_ids(saved.document()), vec![0; 5]);
}

#[test]
fn unchanged_form_fixtures_keep_attributes_and_legacy_nonconflicting_ids() {
    for stem in ["form-01", "form-02"] {
        let golden = open(&format!("samples/{stem}.hwp"));
        let expected = form_ids(golden.document());
        for path in [
            format!("samples/{stem}.hwp"),
            format!("samples/hwpx/{stem}.hwpx"),
        ] {
            let core = open(&path);
            let before_fields = field_ids(&core);
            let bytes = core.export_hwp_native().unwrap();
            let saved = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(form_ids(saved.document()), expected, "{path}");
            assert_eq!(field_ids(&saved), before_fields, "{path}");
            let attributes = |doc: &Document| {
                doc.sections
                    .iter()
                    .flat_map(|s| &s.paragraphs)
                    .flat_map(|p| &p.controls)
                    .filter_map(|c| match c {
                        Control::Form(f) => Some((
                            f.name.clone(),
                            f.width,
                            f.height,
                            f.caption.clone(),
                            f.text.clone(),
                        )),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            };
            // HWPX selectedValue="" becomes its first list item in the legacy
            // HWP ComboBox writer. Compare the HWP oracle, not unlike IR slots.
            assert_eq!(attributes(saved.document()), attributes(golden.document()));
            assert_eq!(core.export_hwp_native().unwrap(), bytes);
        }
    }
}
