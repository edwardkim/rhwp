//! [#6802] 차례 칸의 점 채움 줄을 저장대로 한 줄로 두고, 넘치는 점만 끊는다.
//!
//! ## 무엇이 문제였나
//!
//! 옛 차례 표는 제목 뒤를 점 문자로 직접 채워 한 줄을 만든다
//! (`Ⅰ. 사업개요 ․․․․․…`, `․` = U+2024). 그 점 개수는 **한/글의 메트릭으로** 줄을 꽉
//! 채우도록 정해져 있어, 우리 추정 폭에서는 저장 한 줄이 셀 폭을 몇 배 넘는 것처럼 보인다.
//! 그때 `#2525`·`#2291` 이 세운 "저장 1줄이 내폭을 1.8× 넘으면 부실 저장" 규칙이 발동해
//! 한 줄짜리 문단이 네 줄로 접혔고, 접힌 줄들이 **뒤 문단의 저장 `vertical_pos` 위에**
//! 그대로 겹쳐 그려졌다.
//!
//! ```text
//!   저장 사다리(칸 0, 14문단)   p[1] vpos= 5760  p[2] vpos= 9760  p[3] vpos=13760 …
//!                                └ 문단마다 정확히 4000 HU = 한 줄
//!   수정 전 렌더                pi=1 line0..line3 (4줄) → line1 이 pi=2 의 자리에
//! ```
//!
//! ## 기대값의 출처 — 한/글 2020 정본
//!
//! `tests/fixtures/issue6802/1400000-200600006_toc_leader_fill-2020.pdf`
//! (한/글 2020 11.0.0.9136 변환) 2쪽은 차례 줄을 **한 줄**로 두고 점을 상자 안에서 끊는다.
//!
//! | 줄 | 정본 x0..x_end (pt) | rhwp 수정 후 (pt) |
//! | --- | --- | --- |
//! | `Ⅰ. 사업개요 ․․․` | 56.6 .. 479.6 | 56.0 .. 476.3 |
//! | `Ⅳ. 제안안내 및 …` | 56.6 .. 478.1 | 56.0 .. 476.4 |
//! | ` 서식1) 제안단체 현황` | 56.6 .. 216.6 | 56.0 .. 216.0 |
//!
//! 점이 쪽 번호 칸(정본 x=486.2)이나 용지 밖으로 이어지지 않는다는 것이 핵심이다 —
//! 수정 전 rhwp 는 점을 용지 오른쪽 끝(793.5px = 595pt)까지 그렸다.
//!
//! ## 잠그는 것
//!
//! 1. 차례 칸의 문단마다 줄이 **하나**다(저장 사다리와 같은 수).
//! 2. 그 줄들의 잉크가 칸 오른쪽을 넘지 않는다(점을 끊는다).
//! 3. 2쪽에 글자 겹침·쪽 밖 요소가 없다.
//!
//! 반례(`#2291` task2287 r183c8 · `#2525` hwpx-02 p5)는 채움 없이 넘치는 저장 1줄이라
//! 종전대로 재래핑된다 — 그 시험들이 이 변경 뒤에도 그대로 통과한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "tests/fixtures/issue6802/1400000-200600006_toc_leader_fill.hwp";
/// 차례 표가 있는 쪽(0 기준).
const PAGE: u32 = 1;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("픽스처")).expect("문서 로드")
}

/// 차례 칸(첫 칸)의 줄 노드와 그 줄의 글자 오른쪽 끝.
fn toc_cell_lines(root: &RenderNode) -> (f64, Vec<(f64, f64, String)>) {
    fn find_cell<'a>(node: &'a RenderNode, out: &mut Option<&'a RenderNode>) {
        if out.is_some() {
            return;
        }
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            let has_toc = collect_text(node).contains("사업개요");
            if has_toc {
                *out = Some(node);
                return;
            }
        }
        for child in &node.children {
            find_cell(child, out);
        }
    }
    fn collect_text(node: &RenderNode) -> String {
        let mut s = String::new();
        if let RenderNodeType::TextRun(run) = &node.node_type {
            s.push_str(run.display_or_text());
        }
        for child in &node.children {
            s.push_str(&collect_text(child));
        }
        s
    }
    let mut cell = None;
    find_cell(root, &mut cell);
    let cell = cell.expect("차례 칸");
    let mut lines = Vec::new();
    fn walk(node: &RenderNode, lines: &mut Vec<(f64, f64, String)>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let mut right = node.bbox.x;
            let mut text = String::new();
            fn runs(node: &RenderNode, right: &mut f64, text: &mut String) {
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    text.push_str(run.display_or_text());
                    *right = right.max(node.bbox.x + node.bbox.width);
                }
                for child in &node.children {
                    runs(child, right, text);
                }
            }
            runs(node, &mut right, &mut text);
            lines.push((node.bbox.y, right, text));
        }
        for child in &node.children {
            walk(child, lines);
        }
    }
    walk(cell, &mut lines);
    lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    (cell.bbox.x + cell.bbox.width, lines)
}

