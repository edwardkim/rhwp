//! `threat-scan` 구조 위협 탐지 계약.
//!
//! 무기화 문서를 저장소에 커밋하지 않고 **시험 시점에 합성한다**
//! (`tests/issue_2550_bin_data_decompression_bomb.rs` 와 같은 방침). 어떤 픽스처도
//! 실제 악성 코드가 아니라 매직 바이트·손상 헤더 같은 **구조 신호**만 담는다.
//!
//! 고정하는 것:
//! - 내장 실행체(MZ/PE) 스트림 → `embedded_executable` high 신고,
//! - 스트림 밖을 가리키는 레코드 → `malformed_record` high 신고,
//! - 정상 문서 → `clean`(오탐 없음 — 한글 기본 Scripts 스텁에 걸리지 않는다),
//! - HWPX 내장 실행체·원격 외부참조 신고,
//! - 봉투가 `--json` 출처 표지(untrustedContent/untrustedFields)를 실제로 싣는다,
//! - 결정론 — 같은 입력은 같은 봉투.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::queries::threat_scan;

// ── 픽스처 조립 도구 (모두 합성) ────────────────────────────────────────────

/// 256바이트 HWP5 FileHeader. `flags` 로 압축/스크립트 비트를 정한다.
fn file_header(flags: u32) -> Vec<u8> {
    let mut d = vec![0u8; 256];
    d[..17].copy_from_slice(b"HWP Document File");
    d[35] = 5; // major = 5.0
    d[36..40].copy_from_slice(&flags.to_le_bytes());
    d
}

/// 정상 레코드 바이트 (헤더 4B + 데이터). `data.len() < 0xFFF` 를 전제한다.
fn record(tag_id: u16, level: u16, data: &[u8]) -> Vec<u8> {
    let size = data.len() as u32;
    assert!(size < 0xFFF, "픽스처는 작은 레코드만 쓴다");
    let header = (tag_id as u32) | ((level as u32) << 10) | (size << 20);
    let mut out = header.to_le_bytes().to_vec();
    out.extend_from_slice(data);
    out
}

/// 스트림 밖을 가리키는 레코드: `declared` 바이트를 선언하지만 실제 데이터는 그보다 짧다.
fn oversized_record(tag_id: u16, declared: u32, actual: &[u8]) -> Vec<u8> {
    assert!(declared < 0xFFF, "12비트 크기 필드 안에서 오버런을 만든다");
    let header = (tag_id as u32) | (declared << 20);
    let mut out = header.to_le_bytes().to_vec();
    out.extend_from_slice(actual);
    out
}

fn build_hwp5(streams: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let refs: Vec<(&str, &[u8])> = streams.iter().map(|(n, d)| (*n, d.as_slice())).collect();
    rhwp::serializer::mini_cfb::build_cfb(&refs).expect("합성 HWP5 CFB 조립")
}

fn build_hwpx(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut out);
        for (name, data) in entries {
            let method = if *name == "mimetype" {
                zip::CompressionMethod::Stored
            } else {
                zip::CompressionMethod::Deflated
            };
            let opts = SimpleFileOptions::default().compression_method(method);
            zip.start_file(*name, opts).expect("zip 엔트리 시작");
            zip.write_all(data).expect("zip 엔트리 쓰기");
        }
        zip.finish().expect("zip 마감");
    }
    out.into_inner()
}

/// FileHeader + 정상 DocInfo/BodyText + 정상 이미지 BinData 로 이루어진 깨끗한 숙주.
fn clean_hwp5() -> Vec<u8> {
    let doc_info = record(0x10, 0, &[0x01, 0x02, 0x03]); // DOCUMENT_PROPERTIES 모사
    let body = record(0x42, 0, &[0xAA, 0xBB]); // PARA_HEADER 모사
    let bmp = b"BM\x8a\x00\x00\x00 benign bitmap".to_vec();
    build_hwp5(&[
        ("/FileHeader", file_header(0)),
        ("/DocInfo", doc_info),
        ("/BodyText/Section0", body),
        ("/BinData/BIN0001.bmp", bmp),
    ])
}

/// 실행 파일 매직 페이로드 — **실제 악성 코드가 아니라** MZ/PE 헤더 모양뿐이다.
fn fake_pe_bytes() -> Vec<u8> {
    let mut v = b"MZ\x90\x00\x03\x00\x00\x00".to_vec();
    v.extend_from_slice(&[0u8; 56]);
    v.extend_from_slice(b"PE\x00\x00"); // PE 시그니처 모양
    v.extend_from_slice(&[0u8; 32]);
    v
}

