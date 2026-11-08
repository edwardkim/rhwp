//! Semantic/Unicode/native-query contracts, not OS accessibility registration
//! or a screen-reader/Hancom visual oracle.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use rhwp::native_host::accessibility::{
    native_scalar_to_utf16, native_utf16_to_scalar, NativeAccessibilityGeometryStatus,
    NativeAccessibilityLimitation, NativeAccessibilityRange, NativeAccessibilityRequest,
    NativeAccessibilityRole, MAX_ACCESSIBILITY_DEPTH, MAX_ACCESSIBILITY_NODES,
    MAX_ACCESSIBILITY_TEXT_BYTES, MAX_ACCESSIBILITY_WORK_ITEMS,
};
use rhwp::native_host::editor::NativeEditor;
use rhwp::native_host::{NativeDocumentSession, NativeHostError};
use rhwp::DocumentCore;

fn core(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).unwrap();
    }
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

fn multi_paragraph() -> DocumentCore {
    let mut core = core("firstsecondthird");
    core.split_paragraph_native(0, 0, 5, None).unwrap();
    core.split_paragraph_native(0, 1, 6, None).unwrap();
    core
}

fn nested() -> NativeDocumentSession {
    let mut core = core("");
    core.paste_html_native(0, 0, 0,
        "<table><tr><td><p>outer</p><table><tr><td><p>가😀나</p><p>second</p></td><td><p>sibling</p></td></tr></table></td></tr></table>",
    ).unwrap();
    NativeDocumentSession::from_core(core)
}

#[test]
fn issue_536_native_accessibility_fast_snapshot_is_read_only_and_scalar_addressed() {
    let session = NativeDocumentSession::from_core(core("A😀가"));
    let before = format!("{:?}", session.document().document());
    let selection = NativeTextSelection {
        anchor: body(0, 1),
        focus: body(0, 2),
    };
    let snapshot = session
        .accessibility_snapshot(&NativeAccessibilityRequest::default(), Some(&selection))
        .unwrap();
    assert_eq!(snapshot.scope, "bodyAndTableCellText");
    assert_eq!(snapshot.revision, session.revision());
    assert_eq!(snapshot.font_generation, session.font_generation());
    assert!(snapshot.complete && !snapshot.truncated);
    assert_eq!(snapshot.nodes.len(), 1);
    assert_eq!(snapshot.nodes[0].text, "A😀가");
    assert_eq!(snapshot.nodes[0].start, body(0, 0));
    assert_eq!(snapshot.nodes[0].end, body(0, 3));
    assert_eq!(snapshot.nodes[0].key, "r0:s0:p0");
    assert_eq!(
        snapshot.nodes[0].role,
        NativeAccessibilityRole::BodyParagraph
    );
    assert!(snapshot.nodes[0].geometry.is_none());
    assert_eq!(snapshot.selection.as_ref(), Some(&selection));
    assert!(snapshot.selection_geometry.is_none());
    assert_eq!(format!("{:?}", session.document().document()), before);
    assert_eq!(session.revision(), 0);
}

#[test]
fn issue_536_native_accessibility_nested_cells_keep_full_paths_and_structural_order() {
    let session = nested();
    let snapshot = session
        .accessibility_snapshot(&NativeAccessibilityRequest::default(), None)
        .unwrap();
    let texts: Vec<_> = snapshot
        .nodes
        .iter()
        .map(|node| node.text.as_str())
        .collect();
    let index = |text| {
        texts
            .iter()
            .position(|candidate| *candidate == text)
            .unwrap()
    };
    assert!(index("outer") < index("가😀나"));
    assert!(index("가😀나") < index("second"));
    assert!(index("second") < index("sibling"));
    let node = &snapshot.nodes[index("가😀나")];
    assert_eq!(node.role, NativeAccessibilityRole::TableCellParagraph);
    assert_eq!(node.start.cell_path.len(), 2);
    assert_eq!(node.end.scalar_offset, 3);
    for node in &snapshot.nodes {
        session
            .document()
            .validate_native_text_position(&node.start)
            .unwrap();
        session
            .document()
            .validate_native_text_position(&node.end)
            .unwrap();
    }
    let unique: std::collections::HashSet<_> =
        snapshot.nodes.iter().map(|node| &node.key).collect();
    assert_eq!(unique.len(), snapshot.nodes.len());
}

