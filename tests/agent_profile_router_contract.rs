//! [#3629] 에이전트 역할 라우터 — 직무 프로필별 도구 세트·레시피 계약.
//! 단일 출처(agent_profiles::PROFILES)가 capabilities --mcp --profile 과
//! mcp-serve --profile 양쪽을 구동한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Output, Stdio};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

#[test]
fn profile_filters_manifest_and_carries_recipe() {
    let out = run(&["capabilities", "--mcp", "--profile", "행정서식"]);
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("manifest");
    let names: Vec<&str> = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    // 직무 도구는 있고, 무관 도구는 없어야 한다.
    for must in ["hwp_fields", "hwp_fill_fields", "hwp_set_checkbox"] {
        assert!(names.contains(&must), "{names:?}");
    }
    for not in ["hwp_build_from_ingest", "hwp_export_markdown"] {
        assert!(!names.contains(&not), "{not} 는 행정서식 밖: {names:?}");
    }
    // 레시피·자기서술.
    assert_eq!(v["profile"]["name"], "행정서식");
    assert!(v["profile"]["recipe"].as_array().unwrap().len() >= 3, "{v}");
    assert!(v["profiles"].as_array().unwrap().len() >= 5, "{v}");
}

#[test]
fn full_profile_and_no_profile_are_unfiltered() {
    let all = run(&["capabilities", "--mcp"]);
    let av: serde_json::Value = serde_json::from_slice(&all.stdout).unwrap();
    let full = run(&["capabilities", "--mcp", "--profile", "개발통합"]);
    let fv: serde_json::Value = serde_json::from_slice(&full.stdout).unwrap();
    assert_eq!(
        av["tools"].as_array().unwrap().len(),
        fv["tools"].as_array().unwrap().len(),
        "개발통합은 필터 없음"
    );
    assert!(av["profile"].is_null(), "무프로필은 profile null: {av}");
}

#[test]
fn unknown_profile_is_usage_error_with_listing() {
    let out = run(&["capabilities", "--mcp", "--profile", "없는역할"]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("행정서식"), "목록 안내가 있어야 합니다: {err}");
    // mcp-serve 쪽도 같은 계약.
    let out2 = run(&["mcp-serve", "--profile", "없는역할"]);
    assert_eq!(out2.status.code(), Some(2));
}

#[test]
fn serve_profile_filters_tools_list_and_session() {
    // 행정서식(session=true)은 세션 도구 포함, 데이터분석(session=false)은 제외.
    for (profile, expect_session) in [("행정서식", true), ("데이터분석", false)] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .args(["mcp-serve", "--profile", profile])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("mcp-serve");
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-06-18","capabilities":{{}},"clientInfo":{{"name":"t","version":"0"}}}}}}"#).unwrap();
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{{}}}}"#
        )
        .unwrap();
        stdin.flush().unwrap();
        let mut names: Vec<String> = Vec::new();
        let mut line = String::new();
        loop {
            line.clear();
            assert!(stdout.read_line(&mut line).unwrap() > 0, "조기 종료");
            let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
            if v.get("id").and_then(|i| i.as_i64()) == Some(2) {
                names = v["result"]["tools"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|t| t["name"].as_str().map(String::from))
                    .collect();
                break;
            }
        }
        let _ = child.kill();
        let _ = child.wait();
        assert_eq!(
            names.contains(&"hwp_open".to_string()),
            expect_session,
            "{profile}: {names:?}"
        );
        if profile == "데이터분석" {
            assert!(
                names.contains(&"hwp_export_tables".to_string()),
                "{names:?}"
            );
            assert!(!names.contains(&"hwp_fill_fields".to_string()), "{names:?}");
        }
    }
}

#[test]
fn serve_profile_rejects_hidden_session_tool_calls() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["mcp-serve", "--profile", "데이터분석"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mcp-serve");
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"hwp_open","arguments":{{"path":"not-used.hwp"}}}}}}"#
    )
    .unwrap();
    stdin.flush().unwrap();

    let mut line = String::new();
    assert!(stdout.read_line(&mut line).unwrap() > 0, "조기 종료");
    let response: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    let result = &response["result"];
    assert_eq!(result["isError"], true, "{response}");
    assert!(
        result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("세션 도구"),
        "{response}"
    );
    let _ = child.kill();
    let _ = child.wait();
}

