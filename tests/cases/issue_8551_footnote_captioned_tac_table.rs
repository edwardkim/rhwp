//! 관계형 — [#8551] 각주 안의 캡션 달린 글자처럼 취급 표의 소속·순서·포함 계약.
//! 주석 갱신: 2026-11-05 (Asia/Seoul).
//!
//! 원본: `samples/issue8551/footnote_captioned_tac_table.hwp` — rhwp-issue-docs #838 원문
//! 1–3쪽 추출본을 한/글 2020으로 다시 저장한 문서에서 본문 문단 0.4와 그 각주 3만 남기고
//! (`rhwp edit delete-paragraph`·`delete-footnote`) 한/글 2020으로 다시 저장한 합성 입력이다.
//! 각주는 글 문단 + 「캡션 2문단 달린 9행 표 + `주 : …` 글」 문단 + 빈 문단이다.
//! 독립 기준: `pdf/issue8551/footnote_captioned_tac_table-2020.pdf`
//! (Hwp 2020 Print, `pdf_print_method` 0, 1쪽). 한컴은 각주 영역에 캡션과 표를 그린다.
//!
//! 측정일 2026-11-05, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`,
//! 지표 `silhouette_raw_match_percent`, 전체 1쪽:
//! - 수정 전(devel `e96b500e3e`) Native 58.82% — 표·캡션 자리만 비어 있다.
//! - 수정 후 Native 99.89%, fresh WASM 99.98%. 두 경로 전쪽 최저값도 같다.
//! - 증거: `mydocs/pr/assets/issue_8551/`.
//!
//! 이 검사는 절대 px 좌표를 고정하지 않고 각주 영역 안의 소속, 앞 각주 줄 → 캡션 → 표 →
//! `주 :` 줄 순서, 영역 포함, 본문과 겹치지 않음을 검사한다. 수정 전에는 각주 영역에
//! 표 노드가 없어 실패한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8551/footnote_captioned_tac_table.hwp";

fn find_footnote_area(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::FootnoteArea) {
        return Some(node);
    }
    node.children.iter().find_map(find_footnote_area)
}

fn tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::Table(_)) {
        out.push(node);
    }
    for child in &node.children {
        tables(child, out);
    }
}

/// 비어 있지 않은 TextRun의 `(문자열, bbox)`를 출력 순서대로 모은다.
fn runs(node: &RenderNode, out: &mut Vec<(String, BoundingBox)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty() {
            out.push((run.text.clone(), node.bbox));
        }
    }
    for child in &node.children {
        runs(child, out);
    }
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn joined(items: &[(String, BoundingBox)]) -> String {
    compact(
        &items
            .iter()
            .map(|(text, _)| text.as_str())
            .collect::<String>(),
    )
}

/// `needle`을 포함하는 첫 줄(같은 y의 run 묶음)의 위·아래 끝.
fn line_span(items: &[(String, BoundingBox)], needle: &str) -> (f64, f64) {
    let mut rows: Vec<(f64, Vec<&(String, BoundingBox)>)> = Vec::new();
    for item in items {
        match rows.iter_mut().find(|(y, _)| (*y - item.1.y).abs() < 1.0) {
            Some((_, row)) => row.push(item),
            None => rows.push((item.1.y, vec![item])),
        }
    }
    rows.iter()
        .find(|(_, row)| {
            compact(
                &row.iter()
                    .map(|(text, _)| text.as_str())
                    .collect::<String>(),
            )
            .contains(needle)
        })
        .map(|(_, row)| {
            let top = row.iter().map(|(_, b)| b.y).fold(f64::MAX, f64::min);
            let bottom = row
                .iter()
                .map(|(_, b)| b.y + b.height)
                .fold(f64::MIN, f64::max);
            (top, bottom)
        })
        .unwrap_or_else(|| panic!("각주 영역에 `{needle}` 줄"))
}

