//! #6679: compatibility PDF의 usvg font fallback이 같은 coverage probe를 반복하지 않는다.
//!
//! indexed selector는 usvg 0.45의 후보 순서·used-face 제외·호환 조건을 그대로 유지한다.
//! 문자·face-slot 상한 중 하나라도 넘으면 앞서 만든 부분 index까지 버리고 stock
//! selector로 복귀한다.

use rhwp::renderer::pdf::{
    pdf_font_fallback_selector_with_limits, svgs_to_pdf_with_options, PdfExportOptions,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use usvg::fontdb::{Database, FaceInfo, Language, Source, Stretch, Style, Weight, ID};

const FIXTURE_FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");
const EXACT_FACE_FIXTURE: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactFaceSmoke.ttc");
const BASE_FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
const FIRST_CHAR: char = '\u{e100}';
const SECOND_CHAR: char = '\u{e101}';

fn binary_source(bytes: &[u8]) -> Source {
    Source::Binary(Arc::new(bytes.to_vec()))
}

fn push_face(
    database: &mut Database,
    source: Source,
    name: &str,
    style: Style,
    weight: Weight,
    stretch: Stretch,
) -> ID {
    database.push_face_info(FaceInfo {
        id: ID::dummy(),
        source,
        index: 0,
        families: vec![(name.to_string(), Language::English_UnitedStates)],
        post_script_name: name.to_string(),
        style,
        weight,
        stretch,
        monospaced: false,
    })
}

#[test]
fn indexed_selector_preserves_stock_order_and_used_face_exclusion() {
    let source = binary_source(FIXTURE_FONT);
    let mut database = Database::new();
    let base = push_face(
        &mut database,
        binary_source(BASE_FONT),
        "base",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );
    let first = push_face(
        &mut database,
        source.clone(),
        "first",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );
    let second = push_face(
        &mut database,
        source,
        "second",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );

    let indexed = pdf_font_fallback_selector_with_limits(8, 32);
    let stock = usvg::FontResolver::default_fallback_selector();
    let mut indexed_database = Arc::new(database.clone());
    let mut stock_database = Arc::new(database);

    let used = [base];
    assert_eq!(
        indexed(FIRST_CHAR, &used, &mut indexed_database),
        stock(FIRST_CHAR, &used, &mut stock_database)
    );
    assert_eq!(
        indexed(FIRST_CHAR, &used, &mut indexed_database),
        Some(first),
        "fontdb insertion order must remain the fallback order"
    );

    let used = [base, first];
    assert_eq!(
        indexed(FIRST_CHAR, &used, &mut indexed_database),
        stock(FIRST_CHAR, &used, &mut stock_database)
    );
    assert_eq!(
        indexed(FIRST_CHAR, &used, &mut indexed_database),
        Some(second),
        "a face already used for shaping must be excluded on a cache hit"
    );
}

#[test]
fn indexed_selector_preserves_usvg_style_weight_stretch_condition() {
    let source = binary_source(FIXTURE_FONT);
    let mut database = Database::new();
    let base = push_face(
        &mut database,
        binary_source(BASE_FONT),
        "base-normal",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );
    let all_different = push_face(
        &mut database,
        source.clone(),
        "all-three-differ",
        Style::Italic,
        Weight::BOLD,
        Stretch::Expanded,
    );
    let style_matches = push_face(
        &mut database,
        source.clone(),
        "only-style-matches",
        Style::Normal,
        Weight::BOLD,
        Stretch::Expanded,
    );
    let weight_matches = push_face(
        &mut database,
        source.clone(),
        "only-weight-matches",
        Style::Italic,
        Weight::NORMAL,
        Stretch::Expanded,
    );
    let stretch_matches = push_face(
        &mut database,
        source,
        "only-stretch-matches",
        Style::Italic,
        Weight::BOLD,
        Stretch::Normal,
    );

    let indexed = pdf_font_fallback_selector_with_limits(8, 32);
    let stock = usvg::FontResolver::default_fallback_selector();
    let mut indexed_database = Arc::new(database.clone());
    let mut stock_database = Arc::new(database);
    for (used, expected) in [
        (vec![base], style_matches),
        (vec![base, style_matches], weight_matches),
        (vec![base, style_matches, weight_matches], stretch_matches),
    ] {
        let indexed_id = indexed(FIRST_CHAR, &used, &mut indexed_database);
        let stock_id = stock(FIRST_CHAR, &used, &mut stock_database);
        assert_eq!(indexed_id, stock_id);
        assert_eq!(indexed_id, Some(expected));
        assert_ne!(indexed_id, Some(all_different));
    }
}

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TempFontFile {
    dir: std::path::PathBuf,
    path: std::path::PathBuf,
}

impl TempFontFile {
    fn new(file_name: &str, bytes: &[u8]) -> Self {
        let dir = loop {
            let serial = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
            let candidate = std::env::temp_dir()
                .join(format!("rhwp-issue-6679-{}-{serial}", std::process::id()));
            match std::fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create isolated font fixture directory: {error}"),
            }
        };
        let path = dir.join(file_name);
        std::fs::write(&path, bytes).expect("write isolated font fixture");
        Self { dir, path }
    }

    fn remove_file(&self) {
        std::fs::remove_file(&self.path).expect("remove mapped font fixture after lookup");
    }
}

