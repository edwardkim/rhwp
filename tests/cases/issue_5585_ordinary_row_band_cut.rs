//! [#5585] «쪽 경계에서 나눔» 표의 **일반 행**이 내용과 안 여백은 남은 쪽에 다 들어가는데
//! 선언 행 높이만 넘칠 때, 행을 통째로 다음 쪽에 넘기지 않고 쪽 경계에서 자른다.
//!
//! ## 독립 기준 — 한글 2020 PDF (`pdf/issue6929/…-hwp-2020.pdf`, 17쪽)
//!
//! ```text
//!   pi=169 10×2 표, 쪽나눔=나눔(attr 2), 행 3 선언 196.3px · 내용 156.7px + 안 여백 15.1px
//!   14쪽  행 0..3 + 행 3 의 내용 전부(「…통지합니다.」)  ← 행 3 은 여기서 잘린다
//!   15쪽  행 3 의 남은 빈 밴드 → 행 4 「② 회사가…」 … 행 8 끝 「…해지할 수 있습니다.」
//!   16쪽  행 9 「【구글】Advertising Program」부터
//! ```
//!
//! 수정 전 rhwp 는 행 3 을 통째로 15쪽에 넘겨 14쪽에 186px 를 비웠고, 그 밀림이 16쪽에서
//! 한 문단을 넘겨 18쪽이 됐다. `#2236` 의 밴드 컷은 rowspan 이 걸친 행만 맡았다.
//!
//! 이 검사는 쪽수와 함께 **조각의 소속 관계**를 잠근다 — 행 3 의 내용이 14쪽 조각에서
//! 끝나고, 15쪽 조각은 같은 컷에서 이어받아(내용 중복 없음) 빈 밴드 뒤에 행 4 를 둔다.
//! 표를 끝내는 마지막 행의 경계(이어질 행이 없어 빈 밴드가 쪽 경계에서 끝남)는 corpus
//! 표본(156660604, 한글 5쪽)에서 확인했고 저장소 fixture 가 없어 여기서는 잠그지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::Value;

const SAMPLE: &str = "samples/issue6929/148776468_search_ad_terms_press_release.hwp";
const TABLE_PARA: u64 = 169;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("parse 148776468")
}

