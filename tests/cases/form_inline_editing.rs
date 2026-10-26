use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FormObject, FormType};
use rhwp::model::paragraph::{CharShapeRef, MarkpenMark, Paragraph, RangeTag};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn document(kind: FormType, inline: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    let mut form = FormObject {
        form_type: kind,
        name: "선택할 양식".into(),
        caption: "라디오 단추".into(),
        enabled: true,
        width: 8504,
        height: 1984,
        ..Default::default()
    };
    form.common.width = form.width;
    form.common.height = form.height;
    form.common.treat_as_char = inline;
    let para = &mut doc.sections[0].paragraphs[0];
    para.controls = vec![Control::Form(Box::new(form))];
    para.char_count = 9;
    para.has_para_text = true;
    core.set_document(doc);
    core
}

fn form_box(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::FormObject(_)) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(form_box)
}

#[test]
fn inline_forms_have_distinct_carets_and_copy_the_selected_control() {
    for kind in [
        FormType::PushButton,
        FormType::CheckBox,
        FormType::RadioButton,
        FormType::ComboBox,
        FormType::Edit,
    ] {
        let mut core = document(kind, true);
        assert!(core.document().sections[0].paragraphs[0].controls[0].is_logical_inline());
        let tree = core.build_page_render_tree(0).unwrap();
        let bbox = form_box(&tree.root).unwrap();
        let public = HwpDocument::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
        assert_eq!(public.get_logical_length(0, 0).unwrap(), 1);
        let rects: serde_json::Value =
            serde_json::from_str(&public.get_selection_rects(0, 0, 0, 0, 1).unwrap()).unwrap();
        assert!(
            rects
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["width"].as_f64().unwrap_or(0.0) > 0.0),
            "양식만 고른 블록을 표시한다"
        );
        for (fraction, expected) in [(0.1, 0), (0.9, 1)] {
            let hit: serde_json::Value = serde_json::from_str(
                &core
                    .hit_test_native(
                        0,
                        bbox.x + bbox.width * fraction,
                        bbox.y + bbox.height / 2.0,
                    )
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(hit["charOffset"], expected);
        }
        let before: serde_json::Value =
            serde_json::from_str(&core.get_cursor_rect_native(0, 0, 0).unwrap()).unwrap();
        let after: serde_json::Value =
            serde_json::from_str(&core.get_cursor_rect_native(0, 0, 1).unwrap()).unwrap();
        assert!((before["x"].as_f64().unwrap() - bbox.x).abs() < 0.1);
        assert!((after["x"].as_f64().unwrap() - bbox.x - bbox.width).abs() < 0.1);
        core.copy_selection_logical_native(0, 0, 0, 0, 1).unwrap();
        core.paste_internal_native(0, 0, 0).unwrap();
        assert_eq!(
            core.document().sections[0]
                .paragraphs
                .iter()
                .flat_map(|p| &p.controls)
                .filter(|c| matches!(c, Control::Form(_)))
                .count(),
            2,
            "선택한 양식 개체를 실제로 복제한다"
        );
    }
}

#[test]
fn floating_forms_do_not_take_an_inline_slot() {
    let core = document(FormType::RadioButton, false);
    assert!(!core.document().sections[0].paragraphs[0].controls[0].is_logical_inline());
}

