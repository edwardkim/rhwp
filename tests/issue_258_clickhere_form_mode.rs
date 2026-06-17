//! Issue #258: 누름틀 양식 모드 편집 가능 속성 회귀 가드.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::FieldType;
use serde_json::Value;

fn make_doc_with_inserted_clickhere() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    core.insert_text_native(0, 0, 0, "ABC")
        .expect("insert base text");
    core.insert_click_here_field_at(0, 0, 1, "입력하세요", "메모", "name01", true)
        .expect("insert clickhere field");
    core
}

fn assert_clickhere_form_editable(path: &Path) {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let core = DocumentCore::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", path.display(), e));

    let fields = core.collect_all_fields();
    let click_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .collect();

    assert_eq!(
        click_fields.len(),
        2,
        "{} should contain two ClickHere fields",
        path.display()
    );

    for field in click_fields {
        assert!(
            field.field.is_editable_in_form(),
            "{} ClickHere field should be editable in form mode",
            path.display()
        );

        assert!(
            field.location.nested_path.is_empty(),
            "{} sample ClickHere field should be in body text",
            path.display()
        );
        let para = &core.document().sections[field.location.section_index].paragraphs
            [field.location.para_index];
        let range = &para.field_ranges[field.field_range_index];
        let info = core.get_field_info_at(
            field.location.section_index,
            field.location.para_index,
            range.start_char_idx,
        );
        assert!(
            info.contains("\"editableInForm\":true"),
            "field info should expose editableInForm=true for {}: {}",
            path.display(),
            info
        );
    }

    let list_json = core.get_field_list_json();
    assert!(
        list_json.contains("\"editableInForm\":true"),
        "field list should expose editableInForm=true for {}: {}",
        path.display(),
        list_json
    );
    assert!(
        list_json.contains("\"startCharIdx\":"),
        "field list should expose startCharIdx for form navigation: {}",
        list_json
    );
    assert!(
        list_json.contains("\"endCharIdx\":"),
        "field list should expose endCharIdx for form navigation: {}",
        list_json
    );
}

#[test]
fn clickhere_form_editable_attribute_is_preserved_in_hwp_and_hwpx() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_clickhere_form_editable(&repo_root.join("samples/누름틀-2024.hwp"));
    assert_clickhere_form_editable(&repo_root.join("samples/누름틀-2024.hwpx"));
}

#[test]
fn clickhere_hwp_sample_cursor_rects_follow_visible_value() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = fs::read(repo_root.join("samples/누름틀-2024.hwp")).expect("read clickhere sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse clickhere sample");

    let fields = core.collect_all_fields();
    let first = fields
        .iter()
        .find(|f| f.field.field_type == FieldType::ClickHere && f.location.para_index == 0)
        .expect("first clickhere field");
    assert_eq!(first.value, "11223344");

    let para = &core.document().sections[0].paragraphs[0];
    let range = &para.field_ranges[first.field_range_index];
    assert_eq!((range.start_char_idx, range.end_char_idx), (0, 8));

    let mut prev_x = None;
    for offset in range.start_char_idx..=range.end_char_idx {
        let rect: Value = serde_json::from_str(
            &core
                .get_cursor_rect_native(0, 0, offset)
                .expect("cursor rect in clickhere sample"),
        )
        .expect("parse cursor rect");
        let x = rect["x"].as_f64().expect("cursor x");
        if let Some(prev) = prev_x {
            assert!(
                x >= prev,
                "clickhere sample cursor x should be monotonic at offset {offset}: prev={prev}, x={x}"
            );
        }
        prev_x = Some(x);
    }

    let start_rect: Value = serde_json::from_str(
        &core
            .get_cursor_rect_native(0, 0, range.start_char_idx)
            .expect("field start rect"),
    )
    .expect("parse start rect");
    let end_rect: Value = serde_json::from_str(
        &core
            .get_cursor_rect_native(0, 0, range.end_char_idx)
            .expect("field end rect"),
    )
    .expect("parse end rect");
    assert!(
        end_rect["x"].as_f64().unwrap() > start_rect["x"].as_f64().unwrap(),
        "field end cursor should be after start: start={start_rect}, end={end_rect}"
    );
}

