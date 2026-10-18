//! 쪽 끝에 첫 줄만 들어가는 짧은 행을 언제 나누고 언제 통째로 넘기는가.
//!
//! 1480000-201900698 표 15·18(구역 4 `pi=278`·`pi=292`, 151×6 RowBreak, 걸침 칸 없음, 셀 저장
//! 되감김 없음)에서 한/글 2020 PDF 는 쪽 끝 행을 한 번도 가르지 않는다.
//! - 표 15 75행(2줄, 첫 조각 12px)은 47쪽 18.8px 띠에 첫 줄이 들어가도 48쪽으로 통째 간다.
//! - 표 15 110행·표 18 76행(1줄, 선언 18.9px)은 띠에 글줄은 들어가도 선언 높이가 안 들어가
//!   49·53쪽으로 통째 간다. 빈 꼬리만 넘기지 않는다.
//! - 표 18 117행(3줄, 첫 조각 24px)은 54쪽으로 통째 간다.
//!
//! 수정 전 rhwp 는 문단 안 글줄 컷을 25px 최소 상단 기준에서 면제하고(75·117행), 저장 쪽 컷
//! 기록이 없는 표에서도 1줄 행의 빈 꼬리를 다음 쪽으로 넘겨(76·110행) 48·49·53·54쪽이 한 줄씩
//! 어긋났다.
//!
//! 반례: 1342000 교육과정 지도 표(구역 5 `pi=14`)의 120·127행은 위 행에서 내려온 걸침 칸이
//! 행 윗변을 가로지른다. 한/글 2020 PDF 는 두 행을 165·166쪽 끝에 첫 줄만 남기고 나눈다.
//!
//! 21761835 전직 면제 표(hwpx, `pi=4`, 78×5)에서 한/글 2020 PDF(저장소
//! `pdf/task2146/21761835_jeonjik_exemption_table-2020.pdf`, MCP 재변환본과 텍스트 동일)는
//! 41행을 4쪽에 통째로 두고, 걸침 칸 블록 53–56 의 끝 56행(내용 65px, 선언 88px)을 4쪽 끝
//! 83px 띠에서 닫은 뒤 5쪽을 57행으로 시작한다. 수정 전 rhwp 는 41행을 3·4쪽에 가르거나,
//! 41행을 고친 뒤에는 56행의 빈 띠 5px 를 5쪽 첫머리로 넘겼다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const SAMPLE_1480000: &str = "samples/issue5941/1480000-201900698-native-neartop-reset.hwp";
const SAMPLE_21761835: &str = "samples/hwpx/21761835_jeonjik_exemption_table.hwpx";
const SAMPLE_1613000: &str = "samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp";
const SAMPLE_80168: &str = "samples/80168_regulatory_analysis.hwp";
const SAMPLE_MYEONJEOK: &str = "samples/task1700/myeonjeok_wrap_10page.hwp";
const SAMPLE_70833: &str = "samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp";
const SAMPLE_1342000: &str =
    "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp";

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

/// (startRow, endRow, startCut, endCut) — 쪽 `page`(0-기준)에 놓인 표 `para` 조각.
fn table_fragment(core: &DocumentCore, page: u32, para: u64) -> (u64, u64, Vec<u64>, Vec<u64>) {
    let items = core.dump_page_items_json(Some(page));
    let cut = |item: &serde_json::Value, key: &str| -> Vec<u64> {
        item.get(key)
            .and_then(|v| v.as_array())
            .map(|cut| cut.iter().filter_map(|u| u.as_u64()).collect())
            .unwrap_or_default()
    };
    items
        .as_array()
        .and_then(|pages| pages.first())
        .and_then(|page| page.get("columns"))
        .and_then(|columns| columns.as_array())
        .into_iter()
        .flatten()
        .filter_map(|column| column.get("items").and_then(|items| items.as_array()))
        .flatten()
        .find(|item| {
            item.get("paraIndex").and_then(|v| v.as_u64()) == Some(para)
                && item.get("kind").and_then(|v| v.as_str()) == Some("partialTable")
        })
        .map(|item| {
            (
                item.get("startRow").and_then(|v| v.as_u64()).unwrap_or(0),
                item.get("endRow").and_then(|v| v.as_u64()).unwrap_or(0),
                cut(item, "startCut"),
                cut(item, "endCut"),
            )
        })
        .unwrap_or_else(|| panic!("{}쪽에 표 pi={para} 조각이 없다", page + 1))
}

/// 쪽 `page` 가 행 `row` 를 처음부터 통째로 받고, 앞쪽은 그 행을 갖지 않는다.
fn assert_whole_row_boundary(core: &DocumentCore, page: u32, para: u64, row: u64) {
    let (_, prev_end, _, prev_end_cut) = table_fragment(core, page - 1, para);
    let (start, _, start_cut, _) = table_fragment(core, page, para);
    assert_eq!(
        (prev_end, start),
        (row, row),
        "{}쪽은 {row}행 앞에서 끝나고 {}쪽이 {row}행으로 시작해야 한다",
        page,
        page + 1
    );
    assert!(
        prev_end_cut.is_empty() && start_cut.is_empty(),
        "{row}행은 나뉘지 않아야 한다: 앞 끝 컷 {prev_end_cut:?}, 시작 컷 {start_cut:?}"
    );
}

