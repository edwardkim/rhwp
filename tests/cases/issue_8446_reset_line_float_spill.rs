//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8448의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:52+09:00 (`1e4e320bd84a05de20ee434a49500a9c59548d47`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8446/156569782-reset-line-logo-empty.hwp`; 독립 Print `pdf/issue8446/156569782-reset-line-logo-empty-2022.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 99.87% / fresh WASM 99.91%.
//! 주장 1쪽 raw: Native 99.87% / fresh WASM 99.91%.
//! 입력 `samples/issue8446/156667426-reset-line-logo.hwp`; 독립 Print `pdf/issue8446/156667426-reset-line-logo-2022.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 96.75% / fresh WASM 96.77%.
//! 주장 2쪽 raw: Native 96.75% / fresh WASM 96.77%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8448_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-02T11:00:53+09:00 (새 파일).
//! 2026-11-02 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`),
//! 기준은 각 원본의 독립 한컴 2022 12.0.0.4547 Print(pdf/issue8446/*-2022.pdf):
//! - 156667426(HWP, 3쪽): 로고 1쪽 Native 99.76% / fresh WASM 99.92%, 2쪽 96.75% / 96.80%, 3쪽 99.83% / 99.83%.
//!   전 3쪽 최저 Native 96.75% / WASM 96.80%. 수정 전 로고만 2쪽으로 넘어가 4쪽(측정 불가).
//! - 156569782(HWP, 2쪽): 로고 1쪽 Native 99.67% / fresh WASM 99.91%, 빈 줄 2쪽 100.00% / 100.00%.
//!   수정 전 1쪽 96.09%, 2쪽(로고만) 0.00%.
//!
//! 증거: mydocs/pr/assets/issue_8446/{native,wasm}/*_silhouette.tsv.
//!
//! 기대값은 저장 LINE_SEG 와 독립 Print 로 정한다: 줄1 vpos 가 0 으로 되감겨 그 줄은 다음 쪽 첫머리지만,
//! 한컴은 줄1 에 앵커된 비-TAC 로고를 떠나는 쪽의 표 아래(줄0 끝)에 두고 줄1 의 나머지만 넘긴다.
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

fn load(sample: &str, pages: u32) -> (DocumentCore, Vec<Vec<Placed>>) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(
        core.page_count(),
        pages,
        "{sample}: 독립 한/글 2022 Print 전체 쪽수"
    );
    let per_page = (0..pages)
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("쪽 render tree");
            let mut items = Vec::new();
            placed(&tree.root, &mut items);
            items
        })
        .collect();
    (core, per_page)
}

fn pages_of(per_page: &[Vec<Placed>], kind: &str, para: usize, ctrl: usize) -> Vec<usize> {
    per_page
        .iter()
        .enumerate()
        .filter(|(_, items)| {
            items
                .iter()
                .any(|i| i.0 == kind && i.1 == para && i.2 == ctrl)
        })
        .map(|(page, _)| page)
        .collect()
}

fn one(items: &[Placed], kind: &str, para: usize, ctrl: usize) -> Placed {
    let found: Vec<_> = items
        .iter()
        .filter(|i| i.0 == kind && i.1 == para && i.2 == ctrl)
        .copied()
        .collect();
    assert_eq!(found.len(), 1, "{kind} {para}:{ctrl} 하나: {items:?}");
    found[0]
}

/// 담당 부서 표(줄0)와 로고(줄1)가 같은 쪽, 로고 위끝 = 문단 상단 + 줄0 전진(lh+ls).
/// 표 위끝 = 문단 상단 + 표 바깥 여백 위.
fn assert_logo_stays_under_table(
    sample: &str,
    per_page: &[Vec<Placed>],
    para: usize,
    leaving: usize,
    line0_advance_hu: f64,
    table_margin_top_hu: f64,
) {
    assert_eq!(
        pages_of(per_page, "rect", para, 1),
        vec![leaving],
        "{sample}: 로고는 떠나는 쪽에만"
    );
    assert_eq!(
        pages_of(per_page, "table", para, 0),
        vec![leaving],
        "{sample}: 담당 부서 표는 떠나는 쪽"
    );
    let table = one(&per_page[leaving], "table", para, 0);
    let logo = one(&per_page[leaving], "rect", para, 1);
    assert!(
        logo.3 >= table.4 - 0.5,
        "{sample}: 로고 {logo:?} 가 표 {table:?} 위로 겹쳤다"
    );
    let stored = (line0_advance_hu - table_margin_top_hu) * 96.0 / 7200.0;
    assert!(
        (logo.3 - table.3 - stored).abs() <= 1.0,
        "{sample}: 표 위끝→로고 위끝 {:.2} ≠ 저장 줄0 전진 {stored:.2}",
        logo.3 - table.3
    );
}

/// 156667426(HWP): 문단 16 줄0(vpos 62589, lh 3130, ls 300) 담당 부서 표, 줄1(vpos 0) 로고 + 「붙임」 표,
/// 줄2 본문 표.
#[test]
fn reset_line_logo_stays_on_leaving_page_and_line_rest_moves() {
    let sample = "samples/issue8446/156667426-reset-line-logo.hwp";
    let (_core, per_page) = load(sample, 3);
    assert_logo_stays_under_table(sample, &per_page, 16, 0, 3130.0 + 300.0, 283.0);
    // 줄1 의 「붙임」 표와 줄2 표는 리셋대로 다음 쪽(2쪽)에서 시작한다 — 로고만 있는 쪽은 없다.
    assert_eq!(
        pages_of(&per_page, "table", 16, 2),
        vec![1],
        "{sample}: 「붙임」 표는 2쪽"
    );
    assert_eq!(
        pages_of(&per_page, "table", 16, 3),
        vec![1],
        "{sample}: 줄2 표는 2쪽"
    );
}

/// 156569782(HWP): 문단 15 줄0(vpos 62340, lh 3696, ls 780) 담당 부서 표, 줄1(vpos 0) 로고만.
/// 한컴 2쪽은 그 빈 줄 하나뿐인 쪽이다 — 로고를 남겨도 쪽은 열린다.
#[test]
fn reset_line_logo_stays_on_leaving_page_even_when_line_is_empty() {
    let sample = "samples/issue8446/156569782-reset-line-logo-empty.hwp";
    let (_core, per_page) = load(sample, 2);
    assert_logo_stays_under_table(sample, &per_page, 15, 0, 3696.0 + 780.0, 283.0);
    assert!(
        per_page[1].iter().all(|i| i.1 != 15),
        "{sample}: 2쪽에 문단 15 개체가 없어야 한다: {:?}",
        per_page[1]
    );
}
