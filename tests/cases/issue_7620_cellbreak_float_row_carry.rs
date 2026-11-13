//! 관계형·고정형(전체 쪽수).
//! 주석 갱신: 2026-11-13T11:18:31+09:00.
//! 직전 파일 변경: 2026-11-12T00:19:18+09:00, 0bddada6573f11ed92c3b1943271f14e6b0ce383 (Git committer 시각).
//! 2026-11-13 실측: source e5a6a1ff9f, raw `silhouette_raw_match_percent`, 96dpi,
//! Desktop Hancom 2020 Print·pypdfium2, `scripts/visual_sweep.py --silhouette-only --compat 2022`.
//! e-cellbreak-row-moves-next-page: 전2쪽 Native/fresh WASM 각각 100.00%, 전쪽 최저 각각 100.00%.
//! f-rowbreak-row-splits-at-line: 전2쪽 Native/fresh WASM 각각 100.00%, 전쪽 최저 각각 100.00%.
//! 근거: mydocs/pr/assets/sacru2red-20261113/final/README.md (전쪽 TSV·직접 판독).
//! 보정 전 Native p2는 각각 94.62/93.46%, WASM p2는 94.90/93.79%였고,
//! 실제 빈 줄의 전진이 저장 1600HU보다 600HU 커 아래의 관계 검사가 실패했다.
//!
//! Issue #7620 후속 3 — 비TAC 자리 차지 표의 쪽 경계 나눔 «셀 단위로 나눔»(CellBreak, 1)과
//! «나눔»(RowBreak, 2)에서 쪽 바닥에 걸린 여러 줄 행의 이월.
//!
//! 한/글은 CellBreak 표에서 쪽 바닥에 걸린 행을 자르지 않고 행 전체를 다음 쪽으로 넘기며,
//! 같은 입력의 RowBreak 표는 그 행의 글줄 경계에서 자르고 남은 줄을 다음 쪽이 이어받는다.
//! TAC 표의 같은 계약은 `issue_7288_cell_unit_row_is_atomic`이 다룬다. 이 파일은 #7620이 다루는
//! 문단 기준 자리 차지 표 경로를 같은 대조 방식으로 고정한다.
//!
//! 입력은 `samples/issue7620/generate-cellbreak.mjs`로 만든 생성 대조군이다(한컴 저장본 아님).
//! 두 입력은 쪽 경계 나눔 값만 다르다. 현재 기준은 같은 입력의 Windows Hancom 2020
//! 11.0.0.9136 MCP Print(method 0, one-up), `pdf/issue7620/{e,f}-*-2020.pdf`다.
//! 원 기여의 웹 PDF는 참고 이력이며 Desktop Print 기준으로 사용하지 않는다.
//! 행 이월 회귀에 더해 실제 빈 줄의 후행 간격을 한 번만 소비하는 관계를 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::model::table::TablePageBreak;
use rhwp::wasm_api::HwpDocument;

/// 생성기의 앞 문단 수(= 표 문단 번호), 행 수, 셀마다의 줄 수.
const HOST: usize = 30;
const ROWS: usize = 3;
const LINES: usize = 6;
/// 독립 기준 PDF의 전체 쪽수(두 입력 모두).
const PDF_PAGES: u32 = 2;

fn load(rel: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("{rel} 파싱: {e:?}"))
}

/// 고정 입력의 전제: 비TAC · 자리 차지 · 문단 기준 · 오프셋 0 · 지정한 쪽 경계 나눔.
fn assert_host_table(doc: &HwpDocument, rel: &str, page_break: TablePageBreak) {
    let para = &doc.document().sections[0].paragraphs[HOST];
    let t = match &para.controls[..] {
        [Control::Table(t)] => t,
        other => panic!("{rel}: 전제 — 표 문단에 표 하나만 있어야 한다 ({other:?})"),
    };
    assert!(!t.common.treat_as_char, "{rel}: 전제 — 비TAC");
    assert_eq!(
        t.common.text_wrap,
        TextWrap::TopAndBottom,
        "{rel}: 전제 — 자리 차지"
    );
    assert_eq!(
        t.common.vert_rel_to,
        VertRelTo::Para,
        "{rel}: 전제 — 문단 기준"
    );
    assert_eq!(t.common.vertical_offset, 0, "{rel}: 전제 — 세로 오프셋 0");
    assert_eq!(t.page_break, page_break, "{rel}: 전제 — 쪽 경계 나눔");
    assert_eq!(usize::from(t.row_count), ROWS, "{rel}: 전제 — 행 수");
}

