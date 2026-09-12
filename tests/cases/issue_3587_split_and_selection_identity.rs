//! #3587 A3: split moves children; selection paste clones them.
//! API/IR contracts, not independently authored Hancom visual fixtures.
use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, paragraph::Paragraph, table::Table};
use serde_json::Value;
use std::collections::BTreeSet;

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}

fn create(core: &mut DocumentCore) -> (usize, usize) {
    let pi = core.document().sections[0].paragraphs.len() - 1;
    let result: Value =
        serde_json::from_str(&core.create_table_native(0, pi, 0, 4, 2).unwrap()).unwrap();
    (
        result["paraIdx"].as_u64().unwrap() as usize,
        result["controlIdx"].as_u64().unwrap() as usize,
    )
}

fn table(core: &DocumentCore, pi: usize, ci: usize) -> &Table {
    let Control::Table(t) = &core.document().sections[0].paragraphs[pi].controls[ci] else {
        panic!("expected table")
    };
    t
}

fn top_ids(core: &DocumentCore) -> Vec<u32> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            Control::Table(t) => Some(t.common.instance_id),
            _ => None,
        })
        .collect()
}

#[test]
fn split_reserves_existing_ids_instead_of_assuming_hash_uniqueness() {
    let mut core = blank();
    let (pi, ci) = create(&mut core);
    let source_id = table(&core, pi, ci).common.instance_id;
    let formerly_generated = source_id
        .wrapping_mul(0x9e37_79b1)
        .wrapping_add(2)
        .wrapping_add(2 * 0x1000);
    let mut blocker = table(&core, pi, ci).clone();
    blocker.common.instance_id = formerly_generated;
    blocker.raw_ctrl_data[32..36].copy_from_slice(&formerly_generated.to_le_bytes());
    core.document_mut().sections[0].paragraphs.push(Paragraph {
        controls: vec![Control::Table(Box::new(blocker))],
        ..Default::default()
    });
    core.split_table_native(0, pi, ci, 2).unwrap();
    let ids = top_ids(&core);
    assert_eq!(ids.len(), 3);
    assert_eq!(ids.iter().copied().collect::<BTreeSet<_>>().len(), 3);
    assert_eq!(ids[0], source_id);
    assert_eq!(ids[2], formerly_generated);
    let back = table(&core, pi + 2, 0);
    assert_eq!(back.row_count, 2);
    assert_eq!(
        back.common.instance_id.to_le_bytes(),
        back.raw_ctrl_data[32..36]
    );
    for output in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(top_ids(&DocumentCore::from_bytes(&output).unwrap()), ids);
    }
}

#[test]
fn split_preserves_moved_children_and_intervening_blank_paragraph() {
    let mut core = blank();
    let (pi, ci) = create(&mut core);
    let Control::Table(t) = &mut core.document_mut().sections[0].paragraphs[pi].controls[ci] else {
        unreachable!()
    };
    let moved = &mut t.cells[4].paragraphs[0];
    moved.text = "moved, not copied".into();
    moved.raw_header_extra.resize(10, 0);
    moved.raw_header_extra[6..10].copy_from_slice(&98765u32.to_le_bytes());
    let expected = serde_json::to_value(&t.cells[4].paragraphs).unwrap();
    let before_count = core.document().sections[0].paragraphs.len();
    core.split_table_native(0, pi, ci, 2).unwrap();
    assert_eq!(
        core.document().sections[0].paragraphs.len(),
        before_count + 2
    );
    let blank = &core.document().sections[0].paragraphs[pi + 1];
    assert!(blank.text.is_empty() && blank.controls.is_empty());
    assert_eq!(
        serde_json::to_value(&table(&core, pi + 2, 0).cells[0].paragraphs).unwrap(),
        expected
    );
}

#[test]
fn rejected_split_leaves_document_and_events_unchanged() {
    let mut core = blank();
    let (pi, ci) = create(&mut core);
    for at in [0, 4, u16::MAX] {
        let before = core.export_hwp_native().unwrap();
        let events = core.serialize_event_log();
        assert!(core.split_table_native(0, pi, ci, at).is_err());
        assert_eq!(core.export_hwp_native().unwrap(), before);
        assert_eq!(core.serialize_event_log(), events);
    }
}

// Use the actual selection API, with a table-bearing paragraph as its middle
// paragraph. Its neighboring empty paragraphs are intentional clipboard input.
fn selection(core: &mut DocumentCore, source: Table) {
    let start = core.document().sections[0].paragraphs.len();
    core.document_mut().sections[0].paragraphs.extend([
        Paragraph::default(),
        Paragraph {
            controls: vec![Control::Table(Box::new(source))],
            ..Default::default()
        },
        Paragraph::default(),
    ]);
    // Refresh composed/dirty caches after constructing this internal fixture.
    *core = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    core.copy_selection_native(0, start, 0, start + 2, 0)
        .unwrap();
}

fn check_selection_route(route: u8) {
    let mut core = blank();
    let (pi, ci) = create(&mut core);
    let mut source = table(&core, pi, ci).clone();
    source.common.instance_id = 81234;
    source.raw_ctrl_data[32..36].copy_from_slice(&81234u32.to_le_bytes());
    selection(&mut core, source);
    let source_pi = core.document().sections[0].paragraphs.len() - 2;
    let original = serde_json::to_value(table(&core, source_pi, 0)).unwrap();
    let mut inserted = Vec::new();
    for _ in 0..2 {
        if route == 0 {
            let dst = core.document().sections[0].paragraphs.len();
            core.split_paragraph_native(0, dst - 1, 0).unwrap();
            core.paste_internal_native(0, dst, 0).unwrap();
            inserted.push(table(&core, dst + 1, 0).common.instance_id);
        } else {
            let cell_pi = table(&core, pi, ci).cells[0].paragraphs.len() - 1;
            if route == 1 {
                core.paste_internal_in_cell_native(0, pi, ci, 0, cell_pi, 0)
                    .unwrap();
            } else {
                core.paste_internal_in_cell_by_path_native(0, pi, &[(ci, 0, cell_pi)], 0)
                    .unwrap();
            }
            let Control::Table(copy) =
                &table(&core, pi, ci).cells[0].paragraphs[cell_pi + 1].controls[0]
            else {
                panic!("copied table")
            };
            inserted.push(copy.common.instance_id);
            assert_eq!(
                copy.common.instance_id.to_le_bytes(),
                copy.raw_ctrl_data[32..36]
            );
        }
    }
    assert!(inserted.iter().all(|id| *id != 0 && *id != 81234));
    assert_ne!(inserted[0], inserted[1]);
    assert_eq!(
        serde_json::to_value(table(&core, source_pi, 0)).unwrap(),
        original
    );
}

#[test]
fn body_selection_paste_allocates_control_identities() {
    check_selection_route(0);
}

#[test]
fn cell_selection_paste_allocates_control_identities() {
    check_selection_route(1);
}

#[test]
fn path_selection_paste_allocates_control_identities() {
    check_selection_route(2);
}
