//! Owned paragraph-list preservation; visual scope and known note paint gap: stage13.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{
        control::Control,
        footnote::{Endnote, Footnote},
        header_footer::MasterPage,
        paragraph::Paragraph,
        shape::TextBox,
        text_clip,
    },
    parser::pdf::PdfImportOptions,
    DocumentCore,
};
use std::{fs, path::PathBuf};
fn source() -> DocumentCore {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7998/text-clip-placement/sources/first-hole.pdf");
    DocumentCore::from_pdf_bytes(&fs::read(p).unwrap(), &PdfImportOptions::default()).unwrap()
}
fn make(kind: &str) -> DocumentCore {
    let mut c = source();
    let mut d = c.document().clone();
    let mut owner = d.sections[0].paragraphs[0].clone();
    owner.controls.retain(|v| matches!(v, Control::Shape(_)));
    d.sections[0].paragraphs[0]
        .controls
        .retain(|v| !matches!(v, Control::Shape(_) | Control::Table(_)));
    match kind {
        "footnote" => d.sections[0].paragraphs[0]
            .controls
            .push(Control::Footnote(Box::new(Footnote {
                number: 1,
                paragraphs: vec![owner],
                instance_id: 170013,
                ..Default::default()
            }))),
        "endnote" => d.sections[0].paragraphs[0]
            .controls
            .push(Control::Endnote(Box::new(Endnote {
                number: 1,
                paragraphs: vec![owner],
                instance_id: 170014,
                ..Default::default()
            }))),
        "master" => {
            let p = &d.sections[0].section_def.page_def;
            let (w, h) = if p.landscape {
                (p.height, p.width)
            } else {
                (p.width, p.height)
            };
            d.sections[0].section_def.master_pages.push(MasterPage {
                paragraphs: vec![owner],
                text_width: w,
                text_height: h,
                ..Default::default()
            });
            let def = d.sections[0].section_def.clone();
            let mut mirrored = false;
            for ctrl in &mut d.sections[0].paragraphs[0].controls {
                if let Control::SectionDef(s) = ctrl {
                    **s = def.clone();
                    mirrored = true;
                }
            }
            if !mirrored {
                d.sections[0].paragraphs[0]
                    .controls
                    .insert(0, Control::SectionDef(Box::new(def)));
            }
        }
        _ => panic!("unknown owner"),
    }
    c.set_document(d);
    c
}
fn paras<'a>(d: &'a rhwp::model::document::Document, kind: &str) -> &'a [Paragraph] {
    if kind == "master" {
        return &d.sections[0].section_def.master_pages[0].paragraphs;
    }
    d.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Footnote(n) if kind == "footnote" => Some(n.paragraphs.as_slice()),
            Control::Endnote(n) if kind == "endnote" => Some(n.paragraphs.as_slice()),
            _ => None,
        })
        .unwrap()
}
fn textbox(ps: &[Paragraph]) -> &TextBox {
    ps[0]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Shape(s) = c {
                s.drawing().and_then(|d| d.text_box.as_ref())
            } else {
                None
            }
        })
        .unwrap()
}
// Read source directly; its first shape is a body control rather than a note.
fn original_clip() -> Option<Vec<text_clip::TextClipPath>> {
    textbox(&source().document().sections[0].paragraphs)
        .paint_clip
        .clone()
}
#[test]
fn footnote_endnote_and_master_clip_metadata_survives_both_real_writers() {
    let expected = original_clip();
    for kind in ["footnote", "endnote", "master"] {
        let c = make(kind);
        let encoded = text_clip::encode(c.document()).unwrap().unwrap();
        let records: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(
            records["records"].as_array().unwrap().len(),
            1,
            "canonical master and body SectionDef mirror are one owner"
        );
        for saved in [
            c.export_hwp_with_adapter_snapshot_with_report().unwrap(),
            c.export_hwpx_native_with_report().unwrap(),
        ] {
            let n = DocumentCore::from_bytes(saved.bytes()).unwrap();
            let tb = textbox(paras(n.document(), kind));
            assert_eq!(tb.paragraphs[0].text.trim(), "MERGED");
            assert_eq!(
                tb.paint_clip, expected,
                "source paths, winding and owner must survive actual storage"
            );
            assert!(
                saved
                    .content_loss()
                    .losses()
                    .iter()
                    .any(|l| l.path.contains("글상자 텍스트 클립")),
                "native Hancom limitation stays explicit"
            );
        }
    }
}
#[test]
fn canonical_master_and_note_owners_share_one_global_identity_domain() {
    for kind in ["footnote", "endnote", "master"] {
        let c = make(kind);
        let mut d = c.document().clone();
        let duplicate = paras(&d, kind)[0].controls[0].clone();
        d.sections[0].paragraphs[0].controls.push(duplicate);
        assert!(
            text_clip::encode(&d).is_err(),
            "body/list ownership collision must fail closed: {kind}"
        );
    }
}
#[test]
fn stale_note_or_master_text_cannot_receive_an_old_clip_record() {
    for kind in ["footnote", "endnote", "master"] {
        let c = make(kind);
        let bytes = text_clip::encode(c.document()).unwrap().unwrap();
        let mut d = c.document().clone();
        let ps = if kind == "master" {
            &mut d.sections[0].section_def.master_pages[0].paragraphs
        } else {
            d.sections[0].paragraphs[0]
                .controls
                .iter_mut()
                .find_map(|c| match c {
                    Control::Footnote(n) => Some(&mut n.paragraphs),
                    Control::Endnote(n) => Some(&mut n.paragraphs),
                    _ => None,
                })
                .unwrap()
        };
        for c in &mut ps[0].controls {
            if let Control::Shape(s) = c {
                if let Some(tb) = s.drawing_mut().and_then(|d| d.text_box.as_mut()) {
                    tb.paint_clip = None;
                    tb.paragraphs[0].text.insert(0, 'X');
                }
            }
        }
        text_clip::decode(&mut d, &bytes);
        let tb = textbox(paras(&d, kind));
        assert!(tb.paint_clip.is_none());
        assert_eq!(tb.paragraphs[0].text.trim(), "XMERGED");
    }
}
