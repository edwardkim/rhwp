//! [#7330] 구현 속성(bit31) 합성 줄 문단 뒤의 저장 vpos 는 **전방 이동의 근거가 아니다** —
//! 점프 크기와 무관하다.
//!
//! 결재문서를 외부 도구로 가림(***) 처리하면 가린 문단의 줄 수가 줄고 그 LINE_SEG 에 bit31 이
//! 붙지만, 뒤 문단의 저장 vpos 는 옛 줄 수 기준으로 남는다. 한/글은 그 간격을 재현하지 않는다.
//! 종전(#1811 v2)에는 48px 넘는 전방 점프를 "구조적 재앵커"로 허용해 뒤 본문이 옛 위치로 밀렸다.
//!
//! 공개 합성 입력(`samples/masked_stale_vpos/make_masked_stale_vpos.py`):
//! - `unmasked_hancom_saved.hwp`: 생성기 원문을 한/글 2020(hwp2024Convert MCP engine 2020)이
//!   저장한 것. pi=3 은 4줄, pi=4 저장 vpos 11200.
//! - `masked_stale_vpos.hwpx`: 그 저장본(HWPX)의 pi=3 을 `*` 12자·1줄(bit31)로 가린 것. pi=4 저장
//!   vpos 는 옛 값 11200 그대로(실제 끝 6400 보다 64px 아래). 정본
//!   `pdf/masked_stale_vpos/masked_stale_vpos-2020.pdf` 는 pi=4 를 pi=3 바로 다음 줄에 둔다.
//!
//! 기대값은 한/글 저장 줄 간격(대조 원문)과 정본 PDF 의 배치다. 절대 픽셀은 쓰지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const MASKED: &str = "samples/masked_stale_vpos/masked_stale_vpos.hwpx";
const ORIGINAL: &str = "samples/masked_stale_vpos/unmasked_hancom_saved.hwp";
const HU_PER_PX: f64 = 75.0;

fn lines(sample: &str) -> Vec<(usize, f64, i32)> {
    fn walk(node: &RenderNode, out: &mut Vec<(usize, f64, i32)>) {
        if let RenderNodeType::TextLine(tl) = &node.node_type {
            if let (Some(para), Some(vpos)) = (tl.para_index, tl.vpos) {
                out.push((para, node.bbox.y, vpos));
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    assert_eq!(core.page_count(), 1, "{sample}: 1쪽");
    let tree = core.build_page_render_tree(0).expect("render 1쪽");
    let mut out = Vec::new();
    walk(&tree.root, &mut out);
    out
}

fn first_line(lines: &[(usize, f64, i32)], para: usize) -> (f64, i32) {
    lines
        .iter()
        .find(|(p, _, _)| *p == para)
        .map(|(_, y, v)| (*y, *v))
        .unwrap_or_else(|| panic!("pi={para} 줄이 있어야 한다"))
}

#[test]
fn masked_paragraph_does_not_push_successor_to_stale_vpos() {
    let lines = lines(MASKED);
    let (y1, v1) = first_line(&lines, 1);
    let (y2, v2) = first_line(&lines, 2);
    let (y3, _) = first_line(&lines, 3);
    let (y4, v4) = first_line(&lines, 4);
    // 같은 문단 모양의 한 줄 문단 간격(한/글 저장 pi1→pi2).
    let pitch = f64::from(v2 - v1) / HU_PER_PX;
    assert!((y2 - y1 - pitch).abs() < 0.5, "가림 앞 저장 간격 유지");
    let stale = f64::from(v4) / HU_PER_PX - f64::from(v2) / HU_PER_PX - pitch;
    assert!(
        stale > 48.0,
        "입력 전제: pi=4 저장 vpos 가 48px 넘게 앞선다 ({stale:.1}px)"
    );
    assert!(
        (y4 - y3 - pitch).abs() < 1.0,
        "가린 1줄 문단 바로 다음 줄에 pi=4 가 온다(정본): y4-y3={:.1}, pitch={pitch:.1}",
        y4 - y3
    );
    // 뒤 문단은 pi=4 에서 저장 간격대로 이어진다.
    for para in 5..=7 {
        let (y, v) = first_line(&lines, para);
        let expected = y4 + f64::from(v - v4) / HU_PER_PX;
        assert!(
            (y - expected).abs() < 1.0,
            "pi={para}: y={y:.1}, expected={expected:.1}"
        );
    }
}

#[test]
fn unmasked_original_keeps_hancom_stored_ladder() {
    // 대조: bit31 없는 한/글 저장본은 저장 vpos 그대로(pi=3 4줄 뒤 pi=4).
    let lines = lines(ORIGINAL);
    let (y0, v0) = first_line(&lines, 0);
    assert!(
        lines.iter().filter(|(p, _, _)| *p == 3).count() >= 3,
        "원문 pi=3 은 여러 줄"
    );
    for &(para, y, v) in &lines {
        let expected = y0 + f64::from(v - v0) / HU_PER_PX;
        assert!(
            (y - expected).abs() < 1.0,
            "pi={para}: y={y:.1}, 저장 {expected:.1}"
        );
    }
}
