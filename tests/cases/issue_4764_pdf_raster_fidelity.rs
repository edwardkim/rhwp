//! #4764 residual PDF raster fidelity after #3820 page-count lock.
//!
//! Does not rewrite #3772 ExtraLight bold or #3773 svg2pdf SubsetError.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rhwp::renderer::pdf_raster_fidelity::{
    build_multipage_pdf, classify_page_residual, compare_document_pages, extract_isolated_page,
    left_strip_text_deficit, load_page_catalog_from_path, locked_page_count, page_input,
    parse_pdf_page_tree, rect_content, render_synthetic_page, ClassificationInput, CorpusId,
    IsolatedPageStatus, IsolationPolicy, PageRecord, PdfBuildPage, RasterFingerprint,
    RasterPageSpec, RasterPrimitive, ResidualClass, WrapGeometry, ADMIN_HANDBOOK_PAGE_COUNT,
    ISSUE4090_PAGE_COUNT, REGULATORY_76076_PAGE_COUNT,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue_4764")
        .join(name)
}

fn load_all_records() -> Vec<PageRecord> {
    load_page_catalog_from_path(&fixture("catalog_all.tsv")).expect("catalog_all.tsv")
}

fn spec(width: u32, height: u32, prims: Vec<RasterPrimitive>) -> RasterPageSpec {
    RasterPageSpec {
        width,
        height,
        background: [255, 255, 255, 255],
        primitives: prims,
    }
}

fn ink_block(x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) -> RasterPrimitive {
    RasterPrimitive { x, y, w, h, color }
}

fn fp(spec: &RasterPageSpec) -> RasterFingerprint {
    RasterFingerprint::from_rgba(&render_synthetic_page(spec).unwrap())
}

#[test]
fn issue_4764_residual_class_tokens_round_trip() {
    for class in [
        ResidualClass::None,
        ResidualClass::Glyph,
        ResidualClass::Paint,
        ResidualClass::WrapFlow,
        ResidualClass::FontWidth,
        ResidualClass::FontWeight,
        ResidualClass::TablePlace,
        ResidualClass::FontEnv,
    ] {
        assert_eq!(ResidualClass::parse(class.as_str()), Some(class));
    }
    assert_eq!(ResidualClass::parse("wrap"), Some(ResidualClass::WrapFlow));
    assert!(!ResidualClass::FontEnv.is_layout_defect());
    assert!(ResidualClass::WrapFlow.is_layout_defect());
}

#[test]
fn issue_4764_locked_page_counts_are_fixed() {
    assert_eq!(ADMIN_HANDBOOK_PAGE_COUNT, 383);
    assert_eq!(REGULATORY_76076_PAGE_COUNT, 82);
    assert_eq!(ISSUE4090_PAGE_COUNT, 17);
    assert_eq!(locked_page_count(CorpusId::AdminHandbookHwp), Some(383));
    assert_eq!(locked_page_count(CorpusId::AdminHandbookHwpx), Some(383));
    assert_eq!(locked_page_count(CorpusId::Regulatory76076), Some(82));
    assert_eq!(locked_page_count(CorpusId::Issue4090), Some(17));
}

#[test]
fn issue_4764_catalog_covers_every_mapped_corpus() {
    let rows = load_all_records();
    let mut counts: HashMap<CorpusId, usize> = HashMap::new();
    for row in &rows {
        *counts.entry(row.corpus).or_insert(0) += 1;
        if let Some(locked) = locked_page_count(row.corpus) {
            assert_eq!(row.locked_count, locked, "{:?} lock drifted", row.corpus);
        }
        assert_eq!(row.human_page, row.page_index + 1);
    }
    assert_eq!(counts[&CorpusId::AdminHandbookHwp], 383);
    assert_eq!(counts[&CorpusId::AdminHandbookHwpx], 383);
    assert_eq!(counts[&CorpusId::Regulatory76076], 82);
    assert_eq!(counts[&CorpusId::Issue4090], 17);
    assert!(rows.len() >= 383 + 383 + 82 + 17);
}

