//! [#4771] 원본 IR과 renderer-only 조판 상태의 persistence 경계.

use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::serialize_hwpx;

#[test]
fn hwpx_does_not_persist_layout_only_fill_lines() {
    let mut paragraph = Paragraph::new_empty();
    paragraph.line_segs.push(LineSeg {
        text_start: 1,
        vertical_pos: 1_600,
        line_height: 1_000,
        text_height: 1_000,
        baseline_distance: 850,
        line_spacing: 600,
        segment_width: 42_520,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
        ..Default::default()
    });
    paragraph.layout_only_fill_lines = 1;

    let mut document = Document::default();
    document.sections.push(Section {
        paragraphs: vec![paragraph],
        ..Default::default()
    });

    let bytes = serialize_hwpx(&document).expect("HWPX 직렬화");
    let reparsed = parse_hwpx(&bytes).expect("HWPX 재파싱");
    let persisted = &reparsed.sections[0].paragraphs[0].line_segs;

    assert_eq!(
        persisted.len(),
        1,
        "renderer-only suffix가 HWPX 파일 데이터가 되면 안 된다: {persisted:?}"
    );
    assert_eq!(persisted[0].text_start, 0);
}
