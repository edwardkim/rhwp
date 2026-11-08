use rhwp_pdf_oxide_poc::inspect_page;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args()
        .nth(1)
        .ok_or("usage: nested_probe PDF OUTPUT_JSON")?;
    let output = std::env::args().nth(2).ok_or("OUTPUT_JSON missing")?;
    let bytes = std::fs::read(&input)?;
    let pdf = pdf_oxide::PdfDocument::from_bytes(bytes.clone())?;
    let mut pages = Vec::new();
    for page in 0..pdf.page_count()? {
        pages.push(match inspect_page(&bytes, page) {
            Ok(v) => json!({"page":page+1,"status":"inspected",
                "tableDetection":v["tableDetection"],"recognition":v["recognition"]}),
            Err(error) => json!({"page":page+1,"status":"unsupported","error":error}),
        });
    }
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "input":input,"sourcePageCount":pages.len(),"pages":pages,
            "scope":"all-page detection diagnostic, not all-page conversion or Hancom acceptance"
        }))?,
    )?;
    Ok(())
}
