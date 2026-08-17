//! Planner 어댑터 — 설계 문서 §5의 "BYO-두뇌".
//!
//! OpenAI 호환 `/v1/chat/completions` 규격 하나로 로컬 서버·사내 게이트웨이·
//! 클라우드를 전부 덮는다. LLM 은 **계획 수립만** 하고, 실행·검증·회계는
//! 전부 결정론 자산(rhwp CLI + 저널)이 한다. 이 모듈은 순수 함수
//! (요청 조립·응답 파싱·오류 분류·CLI 인자 치환)만 담고, HTTP 왕복은
//! `commands.rs` 쪽 tauri 커맨드가 담당한다.

use serde::Serialize;
use serde_json::{json, Value};

/// 로컬 LLM 서버 자동 탐지 포트 후보.
/// 제품명을 코드에 박지 않는다 — "태그 목록 API(`/api/tags`)" 또는
/// "OpenAI 호환(`/v1/models`)" 두 가지 목록 규격만 본다.
pub const LOCAL_PORT_CANDIDATES: &[u16] = &[11434, 1234];

/// base URL 정규화 — 끝 `/` 와 끝 `/v1` 을 걷어내 origin 형태로 만든다.
pub fn normalize_base_url(raw: &str) -> String {
    let mut s = raw.trim().trim_end_matches('/').to_string();
    if s.to_ascii_lowercase().ends_with("/v1") {
        s.truncate(s.len() - 3);
        s = s.trim_end_matches('/').to_string();
    }
    s
}

pub fn chat_url(base: &str) -> String {
    format!("{}/v1/chat/completions", normalize_base_url(base))
}
pub fn models_url(base: &str) -> String {
    format!("{}/v1/models", normalize_base_url(base))
}
pub fn tags_url(base: &str) -> String {
    format!("{}/api/tags", normalize_base_url(base))
}

/// 로컬 주소인가 — 프라이버시 모드 배지(로컬 모델/외부 API) 판정.
pub fn is_local_url(base: &str) -> bool {
    let b = normalize_base_url(base).to_ascii_lowercase();
    let host = b
        .strip_prefix("http://")
        .or_else(|| b.strip_prefix("https://"))
        .unwrap_or(&b);
    let host = host.split([':', '/']).next().unwrap_or("");
    host == "127.0.0.1" || host == "localhost" || host == "::1" || host == "[::1]"
}