// ── 탐지 계약 ───────────────────────────────────────────────────────────────

#[test]
fn clean_hwp5_document_is_reported_clean() {
    let report = threat_scan::scan_bytes("clean.hwp", &clean_hwp5());
    assert_eq!(report.format, "hwp5");
    assert!(
        report.clean(),
        "정상 문서는 clean 이어야 한다(한글 기본 Scripts 스텁·정상 이미지에 걸리면 안 된다): {:?}",
        report.findings
    );
}

#[test]
fn embedded_pe_in_bindata_is_flagged_high() {
    let mut streams = vec![
        ("/FileHeader", file_header(0)),
        ("/DocInfo", record(0x10, 0, &[0x01])),
        ("/BodyText/Section0", record(0x42, 0, &[0xAA])),
        ("/BinData/BIN0002.OLE", fake_pe_bytes()),
    ];
    // 정상 이미지도 함께 둬서, 걸리는 것이 실행체 스트림뿐임을 본다.
    streams.push(("/BinData/BIN0001.bmp", b"BM benign".to_vec()));
    let report = threat_scan::scan_bytes("evil.hwp", &build_hwp5(&streams));

    let hits: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.kind == "embedded_executable")
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "실행체 스트림 하나만 걸려야 한다: {:?}",
        report.findings
    );
    assert_eq!(hits[0].severity, "high");
    assert!(
        hits[0].location.contains("BIN0002.OLE"),
        "주소가 실행체 스트림을 가리켜야 한다: {}",
        hits[0].location
    );
    assert!(
        hits[0].detail.is_none(),
        "실행체 신고에는 문서 파생 detail 이 없다"
    );
}

#[test]
fn malformed_oversized_record_is_flagged_high() {
    // DocInfo 에 스트림 밖을 가리키는 레코드(선언 100B, 실제 2B).
    let streams = vec![
        ("/FileHeader", file_header(0)),
        ("/DocInfo", oversized_record(0x10, 100, &[0xAA, 0xBB])),
        ("/BodyText/Section0", record(0x42, 0, &[0x00])),
    ];
    let report = threat_scan::scan_bytes("malformed.hwp", &build_hwp5(&streams));

    let hits: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.kind == "malformed_record")
        .collect();
    assert!(
        !hits.is_empty(),
        "스트림 밖을 가리키는 레코드는 malformed_record 로 신고돼야 한다: {:?}",
        report.findings
    );
    assert_eq!(hits[0].severity, "high");
    assert!(
        hits[0].location.contains("DocInfo"),
        "주소: {}",
        hits[0].location
    );
}

#[test]
fn script_flag_is_flagged_but_default_stub_is_not() {
    // script 비트(0x08)를 켠 문서 → macro_script.
    let with_flag = build_hwp5(&[
        ("/FileHeader", file_header(0x08)),
        ("/DocInfo", record(0x10, 0, &[0x01])),
        ("/BodyText/Section0", record(0x42, 0, &[0xAA])),
        // 한글 기본 스텁을 흉내낸 작은 Scripts 스트림 — 플래그가 꺼지면 걸리면 안 된다.
        ("/Scripts/DefaultJScript", vec![0u8; 16]),
    ]);
    let report = threat_scan::scan_bytes("macro.hwp", &with_flag);
    assert!(
        report.findings.iter().any(|f| f.kind == "macro_script"),
        "script 플래그가 켜지면 macro_script 로 신고돼야 한다: {:?}",
        report.findings
    );

    // 플래그가 꺼진 채 기본 Scripts 스텁만 있는 문서 → 걸리면 안 된다(오탐 가드).
    let stub_only = build_hwp5(&[
        ("/FileHeader", file_header(0)),
        ("/DocInfo", record(0x10, 0, &[0x01])),
        ("/BodyText/Section0", record(0x42, 0, &[0xAA])),
        ("/Scripts/DefaultJScript", vec![0u8; 16]),
    ]);
    let report = threat_scan::scan_bytes("stub.hwp", &stub_only);
    assert!(
        !report.findings.iter().any(|f| f.kind == "macro_script"),
        "기본 Scripts 스텁(플래그 꺼짐)은 macro_script 로 걸리면 안 된다: {:?}",
        report.findings
    );
}

#[test]
fn hwpx_embedded_pe_is_flagged() {
    let hwpx = build_hwpx(&[
        ("mimetype", b"application/hwp+zip".to_vec()),
        ("BinData/evil.bin", fake_pe_bytes()),
    ]);
    let report = threat_scan::scan_bytes("evil.hwpx", &hwpx);
    assert_eq!(report.format, "hwpx");
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.kind == "embedded_executable" && f.location.contains("BinData/evil.bin")),
        "HWPX BinData 실행체가 신고돼야 한다: {:?}",
        report.findings
    );
}

