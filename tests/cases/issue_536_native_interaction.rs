//! Typed host boundary checks, not a new layout/visual oracle.

use rhwp::document_core::native_interaction::{
    NativeCellPathEntry, NativeInteractionError, NativeTextPosition, NativeTextSelection,
    NativeVerticalDirection, NATIVE_CELL_PATH_MAX_DEPTH,
};
use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, paragraph::Paragraph};

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}

fn body(paragraph: u32, scalar_offset: u32) -> NativeTextPosition {
    NativeTextPosition {
        section: 0,
        paragraph,
        scalar_offset,
        cell_path: Vec::new(),
    }
}

fn nested_cells() -> DocumentCore {
    let mut core = blank();
    core.paste_html_native(0, 0, 0,
        "<table><tr><td><p>outer</p><table><tr><td><p>가😀나</p><p>second</p></td><td><p>sibling</p></td></tr></table></td></tr></table>",
    ).unwrap();
    core
}

fn find_text(core: &DocumentCore, text: &str) -> NativeTextPosition {
    fn descend(paragraph: &Paragraph, text: &str, path: &mut Vec<NativeCellPathEntry>) -> bool {
        if paragraph.text == text {
            return true;
        }
        for (control, item) in paragraph.controls.iter().enumerate() {
            if let Control::Table(table) = item {
                for (cell, item) in table.cells.iter().enumerate() {
                    for (index, paragraph) in item.paragraphs.iter().enumerate() {
                        path.push(NativeCellPathEntry {
                            control: control as u32,
                            cell: cell as u32,
                            paragraph: index as u32,
                        });
                        if descend(paragraph, text, path) {
                            return true;
                        }
                        path.pop();
                    }
                }
            }
        }
        false
    }
    for (section, item) in core.document().sections.iter().enumerate() {
        for (paragraph, item) in item.paragraphs.iter().enumerate() {
            let mut cell_path = Vec::new();
            if descend(item, text, &mut cell_path) {
                return NativeTextPosition {
                    section: section as u32,
                    paragraph: paragraph as u32,
                    scalar_offset: 0,
                    cell_path,
                };
            }
        }
    }
    panic!("missing fixture text {text}");
}

#[test]
fn scalar_positions_do_not_accept_utf16_or_byte_offsets() {
    let mut core = blank();
    core.insert_text_native(0, 0, 0, "가A😀나").unwrap();
    for offset in 0..=4 {
        core.validate_native_text_position(&body(0, offset))
            .unwrap();
    }
    // Four Unicode scalars, five UTF-16 code units and eleven UTF-8 bytes.
    for offset in [5, 11, u32::MAX] {
        assert!(matches!(
            core.validate_native_text_position(&body(0, offset)),
            Err(NativeInteractionError::InvalidPosition(_))
        ));
    }
    let mut invalid = body(0, 0);
    invalid.section = u32::MAX;
    assert!(core.validate_native_text_position(&invalid).is_err());
    assert!(core
        .validate_native_text_position(&body(u32::MAX, 0))
        .is_err());
}

#[test]
fn body_caret_hit_and_reversed_selection_reuse_existing_geometry() {
    let mut core = blank();
    core.insert_text_native(0, 0, 0, "가A😀나").unwrap();
    let rect = core.native_caret_rect(&body(0, 0)).unwrap();
    let legacy: serde_json::Value =
        serde_json::from_str(&core.get_cursor_rect_native(0, 0, 0).unwrap()).unwrap();
    assert_eq!(rect.x, legacy["x"].as_f64().unwrap());
    assert_eq!(rect.y, legacy["y"].as_f64().unwrap());
    assert_eq!(rect.height, legacy["height"].as_f64().unwrap());
    let hit = core
        .native_hit_test(rect.page, rect.x + 0.01, rect.y + rect.height / 2.0)
        .unwrap();
    assert_eq!(hit.position, body(0, 0));
    assert!(hit.caret.is_some());
    let selection = NativeTextSelection {
        anchor: body(0, 1),
        focus: body(0, 3),
    };
    let forward = core.native_selection_rects(&selection).unwrap();
    assert!(!forward.is_empty());
    assert!(forward
        .iter()
        .all(|rect| rect.bounds.width > 0.0 && rect.bounds.height > 0.0));
    assert_eq!(
        forward,
        core.native_selection_rects(&NativeTextSelection {
            anchor: selection.focus,
            focus: selection.anchor,
        })
        .unwrap()
    );
    assert!(core
        .native_selection_rects(&NativeTextSelection {
            anchor: body(0, 2),
            focus: body(0, 2),
        })
        .unwrap()
        .is_empty());
}

