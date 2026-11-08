//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8428의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:31+09:00 (`41eb40eb5a5b9082b5ab80a05e89404d3f5eb5ba`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue2004_cell_image_stack.hwp`; 독립 Print `pdf/issue2004_cell_image_stack-hwp-2020.pdf`.
//! 전체 8쪽, 전쪽 최저 Native 99.95% / fresh WASM 99.95%.
//! 주장 1쪽 raw: Native 99.99% / fresh WASM 100.00%.
//! 입력 `samples/issue2004_cell_image_stack.hwpx`; 독립 Print `pdf/issue2004_cell_image_stack-hwpx-2020.pdf`.
//! 전체 8쪽, 전쪽 최저 Native 99.95% / fresh WASM 99.95%.
//! 주장 1쪽 raw: Native 99.99% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8428_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-01T14:52:05+09:00 (새 파일).
//! 2026-11-01 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`):
//! samples/issue2004_cell_image_stack.hwp(pdf/issue2004_cell_image_stack-hwp-2020.pdf, 8쪽)
//! 1쪽 Native 99.72% / fresh WASM 99.80%(수정 전 Native 97.75%). 전 8쪽 최저 Native 99.72% /
//! fresh WASM 98.87%.
//!
//! 증거: mydocs/pr/assets/issue_8419/{native,wasm}/issue2004_cell_image_stack_silhouette.tsv.
//! 기대 폭은 HBATANG.TTF hmtx(178/1024em)와 독립 Print 의 Haansoft Batang CID 출력에서 정했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

/// HBATANG.TTF `onedotenleader` hmtx 178 / unitsPerEm 1024.
const SUPPLIER_EM: f64 = 178.0 / 1024.0;

fn runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        out.push(node);
    }
    for child in &node.children {
        runs(child, out);
    }
}

#[test]
fn batang_symbol_slot_one_dot_leader_uses_haansoft_batang_advance() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2004_cell_image_stack.hwp");
    let bytes = std::fs::read(path).expect("공개 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 파싱");
    assert_eq!(core.page_count(), 8, "독립 Print 와 같은 전 8쪽");
    let source = core.document();
    let mut checked = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽 배치");
        let mut nodes = Vec::new();
        runs(&tree.root, &mut nodes);
        for node in nodes {
            let RenderNodeType::TextRun(run) = &node.node_type else {
                unreachable!()
            };
            if !run.text.contains('\u{2024}') {
                continue;
            }
            let shape = &source.doc_info.char_shapes
                [run.char_shape_id.expect("원본 글자 모양 소속") as usize];
            // 기호 슬롯(5)의 원본 face 가 바탕인 글자만 이 계약의 대상이다.
            let symbol_face = &source.doc_info.font_faces[5][shape.font_ids[5] as usize].name;
            if symbol_face != "바탕" {
                continue;
            }
            // 실제 배치가 소비한 상자 폭을 본다. 이 원본에서 `․`는 자기 글자 모양의 단독
            // run 이므로 run 폭이 곧 그 글자의 전진(자간 포함)이다.
            assert_eq!(run.text, "\u{2024}", "단독 run 전제");
            let em = run.style.font_size * run.style.ratio;
            let expected = em * SUPPLIER_EM + run.style.letter_spacing.max(0.0);
            assert!(
                (node.bbox.width - expected).abs() <= 1.0,
                "{page}쪽: `․` 배치 폭 {:.2}px, 공급 face 기대 {expected:.2}px (1em={em:.2})",
                node.bbox.width
            );
            checked += 1;
        }
    }
    assert_eq!(
        checked, 3,
        "원본의 바탕 기호 슬롯 `․` 세 곳을 모두 확인한다"
    );
}
