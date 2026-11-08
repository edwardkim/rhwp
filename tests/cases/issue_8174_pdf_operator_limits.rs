//! Synthetic parser allocation/work contracts; no layout or rendering baseline.
#![cfg(feature = "pdf-import")]
use pdf_oxide::{
    content::{parser::parse_content_with_budget, Operator},
    import_limits::{ImportBudget, PdfImportLimits},
};
use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy, PdfImportProfile};
use std::{io::Write, sync::Arc};

fn budget(stream_bytes: usize, decoded_bytes: usize) -> Arc<ImportBudget> {
    Arc::new(
        ImportBudget::new(PdfImportLimits {
            stream_bytes,
            decoded_bytes: decoded_bytes as u64,
            ..Default::default()
        })
        .unwrap(),
    )
}

#[test]
fn operator_collection_respects_single_buffer_capacity_before_growth() {
    let slot = std::mem::size_of::<Operator>();
    for count in [1, 3, 9, 33] {
        // The spare bytes cannot hold another complete operator. This protects
        // byte limits on both Native and wasm32, whose enum sizes differ.
        let cap = slot * count + slot - 1;
        let request = budget(cap, 1024 * 1024);
        let source = b"n\n".repeat(count);
        let parsed = parse_content_with_budget(&source, &request).unwrap();
        assert_eq!(parsed.len(), count);
        assert!(parsed.capacity() * slot <= cap);
        assert!(request.snapshot().decoded_bytes >= (parsed.capacity() * slot) as u64);
        assert!(parse_content_with_budget(&b"n\n".repeat(count + 1), &request).is_err());
        assert!(request.check().is_err());
        assert!(parse_content_with_budget(b"n", &request).is_err());
        assert_eq!(
            parse_content_with_budget(b"n", &budget(cap, 1024 * 1024))
                .unwrap()
                .len(),
            1
        );
    }
    let too_small = budget(slot - 1, 1024);
    assert!(parse_content_with_budget(b"n", &too_small).is_err());
    assert_eq!(too_small.snapshot().decoded_bytes, 0);
}

#[test]
fn operator_collection_charges_the_request_before_allocating_storage() {
    let slot = std::mem::size_of::<Operator>();
    let exact = budget(slot, slot);
    let parsed = parse_content_with_budget(b"n", &exact).unwrap();
    assert_eq!(parsed.capacity(), 1);
    assert_eq!(exact.snapshot().decoded_bytes, slot as u64);
    let short = budget(slot, slot - 1);
    assert!(parse_content_with_budget(b"n", &short).is_err());
    let failure = short.snapshot().failure.unwrap();
    assert_eq!(failure.reason, "document decoded bytes");
    assert_eq!(failure.used, Some(0));
    assert_eq!(failure.requested, Some(slot as u64));
    assert_eq!(failure.limit, Some((slot - 1) as u64));
    assert!(parse_content_with_budget(b"n", &short).is_err());
    assert!(parse_content_with_budget(b"n", &budget(slot, slot)).is_ok());
}

fn forty_pages(extra_operator: bool) -> Vec<u8> {
    operator_pages(40, 100_000, extra_operator)
}

fn operator_pages(page_count: usize, pairs: usize, extra_operator: bool) -> Vec<u8> {
    let content = b"q Q\n".repeat(pairs);
    let mut objects = vec![b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(), vec![]];
    let mut kids = Vec::new();
    for page in 0..page_count {
        let number = objects.len() + 1;
        kids.push(format!("{number} 0 R"));
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << >> /Contents {} 0 R >>", number + 1).into_bytes());
        let mut encoder = flate2::write::ZlibEncoder::new(vec![], flate2::Compression::fast());
        encoder.write_all(&content).unwrap();
        if extra_operator && page + 1 == page_count {
            encoder.write_all(b"n\n").unwrap();
        }
        let compressed = encoder.finish().unwrap();
        let mut stream = format!(
            "<< /Length {} /Filter /FlateDecode >>\nstream\n",
            compressed.len()
        )
        .into_bytes();
        stream.extend(compressed);
        stream.extend(b"\nendstream");
        objects.push(stream);
    }
    objects[1] = format!(
        "<< /Type /Pages /Count {page_count} /Kids [{}] >>",
        kids.join(" ")
    )
    .into_bytes();
    let mut bytes = b"%PDF-1.7\n".to_vec();
    let mut offsets = vec![0];
    for (index, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        writeln!(bytes, "{} 0 obj", index + 1).unwrap();
        bytes.extend(object);
        bytes.extend(b"\nendobj\n");
    }
    let xref = bytes.len();
    write!(bytes, "xref\n0 {}\n0000000000 65535 f \n", offsets.len()).unwrap();
    for offset in &offsets[1..] {
        writeln!(bytes, "{offset:010} 00000 n ").unwrap();
    }
    write!(
        bytes,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        offsets.len()
    )
    .unwrap();
    bytes
}

