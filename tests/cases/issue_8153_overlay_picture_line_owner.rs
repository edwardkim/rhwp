//! 관계형·고정형: 셀 안 그림은 소유 줄의 위치를 유지하고 셀 안에 표시된다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 260쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 260쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 첫 줄과 그림 소유 줄의 저장 위치 차이는 원본에서 확인하고, 그림 상단은 독립 Print와 비교한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn collect_picture(
    node: &RenderNode,
    cell: Option<BoundingBox>,
    found: &mut Vec<(BoundingBox, BoundingBox)>,
    lines: &mut Vec<(BoundingBox, BoundingBox)>,
) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node.bbox)
    } else {
        cell
    };
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.bin_data_id == 417 {
            found.push((node.bbox, cell.expect("그림을 소유한 셀")));
        }
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_))
        && text(node).contains("처리 방안을 기술하였습니까?")
    {
        lines.push((node.bbox, cell.expect("글줄을 소유한 셀")));
    }
    for child in &node.children {
        collect_picture(child, cell, found, lines);
    }
}

fn verify(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("편람 원본")).expect("편람 파싱");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print와 전체 쪽수 일치");
    let Control::Table(table) = &core.document().sections[9].paragraphs[89].controls[0] else {
        panic!("원본 그림을 소유한 표")
    };
    let source: Vec<_> = table
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .flat_map(|para| {
            para.controls
                .iter()
                .filter_map(move |control| match control {
                    Control::Picture(picture) if picture.image_attr.bin_data_id == 417 => {
                        Some((para, picture))
                    }
                    _ => None,
                })
        })
        .collect();
    assert_eq!(source.len(), 1, "해당 원본 그림의 소유 문단");
    let (source_para, source_picture) = source[0];
    assert_eq!(source_para.line_segs.len(), 2, "원본 두 줄 보존");
    assert_eq!(
        source_para.line_segs[1].vertical_pos - source_para.line_segs[0].vertical_pos,
        1144,
        "원본 두 줄의 저장 간격"
    );
    assert_eq!(source_picture.common.vertical_offset, 73);
    let page = core.build_page_render_tree(259).expect("260쪽");
    let mut found = Vec::new();
    let mut lines = Vec::new();
    collect_picture(&page.root, None, &mut found, &mut lines);
    assert_eq!(found.len(), 1, "260쪽의 해당 그림은 한 번만 표시된다");
    assert_eq!(lines.len(), 1, "그림을 소유한 문단의 두 번째 줄");
    let (picture, cell) = found[0];
    let (line, line_cell) = lines[0];
    assert_eq!(
        (cell.x, cell.y, cell.width, cell.height),
        (line_cell.x, line_cell.y, line_cell.width, line_cell.height),
        "그림과 글줄이 같은 셀을 소유한다"
    );
    assert!(picture.y >= cell.y && picture.y + picture.height <= cell.y + cell.height);
    // 원본 그림의 문단 기준 수직 오프셋 73HU를 해당 글줄에 적용한다.
    assert!((picture.y - line.y - 73.0 * 96.0 / 7200.0).abs() <= 2.0);
    // 양쪽 독립 Print 260쪽 이미지 xref 1992의 사각형을 96dpi로 환산한다.
    let expected_points = [115.915, 469.155, 236.145, 498.192];
    let actual = [
        picture.x,
        picture.y,
        picture.x + picture.width,
        picture.y + picture.height,
    ];
    for (edge, expected) in actual.into_iter().zip(expected_points) {
        assert!(
            (edge - expected * 96.0 / 72.0).abs() <= 2.0,
            "독립 Print 그림 경계: 실제 {edge}, 기준 {}",
            expected * 96.0 / 72.0
        );
    }
}

#[test]
fn hwp_overlay_picture_keeps_its_owned_line() {
    verify("hwp");
}

#[test]
fn hwpx_overlay_picture_keeps_its_owned_line() {
    verify("hwpx");
}
