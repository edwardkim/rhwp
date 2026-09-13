//! Native row-copy contracts. Synthetic tables are not Hancom visual oracles.
use rhwp::{
    document_core::{
        DocumentCore, ParagraphBlockLimits, ParagraphBlockPathStep as Step, RepeatTableRowsRequest,
        TemplateBinding, TemplateFillTarget,
    },
    model::{
        control::{Control, Field, FieldType},
        paragraph::{FieldRange, Paragraph},
        table::{Cell, Table, TableZone},
    },
};
use std::collections::{BTreeMap, BTreeSet};

fn para(s: &str) -> Paragraph {
    let mut p = Paragraph::default();
    p.insert_text_at(0, s);
    p
}
fn core() -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    let mut t = Table {
        row_count: 4,
        col_count: 2,
        repeat_header: true,
        ..Default::default()
    };
    t.common.instance_id = 100;
    t.common.width = 12000;
    t.common.height = 4000;
    for row in 0..4 {
        for col in 0..2 {
            t.cells.push(Cell {
                row,
                col,
                row_span: 1,
                col_span: 1,
                width: 6000,
                height: 1000,
                is_header: row == 0,
                paragraphs: vec![para(&format!("{row},{col}")), para(""), para("tail")],
                ..Default::default()
            });
        }
    }
    t.zones.push(TableZone {
        start_row: 1,
        end_row: 1,
        start_col: 0,
        end_col: 1,
        ..Default::default()
    });
    t.rebuild_grid();
    c.document_mut().sections[0].paragraphs = vec![
        para("before"),
        Paragraph {
            controls: vec![Control::Table(Box::new(t))],
            char_count: 9,
            control_mask: 1 << 11,
            has_para_text: true,
            ..Default::default()
        },
        para("after"),
    ];
    c
}
fn table(c: &DocumentCore) -> &Table {
    let Control::Table(t) = &c.document().sections[0].paragraphs[1].controls[0] else {
        panic!("table")
    };
    t
}
fn table_mut(c: &mut DocumentCore) -> &mut Table {
    let Control::Table(t) = &mut c.document_mut().sections[0].paragraphs[1].controls[0] else {
        panic!("table")
    };
    t
}
fn request() -> RepeatTableRowsRequest {
    RepeatTableRowsRequest {
        section_index: 0,
        paragraph_index: 1,
        control_index: 0,
        start_row: 1,
        end_row: 2,
        insert_before: 2,
        bindings: vec![TemplateBinding {
            key: "value".into(),
            target: TemplateFillTarget::TextRange {
                path: vec![
                    Step::Paragraph(0),
                    Step::Control(0),
                    Step::Cell(0),
                    Step::Paragraph(0),
                ],
                start: 0,
                end: 3,
            },
        }],
        records: vec![
            BTreeMap::from([("value".into(), "one😀".into())]),
            BTreeMap::from([("value".into(), "two\nline".into())]),
        ],
        limits: ParagraphBlockLimits::default(),
    }
}
fn state(c: &DocumentCore) -> (String, String, String) {
    (
        format!("{:?}", c.document()),
        c.serialize_event_log(),
        format!("{:?}", c.get_clipboard_text_native()),
    )
}
fn reject(c: &mut DocumentCore, r: &RepeatTableRowsRequest, part: &str) {
    let before = state(c);
    let e = c.repeat_and_fill_table_rows_native(r).unwrap_err();
    assert!(e.to_string().contains(part), "{e}");
    assert_eq!(state(c), before);
}