#[test]
fn standard_eight_million_limit_preserves_lower_caps_and_exact_rejection() {
    let standard = PdfImportPolicy::default();
    let source = forty_pages(false);
    let parsed = parse_pdf_with_policy(&source, &Default::default(), standard).unwrap();
    assert_eq!(parsed.metadata.pages.len(), 40);
    assert_eq!(parsed.metadata.resource_policy.backend.operators, 8_000_000);
    assert_eq!(parsed.metadata.resource_usage.operators, 8_000_000);
    drop(parsed);
    for (input, policy, cap) in [
        (forty_pages(true), standard, 8_000_000),
        (
            source.clone(),
            standard
                .with_limits_json("{\"backend\":{\"operators\":4000000}}")
                .unwrap(),
            4_000_000,
        ),
        (
            source.clone(),
            PdfImportPolicy::for_profile(PdfImportProfile::Constrained),
            250_000,
        ),
    ] {
        let error = parse_pdf_with_policy(&input, &Default::default(), policy)
            .err()
            .unwrap();
        assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
        let failure = error
            .diagnostics
            .unwrap()
            .resource_usage
            .unwrap()
            .failure
            .unwrap();
        assert_eq!(failure.reason, "content operators");
        assert_eq!(
            (failure.used, failure.requested, failure.limit),
            (Some(cap), Some(1), Some(cap))
        );
    }
    let selected = rhwp::parser::pdf::PdfImportOptions {
        source_pages: Some(vec![0]),
        ..Default::default()
    };
    let next = parse_pdf_with_policy(&source, &selected, standard).unwrap();
    assert_eq!(next.metadata.resource_policy.backend.operators, 8_000_000);
    assert_eq!(next.metadata.resource_usage.operators, 200_000);
    assert!(standard
        .with_limits_json("{\"backend\":{\"operators\":8000001}}")
        .is_err());
}

#[test]
fn dense_single_page_uses_request_ceiling_and_preserves_explicit_lower_caps() {
    // Graphics-state pairs paint nothing: this exercises admission/accounting,
    // not a rendering baseline. The old page-count multiplier rejects it.
    let source = operator_pages(1, 320_000, false);
    let standard = PdfImportPolicy::default();
    let parsed = parse_pdf_with_policy(&source, &Default::default(), standard).unwrap();
    assert_eq!(parsed.metadata.resource_usage.operators, 640_000);
    assert_eq!(parsed.metadata.resource_usage.content_parse_calls, 1);
    for policy in [
        PdfImportPolicy::for_profile(PdfImportProfile::Constrained),
        standard
            .with_limits_json(r#"{"backend":{"operators":250000}}"#)
            .unwrap(),
        standard
            .with_limits_json(r#"{"backend":{"operators":639999}}"#)
            .unwrap(),
    ] {
        let error = parse_pdf_with_policy(&source, &Default::default(), policy)
            .err()
            .unwrap();
        assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
        let failure = error
            .diagnostics
            .unwrap()
            .resource_usage
            .unwrap()
            .failure
            .unwrap();
        assert_eq!(failure.reason, "content operators");
        assert_eq!(failure.limit, Some(policy.backend.operators as u64));
    }
    let exact = standard
        .with_limits_json(r#"{"backend":{"operators":640000}}"#)
        .unwrap();
    assert!(parse_pdf_with_policy(&source, &Default::default(), exact).is_ok());
}
