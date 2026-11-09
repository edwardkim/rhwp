//! 관계형: 표의 마지막 행 내용을 모두 담은 끝 조각 뒤에 빈 쪽을 만들지 않는다.
//!
//! 입력 `samples/issue8693/table-end-row-clamp.hwp`(공개 샘플 조합 합성, 저장 줄 없음):
//! 칸 선언 2000 HU 행 32개짜리 표. 선언 합(853.3px)이 남은 본문(847.9px)보다 5.4px 크고,
//! 마지막 행 내용(한 줄)은 남은 공간에 들어간다.
//! 기준 `pdf/issue8693/table-end-row-clamp-webhwp-print.pdf`(웹한글기안기 인쇄):
//! 1쪽, 마지막 행만 남은 공간(20px)으로 줄여 같은 쪽에서 끝난다.
//!
//! 수정 전 rhwp 는 끝 행을 내용 컷으로 모두 담은 조각을 낸 뒤에도 무조건 다음 쪽으로 넘어가,
//! 항목이 하나도 없는 2쪽을 만들었다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8693/table-end-row-clamp.hwp";

fn texts(node: &RenderNode, out: &mut Vec<String>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty() {
            out.push(run.text.trim().to_string());
        }
    }
    for child in &node.children {
        texts(child, out);
    }
}

fn table_bottoms(node: &RenderNode, out: &mut Vec<f64>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push(node.bbox.y + node.bbox.height);
    }
    for child in &node.children {
        table_bottoms(child, out);
    }
}

#[test]
fn completed_end_row_keeps_table_on_one_page() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("합성 재현물");
    let core = DocumentCore::from_bytes(&bytes).expect("재현물 배치");

    assert_eq!(core.page_count(), 1, "웹한글기안기 인쇄 쪽수");

    let tree = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let mut words = Vec::new();
    texts(&tree.root, &mut words);
    let joined = words.join(" ");
    assert!(joined.contains("ROW 1"), "첫 행이 1쪽에 있다");
    assert!(
        joined.contains("ROW 32"),
        "마지막 행 글이 1쪽에 있다: {joined}"
    );

    // 표는 본문 아래 경계 안에서 끝난다(마지막 행을 남은 공간으로 줄인다).
    let section = &core.document().sections[0];
    let page = &section.section_def.page_def;
    let px = |hu: i64| rhwp::renderer::hwpunit_to_px(hu as i32, rhwp::renderer::DEFAULT_DPI);
    let body_bottom = px(i64::from(page.height))
        - px(i64::from(page.margin_bottom))
        - px(i64::from(page.margin_footer));
    let mut bottoms = Vec::new();
    table_bottoms(&tree.root, &mut bottoms);
    let bottom = bottoms.into_iter().fold(f64::MIN, f64::max);
    assert!(bottom > 0.0, "1쪽에 표가 있다");
    assert!(
        bottom <= body_bottom + 0.5,
        "표 아래 끝 {bottom:.1}px 이 본문 아래 경계 {body_bottom:.1}px 안에 있다"
    );
}

// 대조군: 같은 빈 문단의 두 번째 표가 끝 행을 모두 담고 쪽을 채운 경우.
//
// 입력 `samples/issue8693/two-tables-host-line-{no-room,fits}.hwp`(같은 공개 샘플 조합, 저장 줄 없음):
// 한 빈 문단 안에 표 A(31행)와 표 B(32행×2050 HU, 선언 합이 새 쪽 본문보다 몇 px 큼).
// A 는 1쪽, B 는 2쪽 위에서 시작해 끝 행을 남은 공간으로 줄여 쪽을 채운다. 두 재현물은 A 행 높이만
// 달라 A 뒤 1쪽 잔여가 8.9px(no-room)·15.1px(fits)이고, 문단 글줄 높이는 13.3px 이다.
// 기준 `pdf/issue8693/two-tables-host-line-*-webhwp-print.pdf`(웹한글기안기 인쇄):
// no-room 은 3쪽(3쪽은 빈 쪽 — A 뒤에 글줄 자리가 없어 글줄이 B 뒤로 밀림), fits 는 2쪽.

fn page_words(core: &DocumentCore, page: u32) -> String {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut words = Vec::new();
    texts(&tree.root, &mut words);
    words.join(" ")
}

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).expect("합성 재현물");
    DocumentCore::from_bytes(&bytes).expect("재현물 배치")
}

#[test]
fn host_line_without_room_after_previous_table_keeps_trailing_page() {
    let core = load("samples/issue8693/two-tables-host-line-no-room.hwp");
    assert_eq!(core.page_count(), 3, "웹한글기안기 인쇄 쪽수");
    let p2 = page_words(&core, 1);
    assert!(
        p2.contains("B 1") && p2.contains("B 32"),
        "표 B 는 2쪽에서 끝난다: {p2}"
    );
    let p3 = page_words(&core, 2);
    assert!(p3.is_empty(), "3쪽은 빈 쪽: {p3}");
}

#[test]
fn host_line_with_room_after_previous_table_ends_on_table_page() {
    let core = load("samples/issue8693/two-tables-host-line-fits.hwp");
    assert_eq!(core.page_count(), 2, "웹한글기안기 인쇄 쪽수");
    let p2 = page_words(&core, 1);
    assert!(
        p2.contains("B 1") && p2.contains("B 32"),
        "표 B 는 2쪽에서 끝난다: {p2}"
    );
}
