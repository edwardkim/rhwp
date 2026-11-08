//! 관계형: 합성 요청/CLI·누적 예산 계약. 조판 좌표·렌더 golden 검사가 아니다.
//! 주석 갱신: 2026-11-01 12:17 +09:00. 시각 측정은 예산 산술 계약에 비해당.
//! 직전 파일 변경: 5ef817b434d44bb827a7bd92d5514560c3538a3f, 2026-11-01T12:10:31+09:00.
//! #8413은 wasm32 주소 범위를 넘는 누적 작업량과 실패 시 카운터/요청 분리를 검증한다.
#![cfg(feature = "pdf-import")]
use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy, PdfImportProfile};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn image_pdf() -> Vec<u8> {
    let content = b"q 40 0 0 20 10 10 cm /Im1 Do Q";
    let pixels = [255u8, 0, 0].repeat(8);
    let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
    stream.extend(content);
    stream.extend(b"\nendstream");
    let mut image = format!("<< /Type /XObject /Subtype /Image /Width 4 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>\nstream\n", pixels.len()).into_bytes();
    image.extend(pixels);
    image.extend(b"\nendstream");
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /XObject << /Im1 5 0 R >> >> /Contents 4 0 R >>".to_vec(),
        stream, image,
    ];
    pdf_objects(&objects)
}

