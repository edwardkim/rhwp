//! [#3880] 봉투 정합 계약 — 자기서술과 실물이 어긋나지 않는다.
//!
//! 이 표면의 봉투는 에이전트가 **믿고 판단하는 근거**다. 봉투가 실물과 다르면
//! 에이전트는 틀린 판단을 하고도 그것을 모른다. 여기 모은 것은 전부
//! "죽지는 않는데 조용히 거짓말하는" 부류다.
//!
//! # T1 — 건너뛴 것을 봉투가 밝힌다
//!
//! `info` 의 인간 출력은 `warnings: N` 과 상세를 내는데, JSON 분기는 그 앞에서
//! `return EXIT_OK` 로 끝나 도달하지 못했다. 그래서 리소스가 조용히 잘린 문서가
//! **exit 0 + 완전해 보이는 봉투**를 냈다 — `fonts` 가 부분 목록인데 봉투는
//! 그렇다고 말하지 않았다. #3719 불변식 "부분 목록 금지" 위반이다.
//!
//! # T3 — 봉투 키는 한 가지 표기법이다
//!
//! `export-structure` 최상위는 `nodeCount` 인데 중첩된 `structure` 객체만
//! `node_count` 로 나갔다. 별칭 조회 계층이 없는 정적 매핑 언어(C#·Swift)에서는
//! 이 필드가 **사라진다** — M20 이 시작되면 바로 부딪힌다.

use std::process::{Command, Output};

fn rhwp_bin() -> String {
    env!("CARGO_BIN_EXE_rhwp").to_string()
}

fn repo(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], out: &Output) -> String {
    format!(
        "명령: rhwp {}\n종료코드: {:?}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn json_of(args: &[&str]) -> serde_json::Value {
    let out = run(args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(args, &out));
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, &out)
        )
    })
}

/// 표본 하나를 고른다. 확장자별 첫 파일이면 충분하다.
fn first_sample(ext: &str) -> Option<std::path::PathBuf> {
    let dirs = [repo("samples"), repo("samples/hml")];
    for dir in dirs {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut hits: Vec<std::path::PathBuf> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
            })
            .collect();
        hits.sort();
        if let Some(p) = hits.into_iter().next() {
            return Some(p);
        }
    }
    None
}

// ── T1 ────────────────────────────────────────────────────────────────────

#[test]
fn info_envelope_always_declares_warnings() {
    let Some(doc) = first_sample("hwp") else {
        panic!("표본이 없습니다 — 이 시험이 공허하게 통과합니다");
    };
    let p = doc.to_str().unwrap();
    let v = json_of(&["info", "--json", p]);

    // 키가 **항상** 있어야 한다. 없으면 소비자가 "경고 없음"과
    // "이 빌드는 경고를 모름"을 구별할 수 없다.
    assert!(
        v.get("warnings").is_some(),
        "info 봉투에 warnings 키가 없습니다: {v}"
    );
    assert!(
        v["warnings"].is_array(),
        "warnings 는 배열이어야 합니다: {}",
        v["warnings"]
    );
}

#[test]
fn info_envelope_carries_actual_parser_warnings() {
    // HML 파서가 건너뛴 요소를 실제로 싣는지 본다. 빈 배열만 확인하면
    // "항상 빈 배열을 내는" 구현도 위 시험을 통과한다.
    let Some(doc) = first_sample("hml") else {
        eprintln!("HML 표본이 없어 건너뜁니다 — 경고 원천이 현재 HML 하나뿐입니다");
        return;
    };
    let p = doc.to_str().unwrap();
    let v = json_of(&["info", "--json", p]);
    let warnings = v["warnings"].as_array().expect("warnings 배열");

    assert!(
        !warnings.is_empty(),
        "이 HML 표본은 인간 출력에서 경고를 내는데 봉투가 비었습니다.\n\
         봉투가 '건너뛴 것 없음'이라고 거짓말하면 에이전트는 부분 결과를 전부로 믿습니다.\n{v}"
    );
    for w in warnings {
        for key in ["code", "xmlPath", "message"] {
            assert!(
                w.get(key).is_some(),
                "warnings[] 항목에 {key} 가 없습니다: {w}"
            );
        }
    }
}

#[test]
fn info_warnings_is_declared_in_capabilities() {
    // 봉투에 넣고 자기서술에 빠뜨리면 매니페스트만 읽는 소비자가 이 필드를 모른다.
    let caps = json_of(&["capabilities"]);
    let cmds = caps["commands"].as_array().expect("commands 배열");
    let info = cmds
        .iter()
        .find(|c| c["name"] == "info")
        .expect("info 명령 항목");
    let fields: Vec<&str> = info["recordFields"]
        .as_array()
        .expect("recordFields")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(
        fields.contains(&"warnings"),
        "capabilities 의 info.recordFields 에 warnings 가 없습니다: {fields:?}"
    );
}

// ── T3 ────────────────────────────────────────────────────────────────────

/// 봉투 전체를 재귀로 훑어 `_` 가 든 키를 모은다.
fn snake_case_keys(v: &serde_json::Value, path: &str, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Object(m) => {
            for (k, child) in m {
                if k.contains('_') {
                    out.push(format!("{path}{k}"));
                }
                snake_case_keys(child, &format!("{path}{k}."), out);
            }
        }
        serde_json::Value::Array(a) => {
            for child in a {
                snake_case_keys(child, path, out);
            }
        }
        _ => {}
    }
}

#[test]
fn export_structure_envelope_has_no_snake_case_keys() {
    let Some(doc) = first_sample("hwp") else {
        panic!("표본이 없습니다");
    };
    let p = doc.to_str().unwrap();
    let v = json_of(&["export-structure", "--json", p]);

    let mut bad = Vec::new();
    snake_case_keys(&v, "", &mut bad);
    assert!(
        bad.is_empty(),
        "봉투에 snake_case 키가 섞였습니다: {bad:?}\n\
         별칭 조회 계층이 없는 정적 매핑 언어(C#·Swift)에서는 이 필드가 사라집니다.\n{v}"
    );

    // 값이 실제로 옮겨졌는지도 본다 — 키만 지우는 수정을 막는다.
    assert!(
        v["structure"].get("nodeCount").is_some(),
        "structure.nodeCount 가 없습니다 — 이름만 바꾸고 값을 잃었습니다: {}",
        v["structure"]
    );
}

/// 다른 조회 봉투에도 같은 규약을 적용한다. 하나를 고치고 다음을 놓치지 않기 위해서다.
#[test]
fn query_envelopes_share_the_camel_case_rule() {
    let Some(doc) = first_sample("hwp") else {
        panic!("표본이 없습니다");
    };
    let p = doc.to_str().unwrap();

    for args in [
        vec!["info", "--json", p],
        vec!["digest", "--json", p],
        vec!["fields", "--json", p],
        vec!["export-tables", "--json", p],
    ] {
        let v = json_of(&args);
        let mut bad = Vec::new();
        snake_case_keys(&v, "", &mut bad);
        assert!(
            bad.is_empty(),
            "`rhwp {}` 봉투에 snake_case 키가 섞였습니다: {bad:?}",
            args.join(" ")
        );
    }
}
