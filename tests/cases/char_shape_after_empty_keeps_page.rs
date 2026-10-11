//! 빈 문단 뒤 서식 변경의 최종 배치·저장 왕복 및 실제 저장 쪽 경계 계약.
//! 독립 한컴 Print PDF와 입력 생성 절차는 fixtures/pr7468/README.md에 보존한다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

#[derive(Debug)]
struct RenderedLine {
    paragraph: usize,
    page: u32,
    y: f64,
    height: f64,
    text: String,
}

fn rendered_lines(core: &DocumentCore) -> Vec<RenderedLine> {
    fn text(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(run.display_or_text());
        }
        for child in &node.children {
            text(child, out);
        }
    }
    fn walk(node: &RenderNode, page: u32, out: &mut Vec<RenderedLine>) {
        if let RenderNodeType::TextLine(info) = &node.node_type {
            if let Some(paragraph) = info.para_index {
                let mut content = String::new();
                text(node, &mut content);
                out.push(RenderedLine {
                    paragraph,
                    page,
                    y: node.bbox.y,
                    height: node.bbox.height,
                    text: content,
                });
            }
        }
        for child in &node.children {
            walk(child, page, out);
        }
    }
    let mut lines = Vec::new();
    for page in 0..core.page_count() {
        walk(
            &core.build_page_render_tree(page).unwrap().root,
            page,
            &mut lines,
        );
    }
    lines
}

fn close(actual: f64, expected: f64, context: &str) {
    assert!(
        (actual - expected).abs() < 0.05,
        "{context}: {actual} != {expected}"
    );
}

fn reopened(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap()
}

fn assert_empty_paragraph_geometry(core: &DocumentCore, deleted: bool) -> Vec<RenderedLine> {
    let expected = if deleted {
        vec!["Title", "A", "B", "C", "D"]
    } else {
        vec!["Title", "A", "B", "", "C", "D"]
    };
    let lines = rendered_lines(core);
    assert_eq!(
        core.page_count(),
        1,
        "empty paragraph must not create a page"
    );
    assert_eq!(
        lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>(),
        expected,
        "rendered content order, omission and duplication"
    );
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(
            (line.paragraph, line.page),
            (index, 0),
            "paragraph ownership"
        );
    }
    for pair in lines.windows(2) {
        assert!(
            pair[0].y + pair[0].height <= pair[1].y + 0.05,
            "text lines overlap: {pair:?}"
        );
    }
    // 입력의 13pt 글자와 160% 문단 줄간격이 정하는 전진량이다.
    // 실제 출력에서 읽은 절대 C/D 좌표를 golden으로 고정하지 않는다.
    let pitch_13pt = 13.0 * 96.0 / 72.0 * 1.6;
    let empty_pitch_20pt = 20.0 * 96.0 / 72.0 * 1.6;
    let c = if deleted { 3 } else { 4 };
    close(lines[c + 1].y - lines[c].y, pitch_13pt, "C/D line advance");
    close(
        lines[c].y - lines[2].y,
        pitch_13pt + if deleted { 0.0 } else { empty_pitch_20pt },
        "B/C empty-line space",
    );
    lines
}

