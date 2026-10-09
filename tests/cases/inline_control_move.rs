//! 인라인 이동의 글자·서식·슬롯·소유 주소·저장 계약.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::{
        inline_control::{InlineControlAddress, InlineControlCaret, InlineControlOwner},
        DocumentCore,
    },
    model::{
        control::{Control, Equation, Field, FieldType},
        paragraph::{CharShapeRef, FieldRange, MarkpenMark, Paragraph, RangeTag, TitleMark},
        shape::CommonObjAttr,
        table::{Cell, Table},
    },
};

fn body(para: usize) -> InlineControlOwner {
    InlineControlOwner::Body {
        section_index: 0,
        paragraph_index: para,
        cell_path: vec![],
    }
}
fn source(owner: InlineControlOwner, ci: usize) -> InlineControlAddress {
    InlineControlAddress {
        owner,
        control_index: ci,
    }
}
fn caret(owner: InlineControlOwner, offset: usize) -> InlineControlCaret {
    InlineControlCaret {
        owner,
        char_offset: offset,
    }
}
fn equation(id: u32) -> Control {
    Control::Equation(Box::new(Equation {
        common: CommonObjAttr {
            treat_as_char: true,
            width: 1000,
            height: 1000,
            instance_id: id,
            ..Default::default()
        },
        script: format!("{id}"),
        font_size: 1000,
        ..Default::default()
    }))
}
fn paragraph(text: &str, items: Vec<(usize, Control)>) -> Paragraph {
    let mut offset = 0;
    let mut offsets = vec![];
    for (i, ch) in text.chars().enumerate() {
        offset += 8 * items.iter().filter(|(position, _)| *position == i).count() as u32;
        offsets.push(offset);
        offset += if ch == '\t' { 8 } else { ch.len_utf16() as u32 };
    }
    offset += 8 * items
        .iter()
        .filter(|(position, _)| *position == text.chars().count())
        .count() as u32;
    Paragraph {
        text: text.into(),
        char_offsets: offsets,
        char_count: offset + 1,
        controls: items.into_iter().map(|(_, control)| control).collect(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        has_para_text: true,
        ..Default::default()
    }
}
fn core(paragraphs: Vec<Paragraph>) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    while doc.doc_info.char_shapes.len() < 3 {
        doc.doc_info
            .char_shapes
            .push(doc.doc_info.char_shapes[0].clone());
    }
    doc.sections[0].paragraphs = paragraphs;
    doc.sections[0].raw_stream = None;
    core.set_document(doc);
    core
}
fn object_ids(para: &Paragraph) -> Vec<u32> {
    para.controls
        .iter()
        .filter_map(|control| match control {
            Control::Equation(eq) => Some(eq.common.instance_id),
            _ => None,
        })
        .collect()
}
fn text_styles(para: &Paragraph) -> Vec<u32> {
    para.char_offsets
        .iter()
        .map(|offset| {
            para.char_shapes
                .iter()
                .rfind(|run| run.start_pos <= *offset)
                .unwrap()
                .char_shape_id
        })
        .collect()
}
fn unchanged(core: &mut DocumentCore, source: &InlineControlAddress, dest: &InlineControlCaret) {
    let before = format!("{:?}", core.document());
    let events = core.serialize_event_log();
    assert!(core.move_inline_control_native(source, dest).is_err());
    assert_eq!(format!("{:?}", core.document()), before);
    assert_eq!(core.serialize_event_log(), events);
}