/// 쪽 `page` 끝에서 행 `row` 가 나뉘고, 다음 쪽이 같은 컷에서 잇는다.
fn assert_split_row_boundary(core: &DocumentCore, page: u32, para: u64, row: u64) {
    let (_, end, _, end_cut) = table_fragment(core, page, para);
    let (start, _, start_cut, _) = table_fragment(core, page + 1, para);
    assert_eq!(
        (end, start),
        (row + 1, row),
        "{}쪽 끝과 {}쪽 첫머리가 모두 {row}행이어야 한다",
        page + 1,
        page + 2
    );
    assert!(
        end_cut.iter().any(|units| *units > 0),
        "{row}행은 {}쪽에 첫 줄을 남겨야 한다: 끝 컷 {end_cut:?}",
        page + 1
    );
    assert_eq!(
        start_cut, end_cut,
        "앞 조각의 끝 컷에서 이어야 유닛 누락·중복이 없다"
    );
}

#[test]
fn table_15_moves_short_bottom_rows_whole() {
    let core = core(SAMPLE_1480000);
    assert_eq!(core.page_count(), 205, "한/글 2020 PDF 와 같은 205쪽");
    // 48쪽(0-기준 47)은 75행, 49쪽(48)은 110행으로 시작한다.
    assert_whole_row_boundary(&core, 47, 278, 75);
    assert_whole_row_boundary(&core, 48, 278, 110);
}

#[test]
fn table_18_moves_short_bottom_rows_whole() {
    let core = core(SAMPLE_1480000);
    // 53쪽(0-기준 52)은 76행, 54쪽(53)은 117행으로 시작한다.
    assert_whole_row_boundary(&core, 52, 292, 76);
    assert_whole_row_boundary(&core, 53, 292, 117);
}

#[test]
fn rows_inside_a_crossing_rowspan_still_split_after_one_line() {
    let core = core(SAMPLE_1342000);
    // 165쪽(0-기준 164) 끝의 120행, 166쪽(165) 끝의 127행.
    assert_split_row_boundary(&core, 164, 14, 120);
    assert_split_row_boundary(&core, 165, 14, 127);
}

#[test]
fn rowspan_block_end_row_closes_at_page_bottom() {
    let core = core(SAMPLE_21761835);
    assert_eq!(core.page_count(), 6, "한/글 2020 PDF 와 같은 6쪽");
    // 4쪽(0-기준 3)은 41행, 5쪽(4)은 57행으로 시작한다.
    assert_whole_row_boundary(&core, 3, 4, 41);
    assert_whole_row_boundary(&core, 4, 4, 57);
}

/// 1613000 용어 비교 표(`pi=1143`, 32×5): 17·28행은 rhwp 에서 통째로 들어가지만 그 쪽의 마지막
/// 행이고, 셀 저장 줄이 뒷줄을 새 원점(vpos 0)에서 다시 시작한다. 한/글 2020 PDF 는 17행을
/// `Performance`/`Criteria`, 28행을 `(역량 단위에서`/`사용)` 에서 126·127쪽 끝에 가른다.
#[test]
fn stored_restart_cuts_the_last_row_of_a_page() {
    let core = core(SAMPLE_1613000);
    assert_eq!(core.page_count(), 204, "한/글 2020 PDF 와 같은 204쪽");
    assert_split_row_boundary(&core, 125, 1143, 17);
    assert_split_row_boundary(&core, 126, 1143, 28);
}

/// 1613000 구역 2 첫 표(`pi=1`, 29×3)의 개체 높이 58227HU 는 행 0–26 선언 합과 정확히 같다(행 경계).
/// 그 경계를 닫는 26행과 28행만 두 줄로 선언보다 자랐다. 한/글 2020 PDF 는 셀 높이를 지키며
/// 26행 첫 줄(`4.1.1 시설교육의`)까지를 185쪽에 두고 둘째 줄(`에 적용되는 규정을 설명`)을 186쪽에서 잇는다.
/// 186쪽 표 조각 뒤 빈 문단 `pi=2` 는 저장 줄(vpos 7069)대로 표 바로 아래에 놓이므로 `pi=4` 표는 186쪽에
/// 21행(`2.3 팀 행동`)까지 담고, 187쪽 끝에서 끝난 뒤 188쪽은 `과목 3` 제목과 `pi=7` 표 첫 조각으로 시작한다.
/// 수정 전 rhwp 는 행을 0.906 배로 줄여 `pi=1` 을 185쪽에 통째로 두었고(186쪽 `pi=4` 0..25행),
/// 186–191쪽 내용이 한/글과 어긋났다. 줄이기만 막으면 표 조각 뒤 문단이 호스트 아래 간격만큼 밀려
/// 188쪽에 제목만 남고 `pi=7` 이 189쪽으로 넘어간다.
#[test]
fn grown_boundary_row_continues_and_successor_keeps_its_stored_line() {
    let core = core(SAMPLE_1613000);
    assert_split_row_boundary(&core, 184, 1, 26);
    assert_whole_row_boundary(&core, 186, 4, 21);
    let (start, _, start_cut, _) = table_fragment(&core, 187, 7);
    assert!(
        start == 0 && start_cut.is_empty(),
        "188쪽은 제목 뒤에 pi=7 표를 0행부터 시작해야 한다: 시작 {start}행 {start_cut:?}"
    );
}

