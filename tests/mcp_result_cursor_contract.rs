//! [#4854] 세션 도구 결과의 **이어보기** 계약 — 절단이 손실로 끝나지 않는다.
//!
//! #3787 S7 이 넣은 자원 상한(`maxMatches`·`maxChars`)은 컨텍스트 범람을 막지만,
//! 상한만 있고 이어보기가 없으면 호출자는 "컨텍스트를 지키고 뒤쪽을 잃거나"
//! "전부 받고 범람하거나" 둘 중 하나만 고를 수 있었다. `hwp_doc_search` 는 특히
//! `take(n)` 이라 n+1 번째 이후 매치에 **도달할 인자 자체가 없었다**.
//!
//! 이 파일이 못 박는 것은 네 가지다.
//!
//! 1. 창을 옮겨 가며 부르면 전수에 도달한다 — 상한을 켠 채로.
//! 2. 창들을 이어 붙이면 원본과 **정확히** 같다(중복 0·누락 0·순서 보존).
//! 3. "더 있는가"의 판정은 `nextOffset` 의 있음/없음 하나다.
//! 4. 인자를 생략하면 종전 봉투와 바이트까지 같다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// 본문에 조사 "의" 가 다수 나오는 HWP3 표본 — 창 넘기기를 여러 번 돌릴 표적.
const SAMPLE: &str = "samples/hwp3-sample.hwp";
/// 검색어. 표본에서 매치가 충분히 많아야 창 넘기기가 의미를 가진다.
const QUERY: &str = "의";

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
                "clientInfo": {"name": "result-cursor-test", "version": "0"}
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

    /// 도구 호출의 원문(text)까지 돌려준다 — "바이트까지 같다"를 검사하려면
    /// 파싱된 값이 아니라 직렬화 원문을 비교해야 한다.
    fn call_raw(&mut self, name: &str, args: serde_json::Value) -> (bool, String) {
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
        (is_error, text)
    }

    fn call(&mut self, name: &str, args: serde_json::Value) -> (bool, serde_json::Value) {
        let (is_error, text) = self.call_raw(name, args);
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

/// 매치의 신원 — 창을 이어 붙였을 때 중복·누락·순서를 판정할 좌표.
fn match_key(m: &serde_json::Value) -> String {
    format!(
        "{}:{}:{}:{}",
        m["section"], m["paragraph"], m["charOffset"], m["length"]
    )
}

#[test]
fn search_offset_reaches_matches_beyond_max_matches() {
    // 이 계약의 핵심. maxMatches 를 켠 채 창을 넘기면 **마지막 매치까지** 닿는다.
    // 종전(take(n) 전용)에는 n+1 번째 이후에 도달할 인자가 없었다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    let (err, full) = s.call(
        "hwp_doc_search",
        serde_json::json!({"docId": doc_id, "query": QUERY}),
    );
    assert!(!err, "{full}");
    let total = full["totalMatchCount"].as_u64().expect("totalMatchCount") as usize;
    assert!(
        total >= 3,
        "전제: 창 넘기기를 검사하려면 매치가 3건 이상이어야 합니다 (total={total})"
    );
    let expected: Vec<String> = full["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .map(match_key)
        .collect();

    // 한 번에 1건씩만 받는 가장 인색한 창으로 전수를 훑는다.
    let mut seen: Vec<String> = Vec::new();
    let mut offset = 0u64;
    let mut hops = 0;
    loop {
        let (err, v) = s.call(
            "hwp_doc_search",
            serde_json::json!({"docId": doc_id, "query": QUERY, "maxMatches": 1, "offset": offset}),
        );
        assert!(!err, "offset={offset} 에서 실패: {v}");
        assert_eq!(
            v["totalMatchCount"].as_u64(),
            Some(total as u64),
            "totalMatchCount 는 창과 무관하게 고정이어야 합니다: {v}"
        );
        for m in v["matches"].as_array().expect("matches") {
            seen.push(match_key(m));
        }
        hops += 1;
        assert!(hops <= total + 2, "창 넘기기가 끝나지 않습니다 (무한 루프)");
        match v.get("nextOffset").and_then(|n| n.as_u64()) {
            Some(next) => {
                assert!(next > offset, "nextOffset 이 전진하지 않습니다: {v}");
                offset = next;
            }
            // nextOffset 없음 = 더 없음. 이 신호 하나로 종료를 판정한다.
            None => break,
        }
    }

    assert_eq!(
        seen, expected,
        "창을 이어 붙인 결과가 전수와 다릅니다 (중복·누락·순서)"
    );
    assert_eq!(seen.len(), total, "전수 {total} 건에 도달하지 못했습니다");
}

#[test]
fn search_window_partition_is_exact_for_larger_windows() {
    // 창 크기가 1 이 아닐 때도 분할이 정확한가 — 경계에서 1건이 겹치거나 새면
    // 에이전트는 같은 자리를 두 번 고치거나 한 자리를 놓친다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (_, full) = s.call(
        "hwp_doc_search",
        serde_json::json!({"docId": doc_id, "query": QUERY}),
    );
    let total = full["totalMatchCount"].as_u64().expect("totalMatchCount") as usize;
    let expected: Vec<String> = full["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .map(match_key)
        .collect();

    for window in [2usize, 3, 7] {
        let mut seen: Vec<String> = Vec::new();
        let mut offset = 0u64;
        loop {
            let (err, v) = s.call(
                "hwp_doc_search",
                serde_json::json!({
                    "docId": doc_id, "query": QUERY, "maxMatches": window, "offset": offset
                }),
            );
            assert!(!err, "{v}");
            let got = v["matches"].as_array().expect("matches");
            assert!(got.len() <= window, "창 크기를 넘겨 반환했습니다: {v}");
            for m in got {
                seen.push(match_key(m));
            }
            match v.get("nextOffset").and_then(|n| n.as_u64()) {
                Some(next) => offset = next,
                None => break,
            }
        }
        assert_eq!(seen, expected, "창={window} 에서 분할이 어긋났습니다");
        assert_eq!(seen.len(), total, "창={window} 에서 전수 미도달");
    }
}

#[test]
fn search_offset_past_total_is_success_not_error() {
    // 마지막 창을 넘겨 부르는 일은 정상 루프에서 일어난다. 여기서 오류를 내면
    // 성실한 호출자의 마지막 한 번이 **항상** 실패한다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (_, full) = s.call(
        "hwp_doc_search",
        serde_json::json!({"docId": doc_id, "query": QUERY}),
    );
    let total = full["totalMatchCount"].as_u64().expect("totalMatchCount");

    let (err, v) = s.call(
        "hwp_doc_search",
        serde_json::json!({"docId": doc_id, "query": QUERY, "offset": total + 10}),
    );
    assert!(
        !err,
        "총량 초과 오프셋은 오류가 아니라 '더 없음'입니다: {v}"
    );
    assert_eq!(v["matches"].as_array().map(Vec::len), Some(0), "{v}");
    assert!(
        v.get("nextOffset").is_none(),
        "더 없는데 nextOffset 이 붙었습니다: {v}"
    );
    assert_eq!(
        v["totalMatchCount"].as_u64(),
        Some(total),
        "총량은 창과 무관해야 합니다: {v}"
    );
}

