//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8444의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:50+09:00 (`8eaf7e441a922c4efaccfabcc3f62e4a02af0b24`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8441/156578788-tac-logo-line.hwp`; 독립 Print `pdf/issue8441/156578788-tac-logo-line-2022.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 99.57% / fresh WASM 99.56%.
//! 주장 1쪽 raw: Native 99.95% / fresh WASM 99.95%.
//! 입력 `samples/issue8441/156669499-tac-logo-line.hwpx`; 독립 Print `pdf/issue8441/156669499-tac-logo-line-2022.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 99.96% / fresh WASM 99.98%.
//! 주장 1쪽 raw: Native 99.96% / fresh WASM 99.98%.
//! 입력 `samples/issue8441/156715172-tac-table-picture.hwpx`; 독립 Print `pdf/issue8441/156715172-tac-table-picture-2022.pdf`.
//! 전체 4쪽, 전쪽 최저 Native 99.09% / fresh WASM 99.10%.
//! 주장 4쪽 raw: Native 99.09% / fresh WASM 99.10%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8444_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-02T09:11:47+09:00 (새 파일).
//! 2026-11-02 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`),
//! 기준은 각 원본의 독립 한컴 2022 12.0.0.4547 Print(pdf/issue8441/*-2022.pdf):
//! - 156578788(HWP, 2쪽): 로고 1쪽 Native 98.14% / fresh WASM 98.79%, 2쪽 99.59% / 99.58%.
//!   전 2쪽 최저 Native 98.14% / WASM 98.79%. 수정 전에는 로고만 2쪽으로 넘어가 3쪽(측정 불가).
//! - 156669499(HWPX, 1쪽): 1쪽 Native 99.80% / fresh WASM 99.98%. 수정 전 2쪽(측정 불가).
//! - 156715172(HWPX, 4쪽): 포스터 4쪽 Native 98.48% / fresh WASM 98.44%. 전 4쪽 최저
//!   Native 98.48% / WASM 98.44%. 수정 전 포스터만 5쪽으로 넘어가 5쪽(측정 불가), 쪽수만 고친
//!   중간 단계에서는 4쪽 Native 39.32%(그림 y 이중 계상) → 76.28%(내어쓰기 x 누락).
//!
//! 증거: mydocs/pr/assets/issue_8441/{native,wasm}/*_silhouette.tsv.
//!
//! 기대값은 구현이 아닌 저장 LINE_SEG 와 독립 Print 로 정한다: 2번째 줄 vpos+lh 가 쪽 본문 높이
//! (70020 HU) 이하이므로 한컴은 같은 쪽에 둔다(156578788 66269+3736=70005, 156669499 66076+3736=69812).
//! 줄 뒤 간격(840·652)은 다음 줄과의 거리라 쪽 끝 줄의 수용 판정에 들어가지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

/// (종류, 문단, 컨트롤, 위끝, 아래끝, 왼끝)
type Placed = (&'static str, usize, usize, f64, f64, f64);

fn placed(node: &RenderNode, out: &mut Vec<Placed>) {
    let (top, bottom, left) = (node.bbox.y, node.bbox.y + node.bbox.height, node.bbox.x);
    match &node.node_type {
        RenderNodeType::Table(t) => {
            if let (Some(p), Some(c)) = (t.para_index, t.control_index) {
                out.push(("table", p, c, top, bottom, left));
            }
        }
        RenderNodeType::Image(i) => {
            if let (Some(p), Some(c)) = (i.para_index, i.control_index) {
                out.push(("image", p, c, top, bottom, left));
            }
        }
        RenderNodeType::Rectangle(r) => {
            if let (Some(p), Some(c)) = (r.para_index, r.control_index) {
                out.push(("rect", p, c, top, bottom, left));
            }
        }
        _ => {}
    }
    for child in &node.children {
        placed(child, out);
    }
}

fn verify(sample: &str, pages: u32, host: usize) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(
        core.page_count(),
        pages,
        "{sample}: 독립 한/글 2022 Print 전체 쪽수"
    );
    let mut owners = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽 render tree");
        let mut items = Vec::new();
        placed(&tree.root, &mut items);
        let hosted: Vec<_> = items.into_iter().filter(|i| i.1 == host).collect();
        if !hosted.is_empty() {
            owners.push((page, hosted));
        }
    }
    // 담당 부서 표(컨트롤 0)와 로고 글상자(컨트롤 1)는 한 쪽에만, 같은 쪽에 있다.
    assert_eq!(
        owners.len(),
        1,
        "{sample}: 문단 {host} 이 여러 쪽에 나뉨: {owners:?}"
    );
    let (page, hosted) = &owners[0];
    let tables: Vec<_> = hosted
        .iter()
        .filter(|i| i.0 == "table" && i.2 == 0)
        .collect();
    let logos: Vec<_> = hosted
        .iter()
        .filter(|i| i.0 == "rect" && i.2 == 1)
        .collect();
    assert_eq!(
        tables.len(),
        1,
        "{sample} 쪽 {page}: 담당 부서 표 하나: {hosted:?}"
    );
    assert_eq!(
        logos.len(),
        1,
        "{sample} 쪽 {page}: 로고 글상자 하나: {hosted:?}"
    );
    let (table, logo) = (tables[0], logos[0]);
    // 로고 줄은 표 줄 아래에 있고 표와 겹치지 않는다.
    assert!(
        logo.3 >= table.4 - 0.5,
        "{sample}: 로고 {logo:?} 가 표 {table:?} 위로 겹쳤다"
    );
    // 로고 아래끝이 그 쪽 본문 영역 안에 있다.
    let layout = core.build_page_render_tree(*page).expect("쪽 render tree");
    let body_bottom = page_body_bottom(&layout.root).expect("본문 영역");
    assert!(
        logo.4 <= body_bottom + 0.5,
        "{sample}: 로고 아래끝 {:.2} 이 본문 아래끝 {body_bottom:.2} 을 넘었다",
        logo.4
    );
}

