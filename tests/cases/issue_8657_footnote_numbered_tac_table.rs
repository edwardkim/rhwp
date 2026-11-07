//! 관계형: 각주 번호와 같은 문단의 글자처럼 취급 표가 각주 영역 안·번호 뒤에 그려지고 내용이 보존되는지 검사한다.
//! 주석 갱신: 2026-11-06T11:55:31+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-06 KST, source = upstream/devel 170b886f16 + #8652·#8657 수정(커밋 f2340ed58d 코드와 동일).
//! 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw, 독립 한컴 2020 Print
//! `pdf/issue8652/1740000-201700020_legislation_interim_summary-2020.pdf`(10쪽).
//! 입력 `samples/issue8652/1740000-201700020_legislation_interim_summary.hwp`(10k 공개 연구용역 보고서), 전체 10쪽=10쪽.
//! 수정 전 Native: 1쪽 69.01·7쪽 99.00·8쪽 30.25(전쪽 최저 30.25).
//! 수정 후 Native 전쪽 최저 99.17(1쪽 99.94·7쪽 100.00·8쪽 99.99) / fresh WASM 전쪽 최저 99.97(1·7·8쪽 100.00).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8652/{native,wasm,png}/`.
//! 확대 판독: 1쪽 각주 표 왼쪽 97.1pt(PDF 괘선 97.5)·번호 「1)」 기준선 729.1pt, 15행 내용·굵은 행이 PDF 와 겹친다.
//! 같은 assertion 이 수정 전 FAIL(각주 영역에 표 0개) / 수정 후 PASS.
//! 독립 절대 px 좌표를 고정하지 않는다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8652/1740000-201700020_legislation_interim_summary.hwp";

fn open(path: &str) -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

fn footnote_area(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        return Some(node);
    }
    node.children.iter().find_map(footnote_area)
}

fn collect<'a>(
    node: &'a RenderNode,
    tables: &mut Vec<BoundingBox>,
    runs: &mut Vec<(&'a str, BoundingBox)>,
) {
    match &node.node_type {
        RenderNodeType::Table(_) => tables.push(node.bbox),
        RenderNodeType::TextRun(run) => runs.push((run.text.as_str(), node.bbox)),
        _ => {}
    }
    for child in &node.children {
        collect(child, tables, runs);
    }
}

#[test]
fn numbered_footnote_tac_table_is_painted_after_its_number() {
    let core = open(SOURCE);
    let tree = core.build_page_render_tree(0).expect("1쪽 배치");
    let area = footnote_area(&tree.root).expect("1쪽 각주 영역");
    let mut tables = Vec::new();
    let mut runs = Vec::new();
    collect(area, &mut tables, &mut runs);

    assert_eq!(tables.len(), 1, "각주 1) 의 표가 누락·중복 없이 그려진다");
    let table = tables[0];
    assert!(
        table.y + 0.5 >= area.bbox.y
            && table.y + table.height <= area.bbox.y + area.bbox.height + 0.5,
        "표는 각주 영역 세로 범위 안에 있다: table={table:?}, area={:?}",
        area.bbox
    );

    let text: String = runs.iter().map(|(text, _)| *text).collect();
    for label in [
        "제1조",
        "목적",
        "제9조의2",
        "방호진단",
        "제11조",
        "시행세칙",
    ] {
        assert!(
            text.contains(label),
            "각주 표 내용 「{label}」 보존: {text}"
        );
    }

    // 번호 「1)」은 표 왼쪽에 있고 표는 번호 글자 뒤에서 시작한다.
    let (number_text, number) = runs
        .iter()
        .find(|(text, bbox)| text.trim_start().starts_with("1)") && bbox.x < table.x)
        .expect("각주 번호 「1)」");
    // 번호 줄은 「1)」 뒤 공백까지 한 런일 수 있다. 「1)」 두 글자 몫의 끝만 쓴다.
    let chars = number_text.chars().count().max(1) as f64;
    let number_glyph_right = number.x + number.width * 2.0 / chars;
    assert!(
        table.x + 0.5 >= number_glyph_right,
        "표는 번호 글자 뒤에서 시작한다: number={number:?}, table={table:?}"
    );
}