#[test]
fn same_paragraph_forward_and_back_preserve_unicode_order_styles_data_and_undo() {
    let mut p = paragraph("가😀나\t다", vec![(1, equation(41)), (3, equation(42))]);
    p.ctrl_data_records = vec![Some(vec![4, 1, 7, 9]), Some(vec![4, 2, 7, 9])];
    p.char_shapes = vec![
        CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        },
        CharShapeRef {
            start_pos: 1,
            char_shape_id: 1,
        },
        CharShapeRef {
            start_pos: 9,
            char_shape_id: 2,
        },
        CharShapeRef {
            start_pos: 12,
            char_shape_id: 0,
        },
    ];
    p.range_tags = vec![RangeTag {
        start: 1,
        end: 9,
        tag: 5,
    }];
    let mut core = core(vec![p]);
    let original = core.document().sections[0].paragraphs[0].clone();
    let snapshot = core.save_snapshot_native();
    let result = core
        .move_inline_control_native(&source(body(0), 0), &caret(body(0), 7))
        .unwrap();
    assert!(result.changed);
    assert_eq!(result.address.control_index, 1);
    assert_eq!(result.char_offset, 6);
    let p = &core.document().sections[0].paragraphs[0];
    assert_eq!(p.text, original.text);
    assert_eq!(p.char_offsets, vec![0, 1, 3, 12, 20]);
    assert_eq!(object_ids(p), vec![42, 41]);
    assert_eq!(text_styles(p), text_styles(&original));
    assert_eq!(
        p.ctrl_data_records,
        vec![Some(vec![4, 2, 7, 9]), Some(vec![4, 1, 7, 9])]
    );
    assert!(p
        .range_tags
        .iter()
        .any(|tag| tag.start == 21 && tag.end == 29 && tag.tag == 5));
    core.move_inline_control_native(&result.address, &caret(body(0), 1))
        .unwrap();
    let p = &core.document().sections[0].paragraphs[0];
    assert_eq!(p.char_offsets, original.char_offsets);
    assert_eq!(object_ids(p), vec![41, 42]);
    assert_eq!(text_styles(p), text_styles(&original));
    core.restore_snapshot_native(snapshot).unwrap();
    assert_eq!(
        core.document().sections[0].paragraphs[0].char_offsets,
        original.char_offsets
    );
}

#[test]
fn adjacent_objects_have_distinct_before_after_carets_and_no_op_emits_nothing() {
    let mut core = core(vec![paragraph(
        "ab",
        vec![(1, equation(41)), (1, equation(42)), (1, equation(43))],
    )]);
    for offset in [2, 3] {
        let before = format!("{:?}", core.document());
        let events = core.serialize_event_log();
        assert!(
            !core
                .move_inline_control_native(&source(body(0), 1), &caret(body(0), offset))
                .unwrap()
                .changed
        );
        assert_eq!(format!("{:?}", core.document()), before);
        assert_eq!(core.serialize_event_log(), events);
    }
    core.move_inline_control_native(&source(body(0), 2), &caret(body(0), 2))
        .unwrap();
    assert_eq!(
        object_ids(&core.document().sections[0].paragraphs[0]),
        vec![41, 43, 42]
    );
}

#[test]
fn cross_paragraph_move_keeps_paragraphs_and_both_save_formats() {
    let mut core = core(vec![
        paragraph("앞뒤", vec![(1, equation(41))]),
        paragraph("ABC", vec![(2, equation(42))]),
    ]);
    let result = core
        .move_inline_control_native(&source(body(0), 0), &caret(body(1), 1))
        .unwrap();
    assert_eq!(result.address, source(body(1), 0));
    assert_eq!(core.document().sections[0].paragraphs.len(), 2);
    assert_eq!(
        core.document().sections[0].paragraphs[0].char_offsets,
        vec![0, 1]
    );
    assert_eq!(
        core.document().sections[0].paragraphs[1].char_offsets,
        vec![0, 9, 18]
    );
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(reopened.document().sections[0].paragraphs.len(), 2);
        let p = &reopened.document().sections[0].paragraphs;
        assert_eq!((&p[0].text, &p[1].text), (&"앞뒤".into(), &"ABC".into()));
        assert!(object_ids(&p[0]).is_empty());
        assert_eq!(object_ids(&p[1]), vec![41, 42]);
        assert_eq!(p[1].control_text_positions(), vec![1, 2]);
    }
}