/// 저장이 한 줄로 둔 차례 문단은 한 줄로 그려진다.
#[test]
fn toc_paragraphs_keep_their_stored_single_line() {
    let core = core();
    let tree = core.build_page_render_tree(PAGE).expect("render tree");
    let (_cell_right, lines) = toc_cell_lines(&tree.root);

    let entries: Vec<&(f64, f64, String)> = lines
        .iter()
        .filter(|(_, _, t)| t.contains('\u{2024}'))
        .collect();
    assert_eq!(
        entries.len(),
        5,
        "점 채움 차례 줄은 5개여야 한다 (Ⅰ~Ⅳ·□ 첨부서식). got {:?}",
        lines
            .iter()
            .map(|l| l.2.chars().take(12).collect::<String>())
            .collect::<Vec<_>>()
    );

    // 수정 전에는 한 문단이 네 줄로 접혀 같은 y 간격이 깨졌다. 저장 사다리는 문단마다
    // 4000 HU(=53.3px)이고 차례 줄은 그 두 배(빈 문단 한 개씩 사이)마다 온다.
    for pair in entries.windows(2) {
        let gap = pair[1].0 - pair[0].0;
        assert!(
            (100.0..=112.0).contains(&gap),
            "차례 줄 간격이 저장 사다리와 다르다: {gap:.1}px"
        );
    }
}

/// 넘치는 점은 칸 안에서 끊긴다 — 쪽 번호 칸·용지 밖으로 이어지지 않는다.
#[test]
fn leader_dots_are_cut_inside_the_cell() {
    let core = core();
    let tree = core.build_page_render_tree(PAGE).expect("render tree");
    let (cell_right, lines) = toc_cell_lines(&tree.root);
    let widest = lines
        .iter()
        .filter(|(_, _, t)| t.contains('\u{2024}'))
        .map(|(_, right, _)| *right)
        .fold(0.0_f64, f64::max);

    // 불변식: 점 채움 줄의 잉크는 **자기 칸 안**에서 끝난다. 수정 전 rhwp 는 점을
    // 용지 오른쪽 끝(793.5px)까지 그려 쪽 번호 칸을 덮었다. 정본(한/글 2020) 2쪽의
    // 같은 줄은 479.6pt = 639.5px @96dpi 에서 끝난다.
    assert!(
        widest <= cell_right + 0.5,
        "점 채움이 칸 밖으로 이어진다: 오른쪽 끝 {widest:.1}px, 칸 오른쪽 {cell_right:.1}px \
         (정본 639.5px, 용지 793.7px)"
    );
    assert!(
        widest >= 600.0,
        "점을 너무 많이 끊었다: 오른쪽 끝 {widest:.1}px (정본 639.5px)"
    );
}

/// 그 쪽에 글자 겹침·쪽 밖 요소가 없다.
#[test]
fn page_has_no_overlap_or_off_canvas() {
    let core = core();
    let tree = core.build_page_render_tree(PAGE).expect("render tree");
    let anomalies = scan_page(
        PAGE,
        &tree.root,
        core.page_count(),
        &AnomalyOptions::default(),
    );
    assert!(
        anomalies.text_overlap.is_empty(),
        "글자 겹침 {}건 (수정 전 4건)",
        anomalies.text_overlap.len()
    );
    assert!(
        anomalies.off_canvas.is_empty(),
        "쪽 밖 요소 {}건 (수정 전 1건: 표가 용지를 311.7px 넘었다)",
        anomalies.off_canvas.len()
    );
}
