//! #6963 출력 레이어의 링크 영역과 실제 PDF annotation 계약.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::{hyperlink::HyperlinkTarget, DocumentCore};
use rhwp::paint::{ClipKind, LayerNode, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::hyperlinks::{export_uri, PdfLink};
use rhwp::renderer::pdf::{
    svg2pdf_subset_error_stub, svg2pdf_to_chunk, svgs_to_pdf_with_links,
    svgs_to_pdf_with_links_and_to_chunk, PdfExportOptions,
};
use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};

fn blank(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    core
}
fn artifact(name: &str, bytes: &[u8]) {
    if let Some(dir) = std::env::var_os("RHWP_HYPERLINK_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(std::path::Path::new(&dir).join(name), bytes).unwrap();
    }
}
fn text_run(text: &str, start: usize, positions: Vec<f64>) -> TextRunNode {
    TextRunNode {
        text: text.into(),
        style: Default::default(),
        char_shape_id: Some(0),
        para_shape_id: Some(0),
        section_index: Some(0),
        para_index: Some(0),
        char_start: Some(start),
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 10.0,
        field_marker: FieldMarkerType::None,
        layout_positions: Some(positions),
        display_text: None,
    }
}
fn synthetic_tree(run: TextRunNode, clip: BoundingBox) -> PageLayerTree {
    let bounds = BoundingBox::new(10.0, 20.0, 65.0, 12.0);
    let leaf = LayerNode::leaf(bounds, None, vec![PaintOp::text_run(bounds, run)]);
    PageLayerTree::new(
        100.0,
        100.0,
        LayerNode::clip_rect(bounds, None, clip, leaf, ClipKind::TableCell),
    )
}

#[test]
fn unequal_character_boundaries_and_nested_clip_define_the_link_rect() {
    let mut core = blank("Wi😀X");
    core.insert_hyperlink_native(&HyperlinkTarget::body(0, 0), 1, 3, "https://example.com")
        .unwrap();
    let run = text_run("Wi😀X", 0, vec![0.0, 24.0, 28.0, 51.0, 65.0]);
    let tree = synthetic_tree(run.clone(), BoundingBox::new(0.0, 0.0, 100.0, 100.0));
    let links = core.hyperlinks_in_layer_tree(&tree).unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!((links[0].rect.x, links[0].rect.width), (34.0, 27.0));
    let clipped = synthetic_tree(run, BoundingBox::new(40.0, 22.0, 10.0, 8.0));
    let links = core.hyperlinks_in_layer_tree(&clipped).unwrap();
    assert_eq!(
        (
            links[0].rect.x,
            links[0].rect.y,
            links[0].rect.width,
            links[0].rect.height
        ),
        (40.0, 22.0, 10.0, 8.0)
    );
}

#[test]
fn cross_page_run_fragments_intersect_the_same_logical_link() {
    let mut core = blank("Wi😀X");
    core.insert_hyperlink_native(&HyperlinkTarget::body(0, 0), 1, 3, "https://example.com")
        .unwrap();
    let clip = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
    let first = core
        .hyperlinks_in_layer_tree(&synthetic_tree(
            text_run("Wi", 0, vec![0.0, 24.0, 28.0]),
            clip,
        ))
        .unwrap();
    let second = core
        .hyperlinks_in_layer_tree(&synthetic_tree(
            text_run("😀X", 2, vec![0.0, 23.0, 37.0]),
            clip,
        ))
        .unwrap();
    assert_eq!(
        (first[0].start, first[0].end, first[0].rect.width),
        (1, 2, 4.0)
    );
    assert_eq!(
        (second[0].start, second[0].end, second[0].rect.width),
        (2, 3, 23.0)
    );
}

#[test]
fn clipped_out_links_are_absent_and_unsupported_rotation_is_reported() {
    let mut core = blank("Wi😀X");
    core.insert_hyperlink_native(&HyperlinkTarget::body(0, 0), 1, 3, "https://example.com")
        .unwrap();
    let mut run = text_run("Wi😀X", 0, vec![0.0, 24.0, 28.0, 51.0, 65.0]);
    let hidden = synthetic_tree(run.clone(), BoundingBox::new(0.0, 50.0, 10.0, 10.0));
    assert!(core.hyperlinks_in_layer_tree(&hidden).unwrap().is_empty());
    run.rotation = 90.0;
    let tree = synthetic_tree(run, BoundingBox::new(0.0, 0.0, 100.0, 100.0));
    assert!(core.hyperlinks_in_layer_tree(&tree).is_err());
}