impl Drop for TempFontFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_dir(&self.dir);
    }
}

fn file_backed_database(path: &std::path::Path, candidate_count: usize) -> (Database, Vec<ID>) {
    let source = Source::File(path.to_path_buf());
    let mut database = Database::new();
    let mut ids = vec![push_face(
        &mut database,
        binary_source(BASE_FONT),
        "base-without-pua",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    )];
    ids.extend((0..candidate_count).map(|index| {
        push_face(
            &mut database,
            source.clone(),
            &format!("face-{index}"),
            Style::Normal,
            Weight::NORMAL,
            Stretch::Normal,
        )
    }));
    (database, ids)
}

#[test]
fn repeated_character_reuses_coverage_without_remapping_the_file() {
    let fixture = TempFontFile::new("fallback.ttf", FIXTURE_FONT);
    let (database, ids) = file_backed_database(&fixture.path, 1);
    let selector = pdf_font_fallback_selector_with_limits(8, 32);
    let mut database = Arc::new(database);
    let used = [ids[0]];

    assert_eq!(selector(FIRST_CHAR, &used, &mut database), Some(ids[1]));
    fixture.remove_file();
    assert_eq!(
        selector(FIRST_CHAR, &used, &mut database),
        Some(ids[1]),
        "a cached coverage result must not reopen or remap the deleted backing file"
    );

    let stock = usvg::FontResolver::default_fallback_selector();
    assert_eq!(
        stock(FIRST_CHAR, &used, &mut database),
        None,
        "the stock resolver proves that a fresh face-data mapping now fails"
    );
}

#[test]
fn repeated_character_reuses_negative_coverage_without_remapping_the_file() {
    let unsupported_fixture = TempFontFile::new("unsupported.ttf", BASE_FONT);
    let supported_fixture = TempFontFile::new("supported.ttf", FIXTURE_FONT);
    let mut database = Database::new();
    let base = push_face(
        &mut database,
        binary_source(BASE_FONT),
        "base-without-pua",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );
    let formerly_unsupported = push_face(
        &mut database,
        Source::File(unsupported_fixture.path.clone()),
        "first-file-backed-candidate",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );
    let supported = push_face(
        &mut database,
        Source::File(supported_fixture.path.clone()),
        "second-file-backed-candidate",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );
    let selector = pdf_font_fallback_selector_with_limits(8, 32);
    let mut database = Arc::new(database);
    let used = [base];

    assert_eq!(selector(FIRST_CHAR, &used, &mut database), Some(supported));
    std::fs::write(&unsupported_fixture.path, FIXTURE_FONT)
        .expect("replace the negative-probe fixture with a supporting font");
    assert_eq!(
        selector(FIRST_CHAR, &used, &mut database),
        Some(supported),
        "a cached negative coverage result must not remap the changed backing file"
    );

    let stock = usvg::FontResolver::default_fallback_selector();
    assert_eq!(
        stock(FIRST_CHAR, &used, &mut database),
        Some(formerly_unsupported),
        "the stock resolver proves that a fresh probe now observes the earlier face"
    );
}

#[test]
fn character_limit_discards_the_partial_index_and_uses_stock_fallback() {
    let fixture = TempFontFile::new("char-cap.ttf", FIXTURE_FONT);
    let (database, ids) = file_backed_database(&fixture.path, 1);
    let selector = pdf_font_fallback_selector_with_limits(1, 32);
    let mut database = Arc::new(database);
    let used = [ids[0]];

    assert_eq!(selector(FIRST_CHAR, &used, &mut database), Some(ids[1]));
    assert_eq!(
        selector(SECOND_CHAR, &used, &mut database),
        Some(ids[1]),
        "the overflow-triggering lookup must be delegated to stock fallback"
    );

    fixture.remove_file();
    assert_eq!(
        selector(FIRST_CHAR, &used, &mut database),
        None,
        "overflow must discard the previously cached character globally"
    );
}

