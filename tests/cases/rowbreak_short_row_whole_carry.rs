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
