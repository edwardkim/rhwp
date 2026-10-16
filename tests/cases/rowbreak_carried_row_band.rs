//! 저장 첫 조각 프레임이 행 안에서 끝날 때, 그 행의 남은 빈 띠를 다음 쪽 첫머리로 넘기는 계약.
//!
//! 1480000-201900698 표 14(구역 4 `pi=275`, 3×5 RowBreak, `Name` 이 0~1행에 걸침)는 저장
//! 높이 5726 HU(76.3px)가 0행(47.0px) + 1행의 29.3px 이다. 1행 내용(17.1px)은 그 안에 다
//! 들지만 한/글 2020 PDF 는 1행을 닫지 않는다 — 45쪽에 1행 글자를 두고, 1행의 남은 23.0px
//! 빈 띠를 46쪽 첫머리에 그린 뒤 2행을 잇는다. 되감긴 후속 문단 `pi=276` 의 저장 원점
//! 5696 HU(75.95px)가 «띠 23.04 + 2행 45.36 + 바깥 위·아래 여백 7.55» 와 같다는 것이 그
//! 배치의 저장 근거다.
//!
//! 수정 전 rhwp 는 1행을 프레임 안에서 닫아(29.3px) 띠를 버렸고, 46쪽 이후가 23px 올라갔다.
//! 띠로 시작하는 끝 조각은 바깥 아래 여백도 예약하지 못해 그리기가 후속 문단을 3.77px
//! 위에 두었다(조판 흐름은 여백을 더했다).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5941/1480000-201900698-native-neartop-reset.hwp";
const SECTION: usize = 4;
const TABLE_PARA: u64 = 275;
/// 표 14 의 바깥 아래 여백 283 HU (96dpi).
const OUTER_MARGIN_BOTTOM_PX: f64 = 283.0 / 75.0;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("parse 1480000-201900698")
}

/// (startRow, endRow, startCut, endCut) — 쪽 `page` 에 놓인 표 14 조각.
fn table_fragment(core: &DocumentCore, page: u32) -> Option<(u64, u64, Vec<u64>, Vec<u64>)> {
    let items = core.dump_page_items_json(Some(page));
    let cut = |item: &serde_json::Value, key: &str| -> Vec<u64> {
        item.get(key)
            .and_then(|v| v.as_array())
            .map(|cut| cut.iter().filter_map(|u| u.as_u64()).collect())
            .unwrap_or_default()
    };
    items
        .as_array()?
        .first()?
        .get("columns")?
        .as_array()?
        .iter()
        .filter_map(|column| column.get("items").and_then(|items| items.as_array()))
        .flatten()
        .find(|item| {
            item.get("paraIndex").and_then(|v| v.as_u64()) == Some(TABLE_PARA)
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
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn table_14_carries_row_1_blank_band_to_page_46() {
    let core = core();

    // 45쪽(0-기준 44): 0행과 1행의 글자까지. 1행은 내용을 다 쓴 컷으로 끝난다.
    let (start, end, start_cut, end_cut) =
        table_fragment(&core, 44).expect("45쪽에 표 14 첫 조각");
    assert_eq!((start, end), (0, 2), "45쪽은 0·1행을 받는다");
    assert!(start_cut.is_empty());
    assert!(
        !end_cut.is_empty(),
        "1행은 프레임 안에서 닫히지 않고 컷으로 끝나야 한다(남은 띠를 넘긴다)"
    );

    // 46쪽(0-기준 45): 같은 컷에서 1행을 이어(빈 띠) 2행까지.
    let (start, end, start_cut, _) = table_fragment(&core, 45).expect("46쪽에 표 14 끝 조각");
    assert_eq!((start, end), (1, 3), "46쪽은 1행의 남은 띠부터 잇는다");
    assert_eq!(start_cut, end_cut, "앞 조각의 끝 컷에서 이어야 유닛 누락·중복이 없다");
}

#[test]
fn carried_band_fragment_closes_its_outer_bottom_margin_before_the_successor() {
    let core = core();
    let tree = core.build_page_render_tree(45).expect("46쪽 렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);

    let table_bottom = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Table(t)
                if t.section_index == Some(SECTION)
                    && t.para_index == Some(TABLE_PARA as usize)
                    && t.cell_context.is_none() =>
            {
                Some(node.bbox.y + node.bbox.height)
            }
            _ => None,
        })
        .expect("46쪽 표 14 조각");
    let successor_top = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextLine(line)
                if line.section_index == Some(SECTION)
                    && line.para_index == Some(TABLE_PARA as usize + 1) =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .expect("46쪽 후속 문단 pi=276");

    // 조판 흐름과 그리기가 같은 결과를 소비한다: 후속 문단은 표 아래 바깥 여백 뒤에서 시작한다.
    let gap = successor_top - table_bottom;
    assert!(
        (gap - OUTER_MARGIN_BOTTOM_PX).abs() <= 0.5,
        "후속 문단은 표 아래 + 바깥 아래 여백({OUTER_MARGIN_BOTTOM_PX:.2}px)에서 시작해야 한다: gap={gap:.2}"
    );
}
