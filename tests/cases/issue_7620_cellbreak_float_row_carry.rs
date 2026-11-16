//! 관계형·고정형(독립 PDF의 전체 쪽수).
//! 주석 갱신: 2026-11-13T14:13:18+09:00.
//! 직전 파일 변경: 2026-11-13T11:51:27+09:00, b4b8c2726037583a605f3f2f16b641654e19b9a3 (Git committer 시각).
//! 2026-11-13 실측: 제품 source e5a6a1ff9f(a785e23f68과 생산 파일 동일), 96dpi·compat 2022,
//! Desktop Hancom 2020 11.0.0.9136 Print(method 0, one-up), pypdfium2 5.13.0,
//! scripts/visual_sweep.py --silhouette-only의 silhouette_raw_match_percent.
//! 표지 e·f: 원본별 전 2쪽 Native/fresh WASM 모두 100.00%, 각각 전쪽 최저 100.00%.
//! 구형 표지 없는 e·f(legacy-unmarked): 원본별 전 2쪽 Native/fresh WASM 모두 100.00%, 각각 최저 100.00%.
//! 한컴 실제 재저장 e·f 정상 대조군도 각 전 2쪽·양쪽 100.00%다(이 파일의 고정 좌표 기대값으로 쓰지 않음).
//! 근거: mydocs/pr/assets/pr8975-updated-66900a40/README.md (입력·PDF·전쪽 TSV·직접 판독).
//! 표지 입력은 보정 전에도 양쪽 전 4쪽이 100.00%다. 구형 입력 p2는 보정 전 Native 94.62/93.46%,
//! WASM 94.90/93.79%이며 실제 빈 줄 전진이 원본 1600HU보다 600HU 커 아래 전진 관계를 위반했다.
//!
//! #8975의 기여자 head 66900a40be에서 라벨의 독립 내용 순서·소유 셀·포함·글줄 비겹침·열 정렬과
//! 반례 판정력 검사를 보존했다. #9047의 실제 빈 줄 전진 관계도 함께 보호한다.
//! 입력은 rhwp 생성 대조군이며 표지 없는 구형과 /RhwpLineSegTags 표지가 있는 현행을 구분한다.
//! CellBreak는 행 전체를 다음 쪽으로, RowBreak는 글줄 경계에서 이월한다. 두 모드는 같은 내용을 보존한다.
//! 원 기여의 웹 PDF는 이력이고 판정 기준은 같은 입력의 실제 Desktop Print다.
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
/// 열 라벨(열 번호 순).
const COLS: [&str; 2] = ["가", "나"];
/// 독립 기준 PDF의 전체 쪽수(두 입력 모두).
const PDF_PAGES: u32 = 2;
/// 포함·정렬 판정의 좌표 허용치(px). 렌더 트리 bbox의 부동소수 반올림만 흡수한다.
const EPS: f64 = 0.5;

const E_REL: &str = "samples/issue7620/e-cellbreak-row-moves-next-page.hwp";
const F_REL: &str = "samples/issue7620/f-rowbreak-row-splits-at-line.hwp";
const E_LEGACY_REL: &str = "samples/issue7620/legacy-unmarked/e-cellbreak-row-moves-next-page.hwp";
const F_LEGACY_REL: &str = "samples/issue7620/legacy-unmarked/f-rowbreak-row-splits-at-line.hwp";

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

/// 렌더 트리 bbox(왼·위·오른·아래).
#[derive(Clone, Copy, Debug)]
struct Rect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl Rect {
    fn of(node: &serde_json::Value) -> Rect {
        let b = &node["bbox"];
        let f = |k: &str| b[k].as_f64().unwrap_or(f64::NAN);
        let (x, y) = (f("x"), f("y"));
        Rect {
            left: x,
            top: y,
            right: x + f("w"),
            bottom: y + f("h"),
        }
    }

    /// `self`가 `outer` 안에 있다(허용치 `EPS`). NaN이면 거짓.
    fn inside(&self, outer: &Rect) -> bool {
        self.left >= outer.left - EPS
            && self.right <= outer.right + EPS
            && self.top >= outer.top - EPS
            && self.bottom <= outer.bottom + EPS
    }
}

/// 셀 글줄을 소유한 셀(렌더 트리 `Cell`, 행·열 0부터).
#[derive(Clone, Copy, Debug)]
struct CellBox {
    row: u64,
    col: u64,
    rect: Rect,
}