/// 쪽 `page`(0-기준) 렌더 트리에서 표 칸 (row, col) 의 마지막 상자 (윗변, 아랫변).
fn cell_box(core: &DocumentCore, page: u32, row: u16, col: u16) -> (f64, f64) {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell) if cell.row == row && cell.col == col => {
                Some((node.bbox.y, node.bbox.y + node.bbox.height))
            }
            _ => None,
        })
        .next_back()
        .unwrap_or_else(|| panic!("{}쪽 셀 ({row},{col})", page + 1))
}

/// 셀 저장 줄의 새 원점 재시작(vpos 0)에서 자른 쪽 끝 행은 남은 쪽 공간을 채우지 않는다.
/// 한/글 2020 PDF 는 그 행을 max(행 선언 높이, 자른 줄까지의 내용 높이) 상자로 닫고 아래 본문을 비운다.
/// 기대값은 저장 메트릭으로 정한다(셀 위아래 여백 141HU 씩).
/// - 1613000 127쪽 행 28: 둘째 문단 첫 줄(vpos 5760, lh 1200)까지 = (6960 + 282) HU = 96.56px (선언 17.1px).
/// - 1613000 126쪽 행 17: 한 줄 lh 1200 = (1200 + 282) HU = 19.76px (선언 17.1px).
/// - 1480000 43쪽 행 118: 한 줄 lh 900 = 15.76px < 선언 1417 HU = 18.89px.
/// - myeonjeok 1쪽 행 29: 한 줄 lh 800 = 14.43px < 선언 2362 HU = 31.49px. 끝 줄간격(480HU)을
///   다시 접지 않는다(한/글 2020 MCP 변환본: 표 아래 ≈ 1080.5px).
///
/// 수정 전 rhwp 는 행을 남은 쪽 공간(136·57·39·34.9px)까지 늘려 127쪽 가운데 정렬 칸 글자가 한 줄 아래로 내려갔다.
#[test]
fn stored_restart_end_row_closes_at_its_declared_or_cut_content_box() {
    let px = |hu: f64| hu / 75.0;
    let cases: [(&str, u32, u16, f64); 4] = [
        (SAMPLE_1613000, 126, 28, px(6960.0 + 282.0)),
        (SAMPLE_1613000, 125, 17, px(1200.0 + 282.0)),
        (SAMPLE_1480000, 42, 118, px(1417.0)),
        (SAMPLE_MYEONJEOK, 0, 29, px(2362.0)),
    ];
    for (sample, page, row, expected) in cases {
        let core = core(sample);
        let (top, bottom) = cell_box(&core, page, row, 0);
        assert!(
            (bottom - top - expected).abs() <= 0.5,
            "{sample} {}쪽 행 {row} 끝 상자는 {expected:.2}px 여야 한다: {:.2}",
            page + 1,
            bottom - top
        );
    }
}

/// 이어받은 행도 셀 저장 줄의 다음 새 원점 재시작에서 닫는다. 1613000 표 `pi=1133`(머리행 + 4쪽에
/// 걸친 2칸 행 1)의 저장 줄: 왼칸은 9번째 문단 넷째 줄(유닛 40), 오른칸은 16번째 문단 넷째 줄(유닛 37)과
/// 32번째 문단 첫 줄(유닛 70, 앞 문단 끝 vpos 64940 → 0)에서 vpos 가 0 으로 돌아간다.
/// 한/글 2020 PDF 는 123쪽을 [40,37], 124쪽을 [51(왼칸 끝),70]에서 닫고 오른칸 32번째 문단
/// 「다. 시정훈련…」 네 줄을 125쪽에 통째로 둔다. 123쪽 행 상자는 오른칸 재시작 앞 줄 끝
/// (vpos 66020 + lh 1200) + 셀 위아래 여백 282 = 67502 HU = 900.03px(PDF 표 윗변→아랫변 899.96)다.
/// 수정 전 rhwp 는 남은 쪽 공간을 채워 [41,37]·[51,72]에서 잘라 124쪽 표가 51px 더 내려갔다.
#[test]
fn continued_row_closes_at_its_next_stored_restart() {
    let core = core(SAMPLE_1613000);
    let (_, end, _, end_cut) = table_fragment(&core, 122, 1133);
    assert_eq!((end, end_cut.clone()), (2, vec![40, 37]), "123쪽 끝 컷");
    let (start, end, start_cut, next_cut) = table_fragment(&core, 123, 1133);
    assert_eq!(
        (start, start_cut),
        (1, end_cut),
        "124쪽은 123쪽 컷에서 잇는다"
    );
    assert_eq!((end, next_cut.clone()), (2, vec![51, 70]), "124쪽 끝 컷");
    let (_, _, start_cut, _) = table_fragment(&core, 124, 1133);
    assert_eq!(start_cut, next_cut, "125쪽은 124쪽 컷에서 잇는다");
    let (top, bottom) = cell_box(&core, 122, 1, 1);
    let expected = (66020.0 + 1200.0 + 282.0) / 75.0;
    assert!(
        (bottom - top - expected).abs() <= 0.5,
        "123쪽 행 1 상자는 {expected:.2}px 여야 한다: {:.2}",
        bottom - top
    );
}

