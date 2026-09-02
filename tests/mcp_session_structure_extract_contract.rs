//! [#4856] `mcp-serve` 세션 조회 파리티 — 열린 핸들에서 재파싱 없이 개요·조문
//! 구조(hwp_doc_structure)와 날짜·금액·수량(hwp_doc_extract_data)을 뽑는다.
//!
//! 무상태 표면(export-structure·extract-data)에는 있으나 세션에는 없던 두 축을 채운다.
//! 계약: 두 도구가 tools/list 에 **나오고**(읽기 전용 annotations) tools/call 로
//! **불린다**. 봉투는 무상태 CLI 와 동형(같은 코어·같은 봉투 helper 재사용)이라, 세션
//! 판과 무상태 판이 같은 문서에서 같은 값을 낸다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// 개요/조문·숫자(‥장·100%)가 있는 HWP3 표본 — 구조·데이터 추출의 안정 표적.
const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
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
                "clientInfo": {"name": "session-structure-extract-test", "version": "0"}
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

    fn listed_tool(&mut self, name: &str) -> serde_json::Value {
        let r = self.request("tools/list", serde_json::json!({}));
        r["result"]["tools"]
            .as_array()
            .expect("tools 배열")
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("{name} 이 tools/list 에 없습니다"))
            .clone()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// ① 두 새 도구가 tools/list 에 나오고, 읽기 전용·비파괴 annotations 를 단다.
#[test]
fn new_session_tools_are_listed_read_only() {
    let mut s = Server::started();
    for name in ["hwp_doc_structure", "hwp_doc_extract_data"] {
        let t = s.listed_tool(name);
        assert!(t["description"].is_string(), "{name}: 설명 누락: {t}");
        assert!(
            t["inputSchema"]["properties"]["docId"].is_object(),
            "{name}: docId 입력 스키마 누락: {t}"
        );
        let a = &t["annotations"];
        assert_eq!(a["readOnlyHint"], true, "{name}: {a}");
        assert_eq!(a["destructiveHint"], false, "{name}: {a}");
        assert_eq!(a["idempotentHint"], true, "{name}: {a}");
        assert_eq!(a["openWorldHint"], false, "{name}: {a}");
    }
}

/// ② hwp_doc_structure 는 무상태 export-structure 와 동형 봉투를 낸다.
#[test]
fn session_structure_matches_stateless() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let stateless = run_cli(&["export-structure", src.to_str().unwrap(), "--json"]);
    let sv: serde_json::Value =
        serde_json::from_slice(&stateless.stdout).expect("export-structure --json");

    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, v) = s.call("hwp_doc_structure", serde_json::json!({"docId": doc_id}));
    assert!(!err, "hwp_doc_structure 실패: {v}");
    // 동형: mode·nodeCount 가 무상태 판과 같고, structure 는 객체다.
    assert_eq!(v["mode"], sv["mode"], "mode 동형: {v}");
    assert_eq!(v["nodeCount"], sv["nodeCount"], "nodeCount 동형: {v}");
    assert!(v["structure"].is_object(), "structure 객체: {v}");
    // source 자리에는 경로 대신 핸들 docId 가 들어간다.
    assert_eq!(v["source"], serde_json::json!(doc_id), "source=docId: {v}");
}

/// ②-b mode 를 명시하면 그대로 반영되고, 오타는 조용히 auto 로 되돌아가지 않는다.
#[test]
fn session_structure_mode_is_honored_and_typos_rejected() {
    let src = sample();
    if !src.exists() {
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    let (err, v) = s.call(
        "hwp_doc_structure",
        serde_json::json!({"docId": doc_id, "mode": "outline"}),
    );
    assert!(!err, "{v}");
    assert_eq!(v["mode"], "outline", "명시한 mode 가 반영돼야 합니다: {v}");

    let (err, v) = s.call(
        "hwp_doc_structure",
        serde_json::json!({"docId": doc_id, "mode": "nonsense"}),
    );
    assert!(err, "mode 오타는 isError 여야 합니다: {v}");
}

/// ③ hwp_doc_extract_data 는 무상태 extract-data 와 동형 봉투를 낸다.
#[test]
fn session_extract_data_matches_stateless() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let stateless = run_cli(&["extract-data", src.to_str().unwrap(), "--json"]);
    let sv: serde_json::Value =
        serde_json::from_slice(&stateless.stdout).expect("extract-data --json");
    let expected_total = sv["totalItemCount"].as_u64().expect("totalItemCount");

    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, v) = s.call("hwp_doc_extract_data", serde_json::json!({"docId": doc_id}));
    assert!(!err, "hwp_doc_extract_data 실패: {v}");
    assert_eq!(v["kind"], "all", "기본 kind=all: {v}");
    assert_eq!(
        v["totalItemCount"].as_u64(),
        Some(expected_total),
        "totalItemCount 동형: {v}"
    );
    assert_eq!(
        v["itemCount"], sv["itemCount"],
        "itemCount 동형(절단 없음): {v}"
    );
    assert_eq!(v["counts"], sv["counts"], "counts 동형: {v}");
    assert!(v["items"].is_array(), "items 배열: {v}");
    assert_eq!(v["source"], serde_json::json!(doc_id), "source=docId: {v}");
}

/// ③-b limit 절단은 표시만 자르고 총량은 totalItemCount 로 보고한다(전제: 표본에 2건+).
#[test]
fn session_extract_data_limit_truncates_display_only() {
    let src = sample();
    if !src.exists() {
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, full) = s.call("hwp_doc_extract_data", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{full}");
    let total = full["totalItemCount"].as_u64().unwrap_or(0);
    if total < 2 {
        eprintln!("표본 데이터가 2건 미만 — limit 절단 검증 건너뜀");
        return;
    }
    let (err, v) = s.call(
        "hwp_doc_extract_data",
        serde_json::json!({"docId": doc_id, "limit": 1}),
    );
    assert!(!err, "{v}");
    assert_eq!(
        v["itemCount"].as_u64(),
        Some(1),
        "표시는 1건으로 잘린다: {v}"
    );
    assert_eq!(
        v["totalItemCount"].as_u64(),
        Some(total),
        "총량은 그대로: {v}"
    );
    assert_eq!(v["truncated"], true, "절단 표지: {v}");
    // kind 오타는 조용히 all 로 되돌아가지 않는다.
    let (err, bad) = s.call(
        "hwp_doc_extract_data",
        serde_json::json!({"docId": doc_id, "kind": "nonsense"}),
    );
    assert!(err, "kind 오타는 isError 여야 합니다: {bad}");
}

/// ④ 닫힌/모르는 핸들은 isError + nextCall(hwp_open) 로 재발급을 안내한다(두 도구 공통).
#[test]
fn new_session_tools_recover_from_missing_handle() {
    let mut s = Server::started();
    for name in ["hwp_doc_structure", "hwp_doc_extract_data"] {
        let (err, v) = s.call(name, serde_json::json!({"docId": "doc-없음"}));
        assert!(err, "{name}: 모르는 핸들은 isError 여야 합니다: {v}");
        assert_eq!(
            v["nextCall"]["name"], "hwp_open",
            "{name}: 재발급 안내(nextCall=hwp_open): {v}"
        );
    }
}
