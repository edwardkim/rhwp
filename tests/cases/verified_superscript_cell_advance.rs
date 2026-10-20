//! #5756: 독립 Print의 원문 줄이 위첨자와 본문을 같은 셀 안에 보존한다.
//! 96dpi 전 3쪽 Native/fresh WASM 및 확대 판독 근거는 단계 1331 기록에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn check_original_line(
    node: &RenderNode,
    cell: Option<&RenderNode>,
    core: &DocumentCore,
    page: u32,
    owners: &mut Vec<u32>,
) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let runs: Vec<_> = node
            .children
            .iter()
            .filter_map(|child| match &child.node_type {
                RenderNodeType::TextRun(run) => Some((child, run)),
                _ => None,
            })
            .collect();
        let text: String = runs.iter().map(|(_, run)| run.text.as_str()).collect();
        if text.contains("39.3") && text.contains("78.8%") {
            let cell = cell.expect("위첨자 문장은 원본 표 셀에 소속한다");
            let context = runs[0].1.cell_context.as_ref().expect("원본 셀 경로");
            assert_eq!(context.path.len(), 1, "이 원본은 중첩되지 않은 표다");
            let entry = &context.path[0];
            let section = runs[0].1.section_index.expect("원본 구역 소유");
            let parent = &core.document().sections[section].paragraphs[context.parent_para_index];
            let Control::Table(table) = &parent.controls[entry.control_index] else {
                panic!("원본 경로는 표를 가리켜야 한다");
            };
            let source = &table.cells[entry.cell_index].paragraphs[entry.cell_para_index];
            assert!(source.text.contains("39.3") && source.text.contains("78.8%"));
            let characters: Vec<_> = source.text.chars().collect();
            let mut cursor = runs[0].1.char_start.expect("첫 원문 문자 위치");
            let mut prior_end = None;
            let mut superscripts = 0;
            let mut body_runs = 0;
            let roundoff = f64::EPSILON * cell.bbox.width.max(1.0) * 128.0;
            for (child, run) in runs {
                let owner = run.cell_context.as_ref().expect("각 런의 셀 소유");
                assert_eq!(owner.parent_para_index, context.parent_para_index);
                assert_eq!(owner.path, context.path);
                assert_eq!(run.section_index, Some(section));
                let start = run.char_start.expect("원문 문자 위치");
                assert!(start >= cursor, "원문 문자 순서 역전·중복 방지");
                assert!(
                    characters[cursor..start].iter().all(|c| c.is_whitespace()),
                    "런 사이의 가시 내용 누락 방지"
                );
                let end = start + run.text.chars().count();
                let expected: String = characters[start..end].iter().collect();
                assert_eq!(run.text, expected, "같은 원문 범위의 내용 보존");
                cursor = end;
                let saved = &core.document().doc_info.char_shapes
                    [run.char_shape_id.expect("원본 글자 모양") as usize];
                assert_eq!(run.style.superscript, saved.superscript);
                if run.style.superscript {
                    superscripts += 1;
                } else if !run.text.trim().is_empty() {
                    body_runs += 1;
                }
                assert!(
                    child.bbox.x + roundoff >= cell.bbox.x
                        && child.bbox.x + child.bbox.width
                            <= cell.bbox.x + cell.bbox.width + roundoff,
                    "문자 전진은 소유 셀의 좌우 경계 안에 있어야 한다"
                );
                if let Some(end) = prior_end {
                    assert!(child.bbox.x + roundoff >= end, "앞뒤 런은 겹치지 않는다");
                }
                prior_end = Some(child.bbox.x + child.bbox.width);
            }
            assert!(
                superscripts > 0 && body_runs > 0,
                "본문과 위첨자를 함께 검증한다"
            );
            assert!(
                node.bbox.y + roundoff >= cell.bbox.y
                    && node.bbox.y + node.bbox.height <= cell.bbox.y + cell.bbox.height + roundoff,
                "원문 줄은 같은 셀의 세로 점유에 포함된다"
            );
            owners.push(page);
        }
    }
    for child in &node.children {
        check_original_line(child, cell, core, page, owners);
    }
}

#[test]
fn print_verified_superscript_advance_preserves_cell_ownership_content_and_non_overlap() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5756/156732409_superscript_advance.hwp");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("공식 원본")).expect("원본 문서 열기");
    assert_eq!(core.page_count(), 3, "독립 Print의 전체 쪽수");
    let mut owners = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("실제 쪽 출력");
        check_original_line(&tree.root, None, &core, page, &mut owners);
    }
    assert_eq!(owners, [2], "원본 문장은 Print의 3쪽에 한 번만 출력한다");
}
