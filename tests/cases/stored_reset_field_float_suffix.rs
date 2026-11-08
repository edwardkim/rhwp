//! 구역 첫 빈 문단이 소유한 용지 기준 자리차지 밴드 아래에서 본문이 시작하고, 뒤 문단 P 는
//! 저장 쪽 경계(vpos=0)와 같은 줄에서 나뉜다.
//!
//! 한/글 2020 이 열어 HWP 로 저장한 합성 입력(`samples/stored_reset_float/gen_field_float_reset.py`).
//! 구역 첫 빈 문단(pi=0)이 용지 기준 자리차지 사각형 A 를 소유하고, 문단 P(pi=22)는 하이퍼링크
//! 필드로 시작해 끝 근처에 용지 기준·글 앞으로 사각형을 둔다. 저장 줄은 P 의 셋째 줄에 vpos=0
//! 경계를 둔다.
//!
//! ```text
//!   입력                          정본(한/글 2020 Print)
//!   field_float_reset             2쪽 · P 줄 0–1 은 1쪽(…낱말25), 줄 2– 는 2쪽(낱말26…)
//!   field_float_reset_plain       같음 (필드·글 앞 개체 없는 대조군)
//! ```
//!
//! pi=0 은 자기 사각형 A 의 밴드 아래에 놓인다(저장 vpos 30915 = A 하단). #7696 이 쪽·종이 기준
//! 밴드를 흐름에 계상한 뒤로 rhwp 는 pi=0 을 밴드 위에 둔 채 뒤 문단만 밀어 본문 전체가 줄 하나
//! 위에 그려지고, 두 입력 모두 P 를 줄 3에서 나눴다(정본 줄 2).
//!
//! 이 입력은 밴드가 바르게 계상되면 흐름만으로도 저장 경계와 같은 줄에서 나뉜다. 따라서 필드·경계
//! 뒤 개체가 저장 경계 수용을 막는 판정(#8124)의 검출 근거는 아니다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const P: u64 = 22;

fn core(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse")
}

/// 쪽마다 (항목 종류, 문단, 시작 줄, 끝 줄).
fn items(core: &DocumentCore) -> Vec<Vec<(String, u64, Option<u64>, Option<u64>)>> {
    let pages = core.dump_page_items_json(None);
    pages
        .as_array()
        .unwrap()
        .iter()
        .map(|page| {
            page["columns"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|column| column["items"].as_array().unwrap().iter())
                .map(|item| {
                    (
                        item["kind"].as_str().unwrap_or_default().to_string(),
                        item["paraIndex"].as_u64().unwrap_or(u64::MAX),
                        item["startLine"].as_u64(),
                        item["endLine"].as_u64(),
                    )
                })
                .collect()
        })
        .collect()
}

fn text(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

#[test]
fn paragraph_after_section_paper_band_splits_at_stored_line() {
    for sample in [
        "samples/stored_reset_float/field_float_reset.hwp",
        "samples/stored_reset_float/field_float_reset_plain.hwp",
    ] {
        let core = core(sample);
        assert_eq!(core.page_count(), 2, "{sample}: 정본 2쪽");
        let pages = items(&core);
        let p_parts: Vec<_> = pages
            .iter()
            .enumerate()
            .flat_map(|(page, items)| {
                items
                    .iter()
                    .filter(|(kind, para, ..)| *para == P && kind.ends_with("aragraph"))
                    .map(move |(kind, _, start, end)| (page, kind.clone(), *start, *end))
            })
            .collect();
        assert_eq!(
            p_parts,
            vec![
                (0, "partialParagraph".to_string(), Some(0), Some(2)),
                (1, "partialParagraph".to_string(), Some(2), Some(6)),
            ],
            "{sample}: P 는 저장 경계(줄 2)에서 나뉜다"
        );
        assert!(
            pages[1]
                .iter()
                .any(|(kind, para, ..)| kind == "fullParagraph" && *para == P + 1),
            "{sample}: P 뒤 문단은 2쪽"
        );
        let page_text = |page: u32| text(&core.build_page_render_tree(page).expect("page").root);
        let (first, second) = (page_text(0), page_text(1));
        assert!(
            first.contains("낱말25") && !first.contains("낱말26"),
            "{sample}: 1쪽은 낱말25 에서 끝난다"
        );
        assert!(
            second.contains("낱말26")
                && second.contains("뒤낱말08")
                && second.contains("P 뒤 문단"),
            "{sample}: 2쪽은 낱말26 부터"
        );
    }
    // 경계 뒤 글 앞으로 사각형은 뒤 조각과 같은 쪽에 놓인다.
    let pages = items(&core("samples/stored_reset_float/field_float_reset.hwp"));
    assert!(
        pages[1]
            .iter()
            .any(|(kind, para, ..)| kind == "shape" && *para == P),
        "경계 뒤 사각형은 2쪽 — {pages:?}"
    );
}

/// 앵커 문단 자신도 자기 쪽·종이 기준 자리차지 밴드 아래에서 시작한다. 그 뒤 문단 pi=1 의 첫 글줄은
/// 사각형 A 하단에서 pi=0 의 저장 줄 한 칸(줄 높이 + 줄 간격) 아래다(한/글 2020 Print 정본).
#[test]
fn section_first_paragraph_starts_below_its_own_paper_band() {
    for sample in [
        "samples/stored_reset_float/field_float_reset.hwp",
        "samples/stored_reset_float/field_float_reset_plain.hwp",
    ] {
        let core = core(sample);
        let owner = &core.document().sections[0].paragraphs[0].line_segs[0];
        let pitch = rhwp::renderer::hwpunit_to_px(owner.line_height + owner.line_spacing, 96.0);
        let tree = core.build_page_render_tree(0).expect("page");
        let mut rects = Vec::new();
        let mut lines = Vec::new();
        fn walk(node: &RenderNode, rects: &mut Vec<(f64, f64)>, lines: &mut Vec<(f64, String)>) {
            match &node.node_type {
                RenderNodeType::Rectangle(_) => {
                    rects.push((node.bbox.y, node.bbox.y + node.bbox.height))
                }
                RenderNodeType::TextLine(_) => lines.push((node.bbox.y, text(node))),
                _ => {}
            }
            for child in &node.children {
                walk(child, rects, lines);
            }
        }
        walk(&tree.root, &mut rects, &mut lines);
        assert_eq!(rects.len(), 1, "{sample}: 1쪽 사각형은 A 하나 — {rects:?}");
        let band_bottom = rects[0].1;
        let first_body = lines
            .iter()
            .find(|(_, line)| line.contains("field-float"))
            .unwrap_or_else(|| panic!("{sample}: pi=1 글줄 — {lines:?}"));
        assert!(
            (first_body.0 - (band_bottom + pitch)).abs() <= 1.0,
            "{sample}: pi=1 은 A 하단 {band_bottom:.1} + pi=0 줄 한 칸 {pitch:.1} 에서 시작 — {:.1}",
            first_body.0
        );
        assert!(
            lines.iter().all(|(top, _)| *top >= band_bottom - 0.5),
            "{sample}: 본문 글줄은 A 밴드에 들어가지 않는다 — {lines:?}"
        );
    }
}
