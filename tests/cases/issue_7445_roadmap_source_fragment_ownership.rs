//! #7445 첫 댓글: 로드맵 302쪽 전수 시각 수용 후 원본 행 소속을 복원한다.
//! 독립 한컴 PDF와 Native/실제 fresh WASM 증적은 resolution-20261005에 있다.
//! 절대 픽셀 좌표 대신 국가 행의 쪽 소속·셀 누락/중복·끝 출처를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue5941/1490000-201600081_roadmap_research.hwp";
const SECTION: usize = 6;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("전쪽 시각 수용 후 복원한 원본");
    DocumentCore::from_bytes(&bytes).expect("원본 로드")
}

fn table_for_host(node: &RenderNode, host: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.section_index == Some(SECTION) && table.para_index == Some(host))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table_for_host(child, host))
}

fn text(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

fn assert_table_owners(host: usize, page_ranges: &[(u32, u16, u16)]) {
    let core = core();
    assert_eq!(core.page_count(), 302, "독립 PDF와 같은 원본 쪽 수");
    let Control::Table(source) = &core.document().sections[SECTION].paragraphs[host].controls[0]
    else {
        panic!("독립 PDF에서 확인한 표 호스트");
    };
    assert!(source.cells.iter().all(|cell| cell.row_span == 1));
    let mut owners = BTreeMap::<usize, usize>::new();
    for &(page, first, end) in page_ranges {
        let tree = core.build_page_render_tree(page - 1).expect("PDF 대응 쪽");
        let table = table_for_host(&tree.root, host).expect("같은 쪽의 원본 표 조각");
        let mut rows = BTreeSet::new();
        for node in &table.children {
            let RenderNodeType::TableCell(cell) = &node.node_type else {
                continue;
            };
            rows.insert(cell.row);
            let index = cell.model_cell_index.expect("원본 모델 셀 소속") as usize;
            let original = &source.cells[index];
            assert_eq!((cell.row, cell.col), (original.row, original.col));
            *owners.entry(index).or_default() += 1;
            if cell.col == 0 && cell.col_span == 1 && cell.row >= first && cell.row < end {
                let country = original.paragraphs[0].text.trim();
                assert!(
                    !country.is_empty() && text(node).contains(country),
                    "{page}쪽의 원본 국가 이름을 보존해야 합니다: {country}"
                );
            }
        }
        let expected: BTreeSet<_> = std::iter::once(0).chain(first..end).collect();
        assert_eq!(rows, expected, "독립 PDF {page}쪽의 국가 행 소속");
    }
    for (index, cell) in source.cells.iter().enumerate() {
        let expected = if cell.row == 0 { page_ranges.len() } else { 1 };
        assert_eq!(
            owners.get(&index).copied(),
            Some(expected),
            "머리행은 각 조각에 한 번, 데이터 셀은 문서 전체에서 한 번만 소유: 행 {} 열 {}",
            cell.row,
            cell.col
        );
    }
}

#[test]
fn fixed_term_table_keeps_finland_on_the_second_fragment() {
    // PDF 296쪽은 Estonia로 끝나고 297쪽은 Finland로 시작한다.
    assert_table_owners(
        1305,
        &[(296, 1, 9), (297, 9, 21), (298, 21, 30), (299, 30, 35)],
    );
}

#[test]
fn agency_table_keeps_germany_on_the_second_fragment_and_preserves_its_source() {
    // PDF 300쪽은 France로 끝나고 301쪽은 Germany로 시작한다.
    assert_table_owners(1306, &[(300, 1, 11), (301, 11, 24), (302, 24, 33)]);
    let core = core();
    let tree = core.build_page_render_tree(301).expect("PDF 마지막 쪽");
    let page_text = text(&tree.root);
    let last_country = page_text.find("United States").expect("마지막 국가");
    let note = page_text.find("2013년 기준 내용임").expect("표 뒤 주석");
    let source = page_text.find("OECD(2014)").expect("독립 PDF의 출처");
    assert!(
        last_country < note && note < source,
        "표 끝 → 주석 → 출처 순서"
    );
}
