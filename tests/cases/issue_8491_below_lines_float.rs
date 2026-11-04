//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8498의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:42:09+09:00 (`c20a88326037bf49ef914eb0eacf5d4e73744e63`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8491/156602236-press-logo.hwpx`; 독립 Print `pdf/issue8491/156602236-press-logo-2022.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 99.51% / fresh WASM 99.48%.
//! 주장 1쪽 raw: Native 99.91% / fresh WASM 99.94%.
//! 입력 `samples/issue8491/156736331-press-logo.hwpx`; 독립 Print `pdf/issue8491/156736331-press-logo-2022.pdf`.
//! 전체 5쪽, 전쪽 최저 Native 92.45% / fresh WASM 93.05%.
//! 주장 2쪽 raw: Native 99.95% / fresh WASM 99.99%.
//! 주장 5쪽 raw: Native 92.45% / fresh WASM 93.05%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8498_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03T04:10:00+09:00 (새 파일, 직전 파일 커밋 없음).
//! 2026-11-03 측정 raw(`silhouette_raw_match_percent`, 96dpi, visual_sweep --silhouette-only,
//! 코드 267e4deb9d Native skia 03:48+09:00 / fresh WASM 03:49+09:00):
//! - 156602236(HWPX, 한컴 3쪽 = rhwp 3쪽): Native 99.47/99.45/99.67%, fresh WASM
//!   100.00/99.31/99.99%; 전3쪽 최저 Native 99.45% / WASM 99.31%.
//! - 156736331(HWPX, 한컴 5쪽 = rhwp 5쪽): Native 99.31/99.96/99.98/99.92/92.45%, fresh WASM
//!   99.69/99.99/100.00/99.93/93.08%; 전5쪽 최저 Native 92.45% / WASM 93.08%(5쪽).
//! - 수정 전(base ea86816b53 와 같은 src) Native: 156602236 4쪽(2쪽 5.94%), 156736331 6쪽(3쪽 3.34%).
//!
//! 근거: mydocs/pr/assets/issue_8491/{native,wasm}/*_silhouette.tsv.
//!
//! 문단 자기 저장 줄들 아래에서 시작하는 문단 기준 자리차지 개체는 그 줄들을 밀지 않는다(#8491).
//!
//! 보도자료 끝 문단 하나에 「담당 부서」 TAC 표(줄 0)와 정책브리핑·공공누리 로고 상자(비-TAC
//! 자리차지 사각형, 세로=문단 양수 오프셋)가 함께 있다. 로고 상자 위끝은 표 줄 아래끝보다 아래다.
//! 한/글 2022 Print 는 표와 로고 상자를 같은 쪽에 두고 로고 상자 아래끝이 본문 아래끝을 조금
//! 넘어도 그 쪽 아래 여백에 그린다. 수정 전 rhwp 는 로고 상자 높이를 문단 머리에서 먼저 흐름에
//! 더해 같은 문단의 표를 다음 쪽으로 넘겼다(쪽이 하나 늘었다).
//!
//! 기대값은 저장 LINE_SEG·개체 속성과 독립 한컴 Print(`pdf/issue8491/*-2022.pdf`)로 정한다.
//! - 156602236(HWPX, 한컴 3쪽): 문단 14 줄0 vpos 64117 lh 3564, 사각형 세로 오프셋 4138 HU,
//!   표 바깥 여백 위 0.
//! - 156736331(HWPX, 한컴 5쪽): 문단 24 줄0 vpos 64856 lh 5261, 사각형 세로 오프셋 6280 HU,
//!   표 바깥 여백 위 283.

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

fn hu(v: f64) -> f64 {
    v * 96.0 / 7200.0
}

/// 표와 로고 상자가 같은 쪽에 있고, 로고 상자가 표 아래에 오프셋 그대로 놓이는지 본다.
fn check(sample: &str, pages: u32, para: usize, offset_hu: f64, table_outer_top_hu: f64) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(
        core.page_count(),
        pages,
        "{sample}: 독립 한/글 2022 Print 전체 쪽수"
    );
    let mut found = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render tree");
        let mut items = Vec::new();
        placed(&tree.root, &mut items);
        for item in items.into_iter().filter(|i| i.1 == para) {
            found.push((page, item));
        }
    }
    let find = |kind: &str, ctrl: usize| {
        found
            .iter()
            .find(|(_, i)| i.0 == kind && i.2 == ctrl)
            .copied()
            .unwrap_or_else(|| panic!("{sample}: {kind} {para}:{ctrl} 없음 {found:?}"))
    };
    let (rect_page, rect) = find("rect", 0);
    let (table_page, table) = find("table", 1);
    assert_eq!(
        rect_page, table_page,
        "{sample}: 같은 문단의 표와 로고 상자는 같은 쪽(한컴 Print)"
    );
    assert!(
        rect.3 >= table.4 - 0.5,
        "{sample}: 로고 상자 {rect:?} 가 표 {table:?} 위로 겹쳤다"
    );
    let expected = hu(offset_hu - table_outer_top_hu);
    assert!(
        (rect.3 - table.3 - expected).abs() <= 1.0,
        "{sample}: 표 위끝→로고 상자 위끝 {:.2} ≠ 저장 오프셋 차 {expected:.2}",
        rect.3 - table.3
    );
}

#[test]
fn press_logo_below_contact_table_stays_on_table_page() {
    check(
        "samples/issue8491/156602236-press-logo.hwpx",
        3,
        14,
        4138.0,
        0.0,
    );
    check(
        "samples/issue8491/156736331-press-logo.hwpx",
        5,
        24,
        6280.0,
        283.0,
    );
}
