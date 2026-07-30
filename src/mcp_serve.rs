//! [#3140] `mcp-serve` — rhwp 를 MCP(Model Context Protocol) 서버로 노출한다.
//!
//! 전송은 MCP 표준 stdio(줄 단위 JSON-RPC 2.0)다. `capabilities --mcp`(#3263)가
//! 도구 **선언**을 냈다면, 본 모듈은 그 선언을 단일 출처(`crate::mcp_tool_definitions`)로
//! 공유하면서 **실행**까지 잇는다:
//!
//! - 무상태 도구(`hwp_info` 등 13종): 선언의 `cli.args` 배선을 그대로 해석해 자기 자신을
//!   서브프로세스로 실행한다 — 검증된 CLI 계약(#2707 종료 코드, stdout 순수성)을 문자
//!   그대로 재사용하므로 서버와 CLI 가 어긋날 수 없다.
//! - 세션 도구(`hwp_open`/`hwp_doc_text`/`hwp_close`): #3140 이 짚은 "상태 유지" 공백.
//!   문서를 한 번 파싱해 핸들로 잡아두고, 재파싱 없이 반복 조회한다.
//! - 세션 편집(`hwp_doc_fill_fields`/`hwp_doc_save`, #3598): 열린 핸들의 IR 에 편집을
//!   **누적**하고 save 에서 한 번만 기록한다 — 판정 어휘(filledCount/notFound/ambiguous)와
//!   형식 보존(#3383)은 무상태 `edit` 경로와 같은 코어 함수를 재사용해 동형을 보장한다.
//!
//! 의존성은 추가하지 않는다 — 프로토콜 표면(initialize/ping/tools/list/tools/call)이
//! 좁아 serde_json 만으로 충분하고, WASM 대상에는 아예 포함되지 않는다.

use std::collections::HashMap;
use std::io::{BufRead, Write};

use rhwp::wasm_api::HwpDocument;

const PROTOCOL_VERSION: &str = "2025-06-18";
/// JSON-RPC 2.0 예약 오류 코드.
const PARSE_ERROR: i64 = -32700;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// 열린 문서 핸들 하나 — 편집·저장의 형식 보존(#3383)을 위해 원본 형식을 기억한다.
struct SessionDoc {
    doc: HwpDocument,
    /// 원본이 HWPX 였는가. save 는 이 값으로 산출 형식을 정한다(HWPX→HWPX, 그 외→HWP5).
    source_is_hwpx: bool,
}

/// 열린 문서 핸들 테이블. 서버 프로세스가 사는 동안 유지된다.
struct Sessions {
    docs: HashMap<String, SessionDoc>,
    next_id: u64,
}

impl Sessions {
    fn new() -> Self {
        Sessions {
            docs: HashMap::new(),
            next_id: 1,
        }
    }
}

pub fn run() -> i32 {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let tool_defs = crate::mcp_tool_definitions();
    let mut sessions = Sessions::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let msg: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                write_msg(
                    &stdout,
                    &error_response(
                        serde_json::Value::Null,
                        PARSE_ERROR,
                        &format!("JSON 파싱 실패: {e}"),
                    ),
                );
                continue;
            }
        };

        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(serde_json::json!({}));

        // 알림(id 없음)은 응답하지 않는다.
        let Some(id) = id else {
            continue;
        };

        let response = match method {
            "initialize" => ok_response(
                id,
                serde_json::json!({
                    "protocolVersion": params.get("protocolVersion")
                        .and_then(|v| v.as_str())
                        .unwrap_or(PROTOCOL_VERSION),
                    "capabilities": { "tools": {} },
                    "serverInfo": {
                        "name": "rhwp",
                        "version": rhwp::version(),
                    }
                }),
            ),
            "ping" => ok_response(id, serde_json::json!({})),
            "tools/list" => {
                ok_response(id, serde_json::json!({ "tools": served_tools(&tool_defs) }))
            }
            "tools/call" => match handle_tool_call(&params, &tool_defs, &mut sessions) {
                Ok(result) => ok_response(id, result),
                Err(e) => error_response(id, INVALID_PARAMS, &e),
            },
            other => error_response(
                id,
                METHOD_NOT_FOUND,
                &format!("지원하지 않는 메서드: {other}"),
            ),
        };
        write_msg(&stdout, &response);
    }
    crate::EXIT_OK
}

fn write_msg(stdout: &std::io::Stdout, msg: &serde_json::Value) {
    let mut lock = stdout.lock();
    // stdout 순수성: 프로토콜 스트림에는 JSON-RPC 한 줄만 나간다.
    let _ = writeln!(lock, "{msg}");
    let _ = lock.flush();
}

