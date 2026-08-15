//! [#4220 T3] MCP tool annotations 계약 — 도구 성격 힌트가 실물 선언과 어긋나지 않는다.
//!
//! MCP 2025-03-26 개정판이 신설한 ToolAnnotations(readOnlyHint/destructiveHint/
//! idempotentHint/openWorldHint)를 rhwp 는 손으로 나열하지 않고 기존 선언에서
//! 유도한다(단일 출처): readOnlyHint 는 outputFields 의 산출 경로 필드에서,
//! destructiveHint 는 cli 배선의 `--in-place` 축에서, 세션 도구의 읽기/편집 경계는
//! `agent_profiles::SESSION_READ_TOOLS` 에서 온다. 여기서 검증하는 것은 유도
//! 구현의 재실행이 아니라 **실물 출력끼리의 정합**이다 — 매니페스트의 주석이
//! 같은 매니페스트의 category·배선·봉투 선언과 모순되면 실패한다.
//!
//! 스펙 근거: https://modelcontextprotocol.io/specification/2025-06-18/server/tools
//! (Tool.annotations) + schema.ts ToolAnnotations — 기본값은 readOnlyHint=false,
//! destructiveHint=true, idempotentHint=false, openWorldHint=true 이므로, 기본값에
//! 기대지 않고 4필드를 전부 명시하는 것 자체가 계약이다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// 스펙이 정의한 4필드 — 이 목록 밖 필드는 내지 않고, 이 목록은 전부 낸다.
const ANNOTATION_FIELDS: [&str; 4] = [
    "readOnlyHint",
    "destructiveHint",
    "idempotentHint",
    "openWorldHint",
];

fn run_json(args: &[&str]) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패");
    assert_eq!(
        output.status.code(),
        Some(0),
        "rhwp {} 실패:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout 이 순수 JSON 이 아니다")
}

fn manifest_tools() -> Vec<serde_json::Value> {
    run_json(&["capabilities", "--mcp"])["tools"]
        .as_array()
        .expect("tools 배열")
        .clone()
}

/// annotations 가 4필드를 정확히, 전부 boolean 으로 선언했는지.
fn assert_four_bool_fields(name: &str, annotations: &serde_json::Value) {
    let obj = annotations
        .as_object()
        .unwrap_or_else(|| panic!("{name}: annotations 가 객체가 아니다: {annotations}"));
    for field in ANNOTATION_FIELDS {
        assert!(
            obj.get(field).is_some_and(serde_json::Value::is_boolean),
            "{name}: annotations.{field} 가 boolean 으로 선언돼야 한다: {annotations}"
        );
    }
    let extra: Vec<&String> = obj
        .keys()
        .filter(|k| !ANNOTATION_FIELDS.contains(&k.as_str()))
        .collect();
    assert!(
        extra.is_empty(),
        "{name}: 스펙 밖 annotations 필드 {extra:?} — title 등은 별도 판단 없이 싣지 않는다"
    );
}

/// ① 매니페스트의 전 도구가 annotations 4필드를 선언한다.
#[test]
fn every_manifest_tool_declares_all_four_annotation_fields() {
    let tools = manifest_tools();
    assert!(
        tools.len() >= 30,
        "도구가 너무 적다({}) — 가드가 공허해진다",
        tools.len()
    );
    for t in &tools {
        let name = t["name"].as_str().expect("name");
        assert_four_bool_fields(name, &t["annotations"]);
    }
}

/// ② readOnlyHint=true ↔ 그 도구의 봉투가 산출 경로 필드를 선언하지 않는다.
///    그리고 편집 category(edit) 명령으로 내려가는 도구는 절대 readOnly 가 아니다 —
///    category 는 capabilities 명령 표면의 독립 선언이라 유도 구현과 원천이 다르다.
#[test]
fn read_only_tools_write_nothing_and_are_never_edit_category() {
    let caps = run_json(&["capabilities"]);
    let categories: BTreeMap<String, String> = caps["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .map(|c| {
            (
                c["name"].as_str().unwrap().to_string(),
                c["category"].as_str().unwrap().to_string(),
            )
        })
        .collect();

    for t in manifest_tools() {
        let name = t["name"].as_str().expect("name");
        let read_only = t["annotations"]["readOnlyHint"].as_bool().expect("bool");
        let writes = t["outputFields"].as_array().is_some_and(|fields| {
            fields
                .iter()
                .any(|f| matches!(f.as_str(), Some("output" | "outputDir")))
        });
        assert_eq!(
            read_only, !writes,
            "{name}: readOnlyHint({read_only}) 가 봉투의 산출 경로 선언(writes={writes})과 모순된다"
        );

        let command = t["cli"]["command"].as_str().expect("cli.command");
        let category = categories
            .get(command)
            .unwrap_or_else(|| panic!("{name}: cli.command {command} 가 capabilities 에 없다"));
        if category == "edit" {
            assert!(
                !read_only,
                "{name}: category=edit 명령({command})으로 내려가는 도구가 readOnlyHint=true 다"
            );
        }
    }
}

