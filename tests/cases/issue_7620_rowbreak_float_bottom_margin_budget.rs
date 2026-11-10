//! 관계형.
//! 주석 갱신: 2026-11-10T16:20:00+09:00 (신규 파일).
//! 2026-11-10 측정 raw(`silhouette_raw_match_percent`, 96dpi, 기준 PDF pypdfium2 래스터,
//! `scripts/visual_sweep.py --silhouette-only --compat 2022`), 원본별 전2쪽:
//!   b-rowbreak-bottom-margin-overflow: 1·2쪽 Native·fresh WASM 각 100.00%
//!     (수정 전 devel b5064fae1: Native 89.88% / 59.65%, fresh WASM 90.21% / 60.49%);
//!     전2쪽 최저 Native 100.00% / fresh WASM 100.00%.
//!   c-rowbreak-table-moves-next-page: 1·2쪽 Native·fresh WASM 각 100.00%; 최저 100.00% / 100.00%.
//!   d-rowbreak-splits-at-row-boundary: 1·2쪽 Native·fresh WASM 각 100.00%; 최저 100.00% / 100.00%.
//! 근거: mydocs/pr/assets/issue_7620_rowbreak_bottom_margin_budget/README.md (TSV 원본·대표 PNG).
//!
//! Issue #7620 후속 — 쪽 경계 나눔(RowBreak) 자리 차지 표의 첫 조각과 바깥 아래 여백.
//!
//! 한/글은 문단 기준 자리 차지 표의 조각마다 바깥 아래 여백을 남긴다(#8149). 쪽 바닥에서
//! `바깥 위 여백 + 첫 행 + 바깥 아래 여백`이 들어가지 않으면 표를 나누지 않는다. 표만 다음 쪽
//! 위(본문 윗변 + 바깥 위 여백)로 옮기고, 표 문단과 뒤 글줄은 현재 쪽에 계속 흘린다. 한 행이라도
//! 들어가면 행 경계에서 나누며 첫 조각 아래에도 바깥 아래 여백을 남긴다.
//!
//! 입력은 `samples/issue7620/generate-budget.mjs`로 만든 생성 대조군이다(한컴 저장본 아님).
//! 기준은 같은 입력의 한컴독스 Print PDF `pdf/issue7620/*-hancomdocs.pdf`다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::model::table::{Table, TablePageBreak};
use rhwp::wasm_api::HwpDocument;

/// 위치 비교 허용치(px). 저장 HWPUNIT 정수와 렌더 트리 소수 한 자리 반올림을 덮는다.
const TOL_PX: f64 = 0.6;

fn load(rel: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("{rel} 파싱: {e:?}"))
}

/// 96dpi 에서 HWPUNIT → px.
fn hu_px(hu: i32) -> f64 {
    f64::from(hu) / 75.0
}

/// 고정 입력의 전제: 비TAC · 자리 차지 · 문단 기준 · 오프셋 0 · RowBreak · 빈 표 문단.
fn host_table(doc: &HwpDocument, rel: &str, host: usize) -> Table {
    let para = &doc.document().sections[0].paragraphs[host];
    let t = match &para.controls[..] {
        [Control::Table(t)] => (**t).clone(),
        other => panic!("{rel}: 전제 — 표 문단에 표 하나만 있어야 한다 ({other:?})"),
    };
    assert!(para.text.trim().is_empty(), "{rel}: 전제 — 빈 문단");
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
    assert_eq!(
        t.page_break,
        TablePageBreak::RowBreak,
        "{rel}: 전제 — 쪽 나눔 RowBreak"
    );
    assert!(t.outer_margin_bottom > 0, "{rel}: 전제 — 바깥 아래 여백");
    t
}

/// 한 쪽에서 본 표 조각과 본문 글줄.
struct PageView {
    body_top: f64,
    body_bottom: f64,
    /// 표 문단(`host`)의 표 상자 (y, 아랫변).
    tables: Vec<(f64, f64)>,
    /// 본문 글줄의 (문단 번호, 윗변, 아랫변). 셀 안 줄은 빼고 본다.
    lines: Vec<(u64, f64, f64)>,
}