fn pdf_objects(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend(format!("{} 0 obj\n", index + 1).as_bytes());
        bytes.extend(object);
        bytes.extend(b"\nendobj\n");
    }
    let xref = bytes.len();
    bytes.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for offset in offsets {
        bytes.extend(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    bytes
}

#[test]
fn standard_accepts_object_rich_catalog_and_keeps_constrained_and_host_limits() {
    // Scalars in a private catalog array consume real parser objects even
    // though no additional xref objects or visible content are introduced.
    let catalog = format!(
        "<< /Type /Catalog /Pages 2 0 R /Probe [{}] >>",
        "0 ".repeat(60_000)
    );
    let source = pdf_objects(&[
        catalog.into_bytes(),
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << >> >>".to_vec(),
    ]);
    let standard = PdfImportPolicy::default();
    let result = parse_pdf_with_policy(&source, &Default::default(), standard).unwrap();
    let used = result.metadata.resource_usage.parsed_objects;
    assert!(used > 50_000);
    for policy in [
        PdfImportPolicy::for_profile(PdfImportProfile::Constrained),
        standard
            .with_limits_json(r#"{"backend":{"objects":50000}}"#)
            .unwrap(),
        standard
            .with_limits_json(&json!({"backend":{"objects":used-1}}).to_string())
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
        assert_eq!(failure.reason, "parsed object count");
        assert_eq!(failure.limit, Some(policy.backend.objects as u64));
    }
    let exact = standard
        .with_limits_json(&json!({"backend":{"objects":used}}).to_string())
        .unwrap();
    assert!(parse_pdf_with_policy(&source, &Default::default(), exact).is_ok());
    assert!(parse_pdf_with_policy(&image_pdf(), &Default::default(), standard).is_ok());
}

#[test]
fn standard_accepts_8k_samples_but_keeps_explicit_pixel_and_buffer_caps() {
    use std::io::Write;
    // A generated constant-gray 8K sample. This verifies source pixel storage
    // and resource admission, not placement or a rendered-page golden.
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    let row = [137; 7680];
    for _ in 0..4320 {
        encoder.write_all(&row).unwrap();
    }
    let pixels = encoder.finish().unwrap();
    let mut image = format!("<< /Type /XObject /Subtype /Image /Width 7680 /Height 4320 /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n", pixels.len()).into_bytes();
    image.extend(pixels);
    image.extend(b"\nendstream");
    let content = b"q 40 0 0 20 10 10 cm /Im Do /Im Do Q";
    let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
    stream.extend(content);
    stream.extend(b"\nendstream");
    let source = pdf_objects(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /XObject << /Im 5 0 R >> >> /Contents 4 0 R >>".to_vec(),
        stream, image,
    ]);
    let standard = PdfImportPolicy::default();
    for limits in [
        r#"{"maxImagePixels":16000000}"#,
        r#"{"maxDocumentImagePixels":64000000}"#,
        r#"{"backend":{"streamBytes":33554432}}"#,
        r#"{"backend":{"codecAllocationBytes":1024}}"#,
    ] {
        let policy = standard.with_limits_json(limits).unwrap();
        let error = parse_pdf_with_policy(&source, &Default::default(), policy)
            .err()
            .unwrap_or_else(|| panic!("expected resource rejection for {limits}"));
        assert_eq!(error.code, "PDF_RESOURCE_LIMIT", "{limits}: {error:?}");
    }
    let parsed = parse_pdf_with_policy(&source, &Default::default(), standard).unwrap();
    assert_eq!(parsed.document.bin_data_content.len(), 2);
    for binary in &parsed.document.bin_data_content {
        let image = image::load_from_memory(&binary.data.load())
            .unwrap()
            .to_luma8();
        assert_eq!(image.dimensions(), (7680, 4320));
        assert!(image.as_raw().iter().all(|&sample| sample == 137));
    }
}

#[test]
fn shared_image_source_keeps_placement_mutations_and_requests_independent() {
    use pdf_oxide::{extractors::images::ImageData, PdfDocument};
    let source = image_pdf();
    let pdf =
        PdfDocument::from_bytes_with_limits(source.clone(), PdfImportPolicy::default().backend)
            .unwrap();
    let budget = pdf.import_budget().unwrap();
    let mut handles = pdf.page_image_handles(0).unwrap();
    let first = handles[0].decode().unwrap();
    let png = first.to_png_bytes_with_budget(budget).unwrap();
    let work = budget.snapshot().codec_work_units;
    let second = handles[0].decode().unwrap();
    let (ImageData::Raw { pixels: a, .. }, ImageData::Raw { pixels: b, .. }) =
        (first.data(), second.data())
    else {
        panic!("raw RGB source")
    };
    assert_eq!(
        a.as_ptr(),
        b.as_ptr(),
        "immutable pixels should share storage"
    );
    assert_eq!(second.to_png_bytes_with_budget(budget).unwrap(), png);
    assert_eq!(
        budget.snapshot().codec_work_units,
        work,
        "cached PNG must not decode/encode pixels again"
    );

    handles[0].opacity = 0.5;
    let translucent = handles[0].decode().unwrap();
    let alpha_png = translucent.to_png_bytes_with_budget(budget).unwrap();
    let rgba = image::load_from_memory(&alpha_png).unwrap().to_rgba8();
    assert!(rgba.pixels().all(|p| p.0 == [255, 0, 0, 128]));
    handles[0].opacity = 1.0;
    assert_eq!(
        handles[0]
            .decode()
            .unwrap()
            .to_png_bytes_with_budget(budget)
            .unwrap(),
        png
    );
    assert_eq!(first.to_png_bytes_with_budget(budget).unwrap(), png);

    // An identical object number in another request must not reuse this cache.
    let mut other_source = source;
    let raw = [255, 0, 0].repeat(8);
    let start = other_source
        .windows(raw.len())
        .position(|bytes| bytes == raw)
        .unwrap();
    other_source[start..start + raw.len()].copy_from_slice(&[0, 0, 255].repeat(8));
    let other =
        PdfDocument::from_bytes_with_limits(other_source, PdfImportPolicy::default().backend)
            .unwrap();
    let blue = other.page_image_handles(0).unwrap()[0]
        .decode()
        .unwrap()
        .to_png_bytes_with_budget(other.import_budget().unwrap())
        .unwrap();
    assert!(image::load_from_memory(&blue)
        .unwrap()
        .to_rgb8()
        .pixels()
        .all(|p| p.0 == [0, 0, 255]));
}

#[test]
fn profiles_preserve_standard_and_reject_ambiguous_limit_documents() {
    assert_eq!(
        serde_json::to_value(PdfImportPolicy::for_profile(PdfImportProfile::Standard)).unwrap(),
        serde_json::to_value(PdfImportPolicy::default()).unwrap()
    );
    let standard = PdfImportPolicy::default();
    for invalid in [
        "null",
        "[]",
        "true",
        "{\"unknown\":1}",
        "{\"maxSelectedPages\":0}",
        "{\"maxSelectedPages\":null}",
        "{\"maxSelectedPages\":-1}",
        "{\"maxSelectedPages\":1.5}",
        "{\"maxSelectedPages\":\"2\"}",
        "{\"maxSelectedPages\":1025}",
        "{\"maxSelectedPages\":1,\"maxSelectedPages\":2}",
        "{\"backend\":null}",
        "{\"backend\":[]}",
        "{\"backend\":{\"inputBytes\":null}}",
        "{\"backend\":{\"inputBytes\":1,\"inputBytes\":2}}",
        "{\"backend\":{\"unknown\":1}}",
        "{\"backend\":{\"inputBytes\":18446744073709551616}}",
    ] {
        let error = standard.with_limits_json(invalid).expect_err(invalid);
        assert_eq!(error.code, "PDF_RESOURCE_POLICY", "{invalid}");
    }
    let constrained = PdfImportPolicy::for_profile(PdfImportProfile::Constrained);
    assert_eq!(constrained.max_selected_pages, 64);
    assert!(constrained
        .with_limits_json("{\"maxSelectedPages\":65}")
        .is_err());
}

#[test]
fn lower_input_and_backend_budgets_reach_core_and_do_not_poison_next_request() {
    let source = image_pdf();
    for cap in [source.len() - 1, source.len()] {
        let policy = PdfImportPolicy::default()
            .with_limits_json(&json!({"backend": {"inputBytes": cap}}).to_string())
            .unwrap();
        let result =
            rhwp::DocumentCore::from_pdf_bytes_with_policy(&source, &Default::default(), policy);
        assert_eq!(result.is_ok(), cap == source.len());
        if let Err(error) = result {
            assert!(error.to_string().contains("PDF_RESOURCE_LIMIT"));
        }
    }
    let policy = PdfImportPolicy::default()
        .with_limits_json("{\"backend\":{\"objects\":1}}")
        .unwrap();
    assert!(
        rhwp::DocumentCore::from_pdf_bytes_with_policy(&source, &Default::default(), policy)
            .is_err()
    );
    let normal = rhwp::DocumentCore::from_pdf_bytes(&source, &Default::default()).unwrap();
    let explicit = rhwp::DocumentCore::from_pdf_bytes_with_policy(
        &source,
        &Default::default(),
        PdfImportPolicy::default(),
    )
    .unwrap();
    assert_eq!(
        normal.export_hwpx_native().unwrap(),
        explicit.export_hwpx_native().unwrap()
    );
}

#[test]
fn pixel_budgets_use_declared_image_area_and_exact_document_boundary() {
    let source = image_pdf();
    for key in ["maxImagePixels", "maxDocumentImagePixels"] {
        for cap in [7, 8] {
            let policy = PdfImportPolicy::default()
                .with_limits_json(&json!({key: cap}).to_string())
                .unwrap();
            let result = parse_pdf_with_policy(&source, &Default::default(), policy);
            assert_eq!(result.is_ok(), cap == 8, "{key}: {cap}");
            if let Err(error) = result {
                assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
            }
        }
    }
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rhwp-8174-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(input: &Path, output: &Path, config: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("import-pdf")
        .arg(input)
        .arg(output)
        .args(["--resource-limits", config.to_str().unwrap(), "--json"])
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn worker_endpoint_rejects_direct_unsupervised_invocation() {
    let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("__pdf-import-worker-v1")
        .env_remove("RHWP_PDF_WORKER_NONCE")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("PDF_PROCESS_PROTOCOL"));
}

#[test]
fn cli_policy_errors_precede_input_read_and_preserve_existing_output() {
    let dir = Scratch::new();
    let input = dir.file("absent.pdf");
    let output = dir.file("keep.hwpx");
    let config = dir.file("limits.json");
    std::fs::write(&output, b"KEEP").unwrap();
    for invalid in [
        json!([]),
        json!({"policy": []}),
        json!({"policy":{"backend":[]}}),
        json!({"execution": []}),
        json!({"unknown":1}),
        json!({"policy":{"backend":{"inputBytes":0}}}),
        json!({"execution":{"wallTimeoutMs":null}}),
        json!({"execution":{"wallTimeoutMs":0}}),
        json!({"execution":{"outputBytes":134217729u64}}),
        json!({"policy":{"maxSelectedPages":1025}}),
    ] {
        std::fs::write(&config, invalid.to_string()).unwrap();
        let result = run(&input, &output, &config, &[]);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("PDF_RESOURCE_POLICY"));
        assert_eq!(std::fs::read(&output).unwrap(), b"KEEP");
    }
    std::fs::write(
        &config,
        "{\"execution\":{\"wallTimeoutMs\":1,\"wallTimeoutMs\":2}}",
    )
    .unwrap();
    assert_eq!(run(&input, &output, &config, &[]).status.code(), Some(2));
    std::fs::write(&config, "{}").unwrap();
    for flags in [
        vec!["--resource-profile", "unknown"],
        vec!["--resource-profile"],
        vec![
            "--resource-profile",
            "standard",
            "--resource-profile",
            "constrained",
        ],
        vec!["--resource-limits", "extra.json"],
    ] {
        let result = run(&input, &output, &config, &flags);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert_eq!(std::fs::read(&output).unwrap(), b"KEEP");
    }
}

#[test]
fn cli_supervised_limits_atomic_output_and_healthy_followup() {
    let dir = Scratch::new();
    let input = dir.file("input.pdf");
    let source = image_pdf();
    std::fs::write(&input, &source).unwrap();
    let output = dir.file("keep.hwpx");
    let config = dir.file("limits.json");
    for (limits, code) in [
        (
            json!({"policy":{"backend":{"inputBytes":source.len()-1}}}),
            "PDF_RESOURCE_LIMIT",
        ),
        (json!({"policy":{"maxImagePixels":7}}), "PDF_RESOURCE_LIMIT"),
        (
            json!({"execution":{"outputBytes":1}}),
            "PDF_PROCESS_OUTPUT_LIMIT",
        ),
        (
            json!({"execution":{"diagnosticsBytes":1}}),
            "PDF_PROCESS_OUTPUT_LIMIT",
        ),
        (
            json!({"execution":{"wallTimeoutMs":1}}),
            "PDF_PROCESS_TIMEOUT",
        ),
    ] {
        std::fs::write(&output, b"KEEP").unwrap();
        std::fs::write(&config, limits.to_string()).unwrap();
        let result = run(&input, &output, &config, &[]);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(code),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(std::fs::read(&output).unwrap(), b"KEEP");
        std::fs::write(&config, "{}").unwrap();
        let result = run(
            &input,
            &output,
            &config,
            &["--resource-profile", "constrained"],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let envelope: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(envelope["resourceProfile"], "constrained");
        assert_eq!(
            envelope["diagnostics"]["resourcePolicy"]["backend"]["inputBytes"],
            16 * 1024 * 1024
        );
        assert_eq!(
            envelope["execution"]["filesystemNetworkSandbox"],
            cfg!(all(
                target_os = "linux",
                any(target_arch = "x86_64", target_arch = "aarch64")
            ))
        );
        #[cfg(target_os = "linux")]
        assert_eq!(envelope["execution"]["addressSpaceLimitEnforced"], true);
        let saved = std::fs::read(&output).unwrap();
        assert_eq!(envelope["bytes"], saved.len());
        assert!(rhwp::DocumentCore::from_bytes(&saved).is_ok());
    }
    assert_eq!(std::fs::read(&input).unwrap(), source);
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 3);
}

#[test]
fn page_progress_tracks_selected_sources_and_failure_never_reports_completion() {
    use rhwp::parser::pdf::{PdfImportError, PdfImportOptions};
    let source = pdf_objects(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Count 3 /Kids [3 0 R 4 0 R 5 0 R] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] >>".to_vec(),
    ]);
    let options = PdfImportOptions {
        source_pages: Some(vec![0, 2]),
        ..Default::default()
    };
    let mut session = rhwp::PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut events = Vec::new();
    let core = session
        .open_with_progress(&source, &options, &mut |p| {
            events.push(p);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        events
            .iter()
            .map(|p| (p.source_page, p.completed_pages))
            .collect::<Vec<_>>(),
        [(1, 0), (1, 1), (3, 1), (3, 2)]
    );
    assert!(events
        .iter()
        .all(|p| p.total_pages == 2 && p.source_page_count == 3));
    let plain = rhwp::PdfImportSession::new(PdfImportPolicy::default())
        .unwrap()
        .open(&source, &options)
        .unwrap();
    assert_eq!(
        core.export_hwpx_native().unwrap(),
        plain.export_hwpx_native().unwrap(),
        "observation cannot alter the document"
    );

    let mut failed = rhwp::PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut events = Vec::new();
    assert!(failed
        .open_with_progress(&source, &options, &mut |p| {
            events.push(p);
            if p.source_page == 3 {
                Err(PdfImportError {
                    code: "PDF_PROGRESS_FAILED",
                    message: "test observer stopped",
                    diagnostics: None,
                })
            } else {
                Ok(())
            }
        })
        .is_err());
    assert_eq!(events.last().unwrap().completed_pages, 1);
    assert!(failed.open(&source, &options).is_err());
    assert!(rhwp::PdfImportSession::new(PdfImportPolicy::default())
        .unwrap()
        .open(&source, &options)
        .is_ok());
}

