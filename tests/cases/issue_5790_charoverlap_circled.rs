//! [#5790] 글자겹침(`hp:compose`)의 원문자가 렌더에서 맨 숫자로 풀리던 결함의 회귀 가드.
//!
//! `circleType="CHAR"`(테두리 없음) + `composeText="③"` 은 한컴이 전각 한 칸에 `③`
//! **한 글자**로 찍는다. 그런데 렌더 경로는 원문자 U+2460~U+2473 을 조건 없이 안쪽
//! 숫자로 풀어 썼다. 테두리를 그리는 겹침에서는 원 글리프가 이중으로 나가는 걸 막는
//! 정당한 처리지만, 테두리를 **안 그리는** `border_type=0` 에서는 그려 줄 동그라미가
//! 아무 데도 없어 `③` 이 맨 `3` 으로 나갔다 — 항목 번호가 본문 숫자로 보인다.
//!
//! 파서는 원문자를 그대로 보존하므로(`export-text` 는 `③` 을 낸다) 렌더 경로만의
//! 결함이다. 이 파일은 그 대조(파서 계약 / 렌더 계약)를 함께 고정한다.
//!
//! 재현 문서: `samples/issue1880_takeplace_oracle_p13.hwpx`
//! (`<hp:compose circleType="CHAR" charSz="-4" composeType="OVERLAP" composeText="③">`
//! 와 `④` 두 건)

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::model::control::Control;
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, TextSourceTable};
use rhwp::parser::parse_document;
use rhwp::renderer::composer::char_overlap_display_text;
use rhwp::renderer::layer_renderer::LayerRenderer;
use rhwp::renderer::svg_layer::SvgLayerRenderer;
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue1880_takeplace_oracle_p13.hwpx";

fn read_sample() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// 실제 글자겹침 leaf와 그 source-bound fallback·clip·layer를 함께 남긴다.
/// 본문의 다른 원문자가 검사 대상 글리프의 누락을 가리지 않도록 한다.
fn retain_overlap_leaves(node: &mut LayerNode) -> bool {
    match &mut node.kind {
        LayerNodeKind::Leaf { ops } => ops
            .iter()
            .any(|op| matches!(op, PaintOp::CharOverlap { .. })),
        LayerNodeKind::Group { children, .. } => {
            children.retain_mut(retain_overlap_leaves);
            !children.is_empty()
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            retain_overlap_leaves(child)
        }
    }
}

fn all_char_overlap_glyphs() -> Vec<String> {
    let doc = HwpDocument::from_bytes(&read_sample()).expect("parse sample");
    let mut out = Vec::new();
    for page in 0..doc.page_count() {
        let mut tree = doc.build_page_layer_tree(page).expect("실제 paint tree");
        if !retain_overlap_leaves(&mut tree.root) {
            continue;
        }
        // 원래 source ID는 유지하고 제거한 본문의 표 항목만 함께 제거한다.
        // 실제 replay의 canonical source 검증을 우회하지 않는다.
        tree.text_sources = TextSourceTable::from_layer_node(&tree.root);
        let mut renderer = SvgLayerRenderer::new();
        renderer
            .render_page(&tree)
            .expect("글자겹침 canonical SVG replay");
        let svg = roxmltree::Document::parse(renderer.output()).expect("유효한 SVG XML");
        out.extend(
            svg.descendants()
                .filter(|node| node.is_element() && node.tag_name().name() == "text")
                .map(|node| {
                    node.descendants()
                        .filter(|child| child.is_text())
                        .filter_map(|child| child.text())
                        .collect::<String>()
                }),
        );
    }
    out
}

/// 대조군 — 파서는 원문자를 그대로 IR 에 남긴다. 렌더 결함이 여기서 온 게 아님을 고정한다.
#[test]
fn issue_5790_parser_keeps_circled_compose_text() {
    let doc = parse_document(&read_sample()).expect("parse sample");
    let overlaps: Vec<(Vec<char>, u8)> = doc
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            Control::CharOverlap(co) => Some((co.chars.clone(), co.border_type)),
            _ => None,
        })
        .collect();

    assert!(
        overlaps.contains(&(vec!['③'], 0u8)) && overlaps.contains(&(vec!['④'], 0u8)),
        "composeText 원문자와 circleType=CHAR(테두리 없음)이 IR 에 그대로 남아야 한다: {overlaps:?}"
    );
}

/// 렌더 계약 — 테두리를 안 그리는 겹침은 `composeText` 를 그대로 그린다.
///
/// 결함 시절 출력: `③`/`④` 대신 `3`/`4` (동그라미도 없고 원문자도 없음).
#[test]
fn issue_5790_svg_draws_circled_char_not_bare_digit() {
    let glyphs = all_char_overlap_glyphs();
    assert_eq!(glyphs.len(), 2, "실제 겹침 두 건의 누락·중복: {glyphs:?}");

    assert!(
        glyphs.iter().any(|g| g == "③") && glyphs.iter().any(|g| g == "④"),
        "테두리 없는 글자겹침은 원문자 그대로 그려야 한다: {glyphs:?}"
    );
    assert!(
        !glyphs.iter().any(|g| g == "3" || g == "4"),
        "원문자를 맨 숫자로 풀면 동그라미가 사라진다 (#5790): {glyphs:?}"
    );
}

/// 표시 문자열 규칙 자체의 단위 고정.
///
/// 테두리를 그릴 때만 안쪽 숫자로 푼다 — 그래야 `#4085`/`#1101` 의 테두리 있는 겹침
/// (`samples/hwpx/k-water-rfp.hwpx` 반전 사각형)에서 원 글리프가 이중으로 나가지 않는다.
#[test]
fn issue_5790_display_text_unwraps_only_when_border_is_drawn() {
    assert_eq!(char_overlap_display_text('③', false), "③");
    assert_eq!(char_overlap_display_text('③', true), "3");
    assert_eq!(char_overlap_display_text('⑳', false), "⑳");
    assert_eq!(char_overlap_display_text('⑳', true), "20");
    // 원문자 범위 밖은 테두리 여부와 무관하게 그대로.
    assert_eq!(char_overlap_display_text('장', false), "장");
    assert_eq!(char_overlap_display_text('장', true), "장");
}