/// chat/completions 요청 본문 조립. temperature 0 — 계획은 결정적일수록 좋다.
pub fn build_chat_request(model: &str, messages: &Value, tools: Option<&Value>) -> Value {
    let mut req = json!({
        "model": model,
        "messages": messages,
        "temperature": 0,
        "stream": false,
    });
    if let Some(t) = tools {
        if t.as_array().map(|a| !a.is_empty()).unwrap_or(false) {
            req["tools"] = t.clone();
            req["tool_choice"] = json!("auto");
        }
    }
    req
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallReq {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// 파싱된 Planner 응답 — kind 는 "toolCalls" | "text".
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedChat {
    pub kind: String,
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCallReq>,
}

/// OpenAI 호환 응답에서 다음 행동을 뽑는다.
pub fn parse_chat_response(v: &Value) -> Result<ParsedChat, String> {
    let msg = v
        .pointer("/choices/0/message")
        .ok_or_else(|| format!("응답에 choices[0].message 가 없습니다: {}", snippet(v)))?;
    let mut tool_calls = Vec::new();
    if let Some(calls) = msg.get("tool_calls").and_then(|t| t.as_array()) {
        for (i, c) in calls.iter().enumerate() {
            let name = c
                .pointer("/function/name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| format!("tool_calls[{i}] 에 function.name 이 없습니다"))?;
            let raw_args = c
                .pointer("/function/arguments")
                .and_then(|a| a.as_str())
                .unwrap_or("{}");
            let arguments: Value =
                serde_json::from_str(raw_args).unwrap_or_else(|_| json!({ "_raw": raw_args }));
            let id = c
                .get("id")
                .and_then(|x| x.as_str())
                .map(String::from)
                .unwrap_or_else(|| format!("call_{i}"));
            tool_calls.push(ToolCallReq {
                id,
                name: name.to_string(),
                arguments,
            });
        }
    }
    let content = msg
        .get("content")
        .and_then(|c| c.as_str())
        .map(String::from)
        .filter(|s| !s.trim().is_empty());
    let kind = if tool_calls.is_empty() {
        "text"
    } else {
        "toolCalls"
    };
    Ok(ParsedChat {
        kind: kind.into(),
        content,
        tool_calls,
    })
}

fn snippet(v: &Value) -> String {
    let s = v.to_string();
    s.chars().take(200).collect()
}

/// HTTP 오류를 사람이 조치할 수 있는 문장으로 분류한다.
pub fn classify_error(status: Option<u16>, detail: &str) -> String {
    match status {
        Some(401) | Some(403) => "인증 실패 — API 키를 확인하세요".into(),
        Some(404) => "경로 또는 모델을 찾을 수 없음 — base URL·모델명을 확인하세요".into(),
        Some(429) => "요청 한도 초과 — 잠시 후 다시 시도하세요".into(),
        Some(400) if detail.to_ascii_lowercase().contains("model") => {
            "모델 없음 — 서버가 해당 모델명을 모릅니다".into()
        }
        Some(s) if s >= 500 => format!("서버 오류 ({s}) — 모델 서버 로그를 확인하세요"),
        Some(s) => format!("HTTP {s}: {}", detail.chars().take(160).collect::<String>()),
        None => {
            let d = detail.to_ascii_lowercase();
            if d.contains("timed out") || d.contains("timeout") {
                "응답 시간 초과 — 서버가 느리거나 모델이 로드 중입니다".into()
            } else if d.contains("connection refused")
                || d.contains("connect")
                || d.contains("connection")
            {
                "연결 거부 — 해당 주소에 서버가 떠 있지 않습니다".into()
            } else {
                format!(
                    "연결 실패: {}",
                    detail.chars().take(160).collect::<String>()
                )
            }
        }
    }
}

/// 모델 목록 응답 파싱 — OpenAI 호환(`data[].id`)과 태그 목록(`models[].name`)
/// 두 규격을 모두 받는다.
pub fn parse_model_list(v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
        for m in arr {
            if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
                out.push(id.to_string());
            }
        }
    }
    if let Some(arr) = v.get("models").and_then(|d| d.as_array()) {
        for m in arr {
            if let Some(id) = m.get("name").and_then(|i| i.as_str()) {
                out.push(id.to_string());
            }
        }
    }
    out
}

/// MCP 도구 정의(`capabilities --mcp`) → OpenAI tools 배열.
/// 도구 정의의 단일 출처 계약 — 스키마를 여기서 재작성하지 않고 그대로 싣는다.
/// `password` 처럼 writeOnly 로 표시된 민감 인자는 노출하지 않는다(데스크 M1 은
/// 암호 문서를 Planner 경로에서 다루지 않는다).
pub fn mcp_tools_to_openai(mcp: &Value, allowlist: Option<&[String]>) -> Value {
    let mut out = Vec::new();
    let Some(tools) = mcp.get("tools").and_then(|t| t.as_array()) else {
        return json!([]);
    };
    for t in tools {
        let Some(name) = t.get("name").and_then(|n| n.as_str()) else {
            continue;
        };
        if let Some(allow) = allowlist {
            if !allow.iter().any(|a| a == name) {
                continue;
            }
        }
        let mut schema = t
            .get("inputSchema")
            .cloned()
            .unwrap_or(json!({"type":"object"}));
        if let Some(props) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
            let sensitive: Vec<String> = props
                .iter()
                .filter(|(_, v)| v.get("writeOnly").and_then(|w| w.as_bool()) == Some(true))
                .map(|(k, _)| k.clone())
                .collect();
            for k in sensitive {
                props.remove(&k);
            }
        }
        out.push(json!({
            "type": "function",
            "function": {
                "name": name,
                "description": t.get("description").and_then(|d| d.as_str()).unwrap_or(""),
                "parameters": schema,
            }
        }));
    }
    json!(out)
}

