//! MCP 인자 검증 계약 — "생략"과 "형식 오류"를 가른다.
//!
//! `args.get(k).and_then(|v| v.as_u64())` 류의 강제변환은 두 경우를 모두 `None` 으로
//! 뭉갰다. `page: -1` 은 "page 생략"과 같아져 한 쪽만 달라던 호출이 **문서 전체**를
//! 성공 응답으로 받아 갔고, `caseSensitive: "false"` 는 기본값 true 로 되돌아가
//! 검색·치환 대상 집합이 조용히 달라졌다. 여기서 고정하는 것은 *조용한 오답의 부재*다.
//!
//! `hwp_search` 축은 배선 결함이다 — `{query}` 가 위치 argv 에 그대로 놓여 '-' 로
//! 시작하는 검색어가 플래그로 먹혔다. CLI 에 `--` 종결자를 넣고 템플릿에서 `--json` 을
//! 구분자 **앞**으로 옮긴다(뒤는 전부 위치 인자다).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// 16쪽 HWP3 표본 — page 축의 "전체 vs 한 쪽"을 눈에 띄게 가른다.
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
                "clientInfo": {"name": "arg-validation-test", "version": "0"}
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

/// isError 응답의 사람 읽는 문자열.
fn msg(v: &serde_json::Value) -> String {
    v.as_str()
        .map(String::from)
        .unwrap_or_else(|| v.to_string())
}

/// 회귀의 핵심: 오타 page 가 "생략"으로 뭉개지면 **문서 전체**가 성공으로 나갔다.
#[test]
fn doc_text_mistyped_page_is_rejected_not_widened_to_whole_document() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc = s.open(&src);

    let (err, whole) = s.call("hwp_doc_text", serde_json::json!({"docId": doc}));
    assert!(!err, "{whole}");
    let all = whole["pageCount"].as_u64().expect("pageCount");
    assert!(all > 1, "전제: 여러 쪽 표본이어야 합니다: {whole}");

    for bad in [
        serde_json::json!(-1),
        serde_json::json!(2.5),
        serde_json::json!("3"),
        serde_json::json!(true),
    ] {
        let (err, v) = s.call(
            "hwp_doc_text",
            serde_json::json!({"docId": doc, "page": bad}),
        );
        assert!(err, "page={bad} 는 isError 여야 합니다: {v}");
        let m = msg(&v);
        assert!(m.contains("page"), "어느 인자인지 지목해야 합니다: {m}");
        assert!(m.contains("정수"), "무엇이 틀렸는지 밝혀야 합니다: {m}");
    }

    // 유효한 한 쪽 요청은 그대로 한 쪽이다(고침이 정상 경로를 좁히지 않았는가).
    let (err, one) = s.call("hwp_doc_text", serde_json::json!({"docId": doc, "page": 0}));
    assert!(!err, "{one}");
    assert_eq!(one["pages"].as_array().map(|a| a.len()), Some(1), "{one}");
}

/// 다수 호스트가 미지정 선택 인자를 `null` 로 직렬화한다 — 이를 오류로 만들면 멀쩡한
/// 호출이 깨진다. `null` 만 관용하고 나머지 오타는 거부한다.
#[test]
fn doc_text_null_page_still_means_omitted() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc = s.open(&src);
    let (err, v) = s.call(
        "hwp_doc_text",
        serde_json::json!({"docId": doc, "page": serde_json::Value::Null}),
    );
    assert!(!err, "null page 는 '생략'이어야 합니다: {v}");
    assert!(v["pageCount"].as_u64().unwrap_or(0) > 1, "{v}");
}

/// 고침 전에는 누락과 오타가 둘 다 "page 가 필요합니다" — 보냈는데 없다고 하는 오진.
#[test]
fn render_page_separates_missing_from_mistyped() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = std::env::temp_dir().join(format!("rhwp-argval-{}.svg", std::process::id()));
    let out = out.to_str().unwrap().to_string();
    let mut s = Server::started();
    let doc = s.open(&src);

    let (err, absent) = s.call(
        "hwp_doc_render_page",
        serde_json::json!({"docId": doc, "output": out}),
    );
    assert!(err, "{absent}");
    assert!(msg(&absent).contains("필요합니다"), "{absent}");

    for bad in [serde_json::json!(-1), serde_json::json!("0")] {
        let (err, v) = s.call(
            "hwp_doc_render_page",
            serde_json::json!({"docId": doc, "page": bad, "output": out}),
        );
        assert!(err, "page={bad}: {v}");
        let m = msg(&v);
        assert!(
            m.contains("정수") && !m.contains("필요합니다"),
            "형식 오류는 '없음'과 다른 문구여야 합니다: {m}"
        );
    }
    assert!(!Path::new(&out).exists(), "거부 경로가 산출물을 남겼습니다");
}