fn ok_response(id: serde_json::Value, result: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error_response(id: serde_json::Value, code: i64, message: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "error": { "code": code, "message": message }
    })
}

/// tools/list 응답: 선언 도구(MCP 필수 3종만 노출) + 세션 도구 3종.
fn served_tools(tool_defs: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut tools: Vec<serde_json::Value> = tool_defs
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t["name"],
                "description": t["description"],
                "inputSchema": t["inputSchema"],
            })
        })
        .collect();
    tools.push(serde_json::json!({
        "name": "hwp_open",
        "description": "문서를 파싱해 세션 핸들(docId)을 연다. 대형 문서를 여러 번 조회할 때 재파싱을 피한다. 조회가 끝나면 hwp_close 로 닫는다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "HWP/HWPX/HML 문서 경로" }
            },
            "required": ["path"]
        }
    }));
    tools.push(serde_json::json!({
        "name": "hwp_doc_text",
        "description": "hwp_open 으로 연 핸들에서 페이지 텍스트를 재파싱 없이 읽는다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string", "description": "hwp_open 이 돌려준 핸들" },
                "page": { "type": "integer", "minimum": 0, "description": "0부터 시작하는 페이지 번호. 생략하면 전체" }
            },
            "required": ["docId"]
        }
    }));
    tools.push(serde_json::json!({
        "name": "hwp_doc_fill_fields",
        "description": "[#3598] hwp_open 으로 연 핸들의 IR 에 누름틀 값을 직접 채운다(디스크 미기록 — hwp_doc_save 가 유일한 기록 지점). 여러 번 호출하면 누적된다. 판정 필드(filledCount/notFound/ambiguous)는 hwp_fill_fields 와 동형이고, 반복 필드는 '이름[N]' 으로 지목한다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string", "description": "hwp_open 이 돌려준 핸들" },
                "data": { "type": "object", "description": "{\"필드이름\":\"값\"} 객체. 반복 필드는 \"이름[N]\"(0 기준)" }
            },
            "required": ["docId", "data"]
        }
    }));
    tools.push(serde_json::json!({
        "name": "hwp_doc_save",
        "description": "[#3598] 핸들에 누적된 편집을 형식 보존(HWPX→HWPX, 그 외→HWP5, #3383 규약)으로 저장한다. 핸들은 저장 후에도 열려 있다 — 이어서 편집·재저장할 수 있다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string", "description": "hwp_open 이 돌려준 핸들" },
                "output": { "type": "string", "description": "출력 파일 경로" }
            },
            "required": ["docId", "output"]
        }
    }));
    tools.push(serde_json::json!({
        "name": "hwp_close",
        "description": "hwp_open 으로 연 핸들을 닫아 메모리를 해제한다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string", "description": "닫을 핸들" }
            },
            "required": ["docId"]
        }
    }));
    tools
}

/// tools/call 본체. Err 는 JSON-RPC 오류(잘못된 요청 구조), Ok(isError=true) 는
/// 도구 실행 실패(MCP 규약: 실행 실패는 프로토콜 오류가 아니라 도구 결과다).
fn handle_tool_call(
    params: &serde_json::Value,
    tool_defs: &[serde_json::Value],
    sessions: &mut Sessions,
) -> Result<serde_json::Value, String> {
    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or("params.name 이 필요합니다")?;
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    match name {
        "hwp_open" => Ok(session_open(&args, sessions)),
        "hwp_doc_text" => Ok(session_doc_text(&args, sessions)),
        "hwp_doc_fill_fields" => Ok(session_fill_fields(&args, sessions)),
        "hwp_doc_save" => Ok(session_save(&args, sessions)),
        "hwp_close" => Ok(session_close(&args, sessions)),
        _ => {
            let Some(def) = tool_defs.iter().find(|t| t["name"] == name) else {
                return Ok(tool_error(format!("알 수 없는 도구: {name}")));
            };
            Ok(run_cli_tool(def, &args))
        }
    }
}

fn tool_error(message: String) -> serde_json::Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true
    })
}

fn tool_ok_text(text: String) -> serde_json::Value {
    // stdout 이 JSON 이면 structuredContent 로도 준다 — 에이전트가 재파싱을 아낀다.
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(v) => serde_json::json!({
            "content": [{ "type": "text", "text": text }],
            "structuredContent": v,
            "isError": false
        }),
        Err(_) => serde_json::json!({
            "content": [{ "type": "text", "text": text }],
            "isError": false
        }),
    }
}

// ── 세션 도구 ──────────────────────────────────────────────────────────────

