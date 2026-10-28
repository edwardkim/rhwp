// #8174 구조 조사 probe. 제품 source: def947c2149df5bc812d801e6e79be19a403b4d1.
// 실행 조건과 관측 범위는 ../README.md에 기록한다. 정식 회귀 검사가 아니다.

use rhwp::model::document::Document;
use rhwp::service::{DocumentService, OpenOptions};
use rhwp::{parse_document, serialize_document, DocumentCore};
use std::error::Error;
fn text(doc: &Document) -> String {
    doc.sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}
fn main() -> Result<(), Box<dyn Error>> {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path)?;
        let parsed = parse_document(&bytes)?;
        let source_text = text(&parsed);
        let hwp = serialize_document(&parsed)?;
        let hwp_reopened = parse_document(&hwp)?;
        assert_eq!(source_text, text(&hwp_reopened));
        let hwpx = rhwp::serializer::serialize_hwpx(&parsed)?;
        let hwpx_reopened = parse_document(&hwpx)?;
        assert_eq!(source_text, text(&hwpx_reopened));
        let opened = DocumentService::new()
            .with_title_scan(false)
            .open_bytes(&bytes, &OpenOptions::new())?;
        assert_eq!(opened.document().sections.len(), parsed.sections.len());
        let mut core: DocumentCore = opened.into_core();
        let marker = "CRATES_IO_CONSUMER_8174";
        core.insert_text_native(0, 0, 0, marker)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let saved = rhwp::serializer::serialize_hwpx(core.document())?;
        let edited = parse_document(&saved)?;
        assert!(text(&edited).contains(marker));
        println!("PASS {} sections={} paragraphs={} text_chars={} hwp_bytes={} hwpx_bytes={} edited_bytes={}",
            path, parsed.sections.len(), parsed.sections.iter().map(|s| s.paragraphs.len()).sum::<usize>(),
            source_text.chars().count(), hwp.len(), hwpx.len(), saved.len());
    }
    assert!(parse_document(b"not a document").is_err());
    assert!(DocumentCore::from_bytes(b"not a document").is_err());
    println!("PASS invalid input returns typed Result");
    Ok(())
}
