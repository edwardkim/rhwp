//! Public-core contracts, not Hancom visual oracle fixtures.
use rhwp::document_core::DocumentCore;
use rhwp::model::{
    control::{Control, Field},
    paragraph::{OrphanFieldEnd, Paragraph},
    shape::{
        CommonObjAttr, ConnectorData, GroupShape, LineShape, RectangleShape, ShapeObject, TextBox,
    },
};
use serde_json::Value;
use std::collections::BTreeSet;

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}
fn append(core: &mut DocumentCore, para: Paragraph) -> usize {
    let paras = &mut core.document_mut().sections[0].paragraphs;
    let index = paras.len();
    paras.push(para);
    index
}
fn host(control: Control) -> Paragraph {
    Paragraph {
        controls: vec![control],
        char_count: 9,
        control_mask: 1 << 11,
        has_para_text: true,
        ..Default::default()
    }
}
fn paste(core: &mut DocumentCore) -> usize {
    let pi = append(core, Paragraph::default());
    let result: Value =
        serde_json::from_str(&core.paste_control_native(0, pi, 0).unwrap()).unwrap();
    assert_eq!(result["ok"], true);
    result["paraIdx"].as_u64().unwrap() as usize
}
fn control(core: &DocumentCore, pi: usize, ci: usize) -> &Control {
    &core.document().sections[0].paragraphs[pi].controls[ci]
}
fn unique_ids(value: &Value, output: &mut Vec<u32>) {
    match value {
        Value::Object(object) => {
            for (name, value) in object {
                let is_field = object.contains_key("field_type");
                if ((name == "instance_id" || name == "inst_id") && !is_field)
                    || (name == "field_id" && is_field)
                {
                    if let Some(id) = value.as_u64() {
                        output.push(id as u32);
                    }
                } else {
                    unique_ids(value, output);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                unique_ids(item, output);
            }
        }
        _ => {}
    }
}
fn ids(control: &Control) -> Vec<u32> {
    let mut ids = Vec::new();
    unique_ids(&serde_json::to_value(control).unwrap(), &mut ids);
    ids
}

#[test]
fn repository_table_and_textbox_copies_have_independent_owned_identities() {
    for (path, pi, ci) in [
        ("samples/hwp_table_test.hwp", 3, 0),
        ("samples/table-in-tbox.hwp", 0, 2),
    ] {
        let bytes = std::fs::read(path).expect("required repository fixture");
        let parsed = DocumentCore::from_bytes(&bytes).unwrap();
        for input in [bytes, parsed.export_hwpx_native().unwrap()] {
            let mut core = DocumentCore::from_bytes(&input).unwrap();
            let original = serde_json::to_value(control(&core, pi, ci)).unwrap();
            let original_ids: BTreeSet<_> = ids(control(&core, pi, ci)).into_iter().collect();
            core.copy_control_native(0, pi, &[], ci).unwrap();
            assert_eq!(
                serde_json::to_value(control(&core, pi, ci)).unwrap(),
                original
            );
            let first = paste(&mut core);
            let second = paste(&mut core);
            let first_ids = ids(control(&core, first, 0));
            let second_ids = ids(control(&core, second, 0));
            assert!(!first_ids.is_empty());
            assert!(
                first_ids
                    .iter()
                    .chain(&second_ids)
                    .all(|id| *id != 0 && !original_ids.contains(id)),
                "{path}"
            );
            assert!(
                second_ids.iter().all(|id| !first_ids.contains(id)),
                "{path}"
            );
            assert_eq!(
                serde_json::to_value(control(&core, pi, ci)).unwrap(),
                original
            );
            for output in [
                core.export_hwp_native().unwrap(),
                core.export_hwpx_native().unwrap(),
            ] {
                let reopened = DocumentCore::from_bytes(&output).unwrap();
                let top_id = |core: &DocumentCore, pi| match control(core, pi, 0) {
                    Control::Table(t) => t.common.instance_id,
                    Control::Shape(s) => s.common().instance_id,
                    other => panic!("unexpected {other:?}"),
                };
                assert_eq!(top_id(&reopened, first), top_id(&core, first));
                assert_eq!(top_id(&reopened, second), top_id(&core, second));
            }
        }
    }
}

fn group(ambiguous: bool, external: bool) -> Control {
    let common = |id| CommonObjAttr {
        instance_id: id,
        width: 7200,
        height: 3600,
        ..Default::default()
    };
    let rect = |id, inst| {
        let mut rect = RectangleShape {
            common: common(id),
            ..Default::default()
        };
        rect.drawing.inst_id = inst;
        ShapeObject::Rectangle(rect)
    };
    let mut line = LineShape {
        common: common(4000),
        connector: Some(ConnectorData {
            start_subject_id: 2000,
            end_subject_id: if external { 888888 } else { 3001 },
            ..Default::default()
        }),
        ..Default::default()
    };
    line.drawing.inst_id = 5000;
    Control::Shape(Box::new(ShapeObject::Group(GroupShape {
        common: common(9000),
        children: vec![
            ShapeObject::Line(line),
            rect(1000, 2000),
            rect(3000, if ambiguous { 2000 } else { 6000 }),
        ],
        ..Default::default()
    })))
}
fn group_of(core: &DocumentCore, pi: usize) -> &GroupShape {
    match control(core, pi, 0) {
        Control::Shape(s) => match s.as_ref() {
            ShapeObject::Group(g) => g,
            _ => panic!("group"),
        },
        _ => panic!("shape"),
    }
}

#[test]
fn forward_connector_references_follow_the_clone_and_external_refs_stay_external() {
    for external in [false, true] {
        let mut core = blank();
        let src = append(&mut core, host(group(false, external)));
        core.copy_control_native(0, src, &[], 0).unwrap();
        let dst = paste(&mut core);
        let group = group_of(&core, dst);
        let ShapeObject::Line(line) = &group.children[0] else {
            panic!("line")
        };
        let conn = line.connector.as_ref().unwrap();
        assert_eq!(
            conn.start_subject_id,
            group.children[1].drawing().unwrap().inst_id
        );
        assert_eq!(
            conn.end_subject_id,
            if external {
                888888
            } else {
                group.children[2].drawing().unwrap().inst_id
            }
        );
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            let g = group_of(&reopened, dst);
            let ShapeObject::Line(l) = &g.children[0] else {
                panic!("line")
            };
            assert_eq!(
                l.connector.as_ref().unwrap().start_subject_id,
                conn.start_subject_id
            );
            assert_eq!(
                l.connector.as_ref().unwrap().end_subject_id,
                conn.end_subject_id
            );
        }
    }
}

