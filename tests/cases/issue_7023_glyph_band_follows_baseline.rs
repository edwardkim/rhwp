//! [Issue #7023] 글자겹침 검출이 em 상자를 줄 상자 **중앙**에 두는데 렌더러는
//! baseline 을 상자 아래쪽에 찍는다 — 부푼 줄 상자에서 띠가 잉크에서 멀어져
//! 진짜 겹침이 침묵한다.
//!
//! `glyph_band_bbox` 의 종전 주석은 중앙을 고른 이유로 "렌더 트리에 baseline
//! 필드가 없다" 를 적고, "방향이 한쪽(위양성 감소)이라 결함을 놓치는 쪽으로는
//! 틀리지 않는다" 고 덧붙였다. 둘 다 사실이 아니었다 — `TextRunNode::baseline`
//! 은 있고(SVG 가 그 값으로 글자를 찍는다), 중앙 가정은 줄 상자가 부풀수록 띠를
//! 잉크에서 `(h − em)/2` 만큼 떼어 놓아 **위음성을 만든다**.
//!
//! 이 시험이 잠그는 실물은 `1480000-201900042` 15쪽의 본문 ↔ 꼬리말 충돌이다.
//!
//! ```text
//!   본문   '□ 고시(안)에 대한 비용‧편익 분석 및 규제영향분석서 작성'
//!          baseline 1061.88   x  94.5 .. 515.3
//!   꼬리말 '- XI -'
//!          baseline 1064.69   x 374.9 .. 410.0
//! ```
//!
//! 세로로 2.8px 떨어져 있고 가로로는 꼬리말이 본문 줄 안에 통째로 들어간다 —
//! 본문이 쪽번호를 덮어 둘 다 읽기 어렵다(`#6920` 과 같은 결함 클래스).
//! 이 짝은 중앙 기준에서도 검출된다. baseline 변경의 음성 대조는
//! `layout_anomaly_glyph_band`의 독립적인 본문 3쌍 테스트가 담당한다.
//!
//! ⚠ 원 이슈가 든 `2769535` 2쪽은 이 수정으로도 신호가 나지 않는다. 그 쪽의
//! `마. 행정박물류` 잉크는 `x 91.6..199.8` 인데 같은 높이의 다른 글자는 `x >= 300`
//! 이라 **가로로 겹치지 않는다**. 그 이슈가 "진짜 겹침 7쌍" 이라고 센 것은 전부
//! 줄 상자 교차이고, 그건 `glyph_band_bbox` 가 없애려고 만든 아티팩트다. 그 쪽의
//! 실제 결함은 소제목이 165.4px 내려가는 `#7018` 이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::layout_anomaly::{scan_document, AnomalyOptions};
use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp";

/// 고정 physical page 96의 본문 줄은 base와 font 환경에 따라 떨어져 있을 수 있다.
/// 최초 증거에 있는 본문/꼬리말 쌍을 내용으로 찾고, 본문 3쌍의 baseline 경계는
/// layout_anomaly_glyph_band의 결정적 render-tree 계약 시험에서 별도로 보호한다.
#[test]
fn issue_7023_documented_body_footer_collision_is_detected() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let report = scan_document(&core, &AnomalyOptions::default()).expect("scan");
    fn text(node: &rhwp::renderer::render_tree::RenderNode, out: &mut String) {
        if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(run.display_or_text());
        }
        for child in &node.children {
            text(child, out);
        }
    }
    let target_page = (0..core.page_count())
        .find(|&page| {
            let tree = core.build_page_render_tree(page).expect("page tree");
            let mut content = String::new();
            text(&tree.root, &mut content);
            content.contains("규제영향분석서 작성") && content.contains("XI")
        })
        .expect("최초 실물 증거의 본문과 로마자 꼬리말이 같은 쪽에 있어야 한다");
    let page = report
        .pages
        .iter()
        .find(|p| p.page == target_page)
        .expect("documented collision page");
    assert!(
        page.text_overlap.iter().any(|o| {
            (o.path_a.contains("/Body/") && o.path_b.contains("/Footer"))
                || (o.path_b.contains("/Body/") && o.path_a.contains("/Footer"))
        }),
        "본문/꼬리말의 실제 글자 띠 겹침을 검출해야 한다: {:?}",
        page.text_overlap
    );
}