#[test]
fn omitting_offset_keeps_legacy_envelope_byte_identical() {
    // 이어보기는 **추가 전용**이다. 인자를 안 보내면 종전과 같은 바이트여야
    // 기존 호출자의 스냅샷·해시가 깨지지 않는다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    let (_, omitted) = s.call_raw(
        "hwp_doc_search",
        serde_json::json!({"docId": doc_id, "query": QUERY}),
    );
    let (_, explicit_zero) = s.call_raw(
        "hwp_doc_search",
        serde_json::json!({"docId": doc_id, "query": QUERY, "offset": 0}),
    );
    assert_eq!(
        omitted, explicit_zero,
        "offset 생략과 offset:0 은 같은 봉투여야 합니다"
    );
    let parsed: serde_json::Value = serde_json::from_str(&omitted).expect("봉투 JSON");
    assert!(
        parsed.get("nextOffset").is_none(),
        "상한이 없어 전수를 실었는데 nextOffset 이 붙었습니다: {parsed}"
    );
    assert!(
        parsed.get("offset").is_none(),
        "기본 창에는 offset 을 싣지 않습니다: {parsed}"
    );

    let (_, text_omitted) = s.call_raw("hwp_doc_text", serde_json::json!({"docId": doc_id}));
    let (_, text_zero) = s.call_raw(
        "hwp_doc_text",
        serde_json::json!({"docId": doc_id, "charOffset": 0}),
    );
    assert_eq!(
        text_omitted, text_zero,
        "charOffset 생략과 0 은 같은 봉투여야 합니다"
    );
}

