//! [Issue #5792] 쪽을 넘어 이어지는 자리차지 표 조각이 3행만 남고, 그 자리에 뒤따르는
//! 문단이 같은 y 에 겹쳐 그려지던 결함의 회귀 가드.
//!
//! 근인: 쪽 하단에서 잘린 overlay 표의 잔여 행(#4568 이어 그리기)이 새 쪽 흐름에
//! 아무 자리도 예약하지 않는다. 그래서 (a) 다음 쪽 본문 문단이 잔여 행과 같은 y 에서
//! 시작해 글자가 겹치고, (b) 그 본문이 배치한 표가 잔여 행의 페인트 상한
//! (`existing_table_top`)을 깎아 남은 행이 통째로 사라진다.
//!
//! 계약(동봉 재현 문서 — 질병관리청 고시 [별표 3] 동물이용 취급시설 기준):
//! 1. pi=9 의 42행 표는 조각을 모두 합쳐 42행 전부가 그려진다(행 소실 0).
//! 2. 그 표 조각과 본문 문단 줄(표 밖 TextLine)이 세로로 겹치지 않는다.
//!
//! 수정 전 실측: 3쪽 조각이 rows 22..25(55.1px)만 남고 rows 25..42 소실,
//! `Ⅱ. 곤충이용`(y=75.6)·`1. 설치기준`(y=103.3)이 조각(75.6..130.7)과 완전 겹침.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue5792/2700727_animal_facility_standards.hwpx";
/// 쪽을 넘어 이어지는 42행 overlay 표의 host 문단.
const SPLIT_TABLE_PARA: usize = 9;

/// (para_index, y0, y1)
type Band = (usize, f64, f64);

fn collect(
    node: &RenderNode,
    lines: &mut Vec<Band>,
    tables: &mut Vec<(Band, BTreeSet<usize>)>,
    in_cell: bool,
) {
    let cell_like = matches!(
        node.node_type,
        RenderNodeType::TableCell(_) | RenderNodeType::Header | RenderNodeType::Footer
    ) || in_cell;
    match &node.node_type {
        RenderNodeType::TextLine(tl) if !cell_like => {
            if let Some(pi) = tl.para_index {
                let b = &node.bbox;
                if b.height > 0.0 && b.height < 150.0 && has_visible_text(node) {
                    lines.push((pi, b.y, b.y + b.height));
                }
            }
        }
        RenderNodeType::Table(tn) if !cell_like => {
            if let Some(pi) = tn.para_index {
                let b = &node.bbox;
                let mut rows = BTreeSet::new();
                collect_rows(node, &mut rows);
                tables.push(((pi, b.y, b.y + b.height), rows));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, lines, tables, cell_like);
    }
}

fn collect_rows(node: &RenderNode, rows: &mut BTreeSet<usize>) {
    if let RenderNodeType::TableCell(tc) = &node.node_type {
        rows.insert(tc.row as usize);
        return;
    }
    for child in &node.children {
        collect_rows(child, rows);
    }
}

fn has_visible_text(node: &RenderNode) -> bool {
    if let RenderNodeType::TextRun(tr) = &node.node_type {
        if tr.text.chars().any(|c| !c.is_whitespace()) {
            return true;
        }
    }
    node.children.iter().any(has_visible_text)
}

#[test]
fn issue_5792_split_overlay_table_keeps_every_row_and_clears_following_text() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let mut painted_rows: BTreeSet<usize> = BTreeSet::new();
    let mut fragment_count = 0usize;
    for page in 0..doc.page_count() {
        let tree = doc
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render tree p{page}: {e}"));
        let mut lines = Vec::new();
        let mut tables = Vec::new();
        collect(&tree.root, &mut lines, &mut tables, false);

        for ((pi, ty0, ty1), rows) in &tables {
            if *pi != SPLIT_TABLE_PARA {
                continue;
            }
            fragment_count += 1;
            painted_rows.extend(rows.iter().copied());
            // 조각과 본문 줄이 세로로 겹치면 글자 뭉침이다(수정 전 3쪽 실측).
            for (lpi, ly0, ly1) in &lines {
                if lpi == pi {
                    continue;
                }
                let dy = ty1.min(*ly1) - ty0.max(*ly0);
                let line_h = ly1 - ly0;
                assert!(
                    !(line_h > 0.0 && dy >= line_h * 0.9),
                    "p{} 에서 pi{} 표 조각({:.1}..{:.1})이 pi{} 본문 줄({:.1}..{:.1})과 겹침 (#5792)",
                    page + 1,
                    pi,
                    ty0,
                    ty1,
                    lpi,
                    ly0,
                    ly1
                );
            }
        }
    }

    assert!(
        fragment_count >= 2,
        "pi{SPLIT_TABLE_PARA} 표가 쪽을 넘어 분할되지 않았다 — 샘플/계약 확인 필요"
    );
    let missing: Vec<usize> = (0..42).filter(|r| !painted_rows.contains(r)).collect();
    assert!(
        missing.is_empty(),
        "pi{SPLIT_TABLE_PARA} 42행 중 {}행이 어느 쪽에도 그려지지 않았다: {:?} (#5792)",
        missing.len(),
        missing
    );
}