#[test]
fn zero_length_field_slot_is_removed_exactly_and_field_indices_and_marks_survive() {
    let field = Control::Field(Field {
        field_type: FieldType::ClickHere,
        field_id: 71,
        ctrl_id: rhwp::parser::tags::FIELD_CLICKHERE,
        command: Field::build_clickhere_command("안내", ""),
        properties: 1 << 15,
        ..Default::default()
    });
    let mut p = paragraph(
        "ABC",
        vec![(1, field), (1, equation(41)), (2, equation(42))],
    );
    // B 앞은 BEGIN/개체/END, C 앞은 제목 표식/두 번째 개체다.
    p.char_offsets = vec![0, 25, 42];
    p.char_count = 44;
    p.field_ranges = vec![FieldRange {
        start_char_idx: 1,
        end_char_idx: 1,
        control_idx: 0,
        inner_slot_count: 1,
        end_field_id: 91,
    }];
    p.title_marks = vec![TitleMark {
        char_idx: 2,
        ignore: false,
    }];
    p.ctrl_data_records = vec![Some(vec![1]), Some(vec![2]), Some(vec![3])];
    let mut core = core(vec![paragraph("서문", vec![]), p, paragraph("XY", vec![])]);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let baseline = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            baseline.document().sections[0].paragraphs[1]
                .field_ranges
                .len(),
            1
        );
    }
    core.move_inline_control_native(&source(body(1), 1), &caret(body(2), 1))
        .unwrap();
    let p = &core.document().sections[0].paragraphs[1];
    assert_eq!(p.text, "ABC");
    assert_eq!(p.char_offsets, vec![0, 17, 34]);
    assert_eq!(p.field_ranges[0].inner_slot_count, 0);
    assert_eq!(p.field_ranges[0].end_field_id, 91);
    assert_eq!(
        p.title_marks,
        vec![TitleMark {
            char_idx: 2,
            ignore: false
        }]
    );
    assert_eq!(p.ctrl_data_records, vec![Some(vec![1]), Some(vec![3])]);
    assert_eq!(object_ids(p), vec![42]);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let p = &reopened.document().sections[0].paragraphs[1];
        assert_eq!(p.text, "ABC");
        assert_eq!(p.field_ranges.len(), 1, "저장된 컨트롤: {:?}", p.controls);
        assert_eq!(p.field_ranges[0].inner_slot_count, 0);
        assert_eq!(
            object_ids(&reopened.document().sections[0].paragraphs[2]),
            vec![41]
        );
    }
}

fn table(child: Paragraph) -> Control {
    Control::Table(Box::new(Table {
        common: CommonObjAttr {
            treat_as_char: true,
            width: 5000,
            height: 3000,
            instance_id: 90,
            ..Default::default()
        },
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            width: 5000,
            height: 3000,
            paragraphs: vec![child],
            ..Default::default()
        }],
        ..Default::default()
    }))
}

#[test]
fn target_sibling_path_reindexes_after_source_removal_and_own_descendant_is_rejected() {
    let mut core = core(vec![paragraph(
        "",
        vec![(0, equation(41)), (0, table(paragraph("셀", vec![])))],
    )]);
    let old_owner = InlineControlOwner::Body {
        section_index: 0,
        paragraph_index: 0,
        cell_path: vec![(1, 0, 0)],
    };
    let result = core
        .move_inline_control_native(&source(body(0), 0), &caret(old_owner, 1))
        .unwrap();
    assert_eq!(
        result.address.owner,
        InlineControlOwner::Body {
            section_index: 0,
            paragraph_index: 0,
            cell_path: vec![(0, 0, 0)]
        }
    );
    let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[0] else {
        panic!()
    };
    assert_eq!(table.cells[0].paragraphs[0].text, "셀");
    assert_eq!(object_ids(&table.cells[0].paragraphs[0]), vec![41]);
    unchanged(
        &mut core,
        &source(body(0), 0),
        &caret(result.address.owner, 0),
    );
}

#[test]
fn protected_cells_distribution_invalid_control_and_invalid_caret_are_atomic() {
    let mut core = core(vec![
        paragraph("앞뒤", vec![(1, equation(41))]),
        paragraph("", vec![(0, table(paragraph("셀", vec![])))]),
    ]);
    let cell_owner = InlineControlOwner::Body {
        section_index: 0,
        paragraph_index: 1,
        cell_path: vec![(0, 0, 0)],
    };
    let Control::Table(table) = &mut core.document_mut().sections[0].paragraphs[1].controls[0]
    else {
        panic!()
    };
    table.cells[0].set_cell_protect(true);
    unchanged(&mut core, &source(body(0), 0), &caret(cell_owner, 0));
    unchanged(&mut core, &source(body(0), 9), &caret(body(1), 0));
    unchanged(&mut core, &source(body(0), 0), &caret(body(1), 900));
    core.document_mut().header.distribution = true;
    unchanged(&mut core, &source(body(0), 0), &caret(body(1), 0));
}

