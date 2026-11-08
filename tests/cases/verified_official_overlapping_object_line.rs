//! 한컴 Print로 확인한 저장 줄의 개체 소유와 후속 본문 관계를 보호한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn append_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        append_text(child, text);
    }
}

fn has_title(node: &RenderNode) -> bool {
    let mut text = String::new();
    append_text(node, &mut text);
    text.contains("2026년 9월 고용동향")
}

fn collect<'a>(
    node: &'a RenderNode,
    parent_is_column: bool,
    body: &mut Vec<&'a RenderNode>,
    groups: &mut Vec<&'a RenderNode>,
    tables: &mut Vec<&'a RenderNode>,
    spacers: &mut Vec<&'a RenderNode>,
) {
    match &node.node_type {
        RenderNodeType::Body { .. } => body.push(node),
        RenderNodeType::Group(group)
            if group.section_index == Some(2)
                && group.para_index == Some(0)
                && group.control_index == Some(3)
                && has_title(node) =>
        {
            groups.push(node)
        }
        RenderNodeType::Table(table)
            if table.section_index == Some(2)
                && table.para_index == Some(2)
                && table.cell_context.is_none() =>
        {
            tables.push(node)
        }
        RenderNodeType::TextLine(line)
            if parent_is_column && line.section_index == Some(2) && line.para_index == Some(1) =>
        {
            spacers.push(node)
        }
        _ => {}
    }
    for child in &node.children {
        collect(
            child,
            matches!(&node.node_type, RenderNodeType::Column(_)),
            body,
            groups,
            tables,
            spacers,
        );
    }
}

#[test]
fn stored_control_only_line_owns_banner_before_summary_table() {
    let input = include_bytes!(
        "../../mydocs/pr/assets/pr8319_evidence_alignment_20261027/official-inputs/ssec2609.hwpx"
    );
    let core = DocumentCore::from_bytes(input).expect("통계청 공식 배포 HWPX");
    let mut matching_pages = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let (mut bodies, mut groups, mut tables, mut spacers) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        collect(
            &tree.root,
            false,
            &mut bodies,
            &mut groups,
            &mut tables,
            &mut spacers,
        );
        if groups.is_empty() {
            continue;
        }
        matching_pages += 1;
        assert_eq!(groups.len(), 1, "요약 배너는 누락·중복 없이 한 번 표시");
        assert_eq!(bodies.len(), 1, "요약 쪽의 본문 영역");
        assert_eq!(tables.len(), 1, "요약 표는 배너와 같은 쪽에 보존");
        assert_eq!(spacers.len(), 1, "배너 뒤 빈 문단의 줄을 보존");
        let banner = &groups[0].bbox;
        let body = &bodies[0].bbox;
        let table = &tables[0].bbox;
        let spacer = &spacers[0].bbox;
        assert!(banner.y >= body.y, "배너는 본문 영역 위로 벗어나지 않음");
        assert!(
            banner.y + banner.height <= spacer.y + spacer.height / 2.0,
            "빈 문단의 중심은 배너의 점유 끝 뒤에 배치"
        );
        assert!(
            spacer.y + spacer.height <= table.y,
            "빈 문단은 요약 표 앞에 보존"
        );
        assert!(
            banner.y + banner.height <= table.y,
            "배너 뒤에 요약 표가 겹치지 않음"
        );
    }
    assert_eq!(matching_pages, 1, "배너와 해당 본문은 한 쪽에 한 번 표시");
}