#[test]
fn nested_cell_caret_hit_and_selection_preserve_the_complete_address() {
    let core = nested_cells();
    let position = find_text(&core, "가😀나");
    assert_eq!(position.cell_path.len(), 2);
    core.check_native_interaction_support(&position).unwrap();
    let caret = core.native_caret_rect(&position).unwrap();
    let hit = core
        .native_hit_test(caret.page, caret.x + 0.01, caret.y + caret.height / 2.0)
        .unwrap();
    assert_eq!(hit.position, position);
    let mut focus = position.clone();
    focus.scalar_offset = 3;
    let rects = core
        .native_selection_rects(&NativeTextSelection {
            anchor: position.clone(),
            focus: focus.clone(),
        })
        .unwrap();
    assert!(!rects.is_empty());
    focus.scalar_offset = 4; // UTF-16 length is not a valid scalar boundary.
    assert!(core.validate_native_text_position(&focus).is_err());
    let second = find_text(&core, "second");
    assert!(!core
        .native_selection_rects(&NativeTextSelection {
            anchor: position.clone(),
            focus: second,
        })
        .unwrap()
        .is_empty());
    let sibling = find_text(&core, "sibling");
    assert!(matches!(
        core.native_selection_rects(&NativeTextSelection {
            anchor: position,
            focus: sibling,
        }),
        Err(NativeInteractionError::Unsupported(_))
    ));
}

#[test]
fn malformed_cell_paths_are_not_flattened_or_defaulted() {
    let core = nested_cells();
    let valid = find_text(&core, "가😀나");
    for field in 0..3 {
        let mut invalid = valid.clone();
        let leaf = invalid.cell_path.last_mut().unwrap();
        match field {
            0 => leaf.control = u32::MAX,
            1 => leaf.cell = u32::MAX,
            _ => leaf.paragraph = u32::MAX,
        }
        assert!(core.validate_native_text_position(&invalid).is_err());
    }
    let mut deep = valid;
    deep.cell_path.resize(
        NATIVE_CELL_PATH_MAX_DEPTH + 1,
        NativeCellPathEntry {
            control: 0,
            cell: 0,
            paragraph: 0,
        },
    );
    assert!(matches!(
        core.validate_native_text_position(&deep),
        Err(NativeInteractionError::InvalidInput(_))
    ));
    for json in [
        r#"{"section":0,"paragraph":0,"scalarOffset":0}"#,
        r#"{"section":0,"paragraph":0,"scalarOffset":1.5,"cellPath":[]}"#,
        r#"{"section":-1,"paragraph":0,"scalarOffset":0,"cellPath":[]}"#,
        r#"{"section":0,"paragraph":0,"scalarOffset":0,"cellPath":[{"control":0,"cell":0}]}"#,
        r#"{"section":0,"paragraph":0,"scalarOffset":0,"cellPath":[],"utf16Offset":0}"#,
    ] {
        assert!(
            serde_json::from_str::<NativeTextPosition>(json).is_err(),
            "{json}"
        );
    }
}

#[test]
fn inline_object_affinity_is_explicitly_unsupported() {
    let mut core = blank();
    core.create_table_native(0, 0, 0, 1, 1).unwrap();
    let mut document = core.document().clone();
    let (paragraph, table) = document.sections[0]
        .paragraphs
        .iter_mut()
        .enumerate()
        .find_map(|(index, paragraph)| {
            paragraph.controls.iter_mut().find_map(|control| {
                if let Control::Table(table) = control {
                    Some((index as u32, table))
                } else {
                    None
                }
            })
        })
        .unwrap();
    table.common.treat_as_char = true;
    core.set_document(document);
    let position = body(paragraph, 0);
    core.validate_native_text_position(&position).unwrap();
    assert!(matches!(
        core.check_native_interaction_support(&position),
        Err(NativeInteractionError::Unsupported(_))
    ));
    assert!(matches!(
        core.native_caret_rect(&position),
        Err(NativeInteractionError::Unsupported(_))
    ));
}

#[test]
fn vertical_navigation_is_typed_and_preserves_nested_paths() {
    let mut core = blank();
    core.insert_text_native(0, 0, 0, "abc").unwrap();
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    core.insert_text_native(0, 1, 0, "def").unwrap();
    let down = core
        .native_move_vertical(&body(0, 0), NativeVerticalDirection::Down, None)
        .unwrap();
    assert_eq!(down.position.paragraph, 1);
    assert!(down.position.cell_path.is_empty());
    assert!(down.caret.is_some());
    let up = core
        .native_move_vertical(
            &down.position,
            NativeVerticalDirection::Up,
            Some(down.preferred_x),
        )
        .unwrap();
    assert_eq!(up.position.paragraph, 0);

    let core = nested_cells();
    let position = find_text(&core, "가😀나");
    let down = core
        .native_move_vertical(&position, NativeVerticalDirection::Down, None)
        .unwrap();
    let second = find_text(&core, "second");
    assert_eq!(down.position.cell_path, second.cell_path);
    assert_eq!(down.position.paragraph, position.paragraph);
    core.validate_native_text_position(&down.position).unwrap();
}

#[test]
fn invalid_geometry_requests_fail_before_layout_queries() {
    let core = blank();
    for coordinate in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            core.native_hit_test(0, coordinate, 0.0),
            Err(NativeInteractionError::InvalidInput(_))
        ));
        assert!(matches!(
            core.native_hit_test(0, 0.0, coordinate),
            Err(NativeInteractionError::InvalidInput(_))
        ));
        assert!(matches!(
            core.native_move_vertical(&body(0, 0), NativeVerticalDirection::Down, Some(coordinate)),
            Err(NativeInteractionError::InvalidInput(_))
        ));
    }
    assert!(core.native_hit_test(core.page_count(), 0.0, 0.0).is_err());
    assert!(core
        .native_move_vertical(&body(0, 0), NativeVerticalDirection::Up, Some(-1.0))
        .is_err());
}
