//! Issue #1749 v2: 누적좌표 문서라도 다음 문단이 명시적 쪽나누기면 saved bounds 를 신뢰한다.
//!
//! 회귀 사례 (`samples/task1749/saved_bounds_cumulative_page_break.hwpx`):
//! - 2쪽 말미 pi=26 은 누적높이 검사 탈락(919.2+36.3 > 930.5px)이지만 저장 bounds
//!   (vpos 137484 − 2쪽 기준 69310 → bottom ≈ 930.3px ≤ avail)로 2쪽 배치가 정답.
//! - 이 문서는 누적좌표(쪽 경계에서도 vpos 리셋 없음)인데 다음 문단 pi=27 이 명시적
//!   [쪽나누기](column_type=Page)라 "vpos 리셋" 검사로는 페이지-마지막 증거를 못 본다.
//!   #1749 1차 게이트가 이 증거를 누락해 pi=26 이 3쪽 단독 문단으로 밀렸다(5쪽→6쪽).
//! - 저장 lineseg 근거: pi=25(vpos=134764)와 pi=26(vpos=137484)은 한 줄(2720HU) 간격
//!   연속 배치 = 한글은 pi=26 을 2쪽 마지막 줄로 인코딩.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;

const HWPX_SAMPLE: &str = "samples/task1749/saved_bounds_cumulative_page_break.hwpx";
const HWP_SAMPLE: &str = "samples/task1749/saved_bounds_cumulative_page_break.hwp";

fn load_sample(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    load_sample(HWPX_SAMPLE)
}

#[test]
fn issue_1749_v2_pi26_stays_on_page_2() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        5,
        "쪽나누기 직전 단일 줄 문단 pi=26 이 밀리면 5쪽 문서가 6쪽이 된다"
    );

    let page2 = doc.dump_page_items(Some(1));
    assert!(
        page2.contains("pi=26"),
        "pi=26 은 2쪽 마지막 문단이어야 한다 (저장 lineseg: pi=25 와 한 줄 간격 연속)\n--- page 2 ---\n{}",
        page2
    );
}

