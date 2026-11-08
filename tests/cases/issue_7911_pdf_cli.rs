//! 관계형·고정형 합성 Parser/IR·CLI 계약: 내용 보존·오류·선택 쪽수. 시각 좌표 golden이 아니다.
//! 주석 갱신: 2026-11-01T00:29:10+09:00 (Asia/Seoul).
//! 직전 파일 변경: 4662c069e557e928ed7968d061121fde90b1f027 / 2026-10-16T15:17:54+09:00.
//! #8415 인증 제어군: 독립 평문 PDF 1쪽, 2026-11-01 96dpi 보정 전 2px 실루엣.
//! Native/fresh WASM 각 100.00%, 전쪽 최저 각 100.00%; 본문·이미지 직접 판독 완료.
//! 근거: mydocs/working/assets/issue8415/password-authentication/ 및 task_m100_8413_implementation.md.
//! 이 수치는 #8415 새 암호 제어군에 한정하며 기존 합성 계약을 실제 문서 시각 검증으로 승격하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rhwp-pdf-cli-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(args: &[&str]) -> Output {
    let binary =
        std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
    Command::new(binary).args(args).output().unwrap()
}
fn path(p: &Path) -> &str {
    p.to_str().unwrap()
}
fn success(output: Output) -> Value {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout is one complete JSON value")
}
fn failure(output: Output, code: i32, contains: &str) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "no partial success JSON");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(contains),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn pdf(pages: &[(&str, &str)]) -> Vec<u8> {
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        String::new(),
    ];
    let mut kids = Vec::new();
    for (text, font) in pages {
        let page = objects.len() + 1;
        kids.push(format!("{page} 0 R"));
        let content = format!("BT /F1 12 Tf 30 140 Td ({text}) Tj ET");
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 {} 0 R >> >> /Contents {} 0 R >>", page + 2, page + 1));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            content.len(),
            content
        ));
        objects.push(format!(
            "<< /Type /Font /Subtype /Type1 /BaseFont /{font} >>"
        ));
    }
    objects[1] = format!(
        "<< /Type /Pages /Count {} /Kids [{}] >>",
        pages.len(),
        kids.join(" ")
    );
    let mut output = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        offsets.push(output.len());
        output.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, object).as_bytes());
    }
    let xref = output.len();
    output.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        output.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    output.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    output
}