/// 조용한 축 되돌림: `"false"`/`0` 이 as_bool() 에서 None → 기본값 true.
/// 치환 쪽은 문서가 바뀌는 축이라 되돌릴 수 없다.
#[test]
fn case_sensitive_rejects_non_boolean_in_search_and_replace() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc = s.open(&src);
    for bad in [
        serde_json::json!("false"),
        serde_json::json!(0),
        serde_json::json!("true"),
    ] {
        let (err, v) = s.call(
            "hwp_doc_search",
            serde_json::json!({"docId": doc, "query": "The", "caseSensitive": bad}),
        );
        assert!(err, "search caseSensitive={bad}: {v}");
        assert!(msg(&v).contains("caseSensitive"), "{v}");

        let (err, v) = s.call(
            "hwp_doc_replace_text",
            serde_json::json!({"docId": doc, "find": "z", "replace": "y", "caseSensitive": bad}),
        );
        assert!(err, "replace caseSensitive={bad}: {v}");
        assert!(msg(&v).contains("caseSensitive"), "{v}");
    }

    // 진짜 불리언은 그대로 통하고 봉투에 반영된다.
    let (err, v) = s.call(
        "hwp_doc_search",
        serde_json::json!({"docId": doc, "query": "The", "caseSensitive": false}),
    );
    assert!(!err, "{v}");
    assert_eq!(v["caseSensitive"], false, "{v}");
}

/// 고침 전: `table:-1` 이어도 "table/row/col 이 필요합니다" — 있는 인자를 없다고 한다.
#[test]
fn set_cell_names_the_bad_axis_instead_of_claiming_absence() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc = s.open(&src);
    for (axis, args) in [
        (
            "table",
            serde_json::json!({"docId": doc, "table": -1, "row": 0, "col": 0, "text": "x"}),
        ),
        (
            "row",
            serde_json::json!({"docId": doc, "table": 0, "row": -2, "col": 0, "text": "x"}),
        ),
        (
            "col",
            serde_json::json!({"docId": doc, "table": 0, "row": 0, "col": "2", "text": "x"}),
        ),
    ] {
        let (err, v) = s.call("hwp_doc_set_cell", args);
        assert!(err, "{axis}: {v}");
        let m = msg(&v);
        assert!(m.contains(axis), "틀린 축을 지목해야 합니다: {m}");
        assert!(
            !m.contains("table/row/col 이 필요합니다"),
            "있는 인자를 없다고 하면 안 됩니다: {m}"
        );
    }

    // 진짜 누락은 여전히 "필요합니다".
    let (err, v) = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({"docId": doc, "row": 0, "col": 0, "text": "x"}),
    );
    assert!(err, "{v}");
    assert!(msg(&v).contains("table 가 필요합니다"), "{v}");
}

/// `"true"`/`1`/`"yes"` 는 고침 전 조용히 false(검정 정규화)로 되돌아갔다 —
/// 셀 서식이 말없이 사라지는 경로다.
#[test]
fn keep_style_rejects_non_boolean() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc = s.open(&src);
    for bad in [
        serde_json::json!("true"),
        serde_json::json!(1),
        serde_json::json!("yes"),
    ] {
        let (err, v) = s.call(
            "hwp_doc_set_cell",
            serde_json::json!({
                "docId": doc, "table": 0, "row": 0, "col": 0, "text": "x", "keepStyle": bad
            }),
        );
        assert!(err, "keepStyle={bad}: {v}");
        assert!(msg(&v).contains("keepStyle"), "{v}");
    }
}

