//! #8153: 저장 쪽 재시작 앞의 조각은 빈 셀과 빈 물리 띠도 함께 소유한다.
//! 314쪽 독립 Print 세로선 끝 483.197998pt를 96dpi에서 ±2px로 대조한다.
//! 근거: mydocs/plans/issue_8153_fragment_physical_band_20261025.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn text(node: &RenderNode) -> String {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.as_str(),
        _ => "",
    };
    own.to_owned() + &node.children.iter().map(text).collect::<String>()
}

fn table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(meta) if meta.row_count == 103 && meta.col_count == 2)
    {
        Some(node)
    } else {
        node.children.iter().find_map(table)
    }
}

fn row(table: &RenderNode) -> Vec<&RenderNode> {
    let mut cells: Vec<_> = table
        .children
        .iter()
        .filter(|cell| {
            matches!(&cell.node_type, RenderNodeType::TableCell(meta) if meta.row == 9 && meta.page_fragment)
        })
        .collect();
    cells.sort_by_key(|cell| match &cell.node_type {
        RenderNodeType::TableCell(meta) => meta.col,
        _ => unreachable!(),
    });
    assert_eq!(cells.len(), 2, "같은 원본 행의 두 조각 셀");
    cells
}

fn verify(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383);
    let first = core.build_page_render_tree(313).unwrap();
    let first = row(table(&first.root).expect("314쪽 법령 대조 표"));
    let expected_bottom = 483.197998 * 96.0 / 72.0;
    for cell in &first {
        let bottom = cell.bbox.y + cell.bbox.height;
        assert!(
            (bottom - expected_bottom).abs() <= 2.0,
            "독립 Print의 조각 하단: 실제 {bottom}, 기대 {expected_bottom}, 허용 ±2px"
        );
    }
    assert!((first[0].bbox.y - first[1].bbox.y).abs() <= 2.0);
    assert!((first[0].bbox.height - first[1].bbox.height).abs() <= 2.0);
    let opening = text(first[0]);
    assert!(opening.starts_with("제5조(문서 처리의 기본 원칙)"));
    assert!(opening.ends_with("전자문서"));
    assert!(
        text(first[1]).is_empty(),
        "완료한 빈 셀도 물리 프레임을 보존한다"
    );
    let next = core.build_page_render_tree(314).unwrap();
    let next = row(table(&next.root).expect("315쪽의 같은 표"));
    let following = text(next[0]);
    assert!(following.starts_with("시스템 또는 업무관리시스템"));
    assert!(following.ends_with("접근할 수 있도록 할 것"));
    assert_eq!(opening.matches("제5조(").count(), 1);
    assert!(
        !following.contains("제5조("),
        "앞 조각의 제목을 중복하지 않는다"
    );
    assert!(text(next[1]).is_empty());
}

#[test]
fn hwp_fragment_keeps_the_source_blank_band_and_following_content() {
    verify("hwp");
}

#[test]
fn hwpx_fragment_keeps_the_source_blank_band_and_following_content() {
    verify("hwpx");
}