#[test]
fn issue_4764_known_leftovers_keep_residual_class() {
    let rows = load_all_records();
    let find = |corpus, human| {
        rows.iter()
            .find(|r| r.corpus == corpus && r.human_page == human)
            .unwrap_or_else(|| panic!("{corpus:?} p{human} missing"))
    };
    for page in [5, 7, 15, 17] {
        let row = find(CorpusId::Issue4090, page);
        assert_eq!(row.residual, ResidualClass::WrapFlow);
        assert!(row.wrap_exclusion_risk);
    }
    assert_eq!(
        find(CorpusId::Issue4491, 26).residual,
        ResidualClass::FontEnv
    );
    assert!(find(CorpusId::Issue4491, 26).font_env_sensitive);
    assert_eq!(
        find(CorpusId::Issue4490, 2).residual,
        ResidualClass::FontWidth
    );
    assert_eq!(
        find(CorpusId::Issue4491, 9).residual,
        ResidualClass::TablePlace
    );
    assert_eq!(
        find(CorpusId::Regulatory76076, 33).residual,
        ResidualClass::Paint
    );
    assert_eq!(
        find(CorpusId::Regulatory76076, 18).residual,
        ResidualClass::Glyph
    );
    assert_eq!(
        find(CorpusId::AdminHandbookHwp, 156).residual,
        ResidualClass::None
    );
}

#[test]
fn issue_4764_one_bad_page_does_not_abort_document() {
    let good = fp(&spec(32, 32, vec![ink_block(4, 4, 8, 8, [0, 0, 0, 255])]));
    let pages = [
        page_input(None, Ok(&good), Ok(&good)),
        page_input(None, Err("isolated decode".into()), Ok(&good)),
        page_input(None, Ok(&good), Ok(&good)),
    ];
    let report = compare_document_pages(CorpusId::Issue4090, IsolationPolicy::Independent, &pages)
        .expect("document must survive");
    assert!(report.document.document_ok);
    assert_eq!(
        report.outcomes[1].status,
        IsolatedPageStatus::IsolatedWarning
    );
    assert_eq!(report.document.compared_pages, 2);
    assert_eq!(report.document.isolated_pages, 1);
}

#[test]
fn issue_4764_wrap_exclusion_is_not_font_env() {
    let oracle = spec(
        120,
        48,
        vec![
            ink_block(2, 6, 48, 28, [16, 16, 16, 255]),
            ink_block(72, 6, 40, 32, [32, 32, 32, 255]),
        ],
    );
    let mut dropped = oracle.clone();
    dropped.primitives.remove(0);
    let o = fp(&oracle);
    let c = fp(&dropped);
    let geom = WrapGeometry {
        page_width: 120,
        page_height: 48,
        table_left: 70,
        table_top: 6,
        table_right: 114,
        table_bottom: 40,
    };
    let sample = left_strip_text_deficit(&o, &c, geom);
    assert!(sample.flagged, "{sample:?}");
    let class = classify_page_residual(ClassificationInput {
        record: None,
        oracle: &o,
        candidate: &c,
        wrap: Some(geom),
        face_substituted: true,
        table_boxes_match: true,
        line_owners_match: true,
    });
    assert_eq!(class, ResidualClass::WrapFlow);
    assert!(class.is_layout_defect());
}

#[test]
fn issue_4764_font_environment_is_not_layout() {
    let oracle = fp(&spec(40, 40, vec![ink_block(8, 8, 18, 10, [8, 8, 8, 255])]));
    let candidate = fp(&spec(
        40,
        40,
        vec![ink_block(8, 8, 18, 10, [90, 90, 90, 255])],
    ));
    let rec = load_all_records()
        .into_iter()
        .find(|r| r.corpus == CorpusId::Issue4491 && r.human_page == 26)
        .expect("4491 p26");
    let class = classify_page_residual(ClassificationInput {
        record: Some(&rec),
        oracle: &oracle,
        candidate: &candidate,
        wrap: None,
        face_substituted: true,
        table_boxes_match: true,
        line_owners_match: true,
    });
    assert_eq!(class, ResidualClass::FontEnv);
    assert!(!class.is_layout_defect());
}