/// [#3629 후속] 조회 전용 직무는 **세션 쓰기 도구를 갖지 못한다.**
///
/// 종전에는 프로필의 세션 축이 `bool` 하나였다. 그래서 대량 RAG·감사용
/// `아카이브검색`(허용 도구 7종에 쓰기 도구가 하나도 없다)이 세션을 쓰려면
/// 편집·저장 4종까지 통째로 열릴 수밖에 없었고, 읽기 전용을 표방한 프로필로
/// `hwp_open` → `hwp_doc_save` 를 이어 **원본 문서를 덮어쓸 수 있었다**(실측).
///
/// 목록과 호출 두 층을 함께 본다 — 목록에서 뺐는데 호출로 들어가면 프로필은
/// 경계가 아니라 추천 목록으로 격하된다(`mcp_serve.rs` 의 우회 차단 주석).
#[test]
fn readonly_profile_serves_no_session_write_tools() {
    const WRITE_TOOLS: [&str; 4] = [
        "hwp_doc_save",
        "hwp_doc_fill_fields",
        "hwp_doc_set_cell",
        "hwp_doc_replace_text",
    ];

    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["mcp-serve", "--profile", "아카이브검색"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mcp-serve");
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"tools/list"}}"#).unwrap();
    stdin.flush().unwrap();
    let mut line = String::new();
    assert!(stdout.read_line(&mut line).unwrap() > 0, "조기 종료");
    let listed: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    let names: Vec<String> = listed["result"]["tools"]
        .as_array()
        .expect("tools 배열")
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();

    // 조회 세션은 열려 있어야 한다 — 이 프로필의 레시피가 hwp_open 을 쓴다.
    assert!(names.contains(&"hwp_open".to_string()), "{names:?}");
    assert!(names.contains(&"hwp_doc_search".to_string()), "{names:?}");
    for w in WRITE_TOOLS {
        assert!(
            !names.contains(&w.to_string()),
            "조회 전용 프로필이 쓰기 도구 {w} 를 노출했습니다: {names:?}"
        );
    }

    // 목록에서 뺐으면 호출로도 들어갈 수 없어야 한다.
    for (i, w) in WRITE_TOOLS.iter().enumerate() {
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":{},"method":"tools/call","params":{{"name":"{w}","arguments":{{"docId":"doc-1","output":"x.hwp"}}}}}}"#,
            10 + i
        )
        .unwrap();
        stdin.flush().unwrap();
        let mut l = String::new();
        assert!(stdout.read_line(&mut l).unwrap() > 0, "조기 종료 ({w})");
        let r: serde_json::Value = serde_json::from_str(l.trim()).unwrap();
        assert_eq!(
            r["result"]["isError"], true,
            "{w} 호출이 우회로 들어갔습니다: {r}"
        );
    }

    let _ = child.kill();
    let _ = child.wait();
}

/// 자기서술도 실제 제공 집합과 같아야 한다 — 선언 7종, 실제 19종이면 매니페스트로
/// 도구 정의를 자동 생성하는 소비자가 실물과 다른 표면을 얻는다.
#[test]
fn capabilities_declares_the_session_tools_it_actually_serves() {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp", "--profile", "아카이브검색"])
        .output()
        .expect("capabilities");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("mcp JSON");
    let declared: Vec<String> = v["profile"]["sessionTools"]
        .as_array()
        .expect("profile.sessionTools 가 있어야 합니다")
        .iter()
        .filter_map(|t| t.as_str().map(String::from))
        .collect();
    assert!(declared.contains(&"hwp_open".to_string()), "{declared:?}");
    assert!(
        !declared.contains(&"hwp_doc_save".to_string()),
        "조회 전용 프로필 선언에 쓰기 도구가 있습니다: {declared:?}"
    );
    assert_eq!(v["profile"]["session"], true, "{v}");

    // 세션을 안 여는 프로필은 sessionTools 가 null 이다.
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp", "--profile", "데이터분석"])
        .output()
        .expect("capabilities");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("mcp JSON");
    assert_eq!(v["profile"]["session"], false, "{v}");
    assert!(v["profile"]["sessionTools"].is_null(), "{v}");
}
