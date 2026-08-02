//! [#3719 §6-1] 세션 편집 도구의 `changedPages` — 눈검증 루프를 세션에서도 닫는다.
//!
//! 무상태 편집 3종은 이미 "어느 쪽을 보라"를 봉투로 지정하지만(#3712), 세션 편집
//! 3종은 못 냈다. 그래서 `hwp_doc_render_page` 가 약속한 "편집 직후 눈검증(VLM)
//! 루프"가 세션 안에서는 **어느 쪽을 렌더할지 모른 채** 남아 있었다 — 에이전트는
//! 전수 렌더(예산 폭발)나 무검증(거짓 통과) 중 하나를 고르게 된다(#3630 F3).
//!
//! 계약의 핵심 셋:
//! ① 세션 봉투의 `changedPages` 가 **무상태 봉투와 같은 쪽**을 답한다 — 추적 근거가
//!    같은 코어(FieldLocation · 치환 전 grep 주소 · resolve_table_cell 호스트 문단)이고
//!    같은 질의(`pages_covering_paragraphs`)를 쓰므로 두 경로가 갈라질 수 없다.
//! ② 지정된 쪽은 **그 자리에서 렌더된다** — 재조판 전에 계산했다면 편집 전 레이아웃의
//!    쪽 번호가 나와 "범위 초과"로 거부되거나 엉뚱한 쪽을 렌더한다(#3704 가 조회
//!    4종에서 고친 스테일과 같은 함정).
//! ③ 변경이 없으면 빈 목록이지 `null` 이 아니다 — `null` 은 "확정 불가, 전체를 보라"라
//!    무변경 호출마다 전수 렌더를 유도하게 된다. 부분 목록은 어느 경우에도 금지.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// 누름틀을 가진 HWP5 서식 — fill/replace 축.
const FIELD_SAMPLE: &str = "samples/field-01.hwp";
/// 본문 최상위 표를 가진 HWP5 — set_cell 축.
const TABLE_SAMPLE: &str = "samples/table-001.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-sesschpages-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn cli_json(args: &[&str]) -> serde_json::Value {
    let out = run_cli(args);
    assert_eq!(out.status.code(), Some(0), "{args:?} 실패");
    serde_json::from_slice(&out.stdout).expect("CLI 봉투가 JSON 이 아닙니다")
}

/// 필드 이름을 **문서에서 읽어** 쓴다 — 하드코딩하면 샘플이 바뀔 때 시험이 조용히
/// notFound 만 확인하는 껍데기가 된다.
fn first_field_name(path: &str) -> Option<String> {
    let v = cli_json(&["fields", path, "--json"]);
    v["fields"]
        .as_array()?
        .iter()
        .find_map(|f| f["name"].as_str().map(str::to_string))
        .filter(|n| !n.is_empty())
}

/// 치환 대상도 문서에서 고른다. 하이픈으로 시작하는 토큰이 argv 에서 flag 로
/// 오해되지 않도록 영숫자·한글 토큰만 받는다.
fn first_word(path: &str) -> Option<String> {
    let v = cli_json(&["export-text", path, "--json"]);
    for page in v["pages"].as_array()? {
        for token in page["text"].as_str().unwrap_or("").split_whitespace() {
            if token.chars().count() >= 2 && token.chars().all(char::is_alphanumeric) {
                return Some(token.to_string());
            }
        }
    }
    None
}