fn exercise_empty_paragraph(by_id: bool) {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp")).unwrap();
    core.insert_text_native(0, 0, 0, "Title").unwrap();
    core.apply_char_format_native(0, 0, 0, 5, r#"{"fontSize":2000}"#)
        .unwrap();
    for (index, text) in ["A", "B", "", "C", "D"].iter().enumerate() {
        core.insert_paragraph_native(0, index + 1).unwrap();
        if !text.is_empty() {
            core.insert_text_native(0, index + 1, 0, text).unwrap();
        }
    }
    core.apply_char_format_native(0, 2, 0, 1, r#"{"fontSize":1300,"bold":true}"#)
        .unwrap();
    if by_id {
        let id = core.document().sections[0].paragraphs[2].char_shapes[0].char_shape_id;
        core.set_char_shape_id_native(0, 4, 0, 1, id).unwrap();
    } else {
        core.apply_char_format_native(0, 4, 0, 1, r#"{"fontSize":1300,"bold":true}"#)
            .unwrap();
    }
    let paragraphs = &core.document().sections[0].paragraphs;
    let empty_end = paragraphs[3].line_segs.last().unwrap();
    let following_start = paragraphs[4].line_segs.first().unwrap();
    assert!(
        following_start.vertical_pos >= empty_end.vertical_pos + empty_end.line_height,
        "following paragraph starts before the empty paragraph ends"
    );
    let before = assert_empty_paragraph_geometry(&core, false);
    assert_empty_paragraph_geometry(&reopened(&core), false);
    core.delete_paragraph_native(0, 3).unwrap();
    let after = assert_empty_paragraph_geometry(&core, true);
    let restored = assert_empty_paragraph_geometry(&reopened(&core), true);
    for (old, new) in before[4..].iter().zip(&after[3..]) {
        close(
            old.y - new.y,
            20.0 * 96.0 / 72.0 * 1.6,
            "deleted empty-line advance",
        );
    }
    for (live, saved) in after.iter().zip(&restored) {
        close(saved.y - live.y, 0.0, "save/reopen line origin");
    }
}

#[test]
fn char_size_after_empty_paragraph_stays_on_one_page() {
    exercise_empty_paragraph(false);
}

#[test]
fn char_shape_id_after_empty_preserves_following_lines_and_reopen() {
    exercise_empty_paragraph(true);
}

#[test]
fn real_stored_zero_still_starts_a_page() {
    // 한컴 2020이 실제 재저장한 70줄/2쪽 대조군. 합성 tag를 수동 패치하지 않았다.
    // sample16의 기존 전쪽 피델리티·65/64쪽 문제는 #7445에 남아 있다.
    let bytes = include_bytes!("../fixtures/pr7468/stored-control.hwp");
    let original = DocumentCore::from_bytes(bytes).unwrap();
    let paragraphs = &original.document().sections[0].paragraphs;
    let boundary = (1..paragraphs.len())
        .find(|&index| {
            matches!((paragraphs[index - 1].line_segs.last(), paragraphs[index].line_segs.first()),
            (Some(prev), Some(next)) if prev.vertical_pos > 5000 && next.vertical_pos == 0
                && prev.tag & 0x8000_0000 == 0 && next.tag & 0x8000_0000 == 0)
        })
        .expect("authentic stored page boundary");
    let baseline = rendered_lines(&original);
    assert_eq!(
        original.page_count(),
        2,
        "independent Hancom Print PDF has two pages"
    );
    assert_eq!(baseline.len(), 70);
    assert_eq!(
        (baseline[boundary - 1].page, baseline[boundary].page),
        (0, 1)
    );
    for (index, line) in baseline.iter().enumerate() {
        assert_eq!(line.text, format!("Line {:03}", index + 1));
        assert_eq!(line.paragraph, index);
    }
    for index in [boundary - 1, boundary] {
        for by_id in [false, true] {
            let mut core = DocumentCore::from_bytes(bytes).unwrap();
            let id = core.document().sections[0].paragraphs[index].char_shapes[0].char_shape_id;
            let size = core.document().doc_info.char_shapes[id as usize].base_size;
            if by_id {
                core.set_char_shape_id_native(0, index, 0, 1, id).unwrap();
            } else {
                core.apply_char_format_native(0, index, 0, 1, &format!(r#"{{"fontSize":{size}}}"#))
                    .unwrap();
            }
            assert_eq!(
                core.document().sections[0].paragraphs[boundary].line_segs[0].vertical_pos,
                0
            );
            for state in [&core, &reopened(&core)] {
                assert_eq!(
                    state.page_count(),
                    2,
                    "adjacent format edit preserves total pages"
                );
                let actual = rendered_lines(state);
                assert_eq!(
                    actual.len(),
                    baseline.len(),
                    "no missing or duplicated lines"
                );
                for (before, after) in baseline.iter().zip(&actual) {
                    assert_eq!(
                        (after.paragraph, after.page, &after.text),
                        (before.paragraph, before.page, &before.text),
                        "stored-boundary ownership/order"
                    );
                    close(
                        after.y - before.y,
                        0.0,
                        "same-format edit preserves stored origin",
                    );
                }
            }
        }
    }
}
