//! [#3627 / 로드맵 R74] MCP resources 표면 계약 — 스키마·레시피 확장분.
//!
//! 계약 둘: ① resources/list 가 스키마 3종·레시피 6편을 포함해 광고한다.
//! ② **광고한 모든 URI 가 실제로 읽힌다** — 목록과 실물의 드리프트 가드.
//!   목록을 하드코딩으로 재대조하지 않고 list → read 왕복으로 잡는다: 리소스가
//!   늘어도 이 가드는 자동으로 따라오고, 목록에만 넣고 read 분기를 빠뜨리는
//!   실수(선언 회피의 역방향)가 즉시 red 가 된다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

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
                "clientInfo": {"name": "resources-contract", "version": "0"}
            }),
        );
        assert!(
            r["result"]["capabilities"]["resources"].is_object(),
            "resources capability 선언이 없습니다: {r}"
        );
        let msg = serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"});
        writeln!(s.stdin, "{msg}").expect("알림 쓰기 실패");
        s.stdin.flush().expect("flush");
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
                .unwrap_or_else(|e| panic!("stdout 이 순수 JSON-RPC 가 아닙니다 ({e}): {line}"));
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

fn listed_resources() -> Vec<serde_json::Value> {
    let mut s = Server::started();
    let r = s.request("resources/list", serde_json::json!({}));
    r["result"]["resources"]
        .as_array()
        .unwrap_or_else(|| panic!("resources/list 에 resources 배열이 없습니다: {r}"))
        .clone()
}

#[test]
fn list_advertises_schemas_and_recipes() {
    let uris: Vec<String> = listed_resources()
        .iter()
        .map(|x| x["uri"].as_str().expect("uri 누락").to_string())
        .collect();
    for expected in [
        "rhwp://schemas/ir",
        "rhwp://schemas/plan",
        "rhwp://schemas/capabilities",
        "rhwp://recipes/01_fill_form_and_submit.md",
        "rhwp://recipes/02_table_csv_roundtrip.md",
        "rhwp://recipes/03_redact_before_sharing.md",
        "rhwp://recipes/04_safety_check_untrusted_doc.md",
        "rhwp://recipes/05_mail_merge_batch_fill.md",
        "rhwp://recipes/06_visual_regression_before_after.md",
    ] {
        assert!(
            uris.iter().any(|u| u == expected),
            "resources/list 에 {expected} 가 없습니다 (실물: {uris:?})"
        );
    }
}

#[test]
fn every_listed_resource_reads_back() {
    let listed = listed_resources();
    let mut s = Server::started();
    for res in &listed {
        let uri = res["uri"].as_str().expect("uri 누락");
        let declared_mime = res["mimeType"].as_str().expect("mimeType 누락");
        let r = s.request("resources/read", serde_json::json!({ "uri": uri }));
        assert!(
            r.get("error").is_none(),
            "{uri}: 광고된 리소스가 읽히지 않습니다 — {r}"
        );
        let c = &r["result"]["contents"][0];
        assert_eq!(c["uri"].as_str(), Some(uri), "{uri}: contents.uri 불일치");
        assert_eq!(
            c["mimeType"].as_str(),
            Some(declared_mime),
            "{uri}: 목록과 read 의 mimeType 이 갈라졌습니다"
        );
        let text = c["text"].as_str().unwrap_or_default();
        assert!(!text.trim().is_empty(), "{uri}: 본문이 비었습니다");
        if declared_mime == "application/json" {
            let v: serde_json::Value = serde_json::from_str(text)
                .unwrap_or_else(|e| panic!("{uri}: application/json 인데 파싱 실패 ({e})"));
            if uri == "rhwp://schemas/ir" {
                assert!(
                    v.get("irSchemaVersion").is_some(),
                    "{uri}: irSchemaVersion 최상위 키가 없습니다 (R82 규약)"
                );
            }
        }
    }
}

#[test]
fn unknown_resource_is_resource_not_found() {
    let mut s = Server::started();
    let r = s.request(
        "resources/read",
        serde_json::json!({ "uri": "rhwp://schemas/없는-스키마" }),
    );
    assert_eq!(
        r["error"]["code"].as_i64(),
        Some(-32002),
        "미지 URI 는 -32002 여야 합니다: {r}"
    );
    assert_eq!(
        r["error"]["data"]["uri"].as_str(),
        Some("rhwp://schemas/없는-스키마"),
        "오류에 문제의 uri 가 실려야 합니다: {r}"
    );
}
