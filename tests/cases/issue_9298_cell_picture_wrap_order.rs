//! 관계형·고정형 — 셀 안 사진의 줄 소속·포함·겹침·그리기 순서와 한컴 인쇄의 쪽별 행 소속.
//! 주석 갱신 2026-11-25 11:24 KST (새 파일).
//! 공개 규제영향분석서 8쪽의 두 사진 자리를 색상 도형 JPEG로 치환했다. 한컴 2024로 독립 인쇄하고
//! HWP로 재저장했다. 96 DPI 전쪽 silhouette_raw_match_percent는 square HWPX 1쪽, front HWP 1쪽,
//! continued HWP·HWPX 각 2쪽 모두 Native/fresh WASM 100.00%다(2026-11-25, ad48653ab3).
//! devel 0fbee2c04e의 어울림은 79.06%, 이어지는 표는 91.77/88.13%였다. 글 앞으로는 실루엣이
//! 100.00%여도 겹친 부분에서 다른 색이 보였다. 한컴 PDF의 이미지 순서와 인쇄 색을 함께 대조했다.
//! 위쪽 괘선이 사진과 겹치는 HWP 1쪽은 최종 ad48653ab3에서 양쪽 100.00%다.
//! 두 사진 위에서 이어지는 갈색 괘선을 한컴 인쇄와 양쪽 출력의 실제 픽셀로 확인했다.
//! 기준 PDF는 `pdf/pr9298/`, 쪽별 수치와 확대 그림은 `mydocs/pr/assets/pr9298/cell-pictures/`에 있다.

use rhwp::model::shape::TextWrap;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn descendants<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        descendants(child, nodes);
    }
}

fn check_pictures(bytes: &[u8], pages: u32, expected: &[(u32, TextWrap)]) {
    let doc = DocumentCore::from_bytes(bytes).expect("사진 표 열기");
    assert_eq!(doc.page_count(), pages, "독립 인쇄와 같은 쪽수");
    let mut rows = Vec::new();
    for page in 0..pages {
        let tree = doc.build_page_render_tree(page).expect("쪽 조판");
        let mut nodes = Vec::new();
        descendants(&tree.root, &mut nodes);
        for node in nodes {
            let RenderNodeType::TableCell(cell) = &node.node_type else {
                continue;
            };
            let mut contents = Vec::new();
            descendants(node, &mut contents);
            let pictures: Vec<_> = contents
                .iter()
                .filter_map(|child| match &child.node_type {
                    RenderNodeType::Image(img) => Some((child.bbox, img)),
                    _ => None,
                })
                .collect();
            if cell.col != 0 {
                assert!(pictures.is_empty(), "사진은 첫 번째 칸 소유");
                continue;
            }
            let row = usize::from(cell.row);
            let (expected_page, wrap) = expected[row];
            assert_eq!(page, expected_page, "원본 {row}행의 쪽");
            rows.push(row);
            assert_eq!(pictures.len(), 2, "{row}행의 두 사진을 한 번씩 그린다");
            let order: Vec<_> = pictures
                .iter()
                .map(|(bbox, img)| {
                    let owner = img.cell_context.as_ref().expect("셀 편집 주소");
                    let slot = owner.path.last().expect("셀 경로");
                    assert_eq!((slot.cell_index, slot.cell_para_index), (row * 2, 0));
                    assert!(node.bbox.contains(bbox), "{row}행 사진이 칸을 벗어남");
                    img.control_index.expect("그림의 원본 순서")
                })
                .collect();
            let sign = pictures[order.iter().position(|&i| i == 0).expect("첫 사진")].0;
            let inline = pictures[order.iter().position(|&i| i == 1).expect("인라인 사진")].0;
            assert!(inline.y < sign.y + sign.height && sign.y < inline.y + inline.height);
            match wrap {
                TextWrap::Square => {
                    assert!(
                        sign.x + sign.width <= inline.x,
                        "어울림 그림 오른쪽의 인라인 사진"
                    );
                    assert!(
                        contents.iter().any(|n| {
                            matches!(n.node_type, RenderNodeType::TextLine(_))
                                && n.bbox.x <= inline.x
                                && inline.x + inline.width <= n.bbox.x + n.bbox.width
                        }),
                        "인라인 사진은 배제 후 남은 글줄 안에 놓인다"
                    );
                }
                TextWrap::InFrontOfText | TextWrap::BehindText => {
                    assert!(
                        sign.intersects(&inline),
                        "가운데 정렬된 사진과 의도적으로 겹친다"
                    );
                    let expected_order = if wrap == TextWrap::InFrontOfText {
                        [1, 0]
                    } else {
                        [0, 1]
                    };
                    assert_eq!(order, expected_order, "독립 인쇄에서 겹친 부분을 덮는 사진");
                }
                _ => unreachable!("표본에 없는 배치"),
            }
        }
    }
    assert_eq!(
        rows,
        (0..expected.len()).collect::<Vec<_>>(),
        "원본 행의 순서와 개수"
    );
}

#[test]
fn whole_cell_uses_its_available_line_and_local_foreground() {
    check_pictures(
        include_bytes!("../fixtures/pr9298_cell_picture_flow/cell-pictures-square.hwpx"),
        1,
        &[(0, TextWrap::Square)],
    );
    check_pictures(
        include_bytes!("../fixtures/pr9298_cell_picture_flow/cell-pictures-front.hwp"),
        1,
        &[(0, TextWrap::InFrontOfText)],
    );
}

#[test]
fn continued_table_keeps_picture_owners_and_paint_order_across_pages() {
    let rows = [
        (0, TextWrap::Square),
        (0, TextWrap::InFrontOfText),
        (0, TextWrap::BehindText),
        (1, TextWrap::Square),
        (1, TextWrap::InFrontOfText),
    ];
    for bytes in [
        include_bytes!("../fixtures/pr9298_cell_picture_flow/cell-pictures-continued.hwp")
            .as_slice(),
        include_bytes!("../fixtures/pr9298_cell_picture_flow/cell-pictures-continued.hwpx")
            .as_slice(),
    ] {
        check_pictures(bytes, 2, &rows);
    }
}

#[test]
fn cell_foreground_preserves_the_table_rule_across_the_photo_edge() {
    let doc = DocumentCore::from_bytes(include_bytes!(
        "../fixtures/pr9298_cell_picture_flow/cell-pictures-rule.hwp"
    ))
    .expect("위쪽 실선이 사진에 걸치는 한컴 저장본");
    let svg = doc
        .render_page_svg_layer_with_profile_native(0, rhwp::paint::RenderProfile::Print)
        .expect("인쇄 그리기 순서");
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let mut pictures = Vec::new();
    let mut top_rule = None;
    for (index, node) in xml.descendants().enumerate() {
        if node.has_tag_name("image") {
            let left = node.attribute("x").unwrap().parse::<f64>().unwrap();
            pictures.push((index, left));
        }
        if node.has_tag_name("line") && node.attribute("stroke") == Some("#403024") {
            assert!(top_rule.replace(index).is_none(), "윗괘선은 한 번만 그린다");
        }
    }
    let [(neighbor, neighbor_left), (sign, sign_left)] = pictures.as_slice() else {
        panic!("한컴 인쇄와 같이 사진 두 장을 한 번씩 그린다");
    };
    assert!(
        neighbor_left > sign_left,
        "오른쪽 이웃 사진 위에 왼쪽 안내판을 그린다"
    );
    assert!(
        neighbor < sign && *sign < top_rule.expect("갈색 윗괘선"),
        "두 사진 위에서도 괘선은 이어진다"
    );
}
