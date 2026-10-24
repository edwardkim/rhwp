//! [#7732] 여러 쪽에 걸친 쪽 나눔 표의 칸 각주 — 행을 줄이지 않는다.
//!
//! 한/글 2020·2024 Print(합성 `samples/issue1937/gen_cell_notes_multipage.py`, 각주 앵커가 첫 쪽
//! 행에 있는 2열 표)는 칸 각주 때문에 행을 줄이지 않는다. 각주는 조각 뒤 남는 공간이나 표가
//! 끝나는 쪽에 놓이고, 첫 쪽은 행으로 본문 바닥까지 찬다. 병합 칸 유무와 무관하다.
//!
//! ```text
//!   입력                          정본 쪽별 행            각주
//!   cellnotes_span_N2             R00–R44 · R45–R69       1)–3) 2쪽
//!   r110_cellnotes_span_N2        R00–R44 · R45–R90 · …   1)–4) 3쪽
//!   cellnotes_plain_N2_L7         행마다 7줄, 쪽 끝 행을 줄 단위로 자른다
//! ```
//!
//! 쪽 끝에서 자른 행의 마지막 줄은 뒤 줄간격 없이 계상한다. 칸 첫 문단에 각주가 있어도 같다.
//!
//! 실물: `samples/issue1937_rowbreak_footnote_overpagination.hwp` 35쪽 122행 표(pi=306)는 각주
//! 18)–40) 앵커가 35–41쪽 행에 있지만 정본은 35쪽을 9번 행까지 채우고 각주를 42·43쪽에 둔다.
//! 수정 전에는 첫 조각이 표 각주 전체를 예약해 35쪽이 행 0–7 에서 끝났다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse")
}

/// 쪽마다 표 조각의 (쪽, 시작 행, 끝 행).
fn table_rows(core: &DocumentCore, para: u64) -> Vec<(usize, u64, u64)> {
    let pages = core.dump_page_items_json(None);
    let mut out = Vec::new();
    for (page, value) in pages.as_array().unwrap().iter().enumerate() {
        for column in value["columns"].as_array().unwrap() {
            for item in column["items"].as_array().unwrap() {
                if item["paraIndex"].as_u64() != Some(para) {
                    continue;
                }
                if let (Some(start), Some(end)) =
                    (item["startRow"].as_u64(), item["endRow"].as_u64())
                {
                    out.push((page, start, end));
                }
            }
        }
    }
    out
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

fn footnote_text(node: &RenderNode) -> String {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        return text(node);
    }
    node.children.iter().map(footnote_text).collect()
}

/// 각 쪽 각주 영역의 각주 개수(번호 `N)` 뒤에 본문이 시작하는 자리).
fn notes_per_page(core: &DocumentCore) -> Vec<usize> {
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("page");
            footnote_text(&tree.root).matches(") 각주 본문").count()
        })
        .collect()
}

#[test]
fn split_table_cell_notes_do_not_shorten_first_page_rows() {
    for (sample, rows, notes) in [
        (
            "samples/issue1937/cellnotes_span_N2.hwpx",
            vec![(0, 0, 45), (1, 45, 70)],
            vec![0, 3],
        ),
        (
            "samples/issue1937/r110_cellnotes_span_N2.hwpx",
            vec![(0, 0, 45), (1, 45, 91), (2, 91, 110)],
            vec![0, 0, 4],
        ),
    ] {
        let core = core(sample);
        assert_eq!(table_rows(&core, 1), rows, "{sample}: 쪽별 행");
        assert_eq!(
            notes_per_page(&core),
            notes,
            "{sample}: 각주는 표가 끝나는 쪽"
        );
    }
}

/// 쪽 본문(각주 영역 밖)의 글자.
fn body_text(node: &RenderNode) -> String {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        return String::new();
    }
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&body_text(child));
    }
    out
}

/// 본문에 보이는 칸 줄 표지 — `행 Rnn 내용` 은 1번 줄, `Rnn 줄 k` 는 k번 줄.
fn body_line_marks(node: &RenderNode) -> Vec<(u32, u32)> {
    let chars: Vec<char> = body_text(node)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let digits = |at: usize, n: usize| -> Option<u32> {
        let s: String = chars.get(at..at + n)?.iter().collect();
        s.chars()
            .all(|c| c.is_ascii_digit())
            .then(|| s.parse().ok())?
    };
    let mut out = Vec::new();
    for at in 0..chars.len() {
        if chars[at] != 'R' {
            continue;
        }
        let Some(row) = digits(at + 1, 2) else {
            continue;
        };
        if chars.get(at + 3) == Some(&'줄') {
            if let Some(line) = digits(at + 4, 1) {
                out.push((row, line));
            }
        } else if chars.get(at + 3..at + 5) == Some(&['내', '용'][..]) {
            out.push((row, 1));
        }
    }
    out
}

#[test]
fn split_row_with_cell_note_keeps_line_that_fits_without_trailing_spacing() {
    // 한/글 2020 Print `pdf/issue1937/cellnotes_plain_N2_L7-2020.pdf` 쪽별 첫·끝 칸 줄
    // (행, 줄). R01·R12·R25 첫 줄에 각주가 있다. 2쪽은 R12 를 다섯째 줄까지 받는다 —
    // 마지막 줄의 뒤 줄간격을 빼면 남은 공간에 들어간다(각주 없는 행과 같은 규칙).
    let expected = [
        ((0, 1), (6, 2)),
        ((6, 3), (12, 5)),
        ((12, 6), (19, 1)),
        ((19, 2), (25, 4)),
        ((25, 5), (31, 7)),
        ((32, 1), (38, 3)),
        ((38, 4), (44, 6)),
        ((44, 7), (51, 2)),
        ((51, 3), (57, 5)),
        ((57, 6), (64, 1)),
        ((64, 2), (69, 7)),
    ];
    let core = core("samples/issue1937/cellnotes_plain_N2_L7.hwpx");
    let pages: Vec<_> = (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("page");
            let marks = body_line_marks(&tree.root);
            marks.first().copied().zip(marks.last().copied())
        })
        .collect();
    let expected: Vec<_> = expected.into_iter().map(Some).collect();
    assert_eq!(pages, expected, "쪽별 첫·끝 칸 줄 (행, 줄) — 쪽수 포함");
}

#[test]
fn issue1937_page35_fills_rows_without_reserving_cell_notes() {
    let core = core("samples/issue1937_rowbreak_footnote_overpagination.hwp");
    let rows = table_rows(&core, 306);
    assert!(rows.len() >= 2, "표 pi=306 조각 — {rows:?}");
    assert_eq!(
        (rows[0].0, rows[0].1, rows[0].2),
        (34, 0, 11),
        "35쪽은 행 0–10(행 10 은 쪽 끝에서 잘림) — {rows:?}"
    );
    assert_eq!(rows[1].1, 10, "36쪽은 행 10 을 이어받는다 — {rows:?}");
    let notes = notes_per_page(&core);
    let note_page = |page: usize| {
        let tree = core.build_page_render_tree(page as u32).expect("page");
        footnote_text(&tree.root)
    };
    for page in 34..=40 {
        assert!(
            !note_page(page).contains("관광개발진흥기금"),
            "{}쪽에 각주 18) 이 오지 않는다",
            page + 1
        );
    }
    assert!(
        note_page(41).contains("관광개발진흥기금"),
        "각주 18) 은 표가 끝나는 42쪽 — {notes:?}"
    );
}