#[test]
fn issue_4764_pdf_page_isolation_keeps_neighbors() {
    let pdf = build_multipage_pdf(&[
        PdfBuildPage {
            width_pt: 100,
            height_pt: 80,
            contents: rect_content(4, 4, 16, 10),
        },
        PdfBuildPage {
            width_pt: 220,
            height_pt: 160,
            contents: "this page is isolated".into(),
        },
        PdfBuildPage {
            width_pt: 90,
            height_pt: 70,
            contents: rect_content(6, 6, 12, 8),
        },
    ])
    .unwrap();
    let tree = parse_pdf_page_tree(&pdf).unwrap();
    assert_eq!(tree.page_count, 3);
    let isolated = extract_isolated_page(&pdf, 1).unwrap();
    let one = parse_pdf_page_tree(&isolated).unwrap();
    assert_eq!(one.page_count, 1);
    assert_eq!(one.pages[0].width_pt, 220);
    assert!(one.pages[0].contents.contains("this page is isolated"));
    let first = parse_pdf_page_tree(&extract_isolated_page(&pdf, 0).unwrap()).unwrap();
    assert_eq!(first.pages[0].width_pt, 100);
}

#[derive(Debug)]
struct IsolationCase {
    corpus: CorpusId,
    page_index: u16,
    scenario: String,
    expect_class: ResidualClass,
    expect_isolated: bool,
    expect_doc_ok: bool,
    expect_page_count: u16,
}

fn load_isolation_cases() -> Vec<IsolationCase> {
    let text = std::fs::read_to_string(fixture("isolation_cases.tsv")).unwrap();
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 || line.is_empty() {
            continue;
        }
        let c: Vec<&str> = line.split('\t').collect();
        rows.push(IsolationCase {
            corpus: CorpusId::parse(c[1]).unwrap_or_else(|| panic!("corpus {}", c[1])),
            page_index: c[2].parse().unwrap(),
            scenario: c[4].to_string(),
            expect_class: ResidualClass::parse(c[6]).unwrap(),
            expect_isolated: c[7] == "1",
            expect_doc_ok: c[8] == "1",
            expect_page_count: c[9].parse().unwrap(),
        });
    }
    rows
}

fn inject_pair(scenario: &str) -> (RasterPageSpec, Option<RasterPageSpec>, bool) {
    let oracle = spec(
        100,
        40,
        vec![
            ink_block(2, 6, 36, 22, [12, 12, 12, 255]),
            ink_block(64, 6, 28, 26, [36, 36, 36, 255]),
        ],
    );
    match scenario {
        "clean" => (oracle.clone(), Some(oracle), false),
        "decode_fail" => (oracle, None, true),
        "wrap_deficit" => {
            let mut dropped = oracle.clone();
            dropped.primitives.remove(0);
            (oracle, Some(dropped), false)
        }
        "glyph_shift" => {
            let mut shifted = oracle.clone();
            shifted.primitives[0].color = [40, 40, 40, 255];
            (oracle, Some(shifted), false)
        }
        "paint_blob" => {
            let mut blob = oracle.clone();
            blob.primitives
                .push(ink_block(10, 2, 80, 36, [0, 0, 0, 255]));
            (oracle, Some(blob), false)
        }
        "font_env" | "font_width" => {
            let mut face = oracle.clone();
            face.primitives[0].color = [70, 70, 70, 255];
            (oracle, Some(face), false)
        }
        "table_place" => {
            let mut moved = oracle.clone();
            moved.primitives[1].x = 80;
            moved.primitives[1].y = 18;
            (oracle, Some(moved), false)
        }
        other => panic!("unknown scenario {other}"),
    }
}