/// 빈 호스트 표의 끝 조각 뒤 문단은, 그 저장 줄 윗변이 표·캡션 상자 아래와 같으면 그 줄에 놓인다.
/// 1613000 125쪽: 표 `pi=1133` 끝 조각과 아래 캡션이 끝난 자리(본문 위 156.08px)가 다음 빈 문단
/// `pi=1134` 의 vpos 11706 과 같고, 한/글 2020 PDF 는 `pi=1135` 「ㅇ 관제시설…」 첫 줄을 저장 vpos 15006
/// 그대로 본문 위(위 여백 2835 + 머리말 2835 HU) + 15006 HU = 275.68px 에 둔다.
/// 수정 전 rhwp 는 호스트 아래 간격(500HU)과 빈 문단 위 간격(700HU)을 더해 16px 아래에 두었다.
/// 같은 규칙으로 task2070 72–110쪽이 한/글 2020 변환본과 맞고 전체 쪽수도 315쪽으로 같아진다.
#[test]
fn empty_host_terminal_successor_keeps_its_stored_line() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    let core = core(SAMPLE_1613000);
    let tree = core.build_page_render_tree(124).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let top = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextLine(line)
                if line.para_index == Some(1135) && line.line_index == Some(0) =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .expect("125쪽 pi=1135 첫 줄");
    let expected = (2835.0 + 2835.0 + 15006.0) / 75.0;
    assert!(
        (top - expected).abs() <= 0.5,
        "pi=1135 첫 줄은 저장 줄 {expected:.2}px 에 놓여야 한다: {top:.2}"
    );
}

/// 이어진 표의 끝 조각 아래 캡션은 통째 표와 같은 간격으로 놓인다. 1613000 `[표 12]`(128쪽, `pi=1143`
/// 끝 조각)와 `[표 11]`(121쪽, 통째)의 표 아래 → 캡션 줄 윗변 간격은 한/글 2020 PDF 에서 모두 12px 다.
/// 수정 전 rhwp 는 끝 조각에만 호스트 줄간격(700HU = 9.33px)을 캡션 위에 더 넣었다.
#[test]
fn continued_table_bottom_caption_keeps_the_whole_table_gap() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    let core = core(SAMPLE_1613000);
    let gap = |page: u32| {
        let tree = core.build_page_render_tree(page).expect("렌더 트리");
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        // 아래 캡션 줄 윗변과 그 바로 위에서 끝나는 표 칸 아랫변의 거리.
        let caption_top = nodes
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextLine(line) if line.caption_owner.is_some() => Some(node.bbox.y),
                _ => None,
            })
            .fold(f64::MIN, f64::max);
        assert!(caption_top > f64::MIN, "{}쪽 캡션 줄", page + 1);
        let table_bottom = nodes
            .iter()
            .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
            .map(|node| node.bbox.y + node.bbox.height)
            .filter(|bottom| *bottom <= caption_top + 0.5)
            .fold(f64::MIN, f64::max);
        caption_top - table_bottom
    };
    let (whole, continued) = (gap(120), gap(127));
    assert!(
        (continued - whole).abs() <= 0.5,
        "끝 조각 캡션 간격 {continued:.2}px 은 통째 표 {whole:.2}px 와 같아야 한다"
    );
}

/// 걸침 칸 안의 쪽 끝 행은 선언 높이가 남은 공간을 넘쳐도, 내용이 비끝 조각 상자 아래(본문 아래 −
/// 바깥 아래 여백 − 100HU) 안에 들면 그 쪽에 남고 상자는 그 선에서 닫힌다. 한/글 2020 PDF(1613000 구역 3
/// 가로 표): 195쪽이 행 43(`4.4`, 선언 20.87px)까지 담고 196쪽이 행 44, 197쪽이 행 71 로 시작하며,
/// 75×3 표는 201쪽이 행 43, 202쪽이 행 65 로 시작한다. 닫는 선은 저장 용지·여백으로 정한다:
/// (59528 − 아래 2835 − 꼬리말 2835 − 바깥 아래 283 − 100) HU = 712.99px (PDF 712.91). 행 43 상자는
/// 쪽 끝 띠를 포함해 그 선에서 끝난다.
/// 수정 전 rhwp 는 선언 높이로 판정해 0.5px 차로 행 43 을 넘기고 196–202쪽이 한 행씩 밀렸다.
#[test]
fn crossing_span_row_closes_at_the_fragment_bottom_line() {
    let core = core(SAMPLE_1613000);
    assert_whole_row_boundary(&core, 195, 1, 44);
    assert_whole_row_boundary(&core, 196, 1, 71);
    assert_whole_row_boundary(&core, 200, 3, 43);
    assert_whole_row_boundary(&core, 201, 3, 65);
    let line = (59528.0 - 2835.0 - 2835.0 - 283.0 - 100.0) / 75.0;
    let (top, bottom) = cell_box(&core, 194, 43, 2);
    assert!(
        line - top >= 17.09 && (bottom - line).abs() <= 0.5,
        "195쪽 행 43 은 내용(17.09px)이 들고 상자가 닫는 선 {line:.2}px 에서 끝나야 한다: {top:.2}..{bottom:.2}"
    );
}