#[test]
fn removing_clickhere_removes_field_text_and_control() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = fs::read(repo_root.join("samples/누름틀-2024.hwp")).expect("read clickhere sample");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse clickhere sample");

    let para_before = &core.document().sections[0].paragraphs[0];
    assert_eq!(para_before.text, "11223344");
    assert_eq!(para_before.controls.len(), 3);
    assert_eq!(para_before.field_ranges.len(), 1);

    core.remove_field_at(0, 0, 8)
        .expect("remove first clickhere");

    let para_after = &core.document().sections[0].paragraphs[0];
    assert_eq!(para_after.text, "");
    assert_eq!(
        para_after.field_ranges.len(),
        0,
        "field range should be removed"
    );
    assert_eq!(
        para_after.controls.len(),
        2,
        "ClickHere control should be removed while SectionDef/ColumnDef remain"
    );
    assert_eq!(para_after.char_offsets, Vec::<u32>::new());

    let fields = core.collect_all_fields();
    let click_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .collect();
    assert_eq!(click_fields.len(), 1);
    assert_eq!(click_fields[0].value, "222212212");
}

#[test]
fn copying_clickhere_preserves_field_control_after_structural_controls_are_stripped() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = fs::read(repo_root.join("samples/누름틀-2024.hwp")).expect("read clickhere sample");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse clickhere sample");

    let before_fields = core.collect_all_fields();
    let before_click_values: Vec<_> = before_fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .map(|f| f.value.as_str())
        .collect();
    assert_eq!(before_click_values, vec!["11223344", "222212212"]);

    core.copy_selection_native(0, 0, 0, 0, 8)
        .expect("copy first clickhere selection");
    assert_eq!(core.get_clipboard_text_native(), "11223344");
    assert!(
        !core.clipboard_has_control_native(),
        "ClickHere text selection should use the text/field paste path"
    );

    core.paste_internal_native(0, 1, 9)
        .expect("paste copied clickhere after second field");

    let after_fields = core.collect_all_fields();
    let after_click_values: Vec<_> = after_fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .map(|f| f.value.as_str())
        .collect();
    assert_eq!(
        after_click_values,
        vec!["11223344", "222212212", "11223344"]
    );

    let pasted = after_fields
        .iter()
        .find(|f| {
            f.field.field_type == FieldType::ClickHere
                && f.location.para_index == 1
                && f.value == "11223344"
        })
        .expect("pasted clickhere should be collected as a field");
    let para = &core.document().sections[pasted.location.section_index].paragraphs
        [pasted.location.para_index];
    let range = &para.field_ranges[pasted.field_range_index];
    assert!(matches!(
        para.controls.get(range.control_idx),
        Some(rhwp::model::control::Control::Field(_))
    ));
}

#[test]
fn pasted_clickhere_reports_field_and_following_input_stays_outside() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    core.insert_click_here_field_at(0, 0, 0, "입력하세요", "", "copied", true)
        .expect("insert clickhere");
    assert!(core.set_active_field(0, 0, 0));
    core.insert_text_native(0, 0, 0, "123")
        .expect("fill clickhere");
    core.clear_active_field();

    core.copy_selection_native(0, 0, 0, 0, 3)
        .expect("copy clickhere value");
    let paste_json = core
        .paste_internal_native(0, 0, 3)
        .expect("paste clickhere after original");
    let paste: Value = serde_json::from_str(&paste_json).expect("parse paste result");
    assert_eq!(
        paste["containsField"], true,
        "Studio paste handler needs this flag to leave caret outside the pasted ClickHere: {paste_json}"
    );
    assert_eq!(paste["charOffset"].as_u64(), Some(6));

    // Studio는 containsField=true를 받으면 마지막 누름틀 끝 경계를 필드 바깥으로 표시한다.
    core.clear_active_field();
    core.insert_text_native(0, 0, 6, "X")
        .expect("following input should be body text");

    let fields = core.collect_all_fields();
    let click_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .collect();
    assert_eq!(click_fields.len(), 2);
    let para = &core.document().sections[0].paragraphs[0];
    assert_eq!(para.text, "123123X");
    let ranges: Vec<_> = click_fields
        .iter()
        .map(|field| {
            let range = &para.field_ranges[field.field_range_index];
            (
                range.start_char_idx,
                range.end_char_idx,
                field.value.as_str(),
            )
        })
        .collect();
    assert_eq!(ranges, vec![(0, 3, "123"), (3, 6, "123")]);
}