#[test]
fn face_slot_limit_discards_the_partial_index_and_uses_stock_fallback() {
    let fixture = TempFontFile::new("candidate-cap.ttf", FIXTURE_FONT);
    let (database, ids) = file_backed_database(&fixture.path, 2);
    let selector = pdf_font_fallback_selector_with_limits(8, 2);
    let mut database = Arc::new(database);
    let used = [ids[0]];

    assert_eq!(selector(FIRST_CHAR, &used, &mut database), Some(ids[1]));
    assert_eq!(
        selector(SECOND_CHAR, &used, &mut database),
        Some(ids[1]),
        "the face-slot overflow lookup must be delegated to stock fallback"
    );

    fixture.remove_file();
    assert_eq!(
        selector(FIRST_CHAR, &used, &mut database),
        None,
        "face-slot overflow must discard the previously cached character globally"
    );
}

#[test]
fn fontdb_change_disables_the_index_and_uses_stock_fallback() {
    let fixture = TempFontFile::new("database-change.ttf", FIXTURE_FONT);
    let (database, ids) = file_backed_database(&fixture.path, 1);
    let selector = pdf_font_fallback_selector_with_limits(8, 32);
    let mut database = Arc::new(database);
    let used = [ids[0]];

    assert_eq!(selector(FIRST_CHAR, &used, &mut database), Some(ids[1]));
    push_face(
        Arc::make_mut(&mut database),
        binary_source(BASE_FONT),
        "added-face-without-pua",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );

    fixture.remove_file();
    assert_eq!(
        selector(FIRST_CHAR, &used, &mut database),
        None,
        "an additively changed fontdb must invalidate prior coverage and delegate to stock"
    );
}

#[test]
fn indexed_selector_keeps_stock_pdf_bytes_for_a_controlled_fontdb() {
    let candidate_source = binary_source(FIXTURE_FONT);
    let mut database = Database::new();
    push_face(
        &mut database,
        binary_source(BASE_FONT),
        "controlled-base",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );
    push_face(
        &mut database,
        candidate_source.clone(),
        "controlled-first",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );
    push_face(
        &mut database,
        candidate_source,
        "controlled-second",
        Style::Normal,
        Weight::NORMAL,
        Stretch::Normal,
    );

    let svg = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="60">"#,
            r#"<text x="4" y="16" font-family="controlled-base">{}</text>"#,
            r#"<text x="4" y="34" font-family="controlled-base">{}</text>"#,
            r#"<text x="4" y="52" font-family="controlled-base">{}</text>"#,
            r#"</svg>"#,
        ),
        FIRST_CHAR, SECOND_CHAR, FIRST_CHAR,
    );

    let mut stock_options = usvg::Options::default();
    stock_options.fontdb = Arc::new(database.clone());
    let stock_tree = usvg::Tree::from_str(&svg, &stock_options).expect("parse with stock resolver");

    let mut indexed_options = usvg::Options::default();
    indexed_options.fontdb = Arc::new(database);
    indexed_options.font_resolver.select_fallback = pdf_font_fallback_selector_with_limits(8, 32);
    let indexed_tree =
        usvg::Tree::from_str(&svg, &indexed_options).expect("parse with indexed resolver");

    let conversion = svg2pdf::ConversionOptions {
        compress: false,
        ..svg2pdf::ConversionOptions::default()
    };
    let stock_pdf = svg2pdf::to_pdf(&stock_tree, conversion, svg2pdf::PageOptions::default())
        .expect("convert stock tree");
    let indexed_pdf = svg2pdf::to_pdf(&indexed_tree, conversion, svg2pdf::PageOptions::default())
        .expect("convert indexed tree");
    assert_eq!(
        indexed_pdf, stock_pdf,
        "the cache must not change font choice, layout, text mapping, or PDF bytes"
    );
}

fn tounicode_contains(pdf: &[u8], unicode: &[u8]) -> bool {
    let Some(block_start) = pdf
        .windows(b"beginbfchar".len())
        .position(|window| window == b"beginbfchar")
    else {
        return false;
    };
    let block = &pdf[block_start + b"beginbfchar".len()..];
    let Some(block_end) = block
        .windows(b"endbfchar".len())
        .position(|window| window == b"endbfchar")
    else {
        return false;
    };
    block[..block_end]
        .windows(unicode.len())
        .any(|window| window == unicode)
}

#[test]
fn compatibility_pdf_keeps_the_fallback_text_surface() {
    let fixture = TempFontFile::new("exact-face.ttc", EXACT_FACE_FIXTURE);
    let svg = String::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="40">
            <text x="4" y="24" font-family="RHWP Exact Face Zero">&#xE104;</text>
        </svg>"#,
    );
    let mut options = PdfExportOptions::default();
    options.font_paths.push(fixture.dir.clone());

    let pdf = svgs_to_pdf_with_options(&[svg], &options)
        .expect("compatibility PDF should render the missing PUA through fallback");
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(
        tounicode_contains(&pdf, b"<E104>"),
        "compatibility PDF must preserve U+E104 in the searchable text surface"
    );
}
