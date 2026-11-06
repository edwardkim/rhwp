//! 관계형·고정형(쪽수): 글자처럼 취급한 표의 셀 각주가 표를 덮지 않게 다음 쪽 각주 영역으로 이어지는지(통째 이월·저장 재시작 분할), 내용이 누락·중복 없이 보존되는지, 전체 쪽수가 독립 PDF 와 같은지 검사한다.
//! 주석 갱신: 2026-11-06T22:11:15+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-06 KST, base = #8677 head d9174a0d09(upstream/devel 08963f6825 + #8677), head = base + #8685 수정(이 PR 의 src 변경과 동일).
//! 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw, 독립 한컴 Print(Hwp 2022 12.0.0.4547 생성).
//! (1) 최종보고서 `samples/issue8677/1411000-201000007_final_report.hwp` / `pdf/issue8677/1411000-201000007_final_report-2020.pdf`, 전체 111쪽=111쪽.
//!     수정 전 Native 27쪽 90.10·28쪽 86.07 / fresh WASM 89.41·86.99 → 수정 후 Native 99.65·99.83 / fresh WASM 99.74·99.93.
//!     수정 후 전쪽 최저 Native 91.34·fresh WASM 91.46(90% 미만 0쪽).
//! (2) 새마을운동 ODA 보고서 `samples/issue8685/1311000-201100125_saemaul_oda_pilot_report.hwp` / `pdf/issue8685/1311000-201100125_saemaul_oda_pilot_report-2020.pdf`, 전체 209쪽=209쪽.
//!     수정 전 Native 15쪽 93.39·16쪽 91.09 / fresh WASM 92.56·91.43 → 수정 후 Native 99.52·99.90 / fresh WASM 99.63·99.90.
//!     수정 후 전쪽 최저 Native 83.42(5~8쪽 목차, 수정 전과 동일, #8697)·fresh WASM 94.00.
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8685/{base,head}/{native,wasm}/`, `png/`.
//! 확대 판독: (1) 27쪽 표 3-3 아래 각주 2)·3) 만, 4) 는 28쪽 각주 영역 맨 위. (2) 15쪽 표 2-2 아래 각주 1) 앞 세 줄(「…이중 문제가」), 나머지는 16쪽.
//! 같은 assertion 이 수정 전 FAIL(두 원본 모두 「각주 영역이 표를 덮는다」: 837.9 < 914.5, 883.4 < 938.5) / 수정 후 PASS.
//! 독립 절대 px 좌표를 고정하지 않는다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

struct Case {
    source: &'static str,
    pdf_pages: u32,
    section: usize,
    host_para: usize,
    /// 각주 첫 줄의 앞부분(표 쪽 또는 다음 쪽에 있어야 하는 줄).
    first_line: &'static str,
    /// 저장 재시작 뒤 첫 줄의 앞부분(분할 사례에서만 다음 쪽에 있어야 하는 줄).
    continued_line: Option<&'static str>,
}

/// 연구용역 최종보고서 27쪽 표 3-3 셀 각주 4) — 저장 각주 줄에 재시작이 없어 각주 전체가
/// 28쪽 각주 영역 맨 위로 넘어간다(한/글 2022 Print).
const WHOLE: Case = Case {
    source: "samples/issue8677/1411000-201000007_final_report.hwp",
    pdf_pages: 111,
    section: 3,
    host_para: 17,
    first_line: "한국은행은 에너지이외 부문을",
    continued_line: None,
};

/// 새마을운동 ODA 시범사업 보고서 15쪽 표 2-2 셀 각주 1) — 저장 각주 줄이 셋째 줄 뒤에서
/// vpos 0 으로 다시 시작한다. 앞 세 줄은 15쪽 표 아래, 나머지는 16쪽 각주 영역(한/글 2022 Print).
const SPLIT: Case = Case {
    source: "samples/issue8685/1311000-201100125_saemaul_oda_pilot_report.hwp",
    pdf_pages: 209,
    section: 0,
    host_para: 292,
    first_line: "경제주체가 가격기구",
    continued_line: Some("되는 것은 부정적 외부성이다"),
};

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

fn host_table(node: &RenderNode, case: &Case) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.section_index == Some(case.section)
            && table.para_index == Some(case.host_para)
            && table.cell_context.is_none()
        {
            return Some(node.bbox);
        }
    }
    node.children
        .iter()
        .find_map(|child| host_table(child, case))
}

/// 쪽의 각주 영역 상자와 그 안 글줄들.
fn footnote_area(node: &RenderNode) -> Option<(BoundingBox, Vec<String>)> {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        let mut lines = Vec::new();
        collect_lines(node, &mut lines);
        return Some((node.bbox, lines));
    }
    node.children.iter().find_map(footnote_area)
}

fn collect_lines(node: &RenderNode, out: &mut Vec<String>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        text_of(node, &mut text);
        out.push(text.trim().to_string());
        return;
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

fn pages_with_line(core: &DocumentCore, needle: &str) -> Vec<u32> {
    (0..core.page_count())
        .filter(|&page| {
            let tree = core.build_page_render_tree(page).expect("쪽 배치");
            footnote_area(&tree.root)
                .is_some_and(|(_, lines)| lines.iter().any(|line| line.contains(needle)))
        })
        .collect()
}

fn check(case: &Case) {
    let core = open(case.source);
    assert_eq!(
        core.page_count(),
        case.pdf_pages,
        "{}: 독립 Print 와 같은 쪽수",
        case.source
    );
    let (table_page, table) = (0..core.page_count())
        .find_map(|page| {
            let tree = core.build_page_render_tree(page).expect("쪽 배치");
            host_table(&tree.root, case).map(|table| (page, table))
        })
        .expect("각주 참조를 가진 표");

    // 표가 놓인 쪽의 각주 영역은 표를 덮지 않는다.
    let tree = core.build_page_render_tree(table_page).expect("쪽 배치");
    if let Some((area, _)) = footnote_area(&tree.root) {
        assert!(
            area.y + 0.5 >= table.y + table.height,
            "{}: {table_page}쪽 각주 영역이 표를 덮는다: area={area:?} table={table:?}",
            case.source
        );
    }

    let first = pages_with_line(&core, case.first_line);
    match case.continued_line {
        None => {
            assert_eq!(
                first,
                vec![table_page + 1],
                "{}: 재시작 없는 각주는 통째로 다음 쪽 각주 영역에 한 번",
                case.source
            );
        }
        Some(continued) => {
            assert_eq!(
                first,
                vec![table_page],
                "{}: 저장 재시작 앞 조각은 표 쪽 각주 영역에 한 번",
                case.source
            );
            assert_eq!(
                pages_with_line(&core, continued),
                vec![table_page + 1],
                "{}: 저장 재시작 뒤 조각은 다음 쪽 각주 영역에 한 번",
                case.source
            );
        }
    }
}

#[test]
fn tac_table_cell_footnote_moves_whole_note_to_next_page() {
    check(&WHOLE);
}

#[test]
fn tac_table_cell_footnote_splits_at_stored_reset() {
    check(&SPLIT);
}
