//! 수학 기호 보충 블록 글자는 기호 슬롯 글꼴의 전진폭으로 잰다.
//!
//! 합성 입력 `samples/lang_slot/symbol_slot_bullets.hwpx`(생성기 `gen_lang_slot_probe.py`)는
//! 한글 슬롯 맑은 고딕 · 기호 슬롯 함초롬바탕인 글자 모양으로 `⦁`(U+2981)·`⟨`(U+27E8)·
//! `⨀`(U+2A00) 뒤에 한글 문장을 적는다. 한/글 2020·2024 Print 는 세 글자를 모두 기호 슬롯
//! 글꼴 HCRBatang 으로 그리고 전진폭은 9.72pt @ 10pt = **0.972em** 이다(함초롬바탕 hmtx 970/1000).
//! 맑은 고딕에는 세 글자가 없다. 수정 전 rhwp 는 한글 슬롯으로 재 반각(0.5em)을 주었다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/lang_slot/symbol_slot_bullets.hwpx";
const SYMBOLS: [char; 3] = ['\u{2981}', '\u{27E8}', '\u{2A00}'];
const FONT_PX: f64 = 10.0 * 96.0 / 72.0;

fn runs_with<'a>(node: &'a RenderNode, ch: char, out: &mut Vec<&'a TextRunNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(ch) {
            out.push(run);
        }
    }
    for child in &node.children {
        runs_with(child, ch, out);
    }
}

#[test]
fn math_symbol_block_uses_symbol_slot_face_and_advance() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse");
    assert_eq!(core.page_count(), 1, "한/글 Print 1쪽");
    let page = core.build_page_render_tree(0).expect("page");
    let layout: serde_json::Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).expect("layout"))
            .expect("layout JSON");
    let layout_runs = layout["runs"].as_array().expect("runs");

    for symbol in SYMBOLS {
        let mut runs = Vec::new();
        runs_with(&page.root, symbol, &mut runs);
        assert_eq!(runs.len(), 1, "U+{:04X} 를 담은 run 하나", symbol as u32);
        assert!(
            runs[0].style.font_family.starts_with("함초롬바탕"),
            "U+{:04X} 는 기호 슬롯 글꼴로 그린다 — {}",
            symbol as u32,
            runs[0].style.font_family
        );

        let run = layout_runs
            .iter()
            .find(|run| {
                run["text"]
                    .as_str()
                    .is_some_and(|text| text.contains(symbol))
            })
            .expect("공개 문자 경계 run");
        let text: Vec<char> = run["text"].as_str().unwrap().chars().collect();
        let x: Vec<f64> = run["charX"]
            .as_array()
            .expect("charX")
            .iter()
            .map(|v| v.as_f64().expect("문자 경계"))
            .collect();
        let at = text.iter().position(|&c| c == symbol).unwrap();
        let advance_em = (x[at + 1] - x[at]) / FONT_PX;
        assert!(
            (advance_em - 0.972).abs() <= 0.02,
            "U+{:04X} 전진폭 {advance_em:.3}em — 한/글 0.972em",
            symbol as u32
        );
    }
}
