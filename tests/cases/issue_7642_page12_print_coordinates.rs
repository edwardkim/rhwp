//! #1880 12쪽 하단 문구: 독립 Print 좌표와 원본 소유·내용을 함께 보호한다.
//! 독립 한컴 Print PDF: pdf/issue7642-roundtrip-contract-print-20261014/
//! issue1880_anchor_stack_sb_convert-2020.pdf (96dpi, 12쪽).
//! 소스 810654da2: 전 13쪽 최저 Native 94.74646%, WASM 94.77676%;
//! 12쪽 Native 99.89607%, WASM 100%. 단계 1331·1332의 직접 판독 기록 참조.
//! 같은 비밀 표·하단 문구의 경계는 이 쪽에만 있어 다른 쪽 점수로 대체하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body)
}

fn owned_runs<'a>(node: &'a RenderNode, owner: (usize, usize), out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if (run.section_index, run.para_index) == (Some(owner.0), Some(owner.1))
            && run.cell_context.is_none())
    {
        out.push(node);
    }
    for child in &node.children {
        owned_runs(child, owner, out);
    }
}

fn owned_tables<'a>(
    node: &'a RenderNode,
    owner: (usize, usize, usize),
    out: &mut Vec<&'a RenderNode>,
) {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if (table.section_index, table.para_index, table.control_index)
            == (Some(owner.0), Some(owner.1), Some(owner.2)))
    {
        out.push(node);
    }
    for child in &node.children {
        owned_tables(child, owner, out);
    }
}

#[test]
fn issue7642_anchor_stack_page12_footer_keeps_print_position_and_source_ownership() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1880_anchor_stack_sb_convert.hwpx");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("공식 원본")).expect("원본 문서 열기");
    let sources: Vec<_> = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, data)| {
            data.paragraphs
                .iter()
                .enumerate()
                .filter(|(_, para)| para.text.trim() == "< 별지 제6호 서식 >")
                .map(move |(paragraph, para)| ((section, paragraph), para.text.as_str()))
        })
        .collect();
    assert_eq!(sources.len(), 1, "내용으로 식별한 단일 원본 문단");
    let (owner, original) = sources[0];
    let table_sources: Vec<_> = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, data)| {
            data.paragraphs
                .iter()
                .enumerate()
                .flat_map(move |(paragraph, para)| {
                    para.controls
                        .iter()
                        .enumerate()
                        .filter_map(move |(control, item)| match item {
                            Control::Table(table)
                                if table.cells.iter().any(|cell| {
                                    cell.paragraphs.iter().any(|p| p.text.contains("Ⅱ급비밀용"))
                                }) =>
                            {
                                Some((section, paragraph, control))
                            }
                            _ => None,
                        })
                })
        })
        .collect();
    assert_eq!(table_sources.len(), 1, "내용으로 식별한 단일 원본 비밀 표");
    let table_owner = table_sources[0];
    assert_eq!(core.page_count(), 13, "독립 Print 전체 쪽수");
    let mut owned_pages = Vec::new();
    let mut table_pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("실제 쪽 출력");
        let mut tables = Vec::new();
        owned_tables(&tree.root, table_owner, &mut tables);
        if !tables.is_empty() {
            assert_eq!(tables.len(), 1, "원본 표를 같은 쪽에 한 번만 출력한다");
            table_pages.push(page);
        }
        let mut runs = Vec::new();
        owned_runs(&tree.root, owner, &mut runs);
        if runs.is_empty() {
            continue;
        }
        owned_pages.push(page);
        let content: String = runs
            .iter()
            .map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) => run.text.as_str(),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(content, original, "원본 문자 순서·누락·중복 보존");
        let body = body(&tree.root).expect("본문 소유 영역");
        let mut body_runs = Vec::new();
        owned_runs(body, owner, &mut body_runs);
        assert_eq!(body_runs.len(), runs.len(), "하단 문구는 본문에만 소속한다");
        for run in &runs {
            assert!(
                run.bbox.x >= body.bbox.x - 0.001
                    && run.bbox.x + run.bbox.width <= body.bbox.x + body.bbox.width + 0.001
                    && run.bbox.y >= body.bbox.y - 0.001
                    && run.bbox.y + run.bbox.height <= body.bbox.y + body.bbox.height + 0.001,
                "원본 문구를 본문 밖으로 밀거나 잘라내지 않는다"
            );
        }
        let mut previous_end = None;
        for node in &runs {
            let RenderNodeType::TextRun(run) = &node.node_type else {
                unreachable!()
            };
            if let Some(end) = previous_end {
                assert!(
                    node.bbox.x >= end - 0.001,
                    "같은 줄의 앞뒤 런은 겹치지 않는다"
                );
            }
            previous_end = Some(node.bbox.x + node.bbox.width);
            // PDF cm=[.12,0,0,-.12,0,841], tm y=6328: 상단 기준 759.36pt.
            let baseline = node.bbox.y + run.baseline;
            assert!(
                (baseline - 1012.48).abs() <= 2.0,
                "독립 Print 기준선: {baseline}"
            );
        }
        // PDF의 '< ' 시작 tm x=472 × .12pt × 96/72 = 75.52 CSS px.
        assert!(
            (runs[0].bbox.x - 75.52).abs() <= 2.0,
            "독립 Print 시작 위치"
        );
        assert_eq!(tables.len(), 1, "표와 하단 문구는 같은 쪽에 소속한다");
        let table = tables[0];
        let RenderNodeType::Table(data) = &table.node_type else {
            unreachable!()
        };
        assert_eq!(
            (data.row_count, data.col_count),
            (2, 2),
            "독립 Print의 표 구조"
        );
        // PDF 경로 좌표: x=247.667pt, 상단 y=469.047pt, 하단 y=430.569pt.
        // A4 높이 841pt에서 상단 원점으로 바꾸고 96/72를 곱한다.
        assert!(
            (table.bbox.x - 330.2226666667).abs() <= 2.0,
            "독립 Print 표 왼쪽"
        );
        assert!(
            (table.bbox.y - 495.9373333333).abs() <= 2.0,
            "독립 Print 표 상단"
        );
        assert!(
            (table.bbox.y + table.bbox.height - 547.2413333333).abs() <= 2.0,
            "독립 Print 표 하단"
        );
        assert!(
            table.bbox.y + table.bbox.height < runs[0].bbox.y,
            "본문 표 뒤에 하단 문구를 겹침 없이 보존한다"
        );
    }
    assert_eq!(owned_pages, [11], "12쪽에만 원본 문단을 한 번 출력한다");
    assert_eq!(table_pages, [11], "원본 비밀 표도 12쪽에만 소속한다");
}
