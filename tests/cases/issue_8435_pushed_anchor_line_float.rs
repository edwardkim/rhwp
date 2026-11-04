//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8437의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:44+09:00 (`66151761f274f52fd9dfbea0e5c75509c68e3539`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8435/156618114-logo-textbox-offset.hwpx`; 독립 Print `pdf/issue8435/156618114-logo-textbox-offset-2022.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 99.19% / fresh WASM 99.10%.
//! 주장 2쪽 raw: Native 99.95% / fresh WASM 99.95%.
//! 입력 `samples/issue8435/156638890-logo-textbox.hwpx`; 독립 Print `pdf/issue8435/156638890-logo-textbox-2022.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 99.95% / fresh WASM 99.95%.
//! 주장 3쪽 raw: Native 99.95% / fresh WASM 99.95%.
//! 입력 `samples/issue8435/156770078-logo-table.hwpx`; 독립 Print `pdf/issue8435/156770078-logo-table-2022.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 99.88% / fresh WASM 99.96%.
//! 주장 3쪽 raw: Native 99.98% / fresh WASM 99.98%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8437_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-01T23:18:07+09:00 (새 파일).
//! 2026-11-01 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`),
//! 기준은 각 원본의 독립 한컴 2022 12.0.0.4547 Print(pdf/issue8435/*-2022.pdf):
//! - 156638890(3쪽): 3쪽 Native 99.95% / fresh WASM 99.95%(수정 전 Native 44.02%), 2쪽 Native 99.99%
//!   (수정 전 90.39%). 전 3쪽 최저 Native 99.95% / WASM 99.95%.
//! - 156770078(3쪽): 3쪽 Native 99.95% / fresh WASM 99.98%(수정 전 Native 42.20%).
//!   전 3쪽 최저 Native 99.47% / WASM 99.98%.
//! - 156618114(2쪽): 2쪽 Native 99.95% / fresh WASM 99.95%(수정 전 Native 51.40%), 1쪽 Native 99.11%
//!   (수정 전 92.81%). 전 2쪽 최저 Native 99.11% / WASM 99.12%.
//!
//! 증거: mydocs/pr/assets/issue_8435/{native,wasm}/*_silhouette.tsv.
//! 기대 간격은 구현이 아닌 저장 줄 사다리(줄0 끝과 줄1 vpos 의 차이 = 개체 상자)와 독립 Print 로 정한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