/// 걸침 칸 안 쪽 끝 행의 이 쪽 상자 = max(컷까지 내용, 행 선언, 걸침 칸마다 «선언 − 이 칸의 앞 행 상자»),
/// 비끝 조각 닫는 선을 넘지 않는다. 한/글 2020 PDF(1613000 구역 1, 가로 7열 표 `pi=2`·`pi=5`):
/// - 139쪽 행 49(1줄 컷): 걸침 칸 «7. ACAS»(r48 rs=4, 5128HU) − 행 48(2줄 2882HU) = 29.95px (PDF 29.91).
/// - 144쪽 행 139(통째, 2줄): 걸침 몫이 행 높이 이하라 띠 없이 2882HU = 38.43px (PDF 38.40).
/// - 154쪽 행 51(1줄 컷): 걸침 몫이 커서 닫는 선 (59528 − 4252 − 바깥 아래 283 − 100) HU = 731.91px 까지
///   (PDF 731.79).
///
/// 수정 전 rhwp 는 144쪽 행 139 를 쪽 끝까지(53.78px), 154쪽 행 51 을 30.0px 로 그렸다.
#[test]
fn crossing_span_end_row_takes_the_larger_span_share_up_to_the_closing_line() {
    let core = core(SAMPLE_1613000);
    let px = |hu: f64| hu / 75.0;
    let row_box = |page: u32, row: u16| {
        let (top, bottom) = cell_box(&core, page, row, 3);
        bottom - top
    };
    let cases = [(138, 49, px(5128.0 - 2882.0)), (143, 139, px(2882.0))];
    for (page, row, expected) in cases {
        let got = row_box(page, row);
        assert!(
            (got - expected).abs() <= 0.5,
            "{}쪽 행 {row} 끝 상자는 {expected:.2}px 여야 한다: {got:.2}",
            page + 1
        );
    }
    let line = px(59528.0 - 4252.0 - 283.0 - 100.0);
    let (_, bottom) = cell_box(&core, 153, 51, 3);
    assert!(
        (bottom - line).abs() <= 0.5,
        "154쪽 행 51 은 닫는 선 {line:.2}px 에서 끝나야 한다: {bottom:.2}"
    );
}

/// 반례: 80168 표(`pi=1056`) 행 23(머리 행, 내용 20.67px)은 남은 22.76px 에 들지만 바깥 아래 여백
/// (141HU)까지 뺀 닫는 선 안에는 들지 않는다. 한/글 2024 PDF 는 이 행을 123쪽 첫머리로 넘기고 122쪽
/// 행 22 를 쪽 끝까지 늘린다.
#[test]
fn crossing_span_row_beyond_the_closing_line_moves_whole() {
    let core = core(SAMPLE_80168);
    assert_whole_row_boundary(&core, 122, 1056, 23);
}

/// 1613000 191쪽: 새 쪽 첫머리에 통째로 든 18×3 표(구역 2 `pi=9`, 문단 기준 자리차지, 세로 오프셋
/// 86.5px)의 상자는 바깥 아래 여백까지 631.9px 에서 닫히고, 뒤 빈 문단 `pi=10` 줄은 그 아래 자기 저장
/// 줄(vpos 47394)에 놓인다. 그래서 `pi=11` 제목 뒤 46×3 표 첫 조각은 한/글 2020 PDF 처럼 `1.2 교육 계획`
/// (행 4)까지 담고 192쪽이 행 5(`1.2.1`)로 시작한다. 수정 전 rhwp 는 이 표의 흐름에 세로 오프셋을
/// 넣지 않아 뒤 문단이 14.5px 위에서 시작했고, 첫 조각이 행 5 까지 담아 192–193쪽이 한 행씩 밀렸다.
#[test]
fn whole_table_on_a_fresh_page_leaves_its_successor_on_the_stored_line() {
    let core = core(SAMPLE_1613000);
    assert_whole_row_boundary(&core, 191, 11, 5);
}

