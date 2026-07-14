use std::path::Path;
use std::process::{Command, Output};

use rhwp::document_core::DocumentCore;
use rhwp::parser::{detect_format, FileFormat};

const FIXTURE: &str = "samples/hml/formatting_table.hml";

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
}

fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("rhwp_hml_{label}_{}_{nonce}", std::process::id()))
}

fn run_export_hml(input: &Path, output: &Path, output_flag: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-hml")
        .arg(input)
        .arg(output_flag)
        .arg(output)
        .output()
        .expect("run rhwp export-hml")
}

fn first_non_empty_paragraph(core: &DocumentCore) -> (usize, usize) {
    core.document()
        .sections
        .iter()
        .enumerate()
        .find_map(|(section_index, section)| {
            section
                .paragraphs
                .iter()
                .position(|paragraph| !paragraph.text.is_empty())
                .map(|paragraph_index| (section_index, paragraph_index))
        })
        .expect("fixture should contain text")
}

#[test]
fn document_core_loads_hml_with_xml_import_normalization() {
    let bytes = std::fs::read(fixture_path()).expect("read real HML fixture");
    assert_eq!(detect_format(&bytes), FileFormat::Hml);
    let core = DocumentCore::from_bytes(&bytes).expect("load HML through shared DocumentCore");

    let non_empty_paragraphs = core
        .document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .filter(|paragraph| !paragraph.text.is_empty())
        .collect::<Vec<_>>();
    assert!(!non_empty_paragraphs.is_empty());
    assert!(
        non_empty_paragraphs
            .iter()
            .all(|paragraph| !paragraph.line_segs.is_empty()),
        "XML imports must synthesize missing line segments before layout"
    );
}

#[test]
fn document_core_preserves_real_hml_page_breaks() {
    let bytes = std::fs::read("samples/hml/aligns.hml").expect("read alignment HML fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("load alignment HML fixture");

    assert_eq!(core.page_count(), 16);
}

#[test]
fn info_reports_hml_contract_fields() {
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("info")
        .arg(fixture_path())
        .output()
        .expect("run rhwp info");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "rhwp info failed: {stderr}");
    assert!(stdout.contains("format: HML"), "missing format: {stdout}");
    assert!(
        stdout.contains("hwpml_version: 2.91"),
        "missing HWPML version: {stdout}"
    );
    assert!(stdout.contains("sections: 1"), "missing sections: {stdout}");
    assert!(stdout.contains("pages: "), "missing pages: {stdout}");
    let expected_size = std::fs::metadata(fixture_path())
        .expect("real HML fixture metadata")
        .len();
    assert!(
        stdout.contains(&format!("크기: {expected_size} bytes")),
        "missing file size: {stdout}"
    );
    for diagnostic in ["구역 수: 1", "페이지 수: 1", "스타일:"] {
        assert!(
            stdout.contains(diagnostic),
            "missing shared document diagnostic `{diagnostic}`: {stdout}"
        );
    }
    assert!(
        stdout.contains("encoding: UTF-8"),
        "missing encoding: {stdout}"
    );
    assert!(
        stdout.contains("resources: 0"),
        "missing resources: {stdout}"
    );
    assert!(
        stdout.contains("warnings: 2"),
        "missing warning count: {stdout}"
    );
    for synthetic_hwp_field in ["버전:", "압축:", "암호화:", "배포용:"] {
        assert!(
            !stdout
                .lines()
                .any(|line| line.starts_with(synthetic_hwp_field)),
            "HML info must not expose synthetic HWP header field `{synthetic_hwp_field}`: {stdout}"
        );
    }
    assert!(
        stderr.contains("/HWPML/TAIL/SCRIPTCODE"),
        "warning path must be written to stderr: {stderr}"
    );
}

#[test]
fn help_lists_hml_for_supported_document_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("--help")
        .output()
        .expect("run rhwp --help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    for command in ["export-svg", "export-pdf", "info", "dump"] {
        let expected = format!("{command} <파일.hwp|파일.hwpx|파일.hml>");
        assert!(stdout.contains(&expected), "missing `{expected}` in help");
    }
    assert!(
        stdout.contains("export-hml <입력.hml> -o <출력.hml>"),
        "missing export-hml usage: {stdout}"
    );
}