#[test]
fn issue_4764_isolation_casebook_preserves_page_count() {
    let cases = load_isolation_cases();
    assert!(
        cases.len() >= 8000,
        "isolation casebook too small: {}",
        cases.len()
    );

    let locked = cases
        .iter()
        .filter(|c| {
            matches!(
                c.corpus,
                CorpusId::AdminHandbookHwp
                    | CorpusId::AdminHandbookHwpx
                    | CorpusId::Regulatory76076
                    | CorpusId::Issue4090
            )
        })
        .count();
    assert_eq!(locked, (383 + 383 + 82 + 17) * 8);

    let mut ran = 0u32;
    for case in &cases {
        if case.page_index % 17 != 0 && case.scenario != "decode_fail" && case.scenario != "clean" {
            continue;
        }
        ran += 1;
        let (oracle_spec, candidate_spec, decode_fail) = inject_pair(&case.scenario);
        let oracle = fp(&oracle_spec);
        let candidate_store = candidate_spec.as_ref().map(fp);
        let neighbor = fp(&spec(
            100,
            40,
            vec![ink_block(8, 8, 10, 10, [0, 0, 0, 255])],
        ));
        let wrap = if case.scenario == "wrap_deficit" {
            Some(WrapGeometry {
                page_width: 100,
                page_height: 40,
                table_left: 60,
                table_top: 6,
                table_right: 96,
                table_bottom: 36,
            })
        } else {
            None
        };
        let face = case.scenario == "font_env";
        let table_match = case.scenario != "table_place";
        let mid = IsolatedPageInputExt {
            inner_oracle: &oracle,
            inner_candidate: candidate_store.as_ref(),
            decode_fail,
            wrap,
            face,
            table_match,
        };
        let pages = [
            page_input(None, Ok(&neighbor), Ok(&neighbor)),
            mid.as_input(),
            page_input(None, Ok(&neighbor), Ok(&neighbor)),
        ];
        let report =
            compare_document_pages(case.corpus, IsolationPolicy::Independent, &pages).unwrap();
        assert_eq!(
            report.document.observed_page_count, 3,
            "{:?} p{} {}",
            case.corpus, case.page_index, case.scenario
        );
        assert_eq!(
            report.outcomes[1].isolated, case.expect_isolated,
            "{:?} p{} {}",
            case.corpus, case.page_index, case.scenario
        );
        if case.scenario == "clean" {
            assert_eq!(report.outcomes[1].residual, ResidualClass::None);
        }
        if case.scenario == "wrap_deficit" {
            assert_eq!(report.outcomes[1].residual, ResidualClass::WrapFlow);
        }
        if case.scenario == "font_env" {
            assert_eq!(report.outcomes[1].residual, ResidualClass::FontEnv);
            assert!(!report.outcomes[1].residual.is_layout_defect());
        }
        assert_eq!(report.document.document_ok, case.expect_doc_ok);
        if let Some(locked) = locked_page_count(case.corpus) {
            assert_eq!(case.expect_page_count, locked);
        }
    }
    assert!(ran > 400, "sampled too few isolation cases: {ran}");
}

struct IsolatedPageInputExt<'a> {
    inner_oracle: &'a RasterFingerprint,
    inner_candidate: Option<&'a RasterFingerprint>,
    decode_fail: bool,
    wrap: Option<WrapGeometry>,
    face: bool,
    table_match: bool,
}

impl<'a> IsolatedPageInputExt<'a> {
    fn as_input(&self) -> rhwp::renderer::pdf_raster_fidelity::IsolatedPageInput<'_> {
        let mut input = if self.decode_fail {
            page_input(None, Err("fixture decode".into()), Ok(self.inner_oracle))
        } else {
            page_input(
                None,
                Ok(self.inner_oracle),
                Ok(self.inner_candidate.expect("candidate")),
            )
        };
        input.wrap = self.wrap;
        input.face_substituted = self.face;
        input.table_boxes_match = self.table_match;
        input.line_owners_match = true;
        input
    }
}

#[test]
fn issue_4764_isolation_tsv_is_well_formed() {
    let cases = load_isolation_cases();
    let mut seen = 0u32;
    for case in &cases {
        seen += 1;
        assert!(case.expect_page_count > 0);
        if case.scenario == "decode_fail" {
            assert!(case.expect_isolated);
            assert!(case.expect_doc_ok);
        }
        if case.scenario == "font_env" {
            assert_eq!(case.expect_class, ResidualClass::FontEnv);
        }
    }
    assert_eq!(seen as usize, cases.len());
}

#[test]
fn issue_4764_pdf_page_manifest_builds() {
    let text = std::fs::read_to_string(fixture("pdf_pages.tsv")).unwrap();
    let mut built = 0u32;
    for (i, line) in text.lines().enumerate() {
        if i == 0 || line.is_empty() {
            continue;
        }
        let c: Vec<&str> = line.split('\t').collect();
        let page_index: u16 = c[1].parse().unwrap();
        if !page_index.is_multiple_of(23) {
            continue;
        }
        let width: u32 = c[2].parse().unwrap();
        let height: u32 = c[3].parse().unwrap();
        let content = if c[4] == "rect" {
            rect_content(8, 8, 24, 16)
        } else {
            format!("BT /F1 10 Tf 12 24 Td (p{page_index}) Tj ET")
        };
        let pdf = build_multipage_pdf(&[
            PdfBuildPage {
                width_pt: width,
                height_pt: height,
                contents: content,
            },
            PdfBuildPage {
                width_pt: width + 10,
                height_pt: height,
                contents: "neighbor".into(),
            },
        ])
        .unwrap();
        let tree = parse_pdf_page_tree(&pdf).unwrap();
        assert_eq!(tree.page_count, 2);
        built += 1;
    }
    assert!(built > 20, "built too few PDF fixtures: {built}");
}

