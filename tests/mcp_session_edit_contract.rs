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

/// [버그] `hwp_doc_save` 는 `output` 경로의 확장자를 무시하고 원본 포맷
/// (`source_is_hwpx`) 만으로 직렬화 형식을 정했다 — HWPX 로 연 핸들을 `.hwp`
/// 경로로 저장해도 zip(HWPX) 바이트를 그대로 써서 확장자와 실제 내용이
/// 어긋났다. CLI 의 `edit_output_format` 은 명시적 출력 확장자를 우선하는데
/// (`.hwp` 지정 시 HWP5 로 변환), MCP 세션 경로만 비동형이었다. `.hwp` 로
/// 저장하면 실제로 HWP5(CFB, `D0 CF 11 E0`) 바이트가 나와야 한다.
#[test]
fn session_save_honors_explicit_hwp_output_extension_for_hwpx_source() {
    let src = sample();
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let hwpx_src = temp_path("ext-src", "hwpx");
    let conv = run_cli(&[
        "export-hwpx",
        src.to_str().unwrap(),
        hwpx_src.to_str().unwrap(),
    ]);
    assert_eq!(conv.status.code(), Some(0), "사전 변환 실패");

    let mut s = Server::started();
    let doc_id = s.open(&hwpx_src);

    // HWPX 핸들인데 출력 경로 확장자는 .hwp — CLI 규약대로 HWP5 로 변환 저장돼야 한다.
    let out = temp_path("ext-out", "hwp");
    let (err, sv) = s.call(
        "hwp_doc_save",
        serde_json::json!({"docId": doc_id, "output": out.to_str().unwrap()}),
    );
    assert!(!err, "{sv}");
    assert_eq!(
        sv["outputFormat"], "hwp5",
        "output 확장자 .hwp 를 존중해 HWP5 로 저장해야 합니다: {sv}"
    );

    let bytes = std::fs::read(&out).expect(".hwp 출력 파일을 읽을 수 없습니다");
    assert!(
        bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]),
        ".hwp 로 저장했는데 HWP5(CFB) 바이트가 아닙니다 — 처음 4바이트: {:02X?}",
        &bytes[..bytes.len().min(4)]
    );
    assert!(
        !bytes.starts_with(b"PK"),
        ".hwp 로 저장했는데 여전히 HWPX(zip) 바이트가 그대로 나왔습니다: {sv}"
    );

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

/// 채움이 문서를 여러 쪽 늘려도 핸들의 페이지 어휘가 즉시 따라와야 한다.
/// hwp_doc_info 는 "편집 후 페이지 수 변화를 추적할 때 쓴다"고 약속하므로,
/// 세션 pageCount 는 저장본을 새로 파싱한 지상 진실과 같아야 하고, 늘어난
/// 마지막 쪽은 hwp_doc_text 로 곧바로 읽혀야 한다(범위 초과가 아니라).
#[test]
fn session_fill_repaginates_page_vocabulary() {
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

    // 한 문단짜리 누름틀에 수천 자를 채워 강제로 여러 쪽을 밀어낸다.
    let huge = "세션 편집 직후 페이지 어휘 갱신 계약 검증용 장문 텍스트. ".repeat(150);
    let (err, fill) = s.call(
        "hwp_doc_fill_fields",
        serde_json::json!({"docId": doc_id, "data": {"회사명": huge}}),
    );
    assert!(!err, "{fill}");
    assert_eq!(fill["filledCount"].as_u64(), Some(1), "{fill}");

    let (err, after) = s.call("hwp_doc_info", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{after}");
    let pages_session = after["pageCount"].as_u64().expect("pageCount");

    // 지상 진실: 저장본을 새로 파싱한 pageCount (보고를 믿지 않는다).
    let out = temp_path("repag", "hwp");
    let (err, sv) = s.call(
        "hwp_doc_save",
        serde_json::json!({"docId": doc_id, "output": out.to_str().unwrap()}),
    );
    assert!(!err, "{sv}");
    let reread = run_cli(&["info", out.to_str().unwrap(), "--json"]);
    let rv: serde_json::Value = serde_json::from_slice(&reread.stdout).expect("info --json");
    let pages_truth = rv["pageCount"].as_u64().expect("pageCount");

    assert!(
        pages_truth > pages_before,
        "전제 확인: 채움이 쪽수를 늘려야 시험이 의미 있다 \
         (before={pages_before}, truth={pages_truth})"
    );
    assert_eq!(
        pages_session, pages_truth,
        "세션 pageCount 가 편집 전 레이아웃에 머물러 있습니다 \
         (세션 {pages_session} vs 신규 파싱 {pages_truth})"
    );

    // 늘어난 마지막 쪽이 세션에서 곧바로 읽혀야 한다.
    let (err, text) = s.call(
        "hwp_doc_text",
        serde_json::json!({"docId": doc_id, "page": pages_truth - 1}),
    );
    assert!(!err, "늘어난 마지막 쪽 읽기가 범위 초과로 거부됨: {text}");

    let _ = std::fs::remove_file(&out);
}

/// 저장은 스냅숏이다. HWP3 핸들은 저장 뒤에도 같은 본문·메타를 유지하고 연속 저장은
/// 같은 바이트를 내며, 뒤이은 편집도 가능해야 한다.
#[test]
fn session_save_does_not_mutate_live_handle_hwp3_source() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample.hwp");
    if !src.exists() {
        eprintln!("HWP3 샘플 없음 — 건너뜀");
        return;
    }
    let mut s = Server::started();
    let doc_id = s.open(&src);
    let (err, before) = s.call("hwp_doc_text", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{before}");
    let (err, info_before) = s.call("hwp_doc_info", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{info_before}");

    let out = temp_path("hwp3snap", "hwp");
    let (err, sv) = s.call(
        "hwp_doc_save",
        serde_json::json!({"docId": doc_id, "output": out.to_str().unwrap()}),
    );
    assert!(!err, "{sv}");
    assert_eq!(sv["outputFormat"], "hwp5", "{sv}");

    let (err, after) = s.call("hwp_doc_text", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{after}");
    let (err, info_after) = s.call("hwp_doc_info", serde_json::json!({"docId": doc_id}));
    assert!(!err, "{info_after}");
    assert_eq!(before, after, "저장이 핸들의 본문을 바꿨습니다");
    assert_eq!(info_before, info_after, "저장이 핸들의 메타를 바꿨습니다");

    let out2 = temp_path("hwp3snap2", "hwp");
    let (err, sv2) = s.call(
        "hwp_doc_save",
        serde_json::json!({"docId": doc_id, "output": out2.to_str().unwrap()}),
    );
    assert!(!err, "{sv2}");
    assert_eq!(
        std::fs::read(&out).expect("1차 산출물"),
        std::fs::read(&out2).expect("2차 산출물"),
        "연속 저장이 서로 다른 바이트를 냈습니다 — 저장이 상태를 남겼다는 뜻입니다"
    );

    let (err, rep) = s.call(
        "hwp_doc_replace_text",
        serde_json::json!({"docId": doc_id, "find": "그림", "replace": "그림"}),
    );
    assert!(!err, "저장 후 편집이 실패했습니다: {rep}");

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&out2);
}
