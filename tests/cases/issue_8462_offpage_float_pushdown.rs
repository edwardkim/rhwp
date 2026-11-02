//! 관계형·고정형(쪽수): 세로 오프셋으로 본문 아래끝보다 아래에 놓인 문단 기준 자리차지 개체는 흐름을 밀지 않는다.
//! 주석 갱신: 2026-11-02T13:53:29+09:00 (새 파일).
//! 2026-11-02 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`),
//! 기준은 각 원본의 독립 한컴 2022 12.0.0.4547 Print(pdf/issue8462/*-2022.pdf):
//! - 156633674(HWPX, 1쪽): Native 99.89% / fresh WASM 100.00%. 수정 전 2쪽(배너만 2쪽, 측정 불가).
//!   조판만 고친 중간 단계에서는 배너가 용지 밖 로고 높이(49.8px)만큼 아래에 그려져 Native 81.11%.
//! - 156552561(HWPX, 3쪽): Native 99.30 / 99.52 / 97.77%, fresh WASM 99.34 / 99.56 / 98.21%.
//!   전 3쪽 최저 Native 97.77% / WASM 98.21%. 수정 전 마지막 빈 문단만 4쪽(측정 불가).
//!
//! 증거: mydocs/pr/assets/issue_8462/{native,wasm}/*_silhouette.tsv.
//!
//! 기대값은 저장 LINE_SEG 와 독립 Print 로 정한다: 156633674 문단 13 줄1 vpos 63821 = 줄0 끝(59135+3846+840)
//! 이라 줄0 의 오프셋 로고(42163 HU, 용지 밖)는 흐름을 밀지 않았고, 한컴은 1쪽에 끝낸다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

/// (종류, 문단, 컨트롤, 위끝, 아래끝)
type Placed = (&'static str, usize, usize, f64, f64);

fn placed(node: &RenderNode, out: &mut Vec<Placed>) {
    let (top, bottom) = (node.bbox.y, node.bbox.y + node.bbox.height);
    match &node.node_type {
        RenderNodeType::Table(t) => {
            if let (Some(p), Some(c)) = (t.para_index, t.control_index) {
                out.push(("table", p, c, top, bottom));
            }
        }
        RenderNodeType::Rectangle(r) => {
            if let (Some(p), Some(c)) = (r.para_index, r.control_index) {
                out.push(("rect", p, c, top, bottom));
            }
        }
        _ => {}
    }
    for child in &node.children {
        placed(child, out);
    }
}

fn load(sample: &str, pages: u32) -> DocumentCore {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(
        core.page_count(),
        pages,
        "{sample}: 독립 한/글 2022 Print 전체 쪽수"
    );
    core
}

/// 156633674(HWPX, 1쪽): 문단 13 줄0 담당 부서 TAC 표 + 오프셋 42163 HU 로고 글상자(ci1),
/// 줄1(vpos 63821) TAC 배너 사각형(ci2).
#[test]
fn offpage_offset_logo_does_not_push_same_paragraph_banner() {
    let sample = "samples/issue8462/156633674-offpage-logo.hwpx";
    let core = load(sample, 1);
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut items = Vec::new();
    placed(&tree.root, &mut items);
    let find = |kind: &str, ctrl: usize| {
        items
            .iter()
            .find(|i| i.0 == kind && i.1 == 13 && i.2 == ctrl)
            .copied()
            .unwrap_or_else(|| panic!("{kind} 13:{ctrl}: {items:?}"))
    };
    let table = find("table", 0);
    let banner = find("rect", 2);
    assert!(
        banner.3 >= table.4 - 0.5,
        "배너 {banner:?} 가 표 {table:?} 위로 겹쳤다"
    );
    // 배너 위끝 = 문단 상단 + 줄1 vpos 차(63821-59135), 표 위끝 = 문단 상단 + 바깥 여백 위 283.
    let stored = (63821.0 - 59135.0 - 283.0) * 96.0 / 7200.0;
    assert!(
        (banner.3 - table.3 - stored).abs() <= 1.0,
        "표 위끝→배너 위끝 {:.2} ≠ 저장 줄 간격 {stored:.2}",
        banner.3 - table.3
    );
}

/// 156552561(HWPX, 3쪽): 문단 38 오프셋 63822 HU 글상자가 흐름을 밀지 않아 마지막 문단까지 3쪽에 남는다.
#[test]
fn offpage_offset_textbox_does_not_open_an_extra_page() {
    let sample = "samples/issue8462/156552561-offpage-textbox.hwpx";
    let core = load(sample, 3);
    let tree = core.build_page_render_tree(2).expect("3쪽 render tree");
    let mut items = Vec::new();
    placed(&tree.root, &mut items);
    assert!(
        items
            .iter()
            .any(|i| i.0 == "table" && i.1 == 38 && i.2 == 1),
        "문단 38 표가 3쪽에 있다: {items:?}"
    );
}
