//! [password-mcp-parity] CLI `--password` 배선 회귀 가드.
//!
//! `src/main.rs::supports_password_stdin()` 는 MCP 도구에 `cli.passwordStdin`
//! 계약을 붙일지 결정한다 — 즉 "이 명령은 전역 `--password`/`--password-stdin`
//! 을 실제로 소비한다"는 약속이다. 그런데 `fields`/`search`/`export-tables`/
//! `extract-pages`/`edit fill-fields`/`edit replace-text`/`edit set-cell` 여섯
//! 명령은 `HwpDocument::from_bytes` 를 직접 호출해 전역 비밀번호(thread-local
//! `CLI_PASSWORD`)를 무시하고 있었다 — MCP 계약은 암호 지원을 선전하지만 CLI는
//! 암호 문서를 절대 열지 못하는 정합 붕괴였다. `load_document()` 로 배선을
//! 맞춘 뒤, 이 테스트가 암호 fixture 로 각 명령을 직접 실행해 그 배선이 실제로
//! 살아 있는지 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PASSWORD: &str = "123456";
const WRONG_PASSWORD: &str = "wrong-password-must-not-echo";
/// HWP5, 비밀번호 "123456", 23쪽. `tests/mcp_password_contract.rs` 의
/// FIXTURES[2] 와 동일한 fixture — 이미 검증된 실측 픽스처를 재사용한다.
const FIXTURE: &str = "samples/HWP5-password-123456.hwpx";

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
}

/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

/// `--password` 로 열리는 여섯 회귀 대상 명령. 각각 최소 성공 형태로 호출한다.
/// (path, output 인자 필요 여부에 따른 임시파일 접미어)
fn regression_commands(path: &str, out_dir: &Path) -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "fields",
            vec!["fields".into(), path.into(), "--json".into()],
        ),
        (
            "search",
            vec!["search".into(), path.into(), "test".into(), "--json".into()],
        ),
        (
            "export-tables",
            vec!["export-tables".into(), path.into(), "--json".into()],
        ),
        (
            "extract-pages",
            vec![
                "extract-pages".into(),
                path.into(),
                out_dir.join("extract-pages.hwpx").display().to_string(),
                "--from".into(),
                "1".into(),
                "--to".into(),
                "1".into(),
                "--json".into(),
            ],
        ),
        (
            "edit fill-fields",
            vec![
                "edit".into(),
                "fill-fields".into(),
                path.into(),
                "--data".into(),
                "{}".into(),
                "-o".into(),
                out_dir.join("fill-fields.hwpx").display().to_string(),
                "--json".into(),
            ],
        ),
        (
            "edit replace-text",
            vec![
                "edit".into(),
                "replace-text".into(),
                path.into(),
                "--find".into(),
                "존재하지않는문자열-xyz".into(),
                "--replace".into(),
                "y".into(),
                "-o".into(),
                out_dir.join("replace-text.hwpx").display().to_string(),
                "--json".into(),
            ],
        ),
        (
            "edit set-cell",
            vec![
                "edit".into(),
                "set-cell".into(),
                path.into(),
                "--table".into(),
                "0".into(),
                "--row".into(),
                "0".into(),
                "--col".into(),
                "0".into(),
                "--text".into(),
                "x".into(),
                "-o".into(),
                out_dir.join("set-cell.hwpx").display().to_string(),
                "--json".into(),
            ],
        ),
    ]
}