fn session_open(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let Some(path) = args.get("path").and_then(|p| p.as_str()) else {
        return tool_error("path 가 필요합니다".into());
    };
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => return tool_error(format!("{path} 읽기 실패: {e}")),
    };
    let doc = match HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return tool_error(format!("{path} 파싱 실패: {e:?}")),
    };
    // [#3598] save 의 형식 보존을 위해 원본 형식을 핸들에 함께 기억한다.
    let source_is_hwpx = matches!(
        rhwp::parser::detect_format(&data),
        rhwp::parser::FileFormat::Hwpx
    );
    let page_count = doc.page_count();
    let doc_id = format!("doc-{}", sessions.next_id);
    sessions.next_id += 1;
    sessions.docs.insert(
        doc_id.clone(),
        SessionDoc {
            doc,
            source_is_hwpx,
        },
    );
    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": doc_id,
            "source": path,
            "pageCount": page_count,
        })
        .to_string(),
    )
}

fn session_doc_text(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return tool_error("docId 가 필요합니다".into());
    };
    let Some(sd) = sessions.docs.get_mut(doc_id) else {
        return tool_error(format!("열려 있지 않은 핸들: {doc_id} (hwp_open 먼저)"));
    };
    let doc = &mut sd.doc;
    let page_count = doc.page_count();
    let pages: Vec<u32> = match args.get("page").and_then(|p| p.as_u64()) {
        Some(p) => {
            let p = p as u32;
            if p >= page_count {
                return tool_error(format!("페이지 범위 초과: {p} (0~{})", page_count - 1));
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };
    let mut page_objs = Vec::with_capacity(pages.len());
    for p in pages {
        match doc.extract_page_text_native(p) {
            Ok(text) => page_objs.push(serde_json::json!({ "page": p, "text": text })),
            Err(e) => return tool_error(format!("페이지 {p} 텍스트 추출 실패: {e:?}")),
        }
    }
    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": doc_id,
            "pageCount": page_objs.len(),
            "pages": page_objs,
        })
        .to_string(),
    )
}

/// [#3598] 열린 핸들의 IR 에 누름틀 값을 채운다 — 디스크 미기록, save 까지 누적.
///
/// 판정 로직(이름 개수 → notFound/ambiguous → `set_field_value_by_name_at`)은 무상태
/// `edit fill-fields`(#3329/#3476)와 같은 코어 경로를 재사용한다 — 두 경로의 판정
/// 어휘가 어긋나면 소비자가 같은 코드로 못 읽는다.
fn session_fill_fields(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return tool_error("docId 가 필요합니다".into());
    };
    let Some(data) = args.get("data").and_then(|d| d.as_object()) else {
        return tool_error("data 는 {\"필드이름\":\"값\"} 객체여야 합니다".into());
    };
    let Some(sd) = sessions.docs.get_mut(doc_id) else {
        return tool_error(format!("열려 있지 않은 핸들: {doc_id} (hwp_open 먼저)"));
    };
    let doc = &mut sd.doc;

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for fi in doc.collect_all_fields().iter() {
        if let Some(n) = fi.field.field_name() {
            *name_counts.entry(n.to_string()).or_insert(0) += 1;
        }
    }

    let mut filled: Vec<serde_json::Value> = Vec::new();
    let mut not_found: Vec<String> = Vec::new();
    let mut ambiguous: Vec<serde_json::Value> = Vec::new();

    // 1차: 판정만 먼저 — 핸들은 살아 있는 상태라, 중간 실패로 절반만 채워진 IR 을
    // 남기지 않도록 적용 전에 전 키를 검증한다.
    let mut apply: Vec<(String, usize, String)> = Vec::new();
    for (key, value) in data {
        let value_str = match value {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let (name, occurrence) = crate::parse_field_key(key);
        let total = name_counts.get(name).copied().unwrap_or(0);
        if total == 0 || occurrence >= total {
            not_found.push(key.clone());
            continue;
        }
        if occurrence == 0 && total > 1 && !key.contains('[') {
            ambiguous.push(serde_json::json!({
                "name": name,
                "matched": 1,
                "total": total,
            }));
        }
        apply.push((name.to_string(), occurrence, value_str));
    }

    // 2차: 적용. 검증을 통과한 키만 남았으므로 실패는 코어 결함 신호다.
    for (name, occurrence, value_str) in &apply {
        if let Err(e) = doc.set_field_value_by_name_at(name, *occurrence, value_str) {
            return tool_error(format!(
                "필드 '{name}' 설정 실패: {e} — 핸들이 부분 편집 상태일 수 있으니 \
                 hwp_close 후 다시 여는 것을 권장합니다"
            ));
        }
        filled.push(serde_json::json!({
            "name": name, "occurrence": occurrence, "value": value_str,
        }));
    }

    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": doc_id,
            "filledCount": filled.len(),
            "filled": filled,
            "notFound": not_found,
            "ambiguous": ambiguous,
        })
        .to_string(),
    )
}