#[test]
fn header_and_note_destinations_return_existing_owner_forms() {
    let mut core = core(vec![paragraph(
        "앞뒤",
        vec![(1, equation(41)), (1, equation(42))],
    )]);
    core.create_header_footer_native(0, true, 0).unwrap();
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "머리말")
        .unwrap();
    let header = InlineControlOwner::HeaderFooter {
        section_index: 0,
        is_header: true,
        apply_to: 0,
        paragraph_index: 0,
        cell_path: vec![],
    };
    let moved = core
        .move_inline_control_native(&source(body(0), 0), &caret(header.clone(), 1))
        .unwrap();
    assert_eq!(moved.address.owner, header);
    core.move_inline_control_native(&moved.address, &caret(body(0), 0))
        .unwrap();
    core.insert_footnote_native(0, 0, 0).unwrap();
    let ci = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Footnote(_)))
        .unwrap();
    let note = InlineControlOwner::Note {
        section_index: 0,
        parent_paragraph_index: 0,
        note_control_index: ci,
        paragraph_index: 0,
        cell_path: vec![],
    };
    let ei = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Equation(_)))
        .unwrap();
    let moved = core
        .move_inline_control_native(&source(body(0), ei), &caret(note.clone(), 0))
        .unwrap();
    let InlineControlOwner::Note {
        note_control_index, ..
    } = moved.address.owner
    else {
        panic!()
    };
    assert_eq!(note_control_index, ci - usize::from(ei < ci));
    core.move_inline_control_native(&moved.address, &caret(body(0), 0))
        .unwrap();
}

#[test]
fn picture_binary_identity_and_control_metadata_survive_cross_owner_move() {
    let bytes = include_bytes!("../../assets/logo/logo-16.png");
    let mut core = core(vec![paragraph("앞뒤", vec![]), paragraph("ABC", vec![])]);
    core.insert_picture_native(
        0,
        0,
        1,
        &[],
        bytes,
        1500,
        1500,
        16,
        16,
        "png",
        "그림",
        None,
        None,
    )
    .unwrap();
    let ci = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Picture(_)))
        .unwrap();
    let Control::Picture(picture) = &mut core.document_mut().sections[0].paragraphs[0].controls[ci]
    else {
        panic!()
    };
    picture.common.treat_as_char = true;
    picture.common.instance_id = 771;
    picture.href = Some("https://example.org/그림".into());
    let original =
        serde_json::to_value(&core.document().sections[0].paragraphs[0].controls[ci]).unwrap();
    let id = core.document().bin_data_content[0].id;
    let baseline_hrefs: Vec<_> = [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ]
    .into_iter()
    .map(|bytes| {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        reopened.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|control| {
                if let Control::Picture(picture) = control {
                    Some(picture.href.clone())
                } else {
                    None
                }
            })
            .unwrap()
    })
    .collect();
    let moved = core
        .move_inline_control_native(&source(body(0), ci), &caret(body(1), 1))
        .unwrap();
    assert_eq!(
        serde_json::to_value(
            &core.document().sections[0].paragraphs[1].controls[moved.address.control_index]
        )
        .unwrap(),
        original
    );
    assert_eq!(core.document().bin_data_content[0].id, id);
    assert_eq!(core.document().bin_data_content[0].data.load(), bytes);
    for (bytes, baseline_href) in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ]
    .into_iter()
    .zip(baseline_hrefs)
    {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            reopened.document().bin_data_content[0].data.load(),
            include_bytes!("../../assets/logo/logo-16.png")
        );
        let Control::Picture(picture) = &reopened.document().sections[0].paragraphs[1].controls[0]
        else {
            panic!()
        };
        assert_eq!(picture.image_attr.bin_data_id, id);
        assert_eq!(picture.common.instance_id, 771);
        assert_eq!(picture.href, baseline_href);
    }
}

#[test]
fn moving_out_of_a_cell_updates_its_owner_after_insertion_before_the_parent() {
    let mut core = core(vec![paragraph(
        "",
        vec![(0, table(paragraph("셀", vec![(1, equation(41))])))],
    )]);
    let owner = InlineControlOwner::Body {
        section_index: 0,
        paragraph_index: 0,
        cell_path: vec![(0, 0, 0)],
    };
    let moved = core
        .move_inline_control_native(&source(owner, 0), &caret(body(0), 0))
        .unwrap();
    assert_eq!(moved.address, source(body(0), 0));
    let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[1] else {
        panic!()
    };
    assert_eq!(table.cells[0].paragraphs[0].text, "셀");
    assert!(table.cells[0].paragraphs[0].controls.is_empty());
}

