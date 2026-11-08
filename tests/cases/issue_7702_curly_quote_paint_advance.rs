//! 같은 원본의 독립 한컴 Print에서 휴먼명조 따옴표는 전각 글리프이고
//! Times 따옴표는 좁은 원 글리프다. 측정이 그 글리프의 폭을 소비한 뒤에는
//! paint에서 다시 절반으로 축소하지 않는다. 절대 위치나 px 기준을 고정하지 않는다.
//! 근거: samples/issue7702/curly_quote_latin_slot.hwpx 및 대응 2020 Print PDF.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::layout::{forces_halfwidth_punct_paint, EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::TextStyle;
const QUOTES: [char; 4] = ['‘', '’', '“', '”'];
fn style(face: &str, trusted: bool) -> TextStyle {
    TextStyle {
        font_family: face.to_owned(),
        font_metric_trusted: trusted,
        font_size: 20.0,
        ratio: 1.0,
        ..Default::default()
    }
}
#[test]
fn true_type_quotes_keep_their_program_glyphs_in_paint() {
    let measurer = EmbeddedTextMeasurer;
    for quote in QUOTES {
        let human = style("휴먼명조", true);
        let times = style("Times New Roman", true);
        let text = format!("{quote}가");
        let human_positions = measurer.compute_char_positions(&text, &human);
        let times_positions = measurer.compute_char_positions(&text, &times);
        assert!(
            human_positions[1] > times_positions[1],
            "독립 Print의 전각 휴먼명조 따옴표는 좁은 Times 따옴표보다 넓다"
        );
        for face in ["휴먼명조", "HumanMyeongJo", "Times New Roman"] {
            assert!(
                !forces_halfwidth_punct_paint(&style(face, true), quote),
                "{face} {quote}: 이미 원 글리프 폭으로 잰 따옴표를 다시 축소하면 안 된다"
            );
        }
    }
}
#[test]
fn legacy_compatibility_quotes_keep_their_narrow_paint() {
    for quote in QUOTES {
        assert!(
            forces_halfwidth_punct_paint(&style("휴먼명조", false), quote),
            "검증된 TTF 프로그램 선택이 없는 호환 폭은 좁은 paint를 유지한다"
        );
    }
}

#[test]
fn document_quote_runs_preserve_the_verified_program_through_paint() {
    use rhwp::document_core::DocumentCore;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    use std::collections::BTreeSet;
    fn walk(node: &RenderNode, seen: &mut BTreeSet<(String, char)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            let face = run
                .style
                .font_family
                .split(',')
                .next()
                .unwrap()
                .trim_matches(['\'', '"', ' ']);
            if matches!(face, "휴먼명조" | "Times New Roman") {
                for quote in run.text.chars().filter(|c| QUOTES.contains(c)) {
                    assert!(
                        run.style.font_metric_trusted,
                        "독립 Print와 같은 TTF 프로그램을 선택한 {face} 따옴표의 신뢰를 보존한다"
                    );
                    assert!(
                        !forces_halfwidth_punct_paint(&run.style, quote),
                        "실제 문서의 {face} {quote} 원 글리프는 다시 축소하지 않는다"
                    );
                    seen.insert((face.to_owned(), quote));
                }
            }
        }
        for child in &node.children {
            walk(child, seen);
        }
    }
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7702/curly_quote_latin_slot.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    let mut seen = BTreeSet::new();
    for page in 0..core.page_count() {
        walk(&core.build_page_render_tree(page).unwrap().root, &mut seen);
    }
    for face in ["휴먼명조", "Times New Roman"] {
        for quote in QUOTES {
            assert!(
                seen.contains(&(face.to_owned(), quote)),
                "{face} {quote} 소유 run 보존"
            );
        }
    }
}