/// 한 쪽에서 본 표 조각과 셀 글줄·본문 글줄.
#[derive(Default)]
struct PageView {
    /// 표 문단(`HOST`)의 표 상자 (윗변, 아랫변).
    tables: Vec<(f64, f64)>,
    /// 표 안 글줄의 라벨(예: `가2-1`) → 윗변.
    cell_lines: Vec<(String, f64)>,
    /// 본문 글줄의 (문단 번호, 윗변). 셀 안 줄은 빼고 본다.
    lines: Vec<(u64, f64)>,
}

fn page_view(doc: &HwpDocument, page: u32) -> PageView {
    fn top_bottom(node: &serde_json::Value) -> (f64, f64) {
        let b = &node["bbox"];
        let y = b["y"].as_f64().unwrap_or(f64::NAN);
        (y, y + b["h"].as_f64().unwrap_or(f64::NAN))
    }
    fn runs(node: &serde_json::Value, out: &mut String) {
        if node["type"].as_str() == Some("TextRun") {
            out.push_str(node["text"].as_str().unwrap_or(""));
        }
        for child in node["children"].as_array().into_iter().flatten() {
            runs(child, out);
        }
    }
    fn walk(node: &serde_json::Value, in_table: bool, view: &mut PageView) {
        let mut in_table = in_table;
        match node["type"].as_str() {
            Some("Table") if !in_table && node["pi"].as_u64() == Some(HOST as u64) => {
                view.tables.push(top_bottom(node));
                in_table = true;
            }
            Some("TextLine") => {
                if in_table {
                    let mut text = String::new();
                    runs(node, &mut text);
                    let label: String = text.chars().filter(|c| !c.is_whitespace()).collect();
                    if !label.is_empty() {
                        view.cell_lines.push((label, top_bottom(node).0));
                    }
                } else if let Some(pi) = node["pi"].as_u64() {
                    view.lines.push((pi, top_bottom(node).0));
                }
                return;
            }
            _ => {}
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, in_table, view);
        }
    }
    let json = doc.get_page_render_tree(page).expect("render tree");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("render tree JSON");
    let mut view = PageView::default();
    walk(&tree, false, &mut view);
    view
}

/// 전 쪽 보기와, 셀 글줄 라벨마다 놓인 쪽 번호 목록(0부터).
fn layout(doc: &HwpDocument, rel: &str) -> (Vec<PageView>, BTreeMap<String, Vec<usize>>) {
    assert_eq!(
        doc.page_count(),
        PDF_PAGES,
        "{rel}: 전체 쪽수가 기준 PDF({PDF_PAGES}쪽)와 같아야 한다"
    );
    let pages: Vec<PageView> = (0..doc.page_count()).map(|p| page_view(doc, p)).collect();
    let mut where_: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (p, v) in pages.iter().enumerate() {
        for (label, _) in &v.cell_lines {
            where_.entry(label.clone()).or_default().push(p);
        }
    }
    // 셀 글줄 전수: 행 r(1부터)·열(가/나)·줄 l(1부터)마다 정확히 한 번.
    for r in 1..=ROWS {
        for col in ["가", "나"] {
            for l in 1..=LINES {
                let label = format!("{col}{r}-{l}");
                let at = where_.get(&label).cloned().unwrap_or_default();
                assert_eq!(
                    at.len(),
                    1,
                    "{rel}: 셀 글줄 {label}은 정확히 한 번 나와야 한다 ({at:?})"
                );
            }
        }
    }
    assert_eq!(
        where_.len(),
        ROWS * 2 * LINES,
        "{rel}: 예상 밖의 셀 글줄이 있다 ({:?})",
        where_.keys().collect::<Vec<_>>()
    );
    // 쪽마다 셀 글줄은 위에서 아래로 행·줄 순서를 지킨다(같은 열 안에서).
    for (p, v) in pages.iter().enumerate() {
        for col in ["가", "나"] {
            let tops: Vec<(&str, f64)> = v
                .cell_lines
                .iter()
                .filter(|(l, _)| l.starts_with(col))
                .map(|(l, y)| (l.as_str(), *y))
                .collect();
            assert!(
                tops.windows(2).all(|w| w[0].1 < w[1].1),
                "{rel}: {}쪽 {col} 열 글줄이 위에서 아래 순서가 아니다 ({tops:?})",
                p + 1
            );
        }
    }
    (pages, where_)
}