fn page_view(doc: &HwpDocument, page: u32, host: usize) -> PageView {
    fn bbox(node: &serde_json::Value) -> (f64, f64) {
        let b = &node["bbox"];
        let y = b["y"].as_f64().unwrap_or(f64::NAN);
        (y, y + b["h"].as_f64().unwrap_or(f64::NAN))
    }
    fn walk(node: &serde_json::Value, host: u64, view: &mut PageView) {
        match node["type"].as_str() {
            Some("Body") if view.body_top.is_nan() => {
                (view.body_top, view.body_bottom) = bbox(node);
            }
            Some("Table") => {
                if node["pi"].as_u64() == Some(host) {
                    view.tables.push(bbox(node));
                }
                return; // 셀 안의 줄은 본문 글줄이 아니다
            }
            Some("TextLine") => {
                if let Some(pi) = node["pi"].as_u64() {
                    let (top, bottom) = bbox(node);
                    view.lines.push((pi, top, bottom));
                }
            }
            _ => {}
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, host, view);
        }
    }
    let json = doc.get_page_render_tree(page).expect("render tree");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("render tree JSON");
    let mut view = PageView {
        body_top: f64::NAN,
        body_bottom: f64::NAN,
        tables: Vec::new(),
        lines: Vec::new(),
    };
    walk(&tree, host as u64, &mut view);
    assert!(
        view.body_top.is_finite(),
        "{}쪽에 Body 노드가 없다",
        page + 1
    );
    view
}

/// 모든 쪽의 보기를 쪽 순서대로 돌려준다.
fn pages(doc: &HwpDocument, host: usize) -> Vec<PageView> {
    (0..doc.page_count())
        .map(|p| page_view(doc, p, host))
        .collect()
}

/// 문단 `pi`의 첫 본문 글줄이 놓인 (쪽 번호 0부터, 윗변).
fn first_line_of(pages: &[PageView], pi: u64) -> (usize, f64) {
    pages
        .iter()
        .enumerate()
        .find_map(|(p, v)| v.lines.iter().find(|l| l.0 == pi).map(|l| (p, l.1)))
        .unwrap_or_else(|| panic!("문단 {pi}의 본문 글줄이 없다"))
}

fn assert_near(what: &str, actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= TOL_PX,
        "{what}: {actual:.1}px 이 기대 {expected:.1}px 과 다르다",
    );
}

/// 첫 행이 바깥 아래 여백과 함께 들어가지 않으면 표를 나누지 않고 다음 쪽 위로 옮기며,
/// 표 문단과 뒤 글줄은 현재 쪽에 남는다.
#[test]
fn issue_7620_rowbreak_without_first_row_moves_table_and_keeps_text() {
    const REL: &str = "samples/issue7620/b-rowbreak-bottom-margin-overflow.hwp";
    const HOST: usize = 37;
    let doc = load(REL);
    let t = host_table(&doc, REL, HOST);
    let pages = pages(&doc, HOST);
    let fragments: Vec<(usize, (f64, f64))> = pages
        .iter()
        .enumerate()
        .flat_map(|(p, v)| v.tables.iter().map(move |b| (p, *b)))
        .collect();
    assert_eq!(
        fragments.len(),
        1,
        "{REL}: 표가 나뉘지 않고 한 쪽에만 있어야 한다 ({fragments:?})"
    );
    let (table_page, (table_top, _)) = fragments[0];
    let after = HOST as u64 + 2; // 표 문단 · 빈 문단 · 「표 뒤 문단」
    let (after_page, after_top) = first_line_of(&pages, after);
    assert!(
        after_page < table_page,
        "{REL}: 뒤 문단({}쪽)이 표({}쪽)보다 앞쪽에 남아야 한다",
        after_page + 1,
        table_page + 1
    );
    let prev = &pages[after_page];
    let (_, last_before) = first_line_of(&pages, HOST as u64 - 1);
    assert!(
        last_before < after_top && after_top < prev.body_bottom,
        "{REL}: 뒤 문단이 앞 문단 아래 본문 안에 있어야 한다"
    );
    let next = &pages[table_page];
    assert_near(
        "옮긴 표 윗변 = 본문 윗변 + 바깥 여백 위",
        table_top,
        next.body_top + hu_px(i32::from(t.outer_margin_top)),
    );
}

