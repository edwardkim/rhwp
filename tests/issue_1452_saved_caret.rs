//! Issue #1452: 텍스트 없이 TAC 그림만 있는 문단의 저장 커서 위치 복원.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::Value;

fn parse_json(label: &str, json: &str) -> Value {
    serde_json::from_str(json).unwrap_or_else(|e| panic!("parse {label} json `{json}`: {e}"))
}

fn load_transparency_core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join("samples/투명도0-50.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("load samples/투명도0-50.hwp")
}

fn picture_count(core: &DocumentCore, para_idx: usize) -> usize {
    core.document().sections[0].paragraphs[para_idx]
        .controls
        .iter()
        .filter(|ctrl| matches!(ctrl, Control::Picture(_)))
        .count()
}

fn collect_image_bboxes(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
    }
    for child in &node.children {
        collect_image_bboxes(child, out);
    }
}

#[test]
fn transparency_sample_restores_saved_caret_after_second_inline_picture() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join("samples/투명도0-50.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let parsed = rhwp::parser::parse_hwp(&bytes).expect("parse raw samples/투명도0-50.hwp");
    let props = &parsed.doc_properties;
    assert_eq!(props.caret_list_id, 0);
    assert_eq!(props.caret_para_id, 0);
    assert_eq!(props.caret_char_pos, 32);

    let mut doc =
        rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("load samples/투명도0-50.hwp");

    let caret_json = doc.get_caret_position().expect("getCaretPosition");
    let caret = parse_json("caret", &caret_json);
    assert_eq!(caret["sectionIndex"], 0);
    assert_eq!(caret["paragraphIndex"], 0);
    assert_eq!(caret["charOffset"], 2);

    let first_line_rect = parse_json(
        "first-line cursor rect",
        &doc.get_cursor_rect_native(0, 0, 0)
            .expect("first line cursor rect"),
    );
    let saved_rect = parse_json(
        "saved cursor rect",
        &doc.get_cursor_rect_native(0, 0, 2)
            .expect("saved cursor rect"),
    );

    assert!(
        saved_rect["y"].as_f64().unwrap() > first_line_rect["y"].as_f64().unwrap(),
        "저장 커서는 첫 줄 시작이 아니라 두 번째 TAC 그림 뒤에 있어야 함: first={first_line_rect}, saved={saved_rect}"
    );
    assert!(
        saved_rect["x"].as_f64().unwrap() > first_line_rect["x"].as_f64().unwrap() + 400.0,
        "저장 커서는 두 번째 줄 첫머리가 아니라 두 번째 TAC 그림 오른쪽 끝에 있어야 함: first={first_line_rect}, saved={saved_rect}"
    );
    assert!(
        saved_rect["height"].as_f64().unwrap() < 40.0,
        "TAC 그림 뒤 캐럿 높이는 그림 높이가 아니라 글자 높이여야 함: saved={saved_rect}"
    );

    doc.set_show_paragraph_marks(true);
    let visible_before_first_picture_rect = parse_json(
        "visible before-first-picture cursor rect",
        &doc.get_cursor_rect_native(0, 0, 0)
            .expect("visible before-first-picture cursor rect"),
    );
    let visible_mark_rect = parse_json(
        "visible paragraph-mark cursor rect",
        &doc.get_cursor_rect_native(0, 0, 2)
            .expect("visible paragraph-mark cursor rect"),
    );
    let visible_before_second_picture_rect = parse_json(
        "visible before-second-picture cursor rect",
        &doc.get_cursor_rect_native(0, 0, 1)
            .expect("visible before-second-picture cursor rect"),
    );
    assert!(
        visible_mark_rect["x"].as_f64().unwrap() <= saved_rect["x"].as_f64().unwrap() + 1.0,
        "문단부호 표시 중 캐럿이 문단부호 오른쪽으로 과도하게 밀리면 안 됨: hidden={saved_rect}, visible={visible_mark_rect}"
    );
    assert_eq!(
        visible_mark_rect["y"], saved_rect["y"],
        "문단부호 표시 여부가 그림 bbox 기준 캐럿 y를 임의로 바꾸면 안 됨: hidden={saved_rect}, visible={visible_mark_rect}"
    );
    assert_eq!(visible_mark_rect["height"], saved_rect["height"]);
    assert!(
        visible_before_first_picture_rect["x"].as_f64().unwrap()
            <= visible_before_second_picture_rect["x"].as_f64().unwrap() + 1.0,
        "첫 번째 그림 앞 커서는 첫 번째 그림 왼쪽 기준에 있어야 함: first={visible_before_first_picture_rect}, second={visible_before_second_picture_rect}"
    );
    assert!(
        visible_before_first_picture_rect["y"].as_f64().unwrap()
            < visible_before_second_picture_rect["y"].as_f64().unwrap(),
        "첫 번째 그림 앞 커서는 첫 번째 그림 bbox 기준선에 있어야 함: first={visible_before_first_picture_rect}, second={visible_before_second_picture_rect}"
    );
    assert!(
        visible_before_second_picture_rect["x"].as_f64().unwrap()
            < visible_mark_rect["x"].as_f64().unwrap(),
        "왼쪽 이동 후 커서는 두 번째 그림의 왼쪽에 있어야 함: before_second={visible_before_second_picture_rect}, visible={visible_mark_rect}"
    );
    assert_eq!(
        visible_before_second_picture_rect["y"], visible_mark_rect["y"],
        "왼쪽 이동 후 두 번째 그림 앞 커서도 문단부호 표시 기준선에 맞아야 함: before_second={visible_before_second_picture_rect}, visible={visible_mark_rect}"
    );
}