fn page_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(page_body)
}

fn page_body_bottom(node: &RenderNode) -> Option<f64> {
    page_body(node).map(|body| body.bbox.y + body.bbox.height)
}

/// 156578788(HWP): 문단 18 줄0 담당 부서 표, 줄1(vpos 66269, lh 3736, ls 840) 로고 글상자.
#[test]
fn tac_logo_line_fits_page_end_by_line_height_hwp() {
    verify("samples/issue8441/156578788-tac-logo-line.hwp", 2, 18);
}

/// 156669499(HWPX): 문단 21 줄0 담당 부서 표, 줄1(vpos 66076, lh 3736, ls 652) 로고 글상자.
#[test]
fn tac_logo_line_fits_page_end_by_line_height_hwpx() {
    verify("samples/issue8441/156669499-tac-logo-line.hwpx", 1, 21);
}

/// 156715172(HWPX): 문단 47 줄0 「붙임 2」 TAC 표(바깥 여백 위 283), 줄1(vpos 4480, lh 65019) TAC 포스터 그림.
/// 줄1 끝 4480+65019 = 69499 ≤ 70020 이라 표와 같은 4쪽에 있고, 그림 줄 위치는 저장 사다리
/// (문단 상단 + 4480)다. 표가 그림 앞 줄을 이미 진행시켰다고 그 줄 높이를 다시 더하지 않는다.
#[test]
fn tac_picture_line_after_tac_table_keeps_stored_line_origin() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue8441/156715172-tac-table-picture.hwpx"),
    )
    .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(core.page_count(), 4, "독립 한/글 2022 Print 전체 쪽수");
    let tree = core.build_page_render_tree(3).expect("4쪽 render tree");
    let mut items = Vec::new();
    placed(&tree.root, &mut items);
    let table = items
        .iter()
        .find(|i| i.0 == "table" && i.1 == 47 && i.2 == 0)
        .copied()
        .unwrap_or_else(|| panic!("4쪽 「붙임 2」 표: {items:?}"));
    let picture = items
        .iter()
        .find(|i| i.0 == "image" && i.1 == 47 && i.2 == 1)
        .copied()
        .unwrap_or_else(|| panic!("4쪽 포스터 그림: {items:?}"));
    assert!(
        picture.3 >= table.4 - 0.5,
        "그림 {picture:?} 가 표 {table:?} 위로 겹쳤다"
    );
    // 표 위끝 = 문단 상단 + 바깥 여백 위 283, 그림 위끝 = 문단 상단 + 저장 줄1 vpos 4480.
    let stored_gap = (4480.0 - 283.0) * 96.0 / 7200.0;
    let gap = picture.3 - table.3;
    assert!(
        (gap - stored_gap).abs() <= 1.0,
        "표 위끝→그림 위끝 {gap:.2} ≠ 저장 줄 간격 {stored_gap:.2}"
    );
    // 그림 줄은 둘째 줄이라 내어쓰기(저장 indent -5908, 해석 -2954 HU)만큼 들어간다.
    let body_left = page_body(&tree.root).expect("본문 영역").bbox.x;
    let hanging = 2954.0 * 96.0 / 7200.0;
    assert!(
        (picture.5 - body_left - hanging).abs() <= 1.0,
        "그림 왼끝 {:.2} ≠ 본문 왼끝 {body_left:.2} + 내어쓰기 {hanging:.2}",
        picture.5
    );
    let body_bottom = page_body_bottom(&tree.root).expect("본문 영역");
    assert!(
        picture.4 <= body_bottom + 0.5,
        "그림 아래끝 {:.2} 이 본문 아래끝 {body_bottom:.2} 을 넘었다",
        picture.4
    );
}