/// 1480000 표 12(`pi=263`) 118행은 둘째 줄이 vpos 0 에서 다시 시작하는 저장 쪽 컷이다. 한/글 2020
/// PDF 는 43쪽 끝에 첫 줄을 두고 44쪽 첫머리에서 `L40-like isoform X1` 을 잇는다. 표 뒤 빈 문단
/// `pi=264` 는 표 시작 쪽의 옛 vpos(72003)를 갖지만, 되감긴 다음 문단 `pi=265` 의 저장 위치가 그
/// 빈 줄 한 줄을 담을 자리를 남기므로 한/글은 빈 줄을 그린다. 그 뒤 `pi=265` 는 빈 줄의 저장 전진
/// (lh 1100 + ls 880)만큼 아래에서 시작하며 trailing 줄간격을 한 번 더 받지 않는다.
#[test]
fn page_44_keeps_the_empty_line_after_the_stored_row_cut() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    let core = core(SAMPLE_1480000);
    assert_split_row_boundary(&core, 42, 263, 118);

    let tree = core.build_page_render_tree(43).expect("44쪽 렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let line_top = |para: usize| {
        nodes.iter().find_map(|node| match &node.node_type {
            RenderNodeType::TextLine(line)
                if line.section_index == Some(4) && line.para_index == Some(para) =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
    };
    let empty_top = line_top(264).expect("44쪽에 표 뒤 빈 문단 pi=264 의 줄이 있어야 한다");
    let next_top = line_top(265).expect("44쪽 문단 pi=265");
    let stored_advance = (1100.0 + 880.0) / 75.0;
    assert!(
        (next_top - empty_top - stored_advance).abs() <= 0.5,
        "pi=265 는 빈 줄의 저장 전진({stored_advance:.2}px)만큼 아래에서 시작해야 한다: {:.2}",
        next_top - empty_top
    );
}

/// 21761835 2쪽: 행 10~11 칸(c1~c4 rs=2, 7940HU)과 행 11~17 칸(c0 rs=7, 50353HU)이 겹친다.
/// 한/글 2020 PDF 는 행 10~11 블록을 선언 높이(105.9px)로 그리고, 긴 칸의 남는 높이를 걸침의
/// 마지막 행 17 에 준다. 1쪽에서 줄어든 칸 (6,0) rs=5 의 남은 몫은 2쪽 행 10 이 받는다.
/// 수정 전 rhwp 는 행 11 에 긴 칸의 남는 높이를 몰아 블록이 139.7px 였다.
#[test]
fn rowspan_extra_goes_to_the_last_row_across_the_page() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    let core = core(SAMPLE_21761835);
    let tree = core.build_page_render_tree(1).expect("2쪽 렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let cell = |row: u16, col: u16| {
        nodes
            .iter()
            .find_map(|node| match &node.node_type {
                RenderNodeType::TableCell(cell)
                    if cell.row == row && cell.col == col && cell.model_cell_index.is_some() =>
                {
                    Some((node.bbox.y, node.bbox.y + node.bbox.height))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("2쪽 셀 ({row},{col})"))
    };
    let block = cell(10, 1);
    let declared_block = 7940.0 / 75.0;
    assert!(
        (block.1 - block.0 - declared_block).abs() <= 0.5,
        "행 10~11 블록은 선언 {declared_block:.2}px 여야 한다: {:.2}",
        block.1 - block.0
    );
    let row12 = cell(12, 1);
    assert!(
        (row12.0 - block.1).abs() <= 0.5,
        "행 12 는 블록 바로 아래에서 시작한다"
    );
    let row17 = cell(17, 1);
    assert!(
        row17.1 - row17.0 > 7002.0 / 75.0 + 30.0,
        "행 17 은 긴 칸의 남는 높이를 받아 선언 93.4px 보다 크다: {:.2}",
        row17.1 - row17.0
    );
}

/// 21761835 2·3쪽: 칸 (20,0) rs=5(18898HU, 251.97px)가 쪽 끝 행 20 을 지나 행 24 까지 간다.
/// 한/글 2020 PDF 는 행 20 첫 조각을 본문 끝까지(59px) 그리고, 3쪽 행 20 조각에 남은 빈 꼬리
/// 한 줄을 위아래 패딩과 함께(17px) 둔다. 걸침 칸이 덮는 행 조각 합(59+17+56+51+25+43 ≈ 251px)이 선언 높이를 지킨다.
/// 수정 전 rhwp 는 2쪽 끝에 7.8px 빈 띠를 남기고(행 20 조각 51px), 3쪽 행 20 을 13.5px 로 그려 합이 248.4px 였다.
#[test]
fn crossing_rowspan_owns_the_page_end_band() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    let core = core(SAMPLE_21761835);
    let cell = |page: u32, row: u16, col: u16| {
        let tree = core.build_page_render_tree(page).expect("렌더 트리");
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        nodes
            .iter()
            .find_map(|node| match &node.node_type {
                RenderNodeType::TableCell(cell)
                    if cell.row == row && cell.col == col && cell.model_cell_index.is_some() =>
                {
                    Some((node.bbox.y, node.bbox.y + node.bbox.height))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{}쪽 셀 ({row},{col})", page + 1))
    };
    let first = cell(1, 20, 1);
    // 저장 줄 3개(1272HU 간격, 마지막 줄 1000HU)와 위아래 패딩 141HU.
    let content = (2.0 * 1272.0 + 1000.0 + 141.0 + 141.0) / 75.0;
    assert!(
        first.1 - first.0 > content + 4.0,
        "2쪽 행 20 조각은 내용({content:.2}px) 아래 쪽 끝 띠까지 차지한다: {:.2}",
        first.1 - first.0
    );
    let resumed = cell(2, 20, 1);
    let line_with_padding = (1000.0 + 141.0 + 141.0) / 75.0;
    assert!(
        (resumed.1 - resumed.0 - line_with_padding).abs() <= 0.5,
        "3쪽 행 20 조각은 빈 줄 하나와 위아래 패딩({line_with_padding:.2}px)이다: {:.2}",
        resumed.1 - resumed.0
    );
    // 걸침 칸 (20,0) 의 총높이는 쪽을 넘어서도 보존된다: 2쪽 행 20 조각 + 3쪽 행 20~24.
    let span: f64 = first.1 - first.0
        + (20..=24)
            .map(|row| {
                let (top, bottom) = cell(2, row, 1);
                bottom - top
            })
            .sum::<f64>();
    let declared_span = 18898.0 / 75.0;
    assert!(
        (span - declared_span).abs() <= 1.5,
        "걸침 칸 (20,0) 이 덮는 행 조각 합은 선언 {declared_span:.2}px 다: {span:.2}"
    );
}

/// 반례: 70833 2쪽 표(`pi=13`, 20×13)는 쪽 끝 행 6 을 지나 이어지는 걸침 칸이 있고 끝 행이 저장 쪽
/// 재시작에서 끊기지만, 걸침 칸과 나뉜 행 모두 선언에 남는 몫이 없다. 한/글 2020 PDF 는 표를 973px 에서
/// 끝내고 본문 끝(1046.9px)까지의 띠를 비워 둔다. 몫 없이 띠를 주면 표가 1036px 까지 늘어난다.
#[test]
fn crossing_rowspan_without_declared_slack_leaves_the_band_empty() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn bottom(node: &RenderNode) -> f64 {
        let own = match &node.node_type {
            RenderNodeType::TableCell(cell) if cell.model_cell_index.is_some() => {
                node.bbox.y + node.bbox.height
            }
            _ => f64::MIN,
        };
        node.children.iter().map(bottom).fold(own, f64::max)
    }
    let core = core(SAMPLE_70833);
    let tree = core.build_page_render_tree(1).expect("2쪽 렌더 트리");
    let body = core.dump_page_items_json(Some(1));
    let area = &body[0]["bodyArea"];
    let body_bottom = area["y"].as_f64().unwrap() + area["height"].as_f64().unwrap();
    let table_bottom = bottom(&tree.root);
    assert!(
        body_bottom - table_bottom > 30.0,
        "남는 몫이 없으면 쪽 끝 띠는 비어 남는다: 표 끝 {table_bottom:.2}, 본문 끝 {body_bottom:.2}"
    );
}

fn page_nodes(core: &DocumentCore, page: u32) -> rhwp::renderer::render_tree::PageRenderTree {
    core.build_page_render_tree(page).expect("렌더 트리")
}

fn walk_nodes<'a>(
    node: &'a rhwp::renderer::render_tree::RenderNode,
    out: &mut Vec<&'a rhwp::renderer::render_tree::RenderNode>,
) {
    out.push(node);
    for child in &node.children {
        walk_nodes(child, out);
    }
}

/// 쪽 전체 글자를 공백 없이 잇는다(글자 조각 경계와 무관하게 내용 소속을 본다).
fn page_text(core: &DocumentCore, page: u32) -> String {
    use rhwp::renderer::render_tree::RenderNodeType;
    let tree = page_nodes(core, page);
    let mut nodes = Vec::new();
    walk_nodes(&tree.root, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>()
        .split_whitespace()
        .collect()
}

/// 1×1 바깥 상자(1613000 표 `pi=585`) 안 중첩 표가 쪽을 넘으면 상자도 갈린다.
/// 셀의 마지막 호스트(문단 4, 2×1 표) 위 저장 줄 간격은 앞 문단 22×2 표가 채운 띠다 — 절대배치 캡의
/// 증거가 아니다. 그 안 11×4 「실기교육」 표의 개체 높이 6006HU 는 앞 세 행 상자(3042 + 1482 + 1482)의
/// 합과 정확히 같고, 한/글 2020 PDF 는 58쪽을 머리행·1·2행에서 닫고 59쪽을 3행 「항공기 상승비행」
/// 부터 잇는다(셀 유닛 [32]). 58쪽 상자 윗변은 본문 위(5670HU) + 바깥 위 여백 283HU = 79.37px
/// (PDF 윗변 79.0)다. 수정 전 rhwp 는 상자를 선언 높이 920.5px 로 재어 58쪽에 통째로 두고 3–10행과
/// 캡션을 잘라 버렸다.
#[test]
fn nested_table_page_frame_splits_its_single_cell_box() {
    use rhwp::renderer::render_tree::RenderNodeType;
    let core = core(SAMPLE_1613000);
    let (_, _, _, end_cut) = table_fragment(&core, 57, 585);
    assert_eq!(end_cut, vec![32], "58쪽 끝 컷");
    let (_, _, start_cut, _) = table_fragment(&core, 58, 585);
    assert_eq!(start_cut, end_cut, "59쪽은 58쪽 컷에서 잇는다");

    let p58 = page_text(&core, 57);
    let p59 = page_text(&core, 58);
    assert!(
        p58.contains("항공기이륙") && !p58.contains("상승비행"),
        "58쪽은 2행에서 닫힌다"
    );
    assert!(
        p59.contains("상승비행") && p59.contains("170") && !p59.contains("항공기이륙"),
        "59쪽은 3행부터 합계 행까지 잇는다"
    );

    let tree = page_nodes(&core, 57);
    let mut nodes = Vec::new();
    walk_nodes(&tree.root, &mut nodes);
    let top = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Table(table)
                if table.para_index == Some(585) && table.cell_context.is_none() =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .expect("58쪽 표 pi=585");
    let expected = (5670.0 + 283.0) / 75.0;
    assert!(
        (top - expected).abs() <= 0.5,
        "58쪽 상자 윗변은 바깥 위 여백을 한 번만 연 {expected:.2}px 여야 한다: {top:.2}"
    );
}

/// 이어진 조각 아래 문단의 저장 줄이 상자 아래 여백과 겹쳐도 그 줄에 놓인다. 1613000 59쪽 표 `pi=585`
/// 끝 조각·캡션 아래(본문 위 226.93px)보다 다음 빈 문단 `pi=586` 의 저장 줄(vpos 17444 = 232.59px)이
/// 아래에 있고, 한/글 2020 PDF 는 `pi=587` 「· 그 밖에…」 첫 줄을 본문 위(5670HU) + vpos 20884 HU =
/// 354.05px(글자 윗변 353.84)에 둔다. 수정 전 rhwp 는 상자 아래에 호스트 아래 간격을 더해 9px 아래에 두었다.
#[test]
fn continuation_successor_keeps_its_stored_line_below_the_box() {
    use rhwp::renderer::render_tree::RenderNodeType;
    let core = core(SAMPLE_1613000);
    let tree = page_nodes(&core, 58);
    let mut nodes = Vec::new();
    walk_nodes(&tree.root, &mut nodes);
    let top = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextLine(line)
                if line.para_index == Some(587) && line.line_index == Some(0) =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .expect("59쪽 pi=587 첫 줄");
    let expected = (5670.0 + 20884.0) / 75.0;
    assert!(
        (top - expected).abs() <= 0.5,
        "pi=587 첫 줄은 저장 줄 {expected:.2}px 에 놓여야 한다: {top:.2}"
    );
}

/// 칸 안 어울림 중첩 표가 문단 기준 왼쪽·가로 오프셋 0 을 저장하면 칸 왼쪽 안쪽에 놓인다.
/// 1613000 58쪽 22×2 「학과교육」 표(셀 폭의 0.96)의 왼끝은 바깥 상자 칸 왼끝 + 칸 안 여백 510HU +
/// 표 바깥 왼 여백 140HU = 8.67px 안쪽이다(한/글 2020 PDF 괘선 92 → 101px). 수정 전 rhwp 는 좁은 중첩
/// 표를 칸 가운데에 두어 10px 오른쪽에 그렸다.
#[test]
fn square_nested_table_with_zero_offset_keeps_its_left_anchor() {
    use rhwp::renderer::render_tree::RenderNodeType;
    let core = core(SAMPLE_1613000);
    let tree = page_nodes(&core, 57);
    let mut nodes = Vec::new();
    walk_nodes(&tree.root, &mut nodes);
    let cell_x = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Table(table)
                if table.para_index == Some(585) && table.cell_context.is_none() =>
            {
                Some(node.bbox.x)
            }
            _ => None,
        })
        .expect("58쪽 표 pi=585");
    let nested_x = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Table(table)
                if table.row_count == 22
                    && table.col_count == 2
                    && table.cell_context.is_some() =>
            {
                Some(node.bbox.x)
            }
            _ => None,
        })
        .expect("58쪽 22×2 중첩 표");
    let expected = (510.0 + 140.0) / 75.0;
    assert!(
        (nested_x - cell_x - expected).abs() <= 0.5,
        "22×2 표 왼끝은 칸 왼끝에서 {expected:.2}px 안쪽이어야 한다: {:.2}",
        nested_x - cell_x
    );
}