#[test]
fn cumulative_work_crosses_wasm32_address_range_without_wrapping_or_raising_a_cap() {
    use pdf_oxide::import_limits::{ImportBudget, PdfImportLimits};
    let standard = PdfImportPolicy::default();
    assert_eq!(standard.backend.decoded_bytes, 8_u64 << 30);
    assert_eq!(
        standard
            .with_limits_json(r#"{"backend":{"decodedBytes":5368709120}}"#)
            .unwrap()
            .backend
            .decoded_bytes,
        5_u64 << 30
    );
    assert!(standard
        .with_limits_json(r#"{"backend":{"decodedBytes":8589934593}}"#)
        .is_err());
    let cap = (1_u64 << 32) + 3;
    let request = ImportBudget::new(PdfImportLimits {
        decoded_bytes: cap,
        ..Default::default()
    })
    .unwrap();
    // Accounting reservations only: no multi-gigabyte allocation is made.
    // Each individual reservation is representable on wasm32.
    request.charge_decoded(1_usize << 31).unwrap();
    request.charge_decoded(1_usize << 31).unwrap();
    request.charge_decoded(3).unwrap();
    assert_eq!(request.snapshot().decoded_bytes, cap);
    assert!(request.charge_decoded(1).is_err());
    let failure = request.snapshot().failure.unwrap();
    assert_eq!(
        (failure.used, failure.requested, failure.limit),
        (Some(cap), Some(1), Some(cap))
    );
    assert_eq!(request.snapshot().decoded_bytes, cap);
    assert!(request.charge_decoded(0).is_err());
    let healthy = ImportBudget::new(PdfImportLimits {
        decoded_bytes: cap,
        ..Default::default()
    })
    .unwrap();
    healthy.charge_decoded(7).unwrap();
    healthy.restrict_work_limits(7, 10).unwrap();
    assert!(healthy.charge_decoded(1).is_err());
}

#[test]
fn unrepresentable_image_dimensions_fail_in_public_import_and_next_request_recovers() {
    use std::io::Write;
    // HWP crop coordinates are signed 32-bit HWPUNIT (1/7200 inch).
    // The import uses 96 dpi: one source pixel occupies 7200/96 = 75 units.
    // These finite, valid Gray streams fit the pixel budget but not that axis.
    let long = i32::MAX as u32 / 75 + 1;
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    let block = [137; 8192];
    let mut remaining = long as usize;
    while remaining > 0 {
        let take = remaining.min(block.len());
        encoder.write_all(&block[..take]).unwrap();
        remaining -= take;
    }
    let pixels = encoder.finish().unwrap();
    for (width, height) in [(long, 1), (1, long)] {
        let content = b"q 40 0 0 20 10 10 cm /Im Do Q";
        let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        stream.extend(content);
        stream.extend(b"\nendstream");
        let mut image = format!("<< /Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n", pixels.len()).into_bytes();
        image.extend(&pixels);
        image.extend(b"\nendstream");
        let source = pdf_objects(&[
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /XObject << /Im 5 0 R >> >> /Contents 4 0 R >>".to_vec(),
            stream, image,
        ]);
        let error = parse_pdf_with_policy(&source, &Default::default(), PdfImportPolicy::default())
            .err()
            .expect("unrepresentable crop axis must not produce a partial document");
        assert_eq!(error.code, "PDF_LOWERING_FAILED");
        assert_eq!(
            error.diagnostics.unwrap().backend_message.as_deref(),
            Some("image dimension exceeds HWP crop coordinate range")
        );
        let healthy = parse_pdf_with_policy(
            &image_pdf(),
            &Default::default(),
            PdfImportPolicy::default(),
        )
        .unwrap();
        assert_eq!(healthy.document.bin_data_content.len(), 1);
    }
}
