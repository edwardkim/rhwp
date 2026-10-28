//! 관계형·고정형: #8153 독립 한컴 2024 KoPub Print의 핵심 출력 경계.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 9쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 94쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 149쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 153쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 163쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 309쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 310쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 383쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 9쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 94쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 149쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 153쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 163쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 309쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 310쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 383쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 두 원본의 383쪽 소유, 묶음 그림/crop/테두리, 중첩 표의 실제 행 하한을 함께 보호한다.
//! 좌표는 독립 PDF의 pt를 96dpi CSS px로 환산하며 허용 오차는 ±2px다.
//! Native/fresh WASM TSV·입력/PDF SHA·직접 판독은 #8153 검증 기록에 연결한다.
//! 근거: mydocs/plans/issue_8153_handbook_regression_replacement_20261025.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, shape::ShapeObject};
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::path::Path;

fn visit<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
    found.push(node);
    for child in &node.children {
        visit(child, found);
    }
}

fn text(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    visit(node, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                Some(run.text.as_str())
            } else {
                None
            }
        })
        .collect()
}

fn ops<'a>(node: &'a LayerNode, found: &mut Vec<(Option<u32>, &'a PaintOp)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                ops(child, found);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            ops(child, found)
        }
        LayerNodeKind::Leaf { ops } => found.extend(ops.iter().map(|op| (node.source_node_id, op))),
    }
}

fn near(actual: f64, expected: f64, label: &str) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 2.0,
        "{label}: {actual}, 기준 {expected}, 허용 ±2px"
    );
}

fn image_edges(bounds: BoundingBox, points: [f64; 4]) {
    for (actual, expected) in [
        bounds.x,
        bounds.y,
        bounds.x + bounds.width,
        bounds.y + bounds.height,
    ]
    .into_iter()
    .zip(points)
    {
        near(actual, expected * 96.0 / 72.0, "원본 소유 그림의 출력 경계");
    }
}

