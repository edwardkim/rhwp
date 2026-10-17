fn main() {
    let args: Vec<_> = std::env::args().collect();
    let path = &args[1];
    let doc = pdf_oxide::document::PdfDocument::open(path).unwrap();
    let text = doc.extract_text(0).unwrap();
    println!("extract_text len={}", text.len());
    if !text.is_empty() {
        let n = text.len().min(300);
        println!("first {}: {:?}", n, &text[..n]);
    }
    let chars = doc.extract_chars(0).unwrap();
    println!("chars total={}", chars.len());
    let fffd = chars.iter().filter(|c| c.char == '\u{FFFD}').count();
    println!("FFFD in chars: {}", fffd);
}