/// 표 안 글줄 하나.
#[derive(Clone, Debug)]
struct CellLine {
    /// 공백을 뺀 글줄 텍스트(예: `가2-1`).
    label: String,
    rect: Rect,
    /// 가장 가까운 조상 셀.
    cell: Option<CellBox>,
    /// 소속 표 조각(`PageView::tables` 번호).
    fragment: usize,
}

/// 한 쪽에서 본 표 조각과 셀 글줄·본문 글줄.
#[derive(Clone, Default)]
struct PageView {
    /// 표 문단(`HOST`)의 표 조각 상자.
    tables: Vec<Rect>,
    /// 표 안 글줄(렌더 트리 순회 순서).
    cell_lines: Vec<CellLine>,
    /// 본문 글줄의 (문단 번호, 윗변). 셀 안 줄은 빼고 본다.
    lines: Vec<(u64, f64)>,
}

fn page_view(doc: &HwpDocument, page: u32) -> PageView {
    fn runs(node: &serde_json::Value, out: &mut String) {
        if node["type"].as_str() == Some("TextRun") {
            out.push_str(node["text"].as_str().unwrap_or(""));
        }
        for child in node["children"].as_array().into_iter().flatten() {
            runs(child, out);
        }
    }
    fn walk(
        node: &serde_json::Value,
        fragment: Option<usize>,
        cell: Option<CellBox>,
        view: &mut PageView,
    ) {
        let (mut fragment, mut cell) = (fragment, cell);
        match node["type"].as_str() {
            Some("Table") if fragment.is_none() && node["pi"].as_u64() == Some(HOST as u64) => {
                view.tables.push(Rect::of(node));
                fragment = Some(view.tables.len() - 1);
            }
            Some("Cell") if fragment.is_some() => {
                cell = Some(CellBox {
                    row: node["row"].as_u64().unwrap_or(u64::MAX),
                    col: node["col"].as_u64().unwrap_or(u64::MAX),
                    rect: Rect::of(node),
                });
            }
            Some("TextLine") => {
                if let Some(fragment) = fragment {
                    let mut text = String::new();
                    runs(node, &mut text);
                    let label: String = text.chars().filter(|c| !c.is_whitespace()).collect();
                    if !label.is_empty() {
                        view.cell_lines.push(CellLine {
                            label,
                            rect: Rect::of(node),
                            cell,
                            fragment,
                        });
                    }
                } else if let Some(pi) = node["pi"].as_u64() {
                    view.lines.push((pi, Rect::of(node).top));
                }
                return;
            }
            _ => {}
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, fragment, cell, view);
        }
    }
    let json = doc.get_page_render_tree(page).expect("render tree");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("render tree JSON");
    let mut view = PageView::default();
    walk(&tree, None, None, &mut view);
    view
}

/// 라벨 `가2-1` → (열 번호 0부터, 행 1부터, 줄 1부터). 형식이 다르면 `None`.
fn parse_label(label: &str) -> Option<(usize, usize, usize)> {
    let col = COLS.iter().position(|c| label.starts_with(c))?;
    let (r, l) = label[COLS[col].len()..].split_once('-')?;
    Some((col, r.parse().ok()?, l.parse().ok()?))
}

/// 열 하나의 독립 기대 순서: 행 1..=ROWS, 그 안에서 줄 1..=LINES.
fn expected_column(col: &str) -> Vec<String> {
    (1..=ROWS)
        .flat_map(|r| (1..=LINES).map(move |l| format!("{col}{r}-{l}")))
        .collect()
}

/// 라벨마다 놓인 쪽 번호 목록(0부터).
fn pages_of_labels(pages: &[PageView]) -> BTreeMap<String, Vec<usize>> {
    let mut where_: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (p, v) in pages.iter().enumerate() {
        for line in &v.cell_lines {
            where_.entry(line.label.clone()).or_default().push(p);
        }
    }
    where_
}

