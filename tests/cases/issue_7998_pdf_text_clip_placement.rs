//! Source-local clipping must follow actual content movement and HF save ownership.
//! Independent PDF operators and visual evidence: pdf/issue7998/text-clip-placement.md.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{
        control::Control,
        header_footer::Footer,
        paragraph::Paragraph,
        shape::{HorzRelTo, TextBox, VertRelTo},
        text_clip::{self, ClipCommand, TextClipPath},
    },
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use std::{fs, path::PathBuf};

const CASES: [&str; 9] = [
    "header-hole",
    "footer-top-hole",
    "footer-center-hole",
    "footer-bottom-hole",
    "footer-no-room-hole",
    "footer-bottom-curve",
    "footer-bottom-parent",
    "footer-bottom-unclipped",
    "header-unclipped",
];
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/text-clip-placement")
}
fn load(key: &str) -> DocumentCore {
    DocumentCore::from_bytes(&fs::read(root().join(format!("{key}.hwpx"))).unwrap()).unwrap()
}
fn hf_paras(core: &DocumentCore) -> &[Paragraph] {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Header(h) => Some(h.paragraphs.as_slice()),
            Control::Footer(h) => Some(h.paragraphs.as_slice()),
            _ => None,
        })
        .unwrap()
}
fn textbox(core: &DocumentCore) -> &TextBox {
    hf_paras(core)[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Shape(s) => s.drawing()?.text_box.as_ref(),
            _ => None,
        })
        .unwrap()
}
fn owner(n: &RenderNode) -> Option<&RenderNode> {
    if n.paint_clip.is_some() {
        Some(n)
    } else {
        n.children.iter().find_map(owner)
    }
}
fn render_text(n: &RenderNode) -> String {
    let own = match &n.node_type {
        RenderNodeType::TextRun(t) => t.text.clone(),
        _ => String::new(),
    };
    own + &n.children.iter().map(render_text).collect::<String>()
}
fn check(core: &DocumentCore, clipped: bool, edited: bool) {
    assert_eq!(core.page_count(), 1);
    assert_eq!(textbox(core).paragraphs[0].text.trim(), "MERGED");
    let tree = core.build_page_render_tree(0).unwrap();
    let text: String = render_text(&tree.root)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    // The host paragraph and its floating shape have separate paint order.
    // Verify their actual owners instead of inventing a flattened reading order.
    assert_eq!(
        text.replace('X', ""),
        "MERGED",
        "hidden source remains uniquely editable"
    );
    assert_eq!(text.matches('X').count(), usize::from(edited));
    assert_eq!(
        hf_paras(core)[0].text.matches('X').count(),
        usize::from(edited)
    );
    let local = &textbox(core).paint_clip;
    assert_eq!(local.is_some(), clipped);
    assert_eq!(owner(&tree.root).is_some(), clipped);
    if let Some(paths) = local {
        let actual = owner(&tree.root).unwrap();
        // These authored controls have zero inner margins. Compare source-local
        // geometry with its real content origin, not a past absolute px result.
        let expected: Vec<TextClipPath> = paths
            .iter()
            .map(|p| p.map(|x, y| (actual.bbox.x + x / 75., actual.bbox.y + y / 75.)))
            .collect();
        let observed = actual.paint_clip.as_ref().unwrap();
        assert_eq!(observed.len(), expected.len());
        for (observed, expected) in observed.iter().zip(&expected) {
            assert_eq!(observed.even_odd, expected.even_odd);
            assert_eq!(observed.commands.len(), expected.commands.len());
            for (a, b) in observed.commands.iter().zip(&expected.commands) {
                let values = |c: &ClipCommand| match *c {
                    ClipCommand::Move(x, y) | ClipCommand::Line(x, y) => vec![x, y],
                    ClipCommand::Cubic(a, b, c, d, e, f) => vec![a, b, c, d, e, f],
                    ClipCommand::Close => vec![],
                };
                assert_eq!(std::mem::discriminant(a), std::mem::discriminant(b));
                assert!(
                    values(a)
                        .iter()
                        .zip(values(b))
                        .all(|(x, y)| (x - y).abs() < 1e-8),
                    "final clipping must use the actual moved content origin"
                );
            }
        }
    }
}

