//! [#6888] **자기 앵커보다 아래로 떨어진 자리차지 개체**가 뒤따르는 표를 자기 높이만큼 민다.
//!
//! `#409` 는 비-TAC · `vert=Para` · TopAndBottom 개체가 뒤따르는 콘텐츠를 개체 높이만큼
//! 밀어낸다고 보고 **조판과 배치 양쪽에서** 그 높이를 흐름에 계상한다. 그 전제는 밴드가
//! 앵커에서 시작할 때(`vertOffset == 0`) 참이다. 양수 오프셋이 밴드를 아래로 내려 놓으면
//! 그 사이에 들어갈 콘텐츠는 밀릴 이유가 없다.
//!
//! `156730935`(해양 모빌리티 엑스포 보도자료) **1쪽** 실측:
//!
//! ```text
//!   도형   TopAndBottom · vert=Para · vertOffset 150.3px · height 62.7px
//!   p17 마지막 vpos 61789 (+ lh 1400 + ls 420 = 63609)
//!   p18 저장 vpos          63609        ← 틈 0, 한글은 도형 자리를 만들지 않았다
//!
//!                  표(p18)              도형
//!   수정 전     1009.0 .. 1077.4     1020.1   표가 본문(94.5..1028.1)을 49.3px 넘는다
//!   수정 후      946.4 .. 1014.8     1020.1
//!   정본(2024)   945.2 .. 1013.6     1018.7
//! ```
//!
//! 도형 위치는 수정 전에도 맞았다 — 어긋난 것은 그 높이를 흐름에 계상한 것뿐이다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6888/156730935-marine-mobility-expo.hwpx";
/// 본문 하단 — 용지 1121.3, 아래 여백 반영. 문서 상수다.
const BODY_BOTTOM: f64 = 1028.1;

fn collect(node: &RenderNode, out: &mut Vec<(String, f64, f64)>) {
    let kind = match &node.node_type {
        RenderNodeType::Table(_) => Some("Table"),
        RenderNodeType::Rectangle(_) => Some("Rect"),
        _ => None,
    };
    if let Some(kind) = kind {
        out.push((kind.to_string(), node.bbox.y, node.bbox.height));
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn page1_boxes() -> Vec<(String, f64, f64)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&fs::read(path).expect("정식 원본")).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("render tree");
    let mut out = Vec::new();
    collect(&tree.root, &mut out);
    out
}

#[test]
fn issue_6888_contact_table_stays_inside_the_body() {
    let boxes = page1_boxes();
    let table = boxes
        .iter()
        .filter(|(kind, y, _)| kind == "Table" && *y > 900.0)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .expect("1쪽 하단 담당자 표");
    let (_, y, h) = table;

    // 정본 945.2..1013.6. 종전에는 1009.0..1077.4 로 본문을 49.3px 넘었다.
    assert!(
        (y - 946.4).abs() <= 1.5,
        "담당자 표가 저장 사다리 자리에 있어야 한다: y={y:.1} (기대 946.4, 정본 945.2)"
    );
    assert!(
        y + h <= BODY_BOTTOM + 0.5,
        "담당자 표가 본문 안에 있어야 한다: {:.1} vs 본문 하단 {BODY_BOTTOM}",
        y + h
    );
}

#[test]
fn issue_6888_displaced_shape_keeps_its_own_position() {
    // **음성 대조** — 이 수정은 개체를 옮기지 않는다. 도형은 종전에도 정본과 1.4px 안이었고
    // (`vertOffset` 150.3px 이 앵커 아래로 내려놓은 자리) 그대로 남아야 한다. 흐름 계상만
    // 거두는 수정이라는 것을 이 시험이 잠근다.
    let boxes = page1_boxes();
    let shape = boxes
        .iter()
        .find(|(kind, y, h)| kind == "Rect" && *y > 1000.0 && (*h - 62.7).abs() <= 0.5)
        .expect("1쪽 하단 자리차지 도형");
    assert!(
        (shape.1 - 1020.1).abs() <= 1.0,
        "도형은 제자리여야 한다: y={:.1} (기대 1020.1, 정본 1018.7)",
        shape.1
    );
}