/// `{"<필드이름>": "<값>"}` — 이름이 실행 시점에 정해지므로 맵으로 만든다.
fn field_data(name: &str, value: &str) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        name.to_string(),
        serde_json::Value::String(value.to_string()),
    );
    serde_json::Value::Object(map)
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
            // 파서 진단이 파이프를 채우면 서버가 블록되어 결함과 무관한 행으로
            // 오진하게 된다(측정 함정) — 읽을 일이 없으니 아예 버린다.
            .stderr(Stdio::null())
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
                "clientInfo": {"name": "session-changed-pages-test", "version": "0"}
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

    fn page_count(&mut self, doc_id: &str) -> u64 {
        let (err, v) = self.call("hwp_doc_info", serde_json::json!({"docId": doc_id}));
        assert!(!err, "hwp_doc_info 실패: {v}");
        v["pageCount"].as_u64().expect("pageCount")
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// `changedPages` 는 배열이거나 null 이다 — 다른 타입이면 소비자가 못 읽는다.
fn changed_pages(v: &serde_json::Value) -> Option<Vec<u64>> {
    // 키 부재와 `null` 은 serde 인덱싱에서 똑같이 Null 로 보인다 — 소비자가 "필드가
    // 없네, 이 버전은 지원 안 하나?" 로 오해하지 않도록 존재 자체를 먼저 못박는다.
    assert!(
        v.as_object()
            .is_some_and(|o| o.contains_key("changedPages")),
        "봉투에 changedPages 키가 없습니다: {v}"
    );
    let field = &v["changedPages"];
    if field.is_null() {
        return None;
    }
    Some(
        field
            .as_array()
            .unwrap_or_else(|| panic!("changedPages 는 배열|null 이어야 합니다: {v}"))
            .iter()
            .map(|p| p.as_u64().unwrap_or_else(|| panic!("쪽 번호는 정수: {v}")))
            .collect(),
    )
}

/// ① fill — 세션 봉투가 무상태 봉투와 **같은 쪽**을 답한다.
#[test]
fn session_fill_changed_pages_matches_stateless() {
    let src = sample(FIELD_SAMPLE);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let path = src.to_str().unwrap();
    let Some(name) = first_field_name(path) else {
        eprintln!("누름틀 없는 샘플 — 건너뜀");
        return;
    };
    let data = field_data(&name, "동형확인");

    // 무상태 판 — 지상 진실.
    let out = temp_path("fill", "hwp");
    let stateless = cli_json(&[
        "edit",
        "fill-fields",
        path,
        "--data",
        &data.to_string(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ]);
    let _ = std::fs::remove_file(&out);
    assert_eq!(stateless["filledCount"].as_u64(), Some(1), "{stateless}");

    // 세션 판.
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, session) = s.call(
        "hwp_doc_fill_fields",
        serde_json::json!({"docId": doc_id, "data": data}),
    );
    assert!(!err, "세션 채움 실패: {session}");
    assert_eq!(session["filledCount"].as_u64(), Some(1), "{session}");

    let sp = changed_pages(&stateless);
    let ss = changed_pages(&session);
    assert_eq!(
        ss, sp,
        "세션·무상태 changedPages 가 갈라졌습니다\n무상태={stateless}\n세션={session}"
    );
    let pages = ss.expect("채움이 있었으니 목록이 확정된다");
    assert!(
        !pages.is_empty(),
        "변경이 있었으니 비어 있지 않다: {session}"
    );
    let total = s.page_count(&doc_id);
    for p in &pages {
        assert!(*p < total, "0 기준·범위 내: {p} < {total}: {session}");
    }
}

/// ② replace — 치환 **전** 매치 주소가 근거이므로 무상태 판과 같은 쪽이 나온다.
#[test]
fn session_replace_changed_pages_matches_stateless() {
    let src = sample(FIELD_SAMPLE);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let path = src.to_str().unwrap();
    let Some(find) = first_word(path) else {
        eprintln!("치환할 토큰 없음 — 건너뜀");
        return;
    };
    let replace = format!("{find}※");

    let out = temp_path("repl", "hwp");
    let stateless = cli_json(&[
        "edit",
        "replace-text",
        path,
        "--find",
        &find,
        "--replace",
        &replace,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ]);
    let _ = std::fs::remove_file(&out);
    let replaced = stateless["replacedCount"].as_u64().unwrap_or(0);
    assert!(replaced > 0, "전제: 토큰이 실제로 치환된다: {stateless}");

    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, session) = s.call(
        "hwp_doc_replace_text",
        serde_json::json!({"docId": doc_id, "find": find, "replace": replace}),
    );
    assert!(!err, "세션 치환 실패: {session}");
    assert_eq!(
        session["replacedCount"].as_u64(),
        Some(replaced),
        "치환 계수부터 동형이어야 한다\n무상태={stateless}\n세션={session}"
    );

    assert_eq!(
        changed_pages(&session),
        changed_pages(&stateless),
        "세션·무상태 changedPages 가 갈라졌습니다\n무상태={stateless}\n세션={session}"
    );
}

/// ③ set_cell — 표 호스트 문단이 근거. 무상태 판과 같은 쪽을 답한다.
#[test]
fn session_set_cell_changed_pages_matches_stateless() {
    let src = sample(TABLE_SAMPLE);
    if !src.exists() {
        eprintln!("표 샘플 없음 — 건너뜀");
        return;
    }
    let path = src.to_str().unwrap();
    let tables = cli_json(&["export-tables", path, "--json"]);
    let Some(table) = tables["tables"]
        .as_array()
        .and_then(|t| t.iter().find(|t| t.get("containerPath").is_none()))
    else {
        eprintln!("본문 최상위 표 없음 — 건너뜀");
        return;
    };
    let (t, r, c) = (
        table["index"].as_u64().expect("index"),
        table["cells"][0]["row"].as_u64().expect("row"),
        table["cells"][0]["col"].as_u64().expect("col"),
    );

    let out = temp_path("cell", "hwp");
    let stateless = cli_json(&[
        "edit",
        "set-cell",
        path,
        "--table",
        &t.to_string(),
        "--row",
        &r.to_string(),
        "--col",
        &c.to_string(),
        "--text",
        "동형",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ]);
    let _ = std::fs::remove_file(&out);

    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, session) = s.call(
        "hwp_doc_set_cell",
        serde_json::json!({"docId": doc_id, "table": t, "row": r, "col": c, "text": "동형"}),
    );
    assert!(!err, "세션 셀 쓰기 실패: {session}");
    assert_eq!(
        session["newText"], stateless["newText"],
        "전제: 같은 칸을 같은 값으로 고쳤다"
    );
    assert_eq!(
        changed_pages(&session),
        changed_pages(&stateless),
        "세션·무상태 changedPages 가 갈라졌습니다\n무상태={stateless}\n세션={session}"
    );
}