/// (종류, 문단, 컨트롤, 위끝, 아래끝)
type Placed = (&'static str, Option<usize>, Option<usize>, f64, f64);

fn load(sample: &str) -> DocumentCore {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(
        core.page_count(),
        3,
        "{sample}: 독립 한/글 2022 Print 전체 쪽수"
    );
    core
}

fn placed(node: &RenderNode, out: &mut Vec<Placed>) {
    let (top, bottom) = (node.bbox.y, node.bbox.y + node.bbox.height);
    match &node.node_type {
        RenderNodeType::Table(t) => out.push(("table", t.para_index, t.control_index, top, bottom)),
        RenderNodeType::Rectangle(r) => {
            out.push(("rect", r.para_index, r.control_index, top, bottom))
        }
        RenderNodeType::Image(i) => out.push(("image", i.para_index, i.control_index, top, bottom)),
        _ => {}
    }
    for child in &node.children {
        placed(child, out);
    }
}

fn page_items(core: &DocumentCore, page: u32) -> Vec<Placed> {
    let tree = core.build_page_render_tree(page).expect("쪽 render tree");
    let mut out = Vec::new();
    placed(&tree.root, &mut out);
    out
}

fn one(items: &[Placed], kind: &str, para: usize, ctrl: usize) -> Placed {
    let found: Vec<_> = items
        .iter()
        .filter(|i| i.0 == kind && i.1 == Some(para) && i.2 == Some(ctrl))
        .copied()
        .collect();
    assert_eq!(found.len(), 1, "{kind} {para}:{ctrl} 하나: {items:?}");
    found[0]
}

/// 156638890 문단 29: 줄0 TAC 담당 부서 표, 줄1(vpos 10895) 로고 글상자.
/// 줄0 끝 6210+452 = 6662 과 줄1 의 차이 4233 이 글상자 높이와 같다.
#[test]
fn pushed_anchor_line_textbox_stays_below_its_table_on_the_new_page() {
    let core = load("samples/issue8435/156638890-logo-textbox.hwpx");
    // 리셋된 문단과 함께 3쪽에 놓인다 — 떠나는 2쪽 말미로 흘리지 않는다.
    let page2 = page_items(&core, 1);
    assert!(
        !page2.iter().any(|i| i.1 == Some(29)),
        "문단 29 의 개체가 2쪽에 남았다: {page2:?}"
    );
    let page3 = page_items(&core, 2);
    let table = one(&page3, "table", 29, 0);
    let rect = one(&page3, "rect", 29, 1);
    // 앵커 줄 자리(표 줄 끝) 아래에서 시작하고, 표와 겹치지 않는다.
    assert!(
        rect.3 >= table.4 - 0.5,
        "글상자 {rect:?} 가 표 {table:?} 위로 겹쳤다"
    );
    let gap = rect.3 - table.4;
    // 저장 줄0 의 표 바깥 여백 아래(283)와 줄 간격(452)이 둘 사이를 정한다.
    let stored_gap = (283.0 + 452.0) * 96.0 / 7200.0;
    assert!(
        (gap - stored_gap).abs() <= 1.0,
        "표 아래 간격 {gap:.2} ≠ 저장 간격 {stored_gap:.2}"
    );
}

/// 156770078 문단 29: 줄0 TAC 담당 부서 표, 줄1(vpos 8558) 로고 표(바깥 여백 283).
/// 줄0 끝 3448+780 = 4228 과 줄1 의 차이 4330 = 283 + 3764 + 283.
#[test]
fn pushed_anchor_line_table_is_painted_below_the_tac_table() {
    let core = load("samples/issue8435/156770078-logo-table.hwpx");
    let page3 = page_items(&core, 2);
    let contact = one(&page3, "table", 29, 0);
    let logo = one(&page3, "table", 29, 1);
    assert!(
        logo.3 >= contact.4 - 0.5,
        "로고 표 {logo:?} 가 담당 부서 표 {contact:?} 위에 겹쳤다"
    );
    // 로고 표 상단 = 담당 부서 표 바깥 여백 아래(283) + 줄 간격(780) + 로고 표 바깥 여백 위(283).
    let stored_gap = (283.0 + 780.0 + 283.0) * 96.0 / 7200.0;
    let gap = logo.3 - contact.4;
    assert!(
        (gap - stored_gap).abs() <= 1.0,
        "표 사이 간격 {gap:.2} ≠ 저장 간격 {stored_gap:.2}"
    );
    // 로고 그림은 모두 로고 표 안에 있다.
    for image in page3.iter().filter(|i| i.0 == "image") {
        assert!(
            image.3 >= logo.3 - 0.5 && image.4 <= logo.4 + 0.5,
            "그림 {image:?} 가 로고 표 {logo:?} 밖에 있다"
        );
    }
}

/// 156618114 문단 14: 줄0 TAC 담당 부서 표(lh 3130, ls 700), 줄1(vpos 9316) 로고 글상자
/// (세로 오프셋 825, 높이 4661). 줄0 끝 3830 과 줄1 의 차이 5486 = 825 + 4661.
/// 오프셋도 앵커 줄 자리 기준이다 — 문단 상단 기준이면 글상자가 표를 덮는다.
#[test]
fn pushed_anchor_line_offset_textbox_keeps_its_offset_below_the_table() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue8435/156618114-logo-textbox-offset.hwpx"),
    )
    .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(core.page_count(), 2, "독립 한/글 2022 Print 전체 쪽수");
    let page1 = page_items(&core, 0);
    assert!(
        !page1.iter().any(|i| i.1 == Some(14)),
        "문단 14 의 개체가 1쪽 말미에 남았다: {page1:?}"
    );
    let page2 = page_items(&core, 1);
    let table = one(&page2, "table", 14, 0);
    let rect = one(&page2, "rect", 14, 1);
    let gap = rect.3 - table.4;
    // 표 바깥 아래 여백 283(줄 높이 3130 = 283 + 2564 + 283) + 줄 간격 700 + 세로 오프셋 825.
    let stored_gap = (283.0 + 700.0 + 825.0) * 96.0 / 7200.0;
    assert!(
        (gap - stored_gap).abs() <= 1.0,
        "표 아래 간격 {gap:.2} ≠ 저장 간격 {stored_gap:.2}"
    );
}
