//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8547의 정식 회귀.
//! 주석 갱신: 2026-11-05T06:15:20+09:00; 직전 파일 변경: 2026-11-05T04:07:14+09:00 (`d5f1feb9132a0a3cc57378eb9924da53ea6c5a50`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-05 KST, 제품 코드 `c183a1ac61c2aa4188989fd93032cae68d3538c1`.
//! 같은 원본·기존 Print의 새 Native/fresh WASM 제품에서 전쪽 측정·3·4쪽 직접 판독 완료.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8532/gangseo_exempt_certificates.hwp`; 독립 Print `pdf/issue8532_gangseo_exempt_certificates-2020.pdf`.
//! 전체 6쪽, 전쪽 최저 Native 99.85% / fresh WASM 99.87%.
//! 주장 3쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 주장 4쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8547_review.md`.
//! 최신 증거: `mydocs/pr/assets/planet6897_20261103/pr_8547/closing-frame-c183a1ac61/`.
//! 새 검사는 화면 절대 px를 고정하지 않고 원본 HU 상자 종료와 실제 셀 종료의 관계를 검사한다.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 쪽 끝에 걸리고, 셀 저장 사다리가 그 자리에서 쪽을 되감으면(다음 줄 vpos 0) 그 되감김이
//! 한/글의 쪽 경계다. 앞 조각이 한 줄 높이여도 고아 방지 최소 높이로 묶음 전체를 다음 쪽으로
//! 넘기지 않는다.
//!
//! - 입력: `samples/issue8532/gangseo_exempt_certificates.hwp` — 공개 자치법규 별표
//!   「[별표 10] 전직시험이 면제되는 자격증 구분표(부산광역시 강서구의회 지방공무원 인사 규칙)」
//!   원본 그대로(72×5 RowBreak 표).
//! - 독립 기준: 같은 HWP 의 한/글 2022 Print `pdf/issue8532_gangseo_exempt_certificates-2020.pdf`
//!   — 6쪽. 아래 표의 낱말별 쪽 등장 횟수는 그 PDF 의 쪽별 텍스트에서 센 값이다.
//! - 수정 전 rhwp 는 3쪽 끝 「위생」 묶음을 4쪽으로 통째 넘겨 4~6쪽 행이 한 칸씩 밀렸다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn collect(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.extend(run.text.chars().filter(|c| !c.is_whitespace()));
    }
    for child in &node.children {
        collect(child, text);
    }
}

fn page_text(core: &DocumentCore, page: u32) -> String {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let mut text = String::new();
    collect(&tree.root, &mut text);
    text
}

/// (낱말, 한/글 Print 1~6쪽 등장 횟수) — 문서 안에서 한 묶음에만 나오는 낱말.
const HANCOM_PAGE_COUNTS: &[(&str, [usize; 6])] = &[
    ("가축인공수정사", [0, 0, 1, 0, 0, 0]),
    ("항공공장정비사", [0, 0, 0, 2, 0, 0]),
    ("항공기관사", [0, 0, 0, 2, 0, 0]),
    ("조리기능장", [0, 0, 0, 0, 1, 0]),
    ("조리기능사", [0, 0, 0, 0, 1, 0]),
    ("제1종운전면허(대형)", [0, 0, 0, 0, 0, 1]),
    ("제2종운전면허(보통)", [0, 0, 0, 0, 0, 1]),
    ("기계정비,조경", [0, 0, 0, 0, 0, 1]),
];

#[test]
fn rowspan_block_splits_at_stored_cell_restart_even_for_one_line() {
    let core = core("samples/issue8532/gangseo_exempt_certificates.hwp");
    assert_eq!(core.page_count(), 6, "한/글 Print 쪽수");
    let pages: Vec<String> = (0..6).map(|page| page_text(&core, page)).collect();
    for (word, expected) in HANCOM_PAGE_COUNTS {
        let actual: Vec<usize> = pages
            .iter()
            .map(|text| text.matches(word).count())
            .collect();
        assert_eq!(
            actual.as_slice(),
            expected.as_slice(),
            "「{word}」 쪽별 등장 횟수(소속·누락·중복)는 한/글과 같다"
        );
    }
}