#[test]
fn issue_536_native_accessibility_node_and_utf8_text_limits_report_real_truncation() {
    let session = NativeDocumentSession::from_core(multi_paragraph());
    let request = NativeAccessibilityRequest {
        max_nodes: 1,
        ..Default::default()
    };
    let snapshot = session.accessibility_snapshot(&request, None).unwrap();
    assert_eq!(snapshot.nodes.len(), 1);
    assert!(!snapshot.complete && snapshot.truncated);
    assert!(snapshot
        .limitations
        .contains(&NativeAccessibilityLimitation::NodeLimit));
    let session = NativeDocumentSession::from_core(core("A😀가"));
    for (budget, expected, scalars) in [(4, "A", 1), (5, "A😀", 2), (8, "A😀가", 3)] {
        let request = NativeAccessibilityRequest {
            max_text_bytes: budget,
            ..Default::default()
        };
        let snapshot = session.accessibility_snapshot(&request, None).unwrap();
        assert_eq!(snapshot.nodes[0].text, expected);
        assert_eq!(snapshot.nodes[0].end.scalar_offset, scalars);
        assert_eq!(snapshot.nodes[0].text_complete, budget == 8);
        assert_eq!(snapshot.truncated, budget != 8);
        assert!(
            snapshot
                .nodes
                .iter()
                .map(|node| node.text.len())
                .sum::<usize>()
                <= budget
        );
    }
}

#[test]
fn issue_536_native_accessibility_range_is_half_open_and_includes_only_selected_subtrees() {
    let session = NativeDocumentSession::from_core(multi_paragraph());
    let mut request = NativeAccessibilityRequest {
        range: Some(NativeAccessibilityRange {
            section: 0,
            start_paragraph: 1,
            end_paragraph: 2,
        }),
        ..Default::default()
    };
    let snapshot = session.accessibility_snapshot(&request, None).unwrap();
    assert_eq!(snapshot.nodes.len(), 1);
    assert_eq!(snapshot.nodes[0].text, "second");
    assert_eq!(snapshot.nodes[0].start.paragraph, 1);
    assert!(snapshot.complete);
    request.range.as_mut().unwrap().end_paragraph = 1;
    assert!(session
        .accessibility_snapshot(&request, None)
        .unwrap()
        .nodes
        .is_empty());
    request.range.as_mut().unwrap().end_paragraph = 99;
    assert!(session.accessibility_snapshot(&request, None).is_err());
    request.range = Some(NativeAccessibilityRange {
        section: 99,
        start_paragraph: 0,
        end_paragraph: 0,
    });
    assert!(session.accessibility_snapshot(&request, None).is_err());
}

#[test]
fn issue_536_native_accessibility_depth_and_control_walk_budgets_are_reported() {
    let session = nested();
    let request = NativeAccessibilityRequest {
        max_depth: 0,
        ..Default::default()
    };
    let snapshot = session.accessibility_snapshot(&request, None).unwrap();
    assert!(snapshot
        .nodes
        .iter()
        .all(|node| node.start.cell_path.is_empty()));
    assert!(snapshot.truncated && !snapshot.complete);
    assert!(snapshot
        .limitations
        .contains(&NativeAccessibilityLimitation::DepthLimit));

    let mut core = core("body");
    let mut document = core.document().clone();
    document.sections[0].paragraphs[0].controls.extend(
        (0..=MAX_ACCESSIBILITY_WORK_ITEMS)
            .map(|_| rhwp::model::control::Control::Bookmark(Default::default())),
    );
    core.set_document(document);
    let snapshot = NativeDocumentSession::from_core(core)
        .accessibility_snapshot(&NativeAccessibilityRequest::default(), None)
        .unwrap();
    assert!(snapshot.truncated);
    assert!(snapshot
        .limitations
        .contains(&NativeAccessibilityLimitation::TraversalLimit));
    assert_eq!(snapshot.nodes[0].text, "body");
}

