//! [#5791] 명령별 `--help` 계약 — 도움말을 물어본 호출은 답을 받는다.
//!
//! 종전 실측(devel `fb434269e`): `capabilities` 98종 중 **79종이 exit 2**
//! ("알 수 없는 옵션: --help"), **5종은 `--help` 를 파일 경로로 읽어 exit 1**,
//! `edit`·`inspect` 하위 92종은 전멸이었다. 대안은 `rhwp --help` 통짜
//! 71,978 B / 1,163 줄 하나뿐이라, 한 명령을 알아내려고 수백 배를 읽어야 했다.
//!
//! 계약의 오라클은 골든 파일이 아니라 **바이너리 자기서술**이다 —
//! `capabilities` 가 싣는 명령·하위 명령을 그대로 순회하므로, 명령이 늘면
//! 이 가드가 덮는 범위도 함께 는다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

/// nextest archive 가 런타임에 주입하는 경로를 먼저 읽고, 없으면 컴파일타임 값을 쓴다
/// (local_validation.md 4.3 의 신규 CLI 통합 테스트 규칙 — #3289).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn capabilities() -> serde_json::Value {
    serde_json::from_slice(&run(&["capabilities"]).stdout).expect("capabilities 봉투")
}

/// (명령 이름, 선언된 하위 명령들).
fn declared() -> Vec<(String, Vec<String>)> {
    capabilities()["commands"]
        .as_array()
        .expect("commands 배열")
        .iter()
        .filter_map(|c| {
            let name = c["name"].as_str()?.to_string();
            let subs = c["subcommands"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s["name"].as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            Some((name, subs))
        })
        .collect()
}

#[test]
fn every_declared_command_answers_help_on_stdout() {
    let commands = declared();
    assert!(
        commands.len() >= 90,
        "명령 수가 갑자기 줄었다: {}",
        commands.len()
    );
    for (name, _) in &commands {
        let out = run(&[name, "--help"]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "rhwp {name} --help 가 exit 0 이 아니다: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = stdout_of(&out);
        assert!(
            !text.trim().is_empty(),
            "rhwp {name} --help 가 stdout 을 비웠다"
        );
        assert!(
            text.contains(name.as_str()),
            "rhwp {name} --help 출력에 자기 이름이 없다:\n{text}"
        );
        assert!(
            out.stderr.is_empty(),
            "도움말은 stdout 이다 — {name} 이 stderr 로 샜다: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn every_declared_subcommand_answers_help() {
    let mut checked = 0usize;
    for (name, subs) in declared() {
        for sub in subs {
            let out = run(&[&name, &sub, "--help"]);
            assert_eq!(
                out.status.code(),
                Some(0),
                "rhwp {name} {sub} --help 가 exit 0 이 아니다: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            let text = stdout_of(&out);
            assert!(
                text.contains(&sub),
                "rhwp {name} {sub} --help 출력에 자기 이름이 없다:\n{text}"
            );
            checked += 1;
        }
    }
    assert!(
        checked >= 90,
        "하위 명령 순회가 {checked}건뿐이다 — 선언을 못 읽었다"
    );
}

#[test]
fn short_flag_answers_the_same_way() {
    let long = stdout_of(&run(&["fields", "--help"]));
    let short = run(&["fields", "-h"]);
    assert_eq!(short.status.code(), Some(0));
    assert_eq!(stdout_of(&short), long, "-h 와 --help 의 답이 다르다");
}

#[test]
fn scoped_help_is_a_fraction_of_the_whole_help() {
    let whole = stdout_of(&run(&["--help"]));
    let scoped = stdout_of(&run(&["fields", "--help"]));
    assert!(
        scoped.contains("fields"),
        "명령별 도움말이 비었다 — 통짜와의 비교가 무의미하다"
    );
    assert!(
        whole.len() > scoped.len() * 50,
        "명령별 도움말이 통짜 대비 충분히 작지 않다 (통짜 {} B, fields {} B)",
        whole.len(),
        scoped.len()
    );
}

#[test]
fn group_help_is_an_index_of_subcommands() {
    let text = stdout_of(&run(&["edit", "--help"]));
    assert!(
        text.contains("하나만 보기"),
        "그룹 도움말에 다음 수가 없다:\n{text}"
    );
    assert!(text.contains("fill-fields"), "그룹 목차에 하위 이름이 없다");
    let whole = stdout_of(&run(&["--help"]));
    assert!(
        text.len() * 3 < whole.len(),
        "그룹 목차가 절 전체를 쏟고 있다 (목차 {} B, 통짜 {} B)",
        text.len(),
        whole.len()
    );
}

#[test]
fn the_whole_help_still_prints_everything() {
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let text = stdout_of(&out);
    assert!(
        text.lines().count() > 1_000,
        "통짜 도움말이 줄어들었다: {}",
        text.lines().count()
    );
    assert!(text.contains("명령:"), "통짜 도움말의 구역 제목이 사라졌다");
    assert!(
        text.contains("  edit fill-fields "),
        "통짜 도움말에서 하위 절이 사라졌다"
    );
}

/// `--help` 가 **값 자리**면 도움말이 아니다 — `--find --help` 는 "--help" 를 찾는 치환이다.
#[test]
fn help_in_a_value_slot_is_not_a_help_request() {
    let out = run(&[
        "edit",
        "replace-text",
        "samples/field-01.hwp",
        "--find",
        "--help",
    ]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "값 자리의 --help 를 도움말로 가로챘다: {}",
        stdout_of(&out)
    );
    assert!(
        stdout_of(&out).trim().is_empty(),
        "값 자리 호출이 stdout 에 도움말을 냈다"
    );
}

#[test]
fn unknown_command_keeps_its_error_path() {
    let out = run(&["definitely-not-a-command", "--help"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("알 수 없는 명령"),
        "모르는 명령의 오류 경로가 바뀌었다"
    );
}

/// 종전에 `--help` 를 **파일 경로로 읽어** exit 1(런타임 실패)이 나던 5종.
#[test]
fn dump_commands_no_longer_read_help_as_a_file() {
    for cmd in [
        "dump-pages",
        "dump-extents",
        "dump-note-shape",
        "dump-records",
        "core-pages",
    ] {
        let out = run(&[cmd, "--help"]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "rhwp {cmd} --help: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("파일을 읽을 수 없습니다"),
            "{cmd} 가 아직 --help 를 파일로 읽는다"
        );
    }
}