#[test]
fn hwpx_remote_external_reference_is_flagged_with_untrusted_detail() {
    let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest>
  <item id="ext1" href="http://malicious.example/payload.dll" media-type="application/octet-stream" isEmbeded="0"/>
</manifest>"#;
    let hwpx = build_hwpx(&[
        ("mimetype", b"application/hwp+zip".to_vec()),
        ("Contents/content.hpf", manifest.as_bytes().to_vec()),
    ]);
    let report = threat_scan::scan_bytes("linked.hwpx", &hwpx);
    let hit = report
        .findings
        .iter()
        .find(|f| f.kind == "external_reference")
        .expect("원격 외부참조가 신고돼야 한다");
    assert_eq!(hit.severity, "medium");
    assert_eq!(
        hit.detail.as_deref(),
        Some("http://malicious.example/payload.dll"),
        "detail 은 문서가 정한 원격 대상을 담아야 한다"
    );
}

#[test]
fn scan_is_deterministic() {
    let bytes = clean_hwp5();
    let a = threat_scan::envelope(&threat_scan::scan_bytes("x.hwp", &bytes));
    let b = threat_scan::envelope(&threat_scan::scan_bytes("x.hwp", &bytes));
    assert_eq!(
        a.to_string(),
        b.to_string(),
        "같은 입력은 같은 봉투여야 한다"
    );
}

// ── CLI 봉투·출처 표지 계약 (실제 바이너리) ─────────────────────────────────

fn run_cli(path: &std::path::Path, args: &[&str]) -> (i32, String) {
    let mut full = vec!["threat-scan", path.to_str().unwrap()];
    full.extend_from_slice(args);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(&full)
        .output()
        .expect("rhwp 실행");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

fn temp_write(name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp_threat_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("작업 폴더");
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("픽스처 쓰기");
    path
}

#[test]
fn cli_json_envelope_carries_provenance_flag_without_doc_strings() {
    // 실행체 신고에는 문서 파생 문자열이 없으므로 untrustedContent=false 여야 한다.
    let mut streams = vec![
        ("/FileHeader", file_header(0)),
        ("/DocInfo", record(0x10, 0, &[0x01])),
        ("/BodyText/Section0", record(0x42, 0, &[0xAA])),
        ("/BinData/BIN0002.OLE", fake_pe_bytes()),
    ];
    streams.push(("/BinData/BIN0001.bmp", b"BM benign".to_vec()));
    let path = temp_write("pe.hwp", &build_hwp5(&streams));

    let (code, stdout) = run_cli(&path, &["--json"]);
    assert_eq!(code, 0, "스캔 성공은 exit 0(판정은 봉투 데이터): {stdout}");
    let env: serde_json::Value = serde_json::from_str(&stdout).expect("봉투 JSON");
    assert_eq!(env["schemaVersion"], "1.0");
    assert_eq!(env["clean"], false);
    assert!(env["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["kind"] == "embedded_executable"));
    // 표지는 늘 실린다. 실행체 신고에는 detail 이 없어 문서 파생 값이 없다.
    assert_eq!(
        env["untrustedContent"], false,
        "실행체 신고에는 문서 파생 문자열이 없다: {env}"
    );
    assert_eq!(env["untrustedFields"], serde_json::json!([]));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn cli_json_envelope_marks_external_reference_detail_untrusted() {
    let manifest = r#"<?xml version="1.0"?>
<manifest><item id="e" href="https://evil.example/x" media-type="application/octet-stream" isEmbeded="0"/></manifest>"#;
    let hwpx = build_hwpx(&[
        ("mimetype", b"application/hwp+zip".to_vec()),
        ("Contents/content.hpf", manifest.as_bytes().to_vec()),
    ]);
    let path = temp_write("ext.hwpx", &hwpx);

    let (code, stdout) = run_cli(&path, &["--json"]);
    assert_eq!(code, 0, "{stdout}");
    let env: serde_json::Value = serde_json::from_str(&stdout).expect("봉투 JSON");
    assert_eq!(
        env["untrustedContent"], true,
        "외부참조 대상(URL)은 문서 파생이라 표지가 켜져야 한다: {env}"
    );
    assert_eq!(
        env["untrustedFields"],
        serde_json::json!(["findings[].detail"]),
        "출처 표지는 findings[].detail 을 가리켜야 한다: {env}"
    );
    let _ = std::fs::remove_file(&path);
}