#[test]
fn enter_before_second_tac_picture_splits_inline_controls() {
    let mut core = load_transparency_core();

    core.split_paragraph_native(0, 0, 1)
        .expect("두 TAC 그림 사이 Enter");

    assert_eq!(core.document().sections[0].paragraphs.len(), 2);
    assert_eq!(
        picture_count(&core, 0),
        1,
        "첫 문단에는 첫 번째 그림만 남아야 한다"
    );
    assert_eq!(
        picture_count(&core, 1),
        1,
        "새 문단에는 두 번째 그림이 이동해야 한다"
    );
}

#[test]
fn repeated_enter_before_second_tac_picture_keeps_picture_after_blank_lines() {
    let mut core = load_transparency_core();

    core.split_paragraph_native(0, 0, 1)
        .expect("두 TAC 그림 사이 Enter");
    core.split_paragraph_native(0, 1, 0)
        .expect("두 번째 그림 앞 Enter 1회 추가");
    core.split_paragraph_native(0, 2, 0)
        .expect("두 번째 그림 앞 Enter 2회 추가");

    assert_eq!(core.document().sections[0].paragraphs.len(), 4);
    assert_eq!(picture_count(&core, 0), 1);
    assert_eq!(picture_count(&core, 1), 0);
    assert_eq!(picture_count(&core, 2), 0);
    assert_eq!(
        picture_count(&core, 3),
        1,
        "반복 Enter 후에도 두 번째 그림은 빈 문단들 뒤에 남아야 한다"
    );
}

#[test]
fn repeated_enter_after_second_tac_picture_appends_blank_lines() {
    let mut core = load_transparency_core();

    core.split_paragraph_native(0, 0, 2)
        .expect("두 번째 TAC 그림 뒤 Enter");
    core.split_paragraph_native(0, 1, 0)
        .expect("두 번째 그림 뒤 Enter 1회 추가");
    core.split_paragraph_native(0, 2, 0)
        .expect("두 번째 그림 뒤 Enter 2회 추가");

    assert_eq!(core.document().sections[0].paragraphs.len(), 4);
    assert_eq!(
        picture_count(&core, 0),
        2,
        "두 번째 그림 뒤 Enter는 두 그림을 앞 문단에 유지해야 한다"
    );
    assert_eq!(picture_count(&core, 1), 0);
    assert_eq!(picture_count(&core, 2), 0);
    assert_eq!(picture_count(&core, 3), 0);
}

