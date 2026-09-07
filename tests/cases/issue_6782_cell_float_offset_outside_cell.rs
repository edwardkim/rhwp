//! [Issue #6782] 조각 표의 셀 앵커 그림에서, **그림을 자기 칸 밖으로 통째로 밀어내는**
//! 세로 오프셋이 그대로 실려 그림이 쪽 위쪽 밖(음수 y)으로 나가 소실되던 결함의 가드.
//!
//! ## 계약 — 부호도 크기도 아니라 **칸과 겹치는가**
//!
//! 한/글은 셀 앵커 그림의 세로 오프셋을 대체로 그대로 적용하지만, 그 결과가 그림을 자기
//! 칸 밖으로 완전히 내보내면 쓰지 않는다. 한/글 2020 오라클 실측이 그 축을 가른다.
//!
//! ```text
//!   문제 그림 (h≈65.7)   한/글 y = 235.9     rhwp 종전 y = −233.4  ← 용지 밖, 소실
//!                                            이 수정   y =  238.7  (한/글과 2.8px)
//!
//!   voff −70,819HU = −944.25px, 셀 valign=Center
//!   232.1 + (79.05 − 65.8 − 944.25) / 2 = −233.4
//! ```
//!
//! ⚠ **음수라고 버리면 안 된다.** `#5734`(156684746 9쪽 왼쪽 칸)의 첫 그림도 저장 vpos 가
//! 0이라 같은 폴백 갈래로 오는데, 거기서는 `−1,079HU`(14.4px)가 **적용되는 것이 정답**이고
//! `issue_5734_cell_float_stack_stored_vpos` 가 그 값을 잠근다. 실제로 「음수 → 0」 판과
//! 「결과 바닥을 칸 상단으로」 판은 **둘 다 그 핀을 깨뜨렸다**(전수 9020 중 그 1건).
//!
//! ```text
//!   #6782  y + h = −167.6 ≤ 칸 상단 232.1   → 칸 밖   → 오프셋 무시
//!   #5734  y + h =  701.1 >  칸 상단 631.0   → 겹침    → 오프셋 적용
//! ```
//!
//! ## 필수 실물 재현물
//! 원문 전체를 samples/issue6782에 보존한다. 조각 표의 문맥과 BinData를 바꾸지 않으며,
//! fixture가 없으면 실패한다. 개인 PC 경로 탐색이나 환경 변수에 따른 묵시적 skip은 없다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const PAGE_INDEX: u32 = 76;
const PAGE_HEIGHT_PX: f64 = 1122.5;
const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";

fn sample() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6782 정식 실물 fixture 읽기")
}

fn collect_cell_images<'a>(
    node: &'a RenderNode,
    cell: Option<&'a RenderNode>,
    out: &mut Vec<(f64, f64, f64)>,
) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        if let Some(cell) = cell {
            out.push((cell.bbox.y, node.bbox.y, node.bbox.height));
        }
    }
    for child in &node.children {
        collect_cell_images(child, cell, out);
    }
}

#[test]
fn offset_that_pushes_a_cell_image_out_of_its_cell_is_not_applied() {
    let bytes = sample();
    let document = HwpDocument::from_bytes(&bytes).expect("parse 1480000-201900042");
    assert_eq!(document.page_count(), 104, "쪽수는 104쪽이어야 한다");

    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p77");
    let mut images = Vec::new();
    collect_cell_images(&tree.root, None, &mut images);

    assert_eq!(images.len(), 11, "77쪽의 칸 안 그림 11개를 보존해야 한다");

    for (cell_y, image_y, image_h) in &images {
        assert!(
            *image_y >= 0.0 && *image_y <= PAGE_HEIGHT_PX,
            "칸 안 그림이 용지 밖으로 나갔다 — 그림 y={image_y:.1}px (용지 0..{PAGE_HEIGHT_PX:.1}). \
             회귀 시 −233.4px 로 인쇄에서 소실된다"
        );
        // 오프셋이 살아 있어도 그림은 자기 칸과 **겹쳐야** 한다. 한/글은 칸 밖으로
        // 통째로 내보내는 오프셋을 쓰지 않는다.
        assert!(
            *image_y + *image_h > *cell_y,
            "칸 안 그림이 자기 칸 위로 통째로 벗어났다 — 칸 {cell_y:.1}px, \
             그림 {image_y:.1}+{image_h:.1}px"
        );
    }
}

#[test]
fn the_ccc_image_is_restored_in_its_original_fragment_cell() {
    fn collect<'a>(node: &'a RenderNode, in_target: bool, out: &mut Vec<&'a RenderNode>) {
        let in_target = match &node.node_type {
            RenderNodeType::TableCell(cell) => {
                cell.row == 4 && cell.col == 3 && cell.model_cell_index == Some(19)
            }
            _ => in_target,
        };
        if in_target && matches!(node.node_type, RenderNodeType::Image(_)) {
            out.push(node);
        }
        for child in &node.children {
            collect(child, in_target, out);
        }
    }
    let document = HwpDocument::from_bytes(&sample()).expect("문서 로드");
    let tree = document.build_page_render_tree(PAGE_INDEX).expect("77쪽");
    let mut images = Vec::new();
    collect(&tree.root, false, &mut images);
    assert_eq!(images.len(), 1, "row4/col3 조각 셀의 CCC 그림 하나");
    let image = images[0];
    if let RenderNodeType::Image(data) = &image.node_type {
        assert_eq!(data.bin_data_id, 79, "원본 CCC 이미지 참조 보존");
        assert_eq!(data.para_index, Some(118));
    }
    // engine 2020 기준 y=235.9. 현재 오차 약 2.8px를 명시적으로 제한한다.
    assert!(
        (image.bbox.y - 235.9).abs() <= 3.0,
        "CCC 위치: {:?}",
        image.bbox
    );
    assert!((image.bbox.height - 65.8).abs() <= 1.0, "CCC 크기 보존");
}