#[test]
fn pdf_cli_help_and_feature_availability_are_honest() {
    let help = run(&["import-pdf", "--help"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(help.stderr.is_empty());
    let text = String::from_utf8(help.stdout).unwrap();
    for contract in [
        "--source-pages",
        "0-based",
        "pdf-import",
        "--json",
        "exit 2",
    ] {
        assert!(text.contains(contract), "missing {contract}");
    }
    let caps = success(run(&["capabilities"]));
    let command = caps["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "import-pdf")
        .unwrap();
    assert_eq!(command["available"], cfg!(feature = "pdf-import"));
    assert_eq!(command["requiresFeature"], "pdf-import");
    assert_eq!(command["json"], true);
    assert_eq!(command["batch"], false);
    assert_eq!(
        caps["formats"]["read"]
            .as_array()
            .unwrap()
            .contains(&json!("pdf")),
        cfg!(feature = "pdf-import")
    );
    let mcp = success(run(&["capabilities", "--mcp"]));
    let tool = mcp["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "hwp_import_pdf")
        .unwrap();
    assert_eq!(tool["inputSchema"]["required"], json!(["path", "output"]));
    assert_eq!(
        tool["cli"]["optionalArgs"][0]["args"],
        json!(["--source-pages", "{sourcePages}"])
    );
}

#[test]
fn pdf_cli_rejects_invalid_arguments_without_touching_output() {
    let dir = Scratch::new();
    let output = dir.path("existing.hwpx");
    std::fs::write(&output, b"keep original output").unwrap();
    for selection in [
        "",
        "0,",
        ",0",
        "1,1",
        "2,1",
        "2-1",
        "0-2,2",
        "-1",
        "+1",
        "1--2",
        "0.5",
        "0, 2",
        "999999999999999999999999999",
    ] {
        failure(
            run(&[
                "import-pdf",
                "absent.pdf",
                path(&output),
                "--source-pages",
                selection,
                "--json",
            ]),
            2,
            "PDF_PAGE_SELECTION",
        );
    }
    for flags in [
        vec!["--page", "0"],
        vec!["--json", "--json"],
        vec!["--source-pages"],
        vec!["--source-pages", "0", "--source-pages", "1"],
        vec!["extra.pdf"],
    ] {
        let mut args = vec!["import-pdf", "absent.pdf", path(&output)];
        args.extend(flags);
        failure(run(&args), 2, "오류");
    }
    failure(
        run(&[
            "import-pdf",
            "absent.pdf",
            path(&output),
            "--output-password",
            "secret",
        ]),
        2,
        "출력 암호",
    );
    assert_eq!(std::fs::read(&output).unwrap(), b"keep original output");
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 1);
}

#[cfg(feature = "pdf-import")]
#[test]
fn pdf_cli_password_stdin_retries_without_replacing_output_on_authentication_failure() {
    use std::io::Write;
    use std::process::Stdio;
    let dir = Scratch::new();
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue8415/authentication/password-r6.pdf");
    let output = dir.path("authenticated.hwpx");
    std::fs::write(&output, b"keep destination").unwrap();
    failure(
        run(&["import-pdf", path(&input), path(&output), "--json"]),
        2,
        "PDF_PASSWORD_REQUIRED",
    );
    assert_eq!(std::fs::read(&output).unwrap(), b"keep destination");
    let with_password = |password: &str| {
        let binary = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
        let mut child = Command::new(binary)
            .args([
                "import-pdf",
                path(&input),
                path(&output),
                "--json",
                "--password-stdin",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        writeln!(child.stdin.take().unwrap(), "{password}").unwrap();
        child.wait_with_output().unwrap()
    };
    failure(with_password("wrong"), 1, "PDF_PASSWORD_INVALID");
    assert_eq!(std::fs::read(&output).unwrap(), b"keep destination");
    let envelope = success(with_password("synthetic-open"));
    assert!(!envelope.to_string().contains("synthetic-open"));
    let saved = std::fs::read(&output).unwrap();
    assert_eq!(envelope["bytes"], saved.len());
    assert!(rhwp::document_core::DocumentCore::from_bytes(&saved).is_ok());
}

#[cfg(not(feature = "pdf-import"))]
#[test]
fn pdf_cli_default_build_explains_disabled_import() {
    let dir = Scratch::new();
    let input = dir.path("input.pdf");
    let output = dir.path("result.hwpx");
    std::fs::write(&input, pdf(&[("FIRST", "Helvetica")])).unwrap();
    failure(
        run(&["import-pdf", path(&input), path(&output), "--json"]),
        1,
        "PDF_IMPORT_DISABLED",
    );
    assert!(!output.exists());
}

#[cfg(feature = "pdf-import")]
#[test]
fn pdf_cli_selects_pages_and_reopens_both_output_formats() {
    let dir = Scratch::new();
    let input = dir.path("input.pdf");
    let bytes = pdf(&[
        ("FIRST", "Helvetica"),
        ("SECOND", "Courier"),
        ("THIRD", "Helvetica"),
    ]);
    std::fs::write(&input, &bytes).unwrap();
    for extension in ["hwpx", "hwp"] {
        let output = dir.path(&format!("result.{extension}"));
        std::fs::write(&output, b"old output").unwrap();
        let envelope = success(run(&[
            "import-pdf",
            "--json",
            "--source-pages",
            "0,2-2",
            path(&input),
            path(&output),
        ]));
        assert_eq!(
            envelope["schemaVersion"],
            rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION
        );
        assert_eq!(
            envelope["format"],
            if extension == "hwp" { "hwp5" } else { "hwpx" }
        );
        assert_eq!(envelope["pageCount"], 2);
        assert_eq!(envelope["diagnostics"]["sourcePageCount"], 3);
        assert_eq!(
            envelope["diagnostics"]["parserAllocationBoundsProven"],
            false
        );
        assert!(envelope["diagnostics"]["paragraphPolicy"].is_string());
        let pages = envelope["diagnostics"]["pages"].as_array().unwrap();
        assert_eq!(
            pages
                .iter()
                .map(|p| p["sourcePage"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            vec![0, 2]
        );
        assert_eq!(
            pages
                .iter()
                .map(|p| p["outputSection"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(envelope["untrustedContent"], true);
        assert_eq!(envelope["untrustedFields"], json!(["diagnostics.pages"]));
        let saved = std::fs::read(&output).unwrap();
        assert_eq!(envelope["bytes"], saved.len());
        let doc = rhwp::document_core::DocumentCore::from_bytes(&saved).unwrap();
        assert_eq!(doc.page_count(), 2);
        for (page, expected) in ["FIRST", "THIRD"].iter().enumerate() {
            let svg = doc.render_page_svg_native(page as u32).unwrap();
            let xml = roxmltree::Document::parse(&svg).unwrap();
            let text: String = xml
                .descendants()
                .filter(|n| n.is_text() && n.ancestors().any(|p| p.has_tag_name("text")))
                .filter_map(|n| n.text())
                .collect();
            assert_eq!(text, *expected);
        }
    }
    let output = dir.path("all.hwpx");
    let envelope = success(run(&["import-pdf", path(&input), path(&output), "--json"]));
    assert_eq!(envelope["pageCount"], 3);
    assert_eq!(
        envelope["diagnostics"]["pages"].as_array().unwrap().len(),
        3
    );
    assert_eq!(std::fs::read(&input).unwrap(), bytes);
    assert_eq!(
        std::fs::read_dir(&dir.0).unwrap().count(),
        4,
        "no sibling temporary files"
    );
}

#[cfg(feature = "pdf-import")]
#[test]
fn pdf_cli_runtime_and_page_selection_failures_preserve_destination() {
    let dir = Scratch::new();
    let input = dir.path("input.pdf");
    let output = dir.path("output.hwpx");
    let original = pdf(&[("FIRST", "Helvetica")]);
    std::fs::write(&input, &original).unwrap();
    std::fs::write(&output, b"keep destination").unwrap();
    for pages in ["1", "0-128", "0-18446744073709551615"] {
        failure(
            run(&[
                "import-pdf",
                path(&input),
                path(&output),
                "--source-pages",
                pages,
                "--json",
            ]),
            2,
            "PDF_PAGE_SELECTION",
        );
    }
    for (bytes, code) in [
        (b"not PDF".to_vec(), "PDF_INVALID_INPUT"),
        (b"%PDF-1.4\ninvalid".to_vec(), "PDF_PARSE_FAILED"),
        (
            vec![0; rhwp::parser::pdf::MAX_INPUT_BYTES + 1],
            "PDF_RESOURCE_LIMIT",
        ),
    ] {
        std::fs::write(&input, bytes).unwrap();
        failure(
            run(&["import-pdf", path(&input), path(&output), "--json"]),
            1,
            code,
        );
        assert_eq!(std::fs::read(&output).unwrap(), b"keep destination");
    }
    std::fs::write(&input, &original).unwrap();
    let directory = dir.path("directory.hwpx");
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("keep"), b"keep").unwrap();
    failure(
        run(&["import-pdf", path(&input), path(&directory), "--json"]),
        1,
        "오류",
    );
    assert_eq!(
        std::fs::read_dir(&dir.0).unwrap().count(),
        3,
        "failed rename removes temp"
    );
    assert_eq!(std::fs::read(directory.join("keep")).unwrap(), b"keep");
    // Failed invocation does not poison a subsequent successful import.
    success(run(&["import-pdf", path(&input), path(&output), "--json"]));
}

#[cfg(unix)]
#[test]
fn pdf_cli_refuses_input_aliases_even_with_an_output_extension() {
    let dir = Scratch::new();
    let input = dir.path("input.hwpx");
    let bytes = pdf(&[("FIRST", "Helvetica")]);
    std::fs::write(&input, &bytes).unwrap();
    let symlink = dir.path("symlink.hwpx");
    let hardlink = dir.path("hardlink.hwp");
    std::os::unix::fs::symlink(&input, &symlink).unwrap();
    std::fs::hard_link(&input, &hardlink).unwrap();
    for alias in [&input, &symlink, &hardlink] {
        failure(
            run(&["import-pdf", path(&input), path(alias), "--json"]),
            2,
            "원본",
        );
    }
    assert_eq!(std::fs::read(&input).unwrap(), bytes);
}