#[cfg(feature = "native-skia")]
fn issue_4764_png_rgba(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
    let pixels = image::RgbaImage::from_pixel(width, height, image::Rgba(rgba));
    let mut bytes = std::io::Cursor::new(Vec::new());
    pixels
        .write_to(&mut bytes, image::ImageFormat::Png)
        .expect("encode test PNG");
    bytes.into_inner()
}

#[cfg(feature = "native-skia")]
fn issue_4764_adjusted_rgb(rgb: [u8; 3], brightness: i8, contrast: i8) -> [u8; 3] {
    let brightness = brightness.clamp(-100, 100) as f32 / 100.0;
    let slope = (100.0 + contrast.clamp(-100, 100) as f32) / 100.0;
    let intercept = ((0.5 - 0.5 * slope) + brightness) * 255.0;
    rgb.map(|channel| {
        ((channel as f32 * slope) + intercept)
            .round()
            .clamp(0.0, 255.0) as u8
    })
}

#[cfg(feature = "native-skia")]
struct Issue4764PdfImageStream {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

#[cfg(feature = "native-skia")]
fn issue_4764_rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

#[cfg(feature = "native-skia")]
fn issue_4764_pdf_dict_usize(dict: &[u8], key: &str) -> Option<usize> {
    let dict = String::from_utf8_lossy(dict);
    let start = dict.find(key)? + key.len();
    dict[start..]
        .split_whitespace()
        .next()
        .and_then(|value| value.parse::<usize>().ok())
}

#[cfg(feature = "native-skia")]
fn issue_4764_pdf_image_streams(pdf: &[u8]) -> Vec<Issue4764PdfImageStream> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut images = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = pdf[cursor..]
        .windows(b"stream".len())
        .position(|window| window == b"stream")
    {
        let stream_marker = cursor + relative_start;
        let Some(dict_start) = issue_4764_rfind(&pdf[..stream_marker], b"<<") else {
            cursor = stream_marker + b"stream".len();
            continue;
        };
        let Some(dict_end) = issue_4764_rfind(&pdf[..stream_marker], b">>") else {
            cursor = stream_marker + b"stream".len();
            continue;
        };
        let dict = &pdf[dict_start..dict_end + 2];
        if !dict
            .windows(b"/Subtype /Image".len())
            .any(|window| window == b"/Subtype /Image")
            || !dict
                .windows(b"/ColorSpace /DeviceRGB".len())
                .any(|window| window == b"/ColorSpace /DeviceRGB")
            || !dict
                .windows(b"/BitsPerComponent 8".len())
                .any(|window| window == b"/BitsPerComponent 8")
        {
            cursor = stream_marker + b"stream".len();
            continue;
        }
        let Some(width) = issue_4764_pdf_dict_usize(dict, "/Width") else {
            cursor = stream_marker + b"stream".len();
            continue;
        };
        let Some(height) = issue_4764_pdf_dict_usize(dict, "/Height") else {
            cursor = stream_marker + b"stream".len();
            continue;
        };
        let mut start = stream_marker + b"stream".len();
        if pdf.get(start..start + 2) == Some(b"\r\n") {
            start += 2;
        } else if pdf.get(start) == Some(&b'\n') {
            start += 1;
        }
        let Some(relative_end) = pdf[start..]
            .windows(b"endstream".len())
            .position(|window| window == b"endstream")
        else {
            break;
        };
        let end = start + relative_end;
        let raw = &pdf[start..end];
        let pixels = if dict
            .windows(b"/Filter /FlateDecode".len())
            .any(|window| window == b"/Filter /FlateDecode")
        {
            let mut decoded = Vec::new();
            if ZlibDecoder::new(raw).read_to_end(&mut decoded).is_err() {
                cursor = end + b"endstream".len();
                continue;
            }
            decoded
        } else {
            raw.to_vec()
        };
        if pixels.len() == width.saturating_mul(height).saturating_mul(3) {
            images.push(Issue4764PdfImageStream {
                width,
                height,
                pixels,
            });
        }
        cursor = end + b"endstream".len();
    }
    images
}