/// 도구 호출 → CLI argv. `cli.args` 의 `{이름}` 템플릿과 `cli.optionalArgs` 의
/// `when` 조건을 치환한다 — mcp-serve 와 같은 규칙, 데스크가 지어내는 매핑은 없다.
pub fn substitute_cli_args(cli: &Value, args: &Value) -> Result<Vec<String>, String> {
    fn render(tok: &str, args: &Value) -> Result<Option<String>, String> {
        if let Some(name) = tok.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            match args.get(name) {
                Some(Value::String(s)) => Ok(Some(s.clone())),
                Some(Value::Number(n)) => Ok(Some(n.to_string())),
                Some(Value::Bool(b)) => Ok(Some(b.to_string())),
                Some(Value::Null) | None => Err(format!("필수 인자 누락: {name}")),
                Some(other) => Ok(Some(other.to_string())),
            }
        } else {
            Ok(Some(tok.to_string()))
        }
    }
    let mut out = Vec::new();
    let base = cli
        .get("args")
        .and_then(|a| a.as_array())
        .ok_or("cli.args 템플릿이 없습니다")?;
    for tok in base {
        let tok = tok.as_str().ok_or("cli.args 토큰이 문자열이 아닙니다")?;
        if let Some(v) = render(tok, args)? {
            out.push(v);
        }
    }
    if let Some(opts) = cli.get("optionalArgs").and_then(|a| a.as_array()) {
        for opt in opts {
            let Some(when) = opt.get("when").and_then(|w| w.as_str()) else {
                continue;
            };
            let present = match args.get(when) {
                None | Some(Value::Null) | Some(Value::Bool(false)) => false,
                Some(Value::String(s)) => !s.is_empty(),
                Some(_) => true,
            };
            if !present {
                continue;
            }
            if let Some(toks) = opt.get("args").and_then(|a| a.as_array()) {
                for tok in toks {
                    let tok = tok.as_str().unwrap_or_default();
                    if let Some(v) = render(tok, args)? {
                        out.push(v);
                    }
                }
            }
        }
    }
    Ok(out)
}

