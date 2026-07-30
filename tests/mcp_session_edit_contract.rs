//! [#3598] `mcp-serve` 세션 편집 2단계 — 열린 핸들 위에서 채우고(hwp_doc_fill_fields)
//! 형식 보존으로 저장한다(hwp_doc_save).
//!
//! 세션의 존재 이유(#3140: 재파싱 회피)가 조회에만 적용되고 편집에는 빠져 있었다.
//! 계약의 핵심: ① 편집은 핸들의 IR 에 **누적**되고 디스크는 save 까지 불변
//! ② 판정 필드(filledCount/notFound/ambiguous)는 무상태 hwp_fill_fields 와 동형
//! ③ 저장은 입력 형식을 보존한다(HWP5→HWP5, HWPX→HWPX, #3383 규약).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// 누름틀 11개(회사명/작성자/… )를 가진 HWP5 서식.
const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-mcpedit-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

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
                "clientInfo": {"name": "session-edit-test", "version": "0"}
            }),
        );
        assert!(r["result"]["serverInfo"]["name"].is_string(), "{r}");
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
            let v: serde_json::Value = serde_json::from_str(line.trim())
                .unwrap_or_else(|e| panic!("stdout 이 JSON-RPC 가 아닙니다 ({e}): {line}"));
            if v.get("id").and_then(|i| i.as_i64()) == Some(id) {
                return v;
            }
        }
    }

    /// tools/call 후 content[0].text 를 JSON 으로 돌려준다. isError 는 호출부가 판정.
    fn call(&mut self, name: &str, args: serde_json::Value) -> (bool, serde_json::Value) {
        let r = self.request(
            "tools/call",
            serde_json::json!({"name": name, "arguments": args}),
        );
        let result = &r["result"];
        let is_error = result["isError"].as_bool().unwrap_or(false);
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let v = serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text));
        (is_error, v)
    }

    fn open(&mut self, path: &Path) -> String {
        let (err, v) = self.call(
            "hwp_open",
            serde_json::json!({"path": path.to_str().unwrap()}),
        );
        assert!(!err, "hwp_open 실패: {v}");
        v["docId"].as_str().expect("docId").to_string()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn session_fill_accumulates_and_save_preserves_hwp5() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    // ① 두 번에 나눠 채운다 — 핸들에 누적되어야 한다.
    let (err, v1) = s.call(
        "hwp_doc_fill_fields",
        serde_json::json!({"docId": doc_id, "data": {"회사명": "세션 주식회사"}}),
    );
    assert!(!err, "1차 채움 실패: {v1}");
    assert_eq!(v1["filledCount"].as_u64(), Some(1), "{v1}");
    assert_eq!(v1["schemaVersion"], "1.0", "{v1}");

    let (err, v2) = s.call(
        "hwp_doc_fill_fields",
        serde_json::json!({"docId": doc_id, "data": {"작성자": "김세션"}}),
    );
    assert!(!err, "2차 채움 실패: {v2}");

    // ② 저장 전에는 디스크에 산출물이 없다 — save 가 유일한 기록 지점이다.
    let out = temp_path("hwp5", "hwp");
    assert!(!out.exists());

    let (err, sv) = s.call(
        "hwp_doc_save",
        serde_json::json!({"docId": doc_id, "output": out.to_str().unwrap()}),
    );
    assert!(!err, "저장 실패: {sv}");
    assert_eq!(sv["outputFormat"], "hwp5", "형식 보존(HWP5): {sv}");
    assert!(sv["bytes"].as_u64().unwrap_or(0) > 0, "{sv}");
    assert!(out.exists(), "저장 후 산출물이 있어야 합니다");

    // ③ 산출물을 **다시 읽어** 두 값이 모두 반영됐는지 대조한다 (보고를 믿지 않는다).
    let reread = run_cli(&["fields", out.to_str().unwrap(), "--json"]);
    let rv: serde_json::Value = serde_json::from_slice(&reread.stdout).expect("fields --json");
    let get = |name: &str| -> String {
        rv["fields"]
            .as_array()
            .expect("fields")
            .iter()
            .find(|f| f["name"] == name)
            .and_then(|f| f["value"].as_str())
            .unwrap_or("")
            .to_string()
    };
    assert_eq!(get("회사명"), "세션 주식회사", "{rv}");
    assert_eq!(get("작성자"), "김세션", "누적 편집이 저장돼야 합니다: {rv}");

    let _ = std::fs::remove_file(&out);
}

#[test]
fn session_save_preserves_hwpx_format() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    // HWPX 입력을 만들어 형식 보존의 반대편도 고정한다.
    let hwpx_src = temp_path("src", "hwpx");
    let conv = run_cli(&[
        "export-hwpx",
        src.to_str().unwrap(),
        hwpx_src.to_str().unwrap(),
    ]);
    assert_eq!(conv.status.code(), Some(0), "사전 변환 실패");

    let mut s = Server::started();
    let doc_id = s.open(&hwpx_src);
    let (err, v) = s.call(
        "hwp_doc_fill_fields",
        serde_json::json!({"docId": doc_id, "data": {"회사명": "HWPX 보존"}}),
    );
    assert!(!err, "{v}");

    let out = temp_path("hwpx", "hwpx");
    let (err, sv) = s.call(
        "hwp_doc_save",
        serde_json::json!({"docId": doc_id, "output": out.to_str().unwrap()}),
    );
    assert!(!err, "{sv}");
    assert_eq!(sv["outputFormat"], "hwpx", "형식 보존(HWPX): {sv}");

    let info = run_cli(&["info", out.to_str().unwrap(), "--json"]);
    let iv: serde_json::Value = serde_json::from_slice(&info.stdout).expect("info --json");
    assert_eq!(iv["format"], "hwpx", "산출물 실측 형식: {iv}");

    let _ = std::fs::remove_file(&hwpx_src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn session_fill_reports_judgment_fields_like_stateless() {
    // 무상태 hwp_fill_fields 와 같은 판정 어휘 — notFound 는 침묵하지 않는다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, v) = s.call(
        "hwp_doc_fill_fields",
        serde_json::json!({"docId": doc_id, "data": {"회사명": "A", "존재하지않는필드": "B"}}),
    );
    assert!(!err, "{v}");
    assert_eq!(v["filledCount"].as_u64(), Some(1), "{v}");
    let missing = v["notFound"].as_array().expect("notFound");
    assert!(
        missing.iter().any(|m| m == "존재하지않는필드"),
        "없는 이름은 보고돼야 합니다: {v}"
    );
    assert!(v["ambiguous"].is_array(), "{v}");
}

#[test]
fn session_edit_tools_reject_closed_handle() {
    let mut s = Server::started();
    for (name, args) in [
        (
            "hwp_doc_fill_fields",
            serde_json::json!({"docId": "doc-999", "data": {"a": "b"}}),
        ),
        (
            "hwp_doc_save",
            serde_json::json!({"docId": "doc-999", "output": "x.hwp"}),
        ),
    ] {
        let (err, v) = s.call(name, args);
        assert!(err, "{name} 는 닫힌 핸들에 isError 여야 합니다: {v}");
    }
}

#[test]
fn session_edit_tools_are_listed() {
    let mut s = Server::started();
    let r = s.request("tools/list", serde_json::json!({}));
    let names: Vec<String> = r["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    for t in ["hwp_doc_fill_fields", "hwp_doc_save"] {
        assert!(names.contains(&t.to_string()), "{t} 누락: {names:?}");
    }
}
