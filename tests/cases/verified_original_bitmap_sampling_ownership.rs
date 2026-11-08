//! 비변환 그림의 원래 소속·내용 및 완료된 쪽의 독립 Print 경계를 보호한다.
//! 전74쪽 양쪽90%와 직접 판독 뒤 등록한다. 62쪽은 관계 검사만 사용한다.
//! 양쪽99%대인 19쪽의 독립 Print 경계에만 ±2px 검사를 추가한다.
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
fn scan<'a>(
    node: &'a RenderNode,
    images: &mut Vec<&'a RenderNode>,
    cells: &mut Vec<BoundingBox>,
    text: &mut String,
) {
    match &node.node_type {
        RenderNodeType::Image(_) => images.push(node),
        RenderNodeType::TextRun(run) => text.push_str(&run.text),
        RenderNodeType::TableCell(_) => cells.push(node.bbox),
        _ => {}
    }
    for child in &node.children {
        scan(child, images, cells, text)
    }
}
#[test]
fn legal_comparison_bitmap_keeps_original_cell_ownership_and_content_order() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6776/78494-virtual-convergence-industry-decree.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(core.page_count(), 74, "같은 원문 독립 Print 전체 쪽수");
    let tree = core.build_page_render_tree(61).unwrap();
    let mut pictures = Vec::new();
    let mut cells = Vec::new();
    let mut text = String::new();
    scan(&tree.root, &mut pictures, &mut cells, &mut text);
    assert_eq!(pictures.len(), 1, "물리62쪽 그림 누락·중복 방지");
    let picture = pictures[0];
    let RenderNodeType::Image(image) = &picture.node_type else {
        unreachable!()
    };
    assert!(
        image.para_index.is_some() && image.control_index.is_some(),
        "원문 소유 연결"
    );
    let rect = image
        .transform
        .effective_image_bbox(&image.paint_bbox(&picture.bbox));
    assert!(
        cells.iter().any(|cell| rect.x >= cell.x
            && rect.y >= cell.y
            && rect.x + rect.width <= cell.x + cell.width
            && rect.y + rect.height <= cell.y + cell.height),
        "비교표의 그림은 해당 셀 내부에 있어야 한다"
    );
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        compact.contains("<조문대비표>")
            && compact.contains("현행")
            && compact.contains("개정안")
            && compact.contains("<신설>"),
        "앞뒤 내용 보존"
    );
    assert!(
        compact.find("현행").unwrap() < compact.find("개정안").unwrap(),
        "비교 열의 내용 순서"
    );
}

#[test]
fn completed_legal_page_nineteen_keeps_source_owned_bitmap_print_bounds() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6776/78494-virtual-convergence-industry-decree.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(core.page_count(), 74);
    let tree = core.build_page_render_tree(18).unwrap();
    let mut pictures = Vec::new();
    let mut cells = Vec::new();
    let mut text = String::new();
    scan(&tree.root, &mut pictures, &mut cells, &mut text);
    assert_eq!(pictures.len(), 1, "물리19쪽 원문 그림의 누락·중복 방지");
    let picture = pictures[0];
    let RenderNodeType::Image(image) = &picture.node_type else {
        unreachable!()
    };
    assert!(
        image.para_index.is_some() && image.control_index.is_some(),
        "원문 그림 소유"
    );
    let rect = image
        .transform
        .effective_image_bbox(&image.paint_bbox(&picture.bbox));
    assert!(
        rect.x >= 0.0
            && rect.y >= 0.0
            && rect.x + rect.width <= tree.root.bbox.width
            && rect.y + rect.height <= tree.root.bbox.height,
        "그림의 쪽 내부 포함"
    );
    // 독립 Print page19의 Im1(xref109) rect를 pt×96/72로 환산했다.
    // 같은 비변환 원본 그림을 검증하는 완료 쪽 중 양쪽99%대이며, 62쪽(양쪽 최저97.60302%)은 고정하지 않는다.
    for (name, actual, printed) in [
        ("left", rect.x, 212.845337),
        ("top", rect.y, 82.309326),
        ("right", rect.x + rect.width, 700.262695),
        ("bottom", rect.y + rect.height, 394.447998),
    ] {
        assert!(
            (actual - printed).abs() <= 2.0,
            "완료된19쪽 {name}: 실제{actual}, 독립Print{printed}"
        );
    }
}