#[test]
fn header_footer_clips_preserve_source_geometry_and_both_real_save_adapters() {
    for key in CASES {
        let mut core = load(key);
        let clipped = !key.contains("unclipped");
        check(&core, clipped, false);
        core.insert_text_in_header_footer_native(0, key.starts_with("header"), 0, 0, 0, "X")
            .unwrap();
        check(&core, clipped, true);
        for saved in [
            core.export_hwp_with_adapter_snapshot_with_report().unwrap(),
            core.export_hwpx_native_with_report().unwrap(),
        ] {
            assert_eq!(
                saved
                    .content_loss()
                    .losses()
                    .iter()
                    .any(|l| l.path.contains("글상자 텍스트 클립")),
                clipped
            );
            check(
                &DocumentCore::from_bytes(saved.bytes()).unwrap(),
                clipped,
                true,
            );
        }
        core.delete_text_in_header_footer_native(0, key.starts_with("header"), 0, 0, 0, 1)
            .unwrap();
        check(&core, clipped, false);
    }
}

fn live_footer(align: u32, margin: u32) -> DocumentCore {
    let mut core = DocumentCore::from_pdf_bytes(
        &fs::read(root().join("sources/first-hole.pdf")).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap();
    let mut doc = core.document().clone();
    let mut footer = doc.sections[0].paragraphs[0].clone();
    footer.controls.retain(|c| matches!(c, Control::Shape(_)));
    for c in &mut footer.controls {
        if let Control::Shape(s) = c {
            s.common_mut().vert_rel_to = VertRelTo::Para;
            s.common_mut().horz_rel_to = HorzRelTo::Para;
        }
    }
    let page = &mut doc.sections[0].section_def.page_def;
    page.margin_bottom = 6000;
    page.margin_footer = margin;
    let def = doc.sections[0].section_def.clone();
    let para = &mut doc.sections[0].paragraphs[0];
    for c in &mut para.controls {
        if let Control::SectionDef(s) = c {
            **s = def.clone();
        }
    }
    para.controls
        .retain(|c| !matches!(c, Control::Shape(_) | Control::Table(_)));
    para.controls.push(Control::Footer(Box::new(Footer {
        paragraphs: vec![footer],
        list_attr: align << 21,
        text_width: 39900,
        ..Default::default()
    })));
    core.set_document(doc);
    core
}
fn origin(core: &DocumentCore) -> (f64, f64) {
    let tree = core.build_page_render_tree(0).unwrap();
    let n = owner(&tree.root).unwrap();
    let ClipCommand::Move(_, y) = n.paint_clip.as_ref().unwrap()[0].commands[0] else {
        panic!("source path start")
    };
    (n.bbox.y, y)
}
#[test]
fn actual_footer_alignment_moves_clip_with_content_and_no_room_does_not_move() {
    let positions: Vec<_> = (0..3)
        .map(|a| {
            let core = live_footer(a, 6000);
            check(&core, true, false);
            origin(&core)
        })
        .collect();
    assert!(positions[0].0 < positions[1].0 && positions[1].0 < positions[2].0);
    for p in &positions[1..] {
        assert!(
            ((p.0 - positions[0].0) - (p.1 - positions[0].1)).abs() < 1e-8,
            "ordinary footer alignment cannot leave clipping at its original page origin"
        );
    }
    assert_eq!(
        origin(&live_footer(0, 1000)),
        origin(&live_footer(2, 1000)),
        "without spare footer height, alignment must not invent a translation"
    );
}

#[test]
fn sidecar_identity_is_unique_across_body_header_and_footer_lists() {
    let original = load("header-hole");
    let mut doc = original.document().clone();
    // Same geometry/text/ID under a different real paragraph list is ambiguous.
    let duplicate = hf_paras(&original)[0].controls[0].clone();
    doc.sections[0].paragraphs[0].controls.push(duplicate);
    assert!(text_clip::encode(&doc).is_err());
    let original = load("footer-bottom-hole");
    let mut doc = original.document().clone();
    doc.sections[0].paragraphs[0]
        .controls
        .push(hf_paras(&original)[0].controls[0].clone());
    assert!(text_clip::encode(&doc).is_err());
}