#[test]
fn issue_536_native_accessibility_omitted_header_note_and_inline_semantics_are_not_complete() {
    use rhwp::model::{control::Control, footnote::Footnote, header_footer::Header};
    let mut core = core("body");
    let mut hidden = core.document().sections[0].paragraphs[0].clone();
    hidden.controls.clear();
    let mut document = core.document().clone();
    document.sections[0].paragraphs[0].controls.extend([
        Control::Header(Box::new(Header {
            paragraphs: vec![hidden.clone()],
            ..Default::default()
        })),
        Control::Footnote(Box::new(Footnote {
            paragraphs: vec![hidden],
            ..Default::default()
        })),
    ]);
    core.set_document(document);
    let snapshot = NativeDocumentSession::from_core(core)
        .accessibility_snapshot(&NativeAccessibilityRequest::default(), None)
        .unwrap();
    assert!(!snapshot.complete);
    assert!(
        !snapshot.truncated,
        "unsupported semantics differ from budget truncation"
    );
    assert_eq!(snapshot.nodes.len(), 1);
    for limitation in [
        NativeAccessibilityLimitation::HeadersFooters,
        NativeAccessibilityLimitation::Notes,
        NativeAccessibilityLimitation::InlineObjects,
    ] {
        assert!(snapshot.limitations.contains(&limitation));
    }
}

#[test]
fn issue_536_native_accessibility_geometry_reuses_core_and_zero_query_budget_stays_honest() {
    let session = NativeDocumentSession::from_core(core("A😀가"));
    let before = format!("{:?}", session.document().document());
    let selection = NativeTextSelection {
        anchor: body(0, 1),
        focus: body(0, 3),
    };
    let request = NativeAccessibilityRequest {
        geometry: true,
        ..Default::default()
    };
    let snapshot = session
        .accessibility_snapshot(&request, Some(&selection))
        .unwrap();
    let node = snapshot.nodes[0].geometry.as_ref().unwrap();
    assert_eq!(
        node.start_caret,
        Some(session.document().native_caret_rect(&body(0, 0)).unwrap())
    );
    assert_eq!(
        node.end_caret,
        Some(session.document().native_caret_rect(&body(0, 3)).unwrap())
    );
    let selected = snapshot.selection_geometry.unwrap();
    assert_eq!(selected.status, NativeAccessibilityGeometryStatus::Complete);
    assert_eq!(
        selected.rectangles,
        session
            .document()
            .native_selection_rects(&selection)
            .unwrap()
    );
    assert!(!selected.rectangles.is_empty());
    assert_eq!(format!("{:?}", session.document().document()), before);
    assert_eq!(session.revision(), 0);
    let limited = NativeAccessibilityRequest {
        max_geometry_queries: 0,
        ..request
    };
    let snapshot = session
        .accessibility_snapshot(&limited, Some(&selection))
        .unwrap();
    assert!(snapshot.truncated && !snapshot.complete);
    assert!(snapshot.nodes[0]
        .geometry
        .as_ref()
        .unwrap()
        .start_caret
        .is_none());
    assert!(snapshot
        .limitations
        .contains(&NativeAccessibilityLimitation::GeometryLimit));
}