#[test]
fn arrow_up_from_second_tac_picture_end_moves_to_first_picture_end() {
    let doc = {
        let repo_root = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(repo_root).join("samples/투명도0-50.hwp");
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("load samples/투명도0-50.hwp")
    };

    let moved = parse_json(
        "move up from second picture end",
        &doc.move_vertical(0, 0, 2, -1, -1.0, u32::MAX, u32::MAX, u32::MAX, u32::MAX)
            .expect("moveVertical ArrowUp"),
    );
    assert_eq!(moved["sectionIndex"], 0);
    assert_eq!(moved["paragraphIndex"], 0);
    assert_eq!(
        moved["charOffset"], 1,
        "두 번째 TAC 그림 끝에서 위쪽 이동하면 첫 번째 TAC 그림 끝으로 가야 한다: {moved}"
    );

    let saved_rect = parse_json(
        "saved cursor rect",
        &doc.get_cursor_rect_native(0, 0, 2)
            .expect("saved cursor rect"),
    );
    assert!(
        moved["y"].as_f64().unwrap() < saved_rect["y"].as_f64().unwrap(),
        "offset=1은 두 번째 그림 앞도 될 수 있으므로, 위쪽 이동 결과는 첫 번째 그림 줄의 좌표를 반환해야 한다: moved={moved}, saved={saved_rect}"
    );
    assert!(
        (moved["x"].as_f64().unwrap() - saved_rect["x"].as_f64().unwrap()).abs() <= 2.0,
        "두 그림의 폭이 같으므로 위쪽 이동 결과는 첫 번째 그림 오른쪽 끝 x에 있어야 한다: moved={moved}, saved={saved_rect}"
    );
}

#[test]
fn clicking_after_second_tac_picture_hits_picture_end_position() {
    let doc = {
        let repo_root = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(repo_root).join("samples/투명도0-50.hwp");
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("load samples/투명도0-50.hwp")
    };

    let saved_rect = parse_json(
        "saved cursor rect",
        &doc.get_cursor_rect_native(0, 0, 2)
            .expect("saved cursor rect"),
    );
    let click_x = saved_rect["x"].as_f64().unwrap() + saved_rect["height"].as_f64().unwrap() / 2.0;
    let click_y = saved_rect["y"].as_f64().unwrap() + saved_rect["height"].as_f64().unwrap() / 2.0;
    let hit = parse_json(
        "hit after second picture",
        &doc.hit_test(0, click_x, click_y)
            .expect("hitTest after second picture"),
    );

    assert_eq!(hit["sectionIndex"], 0);
    assert_eq!(hit["paragraphIndex"], 0);
    assert_eq!(
        hit["charOffset"], 2,
        "두 번째 TAC 그림 뒤 조판부호 근처 클릭은 두 번째 그림 뒤 위치여야 한다: hit={hit}, saved={saved_rect}"
    );
    assert_eq!(
        hit["cursorRect"]["y"], saved_rect["y"],
        "클릭 결과 캐럿도 두 번째 그림 뒤 조판부호 기준선에 있어야 한다: hit={hit}, saved={saved_rect}"
    );
}

#[test]
fn arrow_up_after_enter_before_first_tac_picture_moves_to_previous_picture_end() {
    let mut doc = {
        let repo_root = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(repo_root).join("samples/투명도0-50.hwp");
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("load samples/투명도0-50.hwp")
    };

    doc.split_paragraph_native(0, 0, 0)
        .expect("첫 번째 TAC 그림 앞 Enter");

    let tree = doc.build_page_render_tree(0).expect("build render tree");
    let mut images = Vec::new();
    collect_image_bboxes(&tree.root, &mut images);
    assert_eq!(
        images.len(),
        2,
        "첫 줄 Enter 뒤에도 두 TAC 그림이 모두 렌더되어야 한다: {images:?}"
    );
    images.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let (first_x, first_y, first_w, first_h) = images[0];

    let second_picture_end = parse_json(
        "second picture end after split",
        &doc.get_cursor_rect_native(0, 1, 2)
            .expect("second picture end cursor rect"),
    );
    assert!(
        first_y < second_picture_end["y"].as_f64().unwrap(),
        "첫 줄 Enter 뒤에도 그림 두 줄의 순서가 유지되어야 한다: first_bbox={:?}, second={second_picture_end}",
        images[0]
    );

    let moved = parse_json(
        "move up from second picture end after split",
        &doc.move_vertical(0, 1, 2, -1, -1.0, u32::MAX, u32::MAX, u32::MAX, u32::MAX)
            .expect("moveVertical ArrowUp after split"),
    );

    let expected_first_caret_y =
        ((first_y + (first_h * 0.85 - 12.0 * 0.8).max(0.0)) * 10.0).round() / 10.0;

    assert_eq!(moved["sectionIndex"], 0);
    assert_eq!(moved["paragraphIndex"], 1);
    assert_eq!(
        moved["charOffset"], 1,
        "첫 줄 Enter 뒤에도 두 번째 TAC 그림 끝에서 위쪽 이동하면 첫 번째 TAC 그림 끝으로 가야 한다: {moved}"
    );
    assert_eq!(
        moved["y"].as_f64().unwrap(),
        expected_first_caret_y,
        "위쪽 이동 결과는 첫 번째 TAC 그림 줄의 y를 유지해야 한다: moved={moved}, first_bbox={:?}",
        images[0]
    );
    assert!(
        (moved["x"].as_f64().unwrap() - (first_x + first_w)).abs() <= 2.0,
        "위쪽 이동 결과는 첫 번째 TAC 그림 오른쪽 끝 x에 있어야 한다: moved={moved}, first_bbox={:?}",
        images[0]
    );
}

