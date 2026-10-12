//! [#7860] 표 셀 안에 둔 미주도 본문을 미주 영역에 그린다.
//!
//! # 무엇이 깨져 있었나
//!
//! 미주 수집은 본문 문단의 컨트롤만 보았고, 미주 참조(`EndnoteRef`)는 본문 문단
//! 주소만 담을 수 있었다. 그래서 표 셀 안 미주는 본문 쪽 번호 표시(`3)`)만 남고
//! 미주 영역의 본문이 통째로 빠졌다(목록이 `1) 2) 4) 5) 6)`).
//!
//! # 기대값의 출처 — 한/글 출력 PDF
//!
//! `pdf/endnote-01-2022.pdf`·`pdf/endnote-01-hwp-2020.pdf` 5쪽이 미주 여섯 개를
//! 번호 순서대로 모두 출력한다. 3번은 문단 0.11 의 1×3 표 셀[2] 안 미주다.
//!
//! # 이 검사가 말하지 않는 것
//!
//! 미주 줄의 절대 위치는 잠그지 않는다(실제 위치는 Visual Sweep 으로 본다).
//! 여기서 잠그는 것은 미주 본문의 존재·중복 없음·번호 순서와 본문 표시 보존이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const DOC: &str = "samples/endnote-01.hwp";

/// 한/글 정본 PDF 의 미주 본문(번호 순서).
const NOTE_BODIES: [&str; 6] = [
    "플라스틱액체라는것은",
    "두번째미주입니다.",
    "세번째미주는표안에넣었습니다.",
    "네번째는다음페이지일반문단에서",
    "전문인력의교육이필수입니다.",
    "지적재산권은설계도가많습니다.",
];

fn collect_runs(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((node.bbox.y, node.bbox.x, run.text.clone()));
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 쪽 순서·세로 순서로 늘어놓은 (쪽, 공백 없는 줄) 목록.
fn document_lines() -> Vec<(u32, String)> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DOC);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{DOC} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 5, "한/글 정본은 5쪽이다");

    let mut lines = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render tree");
        let mut runs = Vec::new();
        collect_runs(&tree.root, &mut runs);
        runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut page_lines: Vec<(f64, String)> = Vec::new();
        for (y, _x, text) in runs {
            let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
            match page_lines.last_mut() {
                Some((prev_y, buf)) if (*prev_y - y).abs() < 0.5 => buf.push_str(&text),
                _ => page_lines.push((y, text)),
            }
        }
        lines.extend(page_lines.into_iter().map(|(_, text)| (page, text)));
    }
    lines
}

/// 미주 여섯 개의 본문이 한 번씩, 번호 순서대로 미주 영역에 나온다.
#[test]
fn every_endnote_body_is_drawn_once_in_number_order() {
    let lines = document_lines();
    let mut positions = Vec::new();
    for (index, body) in NOTE_BODIES.iter().enumerate() {
        let hits: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, (_, line))| line.contains(body))
            .map(|(at, _)| at)
            .collect();
        assert_eq!(
            hits.len(),
            1,
            "{}번 미주 본문 `{body}` 이 {}번 나온다(정본 1번). 줄={lines:?}",
            index + 1,
            hits.len()
        );
        let line = &lines[hits[0]].1;
        assert!(
            line.starts_with(&format!("{})", index + 1)),
            "{}번 미주 본문 앞에 번호 `{})` 가 없다: `{line}`",
            index + 1,
            index + 1
        );
        positions.push(hits[0]);
    }
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "미주 본문이 번호 순서가 아니다: {positions:?}"
    );
}

/// 본문 쪽 번호 표시는 미주 여섯 개가 각자 한 번씩, 번호 순서로 남는다.
/// 셀 안 3번 표시는 셀 제목과 같은 쪽(2쪽)에 있다.
#[test]
fn body_markers_stay_once_each_with_cell_marker_on_heading_page() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DOC);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{DOC} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    fn collect_markers(node: &RenderNode, page: u32, out: &mut Vec<(u32, u16, String)>) {
        if let RenderNodeType::FootnoteMarker(marker) = &node.node_type {
            out.push((page, marker.number, marker.text.clone()));
        }
        for child in &node.children {
            collect_markers(child, page, out);
        }
    }
    let mut markers = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render tree");
        collect_markers(&tree.root, page, &mut markers);
    }
    let numbers: Vec<u16> = markers.iter().map(|(_, number, _)| *number).collect();
    assert_eq!(numbers, vec![1, 2, 3, 4, 5, 6], "본문 표시: {markers:?}");

    let heading_page = document_lines()
        .into_iter()
        .find(|(_, line)| line.contains("기술발전에따른예상되는사회적변화"))
        .map(|(page, _)| page)
        .expect("셀 제목 줄");
    assert_eq!(
        markers[2].0, heading_page,
        "3번 표시가 셀 제목 쪽에 있어야 한다: {markers:?}"
    );
}