#[test]
fn password_protected_documents_open_via_global_password_flag_for_every_wired_command() {
    let fixture = fixture_path();
    if !fixture.exists() {
        eprintln!("fixture 없음 — 건너뜀: {}", fixture.display());
        return;
    }
    let path = fixture.to_str().expect("UTF-8 fixture path");
    let out_dir = tempfile_dir();

    for (label, args) in regression_commands(path, &out_dir) {
        // 비밀번호 없이: 암호 문서이므로 실패해야 한다 (EXIT_USAGE=2).
        let mut no_pw_args: Vec<&str> = args.iter().map(String::as_str).collect();
        let no_pw = run(&no_pw_args);
        assert!(
            !no_pw.status.success(),
            "{label}: 비밀번호 없이 암호 문서가 열리면 안 됩니다 (배선 누락 의심)\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&no_pw.stdout),
            String::from_utf8_lossy(&no_pw.stderr)
        );

        // 올바른 비밀번호: --password 를 맨 앞에 붙여 전역 pre-scan 이 소비하게 한다.
        let mut with_pw: Vec<&str> = vec!["--password", PASSWORD];
        with_pw.append(&mut no_pw_args);
        let ok = run(&with_pw);
        assert!(
            ok.status.success(),
            "{label}: --password 로도 암호 문서를 못 엽니다 — CLI가 전역 비밀번호를 무시하고 있습니다\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&ok.stdout),
            String::from_utf8_lossy(&ok.stderr)
        );

        // 틀린 비밀번호: 실패해야 하고, 비밀번호 값이 출력에 에코되면 안 된다.
        let mut wrong_args: Vec<&str> = args.iter().map(String::as_str).collect();
        let mut with_wrong_pw: Vec<&str> = vec!["--password", WRONG_PASSWORD];
        with_wrong_pw.append(&mut wrong_args);
        let wrong = run(&with_wrong_pw);
        assert!(
            !wrong.status.success(),
            "{label}: 틀린 비밀번호로 열리면 안 됩니다"
        );
        let stdout = String::from_utf8_lossy(&wrong.stdout);
        let stderr = String::from_utf8_lossy(&wrong.stderr);
        assert!(
            !stdout.contains(WRONG_PASSWORD) && !stderr.contains(WRONG_PASSWORD),
            "{label}: 오류 출력에 비밀번호를 에코하면 안 됩니다\nstdout={stdout}\nstderr={stderr}"
        );
    }
}

/// [#password-mcp-parity] `supports_password_stdin()` 화이트리스트에 오른 MCP
/// 도구는 전부 `cli.passwordStdin` 계약을 붙여야 한다는 전수 가드.
/// `capabilities --mcp` 출력을 실측해 목록과 실제 배선이 어긋나면 즉시 실패한다.
#[test]
fn every_password_capable_mcp_tool_declares_the_password_stdin_contract() {
    let capabilities = run(&["capabilities", "--mcp"]);
    assert!(capabilities.status.success(), "capabilities --mcp 실패");
    let manifest: serde_json::Value =
        serde_json::from_slice(&capabilities.stdout).expect("capabilities JSON 파싱 실패");
    let tools = manifest["tools"].as_array().expect("tools 배열");

    // src/main.rs::supports_password_stdin() 와 동일한 목록을 이 테스트가 독립적으로
    // 유지한다 — 구현이 목록을 몰래 줄여도(또는 새 암호 지원 명령을 빠뜨려도) 잡는다.
    const EXPECTED_PASSWORD_CAPABLE: &[&str] = &[
        "hwp_info",
        "hwp_digest",
        "hwp_export_text",
        "hwp_export_structure",
        "hwp_ir_diff",
        "hwp_export_svg",
        "hwp_export_pdf",
        "hwp_export_markdown",
        "hwp_convert_hwpx",
        "hwp_convert_hwp5",
        "hwp_split_document",
        "hwp_export_tables",
        "hwp_search",
        "hwp_extract_data",
        "hwp_fields",
        "hwp_inspect_hidden_text",
        "hwp_inspect_injection",
        "hwp_inspect_unicode",
        "hwp_inspect_watermark",
        "hwp_fill_fields",
        "hwp_replace_text",
        "hwp_set_checkbox",
        "hwp_set_cell",
    ];

    for name in EXPECTED_PASSWORD_CAPABLE {
        let tool = tools
            .iter()
            .find(|tool| tool["name"] == *name)
            .unwrap_or_else(|| panic!("{name}: capabilities --mcp 목록에 없음"));
        assert_eq!(
            tool["cli"]["passwordStdin"]["argument"], "password",
            "{name}: cli.passwordStdin.argument 누락/불일치 — {tool}"
        );
        assert_eq!(
            tool["cli"]["passwordStdin"]["flag"], "--password-stdin",
            "{name}: cli.passwordStdin.flag 누락/불일치 — {tool}"
        );
        assert_eq!(
            tool["inputSchema"]["properties"]["password"]["writeOnly"], true,
            "{name}: inputSchema.properties.password.writeOnly 누락 — {tool}"
        );
    }
}

fn tempfile_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-password-stdin-parity-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));
    std::fs::create_dir_all(&dir).expect("임시 출력 디렉터리 생성 실패");
    dir
}
