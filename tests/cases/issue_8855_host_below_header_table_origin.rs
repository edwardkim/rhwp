//! 관계형·고정형 — [#8855] 머리 표 아래에 저장된 빈 host 줄의 단 원점과 전폭 어울림 그림 뒤 글.
//! 주석 갱신: MEASURE_TS (새 파일)
//!
//! 입력: 공개 결재문서 36388005(한컴오피스 2020 저장) 원본. 단 첫 문단 `pi=0` 은 글 없이 문단 기준
//! 자리차지 머리 표(바깥 여백 위 140·아래 852HU)와 빈 누름틀을 갖고, host 줄은 표 상자 바로 아래
//! (vpos 17645 = 0 + 140 + 16653 + 852), 다음 줄은 그 한 줄 뒤(19085 = 17645 + 1200 + 240)에 있다.
//! `pi=6` 은 sw=0 줄의 전폭 어울림 그림(문단 기준·위, 15170HU), `pi=7` 은 그림 끝(42055)에 저장돼 있고,
//! 마지막 `pi=9` 는 쪽 아래 고정 표(365px)이며 저장 vpos 0 이다.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8855/36388005-host-below-header-table-2020.pdf`(2쪽).
//! - 고정형: 전체 쪽수 2.
//! - 관계형 1쪽: 첫 본문 `pi=1` 줄 위끝 − 머리 표 아래끝 = 표 아래 바깥 여백 + host 줄 전진(852 + 1200 +
//!   240HU), `pi=7` 줄은 그림 아래끝 아래에서 시작한다.
//! - 관계형 2쪽: 쪽 아래 고정 표 `pi=9` 는 1쪽에 없고 2쪽에 있다.
//!
//! MEASUREMENTS
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8855/36388005-host-below-header-table.hwpx";
const TOLERANCE_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = Vec::new();
    walk(node, &mut out);
    out
}

fn body_line_top(page: &RenderNode, para: usize) -> Option<f64> {
    nodes(page).into_iter().find_map(|node| match &node.node_type {
        RenderNodeType::TextRun(run)
            if run.section_index == Some(0)
                && run.para_index == Some(para)
                && run.cell_context.is_none() =>
        {
            Some(node.bbox.y)
        }
        _ => None,
    })
}

fn body_table(page: &RenderNode, para: usize, control: usize) -> Option<&RenderNode> {
    nodes(page).into_iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(para) && table.control_index == Some(control))
    })
}

fn body_image(page: &RenderNode, para: usize) -> Option<&RenderNode> {
    nodes(page).into_iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::Image(image) if image.para_index == Some(para))
    })
}

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

#[test]
fn first_body_line_follows_header_table_bottom_margin_and_host_line() {
    let core = open();
    let page = core.build_page_render_tree(0).expect("1쪽");
    let header = body_table(&page.root, 0, 2).expect("1쪽 머리 표 pi=0 ci=2");
    let line = body_line_top(&page.root, 1).expect("1쪽 pi=1 글줄");
    let expected = hu_to_px(852 + 1200 + 240);
    let gap = line - (header.bbox.y + header.bbox.height);
    assert!(
        (gap - expected).abs() <= TOLERANCE_PX,
        "pi=1 글줄이 머리 표 아래끝에서 {gap:.1}px 떨어졌다 — 표 아래 바깥 여백 + host 줄 전진 \
         {expected:.1}px 이어야 한다(수정 전 28.7px)"
    );
}

#[test]
fn text_after_full_width_square_picture_starts_below_it() {
    let core = open();
    let page = core.build_page_render_tree(0).expect("1쪽");
    let picture = body_image(&page.root, 6).expect("1쪽 pi=6 그림");
    let picture_bottom = picture.bbox.y + picture.bbox.height;
    for para in [7, 8] {
        let line = body_line_top(&page.root, para).unwrap_or_else(|| panic!("1쪽 pi={para} 글줄"));
        assert!(
            line + TOLERANCE_PX >= picture_bottom,
            "pi={para} 글줄 위끝 {line:.1} 이 전폭 어울림 그림 아래끝 {picture_bottom:.1} 보다 위다"
        );
    }
}

#[test]
fn page_bottom_footer_table_moves_to_second_page_like_hancom() {
    let core = open();
    assert_eq!(core.page_count(), 2, "한컴 engine 2020 Print PDF 는 2쪽");
    let first = core.build_page_render_tree(0).expect("1쪽");
    let second = core.build_page_render_tree(1).expect("2쪽");
    assert!(
        body_table(&first.root, 9, 0).is_none(),
        "쪽 아래 고정 표 pi=9 는 위 글과 겹치므로 1쪽에 남지 않는다"
    );
    let footer = body_table(&second.root, 9, 0).expect("2쪽 쪽 아래 고정 표 pi=9");
    assert!(footer.bbox.height > 0.0);
}