#[test]
fn locked_object_and_locked_parent_reject_without_mutation() {
    let mut core = core(vec![
        paragraph("앞뒤", vec![(1, equation(41))]),
        paragraph("", vec![(0, table(paragraph("셀", vec![])))]),
    ]);
    let Control::Equation(eq) = &mut core.document_mut().sections[0].paragraphs[0].controls[0]
    else {
        panic!()
    };
    eq.common.locked = true;
    unchanged(&mut core, &source(body(0), 0), &caret(body(1), 0));
    let Control::Equation(eq) = &mut core.document_mut().sections[0].paragraphs[0].controls[0]
    else {
        panic!()
    };
    eq.common.locked = false;
    let Control::Table(table) = &mut core.document_mut().sections[0].paragraphs[1].controls[0]
    else {
        panic!()
    };
    table.common.locked = true;
    let owner = InlineControlOwner::Body {
        section_index: 0,
        paragraph_index: 1,
        cell_path: vec![(0, 0, 0)],
    };
    unchanged(&mut core, &source(body(0), 0), &caret(owner, 0));
}

#[test]
fn markpen_object_ranges_move_between_paragraphs_and_preserve_unpaired_metadata() {
    let mut p = paragraph("AB", vec![(1, equation(41))]);
    p.markpen_marks = vec![
        MarkpenMark {
            char_idx: 1,
            color: Some("#FF0000".into()),
            utf16_pos: Some(1),
        },
        MarkpenMark {
            char_idx: 1,
            color: None,
            utf16_pos: Some(9),
        },
        MarkpenMark {
            char_idx: 2,
            color: Some("unknown".into()),
            utf16_pos: Some(10),
        },
    ];
    p.range_tags.push(RangeTag {
        start: 10,
        end: 10,
        tag: 5,
    });
    let mut core = core(vec![paragraph("서문", vec![]), p, paragraph("XY", vec![])]);
    core.move_inline_control_native(&source(body(1), 0), &caret(body(2), 1))
        .unwrap();
    let p = &core.document().sections[0].paragraphs;
    assert_eq!(p[1].markpen_marks.len(), 1);
    assert_eq!(p[1].markpen_marks[0].color.as_deref(), Some("unknown"));
    assert_eq!(p[1].markpen_marks[0].utf16_pos, Some(2));
    assert!(p[1]
        .range_tags
        .iter()
        .any(|tag| tag.start == 2 && tag.end == 2 && tag.tag == 5));
    assert_eq!(p[2].markpen_marks.len(), 2);
    assert_eq!(
        p[2].markpen_marks
            .iter()
            .map(|mark| mark.utf16_pos)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(9)]
    );
    assert_eq!(p[2].markpen_marks[0].color.as_deref(), Some("#FF0000"));
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let p = &reopened.document().sections[0].paragraphs[2];
        assert_eq!(p.markpen_marks.len(), 2);
        assert_eq!(p.markpen_marks[0].color.as_deref(), Some("#FF0000"));
        assert_eq!(p.markpen_marks[0].utf16_pos, Some(1));
        assert_eq!(p.markpen_marks[1].utf16_pos, Some(9));
    }
}

#[test]
fn nonempty_field_insertion_keeps_object_inside_after_text_is_deleted() {
    let field = Control::Field(Field {
        field_type: FieldType::ClickHere,
        field_id: 71,
        ctrl_id: rhwp::parser::tags::FIELD_CLICKHERE,
        command: Field::build_clickhere_command("안내", ""),
        properties: 1 << 15,
        ..Default::default()
    });
    let mut p = paragraph("AB", vec![(0, field)]);
    p.char_count += 8;
    p.field_ranges = vec![FieldRange {
        start_char_idx: 0,
        end_char_idx: 2,
        control_idx: 0,
        inner_slot_count: 0,
        end_field_id: 71,
    }];
    let mut core = core(vec![
        paragraph("서문", vec![]),
        paragraph("", vec![(0, equation(41))]),
        p,
    ]);
    core.move_inline_control_native(&source(body(1), 0), &caret(body(2), 1))
        .unwrap();
    let p = &mut core.document_mut().sections[0].paragraphs[2];
    assert_eq!(p.field_ranges[0].inner_slot_count, 1);
    p.delete_text_at(0, 2);
    assert_eq!(
        (
            p.field_ranges[0].start_char_idx,
            p.field_ranges[0].end_char_idx
        ),
        (0, 0)
    );
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let p = &reopened.document().sections[0].paragraphs[2];
        assert_eq!(p.field_ranges.len(), 1);
        assert_eq!(p.field_ranges[0].inner_slot_count, 1);
        assert_eq!(object_ids(p), vec![41]);
    }
}