/// 표 문단 줄만 들어가고 표가 들어가지 않으면 표는 다음 쪽 위에 서고, 그 쪽으로 넘어간
/// 뒤 문단은 표 아랫변 + 바깥 아래 여백에서 시작한다(정상 대조군).
#[test]
fn issue_7620_rowbreak_moved_table_reserves_box_on_next_page() {
    const REL: &str = "samples/issue7620/c-rowbreak-table-moves-next-page.hwp";
    const HOST: usize = 39;
    let doc = load(REL);
    let t = host_table(&doc, REL, HOST);
    let pages = pages(&doc, HOST);
    let fragments: Vec<(usize, (f64, f64))> = pages
        .iter()
        .enumerate()
        .flat_map(|(p, v)| v.tables.iter().map(move |b| (p, *b)))
        .collect();
    assert_eq!(
        fragments.len(),
        1,
        "{REL}: 표가 나뉘지 않아야 한다 ({fragments:?})"
    );
    let (table_page, (table_top, table_bottom)) = fragments[0];
    let next = &pages[table_page];
    assert_near(
        "표 윗변 = 본문 윗변 + 바깥 여백 위",
        table_top,
        next.body_top + hu_px(i32::from(t.outer_margin_top)),
    );
    let (after_page, after_top) = first_line_of(&pages, HOST as u64 + 2);
    assert_eq!(
        after_page, table_page,
        "{REL}: 뒤 문단이 표와 같은 쪽에 있어야 한다"
    );
    assert_near(
        "뒤 문단 윗변 = 표 아랫변 + 바깥 여백 아래",
        after_top,
        table_bottom + hu_px(i32::from(t.outer_margin_bottom)),
    );
}

/// 첫 행들이 바깥 아래 여백과 함께 들어가면 행 경계에서 나누고, 첫 조각 아래에 바깥 아래
/// 여백을 남긴다. 모든 행은 한 번씩만 나온다(대조군).
#[test]
fn issue_7620_rowbreak_splits_when_rows_fit_with_bottom_margin() {
    const REL: &str = "samples/issue7620/d-rowbreak-splits-at-row-boundary.hwp";
    const HOST: usize = 20;
    let doc = load(REL);
    let t = host_table(&doc, REL, HOST);
    let pages = pages(&doc, HOST);
    let fragments: Vec<(usize, (f64, f64))> = pages
        .iter()
        .enumerate()
        .flat_map(|(p, v)| v.tables.iter().map(move |b| (p, *b)))
        .collect();
    assert_eq!(
        fragments.len(),
        2,
        "{REL}: 표가 두 조각이어야 한다 ({fragments:?})"
    );
    let (first_page, (first_top, first_bottom)) = fragments[0];
    let (second_page, _) = fragments[1];
    assert_eq!(
        second_page,
        first_page + 1,
        "{REL}: 두 조각은 연속한 쪽에 있어야 한다"
    );
    let (_, host_top) = first_line_of(&pages, HOST as u64);
    assert_near(
        "첫 조각 윗변 = 표 문단 윗변 + 바깥 여백 위",
        first_top,
        host_top + hu_px(i32::from(t.outer_margin_top)),
    );
    assert!(
        first_bottom + hu_px(i32::from(t.outer_margin_bottom)) <= pages[first_page].body_bottom + TOL_PX,
        "{REL}: 첫 조각 아래에 바깥 아래 여백이 남아야 한다 ({first_bottom:.1} + 여백 > 본문 바닥 {:.1})",
        pages[first_page].body_bottom
    );
    // 행 보존: 표 조각 안 셀 줄의 「가N」을 모으면 1..=행 수가 한 번씩이다(누락·중복 없음).
    let mut seen = Vec::new();
    for p in 0..doc.page_count() {
        let json = doc.get_page_render_tree(p).expect("render tree");
        let tree: serde_json::Value = serde_json::from_str(&json).expect("render tree JSON");
        collect_first_column_rows(&tree, HOST as u64, false, &mut seen);
    }
    seen.sort_unstable();
    let rows = u32::try_from(t.row_count).expect("행 수");
    assert_eq!(
        seen,
        (1..=rows).collect::<Vec<_>>(),
        "{REL}: 모든 행이 한 번씩 나와야 한다"
    );
}

/// 표(`host`) 안 셀 줄의 글자를 이어 「가N」의 N 을 모은다.
fn collect_first_column_rows(
    node: &serde_json::Value,
    host: u64,
    in_table: bool,
    out: &mut Vec<u32>,
) {
    let in_table = in_table || (node["type"] == "Table" && node["pi"].as_u64() == Some(host));
    if in_table && node["type"] == "TextLine" {
        let mut text = String::new();
        fn runs(node: &serde_json::Value, text: &mut String) {
            if node["type"] == "TextRun" {
                text.push_str(node["text"].as_str().unwrap_or(""));
            }
            for child in node["children"].as_array().into_iter().flatten() {
                runs(child, text);
            }
        }
        runs(node, &mut text);
        if let Some(n) = text.trim().strip_prefix('가').and_then(|n| n.parse().ok()) {
            out.push(n);
        }
        return;
    }
    for child in node["children"].as_array().into_iter().flatten() {
        collect_first_column_rows(child, host, in_table, out);
    }
}