#[test]
fn issue_536_native_accessibility_selection_geometry_preserves_supported_container_boundaries() {
    let session = NativeDocumentSession::from_core(multi_paragraph());
    let selection = NativeTextSelection {
        anchor: body(0, 1),
        focus: body(1, 2),
    };
    let request = NativeAccessibilityRequest {
        geometry: true,
        ..Default::default()
    };
    let snapshot = session
        .accessibility_snapshot(&request, Some(&selection))
        .unwrap();
    assert_eq!(snapshot.selection.as_ref(), Some(&selection));
    let geometry = snapshot.selection_geometry.unwrap();
    assert_eq!(geometry.status, NativeAccessibilityGeometryStatus::Complete);
    assert_eq!(
        geometry.rectangles,
        session
            .document()
            .native_selection_rects(&selection)
            .unwrap()
    );
    assert!(!geometry.rectangles.is_empty());

    let session = nested();
    let snapshot = session
        .accessibility_snapshot(&NativeAccessibilityRequest::default(), None)
        .unwrap();
    let at = |text| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.text == text)
            .unwrap()
            .start
            .clone()
    };
    let same_cell = NativeTextSelection {
        anchor: at("가😀나"),
        focus: at("second"),
    };
    let supported = session
        .accessibility_snapshot(&request, Some(&same_cell))
        .unwrap();
    assert_eq!(
        supported.selection_geometry.unwrap().rectangles,
        session
            .document()
            .native_selection_rects(&same_cell)
            .unwrap()
    );
    let cross_cell = NativeTextSelection {
        anchor: at("가😀나"),
        focus: at("sibling"),
    };
    let snapshot = session
        .accessibility_snapshot(&request, Some(&cross_cell))
        .unwrap();
    assert_eq!(snapshot.selection.as_ref(), Some(&cross_cell));
    let geometry = snapshot.selection_geometry.unwrap();
    assert_eq!(
        geometry.status,
        NativeAccessibilityGeometryStatus::Unsupported
    );
    assert!(geometry.rectangles.is_empty());
    assert!(snapshot
        .limitations
        .contains(&NativeAccessibilityLimitation::SelectionGeometryUnsupported));
    assert!(!snapshot.complete);
}

#[test]
fn issue_536_native_accessibility_keys_are_revision_scoped_not_cross_revision_ids() {
    let session = NativeDocumentSession::from_core(core("body"));
    let request = NativeAccessibilityRequest::default();
    let before = session.accessibility_snapshot(&request, None).unwrap();
    let selection = NativeTextSelection {
        anchor: body(0, 0),
        focus: body(0, 0),
    };
    let mut editor = NativeEditor::new(session, selection).unwrap();
    editor.replace_selection(0, "x").unwrap();
    let after = editor
        .session()
        .accessibility_snapshot(&request, Some(editor.selection()))
        .unwrap();
    assert_eq!(after.revision, 1);
    assert_ne!(before.nodes[0].key, after.nodes[0].key);
    assert_eq!(before.nodes[0].start, after.nodes[0].start);
    assert_eq!(after.nodes[0].text, "xbody");
}

