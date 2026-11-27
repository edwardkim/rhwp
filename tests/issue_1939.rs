//! 관계형·고정형 회귀 — #7779의 Print 표 행 소속·겹침 방지와 정본 쪽수.
//! 주석 갱신: 2026-11-27 17:57:02 KST. 직전 변경 bde419f3e927f629bb9c863cc55a448bc4175ca1
//! (2026-07-13T22:25:38+09:00).
//! 2026-11-27 한컴 2020 Print(82쪽), 96 DPI, 색상 경계 보정 전 2px 실루엣.
//! 7779-76076-hwpx 전 82쪽 최저는 Native 98.52%, fresh WASM 98.53%였다.
//! 6쪽은 Native 99.98%, fresh WASM 99.99%였다.
//! 7쪽은 Native 100.00%, fresh WASM 100.00%였다.
//! 원본은 samples/issue1891/76076_regulatory_analysis.hwpx이다.
//! 독립 Print PDF는 pdf/issue1891/76076-exact-input-hwp2020-20261004.pdf다.
//! 전쪽 TSV는 mydocs/pr/assets/issues7777_7787/native/7779-76076-hwpx/silhouette.tsv에 있다.
//! 전쪽 TSV는 mydocs/pr/assets/issues7777_7787/wasm/7779-76076-hwpx/silhouette.tsv에 있다.
//!
//! Issue #1939 — HWP5-in-.hwpx 76076 strict render-diff 구조 안정성.
//!
//! #1936으로 페이지 수는 82쪽으로 맞았지만, `render-diff --via hwpx` strict 기준에서
//! page 38/39 경계의 TAC RowBreak 표 host 문단들이 다음 쪽으로 밀리며
//! STRUCT_MISMATCH가 남았다. 원인은 HWP5-origin HWPX export가 넣는
//! "원본 LineSeg 부재" marker가 HWPX 로드 중 TAC 표 높이 보정에 의해 실제
//! LineSeg처럼 변형되어 제거되지 않는 것이었다.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue1891/76076_regulatory_analysis.hwpx";

#[test]
fn issue_1939_hwp5_origin_hwpx_strict_render_diff_is_stable() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(SAMPLE);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));

    let diff = roundtrip_geom(&data, Via::Hwpx)
        .unwrap_or_else(|e| panic!("roundtrip_geom({SAMPLE}): {e:?}"));

    assert_eq!(
        diff.page_count_a, diff.page_count_b,
        "{SAMPLE} HWPX roundtrip 페이지 수 불일치: A={} B={}",
        diff.page_count_a, diff.page_count_b
    );
    // [#2195 정식화] PDF 정답 82 복귀 (본문 NO_LS 실폭 래핑 + anchor 회계 정합).
    assert_eq!(diff.page_count_a, 82, "{SAMPLE} 쪽수 (PDF 정답)");
    assert!(
        diff.max_disp <= 1.0,
        "{SAMPLE} HWPX roundtrip 렌더 변위 {:.2}px > 1.0px",
        diff.max_disp
    );
    for pg in &diff.pages {
        assert!(
            !pg.structure_mismatch,
            "{SAMPLE} page {} 구조 불일치 (node {} vs {}) — HWP5-origin LineSeg 부재 marker 오염 회귀",
            pg.page,
            pg.node_count_a,
            pg.node_count_b
        );
    }
}

fn visible_nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if node.visible {
        out.push(node);
        for child in &node.children {
            visible_nodes(child, out);
        }
    }
}

fn normalized_visible_text(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    visible_nodes(node, &mut nodes);
    nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.display_or_text()),
            _ => None,
        })
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn issue_7779_split_table_keeps_print_rows_above_the_footer() {
    let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("read exact HWP5 input for Hancom 2020 Print");
    let core = DocumentCore::from_bytes(&bytes).expect("parse 76076");
    assert_eq!(core.page_count(), 82, "Hancom 2020 Print page count");

    let text_owners = [
        ("구분", 6),
        ("현행유지안", 6),
        ("규제비용없음", 6),
        ("자전거를이용하는배달종사자는", 6),
        ("사업주의안전조치대상에서제외", 7),
        ("규제대안", 7),
        ("규제비용발생", 7),
    ];
    for page in [6, 7] {
        let tree = core.build_page_render_tree(page - 1).expect("render 76076");
        let mut nodes = Vec::new();
        visible_nodes(&tree.root, &mut nodes);
        let fragment = nodes
            .iter()
            .copied()
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(36))
            })
            .expect("rendered outer fragment of source paragraph 36");
        let footer = tree
            .root
            .children
            .iter()
            .find(|node| matches!(node.node_type, RenderNodeType::Footer))
            .expect("page footer");
        assert_eq!(normalized_visible_text(footer), format!("-{page}-"));
        let fragment_bottom = fragment.bbox.y + fragment.bbox.height;
        assert!(
            fragment_bottom < footer.bbox.y,
            "page {page} table before footer"
        );
        let content = normalized_visible_text(fragment);
        for (text, owner) in text_owners {
            assert_eq!(
                content.matches(text).count(),
                usize::from(page == owner),
                "page {page} source table text {text}"
            );
        }
        if page == 7 {
            let heading = nodes
                .iter()
                .copied()
                .find(|node| {
                    matches!(&node.node_type, RenderNodeType::TextLine(_))
                        && normalized_visible_text(node).contains("이해관계자의견수렴")
                })
                .expect("following body heading");
            assert!(fragment_bottom < heading.bbox.y && heading.bbox.y < footer.bbox.y);
        }
    }
}