fn collect_nodes<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        collect_nodes(child, nodes);
    }
}

#[test]
fn stored_restart_block_owns_source_host_frame_bottom() {
    use rhwp::model::control::Control;

    let core = core("samples/issue8532/gangseo_exempt_certificates.hwp");
    let (section, paragraph, control, host, source) = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(si, section)| {
            section
                .paragraphs
                .iter()
                .enumerate()
                .flat_map(move |(pi, host)| {
                    host.controls
                        .iter()
                        .enumerate()
                        .filter_map(move |(ci, control)| {
                            let Control::Table(table) = control else {
                                return None;
                            };
                            (table.row_count == 72 && table.col_count == 5)
                                .then_some((si, pi, ci, host, table))
                        })
                })
        })
        .next()
        .expect("독립 Print의 전직시험 자격증 표");
    let boundary_row = source
        .cells
        .iter()
        .find(|cell| {
            cell.paragraphs.iter().any(|p| p.text.trim() == "위생사")
                && cell.paragraphs.iter().any(|p| p.text.trim() == "영양사")
        })
        .expect("위생사·영양사가 저장 쪽을 나눠 갖는 원본 칸")
        .row;
    let [host_line] = host.line_segs.as_slice() else {
        panic!("원본 표 호스트는 저장 줄 하나")
    };
    // HWP5 원본 호스트와 개체가 선언한 상자의 종료다. 엔진의 fragment 높이,
    // 줄 측정 helper 또는 보정한 화면 좌표를 기대값으로 재인용하지 않는다.
    let source_bottom_hu = f64::from(host_line.vertical_pos)
        + f64::from(source.common.vertical_offset as i32)
        + f64::from(source.outer_margin_top)
        + f64::from(source.common.height);
    let mut continuation_rows = Vec::new();
    for page in [2, 3] {
        let tree = core.build_page_render_tree(page).expect("독립 Print 3·4쪽");
        let mut nodes = Vec::new();
        collect_nodes(&tree.root, &mut nodes);
        let body = nodes
            .iter()
            .copied()
            .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
            .expect("본문 프레임");
        let table = nodes.iter().copied().find(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
                if t.section_index == Some(section) && t.para_index == Some(paragraph) && t.control_index == Some(control))
        }).expect("원본 소유자가 같은 표 조각");
        let mut cells = Vec::new();
        collect_nodes(table, &mut cells);
        let row_cells: Vec<_> = cells.iter().copied().filter(|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == boundary_row)).collect();
        assert_eq!(
            row_cells.len(),
            usize::from(source.col_count),
            "경계 행의 칸 누락·중복 없음"
        );
        for cell in &row_cells {
            let RenderNodeType::TableCell(owner) = &cell.node_type else {
                unreachable!()
            };
            if page == 2 {
                let actual_bottom_hu =
                    (cell.bbox.y + cell.bbox.height - body.bbox.y) * 7200.0 / 96.0;
                assert!((actual_bottom_hu - source_bottom_hu).abs() <= 2.0,
                    "경계 칸 col={}의 실제 종료 {actual_bottom_hu}HU가 원본 프레임 {source_bottom_hu}HU를 보존해야 함", owner.col);
            }
            if owner.col >= 3 {
                let mut text = String::new();
                collect(cell, &mut text);
                let expected = if page == 2 { "위생사" } else { "영양사" };
                assert_eq!(
                    text, expected,
                    "같은 원본 칸의 저장 줄은 앞·뒤 쪽에 한 번씩 표시"
                );
            }
        }
        continuation_rows.push(
            row_cells
                .iter()
                .map(|cell| cell.bbox.y)
                .fold(f64::INFINITY, f64::min)
                - body.bbox.y,
        );
    }
    assert!(
        continuation_rows[1] < continuation_rows[0],
        "다음 조각은 다음 본문 위쪽에서 이어받음"
    );
}
