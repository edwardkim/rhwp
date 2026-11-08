use pdf_oxide::{
    structure::spatial_table_detector::{
        detect_tables_with_lines, TableDetectionConfig, TableStrategy,
    },
    PdfDocument,
};
fn main() {
    let pdf =
        PdfDocument::from_bytes(std::fs::read(std::env::args().nth(1).unwrap()).unwrap()).unwrap();
    let spans = pdf.extract_spans(0).unwrap();
    let paths = pdf.extract_paths(0).unwrap();
    let config = TableDetectionConfig {
        horizontal_strategy: TableStrategy::Lines,
        vertical_strategy: TableStrategy::Lines,
        text_fallback: false,
        ..Default::default()
    };
    let tables = detect_tables_with_lines(&spans, &paths, &config);
    let words = pdf.extract_words(0).unwrap();
    let word_spans: Vec<_> = words
        .into_iter()
        .map(|word| pdf_oxide::layout::TextSpan {
            text: word.text,
            bbox: word.bbox,
            font_size: word.avg_font_size,
            ..Default::default()
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({
            "publicTables":pdf.extract_tables(0).unwrap().len(),
            "lowLevelTables":tables.len(),"spans":spans.len(),"paths":paths.len(),
            "wordInputLowLevelTables":detect_tables_with_lines(&word_spans,&paths,&config).len(),
            "candidates":tables.iter().map(|t|serde_json::json!({
                "rows":t.rows.len(),"columns":t.col_count,"isRealGrid":t.is_real_grid(),"bbox":t.bbox
            })).collect::<Vec<_>>()
        })
    );
}