#[test]
fn copying_clickhere_after_prefix_preserves_field_value() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    core.insert_text_native(0, 0, 0, "AA")
        .expect("insert prefix");
    core.insert_click_here_field_at(0, 0, 2, "입력하세요", "메모", "prefixed", true)
        .expect("insert clickhere after prefix");
    core.insert_text_native(0, 0, 2, "11223344")
        .expect("insert clickhere value");

    let before_fields = core.collect_all_fields();
    assert_eq!(before_fields.len(), 1);
    assert_eq!(before_fields[0].value, "11223344");
    let before_range =
        &core.document().sections[0].paragraphs[0].field_ranges[before_fields[0].field_range_index];
    assert_eq!(
        (before_range.start_char_idx, before_range.end_char_idx),
        (2, 10)
    );

    core.copy_selection_native(0, 0, 2, 0, 10)
        .expect("copy prefixed clickhere selection");
    assert_eq!(core.get_clipboard_text_native(), "11223344");

    core.paste_internal_native(0, 0, 10)
        .expect("paste prefixed clickhere after original field");

    let after_fields = core.collect_all_fields();
    let after_values: Vec<_> = after_fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .map(|f| f.value.as_str())
        .collect();
    assert_eq!(after_values, vec!["11223344", "11223344"]);
    assert_eq!(
        core.document().sections[0].paragraphs[0].text,
        "AA1122334411223344"
    );
}

