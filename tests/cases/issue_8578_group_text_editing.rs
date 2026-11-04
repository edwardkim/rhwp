//! 관계형: 묶음 글상자와 본문의 클릭 주소가 실제 원문 소유자를 가리키는지 검사한다.
//! 주석 갱신: 2026-11-05T01:24:05+09:00.
//! 입력: 행정업무운영 편람 HWPX. 시각 픽셀 위치를 고정하지 않고 최종 트리의 글자 위치로 클릭한다.
//! Native/fresh WASM 시각 일치율은 이 주소 계약의 판정에 사용하지 않는다.

use rhwp::{
    document_core::DocumentCore,
    renderer::render_tree::{RenderNode, RenderNodeType},
};
use serde_json::Value;

fn load() -> DocumentCore {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/2025 행정업무운영 편람(최종).hwpx");
    DocumentCore::from_bytes(&std::fs::read(file).unwrap()).unwrap()
}

fn find_text<'a>(node: &'a RenderNode, text: &str) -> Option<&'a RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(text)) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_text(child, text))
}

fn hit(core: &DocumentCore, page: u32, text: &str) -> Value {
    let tree = core.build_page_render_tree(page).unwrap();
    let node = find_text(&tree.root, text).unwrap_or_else(|| panic!("{page}: {text}"));
    serde_json::from_str(
        &core
            .hit_test_native(
                page,
                node.bbox.x + node.bbox.width / 2.0,
                node.bbox.y + node.bbox.height / 2.0,
            )
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn contents_body_is_not_intercepted_by_group_decoration() {
    let core = load();
    let h = hit(&core, 3, "행정업무 운영 개요");
    assert!(
        h.get("parentParaIndex").is_none(),
        "목차 본문을 장식 글상자로 바꾸면 안 된다: {h}"
    );
    assert_eq!(h["sectionIndex"], 0);
    assert_eq!(h["paragraphIndex"], 8);
}

#[test]
fn empty_group_owner_has_a_body_caret_on_its_own_page() {
    let core = load();
    for (paragraph, page) in [(4, 2), (6, 3)] {
        let rect: Value =
            serde_json::from_str(&core.get_cursor_rect_native(0, paragraph, 0).unwrap()).unwrap();
        assert_eq!(
            rect["pageIndex"], page,
            "Esc 복귀 주소의 캐럿이 원래 쪽에 남는다"
        );
        assert!(rect["height"].as_f64().unwrap() > 0.0);
        assert!(rect["y"].as_f64().unwrap() >= 0.0);
    }
}

#[test]
fn header_group_does_not_publish_a_body_caret_address() {
    use rhwp::model::{control::Control, shape::ShapeObject};
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.create_header_footer_native(0, true, 0).unwrap();
    for x in [0, 5000] {
        core.create_shape_control_native(
            0,
            0,
            0,
            4000,
            2000,
            x,
            0,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            &[],
        )
        .unwrap();
    }
    let targets = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| matches!(c, Control::Shape(_)).then_some((0, i)))
        .collect::<Vec<_>>();
    core.group_shapes_native(0, &targets).unwrap();
    let mut model = core.document().clone();
    let body = &mut model.sections[0].paragraphs[0];
    let index = body.controls.iter().position(|c|
        matches!(c, Control::Shape(shape) if matches!(shape.as_ref(), ShapeObject::Group(_)))).unwrap();
    let group = body.controls.remove(index);
    let header = body
        .controls
        .iter_mut()
        .find_map(|c| match c {
            Control::Header(header) => Some(header),
            _ => None,
        })
        .unwrap();
    header.paragraphs[0].controls.push(group);
    let bytes = rhwp::serializer::hwpx::serialize_hwpx(&model).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    fn inspect(node: &RenderNode, in_header: bool, groups: &mut usize) {
        let in_header = in_header || matches!(node.node_type, RenderNodeType::Header);
        if in_header {
            if matches!(node.node_type, RenderNodeType::Group(_)) {
                *groups += 1;
            }
            if let RenderNodeType::TextRun(run) = &node.node_type {
                assert!(
                    !(run.section_index == Some(0)
                        && run.para_index == Some(0)
                        && run.cell_context.is_none()
                        && run.char_start.is_some()),
                    "머리말의 묶음을 본문 문단 0의 편집 주소로 노출하면 안 된다"
                );
            }
        }
        for child in &node.children {
            inspect(child, in_header, groups);
        }
    }
    let mut groups = 0;
    inspect(&tree.root, false, &mut groups);
    assert!(groups > 0, "실제 머리말 묶음을 대조한다");
}

#[test]
fn distinct_group_textboxes_resolve_to_their_actual_text() {
    let core = load();
    let title = hit(&core, 2, "행정업무운영 편람");
    let body = hit(&core, 2, "이 편람은");
    assert_ne!(
        title["cellPath"], body["cellPath"],
        "서로 다른 묶음 글상자 주소를 합치면 안 된다"
    );
    for (h, expected) in [(&title, "행정업무운영 편람"), (&body, "이 편람은")] {
        let text = core
            .get_text_in_cell_native(
                0,
                h["parentParaIndex"].as_u64().unwrap() as usize,
                h["controlIndex"].as_u64().unwrap() as usize,
                h["cellIndex"].as_u64().unwrap() as usize,
                h["cellParaIndex"].as_u64().unwrap() as usize,
                0,
                10000,
            )
            .unwrap();
        assert!(
            text.contains(expected),
            "클릭 주소가 다른 원문을 가리킨다: {text}"
        );
    }
}

#[test]
fn group_text_edit_split_merge_and_save_preserve_sibling_ownership() {
    let mut core = load();
    let text = |core: &DocumentCore, index, para| {
        core.get_text_in_cell_native(0, 4, 0, index, para, 0, 10000)
            .unwrap()
    };
    let title = text(&core, 0, 0);
    let body = text(&core, 1, 0);
    let other_body = text(&core, 1, 1);
    let count = core.get_cell_paragraph_count_native(0, 4, 0, 1).unwrap();
    core.insert_text_in_cell_native(0, 4, 0, 1, 0, 3, "검증")
        .unwrap();
    let edited = text(&core, 1, 0);
    assert_eq!(
        edited,
        format!(
            "{}검증{}",
            body.chars().take(3).collect::<String>(),
            body.chars().skip(3).collect::<String>()
        )
    );
    assert_eq!(
        text(&core, 0, 0),
        title,
        "다른 묶음 글상자에는 입력하지 않는다"
    );
    core.split_paragraph_in_cell_native(0, 4, 0, 1, 0, 3, None)
        .unwrap();
    assert_eq!(
        core.get_cell_paragraph_count_native(0, 4, 0, 1).unwrap(),
        count + 1
    );
    core.merge_paragraph_in_cell_native(0, 4, 0, 1, 1).unwrap();
    assert_eq!(
        text(&core, 1, 0),
        edited,
        "Enter/병합에서 문자를 누락·중복하지 않는다"
    );
    assert_eq!(
        text(&core, 1, 1),
        other_body,
        "후속 원문 문단은 소유 순서를 유지한다"
    );
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(text(&reopened, 0, 0), title);
        assert_eq!(text(&reopened, 1, 0), edited);
        assert_eq!(text(&reopened, 1, 1), other_body);
    }
    core.delete_text_in_cell_native(0, 4, 0, 1, 0, 3, 2)
        .unwrap();
    assert_eq!(text(&core, 1, 0), body);
    assert_eq!(text(&core, 0, 0), title);
}