/// 쪽 번호 문단 소속: `page`(0-기준)에 놓인 본문 문단 인덱스 목록.
fn page_paragraphs(core: &DocumentCore, page: u32) -> Vec<u64> {
    let items = core.dump_page_items_json(Some(page));
    items
        .as_array()
        .and_then(|pages| pages.first())
        .and_then(|page| page.get("columns"))
        .and_then(|columns| columns.as_array())
        .into_iter()
        .flatten()
        .filter_map(|column| column.get("items").and_then(|items| items.as_array()))
        .flatten()
        .filter_map(|item| item.get("paraIndex").and_then(|v| v.as_u64()))
        .collect()
}

/// 빈 꼬리말 밴드가 각주를 모두 받는 쪽에서 각주 보수 여백(3000HU)은 각주 자신의 높이를 넘어 본문을
/// 깎지 않는다. 1613000 84쪽은 각주 한 줄(31.4px, 꼬리말 밴드 37.8px 에 들어감)을 갖고, 빈 문단 `pi=817`
/// 의 저장 줄(vpos 68800 + lh 1400 = 936px)은 각주 구분선(PDF·rhwp 모두 본문 위 950.4px)보다 위다.
/// 한/글 2020 PDF 는 85쪽을 `pi=818` 「사. 인천국제공항 계류장 관제소」로 시작하고 그 첫 줄을 본문 위 +
/// vpos 700HU = 84.93px 에 둔다. 수정 전 rhwp 는 보수 여백 40px 로 본문을 931.4px 에서 끊어 `pi=817` 을
/// 85쪽 첫머리로 넘기고 쪽 전체를 37px 내렸다.
#[test]
fn footnote_margin_does_not_cut_body_beyond_the_footnote_height() {
    use rhwp::renderer::render_tree::RenderNodeType;
    let core = core(SAMPLE_1613000);
    let p84 = page_paragraphs(&core, 83);
    let p85 = page_paragraphs(&core, 84);
    assert!(
        p84.contains(&817),
        "84쪽이 빈 문단 pi=817 을 가진다: {p84:?}"
    );
    assert_eq!(
        p85.first(),
        Some(&818),
        "85쪽은 pi=818 로 시작한다: {p85:?}"
    );

    let tree = page_nodes(&core, 84);
    let mut nodes = Vec::new();
    walk_nodes(&tree.root, &mut nodes);
    let top = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextLine(line)
                if line.para_index == Some(818) && line.line_index == Some(0) =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .expect("85쪽 pi=818 첫 줄");
    let expected = (5670.0 + 700.0) / 75.0;
    assert!(
        (top - expected).abs() <= 0.5,
        "pi=818 첫 줄은 저장 줄 {expected:.2}px 에 놓여야 한다: {top:.2}"
    );
}