#[test]
fn export_hml_flags_preserve_edit_reparse_and_raw_fragment() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("load lawful HML");
    let (section_index, paragraph_index) = first_non_empty_paragraph(&core);
    core.insert_text_native(section_index, paragraph_index, 0, "S3_EDIT_")
        .expect("apply public edit");
    let edited = core.export_hml_native().expect("serialize edited input");
    let input = unique_temp_dir("cli_success_input").with_extension("hml");
    std::fs::write(&input, &edited).expect("write edited HML input");

    for output_flag in ["-o", "--output"] {
        let output = unique_temp_dir("cli_success_output").with_extension("hml");
        let command = run_export_hml(&input, &output, output_flag);
        assert!(
            command.status.success(),
            "export-hml failed: {}",
            String::from_utf8_lossy(&command.stderr)
        );
        let bytes = std::fs::read(&output).expect("read CLI HML output");
        assert_eq!(detect_format(&bytes), FileFormat::Hml);
        assert!(
            String::from_utf8_lossy(&bytes).contains("<SCRIPTCODE"),
            "preserved TAIL fragment missing"
        );
        let reparsed = DocumentCore::from_bytes(&bytes).expect("reparse CLI HML output");
        assert!(
            reparsed.document().sections[section_index].paragraphs[paragraph_index]
                .text
                .starts_with("S3_EDIT_")
        );
        assert_eq!(
            reparsed.hml_metadata().expect("output metadata").warnings,
            core.hml_metadata().expect("input metadata").warnings
        );
    }
}

#[test]
fn export_hml_refusals_are_nonzero_structured_and_write_nothing() {
    let exe = env!("CARGO_BIN_EXE_rhwp");
    let non_hml_output = unique_temp_dir("cli_non_hml").with_extension("hml");
    let non_hml = Command::new(exe)
        .arg("export-hml")
        .arg("samples/re-align-center-hancom.hwp")
        .arg("-o")
        .arg(&non_hml_output)
        .output()
        .expect("run non-HML refusal");
    let non_hml_stderr = String::from_utf8_lossy(&non_hml.stderr);
    assert!(!non_hml.status.success());
    assert!(!non_hml_output.exists());
    for field in ["HML_SOURCE_REQUIRED", "/HWPML", "HML 원본 문서"] {
        assert!(
            non_hml_stderr.contains(field),
            "missing {field}: {non_hml_stderr}"
        );
    }

    let fixture = std::str::from_utf8(include_bytes!("../samples/hml/formatting_table.hml"))
        .expect("fixture is UTF-8");
    let lossy_bytes = fixture.replacen("Type=\"None\"", "Type=\"Dash\"", 1);
    let lossy_input = unique_temp_dir("cli_lossy_input").with_extension("hml");
    let lossy_output = unique_temp_dir("cli_lossy_output").with_extension("hml");
    std::fs::write(&lossy_input, lossy_bytes).expect("write lossy HML");
    let expected = DocumentCore::from_bytes(&std::fs::read(&lossy_input).unwrap())
        .expect("parse lossy HML")
        .hml_export_preflight()
        .expect_err("lossy import must block")
        .blockers()[0]
        .clone();
    let lossy = run_export_hml(&lossy_input, &lossy_output, "--output");
    let lossy_stderr = String::from_utf8_lossy(&lossy.stderr);
    assert!(!lossy.status.success());
    assert!(!lossy_output.exists());
    for field in [expected.code, &expected.xml_path, &expected.message] {
        assert!(
            lossy_stderr.contains(field),
            "missing {field}: {lossy_stderr}"
        );
    }
}

#[test]
fn export_hml_never_overwrites_its_input() {
    let input = unique_temp_dir("cli_same_path").with_extension("hml");
    let original = include_bytes!("../samples/hml/formatting_table.hml");
    std::fs::write(&input, original).expect("write HML input");

    let output = run_export_hml(&input, &input, "-o");

    assert!(!output.status.success());
    assert_eq!(
        std::fs::read(&input).expect("read unchanged input"),
        original
    );
}

#[cfg(unix)]
#[test]
fn export_hml_never_overwrites_a_hard_link_to_its_input() {
    let input = unique_temp_dir("cli_hard_link_input").with_extension("hml");
    let output = unique_temp_dir("cli_hard_link_output").with_extension("hml");
    let original = include_bytes!("../samples/hml/formatting_table.hml");
    std::fs::write(&input, original).expect("write HML input");
    std::fs::hard_link(&input, &output).expect("create hard-link output alias");

    let command = run_export_hml(&input, &output, "-o");

    assert_eq!(command.status.code(), Some(2));
    assert_eq!(
        std::fs::read(&input).expect("read unchanged input"),
        original
    );
}