/// 셀 글줄의 전수·내용 순서 검사.
///
/// 열마다 쪽 순서대로, 같은 쪽 안에서는 실제 윗변 순서로 라벨을 늘어놓은 열이
/// 독립 기대 열과 같아야 한다. 같은 쪽의 같은 열에서 윗변이 같은(겹친) 글줄도 거부한다.
/// 렌더 트리 순회 순서는 쓰지 않는다 — 순회 순서와 좌표가 함께 뒤바뀐 배치도 잡기 위해서다.
fn check_inventory_and_order(pages: &[PageView]) -> Result<(), String> {
    let where_ = pages_of_labels(pages);
    let unknown: Vec<&String> = where_
        .keys()
        .filter(|l| !matches!(parse_label(l), Some((_, r, n)) if (1..=ROWS).contains(&r) && (1..=LINES).contains(&n)))
        .collect();
    if !unknown.is_empty() {
        return Err(format!("예상 밖의 셀 글줄이 있다 ({unknown:?})"));
    }
    for col in COLS {
        for label in expected_column(col) {
            let at = where_.get(&label).cloned().unwrap_or_default();
            if at.len() != 1 {
                return Err(format!(
                    "셀 글줄 {label}은 정확히 한 번 나와야 한다 ({at:?})"
                ));
            }
        }
        let mut actual: Vec<String> = Vec::new();
        for (p, v) in pages.iter().enumerate() {
            let mut on_page: Vec<&CellLine> = v
                .cell_lines
                .iter()
                .filter(|l| l.label.starts_with(col))
                .collect();
            on_page.sort_by(|a, b| a.rect.top.total_cmp(&b.rect.top));
            if let Some(w) = on_page
                .windows(2)
                .find(|w| w[0].rect.top + EPS >= w[1].rect.top)
            {
                return Err(format!(
                    "{}쪽 {col} 열 글줄 {}({:.1})·{}({:.1})의 윗변이 겹친다",
                    p + 1,
                    w[0].label,
                    w[0].rect.top,
                    w[1].label,
                    w[1].rect.top
                ));
            }
            actual.extend(on_page.into_iter().map(|l| l.label.clone()));
        }
        let expected = expected_column(col);
        if actual != expected {
            let at = actual
                .iter()
                .zip(&expected)
                .position(|(a, e)| a != e)
                .unwrap_or(actual.len().min(expected.len()));
            return Err(format!(
                "{col} 열 글줄의 위→아래 내용 순서가 행·줄 번호 순서와 다르다 \
                 ({at}번째: 실제 {:?}, 기대 {:?}; 실제 전체 {actual:?})",
                actual.get(at),
                expected.get(at)
            ));
        }
    }
    Ok(())
}

/// 셀 내부 포함 관계: 글줄은 라벨의 행·열과 같은 셀 안, 셀은 소속 표 조각 안에 있고,
/// 같은 셀의 이웃 글줄은 위아래로 겹치지 않는다. 같은 행·줄 번호의 두 열 글줄은 같은 쪽에서
/// 같은 높이에 놓인다(두 열의 셀 속성·글자모양이 같은 생성 입력).
fn check_containment(pages: &[PageView]) -> Result<(), String> {
    for (p, v) in pages.iter().enumerate() {
        for line in &v.cell_lines {
            let (col, r, _) = parse_label(&line.label)
                .ok_or_else(|| format!("{}쪽 라벨 형식 오류 {}", p + 1, line.label))?;
            let cell = line
                .cell
                .ok_or_else(|| format!("{}쪽 {}: 소유 셀이 없다", p + 1, line.label))?;
            if (cell.row, cell.col) != ((r - 1) as u64, col as u64) {
                return Err(format!(
                    "{}쪽 {}: 소유 셀이 (행 {}, 열 {})여야 하는데 (행 {}, 열 {})이다",
                    p + 1,
                    line.label,
                    r - 1,
                    col,
                    cell.row,
                    cell.col
                ));
            }
            if !line.rect.inside(&cell.rect) {
                return Err(format!(
                    "{}쪽 {}: 글줄 {:?}이 셀 {:?} 밖으로 나갔다",
                    p + 1,
                    line.label,
                    line.rect,
                    cell.rect
                ));
            }
            let table = v.tables[line.fragment];
            if !cell.rect.inside(&table) {
                return Err(format!(
                    "{}쪽 {}: 셀 {:?}이 표 조각 {:?} 밖으로 나갔다",
                    p + 1,
                    line.label,
                    cell.rect,
                    table
                ));
            }
        }
        // 같은 셀의 이웃 글줄은 겹치지 않는다(윗변 순으로 정렬해 아랫변 ≤ 다음 윗변).
        let mut by_cell: BTreeMap<(u64, u64), Vec<&CellLine>> = BTreeMap::new();
        for line in &v.cell_lines {
            if let Some(c) = line.cell {
                by_cell.entry((c.row, c.col)).or_default().push(line);
            }
        }
        for lines in by_cell.values_mut() {
            lines.sort_by(|a, b| a.rect.top.total_cmp(&b.rect.top));
            if let Some(w) = lines
                .windows(2)
                .find(|w| w[0].rect.bottom > w[1].rect.top + EPS)
            {
                return Err(format!(
                    "{}쪽 같은 셀의 글줄 {}·{}이 겹친다 ({:?} / {:?})",
                    p + 1,
                    w[0].label,
                    w[1].label,
                    w[0].rect,
                    w[1].rect
                ));
            }
        }
        // 두 열의 같은 행·줄 글줄은 같은 높이.
        for line in v.cell_lines.iter().filter(|l| l.label.starts_with(COLS[0])) {
            let twin = format!("{}{}", COLS[1], &line.label[COLS[0].len()..]);
            if let Some(other) = v.cell_lines.iter().find(|l| l.label == twin) {
                if (other.rect.top - line.rect.top).abs() > EPS {
                    return Err(format!(
                        "{}쪽 {}({:.1})와 {twin}({:.1})의 윗변이 다르다",
                        p + 1,
                        line.label,
                        line.rect.top,
                        other.rect.top
                    ));
                }
            }
        }
    }
    Ok(())
}

