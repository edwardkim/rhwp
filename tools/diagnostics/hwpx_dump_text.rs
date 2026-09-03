//! Stage 2 검증용 — 다양한 텍스트 시나리오 HWPX 산출.

use std::fs;
use std::path::Path;

use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::serializer::serialize_hwpx;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Stage 2.1 — 단일 문단 순수 텍스트
    write_paragraphs("output/stage2_text.hwpx", &["안녕 Hello 123"])?;

    // Stage 2.3 — ref_mixed.hwpx 재현 (다문단 + 소프트 브레이크 + 탭)
    write_paragraphs(
        "output/stage2_mixed.hwpx",
        &[
            "첫째 줄",
            "줄바꿈A\n줄바꿈B", // \n = 소프트 라인브레이크
            "탭\t뒤",
            "끝",
        ],
    )?;

    Ok(())
}

fn write_paragraphs(path: &str, texts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::default();
    let mut section = Section::default();
    for t in texts {
        let mut para = Paragraph::default();
        para.text = t.to_string();
        section.paragraphs.push(para);
    }
    doc.sections.push(section);

    let bytes = serialize_hwpx(&doc)?;
    let p = Path::new(path);
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(p, &bytes)?;
    println!("Wrote {} ({} bytes)", p.display(), bytes.len());
    Ok(())
}
