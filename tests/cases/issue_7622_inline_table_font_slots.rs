//! Native Hancom-edited HWPX and matching Print use Latin Times New Roman for
//! the two curly quotes in a mixed Hangul/TAC-table paragraph. The original
//! text, source slots and genuine stored rows are retained in the fixture.
//! Native/fresh WASM whole-page Visual Sweep: 99.01152% on product 5538efd38.
//! See samples/issue7622/README.md and pr_7622_review.md for independent Print
//! provenance, before/after actual output and the limits of this derivative.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue7622/curly_quote_tac_native.hwpx";

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("native saved fixture"))
        .expect("native saved HWPX")
}

fn host_and_faces(core: &DocumentCore) -> (usize, String, String) {
    let doc = core.document();
    let hosts: Vec<_> = doc.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, para)| para.text.contains('‘') && para.text.contains('’'))
        .collect();
    assert_eq!(hosts.len(), 1, "the independent quote host must exist once");
    let (index, para) = hosts[0];
    assert!(
        para.controls.iter().any(|control| {
            matches!(control, Control::Table(table) if table.common.treat_as_char)
        }),
        "exercise the legacy inline-table paint path"
    );
    assert!(!para.stored_text_partition_is_dirty());
    assert!(!para.line_segs.is_empty());
    assert!(
        para.line_segs.iter().all(|line| {
            line.tag & rhwp::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        }),
        "native saved rows, not manual synthetic layout"
    );
    let shape_id = para.char_shape_id_at(0).expect("declared host shape");
    for (offset, ch) in para.text.chars().enumerate() {
        if ['가', '‘', '나', '’'].contains(&ch) {
            assert_eq!(
                para.char_shape_id_at(offset),
                Some(shape_id),
                "exercise a language-slot change inside one char-shape ID"
            );
        }
    }
    let shape = &doc.doc_info.char_shapes[shape_id as usize];
    let hangul = doc.doc_info.font_faces[0][shape.font_ids[0] as usize]
        .name
        .clone();
    let latin = doc.doc_info.font_faces[1][shape.font_ids[1] as usize]
        .name
        .clone();
    assert_eq!(latin, "Times New Roman", "Print: TimesNewRomanPSMT");
    assert_ne!(hangul, latin, "the fixture must distinguish the two slots");
    (index, hangul, latin)
}

fn body_faces(node: &RenderNode, host: usize, targets: &[char], out: &mut Vec<(char, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(host) && run.cell_context.is_none() {
            let face = run
                .style
                .font_family
                .split(',')
                .next()
                .unwrap_or_default()
                .trim()
                .trim_matches(['\'', '"'])
                .to_owned();
            out.extend(
                run.text
                    .chars()
                    .filter(|ch| targets.contains(ch))
                    .map(|ch| (ch, face.clone())),
            );
        }
    }
    for child in &node.children {
        body_faces(child, host, targets, out);
    }
}

#[test]
fn inline_table_quotes_use_the_independent_print_latin_face() {
    let core = load();
    let (host, _, latin) = host_and_faces(&core);
    assert_eq!(core.page_count(), 1, "whole independent Print page");
    let tree = core
        .build_page_render_tree(0)
        .expect("actual final paint tree");
    let mut faces = Vec::new();
    body_faces(&tree.root, host, &['‘', '’'], &mut faces);
    assert_eq!(
        faces.iter().map(|(ch, _)| *ch).collect::<Vec<_>>(),
        ['‘', '’'],
        "preserve order and exactly one copy of each quote"
    );
    for (ch, face) in faces {
        assert_eq!(
            face, latin,
            "{ch}: independent Print and declared Latin slot"
        );
    }
}

#[test]
fn inline_table_hangul_keeps_its_own_source_face() {
    let core = load();
    let (host, hangul, _) = host_and_faces(&core);
    let tree = core
        .build_page_render_tree(0)
        .expect("actual final paint tree");
    let mut faces = Vec::new();
    body_faces(&tree.root, host, &['가', '나'], &mut faces);
    assert_eq!(
        faces.iter().map(|(ch, _)| *ch).collect::<Vec<_>>(),
        ['가', '나'],
        "normal Hangul control is neither omitted nor duplicated"
    );
    for (ch, face) in faces {
        assert_eq!(face, hangul, "{ch}: Latin repair must preserve Hangul");
    }
}
