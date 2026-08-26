//! [#5601] tac 표 셀의 재조판 배치가 저장 사다리보다 부풀어 마지막 줄이 셀
//! clip 밖으로 나가 화면(studio canvas)·SVG 모두에서 소실되던 회귀 가드.
//!
//! 00451(협약서 별지 서식, 1×1 tac 상자 + 안내 표) 실측 — 수정 전: 재조판이
//! 줄간격을 누적 +23px 부풀려 마지막 줄 `“을” : (인)` 이 y=845.4 로 셀 clip
//! (843.2) 밖에 놓였다. 이슈는 studio 전용으로 접수됐지만 SVG 도 같은 clip
//! 사슬(cell-clip)을 걸어 브라우저 표시상 잘린다 — 요소 존재만 본 오판이었다.
//! 한글 2024 PDF 실측(을 822.0)과 저장 사다리(737.7HU→825.3)는 그 줄을 셀 안에
//! 담는다. 수정: native HWP5 tac 표의 중첩-표 셀에서 저장 앵커 흐름이 셀
//! 안높이에 담기면 extent==total 이어도 저장 앵커 배치를 신뢰한다(Task #362
//! 반증은 extent 가 inner 를 넘는 형상이라 판별자에 안 걸림).

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5601/2537593_supply_agreement_form.hwp";

fn find_line(node: &RenderNode, prefix: &str) -> Option<(f64, f64)> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let text: String = node
            .children
            .iter()
            .filter_map(|c| match &c.node_type {
                RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                _ => None,
            })
            .collect();
        if text.trim_start().starts_with(prefix) {
            return Some((node.bbox.y, node.bbox.y + node.bbox.height));
        }
    }
    node.children.iter().find_map(|c| find_line(c, prefix))
}

fn cell_clip_bottom(node: &RenderNode) -> Option<f64> {
    if let RenderNodeType::TableCell(tc) = &node.node_type {
        if tc.clip && node.bbox.height > 500.0 {
            return Some(node.bbox.y + node.bbox.height);
        }
    }
    node.children.iter().find_map(cell_clip_bottom)
}

#[test]
fn issue_5601_last_line_stays_inside_cell_clip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5601 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5601 fixture");

    assert_eq!(core.page_count(), 1, "00451 은 1쪽 서식이다");

    let page = core.build_page_render_tree(0).expect("render p1");
    let (eul_top, eul_bottom) = find_line(&page.root, "“을” :").expect("마지막 줄 `“을” :`");
    let clip_bottom = cell_clip_bottom(&page.root).expect("협약서 상자 셀 clip");

    // 한글 2024 PDF 실측 822.0 / 저장 사다리 825.3 — 수정 전 rhwp 는 845.4 로
    // 셀 clip(843.2) 밖이었다.
    assert!(
        (eul_top - 825.3).abs() <= 2.0,
        "`“을”` 줄 상단은 저장 사다리 정답(825.3±2)이어야 함 (수정 전 845.4); got {eul_top:.1}"
    );

    // 제목(셀 첫 문단, sb=2000HU 선차감 스냅)도 저장 정답(114.3, 한글 113.9)에
    // 있어야 한다 — Center 셀 column-top 의 spacing_before 재가산 유실이 있으면
    // 87.6 으로 26px 위에 그려진다.
    let (title_top, _) = find_line(&page.root, "물품공급 또는 기술지원협약서").expect("제목 줄");
    assert!(
        (title_top - 114.3).abs() <= 2.0,
        "제목 줄 상단은 저장 사다리 정답(114.3±2)이어야 함 (유실 시 87.6); got {title_top:.1}"
    );
    assert!(
        eul_bottom <= clip_bottom + 0.5,
        "`“을”` 줄({eul_bottom:.1})은 셀 clip({clip_bottom:.1}) 안에 있어야 함 — \
         수정 전엔 clip 밖이라 화면·SVG 모두에서 소실됐다"
    );
}