fn tiny_svg(width: u32) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="100"><rect x="10" y="20" width="30" height="10"/></svg>"#
    )
}
fn link(uri: &str) -> Vec<PdfLink> {
    vec![PdfLink {
        uri: uri.into(),
        rect: BoundingBox::new(10.0, 20.0, 30.0, 10.0),
    }]
}

// 이 writer의 annotation/page 사전은 비압축이다. URI와 /P 참조를 별도로 추적한다.
fn page_uris(pdf: &[u8]) -> Vec<Vec<String>> {
    let text = String::from_utf8_lossy(pdf);
    let objects: Vec<_> = text
        .split("endobj")
        .filter_map(|object| {
            let (header, body) = object.rsplit_once(" 0 obj")?;
            Some((
                header.split_whitespace().last()?.to_string(),
                body.to_string(),
            ))
        })
        .collect();
    objects
        .iter()
        .filter(|(_, body)| {
            body.split_once("/Type ")
                .is_some_and(|(_, tail)| tail.split_whitespace().next() == Some("/Page"))
        })
        .map(|(page_id, body)| {
            let Some((_, annots)) = body.split_once("/Annots [") else {
                return Vec::new();
            };
            let refs: Vec<_> = annots
                .split(']')
                .next()
                .unwrap()
                .split_whitespace()
                .collect();
            refs.chunks_exact(3)
                .map(|reference| {
                    assert_eq!(&reference[1..], &["0", "R"]);
                    let body = &objects.iter().find(|(id, _)| id == reference[0]).unwrap().1;
                    assert!(body.contains(&format!("/P {page_id} 0 R")));
                    assert!(body.contains("/Subtype /Link"));
                    body.split_once("/URI (")
                        .unwrap()
                        .1
                        .split(')')
                        .next()
                        .unwrap()
                        .to_string()
                })
                .collect()
        })
        .collect()
}

#[test]
fn pdf_coordinates_uri_encoding_and_border_are_correct() {
    let pdf = svgs_to_pdf_with_links(
        &[tiny_svg(100)],
        &[link("https://example.com/한글?q=a;b#c")],
        &PdfExportOptions::default(),
    )
    .unwrap();
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("/Rect [7.5 52.5 30 60]"), "{text}");
    assert!(text.contains("/Border [0 0 0]"));
    assert_eq!(
        page_uris(&pdf),
        vec![vec!["https://example.com/%ED%95%9C%EA%B8%80?q=a;b#c"]]
    );
    artifact("coordinate-contract.pdf", &pdf);
}

#[test]
fn skipped_svg_page_does_not_shift_other_pages_links() {
    let pages = [tiny_svg(100), tiny_svg(200), tiny_svg(300)];
    let links = [
        link("https://example.com/first"),
        link("https://example.com/skipped"),
        link("https://example.com/third"),
    ];
    let pdf = svgs_to_pdf_with_links_and_to_chunk(
        &pages,
        &links,
        &PdfExportOptions::default(),
        |tree, embed| {
            if tree.size().width() == 200.0 {
                Err(svg2pdf_subset_error_stub())
            } else {
                svg2pdf_to_chunk(tree, embed)
            }
        },
    )
    .unwrap();
    assert_eq!(
        page_uris(&pdf),
        vec![
            vec!["https://example.com/first"],
            vec!["https://example.com/third"]
        ]
    );
    assert!(!String::from_utf8_lossy(&pdf).contains("/skipped"));
    artifact("skipped-page.pdf", &pdf);
}

