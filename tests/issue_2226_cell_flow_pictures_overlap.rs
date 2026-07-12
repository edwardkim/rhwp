//! Issue #2226: 셀 내 겹침 배치 다중 그림의 flow 합산 — 주보 p2 로고 표.
//!
//! `samples/basic/issue1994_behindtext_table_20200830.hwp` p2 우측 하단 로고 표
//! (2×3, tac 어울림): 셀[2]의 그림 3개(TopAndBottom×2+Square, 오프셋 겹침)와
//! rowspan 셀[0]의 로고(Square)가 측정·wrap 회계에서 직렬 합산/para_top 이중
//! 계상되어 행높이가 1.9× 팽창(r0 131.3/r1 141.5px) → 표 하단이 페이지 밖
//! (865px)으로 밀려 2행 주소 블록("04538 서울특별시…")이 소실됐다.
//!
//! 정정: ①wrap bottom 의 para_top 을 "개체가 줄 위를 채운" 문단에서 사다리
//! 기반 문단 시작으로 ②측정 저장 흐름 신뢰(stored extent < additive 합)
//! ③밀림-빈문단 그림 앵커를 문단 시작으로 (한컴 편집기 조판부호 스크린샷 =
//! 1차 정답지: 행 무성장·2행 온전).

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

fn walk_tables(n: &RenderNode, out: &mut Vec<(usize, f64, f64, f64)>) {
    if let RenderNodeType::Table(t) = &n.node_type {
        if let Some(pi) = t.para_index {
            out.push((pi, n.bbox.y, n.bbox.height, n.bbox.x));
        }
    }
    for c in &n.children {
        walk_tables(c, out);
    }
}

fn collect_text(n: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &n.node_type {
        out.push_str(&run.text);
    }
    for c in &n.children {
        collect_text(c, out);
    }
}

fn has_text(n: &RenderNode, probe: &str) -> bool {
    let mut t = String::new();
    collect_text(n, &mut t);
    t.contains(probe)
}

#[test]
fn issue_2226_footer_table_rows_match_declared_and_address_visible() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue1994_behindtext_table_20200830.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue1994_behindtext_table_20200830.hwp");
    let tree = doc.build_page_render_tree(1).expect("render page 2");

    // 로고 표 = 문단 0.31 호스트. 선언 총높이 11223HU = 149.6px (+행간 보정 ≤ 3px).
    let mut tables = Vec::new();
    walk_tables(&tree.root, &mut tables);
    let (pi, y, h, _) = *tables
        .iter()
        .find(|(pi, _, _, x)| *pi == 31 && *x > 560.0)
        .expect("p2 우측 로고 표(pi=31)를 찾지 못함");
    assert!(
        (h - 149.6).abs() <= 3.0,
        "로고 표(pi={pi}) 높이 {h:.1}px — 선언 149.6px 이탈 (#2226 회귀: 팽창 시 272.7)"
    );
    assert!(
        y + h < 748.0,
        "로고 표 하단 {:.1} 이 페이지 본문 하한을 넘음 — #2226 회귀 (수정 전 865)",
        y + h
    );

    // 2행 주소 블록 텍스트가 실제 방출된다 (수정 전: 페이지 밖 소실).
    for probe in ["04538", "hyanglin", "776"] {
        assert!(
            has_text(&tree.root, probe),
            "주소 블록 텍스트 {probe:?} 미방출 — #2226 회귀"
        );
    }
}
