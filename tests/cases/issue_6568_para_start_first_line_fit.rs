//! [Issue #6568] 쪽 꼬리 적합이 본문 하한을 넘는 줄을 그 쪽에 들여보내던 결함의 가드.
//!
//! 줄 분할 루프의 넘침 판정에는 `li > cursor_line` 면제가 걸려 있어 **조각의 첫 줄은
//! 검사조차 되지 않는다.** 그래서 예산을 넘는 줄이 무조건 그 쪽에 놓였다.
//!
//! 실측 — `samples/issue6542/156678235_mid_para_vpos_rewind.hwp` `pi=59`:
//!
//! ```text
//! 진입 검사   remaining 22.12 >= first_line_h 18.67          → 통과(쪽 안 넘김)
//! 루프 예산   avail_for_lines 18.12 < line_heights[0] 18.67  → 첫 줄이 안 들어감
//! 저장 사다리 ls[0] vpos 68896(=918.6px) + lh 1400 = 937.3px > 본문 하한 933.6px
//! ```
//!
//! 두 지점의 예산이 layout drift 마진(4.0px)만큼 어긋난다. 교정은 **저장 사다리에
//! 직접 묻는 것** — 첫 줄의 *바닥*이 본문 하한 밖이면 한/글은 그 줄을 이 쪽에 두지
//! 않은 것이다(`hwp_first_line_before_reset_fits` 의 거울).
//!
//! 한/글 2024 배분: 6쪽 마지막 글줄 `준비된 보험사…`(734.50pt), 7쪽 첫 글줄
//! `금융당국과 업계는…`(70.60pt). 쪽수는 7 로 불변이고 한 줄만 옮겨간다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6542/156678235_mid_para_vpos_rewind.hwp";

/// 6쪽 조판 누계가 본문 하한(933.6px)을 넘어서는 안 된다.
#[test]
fn page_tail_rejects_line_that_overflows_body_bottom() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    let page = core.build_page_render_tree(5).expect("6쪽 render tree");
    let body = page
        .root
        .children
        .iter()
        .find(|c| {
            matches!(
                c.node_type,
                rhwp::renderer::render_tree::RenderNodeType::Body { .. }
            )
        })
        .expect("6쪽 Body");
    let body_bottom = body.bbox.y + body.bbox.height;

    // 쪽번호(footer)는 본문 밖이므로 Body 아래 자식만 본다.
    let mut deepest: f64 = 0.0;
    fn walk(n: &rhwp::renderer::render_tree::RenderNode, deepest: &mut f64) {
        if matches!(
            n.node_type,
            rhwp::renderer::render_tree::RenderNodeType::TextRun(_)
        ) {
            *deepest = deepest.max(n.bbox.y + n.bbox.height);
        }
        for c in &n.children {
            walk(c, deepest);
        }
    }
    walk(body, &mut deepest);

    assert!(
        deepest <= body_bottom + 1.0,
        "6쪽 글줄이 본문 하한을 넘었다 — #6568 회귀. 최하단 {deepest:.2} > 하한 {body_bottom:.2}"
    );
}

/// 옮겨간 줄은 7쪽 **첫 줄**이어야 한다 — 한/글과 같은 배분.
#[test]
fn moved_line_starts_page_seven() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    fn first_text(core: &DocumentCore, page: u32) -> String {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|_| panic!("{}쪽 render tree", page + 1));
        let mut runs: Vec<(f64, f64, String)> = Vec::new();
        fn walk(n: &rhwp::renderer::render_tree::RenderNode, out: &mut Vec<(f64, f64, String)>) {
            if let rhwp::renderer::render_tree::RenderNodeType::TextRun(r) = &n.node_type {
                out.push((n.bbox.y, n.bbox.x, r.text.clone()));
            }
            for c in &n.children {
                walk(c, out);
            }
        }
        walk(&tree.root, &mut runs);
        runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let top = runs.first().map(|r| r.0).unwrap_or(0.0);
        runs.iter()
            .filter(|r| (r.0 - top).abs() < 2.0)
            .map(|r| r.2.as_str())
            .collect::<String>()
            .replace(' ', "")
    }

    let p7 = first_text(&core, 6);
    assert!(
        p7.starts_with("금융당국과업계는"),
        "7쪽 첫 줄은 `금융당국과 업계는…` 이어야 한다 — #6568 회귀. got {p7:?}"
    );
}
