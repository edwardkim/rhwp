//! 조판 세대 플래그 `--compat` 가 렌더 명령에서도 닿는지 확인한다.
//!
//! #5524 는 축(`hangul2024_layout`)과 `dump-pages --compat` 만 넣었다. 시각·PDF 대조는
//! `export-svg`/`export-pdf` 로 하는데 거기서는 세대를 고를 수 없어, 한글 오라클을 2024 로
//! 띄워 놓고도 rhwp 는 2022 로만 그릴 수 있었다. 이 시험은 렌더 경로가 플래그를 실제로
//! 받고 조판이 바뀌는 것까지 본다 — 선언만 있고 안 먹는 거짓 계약을 막는다.
//!
//! 축이 이분인 근거(2018·2020·2022 는 같은 엔진)는
//! `mydocs/report/hangul_version_oracle_r1_20260807.md` 8절이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5524_hangul2024_compat_letterhead.hwp")
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행")
}

/// `dump-pages --json` 산출에서 각 쪽의 첫 문단 인덱스를 뽑아 조판 지문을 만든다.
fn page_fingerprint(args: &[&str]) -> String {
    let out = run(args);
    assert!(
        out.status.success(),
        "종료코드 {:?}: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let value: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("dump-pages --json 파싱");
    let pages = value["pages"].as_array().expect("pages");
    let mut marks = Vec::new();
    for (index, page) in pages.iter().enumerate() {
        let first = page["columns"]
            .as_array()
            .expect("columns")
            .iter()
            .flat_map(|column| column["items"].as_array().expect("items"))
            .find_map(|item| item["paraIndex"].as_u64());
        marks.push(format!("{index}@{}", first.unwrap_or(u64::MAX)));
    }
    marks.join(",")
}

#[test]
fn compat_2024_changes_pagination_and_2022_is_the_default() {
    let sample = sample();
    let default = page_fingerprint(&["dump-pages", &sample, "--json"]);
    let explicit_2022 = page_fingerprint(&["dump-pages", &sample, "--json", "--compat", "2022"]);
    let compat_2024 = page_fingerprint(&["dump-pages", &sample, "--json", "--compat", "2024"]);

    assert_eq!(
        default, explicit_2022,
        "기본값은 2022 계열이다 — 플래그를 명시해도 같아야 한다"
    );
    assert_ne!(
        default, compat_2024,
        "이 샘플은 두 세대가 갈리는 실문서다 — --compat 2024 가 조판을 바꿔야 한다"
    );
}

#[test]
fn render_commands_accept_the_compat_flag() {
    let sample = sample();
    // 렌더 명령마다 플래그를 실제로 받는지 본다. EXIT_USAGE(2)면 선언만 있고 안 받는 것이다.
    for command in ["export-svg", "export-pdf", "export-render-tree"] {
        let output_dir = std::env::temp_dir().join(format!(
            "rhwp-compat-{command}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let output = output_dir.to_string_lossy().into_owned();
        let out = run(&[command, &sample, "-o", &output, "--compat", "2024"]);
        assert_ne!(
            out.status.code(),
            Some(2),
            "{command} 가 --compat 를 받지 않는다: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let _ = std::fs::remove_dir_all(&output_dir);
        let _ = std::fs::remove_file(&output_dir);
    }
}

#[test]
fn compat_rejects_generations_that_are_not_a_separate_engine() {
    let sample = sample();
    // 2018·2020 은 2022 와 같은 엔진이라 별도 세대로 받지 않는다 (3자 대조 실측).
    for value in ["2018", "2020", "2026", ""] {
        let out = run(&["dump-pages", &sample, "--json", "--compat", value]);
        assert_eq!(
            out.status.code(),
            Some(2),
            "--compat {value} 를 받아들이면 안 된다"
        );
    }
    // 값 자체가 없으면 사용법 오류다.
    let out = run(&["dump-pages", &sample, "--json", "--compat"]);
    assert_eq!(out.status.code(), Some(2));
}
