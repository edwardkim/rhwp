//! Independent geometry, normal Hancom save and Print admission:
//! pdf/issue7998/paint-save-sources.md. No absolute screen-coordinate golden.
#![cfg(feature = "pdf-import")]

use rhwp::{
    model::{control::Control, paragraph::Paragraph, style::BorderLineType, table::Table},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn base() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/paint-save")
}
fn root(c: &DocumentCore) -> (usize, &Table) {
    c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .find_map(|(i, c)| {
            if let Control::Table(t) = c {
                Some((i, t.as_ref()))
            } else {
                None
            }
        })
        .unwrap()
}
fn text(paras: &[Paragraph], out: &mut String) {
    for p in paras {
        out.extend(p.text.chars().filter(|c| !c.is_whitespace()));
        for c in &p.controls {
            match c {
                Control::Table(t) => {
                    for cell in &t.cells {
                        text(&cell.paragraphs, out);
                    }
                }
                Control::Shape(s) => {
                    if let Some(box_) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                        text(&box_.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}
fn contents(c: &DocumentCore) -> String {
    let mut out = String::new();
    text(&c.document().sections[0].paragraphs, &mut out);
    out
}
fn model_frames(t: &Table, out: &mut Vec<(u32, u32)>) {
    out.push((t.common.width, t.common.height));
    for cell in &t.cells {
        for p in &cell.paragraphs {
            for ctrl in &p.controls {
                if let Control::Table(child) = ctrl {
                    model_frames(child, out);
                }
            }
        }
    }
}
fn actual_frames(n: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(n.node_type, RenderNodeType::Table(_)) {
        out.push((n.bbox.width, n.bbox.height));
    }
    for child in &n.children {
        actual_frames(child, out);
    }
}
fn physical_owners(n: &RenderNode, cell: Option<&RenderNode>) {
    if let Some(owner) = cell {
        if matches!(
            n.node_type,
            RenderNodeType::Rectangle(_) | RenderNodeType::Table(_) | RenderNodeType::TextRun(_)
        ) {
            let a = n.bbox;
            let b = owner.bbox;
            assert!(
                a.x >= b.x - 0.5
                    && a.y >= b.y - 0.5
                    && a.x + a.width <= b.x + b.width + 0.5
                    && a.y + a.height <= b.y + b.height + 0.5,
                "source paint and content must remain in their physical cell: {a:?} vs {b:?}"
            );
        }
    }
    let owner = if matches!(n.node_type, RenderNodeType::TableCell(_)) {
        Some(n)
    } else {
        cell
    };
    for child in &n.children {
        physical_owners(child, owner);
    }
}
fn verify(c: &DocumentCore, want: &str) {
    assert_eq!(c.page_count(), 1);
    assert_eq!(
        contents(c),
        want,
        "preserve reading order without omissions or duplication"
    );
    let t = root(c).1;
    let mut declared = Vec::new();
    model_frames(t, &mut declared);
    let tree = c.build_page_render_tree(0).unwrap();
    let mut actual = Vec::new();
    actual_frames(&tree.root, &mut actual);
    assert_eq!(actual.len(), 2, "parent and child remain real tables");
    assert_eq!(actual.len(), declared.len());
    for ((w, h), (dw, dh)) in actual.into_iter().zip(declared) {
        // These normal, complete source frames fit the declared content. Text
        // leading space must not be added again to background geometry.
        assert!((w - f64::from(dw) * 96. / 7200.).abs() < 0.5);
        assert!(
            (h - f64::from(dh) * 96. / 7200.).abs() < 0.5,
            "text lead must not inflate the physical source frame: actual {h}, stored {dh} HU"
        );
    }
    physical_owners(&tree.root, None);
}
fn roundtrip(c: &DocumentCore, want: &str) {
    verify(c, want);
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        verify(&DocumentCore::from_bytes(&bytes).unwrap(), want);
    }
}
#[test]
fn hancom_saved_leading_space_does_not_expand_background_frames() {
    // Produced by Hancom 2020 opening our HWPX and saving HWP, without hand-
    // authored LineSeg. The same file gives 265.2px/43.7px in the previous core,
    // vs the independently declared parent/child heights 180pt/28pt.
    let c = DocumentCore::from_bytes(
        &std::fs::read(base().join("hancom-occluded-parent.hwp")).unwrap(),
    )
    .unwrap();
    let want = contents(&c);
    assert!(want.contains("BEFORECHILDABCD"));
    assert!(want.contains("AFTERCHILD"));
    assert!(want.ends_with("OUTSIDEAFTER"));
    roundtrip(&c, &want);
}
#[test]
fn gapped_and_occluded_area_paint_keeps_owners_across_text_edit_and_save() {
    for key in ["occluded-parent", "periodic-parent"] {
        let mut c = DocumentCore::from_pdf_bytes(
            &std::fs::read(base().join(format!("{key}.pdf"))).unwrap(),
            &PdfImportOptions::default(),
        )
        .unwrap();
        let want = contents(&c);
        let (ci, t) = root(&c);
        assert!(
            t.cells.iter().any(|cell| c.document().doc_info.border_fills
                [cell.border_fill_id as usize - 1]
                .borders
                .iter()
                .any(|b| b.line_type == BorderLineType::None)),
            "real gaps and white occlusion must not be filled with solid native borders"
        );
        // Independent PDF text begins below the top edge, but its cells have
        // no author-declared inset. The lead belongs to the first text line.
        for cell in &t.cells {
            assert_eq!(cell.padding.top, 0);
        }
        roundtrip(&c, &want);
        c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "X")
            .unwrap();
        let edited = contents(&c);
        assert_eq!(edited, want.replacen("BEFORECHILD", "XBEFORECHILD", 1));
        roundtrip(&c, &edited);
        c.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1).unwrap();
        roundtrip(&c, &want);
    }
}
