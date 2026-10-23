//! #8151: 중첩 배경 표는 셀 단 원점에서 바깥여백을 한 번 적용한다.
//! 독립 한컴 Print 144쪽과 전쪽 Native/fresh WASM 시각 근거는
//! mydocs/pr/assets/issue_8151/README.md에 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn stamp_tables(node: &RenderNode, out: &mut Vec<RenderNode>) {
    if let RenderNodeType::Table(t) = &node.node_type {
        if t.section_index == Some(3)
            && t.para_index == Some(4)
            && t.cell_context
                .as_ref()
                .is_some_and(|c| c.parent_para_index == 71)
        {
            out.push(node.clone());
        }
    }
    for child in &node.children {
        stamp_tables(child, out);
    }
}

fn sorted_stamps(core: &DocumentCore) -> Vec<RenderNode> {
    let page = core.build_page_render_tree(143).expect("독립 Print 144쪽");
    let mut stamps = Vec::new();
    stamp_tables(&page.root, &mut stamps);
    stamps.sort_by(|a, b| a.bbox.x.total_cmp(&b.bbox.x));
    assert_eq!(stamps.len(), 3, "같은 소유 문단의 도장 표 세 개");
    stamps
}

fn outer_table(controls: &mut [Control]) -> &mut Table {
    controls
        .iter_mut()
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_mut()),
            _ => None,
        })
        .expect("안내문을 소유한 표")
}

#[test]
fn nested_overlay_left_margin_is_consumed_once_without_advancing_flow() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/2025 행정업무운영 편람(최종).hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "같은 원본의 독립 Print 쪽수");
    let original = sorted_stamps(&core);

    let mut source = core.document().clone();
    let outer = outer_table(&mut source.sections[3].paragraphs[71].controls);
    let cell = outer
        .cells
        .iter_mut()
        .find(|c| c.row == 2 && c.col == 0)
        .unwrap();
    let mut margins = Vec::new();
    for control in &mut cell.paragraphs[4].controls {
        if let Control::Table(table) = control {
            margins.push((table.common.horizontal_offset, table.outer_margin_left));
            table.outer_margin_left = 0;
        }
    }
    margins.sort_by_key(|(offset, _)| *offset);
    assert_eq!(margins.len(), 3, "원본 도장 개체 소유와 수량");
    assert!(margins.iter().all(|(_, margin)| *margin > 0));
    let mut contrast = DocumentCore::new_empty();
    contrast.set_document(source);
    contrast.set_hangul2024_compat(true);
    assert_eq!(
        contrast.page_count(),
        core.page_count(),
        "배경의 가로여백은 흐름을 이월하지 않는다"
    );
    let zero_margin = sorted_stamps(&contrast);
    for ((actual, zero), (_, margin)) in original.iter().zip(&zero_margin).zip(margins) {
        let expected = f64::from(margin) * 96.0 / 7200.0;
        assert!(
            (actual.bbox.x - zero.bbox.x - expected).abs() < 96.0 / 7200.0,
            "도장 표의 바깥여백은 한 번 적용: 실제 이동={}, 원본 여백={expected}",
            actual.bbox.x - zero.bbox.x
        );
        assert!(
            (actual.bbox.y - zero.bbox.y).abs() < 1e-8,
            "가로여백은 문단의 세로 앵커를 바꾸지 않는다"
        );
        assert!((actual.bbox.width - zero.bbox.width).abs() < 1e-8);
        assert!((actual.bbox.height - zero.bbox.height).abs() < 1e-8);
    }
    for pair in original.windows(2) {
        assert!(
            pair[0].bbox.x + pair[0].bbox.width < pair[1].bbox.x,
            "오프셋 순서와 표 사이 비겹침"
        );
        assert!(
            (pair[0].bbox.y - pair[1].bbox.y).abs() < 96.0 / 7200.0,
            "같은 문단의 공통 세로 앵커"
        );
    }
}