#[cfg(feature = "native-skia")]
fn issue_4764_pdf_images_contain_rgb(
    images: &[Issue4764PdfImageStream],
    expected: [u8; 3],
) -> bool {
    images.iter().any(|image| {
        image.width > 0
            && image.height > 0
            && image.pixels.chunks_exact(3).any(|rgb| {
                rgb.iter()
                    .zip(expected)
                    .all(|(actual, expected)| actual.abs_diff(expected) <= 2)
            })
    })
}

#[cfg(feature = "native-skia")]
fn issue_4764_direct_pdf_image_tree(
    image: rhwp::renderer::render_tree::ImageNode,
) -> rhwp::paint::PageLayerTree {
    use rhwp::paint::{LayerNode, PaintOp, RenderProfile};
    use rhwp::renderer::render_tree::BoundingBox;

    let page = BoundingBox::new(0.0, 0.0, 32.0, 32.0);
    rhwp::paint::PageLayerTree::with_profile(
        32.0,
        32.0,
        LayerNode::leaf(
            page,
            None,
            vec![PaintOp::image(
                BoundingBox::new(8.0, 8.0, 16.0, 16.0),
                image,
                None,
            )],
        ),
        RenderProfile::Print,
    )
}

#[cfg(feature = "native-skia")]
fn issue_4764_direct_pdf_page_background_tree(
    image: rhwp::renderer::render_tree::PageBackgroundImage,
) -> rhwp::paint::PageLayerTree {
    use rhwp::paint::{LayerNode, PaintOp, RenderProfile};
    use rhwp::renderer::render_tree::{BoundingBox, PageBackgroundNode};

    let page = BoundingBox::new(0.0, 0.0, 32.0, 32.0);
    rhwp::paint::PageLayerTree::with_profile(
        32.0,
        32.0,
        LayerNode::leaf(
            page,
            None,
            vec![PaintOp::page_background(
                page,
                PageBackgroundNode {
                    background_color: None,
                    border_color: None,
                    border_width: 0.0,
                    gradient: None,
                    image: Some(image),
                },
            )],
        ),
        RenderProfile::Print,
    )
}

#[cfg(feature = "native-skia")]
#[test]
fn issue_4764_direct_pdf_embeds_adjusted_normal_image_pixels() {
    use rhwp::model::style::ImageFillMode;
    use rhwp::renderer::pdf::layer_trees_to_pdf;
    use rhwp::renderer::render_tree::ImageNode;

    let source = [80, 100, 120];
    for (brightness, contrast) in [(20, 0), (0, 20), (0, 8)] {
        let mut image = ImageNode::new(
            1,
            Some(issue_4764_png_rgba(
                4,
                4,
                [source[0], source[1], source[2], 255],
            )),
        );
        image.fill_mode = Some(ImageFillMode::FitToSize);
        image.brightness = brightness;
        image.contrast = contrast;

        let pdf = layer_trees_to_pdf(&[issue_4764_direct_pdf_image_tree(image)])
            .expect("direct PDF export");
        let images = issue_4764_pdf_image_streams(&pdf);

        assert!(
            issue_4764_pdf_images_contain_rgb(
                &images,
                issue_4764_adjusted_rgb(source, brightness, contrast)
            ),
            "PDF image streams should contain brightness/contrast adjusted pixels"
        );
        assert!(
            !issue_4764_pdf_images_contain_rgb(&images, source),
            "PDF image streams should not keep the unadjusted source pixels"
        );
    }
}

#[cfg(feature = "native-skia")]
#[test]
fn issue_4764_direct_pdf_embeds_adjusted_page_background_pixels_in_display_order() {
    use rhwp::model::image::ImageEffect;
    use rhwp::model::style::ImageFillMode;
    use rhwp::renderer::pdf::layer_trees_to_pdf;
    use rhwp::renderer::render_tree::PageBackgroundImage;

    let source = [120, 90, 60];
    let image = PageBackgroundImage {
        data: issue_4764_png_rgba(4, 4, [source[0], source[1], source[2], 255]),
        fill_mode: ImageFillMode::FitToSize,
        brightness: 35,
        contrast: -10,
        effect: ImageEffect::RealPic,
    };

    let pdf = layer_trees_to_pdf(&[issue_4764_direct_pdf_page_background_tree(image)])
        .expect("direct PDF export");
    let images = issue_4764_pdf_image_streams(&pdf);

    assert!(
        issue_4764_pdf_images_contain_rgb(&images, issue_4764_adjusted_rgb(source, -10, 35)),
        "page background must apply display brightness/contrast order"
    );
    assert!(
        !issue_4764_pdf_images_contain_rgb(&images, issue_4764_adjusted_rgb(source, 35, -10)),
        "page background must not use the raw stored brightness/contrast order"
    );
}

