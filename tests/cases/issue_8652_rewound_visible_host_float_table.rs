//! 관계형·고정형(쪽수): 저장 vpos 가 되감긴 비-TAC 문단 기준 표의 호스트 글줄이 표와 같은 쪽·표 위에 놓이고 내용이 보존되는지, 전체 쪽수가 독립 PDF 와 같은지 검사한다.
//! 주석 갱신: 2026-11-06T11:55:31+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-06 KST, source = upstream/devel 170b886f16 + #8652·#8657 수정(커밋 f2340ed58d 코드와 동일).
//! 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw, 독립 한컴 2020 Print
//! `pdf/issue8652/1740000-201700020_legislation_interim_summary-2020.pdf`(10쪽).
//! 입력 `samples/issue8652/1740000-201700020_legislation_interim_summary.hwp`(10k 공개 연구용역 보고서), 전체 10쪽=10쪽.
//! 수정 전 Native: 1쪽 69.01·7쪽 99.00·8쪽 30.25(전쪽 최저 30.25).
//! 수정 후 Native 전쪽 최저 99.17(1쪽 99.94·7쪽 100.00·8쪽 99.99) / fresh WASM 전쪽 최저 99.97(1·7·8쪽 100.00).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8652/{native,wasm,png}/`.
//! 확대 판독: 8쪽 제목 「<단계별 소요기간>」 102.2pt·「입법과정」 133.9pt(PDF 102.1·134.1).
//! 같은 assertion 이 수정 전 FAIL(제목 줄 7쪽 index 6, 표 8쪽 index 7) / 수정 후 PASS.
//! 독립 절대 px 좌표를 고정하지 않는다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8652/1740000-201700020_legislation_interim_summary.hwp";
const TITLE: &str = "<단계별 소요기간>";
/// 제목 글줄과 13×2 표를 함께 가진 호스트 문단.
const HOST_PARA: usize = 66;

fn open(path: &str) -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

fn text_of(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        text_of(child, out);
    }
}

/// 쪽의 본문 텍스트 줄(표 셀 밖)과 그 상자.
fn body_lines(node: &RenderNode, in_table: bool, out: &mut Vec<(String, BoundingBox)>) {
    let in_table = in_table || matches!(node.node_type, RenderNodeType::Table(_));
    if !in_table && matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        text_of(node, &mut text);
        out.push((text.trim().to_string(), node.bbox));
        return;
    }
    for child in &node.children {
        body_lines(child, in_table, out);
    }
}

fn host_table(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(HOST_PARA) && table.cell_context.is_none() {
            return Some(node.bbox);
        }
    }
    node.children.iter().find_map(host_table)
}

#[test]
fn rewound_host_title_line_stays_with_its_table() {
    let core = open(SOURCE);
    assert_eq!(core.page_count(), 10, "독립 Print 와 같은 10쪽");
    let mut title_pages = Vec::new();
    let mut table_pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽 배치");
        let mut lines = Vec::new();
        body_lines(&tree.root, false, &mut lines);
        let titles: Vec<_> = lines.iter().filter(|(text, _)| text == TITLE).collect();
        let table = host_table(&tree.root);
        if !titles.is_empty() {
            title_pages.push((page, titles[0].1));
        }
        if let Some(table) = table {
            table_pages.push((page, table));
        }
    }
    assert_eq!(
        title_pages.len(),
        1,
        "제목 글줄은 누락·중복 없이 한 번: {title_pages:?}"
    );
    assert!(!table_pages.is_empty(), "호스트 표가 배치된다");
    let (title_page, title) = title_pages[0];
    let (first_table_page, table) = table_pages[0];
    assert_eq!(
        title_page, first_table_page,
        "한/글은 제목 줄과 표를 같은 쪽에서 시작한다(저장 vpos 되감김)"
    );
    assert!(
        title.y + title.height <= table.y + 0.5,
        "제목 줄은 표 위에 있다: title={title:?}, table={table:?}"
    );
}