#[test]
fn same_position_forms_copy_only_the_selected_logical_slot() {
    let mut core = document(FormType::RadioButton, true);
    let mut doc = core.document().clone();
    let template = doc.sections[0].paragraphs[0].controls[0].clone();
    let kinds = [
        FormType::PushButton,
        FormType::CheckBox,
        FormType::RadioButton,
        FormType::ComboBox,
        FormType::Edit,
    ];
    doc.sections[0].paragraphs[0].controls = kinds
        .into_iter()
        .enumerate()
        .map(|(index, kind)| {
            let Control::Form(mut form) = template.clone() else {
                unreachable!()
            };
            form.form_type = kind;
            form.name = format!("양식{index}");
            Control::Form(form)
        })
        .collect();
    doc.sections[0].paragraphs[0].char_count = 41;
    for selected in 0..5 {
        core.set_document(doc.clone());
        core.copy_selection_logical_native(0, 0, selected, 0, selected + 1)
            .unwrap();
        core.paste_internal_native(0, 0, 0).unwrap();
        let names: Vec<_> = core.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .filter_map(|c| match c {
                Control::Form(form) => Some(form.name.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(names.len(), 6);
        for index in 0..5 {
            assert_eq!(
                names
                    .iter()
                    .filter(|name| **name == format!("양식{index}"))
                    .count(),
                if index == selected { 2 } else { 1 }
            );
        }
    }
}

#[test]
fn invalid_form_copy_keeps_the_previous_clipboard() {
    let mut core = document(FormType::RadioButton, true);
    core.copy_selection_logical_native(0, 0, 0, 0, 1).unwrap();
    assert!(core.copy_selection_logical_native(0, 0, 1, 0, 0).is_err());
    assert!(core.copy_selection_logical_native(0, 0, 0, 0, 2).is_err());
    core.paste_internal_native(0, 0, 0).unwrap();
    assert_eq!(
        core.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .filter(|c| matches!(c, Control::Form(_)))
            .count(),
        2
    );
}

#[test]
fn copying_a_form_between_unicode_text_keeps_the_neighbouring_text() {
    let mut core = document(FormType::RadioButton, true);
    let mut doc = core.document().clone();
    let para = &mut doc.sections[0].paragraphs[0];
    para.text = "앞🦦뒤".into();
    para.char_offsets = vec![0, 9, 11];
    para.char_count = 13;
    core.set_document(doc);
    core.copy_selection_logical_native(0, 0, 1, 0, 2).unwrap();
    assert_eq!(core.get_clipboard_text_native(), "");
    assert_eq!(core.document().sections[0].paragraphs[0].text, "앞🦦뒤");
    core.paste_internal_native(0, 0, 0).unwrap();
    let paragraphs = &core.document().sections[0].paragraphs;
    assert_eq!(
        paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<String>(),
        "앞🦦뒤"
    );
    assert_eq!(
        paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .filter(|c| matches!(c, Control::Form(_)))
            .count(),
        2
    );
}

#[test]
fn radio_outer_bottom_right_edge_remains_visible_on_white_paper() {
    let core = document(FormType::RadioButton, true);
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    assert!(xml
        .descendants()
        .any(|n| n.has_tag_name("polyline") && n.attribute("stroke") == Some("#a0a0a0")));
    assert!(!xml
        .descendants()
        .any(|n| n.has_tag_name("polyline") && n.attribute("stroke") == Some("#fafafa")));
}

#[test]
fn deleting_one_of_shared_position_forms_preserves_others_and_round_trips() {
    let mut core = document(FormType::RadioButton, true);
    let mut doc = core.document().clone();
    let template = doc.sections[0].paragraphs[0].controls[0].clone();
    doc.sections[0].paragraphs[0].controls = (0..3)
        .map(|index| {
            let Control::Form(mut form) = template.clone() else {
                unreachable!()
            };
            form.name = format!("선택{index}");
            Control::Form(form)
        })
        .collect();
    doc.sections[0].paragraphs[0].char_count = 25;
    for selected in 0..3 {
        core.set_document(doc.clone());
        let before = core.save_snapshot_native();
        core.copy_selection_logical_native(0, 0, selected, 0, selected + 1)
            .unwrap();
        core.delete_range_logical_native(0, 0, selected, 0, selected + 1)
            .unwrap();
        let names = |core: &DocumentCore| {
            core.document().sections[0].paragraphs[0]
                .controls
                .iter()
                .filter_map(|control| match control {
                    Control::Form(form) => Some(form.name.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let expected: Vec<_> = (0..3)
            .filter(|index| *index != selected)
            .map(|index| format!("선택{index}"))
            .collect();
        assert_eq!(names(&core), expected);
        for bytes in [
            core.export_hwp_with_adapter().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            assert_eq!(names(&DocumentCore::from_bytes(&bytes).unwrap()), expected);
        }
        let after = core.save_snapshot_native();
        core.restore_snapshot_native(before).unwrap();
        assert_eq!(names(&core).len(), 3);
        core.restore_snapshot_native(after).unwrap();
        assert_eq!(names(&core), expected);
        core.paste_internal_native(0, 0, 0).unwrap();
        assert_eq!(names(&core).len(), 3);
    }
}

#[test]
fn invalid_form_deletion_keeps_model_events_and_clipboard() {
    let mut core = document(FormType::CheckBox, true);
    core.copy_selection_logical_native(0, 0, 0, 0, 1).unwrap();
    let before = format!("{:?}", core.document());
    let events = core.serialize_event_log();
    for (start, end) in [(1, 0), (0, 2)] {
        assert!(core
            .delete_range_logical_native(0, 0, start, 0, end)
            .is_err());
        assert_eq!(format!("{:?}", core.document()), before);
        assert_eq!(core.serialize_event_log(), events);
    }
    core.paste_internal_native(0, 0, 0).unwrap();
    assert_eq!(
        core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .filter(|c| matches!(c, Control::Form(_)))
            .count(),
        2
    );
}

#[test]
fn deleting_form_between_unicode_text_keeps_text_and_reopened_carets() {
    let mut core = document(FormType::Edit, true);
    let mut doc = core.document().clone();
    let paragraph = &mut doc.sections[0].paragraphs[0];
    paragraph.text = "앞🦦뒤".into();
    paragraph.char_offsets = vec![0, 9, 11];
    paragraph.char_count = 13;
    core.set_document(doc);
    core.delete_range_logical_native(0, 0, 1, 0, 2).unwrap();
    assert_eq!(core.document().sections[0].paragraphs[0].text, "앞🦦뒤");
    assert!(core.document().sections[0].paragraphs[0]
        .controls
        .is_empty());
    for bytes in [
        core.export_hwp_with_adapter().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = HwpDocument::from_bytes(&bytes).unwrap();
        assert_eq!(reopened.document().sections[0].paragraphs[0].text, "앞🦦뒤");
        assert_eq!(reopened.get_logical_length(0, 0).unwrap(), 3);
    }
}

fn cross_paragraph_forms() -> DocumentCore {
    let mut core = document(FormType::RadioButton, true);
    let mut doc = core.document().clone();
    let template = doc.sections[0].paragraphs[0].clone();
    let mut shape_ids = Vec::new();
    for color in [0x112233, 0x445566] {
        let mut style = doc.doc_info.char_shapes[0].clone();
        style.text_color = color;
        shape_ids.push(doc.doc_info.char_shapes.len() as u32);
        doc.doc_info.char_shapes.push(style);
    }
    let endpoints: Vec<_> = ["앞", "끝"]
        .into_iter()
        .enumerate()
        .map(|(side, label)| {
            let mut paragraph = template.clone();
            paragraph.text = if side == 0 {
                "앞🦦뒤"
            } else {
                "둘🦫끝"
            }
            .into();
            paragraph.char_offsets = vec![0, 25, 27];
            paragraph.char_count = 29;
            paragraph.char_shapes = vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: shape_ids[side],
            }];
            paragraph.controls = (0..3)
                .map(|index| {
                    let Control::Form(mut form) = template.controls[0].clone() else {
                        unreachable!()
                    };
                    form.name = format!("{label}{index}");
                    form.caption = format!("설명 {label}{index}");
                    form.fore_color = 0x102030 + index;
                    form.value = i32::from(index == 1);
                    Control::Form(form)
                })
                .collect();
            paragraph
        })
        .collect();
    let middle = Paragraph {
        text: "중간".into(),
        char_offsets: vec![0, 1],
        char_count: 3,
        has_para_text: true,
        ..Paragraph::new_empty_like(&template)
    };
    doc.sections[0].paragraphs = vec![endpoints[0].clone(), middle, endpoints[1].clone()];
    core.set_document(doc);
    core
}

fn form_names(core: &DocumentCore) -> Vec<String> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Form(form) => Some(form.name.clone()),
            _ => None,
        })
        .collect()
}

fn assert_cross_paragraph_remainder(core: &DocumentCore) {
    assert_eq!(form_names(core), ["앞0", "끝2"]);
    let paragraphs = &core.document().sections[0].paragraphs;
    assert_eq!(paragraphs.len(), 1);
    let paragraph = &paragraphs[0];
    assert_eq!(paragraph.text, "앞🦫끝");
    assert_eq!(paragraph.control_text_positions(), [1, 1]);
    for (character, color) in [(0, 0x112233), (1, 0x445566), (2, 0x445566)] {
        let id = paragraph.char_shape_id_at(character).unwrap();
        assert_eq!(
            core.document().doc_info.char_shapes[id as usize].text_color,
            color
        );
    }
    for control in &paragraph.controls {
        let Control::Form(form) = control else {
            continue;
        };
        assert_eq!(form.caption, format!("설명 {}", form.name));
        assert_eq!(
            form.fore_color,
            if form.name == "앞0" {
                0x102030
            } else {
                0x102032
            }
        );
        assert_eq!(form.value, 0);
    }
}

#[test]
fn cross_paragraph_shared_form_boundaries_copy_cut_and_restore_exact_siblings() {
    let mut core = cross_paragraph_forms();
    let mut doc = core.document().clone();
    doc.sections[0].paragraphs[0].range_tags = vec![RangeTag {
        start: 0,
        end: 1,
        tag: 0x03000001,
    }];
    doc.sections[0].paragraphs[2].range_tags = vec![RangeTag {
        start: 25,
        end: 28,
        tag: 0x03000002,
    }];
    core.set_document(doc);
    let original = format!("{:?}", core.document());
    let events = core.serialize_event_log();
    let before = core.save_snapshot_native();
    core.copy_selection_logical_native(0, 0, 2, 2, 3).unwrap();
    assert_eq!(core.get_clipboard_text_native(), "🦦뒤\n중간\n둘");
    assert_eq!(format!("{:?}", core.document()), original);
    assert_eq!(core.serialize_event_log(), events);
    core.paste_internal_native(0, 0, 0).unwrap();
    let copied = form_names(&core);
    for (name, count) in [
        ("앞0", 1),
        ("앞1", 2),
        ("앞2", 2),
        ("끝0", 2),
        ("끝1", 2),
        ("끝2", 1),
    ] {
        assert_eq!(
            copied
                .iter()
                .filter(|actual| actual.as_str() == name)
                .count(),
            count,
            "{name}"
        );
    }
    core.restore_snapshot_native(before).unwrap();
    core.delete_range_logical_native(0, 0, 2, 2, 3).unwrap();
    assert_cross_paragraph_remainder(&core);
    let tags: Vec<_> = core.document().sections[0].paragraphs[0]
        .range_tags
        .iter()
        .map(|tag| (tag.start, tag.end, tag.tag))
        .collect();
    assert_eq!(tags, [(0, 1, 0x03000001), (17, 20, 0x03000002)]);
    for bytes in [
        core.export_hwp_with_adapter().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_cross_paragraph_remainder(&DocumentCore::from_bytes(&bytes).unwrap());
    }
    let after = core.save_snapshot_native();
    core.restore_snapshot_native(before).unwrap();
    assert_eq!(
        form_names(&core),
        ["앞0", "앞1", "앞2", "끝0", "끝1", "끝2"]
    );
    assert_eq!(core.document().sections[0].paragraphs.len(), 3);
    core.restore_snapshot_native(after).unwrap();
    assert_cross_paragraph_remainder(&core);
    assert_eq!(core.get_clipboard_text_native(), "🦦뒤\n중간\n둘");
}

#[test]
fn cross_paragraph_form_invalid_and_empty_ranges_preserve_state_and_clipboard() {
    let mut core = cross_paragraph_forms();
    core.copy_selection_logical_native(0, 0, 1, 0, 2).unwrap();
    let before = format!("{:?}", core.document());
    let events = core.serialize_event_log();
    for (section, start_para, start, end_para, end) in [
        (0, 0, 2, 2, 7),
        (0, 2, 2, 0, 3),
        (1, 0, 2, 2, 3),
        (0, 0, 2, 3, 3),
    ] {
        assert!(core
            .copy_selection_logical_native(section, start_para, start, end_para, end)
            .is_err());
        assert!(core
            .delete_range_logical_native(section, start_para, start, end_para, end)
            .is_err());
        assert_eq!(format!("{:?}", core.document()), before);
        assert_eq!(core.serialize_event_log(), events);
    }
    core.copy_selection_logical_native(0, 0, 2, 0, 2).unwrap();
    core.delete_range_logical_native(0, 0, 2, 0, 2).unwrap();
    assert_eq!(format!("{:?}", core.document()), before);
    assert_eq!(core.serialize_event_log(), events);
    core.paste_internal_native(0, 0, 0).unwrap();
    let names = form_names(&core);
    assert_eq!(
        names.iter().filter(|name| name.as_str() == "앞0").count(),
        2
    );
    assert_eq!(names.len(), 7);
}

#[test]
fn cross_paragraph_form_metadata_and_mixed_object_guards_run_before_mutation() {
    for guard in 0..5 {
        let mut core = cross_paragraph_forms();
        let mut doc = core.document().clone();
        let paragraph = &mut doc.sections[0].paragraphs[2];
        match guard {
            0 => {
                paragraph.range_tags = vec![RangeTag {
                    start: 0,
                    end: 28,
                    tag: 0x03000001,
                }]
            }
            1 => {
                paragraph.markpen_marks = vec![
                    MarkpenMark {
                        char_idx: 0,
                        color: Some("#FFFF00".into()),
                        utf16_pos: Some(0),
                    },
                    MarkpenMark {
                        char_idx: 2,
                        color: None,
                        utf16_pos: Some(27),
                    },
                ]
            }
            2 => {
                paragraph.controls.push(Control::Field(Default::default()));
                paragraph
                    .field_ranges
                    .push(rhwp::model::paragraph::FieldRange {
                        start_char_idx: 0,
                        end_char_idx: 1,
                        control_idx: 3,
                        ..Default::default()
                    });
                paragraph.char_offsets = vec![8, 41, 43];
                paragraph.char_count = 45;
            }
            3 => {
                let Control::Form(form) = &mut paragraph.controls[1] else {
                    unreachable!()
                };
                form.common.treat_as_char = false;
            }
            _ => {
                let mut picture = rhwp::model::image::Picture::default();
                picture.common.treat_as_char = true;
                paragraph.controls.push(Control::Picture(Box::new(picture)));
                paragraph.char_offsets = vec![0, 33, 35];
                paragraph.char_count = 37;
            }
        }
        core.set_document(doc);
        core.copy_selection_logical_native(0, 0, 1, 0, 2).unwrap();
        let before = format!("{:?}", core.document());
        let events = core.serialize_event_log();
        assert!(
            core.delete_range_logical_native(0, 0, 2, 2, 3).is_err(),
            "guard {guard}"
        );
        assert_eq!(format!("{:?}", core.document()), before, "guard {guard}");
        assert_eq!(core.serialize_event_log(), events, "guard {guard}");
        core.paste_internal_native(0, 0, 0).unwrap();
        assert_eq!(
            form_names(&core)
                .iter()
                .filter(|name| name.as_str() == "앞0")
                .count(),
            2,
            "guard {guard}"
        );
    }
}