#[cfg(feature = "native-skia")]
#[test]
fn issue_4764_direct_pdf_still_rejects_pattern8x8_images() {
    use rhwp::model::image::ImageEffect;
    use rhwp::renderer::pdf::layer_trees_to_pdf;
    use rhwp::renderer::render_tree::ImageNode;

    let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [80, 100, 120, 255])));
    image.effect = ImageEffect::Pattern8x8;
    image.brightness = 0;
    image.contrast = 20;

    let error = layer_trees_to_pdf(&[issue_4764_direct_pdf_image_tree(image)]).unwrap_err();

    assert!(
        error.contains("Pattern8x8 effect"),
        "unexpected error: {error}"
    );
    assert!(
        error.contains("use the svg backend"),
        "unexpected error: {error}"
    );
}

#[cfg(feature = "native-skia")]
#[test]
fn issue_4764_direct_pdf_still_rejects_unbaked_image_watermark_tone_or_opacity() {
    use rhwp::renderer::pdf::layer_trees_to_pdf;
    use rhwp::renderer::render_tree::ImageNode;

    for (brightness, contrast) in [(70, -50), (20, 20)] {
        let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [80, 100, 120, 255])));
        image.brightness = brightness;
        image.contrast = contrast;
        assert!(image.is_watermark());

        let error = layer_trees_to_pdf(&[issue_4764_direct_pdf_image_tree(image)]).unwrap_err();
        assert!(
            error.contains("unbaked image watermark tone or opacity"),
            "unexpected error: {error}"
        );
    }
}

#[cfg(feature = "native-skia")]
#[test]
fn issue_4764_direct_pdf_still_rejects_unbaked_background_watermark_tone() {
    use rhwp::model::image::ImageEffect;
    use rhwp::model::style::ImageFillMode;
    use rhwp::renderer::pdf::layer_trees_to_pdf;
    use rhwp::renderer::render_tree::PageBackgroundImage;

    let image = PageBackgroundImage {
        data: issue_4764_png_rgba(4, 4, [80, 100, 120, 255]),
        fill_mode: ImageFillMode::FitToSize,
        brightness: -50,
        contrast: 70,
        effect: ImageEffect::RealPic,
    };
    assert!(image.is_real_picture_watermark_tone_preset());
    let error =
        layer_trees_to_pdf(&[issue_4764_direct_pdf_page_background_tree(image)]).unwrap_err();
    assert!(
        error.contains("unbaked RealPic watermark tone"),
        "unexpected error: {error}"
    );
}

#[cfg(feature = "native-skia")]
#[test]
fn issue_4764_direct_pdf_does_not_adjust_baked_watermark_pixels_twice() {
    use rhwp::paint::{LayerNodeKind, PaintOp, ResolvedImageKind, ResolvedImagePayload};
    use rhwp::renderer::pdf::layer_trees_to_pdf;
    use rhwp::renderer::render_tree::ImageNode;
    use std::sync::Arc;

    let baked = [80, 100, 120];
    let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [20, 30, 40, 255])));
    image.brightness = 70;
    image.contrast = -50;
    let mut tree = issue_4764_direct_pdf_image_tree(image);
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        panic!("image leaf")
    };
    let PaintOp::Image { resolved, .. } = &mut ops[0] else {
        panic!("image paint op")
    };
    *resolved = Some(Arc::new(ResolvedImagePayload {
        data: issue_4764_png_rgba(4, 4, [baked[0], baked[1], baked[2], 255]),
        mime: "image/png",
        kind: ResolvedImageKind::BakedWatermark,
        suppress_effects: true,
    }));

    let pdf = layer_trees_to_pdf(&[tree]).expect("baked watermark PDF export");
    let images = issue_4764_pdf_image_streams(&pdf);
    assert!(issue_4764_pdf_images_contain_rgb(&images, baked));
    assert!(!issue_4764_pdf_images_contain_rgb(
        &images,
        issue_4764_adjusted_rgb(baked, 70, -50)
    ));
}