#[test]
fn clickhere_insert_api_creates_empty_editable_field() {
    let core = make_doc_with_inserted_clickhere();

    let fields = core.collect_all_fields();
    let click_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .collect();
    assert_eq!(click_fields.len(), 1);

    let field = click_fields[0];
    assert_eq!(field.value, "");
    assert_eq!(field.field.guide_text(), Some("입력하세요"));
    assert_eq!(field.field.memo_text(), Some("메모"));
    assert_eq!(field.field.field_name(), Some("name01"));
    assert!(field.field.is_editable_in_form());

    let para = &core.document().sections[0].paragraphs[0];
    let range = &para.field_ranges[field.field_range_index];
    assert_eq!(range.start_char_idx, 1);
    assert_eq!(range.end_char_idx, 1);
    assert_eq!(para.char_offsets, vec![16, 33, 34]);
    assert_eq!(para.char_offsets[1] - (para.char_offsets[0] + 1), 16);

    let info = core.get_field_info_at(0, 0, 1);
    assert!(info.contains(r#""inField":true"#), "{}", info);
    assert!(info.contains(r#""isGuide":true"#), "{}", info);
    assert!(info.contains(r#""editableInForm":true"#), "{}", info);

    let list_json = core.get_field_list_json();
    assert!(
        list_json.contains(r#""guide":"입력하세요""#),
        "{}",
        list_json
    );
    assert!(list_json.contains(r#""name":"name01""#), "{}", list_json);
    assert!(list_json.contains(r#""startCharIdx":1"#), "{}", list_json);
    assert!(list_json.contains(r#""endCharIdx":1"#), "{}", list_json);
}

#[test]
fn clickhere_end_boundary_insert_respects_active_field_state() {
    let mut core = make_doc_with_inserted_clickhere();

    assert!(
        core.set_active_field(0, 0, 1),
        "empty field guide click should activate clickhere"
    );
    core.insert_text_native(0, 0, 1, "값")
        .expect("first input should fill empty clickhere");
    let fields = core.collect_all_fields();
    let field = fields
        .iter()
        .find(|f| f.field.field_type == FieldType::ClickHere)
        .expect("clickhere field");
    let range = &core.document().sections[0].paragraphs[0].field_ranges[field.field_range_index];
    assert_eq!((range.start_char_idx, range.end_char_idx), (1, 2));
    assert_eq!(field.value, "값");

    let info = core.get_field_info_at(0, 0, 2);
    assert!(
        info.contains(r#""inField":true"#),
        "field end should be an editable boundary: {}",
        info
    );
    let _ = core.set_active_field(0, 0, 2);
    core.insert_text_native(0, 0, 2, "1")
        .expect("active field end should append to clickhere value");
    let fields = core.collect_all_fields();
    let field = fields
        .iter()
        .find(|f| f.field.field_type == FieldType::ClickHere)
        .expect("clickhere field after active append");
    let range = &core.document().sections[0].paragraphs[0].field_ranges[field.field_range_index];
    assert_eq!((range.start_char_idx, range.end_char_idx), (1, 3));
    assert_eq!(field.value, "값1");

    core.clear_active_field();
    core.insert_text_native(0, 0, 3, "밖")
        .expect("inactive field end should insert outside clickhere");
    let fields = core.collect_all_fields();
    let field = fields
        .iter()
        .find(|f| f.field.field_type == FieldType::ClickHere)
        .expect("clickhere field after outside insert");
    let para = &core.document().sections[0].paragraphs[0];
    let range = &para.field_ranges[field.field_range_index];
    assert_eq!((range.start_char_idx, range.end_char_idx), (1, 3));
    assert_eq!(field.value, "값1");
    assert_eq!(para.text, "A값1밖BC");
}

#[test]
fn adjacent_clickhere_input_prefers_new_empty_field_at_shared_boundary() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    core.insert_text_native(0, 0, 0, "abc")
        .expect("insert prefix");

    core.insert_click_here_field_at(0, 0, 3, "첫 번째", "", "first", true)
        .expect("insert first clickhere");
    assert!(
        core.set_active_field(0, 0, 3),
        "first empty clickhere should activate"
    );
    core.insert_text_native(0, 0, 3, "123")
        .expect("fill first clickhere");
    core.clear_active_field();

    let inserted_second = core
        .insert_click_here_field_at(0, 0, 6, "두 번째", "", "second", true)
        .expect("insert adjacent second clickhere");
    let second_json: Value =
        serde_json::from_str(&inserted_second).expect("parse inserted second json");
    let second_id = second_json["fieldId"].as_u64().expect("second field id") as u32;

    let shared_info = core.get_field_info_at(0, 0, 6);
    assert!(
        shared_info.contains(&format!(r#""fieldId":{}"#, second_id)),
        "shared boundary should resolve to new empty clickhere: {}",
        shared_info
    );
    assert!(
        shared_info.contains(r#""isGuide":true"#),
        "shared boundary should expose second field as guide: {}",
        shared_info
    );

    let guide_rect: Value = serde_json::from_str(
        &core
            .get_cursor_rect_native(0, 0, 6)
            .expect("second guide cursor rect"),
    )
    .expect("parse second guide cursor rect");
    let guide_x = guide_rect["x"].as_f64().expect("guide x");
    let guide_y = guide_rect["y"].as_f64().expect("guide y");
    let guide_h = guide_rect["height"].as_f64().expect("guide height");
    let guide_hit = core
        .hit_test_native(0, guide_x + 6.0, guide_y + guide_h / 2.0)
        .expect("hit test second guide");
    assert!(
        guide_hit.contains(&format!(r#""fieldId":{}"#, second_id)),
        "mouse hit on second guide should resolve to second clickhere: {}",
        guide_hit
    );
    assert!(
        guide_hit.contains(r#""charOffset":6"#),
        "mouse hit on second guide should keep shared boundary offset: {}",
        guide_hit
    );

    assert!(
        core.set_active_field(0, 0, 6),
        "shared boundary guide click should activate second clickhere"
    );
    core.insert_text_native(0, 0, 6, "123")
        .expect("fill second clickhere");

    let para = &core.document().sections[0].paragraphs[0];
    assert_eq!(para.text, "abc123123");

    let fields = core.collect_all_fields();
    let click_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .collect();
    assert_eq!(click_fields.len(), 2);
    let ranges: Vec<_> = click_fields
        .iter()
        .map(|field| {
            let range = &para.field_ranges[field.field_range_index];
            (
                range.start_char_idx,
                range.end_char_idx,
                field.value.as_str(),
            )
        })
        .collect();
    assert_eq!(ranges, vec![(3, 6, "123"), (6, 9, "123")]);
}

#[test]
fn copying_adjacent_clickheres_preserves_separate_pasted_fields() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    core.insert_text_native(0, 0, 0, "abc")
        .expect("insert prefix");

    core.insert_click_here_field_at(0, 0, 3, "첫 번째", "", "first", true)
        .expect("insert first clickhere");
    assert!(core.set_active_field(0, 0, 3));
    core.insert_text_native(0, 0, 3, "123")
        .expect("fill first clickhere");
    core.clear_active_field();

    core.insert_click_here_field_at(0, 0, 6, "두 번째", "", "second", true)
        .expect("insert adjacent second clickhere");
    assert!(core.set_active_field(0, 0, 6));
    core.insert_text_native(0, 0, 6, "123")
        .expect("fill second clickhere");
    core.clear_active_field();

    core.copy_selection_native(0, 0, 3, 0, 9)
        .expect("copy adjacent clickheres");
    assert_eq!(core.get_clipboard_text_native(), "123123");

    core.split_paragraph_native(0, 0, 9)
        .expect("create paste target paragraph");
    core.paste_internal_native(0, 1, 0)
        .expect("paste adjacent clickheres into next paragraph");

    assert_eq!(
        core.document().sections[0].paragraphs[1].text,
        "123123",
        "pasted visible text should include both clickhere values"
    );

    let fields = core.collect_all_fields();
    let pasted_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere && f.location.para_index == 1)
        .collect();
    assert_eq!(
        pasted_fields.len(),
        2,
        "pasted paragraph should contain two ClickHere fields"
    );
    let original_ids: Vec<_> = fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere && f.location.para_index == 0)
        .map(|f| f.field.field_id)
        .collect();
    let pasted_ids: Vec<_> = pasted_fields.iter().map(|f| f.field.field_id).collect();
    assert_eq!(original_ids.len(), 2);
    assert_eq!(pasted_ids.len(), 2);
    assert_ne!(
        pasted_ids[0], pasted_ids[1],
        "pasted ClickHere fields should have distinct IDs"
    );
    for id in &pasted_ids {
        assert!(
            !original_ids.contains(id),
            "pasted ClickHere ID should not duplicate original IDs: originals={original_ids:?}, pasted={pasted_ids:?}"
        );
    }

    let para = &core.document().sections[0].paragraphs[1];
    assert_eq!(
        para.char_offsets.len(),
        6,
        "pasted paragraph char_offsets should cover both adjacent field values"
    );
    let ranges: Vec<_> = pasted_fields
        .iter()
        .map(|field| {
            let range = &para.field_ranges[field.field_range_index];
            (
                range.start_char_idx,
                range.end_char_idx,
                field.value.as_str(),
            )
        })
        .collect();
    assert_eq!(ranges, vec![(0, 3, "123"), (3, 6, "123")]);

    let svg = core
        .render_page_svg_native(0)
        .expect("render pasted clickheres");
    for digit in ["1", "2", "3"] {
        assert_eq!(
            svg.matches(&format!(">{digit}<")).count(),
            4,
            "original and pasted adjacent ClickHere values should both be visible for digit {digit}: {svg}"
        );
    }

    core.remove_field_at(0, 1, 3)
        .expect("remove pasted first clickhere");
    let remaining_fields: Vec<_> = core
        .collect_all_fields()
        .into_iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere && f.location.para_index == 1)
        .collect();
    assert_eq!(remaining_fields.len(), 1);
    assert_eq!(remaining_fields[0].value, "123");
    assert_eq!(core.document().sections[0].paragraphs[1].text, "123");
}

#[test]
fn clickhere_start_boundary_insert_respects_active_field_state() {
    let mut core = make_doc_with_inserted_clickhere();

    assert!(
        core.set_active_field(0, 0, 1),
        "empty field guide click should activate clickhere"
    );
    core.insert_text_native(0, 0, 1, "값")
        .expect("first input should fill empty clickhere");

    core.clear_active_field();
    core.insert_text_native(0, 0, 1, "앞")
        .expect("inactive field start should insert before clickhere");
    let fields = core.collect_all_fields();
    let field = fields
        .iter()
        .find(|f| f.field.field_type == FieldType::ClickHere)
        .expect("clickhere field after outside prefix");
    let para = &core.document().sections[0].paragraphs[0];
    let range = &para.field_ranges[field.field_range_index];
    assert_eq!((range.start_char_idx, range.end_char_idx), (2, 3));
    assert_eq!(field.value, "값");
    assert_eq!(para.text, "A앞값BC");

    let _ = core.set_active_field(0, 0, 2);
    core.insert_text_native(0, 0, 2, "안")
        .expect("active field start should insert inside clickhere");
    let fields = core.collect_all_fields();
    let field = fields
        .iter()
        .find(|f| f.field.field_type == FieldType::ClickHere)
        .expect("clickhere field after active prefix");
    let para = &core.document().sections[0].paragraphs[0];
    let range = &para.field_ranges[field.field_range_index];
    assert_eq!((range.start_char_idx, range.end_char_idx), (2, 4));
    assert_eq!(field.value, "안값");
    assert_eq!(para.text, "A앞안값BC");
}

#[test]
fn first_input_into_empty_clickhere_is_rendered() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    core.insert_click_here_field_at(0, 0, 0, "입력하세요", "", "name", true)
        .expect("insert empty clickhere");

    assert!(
        core.set_active_field(0, 0, 0),
        "guide click should activate empty clickhere"
    );
    core.insert_text_native(0, 0, 0, "123")
        .expect("first input should fill empty clickhere");

    let fields = core.collect_all_fields();
    let field = fields
        .iter()
        .find(|f| f.field.field_type == FieldType::ClickHere)
        .expect("clickhere field");
    assert_eq!(field.value, "123");

    let svg = core.render_page_svg_native(0).expect("render page 1");
    assert!(
        svg.contains(">1<") && svg.contains(">2<") && svg.contains(">3<"),
        "first clickhere input should be visible in render svg: {}",
        svg
    );

    let start_rect: Value = serde_json::from_str(
        &core
            .get_cursor_rect_native(0, 0, 0)
            .expect("cursor rect at field start"),
    )
    .expect("parse start cursor rect");
    let end_rect: Value = serde_json::from_str(
        &core
            .get_cursor_rect_native(0, 0, 3)
            .expect("cursor rect at field end"),
    )
    .expect("parse end cursor rect");
    let start_x = start_rect["x"].as_f64().expect("start x");
    let end_x = end_rect["x"].as_f64().expect("end x");
    assert!(
        end_x > start_x,
        "field end cursor should move after first input: start={start_rect}, end={end_rect}"
    );
}

#[test]
fn inserted_clickhere_roundtrips_hwp_and_hwpx() {
    let core = make_doc_with_inserted_clickhere();

    let hwp = core.export_hwp_native().expect("export hwp");
    let reparsed_hwp = DocumentCore::from_bytes(&hwp).expect("reparse exported hwp");
    assert_inserted_clickhere_roundtrip(&reparsed_hwp, "HWP");

    let hwpx = core.export_hwpx_native().expect("export hwpx");
    let reparsed_hwpx = DocumentCore::from_bytes(&hwpx).expect("reparse exported hwpx");
    assert_inserted_clickhere_roundtrip(&reparsed_hwpx, "HWPX");
}

fn assert_inserted_clickhere_roundtrip(core: &DocumentCore, label: &str) {
    let fields = core.collect_all_fields();
    let click_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.field.field_type == FieldType::ClickHere)
        .collect();
    assert_eq!(click_fields.len(), 1, "{} ClickHere count", label);

    let field = click_fields[0];
    assert_eq!(field.value, "", "{} value", label);
    assert_eq!(
        field.field.guide_text(),
        Some("입력하세요"),
        "{} guide",
        label
    );
    assert_eq!(field.field.memo_text(), Some("메모"), "{} memo", label);
    assert_eq!(field.field.field_name(), Some("name01"), "{} name", label);
    assert!(
        field.field.is_editable_in_form(),
        "{} editableInForm",
        label
    );
}
