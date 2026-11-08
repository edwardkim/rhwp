//! Independent PDF operators and validation boundary: pdf/issue7998/text-clips.md.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{
        control::Control,
        paragraph::Paragraph,
        text_clip::{self, TextClipPath},
    },
    parser::pdf::PdfImportOptions,
    DocumentCore,
};
use std::{fs, path::PathBuf};

const CASES: [&str; 8] = [
    "nonrectangular-triangle",
    "evenodd-hole",
    "nonzero-hole",
    "parent-intersection",
    "restored-triangle",
    "form-triangle",
    "background-triangle",
    "curve-clip",
];
fn load(key: &str) -> DocumentCore {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/text-clips");
    DocumentCore::from_pdf_bytes(
        &fs::read(root.join(format!("{key}.pdf"))).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap()
}
fn text(paras: &[Paragraph]) -> String {
    let mut out = String::new();
    for p in paras {
        out.extend(p.text.chars().filter(|c| !c.is_whitespace()));
        for c in &p.controls {
            match c {
                Control::Shape(s) => {
                    if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                        out += &text(&tb.paragraphs);
                    }
                }
                Control::Table(t) => {
                    for cell in &t.cells {
                        out += &text(&cell.paragraphs);
                    }
                }
                _ => {}
            }
        }
    }
    out
}
fn clips(core: &DocumentCore) -> Vec<Vec<TextClipPath>> {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .filter_map(|c| {
            let Control::Shape(s) = c else {
                return None;
            };
            s.drawing()?.text_box.as_ref()?.paint_clip.clone()
        })
        .collect()
}
fn check(core: &DocumentCore, key: &str, edited: bool) {
    assert_eq!(core.page_count(), 1);
    assert_eq!(
        text(&core.document().sections[0].paragraphs),
        format!(
            "{}MERGEDHEADERLOWERLEFTLOWERRIGHT",
            if edited { "M" } else { "" }
        ),
        "paint clipping must not delete, duplicate, or reorder editable source text"
    );
    let paths = clips(core);
    assert_eq!(
        paths.len(),
        if key == "restored-triangle" { 0 } else { 2 },
        "only the actual first two text-show invocations have an active nonrectangular clip"
    );
    assert!(paths.iter().all(|p| text_clip::valid(p)));
    if key == "parent-intersection" || key == "form-triangle" {
        assert!(
            paths.iter().all(|p| p.len() == 2),
            "parent intersection must remain explicit"
        );
    }
    if key == "evenodd-hole" || key == "nonzero-hole" {
        assert!(
            paths
                .iter()
                .all(|p| p[0].even_odd == (key == "evenodd-hole")),
            "identical geometry has different source fill rules"
        );
    }
    let layer: serde_json::Value =
        serde_json::from_str(&core.get_page_layer_tree_native(0).unwrap()).unwrap();
    let required = layer["requiredFeatures"].as_array().unwrap();
    assert_eq!(
        required.iter().any(|f| f == "paint.textClipPath"),
        !paths.is_empty()
    );
    fn count(v: &serde_json::Value) -> usize {
        usize::from(v["kind"] == "clipPath")
            + v.get("child").map_or(0, count)
            + v["children"]
                .as_array()
                .map_or(0, |c| c.iter().map(count).sum())
    }
    assert_eq!(
        count(&layer["root"]),
        paths.len(),
        "actual paint must consume every editable clip owner"
    );
}
#[test]
fn active_text_clips_preserve_editable_content_and_actual_paint_ownership() {
    for key in CASES {
        check(&load(key), key, false);
    }
}
#[test]
fn editing_and_both_real_save_adapters_retain_clips_and_report_native_format_limit() {
    for key in CASES {
        let mut core = load(key);
        let ci = core.get_textbox_control_index_native(0, 0) as usize;
        core.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "M")
            .unwrap();
        check(&core, key, true);
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
                key != "restored-triangle",
                "native-format incompatibility must not be hidden"
            );
            check(&DocumentCore::from_bytes(saved.bytes()).unwrap(), key, true);
        }
        core.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1)
            .unwrap();
        check(&core, key, false);
    }
}
#[test]
fn private_clip_metadata_requires_unique_identity_and_matching_current_text() {
    let core = load("nonrectangular-triangle");
    let original = core.document();
    let encoded = text_clip::encode(original).unwrap().unwrap();
    let clear = || {
        let mut d = original.clone();
        for c in &mut d.sections[0].paragraphs[0].controls {
            if let Control::Shape(s) = c {
                if let Some(t) = s.drawing_mut().and_then(|d| d.text_box.as_mut()) {
                    t.paint_clip = None;
                }
            }
        }
        d
    };
    let restored_count = |d: &rhwp::model::document::Document| {
        d.sections[0].paragraphs[0].controls.iter()
        .filter(|c| matches!(c, Control::Shape(s) if s.drawing().and_then(|d| d.text_box.as_ref()).is_some_and(|t| t.paint_clip.is_some())))
        .count()
    };
    let mut doc = clear();
    text_clip::decode(&mut doc, &encoded);
    assert_eq!(restored_count(&doc), 2);
    let mut payload: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    let duplicate = payload["records"][0].clone();
    payload["records"].as_array_mut().unwrap().push(duplicate);
    let mut doc = clear();
    text_clip::decode(&mut doc, &serde_json::to_vec(&payload).unwrap());
    assert_eq!(
        restored_count(&doc),
        0,
        "ambiguous sidecar IDs cannot bind clips"
    );
    let mut doc = clear();
    for c in &mut doc.sections[0].paragraphs[0].controls {
        if let Control::Shape(s) = c {
            if let Some(t) = s.drawing_mut().and_then(|d| d.text_box.as_mut()) {
                t.paragraphs[0].text.insert(0, 'M');
            }
        }
    }
    text_clip::decode(&mut doc, &encoded);
    assert_eq!(
        restored_count(&doc),
        0,
        "stale source fingerprints cannot bind to edited paragraphs"
    );
    for bytes in [
        b"{}".to_vec(),
        b"{\"version\":2,\"records\":[]}".to_vec(),
        vec![b' '; text_clip::MAX_BYTES + 1],
    ] {
        let mut doc = clear();
        text_clip::decode(&mut doc, &bytes);
        assert_eq!(restored_count(&doc), 0);
    }
}
