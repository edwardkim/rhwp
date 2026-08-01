//! [#3601] `mcp-serve` 세션 조회·치환 — 열린 핸들에서 재파싱 없이 검색하고
//! (hwp_doc_search), 치환을 IR 에 누적한다(hwp_doc_replace_text).
//!
//! #3598(fill/save)로 세션 편집이 열렸지만 실무 루프의 나머지 두 동작(찾기·다듬기)이
//! 무상태 전용이라 "열고 → 찾고 → 바꾸고 → 채우고 → 한 번 저장" 흐름이 끊겨 있었다.
//! 주소 어휘(matches[].page 등)와 치환 계수는 무상태 search/replace-text 와 같은 코어
//! 경로(grep/replace_all_native)를 재사용한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// 본문에 조사 "의" 가 다수 나오는 HWP3 표본 — 검색·치환의 안정 표적.
const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-mcpq-{tag}-{}-{}.{ext}",
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
                "clientInfo": {"name": "session-query-test", "version": "0"}
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
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn session_search_matches_stateless_search() {
    // 세션 검색은 무상태 `search --json` 과 같은 매치 수·주소 어휘를 내야 한다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let stateless = run_cli(&["search", src.to_str().unwrap(), "의", "--json"]);
    let sv: serde_json::Value = serde_json::from_slice(&stateless.stdout).expect("search --json");
    let expected = sv["matchCount"].as_u64().expect("matchCount");
    assert!(expected >= 1, "전제: 표본에 검색어가 있어야 합니다");

    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, v) = s.call(
        "hwp_doc_search",
        serde_json::json!({"docId": doc_id, "query": "의"}),
    );
    assert!(!err, "{v}");
    assert_eq!(v["matchCount"].as_u64(), Some(expected), "{v}");
    let first = &v["matches"][0];
    assert!(first["section"].is_u64(), "주소 어휘(section): {first}");
    assert!(first["context"].is_string(), "주소 어휘(context): {first}");
}

#[test]
fn session_replace_accumulates_until_save() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    // ① 핸들 IR 에 치환 — 디스크 미기록.
    let marker = "치환검증마커";
    let (err, rv) = s.call(
        "hwp_doc_replace_text",
        serde_json::json!({"docId": doc_id, "find": "의", "replace": marker}),
    );
    assert!(!err, "{rv}");
    let replaced = rv["replacedCount"].as_u64().expect("replacedCount");
    assert!(replaced >= 1, "{rv}");

    // ② 같은 핸들 검색으로 치환이 IR 에 반영됐는지 본다(재파싱 없이).
    let (err, qv) = s.call(
        "hwp_doc_search",
        serde_json::json!({"docId": doc_id, "query": marker}),
    );
    assert!(!err, "{qv}");
    assert!(qv["matchCount"].as_u64().unwrap_or(0) >= 1, "{qv}");

    // ③ save 후 산출물 재독 — 서버 밖 CLI 로 대조한다.
    let out = temp_path("repl", "hwp");
    let (err, sv) = s.call(
        "hwp_doc_save",
        serde_json::json!({"docId": doc_id, "output": out.to_str().unwrap()}),
    );
    assert!(!err, "{sv}");
    let reread = run_cli(&["search", out.to_str().unwrap(), marker, "--json"]);
    let rr: serde_json::Value = serde_json::from_slice(&reread.stdout).expect("reread search");
    assert!(
        rr["matchCount"].as_u64().unwrap_or(0) >= 1,
        "치환이 산출물에 남아야 합니다: {rr}"
    );

    let _ = std::fs::remove_file(&out);
}

#[test]
fn session_replace_zero_matches_reports_zero() {
    // 치환 0건은 오류가 아니라 계수 0 보고다 — 소비자가 --find 표기를 재점검한다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, v) = s.call(
        "hwp_doc_replace_text",
        serde_json::json!({"docId": doc_id, "find": "존재하지않는문자열XYZW", "replace": "무"}),
    );
    assert!(!err, "{v}");
    assert_eq!(v["replacedCount"].as_u64(), Some(0), "{v}");
}

#[test]
fn session_query_tools_reject_closed_handle() {
    let mut s = Server::started();
    for (name, args) in [
        (
            "hwp_doc_search",
            serde_json::json!({"docId": "doc-999", "query": "x"}),
        ),
        (
            "hwp_doc_replace_text",
            serde_json::json!({"docId": "doc-999", "find": "a", "replace": "b"}),
        ),
    ] {
        let (err, v) = s.call(name, args);
        assert!(err, "{name} 는 닫힌 핸들에 isError 여야 합니다: {v}");
    }
}

#[test]
fn session_query_tools_are_listed() {
    let mut s = Server::started();
    let r = s.request("tools/list", serde_json::json!({}));
    let names: Vec<String> = r["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    for t in ["hwp_doc_search", "hwp_doc_replace_text"] {
        assert!(names.contains(&t.to_string()), "{t} 누락: {names:?}");
    }
}

/// 대량 치환이 문서를 여러 쪽 늘려도 핸들의 페이지 어휘가 즉시 따라와야 한다.
/// 종전에는 replace 가 recompose 만 남기고 재페이지네이션하지 않아 pageCount 와
/// 검색 matches[].page 가 편집 전 레이아웃에 머물렀다.
#[test]
fn session_replace_repaginates_page_vocabulary() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    let (err, before) = s.call("hwp_doc_info", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{before}");
    let pages_before = before["pageCount"].as_u64().expect("pageCount");

    // 전 쪽에 흩어진 조사 하나를 장문 마커로 부풀려 강제로 쪽수를 늘린다.
    let marker = "쪽수팽창마커 ".repeat(20);
    let (err, rep) = s.call(
        "hwp_doc_replace_text",
        serde_json::json!({"docId": doc_id, "find": "의", "replace": marker}),
    );
    assert!(!err, "{rep}");
    assert!(
        rep["replacedCount"].as_u64().unwrap_or(0) > 10,
        "전제 확인: 대량 치환이어야 시험이 의미 있다: {rep}"
    );

    let (err, after) = s.call("hwp_doc_info", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{after}");
    let pages_after = after["pageCount"].as_u64().expect("pageCount");
    assert!(
        pages_after > pages_before,
        "치환 직후 세션 pageCount 가 편집 전 레이아웃에 머물러 있습니다 \
         (before={pages_before}, after={pages_after})"
    );

    // 검색의 page 주소도 새 레이아웃을 봐야 한다 — 문서가 크게 늘었으므로
    // 마커 일부는 편집 전 마지막 쪽 너머에 있어야 정상이다.
    let (err, sr) = s.call(
        "hwp_doc_search",
        serde_json::json!({"docId": doc_id, "query": "쪽수팽창마커"}),
    );
    assert!(!err, "{sr}");
    let max_page = sr["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .filter_map(|m| m["page"].as_u64())
        .max()
        .unwrap_or(0);
    assert!(
        max_page >= pages_before,
        "검색 page 주소가 편집 전 레이아웃에 머물러 있습니다 \
         (max_page={max_page}, 편집 전 쪽수={pages_before})"
    );
}
