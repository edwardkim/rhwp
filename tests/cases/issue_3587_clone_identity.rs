//! #3587 A1: creation identity is independent of table dimensions and file format.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use std::collections::BTreeSet;

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}

fn create(core: &mut DocumentCore, inline: bool) {
    let pi = core.document().sections[0].paragraphs.len() - 1;
    if inline {
        core.create_table_ex_native(0, pi, 0, 2, 2, true, None, None)
            .unwrap();
    } else {
        core.create_table_native(0, pi, 0, 2, 2).unwrap();
    }
}

fn tables(core: &DocumentCore) -> Vec<&Table> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .collect()
}

fn ids(core: &DocumentCore) -> Vec<u32> {
    tables(core).iter().map(|t| t.common.instance_id).collect()
}

// HWP 5.0 common CTRL_HEADER instance identity occupies bytes 32..36.
fn raw_id(table: &Table) -> u32 {
    u32::from_le_bytes(table.raw_ctrl_data[32..36].try_into().unwrap())
}

#[test]
fn equal_size_creations_have_unique_nonzero_common_and_raw_ids() {
    for inline in [false, true] {
        let mut core = blank();
        for _ in 0..3 {
            create(&mut core, inline);
        }
        let all = tables(&core);
        assert_eq!(all.len(), 3);
        assert_eq!(
            all.iter().map(|t| raw_id(t)).collect::<BTreeSet<_>>().len(),
            3
        );
        for table in all {
            assert_ne!(table.common.instance_id, 0);
            assert_eq!(table.common.instance_id, raw_id(table));
        }
    }
}

#[test]
fn identity_survives_both_formats_and_further_creation() {
    let mut core = blank();
    create(&mut core, false);
    create(&mut core, true);
    let expected = ids(&core);
    assert_eq!(expected.iter().copied().collect::<BTreeSet<_>>().len(), 2);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let mut reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(ids(&reopened), expected);
        create(&mut reopened, false);
        create(&mut reopened, true);
        let new_ids = ids(&reopened);
        assert_eq!(new_ids.iter().copied().collect::<BTreeSet<_>>().len(), 4);
        for output in [
            reopened.export_hwp_native().unwrap(),
            reopened.export_hwpx_native().unwrap(),
        ] {
            let roundtrip = DocumentCore::from_bytes(&output).unwrap();
            assert_eq!(ids(&roundtrip), new_ids);
        }
    }
}

#[test]
fn nested_and_disagreeing_raw_identity_are_reserved_without_rewriting_originals() {
    let mut core = blank();
    create(&mut core, false);
    let mut outer = tables(&core)[0].clone();
    let mut nested = outer.clone();
    outer.common.instance_id = u32::MAX;
    outer.raw_ctrl_data[32..36].copy_from_slice(&1u32.to_le_bytes());
    nested.common.instance_id = 2;
    nested.raw_ctrl_data[32..36].copy_from_slice(&3u32.to_le_bytes());
    outer.cells[0].paragraphs[0]
        .controls
        .push(Control::Table(Box::new(nested)));
    core.document_mut().sections[0].paragraphs[0].controls = vec![Control::Table(Box::new(outer))];
    let original = serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap();
    create(&mut core, false);
    let allocated = tables(&core).last().unwrap().common.instance_id;
    assert!(![0, 1, 2, 3, u32::MAX].contains(&allocated));
    assert_eq!(
        serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap(),
        original
    );
}

#[test]
fn allocation_is_deterministic_and_invalid_destination_does_not_mutate_document() {
    let mut first = blank();
    let mut second = blank();
    for inline in [false, true, false] {
        create(&mut first, inline);
        create(&mut second, inline);
    }
    assert_eq!(ids(&first), ids(&second));
    let before = first.export_hwp_native().unwrap();
    assert!(first.create_table_native(usize::MAX, 0, 0, 2, 2).is_err());
    assert!(first
        .create_table_ex_native(0, usize::MAX, 0, 2, 2, true, None, None)
        .is_err());
    assert_eq!(first.export_hwp_native().unwrap(), before);
}

#[test]
fn group_textbox_caption_header_and_master_page_identities_are_reserved() {
    use rhwp::model::control::Field;
    use rhwp::model::header_footer::{Header, MasterPage};
    use rhwp::model::paragraph::Paragraph;
    use rhwp::model::shape::{Caption, GroupShape, RectangleShape, ShapeObject, TextBox};

    let mut core = blank();
    create(&mut core, false);
    let template = tables(&core)[0].clone();
    let para_with = |control| Paragraph {
        controls: vec![control],
        ..Default::default()
    };
    let table_para = |id: u32| {
        let mut table = template.clone();
        table.common.instance_id = id;
        table.raw_ctrl_data[32..36].copy_from_slice(&(id + 1).to_le_bytes());
        para_with(Control::Table(Box::new(table)))
    };
    let field = Field {
        instance_id: Some(4),
        memo_paragraphs: vec![table_para(5)],
        ..Default::default()
    };
    let mut rectangle = RectangleShape::default();
    rectangle.common.instance_id = 2;
    rectangle.drawing.inst_id = 3;
    rectangle.drawing.text_box = Some(TextBox {
        paragraphs: vec![para_with(Control::Field(field))],
        ..Default::default()
    });
    rectangle.drawing.caption = Some(Caption {
        paragraphs: vec![table_para(7)],
        ..Default::default()
    });
    let mut group = GroupShape::default();
    group.common.instance_id = 1;
    group.children.push(ShapeObject::Rectangle(rectangle));
    let section = &mut core.document_mut().sections[0];
    section.paragraphs[0].controls = vec![
        Control::Shape(Box::new(ShapeObject::Group(group))),
        Control::Header(Box::new(Header {
            paragraphs: vec![table_para(11)],
            ..Default::default()
        })),
    ];
    section.section_def.master_pages.push(MasterPage {
        paragraphs: vec![table_para(9)],
        ..Default::default()
    });
    let original = serde_json::to_value(&section.paragraphs[0]).unwrap();
    create(&mut core, false);
    assert!(tables(&core).last().unwrap().common.instance_id > 12);
    assert_eq!(
        serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap(),
        original
    );
}