/// `--` 뒤는 전부 위치 인자다 — 하이픈으로 시작하는 검색어를 그대로 찾는다.
#[test]
fn search_cli_honours_end_of_options_separator() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let p = src.to_str().unwrap();

    let out = run_cli(&["search", p, "--json", "--", "-i"]);
    assert_eq!(out.status.code(), Some(0), "exit: {out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("search --json");
    assert_eq!(v["query"], "-i", "검색어가 그대로 실려야 합니다: {v}");
    assert_eq!(
        v["caseSensitive"], true,
        "'-i' 는 플래그가 아니라 검색어다 — 축이 뒤집히면 안 됩니다: {v}"
    );

    // 구분자 **앞**의 옵션은 여전히 옵션이다(회귀 반대편).
    let out = run_cli(&["search", p, "--json", "--ignore-case", "--", "The"]);
    assert_eq!(out.status.code(), Some(0), "exit: {out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("search --json");
    assert_eq!(v["query"], "The", "{v}");
    assert_eq!(v["caseSensitive"], false, "{v}");
}

/// 선언 배선 자체의 회귀 — `--json` 은 `--` 앞, `{query}` 는 뒤.
#[test]
fn hwp_search_tool_wires_query_after_separator() {
    let caps = run_cli(&["capabilities", "--mcp"]);
    let m: serde_json::Value = serde_json::from_slice(&caps.stdout).expect("capabilities --mcp");
    let tool = m["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_search")
        .expect("hwp_search 선언");
    let targs: Vec<&str> = tool["cli"]["args"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a.as_str())
        .collect();
    let sep = targs.iter().position(|a| *a == "--").expect("`--` 배선");
    let q = targs.iter().position(|a| *a == "{query}").expect("{query}");
    let j = targs.iter().position(|a| *a == "--json").expect("--json");
    assert!(sep < q, "검색어는 구분자 뒤: {targs:?}");
    assert!(
        j < sep,
        "--json 은 구분자 앞이어야 위치 인자가 되지 않는다: {targs:?}"
    );

    // 실행 대조: 서버 경유로도 하이픈 검색어가 통해야 한다.
    let src = sample();
    if !src.exists() {
        return;
    }
    let mut s = Server::started();
    let (err, v) = s.call(
        "hwp_search",
        serde_json::json!({"path": src.to_str().unwrap(), "query": "-i"}),
    );
    assert!(!err, "하이픈 검색어가 거부되면 안 됩니다: {v}");
    assert_eq!(v["query"], "-i", "{v}");
    assert_eq!(v["caseSensitive"], true, "{v}");
}

/// 선언은 `minimum: 0`·`0 기준`인데 CLI/코어는 1 기준이라 0 을 거부했다 — 선언만 읽고
/// 호출을 조립하는 에이전트는 첫 호출에서 exit 1 을 밟는다.
#[test]
fn split_declaration_page_base_matches_execution() {
    let caps = run_cli(&["capabilities", "--mcp"]);
    let m: serde_json::Value = serde_json::from_slice(&caps.stdout).expect("capabilities --mcp");
    let tool = m["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_split_document")
        .expect("hwp_split_document 선언");
    for axis in ["from", "to"] {
        let prop = &tool["inputSchema"]["properties"][axis];
        assert_eq!(prop["minimum"].as_u64(), Some(1), "{axis} 최솟값: {prop}");
        let d = prop["description"].as_str().unwrap_or_default();
        assert!(d.contains("1 기준"), "{axis} 기수를 밝혀야 합니다: {d}");
        assert!(
            !d.contains("0 기준, 포함"),
            "{axis} 기수가 실행과 어긋납니다: {d}"
        );
    }

    // 실행 대조: 선언 최솟값-1 은 거부되고, 최솟값은 통해야 한다.
    let src = sample();
    if !src.exists() {
        return;
    }
    let out = std::env::temp_dir().join(format!("rhwp-argval-split-{}.hwp", std::process::id()));
    let bad = run_cli(&[
        "extract-pages",
        src.to_str().unwrap(),
        out.to_str().unwrap(),
        "--from",
        "0",
        "--to",
        "1",
        "--json",
    ]);
    assert_ne!(bad.status.code(), Some(0), "0 기준 호출은 거부돼야 합니다");
    let good = run_cli(&[
        "extract-pages",
        src.to_str().unwrap(),
        out.to_str().unwrap(),
        "--from",
        "1",
        "--to",
        "1",
        "--json",
    ]);
    assert_eq!(good.status.code(), Some(0), "선언 최솟값은 통해야 합니다");
    let _ = std::fs::remove_file(&out);
}
