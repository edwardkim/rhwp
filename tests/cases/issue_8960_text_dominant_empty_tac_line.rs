//! 관계형·고정형 — [#8960] 글자 높이가 표보다 큰 빈 문단의 TAC 표는 기준선에 놓이고 그 줄은 저장 줄 높이를 점유한다.
//! 주석 갱신: 2026-11-12 06:00 KST (새 파일)
//!
//! 입력: 관악구의회 「후보자 등록 신청서 및 후보자 사퇴서」 공개 서식
//! `samples/issue8960/36439173-empty-tac-line-baseline.hwpx`(한/글 2020 저장, 2쪽).
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8960/36439173-empty-tac-line-baseline-2020.pdf`(2쪽).
//! 두 쪽 모두 「주 : 2. 한글로 …」 줄(lh 1200) → 13pt 빈 문단 한 줄(lh=th=1300, bl=1105,
//! ls=392) 안의 1×1 TAC 표(높이 482HU, 바깥 여백 0 — 회색 구분선) → 「접수 : 사무국 -」 줄이다.
//! 저장 vpos 차이는 앞 줄→구분선 줄 2160HU, 구분선 줄→「접수」 줄 1692HU(= lh 1300 + ls 392).
//! 한/글 PDF 는 구분선 아래끝을 그 줄 기준선(줄 위 + 1105HU)에 두고 「접수」 줄을 저장 vpos 에
//! 둔다. 수정 전 rhwp 는 구분선을 줄 맨 위에 두고 줄 끝을 표 아래끝으로 잘라 「접수」 줄 이하가
//! 약 9px 위로 올라왔다.
//! - 고정형: 전체 쪽수 2(= PDF).
//! - 관계형: 각 쪽에서 앞 글줄 위끝 → 구분선 위끝 = 2160 + 1105 − 482 = 2783HU, 구분선 아래끝 →
//!   「접수」 줄 위끝 = 1300 − 1105 + 392 = 587HU(각 ±1px). 저장 줄 메트릭에서 정한 값이다.
//!
//! 측정(2026-11-12, 이 파일과 같은 커밋의 코드, base = upstream/devel eb84afc036,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 2쪽 / 2쪽, p1 91.12 · p2 91.50, 최저 91.12 (base: p1 75.52 · p2 71.72)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 2쪽 / 2쪽, p1 91.35 · p2 91.62, 최저 91.35
//! - 증거: `mydocs/pr/assets/issue_8960/`(native·wasm TSV, review·overlay PNG)
//! - 남은 차이는 가로 글자 폭(공개 글꼴)이다. 구분선·「접수」 줄의 세로 위치는 PDF 와 겹친다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue8960/36439173-empty-tac-line-baseline.hwpx";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 2;
/// 구분선 표 선언 높이(482HU)의 px 상한 — 이보다 낮은 본문 표는 구분선뿐이다.
const SEPARATOR_MAX_PX: f64 = 10.0;

fn px(hu: i32) -> f64 {
    f64::from(hu) / 75.0
}

fn collect(node: &RenderNode, tables: &mut Vec<(f64, f64)>, lines: &mut Vec<f64>) {
    match &node.node_type {
        RenderNodeType::Table(_) => {
            tables.push((node.bbox.y, node.bbox.y + node.bbox.height));
            return; // 칸 안 글줄은 본문 흐름이 아니다.
        }
        RenderNodeType::TextLine(_) => lines.push(node.bbox.y),
        _ => {}
    }
    for child in &node.children {
        collect(child, tables, lines);
    }
}

#[test]
fn separator_tac_table_sits_on_baseline_and_keeps_stored_line_height() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    for page in 0..PDF_PAGES {
        let tree = core.build_page_render_tree(page).expect("render tree");
        let (mut tables, mut lines) = (Vec::new(), Vec::new());
        collect(&tree.root, &mut tables, &mut lines);
        let separators: Vec<(f64, f64)> = tables
            .iter()
            .copied()
            .filter(|(top, bottom)| bottom - top < SEPARATOR_MAX_PX)
            .collect();
        assert_eq!(
            separators.len(),
            1,
            "{}쪽 구분선 표는 한 번: tables={tables:?}",
            page + 1
        );
        let (top, bottom) = separators[0];
        let above = lines
            .iter()
            .copied()
            .filter(|y| *y < top)
            .fold(f64::MIN, f64::max);
        let below = lines
            .iter()
            .copied()
            .filter(|y| *y >= bottom)
            .fold(f64::MAX, f64::min);
        assert!(
            above > f64::MIN && below < f64::MAX,
            "{}쪽 구분선 앞뒤 글줄이 있어야 한다: lines={lines:?}",
            page + 1
        );
        assert!(
            (top - above - px(2160 + 1105 - 482)).abs() <= 1.0,
            "{}쪽 구분선 아래끝이 그 줄 기준선에 있어야 한다: 앞 줄 {above:.2} → 표 위 {top:.2}",
            page + 1
        );
        assert!(
            (below - bottom - px(1300 - 1105 + 392)).abs() <= 1.0,
            "{}쪽 「접수」 줄은 구분선 줄의 저장 줄 높이와 줄간격 뒤에 와야 한다: 표 아래 {bottom:.2} → 다음 줄 {below:.2}",
            page + 1
        );
    }
}
