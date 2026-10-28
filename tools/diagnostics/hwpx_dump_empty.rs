//! Stage 1 검증용 — 빈 Document를 HWPX로 직렬화해 파일로 떨어뜨린다.
//!
//! 실행:
//! ```
//! cargo run --example hwpx_dump_empty --release
//! ```
//!
//! 출력: `output/stage1_empty.hwpx` (한글2020/한컴오피스에서 열어 호환성 검증)

use std::fs;
use std::path::Path;

use rhwp::model::document::Document;
use rhwp::serializer::serialize_hwpx;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = Document::default();
    // Stage 1에서는 0-섹션 케이스가 유효하지만 한컴에서 열리려면 섹션이 하나는 필요할 것으로
    // 보이므로 빈 Section을 하나 추가해 산출한다.
    let mut doc_with_section = doc;
    doc_with_section
        .sections
        .push(rhwp::model::document::Section::default());

    let bytes = serialize_hwpx(&doc_with_section)?;

    let out_dir = Path::new("output");
    fs::create_dir_all(out_dir)?;
    let out_path = out_dir.join("stage1_empty.hwpx");
    fs::write(&out_path, &bytes)?;

    println!("Wrote {} ({} bytes)", out_path.display(), bytes.len());
    Ok(())
}