#[test]
fn issue_536_native_accessibility_unicode_conversion_rejects_surrogate_interiors_and_large_input() {
    let text = "A😀가e\u{301}";
    let boundaries = [0, 1, 3, 4, 5, 6, 7];
    for (scalar, utf16) in boundaries.into_iter().enumerate() {
        assert_eq!(native_scalar_to_utf16(text, scalar as u32).unwrap(), utf16);
        assert_eq!(native_utf16_to_scalar(text, utf16).unwrap(), scalar as u32);
    }
    assert!(native_utf16_to_scalar(text, 2).is_err());
    assert!(native_utf16_to_scalar(text, 8).is_err());
    assert!(native_scalar_to_utf16(text, 7).is_err());
    assert_eq!(native_scalar_to_utf16("", 0).unwrap(), 0);
    assert_eq!(native_utf16_to_scalar("", 0).unwrap(), 0);
    let huge = "a".repeat(MAX_ACCESSIBILITY_TEXT_BYTES + 1);
    assert!(matches!(
        native_utf16_to_scalar(&huge, 0),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert!(matches!(
        native_scalar_to_utf16(&huge, 0),
        Err(NativeHostError::LimitExceeded(_))
    ));
}

#[test]
fn issue_536_native_accessibility_request_and_selection_validation_fail_before_snapshot() {
    let session = NativeDocumentSession::from_core(core("😀"));
    for request in [
        NativeAccessibilityRequest {
            max_nodes: 0,
            ..Default::default()
        },
        NativeAccessibilityRequest {
            max_nodes: MAX_ACCESSIBILITY_NODES + 1,
            ..Default::default()
        },
        NativeAccessibilityRequest {
            max_text_bytes: MAX_ACCESSIBILITY_TEXT_BYTES + 1,
            ..Default::default()
        },
        NativeAccessibilityRequest {
            max_depth: MAX_ACCESSIBILITY_DEPTH + 1,
            ..Default::default()
        },
    ] {
        assert!(session.accessibility_snapshot(&request, None).is_err());
    }
    let invalid = NativeTextSelection {
        anchor: body(0, 0),
        focus: body(0, 2),
    };
    assert!(session
        .accessibility_snapshot(&NativeAccessibilityRequest::default(), Some(&invalid))
        .is_err());
    assert!(serde_json::from_str::<NativeAccessibilityRequest>(r#"{"page":0}"#).is_err());
    assert!(serde_json::from_str::<NativeAccessibilityRequest>(r#"{"maxNodes":1.5}"#).is_err());
    assert!(serde_json::from_str::<NativeAccessibilityRequest>(
        r#"{"range":{"section":0,"startParagraph":0,"endParagraph":1,"extra":true}}"#
    )
    .is_err());
    let request: NativeAccessibilityRequest =
        serde_json::from_str(r#"{"maxNodes":1,"maxTextBytes":4,"geometry":false}"#).unwrap();
    assert_eq!(request.max_nodes, 1);
    let json =
        serde_json::to_value(session.accessibility_snapshot(&request, None).unwrap()).unwrap();
    assert!(json.get("fontGeneration").is_some());
    assert!(json["nodes"][0].get("textComplete").is_some());
}

#[test]
fn issue_536_native_accessibility_selection_output_and_scalar_budgets_are_explicit() {
    let session = NativeDocumentSession::from_core(multi_paragraph());
    let selection = NativeTextSelection {
        anchor: body(0, 0),
        focus: body(2, 5),
    };
    let core_rects = session
        .document()
        .native_selection_rects(&selection)
        .unwrap();
    assert!(core_rects.len() > 1);
    let request = NativeAccessibilityRequest {
        geometry: true,
        max_selection_rects: 1,
        ..Default::default()
    };
    let snapshot = session
        .accessibility_snapshot(&request, Some(&selection))
        .unwrap();
    let geometry = snapshot.selection_geometry.unwrap();
    assert_eq!(geometry.rectangles, core_rects[..1]);
    assert_eq!(geometry.status, NativeAccessibilityGeometryStatus::Limited);
    assert!(snapshot.truncated && !snapshot.complete);
    assert!(snapshot
        .limitations
        .contains(&NativeAccessibilityLimitation::GeometryLimit));

    let session = NativeDocumentSession::from_core(core(&"a".repeat(4097)));
    let selection = NativeTextSelection {
        anchor: body(0, 0),
        focus: body(0, 4097),
    };
    let snapshot = session
        .accessibility_snapshot(&request, Some(&selection))
        .unwrap();
    let geometry = snapshot.selection_geometry.unwrap();
    assert_eq!(geometry.status, NativeAccessibilityGeometryStatus::Limited);
    assert!(geometry.rectangles.is_empty());
    assert_eq!(
        snapshot.nodes[0].text.len(),
        4097,
        "geometry limits do not discard semantic text"
    );
}
