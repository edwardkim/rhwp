//! 조회 CLI 통합의 공개 진입점·인자 전달 계약.

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    let binary = std::env::var("CARGO_BIN_EXE_rhwp-q")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q").to_string());
    Command::new(binary)
        .args(args)
        .output()
        .expect("rhwp-q 실행")
}

#[test]
fn root_help_keeps_all_twenty_three_query_entry_points() {
    let output = run(&["--help"]);
    assert!(output.status.success(), "{output:?}");
    let help = String::from_utf8(output.stdout).expect("UTF-8 help");
    // 통합 전 공개 실행 타깃 목록을 독립적인 호환성 기대값으로 보존한다.
    let expected = [
        "char-shape",
        "control-layout",
        "cursor-model",
        "cursor-rect",
        "font-layout-evidence",
        "font-trace",
        "form-info",
        "hit-test",
        "kit",
        "markdown",
        "more",
        "object-cycle",
        "objects",
        "pack",
        "page-caret",
        "page-def",
        "page-images",
        "page-items",
        "para-shape",
        "scan-items",
        "section-starts",
        "text-file",
        "text-layout",
    ];
    let actual: Vec<_> = help
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .collect();
    assert_eq!(actual.len(), expected.len(), "{help}");
    for name in expected {
        assert!(actual.contains(&name), "누락된 조회 진입점: {name}");
        let child_help = run(&[name, "--help"]);
        assert!(child_help.status.success(), "{name}: {child_help:?}");
    }
}

#[test]
fn dispatcher_preserves_usage_errors_and_child_json_arguments() {
    for args in [vec![], vec!["unknown-query"], vec!["page-images", "--nope"]] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        assert!(
            output.stdout.is_empty(),
            "오류의 stdout에 JSON 조각을 남기지 않는다"
        );
    }
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/form-01.hwp");
    let source = source.to_str().expect("UTF-8 source");
    let output = run(&["page-images", source, "--page=0", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(value["tool"], "rhwp-q-page-images");
    assert_eq!(value["page"], 0);
    assert_eq!(value["source"], source);
    assert_eq!(value["keys"], serde_json::json!([]));
}

#[test]
fn all_query_commands_explain_arguments_options_examples_and_outputs() {
    let root = run(&["--help"]);
    let root = String::from_utf8(root.stdout).unwrap();
    let mut leaves = Vec::new();
    for command in root.lines().filter_map(|line| line.strip_prefix("  ")) {
        if matches!(command, "kit" | "more" | "pack") {
            let output = run(&[command, "--help"]);
            assert!(output.status.success(), "{output:?}");
            let text = String::from_utf8(output.stdout).unwrap();
            let prefix = format!("  rhwp-q {command} ");
            let children: Vec<_> = text
                .lines()
                .filter_map(|line| line.strip_prefix(&prefix)?.split_whitespace().next())
                .collect();
            assert_eq!(children.len(), if command == "kit" { 50 } else { 51 });
            for child in children {
                leaves.push(vec![command.to_string(), child.to_string()]);
            }
        } else {
            leaves.push(vec![command.to_string()]);
        }
    }
    // 통합 전 공개 단건 20개와 kit 50개, more/pack 각각 51개의 사용법을 보호한다.
    assert_eq!(leaves.len(), 172);
    for mut args in leaves {
        let command = args.join(" ");
        // 실제 파일 없이 도움말을 볼 수 있어야 한다.
        args.extend(["not-an-existing-document.hwp".into(), "--help".into()]);
        let output = run(&args.iter().map(String::as_str).collect::<Vec<_>>());
        assert!(output.status.success(), "{command}: {output:?}");
        assert!(output.stderr.is_empty(), "{command}: {output:?}");
        let text = String::from_utf8(output.stdout).unwrap();
        for heading in [
            "명령:",
            "그룹:",
            "사용법:",
            "설명:",
            "인자:",
            "옵션:",
            "예시:",
            "세부 설명:",
            "출력:",
            "종료 코드:",
        ] {
            assert!(text.contains(heading), "{command}에 {heading} 누락: {text}");
        }
        assert!(text.contains(&format!("명령: {command}\n")), "{text}");
        assert!(text.contains("<파일>"), "{text}");
        assert!(text.contains("--json"), "{text}");
    }
}

#[test]
fn help_describes_actual_kit_options_and_keeps_help_as_a_query_value() {
    for (command, flags) in [
        (
            "cursor-in-hf",
            vec![
                "--section",
                "--header|--footer",
                "--apply-to",
                "--para",
                "--offset",
            ],
        ),
        (
            "cursor-in-footnote",
            vec!["--page", "--index", "--para", "--offset"],
        ),
        (
            "cursor-in-note",
            vec!["--section", "--para", "--ci", "--note-para", "--offset"],
        ),
        ("note-edit-info", vec!["--section", "--para", "--ci"]),
        ("source-image-bytes", vec!["--key", "--include-bytes"]),
        ("bin-data", vec!["--index", "--include-bytes"]),
        (
            "search-all",
            vec!["--q", "--ignore-case", "--include-cells"],
        ),
        ("canvaskit-preflight", vec!["--mode", "--profile"]),
        (
            "fn-selection-rects",
            vec![
                "--index",
                "--start-para",
                "--start-pos",
                "--end-para",
                "--end-pos",
            ],
        ),
    ] {
        let output = run(&["kit", command, "--help"]);
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8(output.stdout).unwrap();
        for flag in flags {
            assert!(text.contains(flag), "{command}에 {flag} 없음: {text}");
        }
    }
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/form-01.hwp");
    let output = run(&[
        "kit",
        "search-all",
        source.to_str().unwrap(),
        "--q",
        "--help",
        "--json",
    ]);
    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["query"], "--help");
    assert_eq!(value["command"], "search-all");
}