fn pictures(core: &DocumentCore, page: u32, expected: &[(usize, usize, [f64; 4])]) {
    let semantic = core.build_page_render_tree(page - 1).expect("그림 소유 쪽");
    let layer = core
        .build_page_layer_tree_with_profile(page - 1, RenderProfile::Print)
        .expect("실제 Print paint");
    let mut nodes = Vec::new();
    visit(&semantic.root, &mut nodes);
    let mut paint = Vec::new();
    ops(&layer.root, &mut paint);
    let mut frames = Vec::new();
    for &(section, paragraph, independent_points) in expected {
        let Control::Shape(source) =
            &core.document().sections[section].paragraphs[paragraph].controls[0]
        else {
            panic!("원본 묶음");
        };
        let ShapeObject::Group(source) = source.as_ref() else {
            panic!("원본 묶음 도형");
        };
        let ShapeObject::Picture(original) = &source.children[0] else {
            panic!("원본 그림");
        };
        let groups: Vec<_> = nodes
            .iter()
            .copied()
            .filter(|node| {
                matches!(&node.node_type,
            RenderNodeType::Group(group) if group.section_index == Some(section)
                && group.para_index == Some(paragraph) && group.control_index == Some(0))
                    && node.children.iter().any(|child| {
                        matches!(&child.node_type, RenderNodeType::Image(image)
                            if image.bin_data_id == original.image_attr.bin_data_id)
                    })
            })
            .collect();
        assert_eq!(groups.len(), 1, "원본 묶음 누락·중복 금지");
        let images: Vec<_> = groups[0]
            .children
            .iter()
            .filter_map(|node| {
                if let RenderNodeType::Image(image) = &node.node_type {
                    (image.bin_data_id == original.image_attr.bin_data_id).then_some((node, image))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(images.len(), 1);
        let (owner, image) = images[0];
        assert_eq!(image.bin_data_id, original.image_attr.bin_data_id);
        assert_eq!(image.crop, original.render_crop_rect(), "원본 crop 창 보존");
        assert_eq!(image.original_size_hu, original.crop_reference_size());
        let raster: Vec<_> = paint
            .iter()
            .filter_map(|(id, op)| match op {
                PaintOp::Image { bbox, .. } if *id == Some(owner.id) => Some(*bbox),
                _ => None,
            })
            .collect();
        assert_eq!(raster.len(), 1, "그림은 실제 Print에 한 번만 출력한다");
        image_edges(raster[0], independent_points);
        let border: Vec<_> = paint
            .iter()
            .filter_map(|(_, op)| match op {
                PaintOp::Rectangle { bbox, rect } if rect.image_frame_owner == Some(owner.id) => {
                    Some((*bbox, rect))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            border.len(),
            1,
            "같은 그림을 소유한 테두리를 한 번 출력한다"
        );
        let (bounds, outline) = border[0];
        assert!(outline.picture_border && outline.style.fill_color.is_none());
        assert_eq!(outline.style.stroke_color, Some(original.border_color));
        assert!(outline
            .print_picture_pen
            .is_some_and(|p| p.iter().all(|v| v.is_finite() && *v > 0.0)));
        near(
            bounds.x + bounds.width / 2.0,
            raster[0].x + raster[0].width / 2.0,
            "그림·테두리 중심 소유",
        );
        near(
            bounds.y + bounds.height / 2.0,
            raster[0].y + raster[0].height / 2.0,
            "그림·테두리 중심 소유",
        );
        frames.push(raster[0]);
    }
    for pair in frames.windows(2) {
        assert!(
            pair[0].y + pair[0].height < pair[1].y,
            "설명 그림 순서·비겹침"
        );
    }
}

fn nested_rows(core: &DocumentCore, page_number: u32, original_height_hu: f64, receipt_y_pt: f64) {
    let page = core
        .build_page_render_tree(page_number - 1)
        .expect("독립 Print의 신청서 소유 쪽");
    let mut nodes = Vec::new();
    visit(&page.root, &mut nodes);
    let tables: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type,
        RenderNodeType::Table(table) if table.row_count == 4 && table.col_count == 7)
                && text(node).contains("00000 신청서")
                && text(node).contains("작성방법")
        })
        .collect();
    assert_eq!(
        tables.len(),
        1,
        "{page_number}쪽 원문 신청서 표의 소유·누락·중복"
    );
    let table = tables[0];
    let cell = |row| {
        let cells: Vec<_> = table
            .children
            .iter()
            .filter(|node| {
                matches!(&node.node_type,
            RenderNodeType::TableCell(cell) if cell.row == row && cell.col == 0)
            })
            .collect();
        assert_eq!(cells.len(), 1);
        cells[0]
    };
    let title = cell(1);
    let notes = cell(2);
    let receipt = cell(3);
    // 두 원본의 제목 줄은 1000HU, 활성 안 여백은 141+141HU다.
    // 전체 높이는 원본 tbl/sz@height: 153쪽 5345HU, 149쪽 5628HU다.
    assert!(
        title.bbox.height * 75.0 >= 1282.0 - 1.0,
        "제목의 실제 줄과 활성 안 여백을 같은 행에 수용한다"
    );
    near(
        table.bbox.height,
        original_height_hu / 75.0,
        "독립적으로 닫힌 원본 표 전체 프레임",
    );
    assert!(title.bbox.y + title.bbox.height <= notes.bbox.y + 1.0 / 75.0);
    assert!(notes.bbox.y + notes.bbox.height <= receipt.bbox.y + 1.0 / 75.0);
    near(
        receipt.bbox.y,
        receipt_y_pt * 96.0 / 72.0,
        "독립 Print의 접수란 시작 경계",
    );
    assert_eq!(text(title).matches("신청서").count(), 1);
    let note_text = text(notes);
    assert_eq!(note_text.matches("작성방법").count(), 1);
    assert!(
        note_text.find("작성방법").unwrap() < note_text.find("표를 합니다").expect("뒤 안내 줄")
    );
    let mut lines = Vec::new();
    visit(notes, &mut lines);
    let lines: Vec<_> = lines
        .into_iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        .collect();
    assert_eq!(
        lines.len(),
        2,
        "원본과 독립 Print의 두 안내 줄 누락·중복 금지"
    );
    for line in lines {
        assert!(
            line.bbox.y >= notes.bbox.y
                && line.bbox.y + line.bbox.height <= notes.bbox.y + notes.bbox.height + 1.0 / 75.0,
            "안내 줄은 소유 셀 안에 있고 뒤 접수란을 침범하지 않는다"
        );
    }
}

fn verify(extension: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("편람 원본")).expect("편람 파싱");
    core.set_hangul2024_compat(true);
    assert_eq!(
        core.page_count(),
        383,
        "동일 원본의 독립 한컴 2024 Print 전체 쪽수"
    );
    let compact = |p| {
        core.extract_page_text_native(p)
            .expect("실제 쪽 내용")
            .split_whitespace()
            .collect::<String>()
    };
    let chapter = core.build_page_render_tree(8).expect("9쪽 장 간지");
    let mut chapter_nodes = Vec::new();
    visit(&chapter.root, &mut chapter_nodes);
    let first_items: Vec<_> = ["1. ‘", "2. "]
        .into_iter()
        .map(|prefix| {
            let matches: Vec<_> = chapter_nodes
                .iter()
                .copied()
                .filter(|node| {
                    matches!(&node.node_type, RenderNodeType::TextRun(run)
                        if run.text.starts_with(prefix))
                })
                .collect();
            assert_eq!(matches.len(), 1, "9쪽 항목의 소유·누락·중복: {prefix}");
            // 두 원본의 독립 한컴 Print에서 xMin=277.68pt다.
            near(matches[0].bbox.x, 277.68 * 96.0 / 72.0, "9쪽 항목 원점");
            matches[0]
        })
        .collect();
    assert!(
        first_items[0].bbox.y + first_items[0].bbox.height <= first_items[1].bbox.y,
        "장 간지 항목의 순서와 비겹침을 보존한다"
    );
    assert!(compact(308).starts_with("부록"), "309쪽 부록의 쪽 소유");
    assert!(compact(309).is_empty(), "부록 간지 뒤 빈 쪽 보존");
    let appendix = core.build_page_render_tree(308).expect("부록 간지");
    let mut appendix_nodes = Vec::new();
    visit(&appendix.root, &mut appendix_nodes);
    let titles: Vec<_> = appendix_nodes
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::TextRun(run)
                if run.text.starts_with("행정업무의"))
        })
        .collect();
    assert_eq!(titles.len(), 1, "부록 간지의 제목은 한 번만 표시한다");
    // 두 원본의 독립 한컴 Print에서 xMin=241.56pt, 96dpi에서 322.08px다.
    near(titles[0].bbox.x, 241.56 * 96.0 / 72.0, "부록 제목 원점");
    assert!(
        compact(382).contains("발행처:행정안전부정보공개과"),
        "마지막 판권 내용 보존"
    );
    pictures(
        &core,
        94,
        &[
            (2, 657, [118.431999, 130.906998, 481.399994, 277.412994]),
            (2, 661, [127.182999, 359.725006, 481.400024, 664.255005]),
        ],
    );
    pictures(
        &core,
        163,
        &[(4, 67, [154.992004, 234.696960, 466.055999, 664.974976])],
    );
    // 같은 닫힌 행 하한을 보호하는 완료 페이지 중 양쪽 raw100인 153쪽을 우선한다.
    nested_rows(&core, 153, 5345.0, 200.5);
    // 149쪽은 다른 안내 행의 선언 높이와 후속 접수란의 실제 소속도 함께 보호한다.
    nested_rows(&core, 149, 5628.0, 634.858032);
}

#[test]
fn hwp_keeps_independent_print_owners_picture_frames_and_nested_rows() {
    verify("hwp");
}

#[test]
fn hwpx_keeps_independent_print_owners_picture_frames_and_nested_rows() {
    verify("hwpx");
}