#[test]
fn issue_1811_hwpx_pi52_rowbreak_cut_matches_hwp_reference() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        5,
        "p5 tail drift 보정 후에도 전체 5쪽이어야 한다"
    );

    let hwp_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(HWP_SAMPLE);
    let hwp_doc =
        rhwp::document_core::DocumentCore::from_bytes(&fs::read(hwp_path).expect("HWP 원문"))
            .expect("HWP 저장 문서");
    assert_eq!(
        hwp_doc.page_count(),
        5,
        "HWP 저장 LINE_SEG 경로는 5쪽을 유지해야 한다"
    );
    // 숫자 컷 대신 정상 PDF가 소유하는 글줄·문단과 원본 물리 프레임을 검사한다.
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn find(node: &RenderNode, predicate: &impl Fn(&RenderNode) -> bool) -> Option<RenderNode> {
        if predicate(node) {
            return Some(node.clone());
        }
        node.children
            .iter()
            .find_map(|child| find(child, predicate))
    }
    fn text(node: &RenderNode) -> String {
        let own = match &node.node_type {
            RenderNodeType::TextRun(run) => run.text.clone(),
            _ => String::new(),
        };
        node.children.iter().fold(own, |mut out, child| {
            out.push_str(&text(child));
            out
        })
    }
    fn star_lines(node: &RenderNode, out: &mut Vec<usize>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let count = text(node).chars().filter(|ch| *ch == '*').count();
            if count > 0 {
                out.push(count);
            }
            return;
        }
        for child in &node.children {
            star_lines(child, out);
        }
    }
    fn para_lines(node: &RenderNode, para_index: usize, out: &mut Vec<RenderNode>) {
        if matches!(&node.node_type,
            RenderNodeType::TextLine(line) if line.para_index == Some(para_index))
        {
            out.push(node.clone());
        }
        for child in &node.children {
            para_lines(child, para_index, out);
        }
    }
    let hwpx_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(HWPX_SAMPLE);
    let hwpx_doc =
        rhwp::document_core::DocumentCore::from_bytes(&fs::read(hwpx_path).expect("HWPX 원문"))
            .expect("HWPX 저장 문서");
    let hwpx_page4 = hwpx_doc.build_page_render_tree(3).expect("HWPX 4쪽");
    let hwpx_page5 = hwpx_doc.build_page_render_tree(4).expect("HWPX 5쪽");
    let table52 = |node: &RenderNode| {
        matches!(&node.node_type,
        RenderNodeType::Table(table) if table.para_index == Some(52))
    };
    let hwpx_first = find(&hwpx_page4.root, &table52).expect("HWPX 4쪽 사회기여 표");
    let hwpx_last = find(&hwpx_page5.root, &table52).expect("HWPX 5쪽 사회기여 표 이어받기");
    let mut host_lines = Vec::new();
    para_lines(&hwpx_page4.root, 52, &mut host_lines);
    assert_eq!(host_lines.len(), 4, "4쪽 host 본문 네 줄을 보존한다");
    assert!(
        host_lines
            .iter()
            .any(|line| text(line).contains("사회기여")),
        "표 위 host 본문이 원문 내용을 보존한다"
    );
    assert!(
        host_lines
            .iter()
            .all(|line| line.bbox.y + line.bbox.height <= hwpx_first.bbox.y),
        "4쪽 host 본문은 표 첫 조각보다 앞에 있어야 한다"
    );
    let mut hwpx_first_lines = Vec::new();
    let mut hwpx_last_lines = Vec::new();
    star_lines(&hwpx_first, &mut hwpx_first_lines);
    star_lines(&hwpx_last, &mut hwpx_last_lines);
    assert_eq!(hwpx_first_lines, [50, 67, 10], "4쪽 표 글줄의 소유와 순서");
    assert_eq!(
        hwpx_last_lines,
        [67, 22, 67, 16, 67, 4],
        "5쪽 표 이어받기 글줄의 소유와 순서"
    );
    let page4 = hwp_doc.build_page_render_tree(3).expect("HWP 4쪽");
    let page5 = hwp_doc.build_page_render_tree(4).expect("HWP 5쪽");
    let first = find(&page4.root, &table52).expect("4쪽 사회기여 표");
    let last = find(&page5.root, &table52).expect("5쪽 사회기여 표 이어받기");
    let mut first_lines = Vec::new();
    let mut last_lines = Vec::new();
    star_lines(&first, &mut first_lines);
    star_lines(&last, &mut last_lines);
    assert_eq!(
        first_lines,
        [50, 67, 10],
        "정상 PDF4쪽은 첫 두 문단의 세 글줄을 소유한다"
    );
    assert_eq!(
        last_lines,
        [67, 22, 67, 16, 67, 4],
        "정상 PDF5쪽은 다음 세 문단의 여섯 글줄을 소유한다"
    );
    let source = &hwp_doc.document().sections[0];
    let source_table = source.paragraphs[52]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("사회기여 원문 표");
    let whole_hu: i64 = source_table
        .get_raw_row_heights()
        .iter()
        .map(|height| i64::from(*height))
        .sum();
    let tail_hu = whole_hu - i64::from(source_table.common.height);
    assert_eq!(
        tail_hu,
        i64::from(source.paragraphs[53].line_segs[0].vertical_pos),
        "원본 행합에서 첫 프레임을 뺀 종료 공간은 다음 빈 문단 원점을 정확히 닫는다"
    );
    let body = find(&page5.root, &|node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .expect("5쪽 본문");
    let following = find(&page5.root, &|node| {
        matches!(&node.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(53))
    })
    .expect("표 뒤 빈 문단");
    let source_tail = rhwp::renderer::hwpunit_to_px(tail_hu as i32, 96.0);
    assert!(
        (following.bbox.y - body.bbox.y - source_tail).abs() < 1e-6,
        "빈 종료 밴드를 보존한 뒤 원문 문단53이 이어져야 한다: {:?}",
        following.bbox
    );
    assert!(
        first.bbox.height >= rhwp::renderer::hwpunit_to_px(source_table.common.height as i32, 96.0),
        "첫 조각도 원문 개체 프레임의 빈 공간을 소유한다"
    );
    let page5_text = hwp_doc.extract_page_text_native(4).expect("5쪽 내용");
    assert!(
        page5_text.contains("청정아트") && page5_text.contains("네트워킹데이 추진계획"),
        "표 뒤 본문과 마지막 추진계획을 보존한다"
    );

    let section = &doc.document().sections[0];
    let table52 = section.paragraphs[52]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("pi=52 table");
    let cell52 = table52
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 0)
        .expect("pi=52 row 2 merged cell");
    let line_counts52: Vec<usize> = cell52
        .paragraphs
        .iter()
        .map(|para| para.line_segs.len())
        .collect();
    assert_eq!(
        line_counts52,
        vec![1, 2, 2, 2, 2, 1],
        "pi=52 RowBreak 셀은 명시 셀 높이와 저장 anchor lineSeg 기준으로 p1/p4까지 2줄 합성이 필요하다"
    );
}

#[test]
fn issue_1811_hwpx_pi57_tac_rowbreak_cell_uses_saved_height() {
    let doc = load_doc();
    let section = &doc.document().sections[0];
    let table57 = section.paragraphs[57]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("pi=57 table");
    let cell57 = table57
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 0)
        .expect("pi=57 row 2 merged cell");
    let line_counts57: Vec<usize> = cell57
        .paragraphs
        .iter()
        .map(|para| para.line_segs.len())
        .collect();
    assert_eq!(
        line_counts57,
        vec![2, 2],
        "TAC RowBreak 셀은 anchor 문단이 없어도 표/셀 저장 높이를 기준으로 부족한 합성 줄을 보강해야 한다"
    );
}