/// 도구가 문서를 바꾸는가 — MCP annotations 의 readOnlyHint 를 신뢰한다.
/// 표기가 없으면 보수적으로 "바꾼다" 취급(승인 카드 경유).
/// UI(agent.js)가 같은 규칙을 쓴다 — 여기 두는 이유는 규칙의 정본과 테스트.
#[allow(dead_code)]
pub fn is_read_only_tool(tool: &Value) -> bool {
    tool.pointer("/annotations/readOnlyHint")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_정규화() {
        assert_eq!(
            normalize_base_url("http://127.0.0.1:11434/"),
            "http://127.0.0.1:11434"
        );
        assert_eq!(
            normalize_base_url("https://api.example.com/v1"),
            "https://api.example.com"
        );
        assert_eq!(
            chat_url("http://localhost:1234/v1/"),
            "http://localhost:1234/v1/chat/completions"
        );
        assert!(is_local_url("http://127.0.0.1:11434"));
        assert!(is_local_url("http://localhost:1234/v1"));
        assert!(!is_local_url("https://api.example.com"));
    }

    #[test]
    fn 요청_조립은_도구가_있을_때만_tools_를_싣는다() {
        let msgs = json!([{"role":"user","content":"안녕"}]);
        let req = build_chat_request("m1", &msgs, None);
        assert_eq!(req["model"], "m1");
        assert!(req.get("tools").is_none());
        assert_eq!(req["temperature"], 0);

        let tools = json!([{"type":"function","function":{"name":"hwp_info"}}]);
        let req2 = build_chat_request("m1", &msgs, Some(&tools));
        assert_eq!(req2["tool_choice"], "auto");
        assert_eq!(req2["tools"].as_array().unwrap().len(), 1);

        let empty = json!([]);
        let req3 = build_chat_request("m1", &msgs, Some(&empty));
        assert!(req3.get("tools").is_none());
    }

    #[test]
    fn 응답_파싱_텍스트와_도구호출() {
        let text = json!({"choices":[{"message":{"role":"assistant","content":"요약입니다"}}]});
        let p = parse_chat_response(&text).unwrap();
        assert_eq!(p.kind, "text");
        assert_eq!(p.content.as_deref(), Some("요약입니다"));

        let tc = json!({"choices":[{"message":{
            "tool_calls":[{"id":"c1","function":{"name":"hwp_info","arguments":"{\"path\":\"a.hwp\"}"}}]
        }}]});
        let p2 = parse_chat_response(&tc).unwrap();
        assert_eq!(p2.kind, "toolCalls");
        assert_eq!(p2.tool_calls[0].name, "hwp_info");
        assert_eq!(p2.tool_calls[0].arguments["path"], "a.hwp");

        // 깨진 arguments 는 _raw 로 보존 — 실행 전에 사람이 볼 수 있어야 한다.
        let bad = json!({"choices":[{"message":{
            "tool_calls":[{"id":"c1","function":{"name":"hwp_info","arguments":"{broken"}}]
        }}]});
        let p3 = parse_chat_response(&bad).unwrap();
        assert_eq!(p3.tool_calls[0].arguments["_raw"], "{broken");

        assert!(parse_chat_response(&json!({"error":"x"})).is_err());
    }

    #[test]
    fn 오류_분류() {
        assert!(classify_error(Some(401), "").contains("인증"));
        assert!(classify_error(Some(404), "").contains("모델"));
        assert!(classify_error(Some(400), "model 'x' not found").contains("모델 없음"));
        assert!(classify_error(Some(503), "").contains("서버 오류"));
        assert!(classify_error(None, "Connection refused (os error 10061)").contains("연결 거부"));
        assert!(classify_error(None, "timed out reading response").contains("시간 초과"));
    }

    #[test]
    fn 모델_목록_두_규격() {
        let openai = json!({"data":[{"id":"m-a"},{"id":"m-b"}]});
        assert_eq!(parse_model_list(&openai), vec!["m-a", "m-b"]);
        let tags = json!({"models":[{"name":"n-1"},{"name":"n-2"}]});
        assert_eq!(parse_model_list(&tags), vec!["n-1", "n-2"]);
        assert!(parse_model_list(&json!({})).is_empty());
    }

    #[test]
    fn mcp_도구_변환은_write_only_인자를_숨긴다() {
        let mcp = json!({"tools":[{
            "name":"hwp_info",
            "description":"메타 조회",
            "inputSchema":{"type":"object","properties":{
                "path":{"type":"string"},
                "password":{"type":"string","writeOnly":true}
            },"required":["path"]}
        }]});
        let tools = mcp_tools_to_openai(&mcp, None);
        let props = &tools[0]["function"]["parameters"]["properties"];
        assert!(props.get("path").is_some());
        assert!(props.get("password").is_none());

        let filtered = mcp_tools_to_openai(&mcp, Some(&["없는것".to_string()]));
        assert!(filtered.as_array().unwrap().is_empty());
    }

    #[test]
    fn cli_인자_치환_필수와_옵션() {
        // hwp_replace_text 실측 템플릿 축약본
        let cli = json!({
            "args":["edit","replace-text","{path}","--find","{find}","--replace","{replace}","--json"],
            "optionalArgs":[
                {"args":["-o","{output}"],"when":"output"},
                {"args":["--dry-run"],"when":"dryRun"}
            ]
        });
        let args = json!({"path":"a.hwp","find":"갑","replace":"을","dryRun":true});
        let argv = substitute_cli_args(&cli, &args).unwrap();
        assert_eq!(
            argv,
            vec![
                "edit",
                "replace-text",
                "a.hwp",
                "--find",
                "갑",
                "--replace",
                "을",
                "--json",
                "--dry-run"
            ]
        );

        // dryRun:false 는 포함하지 않는다
        let args2 =
            json!({"path":"a.hwp","find":"갑","replace":"을","dryRun":false,"output":"out.hwp"});
        let argv2 = substitute_cli_args(&cli, &args2).unwrap();
        assert!(argv2.contains(&"-o".to_string()));
        assert!(!argv2.contains(&"--dry-run".to_string()));

        // 필수 누락은 오류
        let missing = substitute_cli_args(&cli, &json!({"path":"a.hwp","find":"갑"}));
        assert!(missing.is_err());
    }

    #[test]
    fn 읽기전용_판정은_annotations_없으면_보수적() {
        let ro = json!({"annotations":{"readOnlyHint":true}});
        let rw = json!({"annotations":{"readOnlyHint":false}});
        let unknown = json!({});
        assert!(is_read_only_tool(&ro));
        assert!(!is_read_only_tool(&rw));
        assert!(!is_read_only_tool(&unknown));
    }
}
