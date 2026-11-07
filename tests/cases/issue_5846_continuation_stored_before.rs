//! 검사 유형: 관계형 1개(원본 앞 간격·뒤 문단·내용 소유), 고정형 1개(독립 Print 기준선 ±2px).
//! 주석 갱신: 2026-11-07 12:02:43 KST.
//! Native/fresh WASM 전66쪽 최저 99.21550% / 99.21467%, 검사 대상32쪽은 양쪽100%.
//! 입력: samples/task2097/75544_pii_bunseok.hwpx.
//! 독립 기준: pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf의32쪽 텍스트 행렬 원점.
//! 근거: mydocs/pr/assets/issue5846/continuation-before-space-20261107/README.md.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn walk<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        walk(c, out)
    }
}
fn core() -> DocumentCore {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx")).unwrap(),
    )
    .unwrap();
    let env = std::fs::read_to_string(root.join(
        "mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json",
    ))
    .unwrap();
    core.set_font_environment(Some(
        rhwp::renderer::font_environment::FontEnvironment::from_json(&env).unwrap(),
    ))
    .unwrap();
    core
}

#[test]
fn issue_5846_stored_continuation_paragraph_owns_its_source_before_space_once() {
    let core = core();
    let page = core.build_page_render_tree(31).unwrap();
    let mut nodes = vec![];
    walk(&page.root, &mut nodes);
    let table_node = nodes
        .iter()
        .copied()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if table.row_count == 1 && table.col_count == 1 && table.cell_context.is_none())
        })
        .expect("바깥 1×1 저장 표");
    let RenderNodeType::Table(meta) = &table_node.node_type else {
        unreachable!()
    };
    let Control::Table(source) = &core.document().sections[meta.section_index.unwrap()].paragraphs
        [meta.para_index.unwrap()]
    .controls[meta.control_index.unwrap()] else {
        unreachable!()
    };
    let owner = table_node
        .children
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TableCell(_)))
        .unwrap();
    let RenderNodeType::TableCell(cell_meta) = &owner.node_type else {
        unreachable!()
    };
    let source_cell = &source.cells[cell_meta.model_cell_index.unwrap() as usize];
    let pi = source_cell
        .paragraphs
        .iter()
        .position(|p| {
            p.text
                .starts_with("② 개인정보처리자 및 수탁처리자는 개인정보보호 담당관")
        })
        .unwrap();
    let para = &source_cell.paragraphs[pi];
    let stored = &para.line_segs[0];
    assert!(stored.vertical_pos > 0);
    let mut owned = vec![];
    walk(owner, &mut owned);
    let line = owned
        .iter()
        .copied()
        .find(|n| matches!(&n.node_type,RenderNodeType::TextLine(t) if t.para_index==Some(pi)))
        .unwrap();
    let top = if source_cell.apply_inner_margin {
        source_cell.padding.top
    } else {
        source.padding.top
    };
    let source_before_px = f64::from(stored.vertical_pos) * 96.0 / 7200.0;
    let placed_before_px = line.bbox.y - owner.bbox.y - f64::from(top) * 96.0 / 7200.0;
    assert!(
        (placed_before_px - source_before_px).abs() < 0.05,
        "저장 프레임의 앞 간격을 정확히 한 번 소비: actual={placed_before_px}, source={source_before_px}"
    );
    // 뒤 문단은 같은 저장 프레임의 절대 원점을 쓴다. 앞 간격 보정이 문서 전체를 밀지 않는다.
    let next = owned.iter().copied().find(|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(pi + 1))
    }).expect("같은 프레임의 뒤 문단");
    let next_source = &source_cell.paragraphs[pi + 1].line_segs[0];
    let next_relative = next.bbox.y - owner.bbox.y - f64::from(top) * 96.0 / 7200.0;
    assert!((next_relative - f64::from(next_source.vertical_pos) * 96.0 / 7200.0).abs() < 0.05);
    let actual: String = owned
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                (run.para_index == Some(pi)).then(|| run.display_or_text())
            } else {
                None
            }
        })
        .collect();
    let compact = |value: &str| {
        value
            .chars()
            .filter(|ch| !ch.is_whitespace() && !ch.is_control())
            .collect::<String>()
    };
    assert_eq!(
        compact(&actual),
        compact(&para.text),
        "소유 문단의 내용·순서·단일 배치"
    );
}

#[test]
fn issue_5846_first_continuation_baselines_match_independent_print() {
    let core = core();
    let page = core.build_page_render_tree(31).unwrap();
    let mut nodes = vec![];
    walk(&page.root, &mut nodes);
    let first = nodes
        .iter()
        .copied()
        .find(|node| {
            if !matches!(node.node_type, RenderNodeType::TextLine(_)) {
                return false;
            }
            let text: String = node
                .children
                .iter()
                .filter_map(|child| {
                    if let RenderNodeType::TextRun(run) = &child.node_type {
                        Some(run.display_or_text())
                    } else {
                        None
                    }
                })
                .collect();
            text.chars()
                .filter(|ch| !ch.is_whitespace())
                .collect::<String>()
                .starts_with("②개인정보처리자및수탁처리자는개인정보보호담당관")
        })
        .expect("계속되는 조항의 첫 글줄");
    let RenderNodeType::TextLine(first_meta) = &first.node_type else {
        unreachable!()
    };
    let actual: Vec<_> = nodes
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::TextLine(line) = &node.node_type {
                (line.para_index == first_meta.para_index).then_some(node.bbox.y + line.baseline)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(actual.len(), 3, "독립 Print의 세 줄 소유");
    // 독립 PDF의72pt 텍스트 행렬 원점을96DPI로 환산했다. 후보 좌표 재인용이 아니다.
    for (actual, pdf) in actual.into_iter().zip([97.119995, 123.519999, 149.919993]) {
        assert!(
            (actual - pdf).abs() <= 2.0,
            "독립 Print 기준선 ±2px: actual={actual}, pdf={pdf}"
        );
    }
}