/// 전 쪽 보기와, 셀 글줄 라벨마다 놓인 쪽 번호 목록(0부터).
fn layout(doc: &HwpDocument, rel: &str) -> (Vec<PageView>, BTreeMap<String, Vec<usize>>) {
    assert_eq!(
        doc.page_count(),
        PDF_PAGES,
        "{rel}: 전체 쪽수가 기준 PDF({PDF_PAGES}쪽)와 같아야 한다"
    );
    let pages: Vec<PageView> = (0..doc.page_count()).map(|p| page_view(doc, p)).collect();
    check_inventory_and_order(&pages).unwrap_or_else(|e| panic!("{rel}: {e}"));
    check_containment(&pages).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let where_ = pages_of_labels(&pages);
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
    let (tp, last) = pages
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
        atop >= last.bottom,
        "{rel}: 표 뒤 문단({atop:.1})이 표 아랫변({:.1}) 아래여야 한다",
        last.bottom
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
    for rel in [E_REL, E_LEGACY_REL] {
        assert_cellbreak_float_table_carries_whole_row(rel);
    }
}

fn assert_cellbreak_float_table_carries_whole_row(rel: &str) {
    let doc = load(rel);
    assert_host_table(&doc, rel, TablePageBreak::CellBreak);
    let (pages, where_) = layout(&doc, rel);
    for col in COLS {
        for l in 1..=LINES {
            assert_eq!(
                page_of(&where_, &format!("{col}1-{l}")),
                0,
                "{rel}: 1행은 1쪽"
            );
            assert_eq!(
                page_of(&where_, &format!("{col}2-{l}")),
                1,
                "{rel}: CellBreak 표는 쪽 바닥에 걸린 2행을 자르지 않고 통째로 2쪽에 둔다 ({col}2-{l})"
            );
            assert_eq!(
                page_of(&where_, &format!("{col}3-{l}")),
                1,
                "{rel}: 3행은 2쪽"
            );
        }
    }
    assert_eq!(pages[0].tables.len(), 1, "{rel}: 1쪽 표 조각 하나");
    assert_eq!(pages[1].tables.len(), 1, "{rel}: 2쪽 표 조각 하나");
    assert_after_paragraph_below_last_fragment(&doc, &pages, rel);
}

/// RowBreak 대조군: 같은 입력에서 2행은 글줄 경계에서 나뉜다 — 앞 줄은 1쪽, 남은 줄은 2쪽.
/// CellBreak의 «행 통째 이월»이 RowBreak로 넓어지지 않았는지 함께 본다.
#[test]
fn issue_7620_rowbreak_float_table_splits_row_at_line() {
    for rel in [F_REL, F_LEGACY_REL] {
        assert_rowbreak_float_table_splits_row_at_line(rel);
    }
}

