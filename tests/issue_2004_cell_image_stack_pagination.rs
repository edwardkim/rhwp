//! Issue #2004 잔여: 부동 표 셀 이미지 스택 콘텐츠 페이지네이션 회귀 테스트.
//!
//! `samples/issue2004_cell_image_stack.hwp(x)` — 1×1 RowBreak 부동 표(자리차지,
//! tac=false) 셀에 전면급 Square 부동 이미지 5장이 varying offset으로 스택된 문서.
//! 수정 전에는 셀 측정이 저장 높이(871.9px)에 머물러 표가 원자 배치되고 이미지
//! 5장이 한 쪽에 겹쳐 4쪽으로 렌더됐다.
//!
//! 수정: 정규화(`compute_render_normalized`)에서 셀 스택을 이미지 1장짜리 inline
//! 문단 N개로 분할하고 각 문단에 이미지 높이 합성 line_seg를 부여 → 셀 측정이
//! 스택 총높이(4310.6px)를 반영, 기존 RowBreak 분할 스캔에 자연 진입해 쪽당
//! 이미지 1장씩 배치된다.
//!
//! 독립 Print `pdf/issue2004_cell_image_stack-{hwp,hwpx}-2020.pdf`는
//! 8쪽이다(p1~p3 본문, p4~p8 프레임 이미지 1장씩). 쪽수뿐 아니라 원본 그림의
//! 순서·단일 소유·셀 내부 포함을 함께 검사한다. 새 전쪽 Native/fresh WASM
//! 증거와 직접 판독은 mydocs/working/task_m100_7902_stage3.md에 연결한다.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

fn cell_images<'a>(
    node: &'a RenderNode,
    owner: Option<&'a RenderNode>,
    images: &mut Vec<(u16, &'a RenderNode, Option<&'a RenderNode>)>,
) {
    let owner = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        owner
    };
    if let RenderNodeType::Image(image) = &node.node_type {
        images.push((image.bin_data_id, node, owner));
    }
    for child in &node.children {
        cell_images(child, owner, images);
    }
}

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn assert_stack_pagination(rel: &str) {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    assert_eq!(
        doc.page_count(),
        8,
        "{rel}: 독립 Print는8쪽이다. 4쪽이면 그림 스택 미분할, 9쪽 이상이면 과분할 회귀"
    );

    // 정규화된 렌더 트리에서 기대 ID를 복사하지 않고 원본 1×1 표의 그림 순서를 읽는다.
    let source = rhwp::parse_document(&bytes).expect("원본 그림 소유자");
    let stacks: Vec<Vec<u16>> = source
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Table(table) if table.row_count == 1 && table.col_count == 1 => Some(table),
            _ => None,
        })
        .map(|table| {
            table
                .cells
                .iter()
                .flat_map(|cell| &cell.paragraphs)
                .flat_map(|paragraph| &paragraph.controls)
                .filter_map(|control| match control {
                    Control::Picture(picture) => Some(picture.image_attr.bin_data_id),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .filter(|ids| ids.len() == 5)
        .collect();
    assert_eq!(stacks.len(), 1, "{rel}: 원본 그림 스택은 하나");
    let expected = &stacks[0];
    assert_eq!(
        expected
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        5,
        "원본의 서로 다른 그림 다섯 장"
    );
    let mut seen = Vec::new();
    for page in 0..8 {
        let tree = doc.build_page_render_tree(page).expect("전체 쪽 렌더 트리");
        let mut images = Vec::new();
        cell_images(&tree.root, None, &mut images);
        let stack: Vec<_> = images
            .iter()
            .filter(|(id, _, _)| expected.contains(id))
            .collect();
        if page < 3 {
            assert!(stack.is_empty(), "본문 쪽에 스택 그림이 먼저 배치됨");
            // 독립 Print의 본문·후속 본문·명단을 빈 쪽으로 대체하지 않는다.
            let needle = ["농촌진흥청국립원예특작과학원", "우수상은", "수상자명단"][page as usize];
            let body: String = text(&tree.root)
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            assert!(
                body.contains(needle),
                "{rel} {}쪽 본문 누락: {needle}",
                page + 1
            );
            continue;
        }
        assert_eq!(images.len(), 1, "{rel} {}쪽 그림은 한 장", page + 1);
        assert_eq!(stack.len(), 1, "원본 스택 그림의 단일 쪽 소유");
        let (id, image, owner) = *stack[0];
        assert_eq!(id, expected[(page - 3) as usize], "원본 그림 순서");
        let cell = owner.expect("그림의 물리 셀 소유자").bbox;
        let image = image.bbox;
        assert!(image.width > 0.0 && image.height > 0.0, "빈 그림 상자");
        assert!(
            image.x >= cell.x
                && image.y >= cell.y
                && image.x + image.width <= cell.x + cell.width
                && image.y + image.height <= cell.y + cell.height,
            "{rel} {}쪽 그림이 소유 셀 밖으로 나감: {image:?}, {cell:?}",
            page + 1
        );
        seen.push(id);
    }
    assert_eq!(&seen, expected, "원본 그림 전체의 순서·누락·중복 방지");
}

#[test]
fn cell_image_stack_hwp_paginates_to_8_pages() {
    assert_stack_pagination("samples/issue2004_cell_image_stack.hwp");
}

#[test]
fn cell_image_stack_hwpx_paginates_to_8_pages() {
    assert_stack_pagination("samples/issue2004_cell_image_stack.hwpx");
}
