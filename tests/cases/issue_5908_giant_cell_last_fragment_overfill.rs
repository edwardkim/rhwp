//! [Issue #5908] giant cell 의 마지막 쪽 조각이 0-전진으로 무너져 남은 4.5쪽이
//! 한 쪽에 겹쳐 그려진다.
//!
//! `samples/table_giant_cell_overfill.hwpx` 는 본문 문단 0 이 5행×1열 표를 담고,
//! 그 표의 4행 셀 하나에 654문단·40,824자(문서 거의 전부)가 들어 있는 giant cell
//! 문서다. 렌더러는 이 행을 `PartialTable` 조각으로 40쪽에 걸쳐 나누는데, 마지막
//! 조각에서만 무너졌다.
//!
//! 원인: `scan_block_table_split_rows` 의 mixed-nested 재시도가 실측 초과분
//! (`over`)에서 `painted_tail` 을 한 번 더 빼 예산이 1,005.4px → 12.4px 로 0 에
//! 수렴한다. 그 예산으로는 유닛을 못 담아 `res2.consumed_height = 0` 이 되고
//! `end_row = r` 로 되돌아간다. 그런데 `r == cursor_row` 인 continuation 조각은
//! 이미 그 행 중간(`row_start_cut`)에서 시작해 이월할 앞부분이 없으므로, 호출부가
//! `end_row >= row_count && split_end_limit == 0` 을 "나머지가 이 쪽에 다 들어감"
//! 으로 읽어 남은 92유닛을 클립 없는 종결 조각으로 한 쪽에 쏟았다.
//!
//! 수정 전/후 실측 (한글 2024 정본 48쪽):
//!
//! | 항목 | 수정 전 | 수정 후 | 정본 |
//! | --- | --- | --- | --- |
//! | 쪽수 | 42 | 47 | 48 |
//! | 40쪽(0기준 39) 글자 baseline 최댓값 | 5,141.6px | 1,103px | — |
//! | 40쪽 종이(1,122.5px) 밖 글줄 앵커 | 2,124 / 2,923 | 0 | — |
//! | 부속서 Ⅱ 제목이 있는 쪽(0기준) | 39 (겹침) | 44 | 45 |
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/table_giant_cell_overfill.hwpx";

/// 이 문서가 조판하는 A4 세로 종이 높이(px, 96dpi). 84,189HWPUNIT.
const PAPER_HEIGHT_PX: f64 = 1122.5;

/// giant cell 의 마지막 조각이 시작하는 쪽(0 기준) — 수정 전 모든 잔여 내용이
/// 겹쳐 쏟아지던 자리.
const COLLAPSED_PAGE: u32 = 39;

/// SVG 는 글자 단위 `<text>` 로 방출된다 — 순서대로 이어 붙여 문구를 찾는다.
fn svg_text_concat(svg: &str) -> String {
    let mut out = String::new();
    for cap in svg.split("</text>") {
        if let Some(i) = cap.rfind('>') {
            out.push_str(&cap[i + 1..]);
        }
    }
    out
}

/// SVG 의 모든 `<text … y="…">` baseline 을 모은다.
fn text_baselines(svg: &str) -> Vec<f64> {
    let mut out = Vec::new();
    for cap in svg.split("<text ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let Some(ys) = head.find("y=\"") else {
            continue;
        };
        let s = ys + 3;
        if let Some(e) = head[s..].find('"') {
            if let Ok(y) = head[s..s + e].parse::<f64>() {
                out.push(y);
            }
        }
    }
    out
}

fn open_sample() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open sample")
}

#[test]
fn issue_5908_giant_cell_last_fragment_keeps_splitting() {
    let core = open_sample();
    assert_eq!(
        core.page_count(),
        47,
        "giant cell 마지막 조각이 계속 분할돼야 한다 — 0-전진 붕괴 시 42쪽"
    );
}

#[test]
fn issue_5908_collapsed_page_stays_inside_the_paper() {
    let core = open_sample();
    let svg = core
        .render_page_svg_native(COLLAPSED_PAGE)
        .expect("collapsed page svg");
    let baselines = text_baselines(&svg);
    assert!(
        !baselines.is_empty(),
        "{COLLAPSED_PAGE}쪽에 글자가 있어야 한다"
    );
    let max_y = baselines.iter().copied().fold(f64::MIN, f64::max);
    assert!(
        max_y <= PAPER_HEIGHT_PX,
        "{COLLAPSED_PAGE}쪽 최대 글자 baseline({max_y:.1})이 종이({PAPER_HEIGHT_PX}) 안이어야 한다 \
         — 결함 시 5141.6 로 4.5쪽 분량이 겹쳐 나왔다"
    );
}

#[test]
fn issue_5908_annex_two_moves_off_the_collapsed_page() {
    let core = open_sample();
    // 수정 전에는 부속서 Ⅰ 의 표 4쪽 + 부속서 Ⅱ 가 모두 39쪽 한 장에 겹쳤다.
    let collapsed = svg_text_concat(
        &core
            .render_page_svg_native(COLLAPSED_PAGE)
            .expect("collapsed page svg"),
    );
    let collapsed: String = collapsed.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        !collapsed.contains("부속서Ⅱ"),
        "부속서 Ⅱ 제목이 {COLLAPSED_PAGE}쪽에 겹쳐 있으면 안 된다"
    );

    // 한글 2024 정본은 부속서 Ⅱ 를 46쪽(0기준 45)에 둔다. rhwp 는 앞쪽에서 1쪽
    // 적으므로 0기준 44 쪽이며, 어느 쪽이든 붕괴 쪽보다 뒤여야 한다.
    let page_count = u32::try_from(core.page_count()).expect("page count fits u32");
    let annex_page = (COLLAPSED_PAGE..page_count).find(|page| {
        let text = svg_text_concat(&core.render_page_svg_native(*page).expect("page svg"));
        let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        text.contains("부속서Ⅱ")
    });
    assert_eq!(
        annex_page,
        Some(44),
        "부속서 Ⅱ 는 giant cell 이 끝나는 자기 쪽에 있어야 한다"
    );
}