fn page_of(where_: &BTreeMap<String, Vec<usize>>, label: &str) -> usize {
    where_[label][0]
}

/// 표 뒤 문단(표 문단 · 빈 문단 · 「표 뒤 문단」)은 마지막 표 조각 아래, 같은 쪽에 있다.
fn assert_after_paragraph_below_last_fragment(doc: &HwpDocument, pages: &[PageView], rel: &str) {
    let after = HOST as u64 + 2;
    let (ap, atop) = pages
        .iter()
        .enumerate()
        .find_map(|(p, v)| v.lines.iter().find(|l| l.0 == after).map(|l| (p, l.1)))
        .unwrap_or_else(|| panic!("{rel}: 「표 뒤 문단」 글줄이 없다"));
    let (tp, (_, tbottom)) = pages
        .iter()
        .enumerate()
        .flat_map(|(p, v)| v.tables.iter().map(move |b| (p, *b)))
        .last()
        .unwrap_or_else(|| panic!("{rel}: 표 조각이 없다"));
    assert_eq!(
        ap, tp,
        "{rel}: 표 뒤 문단은 마지막 표 조각과 같은 쪽이어야 한다"
    );
    assert!(
        atop >= tbottom,
        "{rel}: 표 뒤 문단({atop:.1})이 표 아랫변({tbottom:.1}) 아래여야 한다"
    );

    // 독립 Desktop Print에서도 보존되는 실제 빈 줄이다. 저장 1000HU 높이와
    // 600HU 후행 간격은 한 번 소비한다. 표 뒤 상대 원점에서 재가산하지 않는다.
    let (bp, btop) = pages
        .iter()
        .enumerate()
        .find_map(|(p, v)| v.lines.iter().find(|l| l.0 == after - 1).map(|l| (p, l.1)))
        .unwrap_or_else(|| panic!("{rel}: 표 뒤 실제 빈 줄이 없다"));
    assert_eq!(bp, ap, "{rel}: 빈 줄과 뒤 본문은 같은 쪽에 남는다");
    let paragraphs = &doc.document().sections[0].paragraphs;
    let blank = &paragraphs[after as usize - 1];
    assert!(blank.text.trim().is_empty() && blank.controls.is_empty());
    let [line] = blank.line_segs.as_slice() else {
        panic!("{rel}: 원본 빈 문단은 한 줄이다");
    };
    let next = &paragraphs[after as usize].line_segs[0];
    let source_advance = line.line_height + line.line_spacing;
    assert_eq!(next.vertical_pos - line.vertical_pos, source_advance);
    let advance_px = f64::from(source_advance) / 75.0;
    // render tree의 각 원점은 소수 첫 자리로 반올림된다(두 원점 차이 최대 0.1px).
    assert!(
        (atop - btop - advance_px).abs() <= 0.1 + 1e-6,
        "{rel}: 빈 줄의 실제 전진 {}은 원본·독립 Print의 {}px를 한 번 소비해야 한다",
        atop - btop,
        advance_px
    );
}

