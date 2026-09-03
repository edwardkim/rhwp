// Task #1067 — shape-001 의 본문 paragraph text/char_offsets/controls 위치 비교.
use rhwp::model::control::Control;
use rhwp::parser::parse_document;

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    for path in &paths {
        let bytes = std::fs::read(path).expect("read");
        let doc = parse_document(&bytes).expect("parse");
        println!("\n=== [{}] ===", path);
        for (si, section) in doc.sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                if para.controls.is_empty() {
                    continue;
                }
                println!(
                    "section {} pi {}: text={:?} char_count={} controls={}",
                    si,
                    pi,
                    para.text,
                    para.char_count,
                    para.controls.len()
                );
                println!("  char_offsets={:?}", para.char_offsets);
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    let label = match ctrl {
                        Control::Shape(s) => {
                            format!("Shape({:?})", std::mem::discriminant(s.as_ref()))
                        }
                        other => format!("{:?}", std::mem::discriminant(other)),
                    };
                    println!("  controls[{}]: {}", ci, label);
                }
            }
        }
    }
}