/// (쪽 번호, 시작 행, 끝 행, 시작 컷, 끝 컷) — 표 pi=169 의 조각을 쪽 순서로.
fn fragments(core: &DocumentCore) -> Vec<(u32, u64, u64, Vec<u64>, Vec<u64>)> {
    let dump = core.dump_page_items_json(None);
    let mut out = Vec::new();
    for (page_idx, page) in dump.as_array().expect("pages array").iter().enumerate() {
        for column in page["columns"].as_array().into_iter().flatten() {
            for item in column["items"].as_array().into_iter().flatten() {
                if item["kind"] == "partialTable" && item["paraIndex"] == TABLE_PARA {
                    let cut = |v: &Value| {
                        v.as_array()
                            .map(|a| a.iter().filter_map(Value::as_u64).collect())
                            .unwrap_or_default()
                    };
                    out.push((
                        page_idx as u32,
                        item["startRow"].as_u64().unwrap(),
                        item["endRow"].as_u64().unwrap(),
                        cut(&item["startCut"]),
                        cut(&item["endCut"]),
                    ));
                }
            }
        }
    }
    out
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn cell_text(cell: &RenderNode) -> String {
    let mut nodes = Vec::new();
    walk(cell, &mut nodes);
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>()
}

/// 쪽의 표 칸들 — (행, 열, 위, 아래, 글자).
fn cells(core: &DocumentCore, page: u32) -> Vec<(u16, u16, f64, f64, String)> {
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TableCell(c) => Some((
                c.row,
                c.col,
                n.bbox.y,
                n.bbox.y + n.bbox.height,
                cell_text(n),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn issue_5585_ordinary_row_band_cut_matches_hancom_page_count() {
    let core = load();
    assert_eq!(
        core.page_count(),
        17,
        "148776468 은 한글 2020 PDF 기준 17쪽이다 (행 3 을 통째로 넘기면 18쪽)"
    );
}

#[test]
fn issue_5585_row_content_stays_and_empty_band_continues() {
    let core = load();
    let frags = fragments(&core);
    assert!(
        frags.len() >= 3,
        "pi=169 조각이 셋 이상이어야 한다: {frags:?}"
    );

    // 행 3 의 내용을 담고 끝나는 조각 — 끝 컷이 행 3 의 모든 유닛을 소비한다.
    let (cut_page, _, end_row, _, end_cut) = frags
        .iter()
        .find(|f| f.2 == 4 && !f.4.is_empty())
        .cloned()
        .unwrap_or_else(|| panic!("행 3 에서 컷으로 끝나는 조각이 없다(통째 이월): {frags:?}"));
    assert_eq!(end_row, 4);

    // 다음 조각은 같은 컷에서 행 3 을 이어받아 행 8 까지 담고, 그다음 조각은 행 9 로 시작한다.
    let next = frags
        .iter()
        .find(|f| f.0 == cut_page + 1)
        .unwrap_or_else(|| panic!("{}쪽 다음 조각 없음: {frags:?}", cut_page + 1));
    assert_eq!(
        (next.1, next.2, &next.3),
        (3, 9, &end_cut),
        "이어받는 조각은 행 3 의 같은 컷에서 시작해 행 8 에서 끝나야 한다: {frags:?}"
    );
    let after = frags
        .iter()
        .find(|f| f.0 == cut_page + 2)
        .unwrap_or_else(|| panic!("{}쪽 조각 없음: {frags:?}", cut_page + 2));
    assert_eq!(
        after.1, 9,
        "그다음 조각은 행 9(【구글】)로 시작한다: {frags:?}"
    );

    // 자른 쪽: 행 3 의 내용(「통지합니다.」)이 이 쪽에 있다.
    let here = cells(&core, cut_page);
    let row3_here: String = here
        .iter()
        .filter(|c| c.0 == 3)
        .map(|c| c.4.as_str())
        .collect();
    assert!(
        row3_here.contains("통지합니다."),
        "행 3 의 마지막 줄이 자른 쪽에 있어야 한다: {row3_here:?}"
    );

    // 이어받은 쪽: 행 3 은 글자 없는 빈 밴드로만 남고, 행 4 가 그 아래에서 시작한다.
    let there = cells(&core, cut_page + 1);
    let band: Vec<_> = there.iter().filter(|c| c.0 == 3).collect();
    assert!(!band.is_empty(), "행 3 의 남은 빈 밴드가 이어져야 한다");
    for c in &band {
        assert!(
            c.4.trim().is_empty(),
            "행 3 의 내용이 중복되면 안 된다: {:?}",
            c.4
        );
        assert!(c.3 - c.2 > 1.0, "빈 밴드는 높이를 가져야 한다: {c:?}");
    }
    let band_bottom = band.iter().map(|c| c.3).fold(f64::MIN, f64::max);
    let row4 = there
        .iter()
        .find(|c| c.0 == 4 && c.4.contains("② 회사가"))
        .unwrap_or_else(|| panic!("행 4 가 이어받은 쪽에 없다"));
    assert!(
        row4.2 >= band_bottom - 0.5,
        "행 4 는 빈 밴드 아래에서 시작해야 한다: row4 top {} < band bottom {band_bottom}",
        row4.2
    );
}

/// 이어받은 HWP5 문단 기준 표 조각이 쪽 머리에서 표의 바깥 위 여백을 다시 열고, 표를 끝낸
/// 뒤 다음 문단은 바깥 아래 여백 뒤에서 시작한다.
///
/// 한글 2020 PDF 16쪽: 본문 위 94.5px, 표 위 괘선 96.0px(바깥 위 여백 140HU=1.87px),
/// 표 아래 괘선 291.0px, 다음 문단 `pi=170` 은 저장 vpos 14916HU(=본문 위 + 198.9px).
/// 수정 전 rhwp 는 조각을 본문 위에 붙이고(94.5px) 다음 문단도 표 바닥에 붙여(289.6px)
/// 16쪽 뒤 본문 전체가 3~4px 위로 올라갔다(Visual Sweep 82.8%).
#[test]
fn issue_5585_continued_para_relative_table_reopens_outer_margins() {
    let core = load();
    let frags = fragments(&core);
    let (page, ..) = *frags
        .iter()
        .find(|f| f.1 == 9 && f.2 == 10)
        .unwrap_or_else(|| panic!("행 9 로 시작하는 끝 조각이 없다: {frags:?}"));
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let body_top = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .map(|n| n.bbox.y)
        .expect("body");
    let table = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
                if t.para_index == Some(TABLE_PARA as usize))
        })
        .expect("표 조각");
    let next_line = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type, RenderNodeType::TextLine(l)
                if l.para_index == Some(TABLE_PARA as usize + 1))
        })
        .expect("표 뒤 문단");
    // 바깥 여백 140HU = 1.87px (96dpi).
    let margin = 140.0 * 96.0 / 7200.0;
    let top_gap = table.bbox.y - body_top;
    assert!(
        (top_gap - margin).abs() < 0.5,
        "이어받은 조각은 쪽 머리에서 바깥 위 여백({margin:.2}px)만큼 내려 시작해야 한다: {top_gap:.2}"
    );
    let bottom_gap = next_line.bbox.y - (table.bbox.y + table.bbox.height);
    assert!(
        bottom_gap >= margin - 0.5,
        "표 뒤 문단은 바깥 아래 여백({margin:.2}px) 뒤에서 시작해야 한다: {bottom_gap:.2}"
    );
}
