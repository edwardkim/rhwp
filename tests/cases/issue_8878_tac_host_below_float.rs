//! 관계형·고정형 — [#8878] 머리 표 아래에 저장된 host 줄의 글자처럼 취급한 표.
//! 주석 갱신: 2026-11-10 08:10 KST (새 파일)
//!
//! 입력: 공개 결재문서 36441289(한컴오피스 2020 저장) 원본. `pi=0` 은 글 없는 host 에 문단 기준
//! 자리차지 머리 표 `ci=2`(바깥 위 140·아래 852HU, 높이 16653HU)와 글자처럼 취급한 표 `ci=4`(바깥
//! 위 141HU) 를 갖는다. host 줄은 표 상자 바로 아래(vpos 17645 = 0 + 140 + 16653 + 852), 다음 문단은
//! 그 줄 전진 뒤(46470 = 17645 + 30025 − 1200)다. 마지막 `pi=5` 는 쪽 아래 고정 표(386.5px)다.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8878/36441289-tac-host-below-float-2020.pdf`(2쪽).
//! - 고정형: 전체 쪽수 2.
//! - 관계형 1쪽: `ci=4` 위끝 − `ci=2` 아래끝 = 머리 표 아래 바깥 여백 + 글자처럼 취급한 표 위 바깥 여백.
//! - 관계형 2쪽: 쪽 아래 고정 표 `pi=5` 는 1쪽에 없고 2쪽에 있다.
//!
//! 측정(2026-11-10, 코드 head 52da22a0af, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, 글꼴 파일 gulim.ttc·malgun.ttf·HYGTRE·HYSNRL·HYGSRB 공급):
//! - Native: 2쪽 / 2쪽, p1 99.97, p2 100.00, 전쪽 최저 99.97 (수정 전 1fd8576c91 은 1쪽으로 쪽수 불일치, p1 29.33)
//! - fresh WASM: 2쪽 / 2쪽, p1 100.00, p2 100.00, 전쪽 최저 100.00
//! - 증거: `mydocs/pr/assets/issue_8878/`
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8878/36441289-tac-host-below-float.hwpx";
const TOLERANCE_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn find_table(node: &RenderNode, para: usize, control: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.para_index == Some(para) && table.control_index == Some(control))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para, control))
}

fn sample_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

fn open(bytes: &[u8]) -> DocumentCore {
    DocumentCore::from_bytes(bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

#[test]
fn inline_table_line_starts_after_header_table_box() {
    let core = open(&sample_bytes());
    let page = core.build_page_render_tree(0).expect("1쪽");
    let header = find_table(&page.root, 0, 2).expect("1쪽 머리 표 pi=0 ci=2");
    let inline = find_table(&page.root, 0, 4).expect("1쪽 글자처럼 취급한 표 pi=0 ci=4");
    let expected = hu_to_px(852 + 141);
    let gap = inline.bbox.y - (header.bbox.y + header.bbox.height);
    assert!(
        (gap - expected).abs() <= TOLERANCE_PX,
        "글자처럼 취급한 표가 머리 표 아래끝에서 {gap:.1}px 떨어졌다 — 두 바깥 여백 합 \
         {expected:.1}px 이어야 한다(수정 전 1.9px)"
    );
}

#[test]
fn page_bottom_table_moves_to_second_page_like_hancom() {
    let core = open(&sample_bytes());
    assert_eq!(core.page_count(), 2, "한컴 engine 2020 Print PDF 는 2쪽");
    let first = core.build_page_render_tree(0).expect("1쪽");
    let second = core.build_page_render_tree(1).expect("2쪽");
    assert!(
        find_table(&first.root, 5, 0).is_none(),
        "쪽 아래 고정 표 pi=5 는 위 글과 겹치므로 1쪽에 남지 않는다"
    );
    let footer = find_table(&second.root, 5, 0).expect("2쪽 쪽 아래 고정 표 pi=5");
    assert!(footer.bbox.height > 0.0);
}

/// 대조군: 다음 문단이 host 줄 전진 뒤에 없으면 host 줄이 표 상자 뒤에 있다는 저장 증거가 없다.
#[test]
fn host_line_without_following_stored_advance_is_not_read_as_below_the_box() {
    let bytes = sample_bytes();
    let mut doc = rhwp::parser::hwpx::parse_hwpx(&bytes).expect("parse");
    for line in &mut doc.sections[0].paragraphs[1].line_segs {
        line.vertical_pos += 500;
    }
    let saved = rhwp::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
    let core = open(&saved);
    let page = core.build_page_render_tree(0).expect("1쪽");
    let header = find_table(&page.root, 0, 2).expect("머리 표");
    let inline = find_table(&page.root, 0, 4).expect("글자처럼 취급한 표");
    let gap = inline.bbox.y - (header.bbox.y + header.bbox.height);
    assert!(
        gap < hu_to_px(852),
        "저장 등식이 깨진 문서의 표 간격이 {gap:.1}px 이다 — 머리 표 아래 여백으로 내리지 않아야 한다"
    );
}
