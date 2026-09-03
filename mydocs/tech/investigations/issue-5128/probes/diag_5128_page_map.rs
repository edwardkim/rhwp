//! #5128 한글 스펙문서 쪽 맵 진단.
//!
//! 원본 HWP5 와 export-hwpx 재파싱의 쪽수·프로필·쪽별 첫 항목을 대조한다.
//!
//!   당시: cargo run --release --example diag_5128_page_map

use rhwp::document_core::DocumentCore;
use rhwp::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/한글문서파일형식_5.0_revision1.3.hwp";

fn first_text(doc: &DocumentCore, page: u32) -> String {
    let pages = doc.dump_page_items_json(Some(page));
    let Some(page_v) = pages.as_array().and_then(|a| a.first()) else {
        return String::new();
    };
    let cols = page_v.get("columns").and_then(|v| v.as_array());
    let Some(cols) = cols else {
        return String::new();
    };
    for col in cols {
        let Some(items) = col.get("items").and_then(|v| v.as_array()) else {
            continue;
        };
        for item in items {
            if let Some(preview) = item.get("textPreview").and_then(|v| v.as_str()) {
                let t = preview.trim();
                if !t.is_empty() {
                    return t.chars().take(40).collect();
                }
            }
            if let Some(kind) = item.get("kind").and_then(|v| v.as_str()) {
                let para = item.get("paraIndex").and_then(|v| v.as_u64()).unwrap_or(0);
                return format!("{kind}#p{para}");
            }
        }
    }
    String::new()
}

fn item_summary(doc: &DocumentCore, page: u32) -> String {
    let pages = doc.dump_page_items_json(Some(page));
    let Some(page_v) = pages.as_array().and_then(|a| a.first()) else {
        return "empty".into();
    };
    let mut kinds = Vec::new();
    if let Some(cols) = page_v.get("columns").and_then(|v| v.as_array()) {
        for col in cols {
            if let Some(items) = col.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
                    let para = item.get("paraIndex").and_then(|v| v.as_u64()).unwrap_or(0);
                    kinds.push(format!("{kind}:{para}"));
                }
            }
        }
    }
    if kinds.len() > 8 {
        format!("{}..+{}", kinds[..8].join(","), kinds.len() - 8)
    } else {
        kinds.join(",")
    }
}

fn print_profile(label: &str, doc: &DocumentCore) {
    let ir = doc.document();
    let p = ir.layout_profile();
    println!(
        "{label} pages={} format={:?} native_hwp5={} origin_hwpx={} stored_pagi={} hwpx_container={} marker={}",
        doc.page_count(),
        ir.provenance.format,
        p.native_hwp5_layout(),
        p.hwp5_origin_hwpx(),
        p.hwp5_stored_pagination_layout(),
        p.hwpx_container(),
        ir.hwpx_aux_entry(HWP5_ORIGIN_HWPX_MARKER_PATH).is_some()
    );
    println!(
        "  sections={} paras={}",
        ir.sections.len(),
        ir.sections
            .iter()
            .map(|s| s.paragraphs.len())
            .sum::<usize>()
    );
}

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let src = DocumentCore::from_bytes(&data).expect("parse source");
    print_profile("SRC", &src);

    let n = src.page_count();
    for i in 0..n {
        println!(
            "SRC p{:03} | {} | {}",
            i + 1,
            first_text(&src, i),
            item_summary(&src, i)
        );
    }

    let bytes = src.export_hwpx_native().expect("export hwpx");
    println!("exported hwpx bytes={}", bytes.len());
    let out = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tools/page_roundtrip/fixtures/issue_5128/export.hwpx");
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&out, &bytes);
    println!("wrote {}", out.display());
    let rt = DocumentCore::from_bytes(&bytes).expect("reparse hwpx");
    print_profile("RT ", &rt);

    let m = rt.page_count();
    for i in 0..m {
        println!(
            "RT  p{:03} | {} | {}",
            i + 1,
            first_text(&rt, i),
            item_summary(&rt, i)
        );
    }

    println!("COMPARE src={n} rt={m}");
    let max = n.max(m);
    for i in 0..max {
        let a = if i < n {
            first_text(&src, i)
        } else {
            "<missing>".into()
        };
        let b = if i < m {
            first_text(&rt, i)
        } else {
            "<missing>".into()
        };
        if a != b {
            println!("DIFF p{:03}\n  src={a}\n  rt ={b}", i + 1);
        }
    }
}
