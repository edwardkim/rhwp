//! [#3609] 세션 도구의 인자 타입은 **거부**로 끝나야지 기본값으로 흡수되면 안 된다.
//!
//! `args.get(k).and_then(as_u64)` 같은 관용구는 "없음" 과 "타입이 틀림" 을 `None`
//! 하나로 뭉갠다. 그러면 잘못 만든 인자가 **오류가 아니라 기본 동작**이 되어 돌아온다.
//!
//! - `hwp_doc_text { page: -1 }` → 한 쪽을 달라고 했는데 **문서 전체**가 온다.
//!   대형 문서에서는 컨텍스트가 통째로 날아가고, 에이전트는 성공으로 읽는다.
//! - `hwp_doc_search { caseSensitive: "false" }` → 요청과 **반대로** 실행하고
//!   봉투에는 `caseSensitive: true` 를 적어 돌려준다. 뒤집힌 줄 알 길이 없다.
//!
//! 범위 검사(`page: 99`)는 원래도 제대로 거절했다. 타입만 통과하던 비대칭을 없앤다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Server {
    /// initialize 까지 마치고 샘플을 연 서버. 핸들 id 를 함께 돌려준다.
    fn opened() -> (Server, String) {
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
        s.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "arg-typing-test", "version": "0"}
            }),
        );
        let r = s.call(
            "hwp_open",
            serde_json::json!({"path": sample().to_string_lossy()}),
        );
        let text = r["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("hwp_open 응답에 text 가 없습니다: {r}"));
        let v: serde_json::Value = serde_json::from_str(text).expect("hwp_open 봉투 JSON");
        let doc_id = v["docId"]
            .as_str()
            .unwrap_or_else(|| panic!("docId 가 없습니다: {v}"))
            .to_string();
        (s, doc_id)
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

    fn call(&mut self, name: &str, args: serde_json::Value) -> serde_json::Value {
        self.request(
            "tools/call",
            serde_json::json!({"name": name, "arguments": args}),
        )
    }

    /// 도구가 isError 로 거부했는지.
    fn rejected(&mut self, name: &str, args: serde_json::Value) -> bool {
        self.call(name, args)["result"]["isError"] == true
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn skip_if_no_sample() -> bool {
    if sample().exists() {
        return false;
    }
    eprintln!("샘플 없음 — 건너뜀");
    true
}

#[test]
fn malformed_page_is_rejected_instead_of_dumping_the_whole_document() {
    if skip_if_no_sample() {
        return;
    }
    let (mut s, doc) = Server::opened();

    // 기준선: 생략은 전체, 정상값은 한 쪽. 이 둘이 구분되지 않으면 아래 판정이 공허하다.
    let all = s.call("hwp_doc_text", serde_json::json!({"docId": doc}));
    let all: serde_json::Value =
        serde_json::from_str(all["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    let total = all["pageCount"].as_u64().expect("pageCount");
    assert!(total > 1, "표본이 2쪽 이상이어야 이 테스트가 의미 있다");

    let one = s.call("hwp_doc_text", serde_json::json!({"docId": doc, "page": 0}));
    let one: serde_json::Value =
        serde_json::from_str(one["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(one["pageCount"], 1, "page:0 은 한 쪽만 와야 한다");

    for bad in [
        serde_json::json!(-1),
        serde_json::json!(1.5),
        serde_json::json!("2"),
        serde_json::json!(true),
        serde_json::json!([0]),
    ] {
        let r = s.call(
            "hwp_doc_text",
            serde_json::json!({"docId": doc, "page": bad}),
        );
        assert_eq!(
            r["result"]["isError"], true,
            "page: {bad} 가 거부되지 않았다 — 잘못된 인자가 조용히 전체 덤프({total}쪽)로 \
             떨어지면 에이전트는 한 쪽을 받았다고 믿는다: {r}"
        );
    }
}

#[test]
fn malformed_boolean_is_rejected_instead_of_flipping_to_the_default() {
    if skip_if_no_sample() {
        return;
    }
    let (mut s, doc) = Server::opened();

    // 기준선: 제대로 준 false 는 봉투에 false 로 실린다.
    let ok = s.call(
        "hwp_doc_search",
        serde_json::json!({"docId": doc, "query": "의", "caseSensitive": false}),
    );
    let ok: serde_json::Value =
        serde_json::from_str(ok["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(
        ok["caseSensitive"], false,
        "불리언 false 가 봉투에 반영되지 않는다: {ok}"
    );

    for bad in [
        serde_json::json!("false"),
        serde_json::json!(0),
        serde_json::json!("true"),
    ] {
        assert!(
            s.rejected(
                "hwp_doc_search",
                serde_json::json!({"docId": doc, "query": "의", "caseSensitive": bad}),
            ),
            "caseSensitive: {bad} 가 거부되지 않았다 — 기본값 true 로 흡수되면 요청과 \
             반대로 실행하고 봉투에는 true 를 적어 돌려준다"
        );
        assert!(
            s.rejected(
                "hwp_doc_replace_text",
                serde_json::json!({
                    "docId": doc, "find": "의", "replace": "X", "caseSensitive": bad
                }),
            ),
            "replace_text 의 caseSensitive: {bad} 가 거부되지 않았다"
        );
    }

    for bad in [serde_json::json!("true"), serde_json::json!(1)] {
        assert!(
            s.rejected(
                "hwp_doc_set_cell",
                serde_json::json!({
                    "docId": doc, "table": 0, "row": 0, "col": 0,
                    "text": "X", "keepStyle": bad
                }),
            ),
            "keepStyle: {bad} 가 거부되지 않았다 — 스타일 유지 요청이 정규화로 뒤집힌다"
        );
    }
}

#[test]
fn missing_and_mistyped_coordinates_say_different_things() {
    if skip_if_no_sample() {
        return;
    }
    let (mut s, doc) = Server::opened();

    let missing = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({"docId": doc, "row": 0, "col": 0, "text": "X"}),
    );
    let missing = missing["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let mistyped = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({"docId": doc, "table": -1, "row": 0, "col": 0, "text": "X"}),
    );
    let mistyped = mistyped["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    assert!(
        missing.contains("필요"),
        "누락은 '필요' 라고 말해야 한다: {missing}"
    );
    assert!(
        !mistyped.contains("필요"),
        "타입 오류에 '필요합니다' 를 돌려주면 에이전트는 보내지 않은 줄 알고 같은 값을 \
         다시 보낸다 — 무한 재시도의 씨앗이다: {mistyped}"
    );
    assert!(
        mistyped.contains("-1"),
        "받은 값을 되돌려줘야 무엇이 틀렸는지 안다: {mistyped}"
    );
}