/// ③ destructiveHint=true ↔ cli 배선에 `--in-place` 축이 있다. 오늘의 파괴 축은
///    hwp_redact 하나다 — 새 in-place 축을 들이면 이 황금 목록을 의식적으로 늘려라.
#[test]
fn destructive_axis_matches_in_place_wiring() {
    let wiring_has_in_place = |cli: &serde_json::Value| {
        let in_args = |args: &serde_json::Value| {
            args.as_array()
                .is_some_and(|a| a.iter().any(|t| t.as_str() == Some("--in-place")))
        };
        in_args(&cli["args"])
            || cli["optionalArgs"]
                .as_array()
                .is_some_and(|opts| opts.iter().any(|o| in_args(&o["args"])))
    };

    let mut destructive: Vec<String> = Vec::new();
    for t in manifest_tools() {
        let name = t["name"].as_str().expect("name");
        let hint = t["annotations"]["destructiveHint"].as_bool().expect("bool");
        let in_place = wiring_has_in_place(&t["cli"]);
        assert_eq!(
            hint, in_place,
            "{name}: destructiveHint({hint}) 가 --in-place 배선 실물({in_place})과 모순된다"
        );
        if hint {
            // 파괴적이면서 읽기 전용일 수는 없다.
            assert_eq!(
                t["annotations"]["readOnlyHint"], false,
                "{name}: destructiveHint=true 인데 readOnlyHint=true 다"
            );
            destructive.push(name.to_string());
        }
    }
    assert_eq!(
        destructive,
        vec!["hwp_redact".to_string()],
        "무상태 표면의 파괴 축 황금 목록이 바뀌었다 — 의도한 변경이면 근거와 함께 갱신하라"
    );
}

/// 무상태 도구는 전부 결정론 변환 — idempotent 이고, 로컬 파일만 다루므로 폐쇄 세계다.
#[test]
fn stateless_tools_are_idempotent_and_closed_world() {
    for t in manifest_tools() {
        let name = t["name"].as_str().expect("name");
        assert_eq!(
            t["annotations"]["idempotentHint"], true,
            "{name}: 무상태 도구는 매번 원본에서 다시 계산한다 — idempotentHint=true 여야 한다"
        );
        assert_eq!(
            t["annotations"]["openWorldHint"], false,
            "{name}: rhwp 는 로컬 파일만 다룬다 — openWorldHint=false 여야 한다"
        );
    }
}

// ── ④ 서버(tools/list) 정합 — 무상태 되비춤 + 세션 도구 일관성 ──────────────