/// [#3598] 핸들에 누적된 편집을 형식 보존(#3383)으로 저장한다. 핸들은 계속 열려 있다.
fn session_save(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return tool_error("docId 가 필요합니다".into());
    };
    let Some(output) = args.get("output").and_then(|o| o.as_str()) else {
        return tool_error("output 이 필요합니다".into());
    };
    let Some(sd) = sessions.docs.get_mut(doc_id) else {
        return tool_error(format!("열려 있지 않은 핸들: {doc_id} (hwp_open 먼저)"));
    };

    let format = if sd.source_is_hwpx {
        crate::EditOutputFormat::Hwpx
    } else {
        crate::EditOutputFormat::Hwp
    };
    let bytes = match crate::edit_serialize(&mut sd.doc, format) {
        Ok(b) => b,
        Err(e) => return tool_error(format!("직렬화 실패: {e}")),
    };
    if let Err(e) = std::fs::write(output, &bytes) {
        return tool_error(format!("{output} 쓰기 실패: {e}"));
    }
    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": doc_id,
            "output": output,
            "outputFormat": format.label(),
            "bytes": bytes.len(),
        })
        .to_string(),
    )
}

fn session_close(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return tool_error("docId 가 필요합니다".into());
    };
    if sessions.docs.remove(doc_id).is_none() {
        return tool_error(format!("열려 있지 않은 핸들: {doc_id}"));
    }
    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": doc_id,
            "closed": true,
        })
        .to_string(),
    )
}

// ── 무상태 도구: 선언된 cli.args 배선을 그대로 실행 ─────────────────────────

/// `cli.args` 템플릿의 `{키}` 자리표시자를 arguments 값으로 치환한다.
/// 값이 문자열이면 그대로, 객체/숫자/불리언이면 JSON 직렬화 문자열로 넣는다
/// (`--data` 가 JSON 문자열을 받는 것과 정합).
fn substitute_args(
    template: &[serde_json::Value],
    args: &serde_json::Value,
) -> Result<Vec<String>, String> {
    let mut out = Vec::with_capacity(template.len());
    for t in template {
        let s = t.as_str().unwrap_or_default();
        if s.starts_with('{') && s.ends_with('}') && s.len() > 2 {
            let key = &s[1..s.len() - 1];
            let Some(v) = args.get(key) else {
                return Err(format!("필수 인자 누락: {key}"));
            };
            out.push(match v {
                serde_json::Value::String(sv) => sv.clone(),
                other => other.to_string(),
            });
        } else {
            out.push(s.to_string());
        }
    }
    Ok(out)
}

fn run_cli_tool(def: &serde_json::Value, args: &serde_json::Value) -> serde_json::Value {
    let template: Vec<serde_json::Value> =
        def["cli"]["args"].as_array().cloned().unwrap_or_default();
    let cli_args = match substitute_args(&template, args) {
        Ok(a) => a,
        Err(e) => return tool_error(e),
    };

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return tool_error(format!("실행 파일 경로 조회 실패: {e}")),
    };
    let mut cmd = std::process::Command::new(exe);
    cmd.args(&cli_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    // stdin 도구(hwp_batch 계열): paths 배열을 한 줄에 하나씩 흘려 넣는다.
    let stdin_paths: Option<String> = args.get("paths").and_then(|p| p.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    });
    if stdin_paths.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return tool_error(format!("CLI 실행 실패: {e}")),
    };
    if let (Some(paths), Some(mut si)) = (stdin_paths, child.stdin.take()) {
        let _ = si.write_all(paths.as_bytes());
        let _ = si.write_all(b"\n");
        // drop 으로 stdin 닫힘 — batch 가 EOF 를 본다.
    }
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return tool_error(format!("CLI 종료 대기 실패: {e}")),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let code = output.status.code().unwrap_or(-1);
    // #2707 계약: 0=성공. 3(ir-diff 차이)·1(batch 부분 실패)도 stdout 에 유효한 JSON
    // 결과가 있으므로 도구 결과로 그대로 전달한다. stdout 이 비어 있을 때만 실패다.
    if stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return tool_error(format!("종료 코드 {code}: {stderr}"));
    }
    tool_ok_text(stdout)
}