#[test]
fn invalid_scheme_outside_page_and_mismatched_lists_are_not_emitted() {
    assert_eq!(
        export_uri("mailto:a@example.com").unwrap(),
        "mailto:a@example.com"
    );
    assert!(export_uri("javascript:alert(1)").is_none());
    assert!(export_uri("file:///private/test").is_none());
    assert!(svgs_to_pdf_with_links(&[tiny_svg(100)], &[], &PdfExportOptions::default()).is_err());
    let mut links = link("javascript:alert(1)");
    links.push(PdfLink {
        uri: "https://example.com".into(),
        rect: BoundingBox::new(150.0, 0.0, 20.0, 20.0),
    });
    let pdf =
        svgs_to_pdf_with_links(&[tiny_svg(100)], &[links], &PdfExportOptions::default()).unwrap();
    assert_eq!(page_uris(&pdf), vec![Vec::<String>::new()]);
}

#[test]
fn hancom_textmail_and_fragment_samples_produce_uri_annotations() {
    for (sample, pages, expected, name) in [
        (
            "samples/basic/Textmail.hwp",
            vec![0],
            "http://www.hancom.co.kr",
            "textmail",
        ),
        (
            "samples/hwpx_sample2.hwpx",
            vec![7],
            "https://apply.lh.or.kr/LH/index.html#MN::CLCC_MN_0010:",
            "lh-fragment",
        ),
    ] {
        let core = DocumentCore::from_bytes(
            &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample)).unwrap(),
        )
        .unwrap();
        let links = core
            .page_hyperlinks_native(pages[0], RenderProfile::Print)
            .unwrap();
        assert!(
            links.iter().any(|l| l.uri == expected),
            "{sample}: {links:?}"
        );
        let pdf = core
            .render_pages_pdf_native_with_profile_and_options(
                &pages,
                RenderProfile::Print,
                &PdfExportOptions::default(),
            )
            .unwrap();
        assert!(page_uris(&pdf)[0].iter().any(|uri| uri == expected));
        let svg = core
            .render_page_svg_layer_with_profile_native(pages[0], RenderProfile::Print)
            .unwrap();
        assert!(svg.contains("data-rhwp-hyperlinks"));
        artifact(&format!("{name}.pdf"), &pdf);
        artifact(&format!("{name}.svg"), svg.as_bytes());
        artifact(
            &format!("{name}-links.json"),
            &serde_json::to_vec_pretty(&links).unwrap(),
        );
        // annotation 추가 전후의 잉크는 같은 SVG로 비교한다.
        let before = rhwp::renderer::pdf::svgs_to_pdf(&[svg]).unwrap();
        artifact(&format!("{name}-without-annotations.pdf"), &before);
    }
}

#[test]
fn a_multiline_link_spans_pages_and_survives_reordered_duplicate_selection() {
    let text = "alpha beta gamma delta ".repeat(240);
    let mut core = blank(&text);
    core.insert_hyperlink_native(
        &HyperlinkTarget::body(0, 0),
        0,
        text.chars().count(),
        "https://example.com/multiline",
    )
    .unwrap();
    assert!(core.page_count() > 1);
    let pages = [core.page_count() - 1, 0, core.page_count() - 1];
    let expected_counts: Vec<_> = pages
        .iter()
        .map(|p| {
            core.page_hyperlinks_native(*p, RenderProfile::Print)
                .unwrap()
                .len()
        })
        .collect();
    assert!(expected_counts[1] > 1);
    let pdf = core
        .render_pages_pdf_native_with_profile_and_options(
            &pages,
            RenderProfile::Print,
            &PdfExportOptions::default(),
        )
        .unwrap();
    assert_eq!(
        page_uris(&pdf).iter().map(Vec::len).collect::<Vec<_>>(),
        expected_counts
    );
    assert!(core.render_pages_pdf_native(&[core.page_count()]).is_err());
    artifact("multiline-reordered.pdf", &pdf);
}

#[cfg(feature = "native-skia")]
#[test]
fn direct_skia_pdf_keeps_link_annotations() {
    let mut core = blank("Clickable link");
    core.insert_hyperlink_native(
        &HyperlinkTarget::body(0, 0),
        0,
        14,
        "https://example.com/direct",
    )
    .unwrap();
    let pdf = core.render_pages_pdf_direct_native(&[0]).unwrap();
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("/Subtype /Link"));
    assert!(text.contains("https://example.com/direct"));
    artifact("direct-skia.pdf", &pdf);
}
