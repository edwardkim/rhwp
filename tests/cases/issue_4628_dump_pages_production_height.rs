//! #4628 dump-pages 문단 높이는 프로덕션 조판 높이와 같아야 한다.
//!
//! 이전 진단은 HeightMeasurer fallback 벡터 합(`sb+Σlh+Σls+sa`)을 말해, ClickHere
//! 차감·표 vpos clamp 가 들어간 `total_height` 와도, TypesetEngine format_paragraph
//! 와도 어긋났다. dump-pages 는 pagination 이 쓰는 format_paragraph 만 읽는다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/form-01.hwp";

fn load_sample(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn full_paragraph_heights(core: &DocumentCore) -> Vec<(usize, usize, f64, f64, f64, f64, f64)> {
    let pages = core.dump_page_items_json(None);
    let mut out = Vec::new();
    for page in pages.as_array().expect("pages") {
        let section = page["section"].as_u64().expect("section") as usize;
        let columns = page["columns"].as_array().expect("columns");
        for col in columns {
            let items = col["items"].as_array().expect("items");
            for item in items {
                if item["kind"].as_str() != Some("fullParagraph") {
                    continue;
                }
                let para = item["paraIndex"].as_u64().expect("paraIndex") as usize;
                let height = &item["height"];
                out.push((
                    section,
                    para,
                    height["total"].as_f64().expect("height.total"),
                    height["spacingBefore"].as_f64().expect("spacingBefore"),
                    height["lineHeightSum"].as_f64().expect("lineHeightSum"),
                    height["lineSpacingSum"].as_f64().expect("lineSpacingSum"),
                    height["spacingAfter"].as_f64().expect("spacingAfter"),
                ));
            }
        }
    }
    out
}

#[test]
fn dump_pages_full_paragraph_height_matches_production_typeset() {
    let core = load_sample(SAMPLE);
    let rows = full_paragraph_heights(&core);
    assert!(
        !rows.is_empty(),
        "form-01.hwp 에 FullParagraph 진단 항목이 있어야 한다"
    );

    let mut fallback_diverged = 0usize;
    for (section, para, dump_total, sb, lh, ls, sa) in &rows {
        let production = core
            .production_paragraph_height(*section, *para, None)
            .unwrap_or_else(|| panic!("production height s{section} p{para}"));
        assert!(
            (dump_total - production).abs() <= 1e-9,
            "dump-pages height.total={dump_total} 이 프로덕션 format_paragraph={production} 과 \
             같아야 한다 (s{section} p{para})"
        );
        let component_sum = *sb + *lh + *ls + *sa;
        assert!(
            (dump_total - component_sum).abs() <= 1e-9,
            "프로덕션 높이는 자기 구성요소 합과 같아야 한다: total={dump_total} \
             sum={component_sum} (s{section} p{para})"
        );
        if let Some(fallback) = core.fallback_measured_paragraph_total_height(*section, *para) {
            if (production - fallback).abs() > 0.01 {
                fallback_diverged += 1;
                assert!(
                    (dump_total - fallback).abs() > 1e-9,
                    "fallback total_height={fallback} 과 다른 프로덕션 값인데 dump-pages 가 \
                     fallback 을 말하면 안 된다 (s{section} p{para})"
                );
            }
        }
    }
    let _ = fallback_diverged;
}

#[test]
fn dump_pages_follows_production_when_fallback_total_diverges() {
    for rel in [SAMPLE, "samples/k-water-rfp.hwp", "samples/hwp3-sample.hwp"] {
        let core = load_sample(rel);
        for (section, para, dump_total, ..) in full_paragraph_heights(&core) {
            let Some(production) = core.production_paragraph_height(section, para, None) else {
                continue;
            };
            let Some(fallback) = core.fallback_measured_paragraph_total_height(section, para)
            else {
                continue;
            };
            if (production - fallback).abs() <= 0.01 {
                continue;
            }
            assert!(
                (dump_total - production).abs() <= 1e-9,
                "{rel} s{section} p{para}: dump={dump_total} production={production} fallback={fallback}"
            );
        }
    }
}