/// 살아있는 mcp-serve 프로세스 (mcp_server_contract.rs 의 최소 재현).
struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Server {
    fn started() -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .arg("mcp-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("rhwp mcp-serve 실행 실패");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        let mut s = Server {
            child,
            stdin,
            stdout,
            next_id: 1,
        };
        let r = s.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "annotations-contract", "version": "0"}
            }),
        );
        assert!(
            r["result"]["serverInfo"].is_object(),
            "initialize 실패: {r}"
        );
        s
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;
        let msg =
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        writeln!(self.stdin, "{msg}").expect("요청 쓰기 실패");
        self.stdin.flush().expect("flush");
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.stdout.read_line(&mut line).expect("응답 읽기 실패");
            assert!(n > 0, "서버가 응답 없이 종료했습니다 (method={method})");
            if line.trim().is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line.trim()).expect("순수 JSON-RPC");
            if v.get("id").and_then(|i| i.as_i64()) == Some(id) {
                return v;
            }
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn served_tools_reflect_manifest_and_session_tools_are_consistent() {
    let manifest: BTreeMap<String, serde_json::Value> = manifest_tools()
        .into_iter()
        .map(|t| {
            (
                t["name"].as_str().unwrap().to_string(),
                t["annotations"].clone(),
            )
        })
        .collect();

    let mut s = Server::started();
    let r = s.request("tools/list", serde_json::json!({}));
    let served = r["result"]["tools"].as_array().expect("tools").clone();

    let mut session_seen = 0usize;
    for t in &served {
        let name = t["name"].as_str().expect("name");
        // 전 도구(무상태 + 세션)가 4필드를 선언한다.
        assert_four_bool_fields(name, &t["annotations"]);

        if let Some(declared) = manifest.get(name) {
            // 무상태 도구: 서버는 매니페스트가 유도한 값을 그대로 되비춘다(단일 출처).
            assert_eq!(
                &t["annotations"], declared,
                "{name}: tools/list 의 annotations 가 capabilities --mcp 선언과 다르다"
            );
            continue;
        }
        session_seen += 1;
        let a = &t["annotations"];
        // 세션에도 개방 세계 축은 없다.
        assert_eq!(a["openWorldHint"], false, "{name}: {a}");
        match name {
            // 핸들 조회 6종 — 환경 무변경 (tree 는 #4357 의 안정 ID 구조 조회).
            "hwp_doc_text" | "hwp_doc_info" | "hwp_doc_fields" | "hwp_doc_tables"
            | "hwp_doc_search" | "hwp_doc_tree" => {
                assert_eq!(a["readOnlyHint"], true, "{name}: {a}");
                assert_eq!(a["destructiveHint"], false, "{name}: {a}");
            }
            // [#4357 W1] 워크스페이스 조회 2종 — 인벤토리·변이 저널. 환경 무변경이고
            // 같은 인자 재호출이 같은 관찰로 수렴한다(저널은 조회 자체가 기록을 안 남긴다).
            "hwp_ws_list" | "hwp_ws_journal" => {
                assert_eq!(a["readOnlyHint"], true, "{name}: {a}");
                assert_eq!(a["destructiveHint"], false, "{name}: {a}");
                assert_eq!(a["idempotentHint"], true, "{name}: {a}");
            }
            // open 계열은 호출마다 새 docId — 읽기 전용이되 멱등은 아니다.
            // ws_open 은 session_open 위임이라 hwp_open 과 같은 판정이다(#4357).
            "hwp_open" | "hwp_ws_open" => {
                assert_eq!(a["readOnlyHint"], true, "{name}: {a}");
                assert_eq!(a["idempotentHint"], false, "{name}: {a}");
            }
            "hwp_close" => {
                assert_eq!(a["readOnlyHint"], true, "{name}: {a}");
                assert_eq!(a["destructiveHint"], false, "{name}: {a}");
            }
            // 파일을 쓰는 세션 축 — render_page 는 추가형, save 는 원본 경로를 받을 수
            // 있는 세션의 in-place 축이라 유일하게 파괴적이다.
            "hwp_doc_render_page" => {
                assert_eq!(a["readOnlyHint"], false, "{name}: {a}");
                assert_eq!(a["destructiveHint"], false, "{name}: {a}");
            }
            "hwp_doc_save" => {
                assert_eq!(a["readOnlyHint"], false, "{name}: {a}");
                assert_eq!(a["destructiveHint"], true, "{name}: {a}");
            }
            // IR 누적 편집 — 디스크는 안 건드리지만 환경(세션 문서 상태)을 바꾼다.
            "hwp_doc_replace_text" => {
                assert_eq!(a["readOnlyHint"], false, "{name}: {a}");
                assert_eq!(a["destructiveHint"], false, "{name}: {a}");
                // 이미 치환된 IR 위에 겹쳐 적용될 수 있다 — 무상태 쪽과 다른 지점.
                assert_eq!(a["idempotentHint"], false, "{name}: {a}");
            }
            "hwp_doc_set_cell" | "hwp_doc_fill_fields" => {
                assert_eq!(a["readOnlyHint"], false, "{name}: {a}");
                assert_eq!(a["destructiveHint"], false, "{name}: {a}");
                assert_eq!(a["idempotentHint"], true, "{name}: {a}");
            }
            // [#4856] 세션 조회 파리티 — 개요/조문(structure)·데이터 추출(extract_data).
            // 환경 무변경이고 같은 인자 재호출이 같은 관찰로 수렴한다(순수 조회).
            "hwp_doc_structure" | "hwp_doc_extract_data" => {
                assert_eq!(a["readOnlyHint"], true, "{name}: {a}");
                assert_eq!(a["destructiveHint"], false, "{name}: {a}");
                assert_eq!(a["idempotentHint"], true, "{name}: {a}");
            }
            other => panic!("계약에 없는 세션 도구 {other} — 이 match 에 판정을 추가하라: {a}"),
        }
    }
    assert_eq!(
        session_seen, 18,
        "세션 도구 수가 달라졌다 — agent_profiles::ALL_SESSION_TOOLS 와 이 계약을 함께 갱신하라"
    );
}