#[test]
fn complete_rows_are_filled_without_flattening_or_replacing_original_identity() {
    let mut c = core();
    let before = serde_json::to_value(table(&c)).unwrap();
    let result = c.repeat_and_fill_table_rows_native(&request()).unwrap();
    let t = table(&c);
    assert_eq!(t.row_count, 6);
    assert_eq!(result.inserted_rows, 2..4);
    assert_eq!(result.source_rows_after, 1..2);
    assert_eq!(t.common.instance_id, 100);
    assert_eq!(t.common.width, 12000);
    assert_eq!(t.common.height, 6000);
    assert!(t.repeat_header);
    assert_eq!(t.cells[4].paragraphs[0].text, "one😀");
    assert_eq!(t.cells[6].paragraphs[0].text, "two\nline");
    assert_eq!(t.cells[4].paragraphs.len(), 3);
    assert!(t.cells[4].paragraphs[1].text.is_empty());
    assert_eq!(t.cells[4].paragraphs[2].text, "tail");
    assert_eq!(
        serde_json::to_value(&t.cells[..4]).unwrap(),
        serde_json::to_value(&before["cells"].as_array().unwrap()[..4]).unwrap()
    );
    assert_eq!(t.cells[8].paragraphs[0].text, "2,0");
    assert_eq!(
        t.zones.iter().map(|z| z.start_row).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(t.row_sizes, vec![2; 6]);
    assert_eq!(t.cell_index_at(5, 1), Some(11));
    assert_eq!(c.document().sections[0].paragraphs.len(), 3);
    assert_eq!(c.document().sections[0].paragraphs[0].text, "before");
    assert_eq!(c.document().sections[0].paragraphs[2].text, "after");
    assert!(result.copies[0].mappings.iter().any(|m| m.source
        == vec![
            Step::Paragraph(1),
            Step::Control(0),
            Step::Cell(2),
            Step::Paragraph(0)
        ]
        && m.destination
            == vec![
                Step::Paragraph(1),
                Step::Control(0),
                Step::Cell(4),
                Step::Paragraph(0)
            ]));
    assert_eq!(
        c.serialize_event_log().matches("TableRowInserted").count(),
        1
    );
}

#[test]
fn source_before_insertion_shifts_original_rows_and_zones_once() {
    let mut c = core();
    let mut r = request();
    r.insert_before = 1;
    let result = c.repeat_and_fill_table_rows_native(&r).unwrap();
    assert_eq!(result.source_rows_after, 3..4);
    let t = table(&c);
    assert_eq!(t.cells[2].paragraphs[0].text, "one😀");
    assert_eq!(t.cells[6].paragraphs[0].text, "1,0");
    assert_eq!(
        t.zones.iter().map(|z| z.start_row).collect::<Vec<_>>(),
        vec![3, 1, 2]
    );
    assert!(t.cells[0].is_header);
}

#[test]
fn internally_closed_vertical_and_horizontal_merges_are_retained() {
    let mut c = core();
    let t = table_mut(&mut c);
    t.zones.clear();
    t.cells.retain(|v| !(v.row == 2 && v.col == 0));
    t.cells[2].row_span = 2;
    // Source is two complete rows; left cell spans both, right cells remain separate.
    let mut r = request();
    r.end_row = 3;
    r.insert_before = 3;
    r.records.truncate(1);
    c.repeat_and_fill_table_rows_native(&r).unwrap();
    let t = table(&c);
    assert_eq!(t.row_count, 6);
    assert_eq!(t.cell_at(3, 0).unwrap().row_span, 2);
    assert_eq!(t.cell_at(4, 0).unwrap().row, 3);
    let mut c = core();
    let t = table_mut(&mut c);
    t.cells.remove(3);
    t.cells[2].col_span = 2;
    t.cells[2].width = 12000;
    c.repeat_and_fill_table_rows_native(&request()).unwrap();
    let t = table(&c);
    assert_eq!(t.cell_at(2, 1).unwrap().col, 0);
    assert_eq!(t.cell_at(2, 0).unwrap().col_span, 2);
}

#[test]
fn crossing_merges_titles_zones_and_invalid_grids_are_rejected_without_mutation() {
    let mut c = core();
    table_mut(&mut c).cells[2].row_span = 2;
    reject(&mut c, &request(), "merge crosses");
    let mut c = core();
    table_mut(&mut c).cells[2].is_header = true;
    reject(&mut c, &request(), "title cells");
    let mut c = core();
    table_mut(&mut c).zones[0].end_row = 2;
    reject(&mut c, &request(), "zone crosses");
    let mut c = core();
    table_mut(&mut c).cells[2].row_span = 0;
    reject(&mut c, &request(), "zero span");
    let mut c = core();
    table_mut(&mut c).cells[3].col = 0;
    reject(&mut c, &request(), "overlapping");
    let mut c = core();
    table_mut(&mut c).cells.remove(3);
    reject(&mut c, &request(), "uncovered");
}

#[test]
fn crossing_destination_merge_and_zone_are_rejected_even_outside_source() {
    let mut c = core();
    table_mut(&mut c).cells[4].row_span = 2;
    let mut r = request();
    r.insert_before = 3;
    reject(&mut c, &r, "merge crosses");
    let mut c = core();
    table_mut(&mut c).zones.push(TableZone {
        start_row: 2,
        end_row: 3,
        start_col: 0,
        end_col: 1,
        ..Default::default()
    });
    reject(&mut c, &r, "zone crosses");
    let mut c = core();
    r.end_row = 3;
    r.insert_before = 2;
    reject(&mut c, &r, "inside source");
}

#[test]
fn no_op_addresses_bad_last_record_and_budgets_keep_live_state() {
    let mut c = core();
    let mut r = request();
    r.records.clear();
    let before = state(&c);
    assert!(c
        .repeat_and_fill_table_rows_native(&r)
        .unwrap()
        .copies
        .is_empty());
    assert_eq!(state(&c), before);
    r.control_index = 9;
    reject(&mut c, &r, "top-level");
    let mut r = request();
    r.records[1].clear();
    reject(&mut c, &r, "missing or extra");
    let mut r = request();
    r.limits.max_structure_bytes = 1;
    reject(&mut c, &r, "budget");
    let mut r = request();
    r.limits.max_copies = 1;
    reject(&mut c, &r, "budget");
    let mut r = request();
    r.limits.max_copies = 1001;
    reject(&mut c, &r, "maxCopies");
    let mut r = request();
    r.bindings.push(r.bindings[0].clone());
    reject(&mut c, &r, "duplicated");
}

#[test]
fn cell_owned_fields_get_fresh_ids_while_source_and_table_ids_survive() {
    let mut c = core();
    let p = &mut table_mut(&mut c).cells[2].paragraphs[0];
    p.controls.push(Control::Field(Field {
        field_type: FieldType::ClickHere,
        field_id: 90,
        ctrl_id: u32::from_le_bytes(*b"klc%"),
        ..Default::default()
    }));
    p.field_ranges.push(FieldRange {
        start_char_idx: 0,
        end_char_idx: 3,
        control_idx: 0,
        ..Default::default()
    });
    p.char_offsets = vec![8, 9, 10];
    p.char_count = 20;
    let mut r = request();
    r.bindings[0].target = TemplateFillTarget::Field {
        path: vec![
            Step::Paragraph(0),
            Step::Control(0),
            Step::Cell(0),
            Step::Paragraph(0),
        ],
        field_range_index: 0,
    };
    c.repeat_and_fill_table_rows_native(&r).unwrap();
    let t = table(&c);
    let mut ids = BTreeSet::new();
    for row in [1, 2, 3] {
        let p = &t.cell_at(row, 0).unwrap().paragraphs[0];
        let Control::Field(f) = &p.controls[0] else {
            panic!("field")
        };
        assert!(ids.insert(f.field_id));
        if row == 1 {
            assert_eq!(f.field_id, 90);
        }
        assert_eq!(p.field_ranges[0].end_char_idx, p.text.chars().count());
    }
    assert_eq!(t.common.instance_id, 100);
}

#[test]
fn actual_labnote_data_row_round_trips_hwp_and_hwpx() {
    let mut c =
        DocumentCore::from_bytes(&std::fs::read("samples/rnote/labnote-001.hwp").unwrap()).unwrap();
    let Control::Table(t) = &c.document().sections[0].paragraphs[12].controls[1] else {
        panic!("required table")
    };
    let source = &t.cells[5];
    assert!(!source.is_header);
    assert!(source.paragraphs[0].text.is_empty());
    let row = source.row;
    let rows = t.row_count;
    let id = t.common.instance_id;
    let cell_count = t.cells.len();
    let mut r = request();
    r.paragraph_index = 12;
    r.control_index = 1;
    r.start_row = row;
    r.end_row = row + source.row_span;
    r.insert_before = r.end_row;
    r.bindings[0].target = TemplateFillTarget::TextRange {
        path: vec![
            Step::Paragraph(0),
            Step::Control(0),
            Step::Cell(0),
            Step::Paragraph(0),
        ],
        start: 0,
        end: 0,
    };
    c.repeat_and_fill_table_rows_native(&r).unwrap();
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let Control::Table(t) = &reopened.document().sections[0].paragraphs[12].controls[1] else {
            panic!("saved table")
        };
        assert_eq!(t.common.instance_id, id);
        assert_eq!(t.row_count, rows + 2 * (r.end_row - r.start_row));
        assert!(t.cells.len() > cell_count);
        assert!(t.cell_at(row, 0).unwrap().paragraphs[0].text.is_empty());
        assert_eq!(
            t.cell_at(r.insert_before, 0).unwrap().paragraphs[0].text,
            "one😀"
        );
    }
}