#[test]
fn issue_8551_footnote_draws_captioned_tac_table_in_owned_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("원본 읽기 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("합성 원본 로드");
    assert_eq!(core.page_count(), 1, "독립 Print PDF 1쪽");
    let page = core.build_page_render_tree(0).expect("1쪽 배치");
    let area = find_footnote_area(&page.root).expect("1쪽 각주 영역");

    // 표는 각주 영역 소속이며 본문에는 없다.
    let mut note_tables = Vec::new();
    tables(area, &mut note_tables);
    assert_eq!(
        note_tables.len(),
        1,
        "각주 3의 캡션 표가 각주 영역에 그려짐"
    );
    let mut all_tables = Vec::new();
    tables(&page.root, &mut all_tables);
    assert_eq!(
        all_tables.len(),
        1,
        "같은 표가 본문 등 다른 곳에 중복되지 않음"
    );
    let table = note_tables[0];
    let RenderNodeType::Table(table_node) = &table.node_type else {
        unreachable!()
    };
    assert_eq!(
        (table_node.row_count, table_node.col_count),
        (9, 6),
        "원본 9행 6열"
    );

    let mut note_runs = Vec::new();
    runs(area, &mut note_runs);
    let note_text = joined(&note_runs);
    for needle in [
        "외교통상부",
        "<표->서비스분야교역규모",
        "(단위:십억US$)",
        "스페인",
        "4,220.0",
        "주:상업서비스",
    ] {
        assert!(
            note_text.contains(needle),
            "각주 영역에 `{needle}` 보존: {note_text}"
        );
    }
    // 각주 내용 순서(앞 글 문단 → 캡션 → 표 → 같은 문단 둘째 줄의 `주 :` 글)는
    // 세로 위치로 검사한다. 렌더 트리의 노드 발행 순서는 내용 순서가 아니다.
    let (_, first_bottom) = line_span(&note_runs, "외교통상부");
    let (caption_top, caption_bottom) = line_span(&note_runs, "<표->서비스분야교역규모");
    let (note_top, _) = line_span(&note_runs, "주:상업서비스");
    let table_top = table.bbox.y;
    let table_bottom = table.bbox.y + table.bbox.height;
    assert!(
        first_bottom <= caption_top,
        "앞 각주 줄 {first_bottom} ≤ 캡션 {caption_top}"
    );
    assert!(
        caption_bottom <= table_top + 0.5,
        "캡션 {caption_bottom} ≤ 표 본체 {table_top}"
    );
    assert!(
        table_bottom <= note_top,
        "표 {table_bottom} ≤ `주 :` 줄 {note_top}"
    );

    // 표 셀 글자는 표 상자 안에 있다.
    let mut cell_runs = Vec::new();
    runs(table, &mut cell_runs);
    assert!(
        joined(&cell_runs).contains("스페인"),
        "셀 글자가 표 노드 소속"
    );
    for (text, b) in &cell_runs {
        assert!(
            b.y >= table_top - 0.5 && b.y + b.height <= table_bottom + 0.5,
            "셀 글자 `{text}` 가 표 안"
        );
    }

    // 표는 각주 영역 안에 있고 본문 글과 겹치지 않는다.
    let a = area.bbox;
    assert!(
        table.bbox.x >= a.x - 0.5
            && table.bbox.x + table.bbox.width <= a.x + a.width + 0.5
            && table_bottom <= a.y + a.height + 0.5,
        "표 {:?} 가 각주 영역 {:?} 안",
        table.bbox,
        a
    );
    let mut page_runs = Vec::new();
    runs(&page.root, &mut page_runs);
    let body_bottom = page_runs
        .iter()
        .filter(|(_, b)| b.y + b.height <= a.y + 0.5)
        .map(|(_, b)| b.y + b.height)
        .fold(f64::MIN, f64::max);
    assert!(
        body_bottom < table_top,
        "본문 끝 {body_bottom} < 각주 표 {table_top}"
    );
}
