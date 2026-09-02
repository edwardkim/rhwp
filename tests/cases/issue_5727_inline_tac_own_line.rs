//! [Issue #5727] 자리차지(글자처럼 취급) 그림의 가로 전진이 다음 줄로 누출된다.
//!
//! 저장 lineseg 가 인라인 개체에 자기 줄을 배정하면(제어문자만 담은 줄) 그 빈
//! composed 줄과 다음 줄의 `char_start` 가 같은 텍스트 인덱스로 붕괴한다
//! (제어문자는 `text` 에 없고 `char_offsets` 갭으로만 남는다). 종전에는 다음 줄이
//! 그 TAC 를 다시 집어, 개체가 다음 줄로 끌려 내려가고 그 줄 텍스트가 개체 폭만큼
//! 오른쪽에서 시작했다.
//!
//! 156732636 실측 (한글 2022): `고용`(줄0) / 로고 그림(줄1, 자기 줄) / `노동부`(줄2,
//! 저장 horzpos=0). 한글 `노동부` x=158.2 — 종전 rhwp 는 251.6(+172=로고 폭).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5727/156732636_inline_logo_cell.hwp";

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn issue_5727_next_line_after_inline_picture_starts_at_stored_horzpos() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);

    // 1쪽 머리 표의 로고 칸: '고용' 줄 아래 '노동부' 줄.
    let goyong = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text == "고용" => Some(n.bbox),
            _ => None,
        })
        .expect("'고용' run");
    let nodongbu = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text == "노동부" => Some(n.bbox),
            _ => None,
        })
        .expect("'노동부' run");

    // 저장 horzpos=0 + Center 정렬 — 로고 폭(170.4px)이 누출되면 x 가 250px 대로
    // 밀린다. 한글 실측 158.2px, 칸 안(±20px 여유)이어야 한다.
    assert!(
        (nodongbu.x - 158.2).abs() < 20.0,
        "'노동부' 는 저장 줄 시작(한글 158.2px)에서 시작해야 한다: x={:.1} (로고 폭 누출이면 +170.4)",
        nodongbu.x
    );
    // 세로 관계는 저장 vpos 그대로 (줄0 → 줄2 = +48.3px).
    assert!(
        (nodongbu.y - goyong.y - 48.3).abs() < 2.0,
        "세로 간격은 저장 vpos 를 따라야 한다: {:.1}",
        nodongbu.y - goyong.y
    );

    // 로고 그림은 자기 줄(줄1: '고용' 줄 + 8px)에 그려진다 — 종전에는 '노동부' 줄에
    // 끌려 내려가 글자와 겹쳤다.
    let logo = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Image(_)
                if (n.bbox.width - 170.4).abs() < 2.0 && n.bbox.y < goyong.y + 60.0 =>
            {
                Some(n.bbox)
            }
            _ => None,
        })
        .expect("로고 이미지");
    assert!(
        (logo.y - (goyong.y + 8.0)).abs() < 3.0,
        "로고는 자기 저장 줄(vpos +8px)에 있어야 한다: y={:.1} ('고용' y={:.1})",
        logo.y,
        goyong.y
    );
    // 이중 렌더 방지 — 같은 크기 로고가 하나만 있어야 한다.
    let logo_count = nodes
        .iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::Image(_))
                && (n.bbox.width - 170.4).abs() < 2.0
                && n.bbox.y < goyong.y + 120.0
        })
        .count();
    assert_eq!(logo_count, 1, "로고는 한 번만 그려져야 한다");
}