#[test]
fn text_char_offset_resumes_and_preserves_page_addresses() {
    // 본문 축. 창을 이어 붙이면 전문과 같아야 하고, 다 건너뛴 쪽이라도 pages[]
    // 에서 빠지면 안 된다 — 빠지면 pageCount 가 줄어 문서가 짧아 보인다.
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    let (err, full) = s.call("hwp_doc_text", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{full}");
    let page_count = full["pageCount"].as_u64().expect("pageCount");
    let whole: String = full["pages"]
        .as_array()
        .expect("pages")
        .iter()
        .filter_map(|p| p["text"].as_str())
        .collect();
    assert!(
        whole.chars().count() > 40,
        "전제: 창 넘기기를 검사할 만큼 본문이 있어야 합니다"
    );

    // 창 크기는 계약이 아니라 **비용**의 문제다. `page` 를 생략한 호출은 매번 전 쪽을
    // 추출하므로 창이 작을수록 홉 수가 늘어 전체 훑기가 제곱으로 비싸진다 — 계약을
    // 증명할 만큼만 작게 잡는다(여러 홉 + 마지막 홉의 부분 창).
    let window = 800usize;
    let mut assembled = String::new();
    let mut offset = 0u64;
    let mut hops = 0;
    loop {
        let (err, v) = s.call(
            "hwp_doc_text",
            serde_json::json!({"docId": doc_id, "maxChars": window, "charOffset": offset}),
        );
        assert!(!err, "charOffset={offset} 에서 실패: {v}");
        assert_eq!(
            v["pageCount"].as_u64(),
            Some(page_count),
            "창을 옮겨도 쪽 주소는 보존해야 합니다: {v}"
        );
        for p in v["pages"].as_array().expect("pages") {
            assembled.push_str(p["text"].as_str().unwrap_or(""));
        }
        hops += 1;
        assert!(
            hops <= whole.chars().count() / window + 4,
            "창 넘기기가 끝나지 않습니다 (무한 루프)"
        );
        match v.get("nextOffset").and_then(|n| n.as_u64()) {
            Some(next) => {
                assert!(next > offset, "nextOffset 이 전진하지 않습니다: {v}");
                offset = next;
            }
            None => break,
        }
    }
    assert_eq!(
        assembled, whole,
        "본문 창을 이어 붙인 결과가 전문과 다릅니다"
    );
}

#[test]
fn text_char_offset_past_total_is_empty_success() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (_, full) = s.call("hwp_doc_text", serde_json::json!({"docId": doc_id}));
    let total: usize = full["pages"]
        .as_array()
        .expect("pages")
        .iter()
        .filter_map(|p| p["text"].as_str())
        .map(|t| t.chars().count())
        .sum();

    let (err, v) = s.call(
        "hwp_doc_text",
        serde_json::json!({"docId": doc_id, "charOffset": total + 100}),
    );
    assert!(!err, "총량 초과 charOffset 은 오류가 아닙니다: {v}");
    assert!(
        v.get("nextOffset").is_none(),
        "더 없는데 nextOffset 이 붙었습니다: {v}"
    );
    let left: usize = v["pages"]
        .as_array()
        .expect("pages")
        .iter()
        .filter_map(|p| p["text"].as_str())
        .map(|t| t.chars().count())
        .sum();
    assert_eq!(left, 0, "총량을 넘겼으면 남은 본문이 없어야 합니다: {v}");
    assert_eq!(
        v["pageCount"].as_u64(),
        full["pageCount"].as_u64(),
        "쪽 주소는 여기서도 보존한다: {v}"
    );
}

#[test]
fn offset_arguments_are_declared_in_tool_schema() {
    // 자기서술이 없으면 호출자는 이 인자의 존재를 알 수 없다 — 선언이 계약의 절반.
    let mut s = Server::started();
    let r = s.request("tools/list", serde_json::json!({}));
    let tools = r["result"]["tools"].as_array().expect("tools");
    let find = |name: &str| {
        tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("{name} 도구가 없습니다"))
            .clone()
    };

    let search = find("hwp_doc_search");
    assert!(
        search["inputSchema"]["properties"]["offset"].is_object(),
        "hwp_doc_search 에 offset 선언이 없습니다: {search}"
    );
    assert_eq!(
        search["inputSchema"]["properties"]["offset"]["minimum"].as_u64(),
        Some(0),
        "오프셋의 하한은 0 이다 (상한과 달리 0 이 유효값): {search}"
    );

    let text = find("hwp_doc_text");
    assert!(
        text["inputSchema"]["properties"]["charOffset"].is_object(),
        "hwp_doc_text 에 charOffset 선언이 없습니다: {text}"
    );
    assert_eq!(
        text["inputSchema"]["properties"]["charOffset"]["minimum"].as_u64(),
        Some(0),
        "{text}"
    );
}

#[test]
fn negative_and_malformed_offsets_are_rejected() {
    // 오프셋 오타를 "생략"으로 뭉개면 창이 조용히 처음으로 되돌아가 같은 구간을
    // 무한히 다시 읽는다. 거부가 유일하게 안전한 처리다(#3884 의 교훈과 같다).
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    for bad in [
        serde_json::json!(-1),
        serde_json::json!(2.5),
        serde_json::json!("3"),
    ] {
        let (err, v) = s.call(
            "hwp_doc_search",
            serde_json::json!({"docId": doc_id, "query": QUERY, "offset": bad}),
        );
        assert!(err, "잘못된 offset({bad})을 받아들였습니다: {v}");

        let (err, v) = s.call(
            "hwp_doc_text",
            serde_json::json!({"docId": doc_id, "charOffset": bad}),
        );
        assert!(err, "잘못된 charOffset({bad})을 받아들였습니다: {v}");
    }
}