/// CellBreak: 쪽 바닥에 걸린 2행은 자르지 않고 6줄 전부 다음 쪽으로 간다. 1행은 1쪽에 남는다.
#[test]
fn issue_7620_cellbreak_float_table_carries_whole_row() {
    const REL: &str = "samples/issue7620/e-cellbreak-row-moves-next-page.hwp";
    let doc = load(REL);
    assert_host_table(&doc, REL, TablePageBreak::CellBreak);
    let (pages, where_) = layout(&doc, REL);
    for col in ["가", "나"] {
        for l in 1..=LINES {
            assert_eq!(
                page_of(&where_, &format!("{col}1-{l}")),
                0,
                "{REL}: 1행은 1쪽"
            );
            assert_eq!(
                page_of(&where_, &format!("{col}2-{l}")),
                1,
                "{REL}: CellBreak 표는 쪽 바닥에 걸린 2행을 자르지 않고 통째로 2쪽에 둔다 ({col}2-{l})"
            );
            assert_eq!(
                page_of(&where_, &format!("{col}3-{l}")),
                1,
                "{REL}: 3행은 2쪽"
            );
        }
    }
    assert_eq!(pages[0].tables.len(), 1, "{REL}: 1쪽 표 조각 하나");
    assert_eq!(pages[1].tables.len(), 1, "{REL}: 2쪽 표 조각 하나");
    assert_after_paragraph_below_last_fragment(&doc, &pages, REL);
}

/// RowBreak 대조군: 같은 입력에서 2행은 글줄 경계에서 나뉜다 — 앞 줄은 1쪽, 남은 줄은 2쪽.
/// CellBreak의 «행 통째 이월»이 RowBreak로 넓어지지 않았는지 함께 본다.
#[test]
fn issue_7620_rowbreak_float_table_splits_row_at_line() {
    const REL: &str = "samples/issue7620/f-rowbreak-row-splits-at-line.hwp";
    let doc = load(REL);
    assert_host_table(&doc, REL, TablePageBreak::RowBreak);
    let (pages, where_) = layout(&doc, REL);
    for col in ["가", "나"] {
        let row2: Vec<usize> = (1..=LINES)
            .map(|l| page_of(&where_, &format!("{col}2-{l}")))
            .collect();
        assert!(
            row2.contains(&0) && row2.contains(&1),
            "{REL}: RowBreak 표는 쪽 바닥에 걸린 2행을 두 쪽에 걸쳐 나눠야 한다 ({col}: {row2:?})"
        );
        assert!(
            row2.windows(2).all(|w| w[0] <= w[1]),
            "{REL}: 2행 글줄은 앞 줄이 1쪽, 뒤 줄이 2쪽인 순서여야 한다 ({col}: {row2:?})"
        );
        for l in 1..=LINES {
            assert_eq!(
                page_of(&where_, &format!("{col}1-{l}")),
                0,
                "{REL}: 1행은 1쪽"
            );
            assert_eq!(
                page_of(&where_, &format!("{col}3-{l}")),
                1,
                "{REL}: 3행은 2쪽"
            );
        }
    }
    // 두 열은 같은 글줄 경계에서 나뉜다(같은 줄 수의 셀).
    let split = |col: &str| {
        (1..=LINES)
            .filter(|l| page_of(&where_, &format!("{col}2-{l}")) == 0)
            .count()
    };
    assert_eq!(
        split("가"),
        split("나"),
        "{REL}: 두 열의 2행 분할 줄 수가 같아야 한다"
    );
    assert_eq!(pages[0].tables.len(), 1, "{REL}: 1쪽 표 조각 하나");
    assert_eq!(pages[1].tables.len(), 1, "{REL}: 2쪽 표 조각 하나");
    assert_after_paragraph_below_last_fragment(&doc, &pages, REL);
}