#[test]
fn transparency_test_sample_text_line_boundaries_keep_visual_affinity() {
    let mut doc = rhwp::wasm_api::HwpDocument::create_empty();
    let text = "1".repeat(240);
    doc.insert_text(0, 0, 0, &text)
        .expect("insert wrapped text");
    let para = 0;

    let first_line = parse_json(
        "first line info",
        &doc.get_line_info(0, para, 0).expect("first line info"),
    );
    let first_line_end = first_line["charEnd"].as_u64().expect("first charEnd") as u32;
    let second_line = parse_json(
        "second line info",
        &doc.get_line_info(0, para, first_line_end)
            .expect("second line info"),
    );
    let second_line_end = second_line["charEnd"].as_u64().expect("second charEnd") as u32;

    assert_eq!(first_line["lineIndex"], 0);
    assert!(
        first_line["lineCount"].as_u64().unwrap_or(0) >= 3,
        "긴 텍스트는 최소 3줄로 wrap되어야 한다: {first_line}"
    );
    assert!(first_line_end > 0);
    assert_eq!(second_line["lineIndex"], 1);
    assert_eq!(second_line["charStart"], first_line_end);
    assert!(second_line_end > first_line_end);

    let body = (u32::MAX, u32::MAX, u32::MAX, u32::MAX);
    let first_end_rect = parse_json(
        "first line end visual rect",
        &doc.get_cursor_rect_on_line(0, para, 0, true, body.0, body.1, body.2, body.3)
            .expect("first line end visual rect"),
    );
    let second_start_rect = parse_json(
        "second line start visual rect",
        &doc.get_cursor_rect_on_line(0, para, 1, false, body.0, body.1, body.2, body.3)
            .expect("second line start visual rect"),
    );
    let second_end_rect = parse_json(
        "second line end visual rect",
        &doc.get_cursor_rect_on_line(0, para, 1, true, body.0, body.1, body.2, body.3)
            .expect("second line end visual rect"),
    );
    let third_start_rect = parse_json(
        "third line start visual rect",
        &doc.get_cursor_rect_on_line(0, para, 2, false, body.0, body.1, body.2, body.3)
            .expect("third line start visual rect"),
    );

    assert!(
        second_start_rect["y"].as_f64().unwrap() > first_end_rect["y"].as_f64().unwrap(),
        "같은 offset={}이라도 Home은 두 번째 줄 시작 좌표를 써야 한다: first_end={first_end_rect}, second_start={second_start_rect}",
        first_line_end
    );
    assert!(
        second_start_rect["x"].as_f64().unwrap() < first_end_rect["x"].as_f64().unwrap() - 100.0,
        "두 번째 줄 시작은 첫 번째 줄 끝이 아니라 왼쪽 줄 시작이어야 한다: first_end={first_end_rect}, second_start={second_start_rect}"
    );
    assert_eq!(
        second_end_rect["y"], second_start_rect["y"],
        "두 번째 줄 End는 두 번째 줄 기준선에 남아야 한다: second_start={second_start_rect}, second_end={second_end_rect}"
    );
    assert!(
        third_start_rect["y"].as_f64().unwrap() > second_end_rect["y"].as_f64().unwrap(),
        "같은 offset={}이라도 다음 줄 Home은 세 번째 줄 시작 좌표를 써야 한다: second_end={second_end_rect}, third_start={third_start_rect}",
        second_line_end
    );
}