#[test]
fn export_hml_rejects_an_option_token_as_the_output_value() {
    let sandbox = unique_temp_dir("cli_output_option");
    std::fs::create_dir_all(&sandbox).expect("create isolated working directory");
    let command = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .current_dir(&sandbox)
        .arg("export-hml")
        .arg(fixture_path())
        .args(["-o", "--bogus"])
        .output()
        .expect("run rhwp export-hml");
    let stderr = String::from_utf8_lossy(&command.stderr);

    assert_eq!(command.status.code(), Some(2));
    assert!(stderr.contains("출력 경로가 필요합니다"), "{stderr}");
    assert!(stderr.contains("사용법:"), "{stderr}");
    assert!(!sandbox.join("--bogus").exists());
}

#[test]
fn export_hml_failed_replacement_leaves_no_temporary_output() {
    let sandbox = unique_temp_dir("cli_atomic_failure");
    std::fs::create_dir_all(&sandbox).expect("create isolated output directory");
    let destination = sandbox.join("destination.hml");
    std::fs::create_dir(&destination).expect("create non-replaceable destination");
    std::fs::write(destination.join("sentinel"), b"old destination")
        .expect("seed destination sentinel");

    let command = run_export_hml(&fixture_path(), &destination, "-o");

    assert_eq!(command.status.code(), Some(1));
    assert_eq!(
        std::fs::read(destination.join("sentinel")).expect("read preserved sentinel"),
        b"old destination"
    );
    assert_eq!(
        std::fs::read_dir(&sandbox)
            .expect("read output directory")
            .filter_map(Result::ok)
            .count(),
        1,
        "failed replacement must remove its sibling temporary file"
    );
}

#[cfg(unix)]
#[test]
fn export_hml_accepts_a_near_name_max_output_basename() {
    let sandbox = unique_temp_dir("cli_near_name_max");
    std::fs::create_dir_all(&sandbox).expect("create output directory");
    let output = sandbox.join(format!("{}.hml", "x".repeat(220)));

    let command = run_export_hml(&fixture_path(), &output, "-o");

    assert!(
        command.status.success(),
        "export-hml failed: {}",
        String::from_utf8_lossy(&command.stderr)
    );
    let bytes = std::fs::read(&output).expect("read near-NAME_MAX output");
    assert_eq!(detect_format(&bytes), FileFormat::Hml);
    DocumentCore::from_bytes(&bytes).expect("near-NAME_MAX output should reparse");
}

#[test]
fn dump_svg_and_pdf_commands_accept_hml() {
    let exe = env!("CARGO_BIN_EXE_rhwp");
    let fixture = fixture_path();

    let dump = Command::new(exe)
        .arg("dump")
        .arg(&fixture)
        .output()
        .expect("run rhwp dump");
    assert!(
        dump.status.success() && dump.stderr.is_empty() && !dump.stdout.is_empty(),
        "HML dump failed: {}",
        String::from_utf8_lossy(&dump.stderr)
    );

    let svg_dir = unique_temp_dir("svg");
    let svg = Command::new(exe)
        .arg("export-svg")
        .arg(&fixture)
        .args(["--output", svg_dir.to_str().expect("UTF-8 temp path")])
        .output()
        .expect("run rhwp export-svg");
    assert!(svg.status.success(), "SVG command failed");
    let svg_count = std::fs::read_dir(&svg_dir)
        .expect("SVG output directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("svg"))
        .count();
    assert!(svg_count > 0, "HML export-svg produced no SVG pages");

    let pdf_path = unique_temp_dir("pdf").with_extension("pdf");
    let pdf = Command::new(exe)
        .arg("export-pdf")
        .arg(&fixture)
        .args(["--output", pdf_path.to_str().expect("UTF-8 temp path")])
        .output()
        .expect("run rhwp export-pdf");
    assert!(pdf.status.success(), "PDF command failed");
    let pdf_bytes = std::fs::read(&pdf_path).expect("PDF output");
    assert!(pdf_bytes.starts_with(b"%PDF-"), "invalid PDF output");
}