/// ④ 지정한 쪽은 **그 자리에서 렌더된다** — 눈검증 루프가 실제로 닫히는지 본다.
///
/// 재조판 전에 계산했다면 편집 전 레이아웃의 쪽 번호가 나온다. 편집이 쪽을 늘린
/// 경우 그 번호는 `hwp_doc_render_page` 에서 "페이지 범위 초과"로 거부된다 — 즉
/// 이 시험은 "쪽 번호가 예쁘다"가 아니라 **약속된 다음 호출이 성공하는가**를 본다.
#[test]
fn session_changed_pages_are_renderable_right_away() {
    let src = sample(FIELD_SAMPLE);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let path = src.to_str().unwrap();
    let Some(name) = first_field_name(path) else {
        eprintln!("누름틀 없는 샘플 — 건너뜀");
        return;
    };

    let mut s = Server::started();
    let doc_id = s.open(&src);
    let before = s.page_count(&doc_id);
    // 쪽을 늘리는 채움 — 재조판 시점이 틀리면 여기서 어긋난다.
    let bulk = "가나다라마바사아자차".repeat(500);
    let (err, v) = s.call(
        "hwp_doc_fill_fields",
        serde_json::json!({"docId": doc_id, "data": field_data(&name, &bulk)}),
    );
    assert!(!err, "채움 실패: {v}");
    let after = s.page_count(&doc_id);
    assert!(
        after > before,
        "전제: 이 채움이 쪽을 늘린다 ({before} → {after}) — 늘지 않으면 시험이 무의미해진다"
    );

    let pages = changed_pages(&v).expect("채움이 있었으니 목록이 확정된다");
    assert!(!pages.is_empty(), "{v}");
    let out = temp_path("render", "svg");
    for p in &pages {
        assert!(*p < after, "재조판 후 쪽 수 범위 안: {p} < {after}: {v}");
        let (rerr, rv) = s.call(
            "hwp_doc_render_page",
            serde_json::json!({"docId": doc_id, "page": p, "output": out.to_str().unwrap()}),
        );
        assert!(!rerr, "changedPages 가 가리킨 {p} 쪽 렌더 실패: {rv}");
        assert!(rv["bytes"].as_u64().unwrap_or(0) > 0, "{rv}");
    }
    let _ = std::fs::remove_file(&out);
}

/// ⑤ 무변경은 빈 목록이다 — `null`("전체를 보라")로 내리면 무변경 호출마다 전수
/// 렌더를 유도한다. 부분 목록 금지와 별개의 축이다.
#[test]
fn session_changed_pages_empty_when_nothing_changed() {
    let src = sample(FIELD_SAMPLE);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);

    let (err, fill) = s.call(
        "hwp_doc_fill_fields",
        serde_json::json!({"docId": doc_id, "data": {"존재하지_않는_누름틀_이름": "x"}}),
    );
    assert!(!err, "{fill}");
    assert_eq!(fill["filledCount"].as_u64(), Some(0), "{fill}");
    assert_eq!(
        changed_pages(&fill),
        Some(Vec::new()),
        "채운 것이 없으면 빈 목록: {fill}"
    );

    let (err, repl) = s.call(
        "hwp_doc_replace_text",
        serde_json::json!({"docId": doc_id, "find": "존재하지_않는_문자열_QZX", "replace": "y"}),
    );
    assert!(!err, "{repl}");
    assert_eq!(repl["replacedCount"].as_u64(), Some(0), "{repl}");
    assert_eq!(
        changed_pages(&repl),
        Some(Vec::new()),
        "치환 0건이면 빈 목록: {repl}"
    );
}

/// ⑥ 선언과 봉투의 드리프트 가드 — 세 도구의 tools/list 설명이 changedPages 를
/// 광고하고, 실제 봉투가 그 키를 낸다. 한쪽만 바뀌면 여기서 잡힌다.
#[test]
fn session_edit_tools_declare_changed_pages() {
    let mut s = Server::started();
    let r = s.request("tools/list", serde_json::json!({}));
    let tools = r["result"]["tools"].as_array().expect("tools 배열");
    for name in [
        "hwp_doc_fill_fields",
        "hwp_doc_replace_text",
        "hwp_doc_set_cell",
    ] {
        let t = tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("{name} 이 tools/list 에 없습니다"));
        assert!(
            t["description"]
                .as_str()
                .unwrap_or_default()
                .contains("changedPages"),
            "{name} 설명이 changedPages 를 광고하지 않습니다: {t}"
        );
    }
}
