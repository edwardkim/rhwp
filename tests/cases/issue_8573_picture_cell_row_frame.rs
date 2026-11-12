//! 검사 유형: 관계형. 저장 줄 없는 그림 전용 셀의 안 여백·행 경계·내용 소속을 검사한다.
//! 주석 갱신: 2026-11-12 13:56:14 KST.
//! 공개 합성 HWPX 6종의 같은 입력을 한컴 2020 11.0.0.9136에서 독립 Print했다.
//! 기준: pdf/issue8573/picture-cell-*-2020.pdf. Native/fresh WASM 전쪽 raw는 검증 후 기록한다.
//! 절대 좌표를 핀하지 않고 그림과 아래 안 여백의 셀 포함, host 사이 행 높이 일치,
//! 독립 Print의 2쪽 경계와 그림 20개·후속 본문의 단일 소속을 함께 검사한다.
//! 비공개 원본 3건을 확보한 것으로 간주하지 않는다. 원 PR #8897의 행 예약/paint 차이를
//! 별도의 공개 입력에서 재현하며 빈 host, 공백 host, 글자 host를 대조한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn load(layout: &str, host: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "samples/issue8573/picture-cell-{layout}-{host}.hwpx"
    ));
    DocumentCore::from_bytes(&std::fs::read(path).expect("공개 합성 입력"))
        .expect("그림 셀 문서 로드")
}

fn visit<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
    found.push(node);
    for child in &node.children {
        visit(child, found);
    }
}

#[test]
fn unstored_picture_rows_keep_the_bottom_padding_inside_the_cell() {
    for host in ["empty", "spaces", "text"] {
        let core = load("short", host);
        let tree = core.build_page_render_tree(0).expect("첫 쪽");
        let mut nodes = Vec::new();
        visit(&tree.root, &mut nodes);
        let cells: Vec<_> = nodes
            .iter()
            .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
            .collect();
        assert_eq!(cells.len(), 4);
        for cell in cells {
            let mut inside = Vec::new();
            visit(cell, &mut inside);
            let picture = inside
                .iter()
                .find(|node| matches!(node.node_type, RenderNodeType::Image(_)))
                .expect("각 행의 그림");
            let top_padding = picture.bbox.y - cell.bbox.y;
            assert!(top_padding > 0.0, "원본의 양수 안 여백");
            assert!(
                picture.bbox.y + picture.bbox.height + top_padding
                    <= cell.bbox.y + cell.bbox.height + 0.5,
                "{host}: 그림과 대칭 아래 안 여백을 행 안에 보존해야 한다"
            );
        }
    }
}

#[test]
fn host_text_does_not_compress_the_picture_row_frame() {
    let heights: Vec<_> = ["empty", "spaces", "text"]
        .iter()
        .map(|host| {
            let core = load("short", host);
            let tree = core.build_page_render_tree(0).expect("첫 쪽");
            let mut nodes = Vec::new();
            visit(&tree.root, &mut nodes);
            nodes
                .into_iter()
                .find(|node| matches!(node.node_type, RenderNodeType::Table(_)))
                .expect("그림 표")
                .bbox
                .height
        })
        .collect();
    assert!(heights
        .iter()
        .all(|height| (height - heights[0]).abs() <= 0.5));
}

#[test]
fn boundary_rows_and_following_text_keep_independent_print_page_ownership() {
    for host in ["empty", "spaces", "text"] {
        let core = load("page-boundary", host);
        assert_eq!(core.page_count(), 2, "독립 Print의 두 쪽: {host}");
        let outer_top = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Table(table) => Some(f64::from(table.outer_margin_top) * 96.0 / 7200.0),
                _ => None,
            })
            .expect("원본 표의 위 바깥 여백");
        let mut images = [0usize; 2];
        let mut after = [0usize; 2];
        let mut end = 0;
        for page in 0..2 {
            let tree = core.build_page_render_tree(page as u32).expect("각 쪽");
            let mut nodes = Vec::new();
            visit(&tree.root, &mut nodes);
            let column = nodes
                .iter()
                .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
                .expect("그림 표의 단");
            let table = nodes
                .iter()
                .find(|node| matches!(node.node_type, RenderNodeType::Table(_)))
                .expect("각 쪽의 표 조각");
            assert!(
                (table.bbox.y - column.bbox.y - outer_top).abs() <= 0.5,
                "{host} {}쪽: 예약한 원본 위 바깥 여백을 실제 첫 괘선에서도 소비해야 한다",
                page + 1
            );
            for node in nodes {
                match &node.node_type {
                    RenderNodeType::Image(_) => images[page] += 1,
                    RenderNodeType::TextRun(run) => {
                        after[page] += usize::from(run.text.starts_with("AFTER TABLE"));
                        end += usize::from(run.text == "END");
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(
            images,
            [19, 1],
            "원본 그림의 누락·중복 없는 쪽 소속: {host}"
        );
        assert_eq!(after, [0, 1], "표 뒤 본문은 다음 쪽에 한 번: {host}");
        assert_eq!(end, 1, "마지막 문구 보존: {host}");
    }
}

#[test]
fn continuation_preserves_the_same_host_gap_after_the_last_picture_row() {
    // 독립 Print에서 같은 host의 종료 간격은 짧은 표와 이어받은 표에서 같다.
    // 절대 좌표 대신 같은 내용·안/바깥 여백이 소유하는 앞뒤 간격을 대조한다.
    for host in ["empty", "spaces", "text"] {
        let gap = |layout: &str, page: u32| {
            let core = load(layout, host);
            let tree = core.build_page_render_tree(page).expect("표 종료 쪽");
            let mut nodes = Vec::new();
            visit(&tree.root, &mut nodes);
            let table = nodes
                .iter()
                .find(|node| matches!(node.node_type, RenderNodeType::Table(_)))
                .expect("종료 표 조각");
            let after = nodes
                .iter()
                .find(|node| {
                    matches!(&node.node_type, RenderNodeType::TextRun(run)
                    if run.text.starts_with("AFTER TABLE"))
                })
                .expect("표 뒤 본문");
            after.bbox.y - table.bbox.y - table.bbox.height
        };
        assert!(
            (gap("short", 0) - gap("page-boundary", 1)).abs() <= 0.5,
            "{host}: 이어받은 표의 위 바깥 여백을 뒤 본문에 다시 소비하면 안 된다"
        );
    }
}