#[test]
fn ambiguous_subject_is_rejected_without_document_events_or_clipboard_mutation() {
    let mut core = blank();
    let src = append(&mut core, host(group(true, false)));
    core.copy_control_native(0, src, &[], 0).unwrap();
    let dst = append(&mut core, Paragraph::default());
    let before = serde_json::to_value(&core.document().sections[0].paragraphs).unwrap();
    let events = core.serialize_event_log();
    let clip = core.get_clipboard_text_native();
    assert!(core
        .paste_control_native(0, dst, 0)
        .unwrap_err()
        .to_string()
        .contains("ambiguous connector subject"));
    assert_eq!(
        serde_json::to_value(&core.document().sections[0].paragraphs).unwrap(),
        before
    );
    assert_eq!(core.serialize_event_log(), events);
    assert_eq!(core.get_clipboard_text_native(), clip);
}

#[test]
fn invalid_destination_does_not_consume_cascade_step() {
    let mut actual = blank();
    let mut expected = blank();
    for core in [&mut actual, &mut expected] {
        let src = append(core, host(group(false, false)));
        core.copy_control_native(0, src, &[], 0).unwrap();
    }
    assert!(actual.paste_control_native(usize::MAX, 0, 0).is_err());
    let a = paste(&mut actual);
    let b = paste(&mut expected);
    assert_eq!(
        serde_json::to_value(control(&actual, a, 0)).unwrap(),
        serde_json::to_value(control(&expected, b, 0)).unwrap()
    );
}

#[test]
fn nested_field_begin_references_change_but_shared_fieldid_metadata_does_not() {
    let mut core = blank();
    let mut rect = RectangleShape::default();
    rect.common.instance_id = 1000;
    rect.drawing.inst_id = 2000;
    rect.drawing.text_box = Some(TextBox {
        paragraphs: vec![
            host(Control::Field(Field {
                field_id: 77,
                instance_id: Some(777),
                ..Default::default()
            })),
            Paragraph {
                orphan_field_ends: vec![OrphanFieldEnd {
                    begin_id_ref: 77,
                    field_id: 777,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    let src = append(
        &mut core,
        host(Control::Shape(Box::new(ShapeObject::Rectangle(rect)))),
    );
    core.copy_control_native(0, src, &[], 0).unwrap();
    let dst = paste(&mut core);
    let Control::Shape(s) = control(&core, dst, 0) else {
        panic!("shape")
    };
    let paras = &s.drawing().unwrap().text_box.as_ref().unwrap().paragraphs;
    let Control::Field(f) = &paras[0].controls[0] else {
        panic!("field")
    };
    assert_ne!(f.field_id, 77);
    assert_eq!(paras[1].orphan_field_ends[0].begin_id_ref, f.field_id);
    assert_eq!(f.instance_id, Some(777));
    assert_eq!(paras[1].orphan_field_ends[0].field_id, 777);
}

#[test]
fn editing_and_deleting_one_table_copy_leaves_original_and_other_copy_intact() {
    let mut core = blank();
    let created: Value =
        serde_json::from_str(&core.create_table_native(0, 0, 0, 2, 2).unwrap()).unwrap();
    let src = created["paraIdx"].as_u64().unwrap() as usize;
    core.copy_control_native(0, src, &[], 0).unwrap();
    let first = paste(&mut core);
    let second = paste(&mut core);
    let original = serde_json::to_value(control(&core, src, 0)).unwrap();
    let other = serde_json::to_value(control(&core, second, 0)).unwrap();
    core.insert_text_in_cell_native(0, first, 0, 0, 0, 0, "independent")
        .unwrap();
    assert_eq!(
        serde_json::to_value(control(&core, src, 0)).unwrap(),
        original
    );
    assert_eq!(
        serde_json::to_value(control(&core, second, 0)).unwrap(),
        other
    );
    core.delete_control_native(0, first, 0).unwrap();
    assert_eq!(
        serde_json::to_value(control(&core, src, 0)).unwrap(),
        original
    );
    let remaining = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter(|c| matches!(c, Control::Table(_)))
        .collect::<Vec<_>>();
    assert_eq!(remaining.len(), 2);
    assert_eq!(serde_json::to_value(remaining[1]).unwrap(), other);
}