fn assert_rowbreak_float_table_splits_row_at_line(rel: &str) {
    let doc = load(rel);
    assert_host_table(&doc, rel, TablePageBreak::RowBreak);
    let (pages, where_) = layout(&doc, rel);
    for col in COLS {
        let row2: Vec<usize> = (1..=LINES)
            .map(|l| page_of(&where_, &format!("{col}2-{l}")))
            .collect();
        assert!(
            row2.contains(&0) && row2.contains(&1),
            "{rel}: RowBreak 표는 쪽 바닥에 걸린 2행을 두 쪽에 걸쳐 나눠야 한다 ({col}: {row2:?})"
        );
        for l in 1..=LINES {
            assert_eq!(
                page_of(&where_, &format!("{col}1-{l}")),
                0,
                "{rel}: 1행은 1쪽"
            );
            assert_eq!(
                page_of(&where_, &format!("{col}3-{l}")),
                1,
                "{rel}: 3행은 2쪽"
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
        split(COLS[0]),
        split(COLS[1]),
        "{rel}: 두 열의 2행 분할 줄 수가 같아야 한다"
    );
    assert_eq!(pages[0].tables.len(), 1, "{rel}: 1쪽 표 조각 하나");
    assert_eq!(pages[1].tables.len(), 1, "{rel}: 2쪽 표 조각 하나");
    assert_after_paragraph_below_last_fragment(&doc, &pages, rel);
}

/// 판정력 확인: 실제 두 입력의 배치는 통과하고, 그 배치에서 만든 반례는 거부한다.
/// - 같은 쪽 인접 글줄(`가2-1`·`가2-2`)의 라벨 역전 → 내용 순서 검사가 거부.
///   (쪽 소속·등장 횟수·순회 순서의 y 증가는 그대로 유지되는 반례)
/// - `가2-1` 글줄을 아래로 10000px 이동(표 밖) → 셀 포함 검사가 거부.
/// - `가2-1`을 다른 행 셀(1행) 소유로 바꿈 → 셀 포함 검사가 거부.
#[test]
fn issue_7620_cell_line_checks_reject_counterexamples() {
    for rel in [E_REL, F_REL] {
        let doc = load(rel);
        let pages: Vec<PageView> = (0..doc.page_count()).map(|p| page_view(&doc, p)).collect();
        check_inventory_and_order(&pages).unwrap_or_else(|e| panic!("{rel}: 정상 대조군 {e}"));
        check_containment(&pages).unwrap_or_else(|e| panic!("{rel}: 정상 대조군 {e}"));

        // 두 반례 대상 글줄은 같은 쪽에 있어야 한다(e: 2쪽, f: 1쪽).
        let find = |pages: &[PageView], label: &str| {
            pages
                .iter()
                .enumerate()
                .find_map(|(p, v)| {
                    v.cell_lines
                        .iter()
                        .position(|l| l.label == label)
                        .map(|i| (p, i))
                })
                .unwrap_or_else(|| panic!("{rel}: {label} 없음"))
        };
        let (p1, i1) = find(&pages, "가2-1");
        let (p2, i2) = find(&pages, "가2-2");
        assert_eq!(p1, p2, "{rel}: 반례 전제 — 가2-1·가2-2가 같은 쪽");

        let mut swapped = pages.clone();
        let lines = &mut swapped[p1].cell_lines;
        let (a, b) = (lines[i1].label.clone(), lines[i2].label.clone());
        lines[i1].label = b;
        lines[i2].label = a;
        let err = check_inventory_and_order(&swapped)
            .expect_err(&format!("{rel}: 인접 글줄 라벨 역전을 거부해야 한다"));
        assert!(err.contains("내용 순서"), "{rel}: 거부 사유 {err}");

        let mut pushed = pages.clone();
        let r = &mut pushed[p1].cell_lines[i1].rect;
        r.top += 10000.0;
        r.bottom += 10000.0;
        let err = check_containment(&pushed)
            .expect_err(&format!("{rel}: 표 밖으로 밀린 글줄을 거부해야 한다"));
        assert!(
            err.contains("셀") && err.contains("밖"),
            "{rel}: 거부 사유 {err}"
        );

        let mut wrong_cell = pages.clone();
        if let Some(c) = wrong_cell[p1].cell_lines[i1].cell.as_mut() {
            c.row = 0;
        }
        let err = check_containment(&wrong_cell)
            .expect_err(&format!("{rel}: 다른 행 셀 소유를 거부해야 한다"));
        assert!(err.contains("소유 셀"), "{rel}: 거부 사유 {err}");
    }
}
