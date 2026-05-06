use std::path::Path;

use rhwp::wasm_api::HwpDocument;

fn json_number(json: &str, key: &str) -> f64 {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern).expect("json key not found") + pattern.len();
    let rest = &json[start..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end].parse::<f64>().expect("json number parse")
}

#[test]
fn issue_598_body_footnote_marker_has_hit_and_cursor_unit() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse footnote-01.hwp");

    assert_eq!(doc.get_control_text_positions(0, 3), "[7]");

    let hit = doc
        .hit_test_body_footnote_marker_native(0, 264.0, 380.0)
        .expect("hit body footnote marker");
    assert!(hit.contains("\"hit\":true"), "hit json: {hit}");
    assert!(hit.contains("\"sectionIndex\":0"), "hit json: {hit}");
    assert!(hit.contains("\"paragraphIndex\":3"), "hit json: {hit}");
    assert!(hit.contains("\"controlIndex\":0"), "hit json: {hit}");
    assert!(hit.contains("\"footnoteIndex\":0"), "hit json: {hit}");

    let marker_left = doc
        .get_cursor_rect_native(0, 3, 7)
        .expect("marker left rect");
    let marker_right = doc
        .get_cursor_rect_native(0, 3, 8)
        .expect("marker right rect");
    let left_x = json_number(&marker_left, "x");
    let right_x = json_number(&marker_right, "x");
    assert!(
        right_x > left_x,
        "marker right caret should be after left caret: left={marker_left}, right={marker_right}"
    );

    let next = doc.navigate_next_editable_wasm(0, 3, 7, 1, "[]");
    assert!(next.contains("\"charOffset\":8"), "next json: {next}");
    let prev = doc.navigate_next_editable_wasm(0, 3, 8, -1, "[]");
    assert!(prev.contains("\"charOffset\":7"), "prev json: {prev}");
}

#[test]
fn issue_598_second_body_footnote_marker_has_same_cursor_unit() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse footnote-01.hwp");

    assert_eq!(doc.get_control_text_positions(0, 7), "[6]");

    let hit = doc
        .hit_test_body_footnote_marker_native(0, 214.0, 698.0)
        .expect("hit second body footnote marker");
    assert!(hit.contains("\"hit\":true"), "hit json: {hit}");
    assert!(hit.contains("\"paragraphIndex\":7"), "hit json: {hit}");
    assert!(hit.contains("\"controlIndex\":0"), "hit json: {hit}");
    assert!(hit.contains("\"footnoteIndex\":1"), "hit json: {hit}");

    let next = doc.navigate_next_editable_wasm(0, 7, 6, 1, "[]");
    assert!(next.contains("\"charOffset\":7"), "next json: {next}");
    let prev = doc.navigate_next_editable_wasm(0, 7, 7, -1, "[]");
    assert!(prev.contains("\"charOffset\":6"), "prev json: {prev}");

    let marker_left = doc
        .get_cursor_rect_native(0, 7, 6)
        .expect("second marker left rect");
    let marker_right = doc
        .get_cursor_rect_native(0, 7, 7)
        .expect("second marker right rect");
    let left_x = json_number(&marker_left, "x");
    let right_x = json_number(&marker_right, "x");
    assert!(
        right_x > left_x,
        "second marker right caret should be after left caret: left={marker_left}, right={marker_right}"
    );
}

#[test]
fn issue_598_body_footnote_marker_can_be_found_and_deleted_from_cursor() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse footnote-01.hwp");

    let backward = doc
        .get_footnote_at_cursor_native(0, 3, 8, "backward")
        .expect("find footnote before cursor");
    assert!(
        backward.contains("\"hit\":true"),
        "backward json: {backward}"
    );
    assert!(
        backward.contains("\"controlIndex\":0"),
        "backward json: {backward}"
    );
    assert!(
        backward.contains("\"charOffset\":7"),
        "backward json: {backward}"
    );
    assert!(
        backward.contains("\"footnoteNumber\":1"),
        "backward json: {backward}"
    );

    let forward = doc
        .get_footnote_at_cursor_native(0, 3, 7, "forward")
        .expect("find footnote after cursor");
    assert!(forward.contains("\"hit\":true"), "forward json: {forward}");
    assert!(
        forward.contains("\"controlIndex\":0"),
        "forward json: {forward}"
    );
    assert!(
        forward.contains("\"charOffset\":7"),
        "forward json: {forward}"
    );

    let deleted = doc
        .delete_footnote_native(0, 3, 0)
        .expect("delete first body footnote");
    assert!(deleted.contains("\"ok\":true"), "deleted json: {deleted}");
    assert!(
        deleted.contains("\"charOffset\":7"),
        "deleted json: {deleted}"
    );
    assert!(
        deleted.contains("\"deletedNumber\":1"),
        "deleted json: {deleted}"
    );

    assert_eq!(doc.get_control_text_positions(0, 3), "[]");

    let missed = doc
        .get_footnote_at_cursor_native(0, 3, 8, "backward")
        .expect("deleted footnote should not be found");
    assert_eq!(missed, "{\"hit\":false}");

    let old_marker_hit = doc
        .hit_test_body_footnote_marker_native(0, 264.0, 380.0)
        .expect("hit old marker position after delete");
    assert_eq!(old_marker_hit, "{\"hit\":false}");

    let second_info = doc
        .get_footnote_info_native(0, 7, 0)
        .expect("remaining footnote info");
    assert!(
        second_info.contains("\"number\":1"),
        "second info json: {second_info}"
    );
}

#[test]
fn issue_598_backspace_before_marker_keeps_marker_anchor_and_undo_restores_it() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse footnote-01.hwp");

    assert_eq!(doc.get_control_text_positions(0, 3), "[7]");
    assert_eq!(
        doc.get_text_range_native(0, 3, 6, 2).expect("text around marker"),
        "체와"
    );

    doc.delete_text_native(0, 3, 6, 1)
        .expect("delete text before footnote marker");
    assert_eq!(
        doc.get_control_text_positions(0, 3),
        "[6]",
        "footnote marker should stay between remaining previous text and following text"
    );
    assert_eq!(
        doc.get_text_range_native(0, 3, 5, 2).expect("text around marker after delete"),
        "액와"
    );

    doc.insert_text_native(0, 3, 6, "체")
        .expect("undo-like insert before footnote marker");
    assert_eq!(
        doc.get_control_text_positions(0, 3),
        "[7]",
        "undo-like insert should restore original footnote marker anchor"
    );
    assert_eq!(
        doc.get_text_range_native(0, 3, 6, 2).expect("text around marker after restore"),
        "체와"
    );
}
