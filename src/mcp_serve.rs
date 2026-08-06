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
/// 이 서버가 실제로 말할 수 있는 프로토콜 개정판 목록.
///
/// 요청받은 값을 그대로 되비추면 서버는 `"9999-99-99"` 같은 존재하지 않는 개정판까지
/// "지원한다"고 답하게 된다 — 그래 놓고 몸통은 `structuredContent`(2025-06-18 신설)처럼
/// 특정 개정판 전용 표면을 내보내므로, 클라이언트는 **끊어야 할 신호를 영영 못 받은 채**
/// 못 읽는 응답을 받는다. 지원 목록을 명시적으로 두는 이유가 이것이다.
/// 새 개정판을 실제로 구현하면 이 배열에 한 줄 더하는 것이 유일한 변경점이다.
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &[PROTOCOL_VERSION];
/// JSON-RPC 2.0 예약 오류 코드.
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
/// [#3627] MCP resources 규약이 못박은 코드 — "Resource not found: -32002".
const RESOURCE_NOT_FOUND: i64 = -32002;

/// 열린 문서 핸들 하나 — 편집·저장의 형식 보존(#3383)을 위해 원본 형식을 기억한다.
struct SessionDoc {
    doc: HwpDocument,
    /// 원본이 HWPX 였는가. save 는 이 값으로 산출 형식을 정한다(HWPX→HWPX, 그 외→HWP5).
    source_is_hwpx: bool,
    /// [#3609] hwp_doc_info 봉투용 — open 시점의 원본 크기·감지 형식.
    size_bytes: usize,
    detected_format: rhwp::parser::FileFormat,
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

pub fn run(args: &[String]) -> i32 {
    // [#3629] 직무 프로필: tools/list 자체를 역할 세트로 필터 — 호스트 설정 한 줄로
    // '행정서식 전용 서버'를 등록한다. 단일 출처는 agent_profiles::PROFILES.
    let mut profile: Option<&'static crate::agent_profiles::AgentProfile> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--profile" => {
                i += 1;
                let Some(name) = args.get(i) else {
                    eprintln!("오류: --profile 뒤에 역할 이름이 필요합니다.");
                    eprintln!("사용 가능: {}", crate::agent_profiles::names().join(", "));
                    return crate::EXIT_USAGE;
                };
                match crate::agent_profiles::find(name) {
                    Some(p) => profile = Some(p),
                    None => {
                        eprintln!("오류: 알 수 없는 프로필 '{name}'");
                        eprintln!("사용 가능: {}", crate::agent_profiles::names().join(", "));
                        return crate::EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return crate::EXIT_USAGE;
            }
        }
        i += 1;
    }
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut tool_defs = crate::mcp_tool_definitions();
    if let Some(p) = profile {
        tool_defs.retain(|t| {
            t["name"]
                .as_str()
                .map(|n| crate::agent_profiles::allows_tool(p, n))
                .unwrap_or(false)
        });
    }
    // 세션 도구도 이름 단위로 건다 — 프로필이 없으면 전 도구, 있으면 그 프로필의
    // session_tools 목록. 종전에는 bool 하나라 조회 전용 직무가 세션을 쓰려면
    // 편집·저장까지 통째로 열렸다.
    let session_allows = move |name: &str| match profile {
        None => true,
        Some(p) => crate::agent_profiles::allows_session_tool(p, name),
    };
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

        // [JSON-RPC 2.0 §5] 파싱은 됐지만 Request 객체가 **아닌** 프레임 — 배열(배치)·
        // 문자열·숫자·불리언·null. 예전에는 이 프레임에서 `msg.get("id")` 가 None 을
        // 돌려줘 알림과 구분되지 않았고, 그래서 한 바이트도 쓰지 않고 다음 줄로 넘어갔다.
        // 스트림은 멀쩡히 살아 있는데 응답 하나만 통째로 증발하므로, 클라이언트는 그 id 를
        // 영원히 기다린다. 프레임에서 id 를 알아낼 방법이 없으니 규약대로 id=null 로
        // -32600 을 돌려준다.
        if !msg.is_object() {
            let reason = if msg.is_array() {
                // MCP 2025-06-18 은 JSON-RPC 배치를 명시적으로 제거했다(changelog
                // "Remove support for JSON-RPC batching"). "배열이라 못 읽었다"가 아니라
                // "이 개정판에 배치가 없다"라고 짚어줘야 호스트가 요청을 한 줄에 하나씩
                // 푸는 쪽으로 고칠 수 있다 — 사유가 곧 수정 지시가 되게 한다.
                "JSON-RPC 배치(배열)는 MCP 2025-06-18 에서 제거되었습니다 — \
                 요청을 한 줄에 하나씩 보내세요"
            } else {
                "요청은 JSON 객체여야 합니다"
            };
            write_msg(
                &stdout,
                &error_response(serde_json::Value::Null, INVALID_REQUEST, reason),
            );
            continue;
        }

        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str());
        let params = msg.get("params").cloned().unwrap_or(serde_json::json!({}));

        // 알림(id 없음)은 응답하지 않는다.
        let Some(id) = id else {
            continue;
        };

        // [JSON-RPC 2.0 §4] method 는 문자열이어야 한다. 없거나 문자열이 아니면 "그런
        // 메서드가 없다"(-32601)가 아니라 "요청 구조가 틀렸다"(-32600)다. 예전에는
        // `unwrap_or("")` 로 빈 이름을 만들어 -32601 로 흘려보냈고, 문구까지
        // "지원하지 않는 메서드: " 처럼 이름이 빈 채로 나가 호출자가 원인을 못 짚었다.
        let Some(method) = method else {
            write_msg(
                &stdout,
                &error_response(id, INVALID_REQUEST, "method 는 문자열이어야 합니다"),
            );
            continue;
        };

        let response = match method {
            "initialize" => ok_response(
                id,
                serde_json::json!({
                    "protocolVersion": negotiate_protocol_version(&params),
                    // [#3627] subscribe/listChanged 는 아직 없다 — 스펙상 빈 객체가
                    // "두 기능 모두 미지원" 의 정식 선언이다(생략이 아니라).
                    "capabilities": { "tools": {}, "resources": {} },
                    "serverInfo": {
                        "name": "rhwp",
                        "version": rhwp::version(),
                    }
                }),
            ),
            "ping" => ok_response(id, serde_json::json!({})),
            "tools/list" => ok_response(
                id,
                serde_json::json!({
                    "tools": served_tools(&tool_defs, &session_allows)
                }),
            ),
            "tools/call" => {
                match handle_tool_call(&params, &tool_defs, &session_allows, &mut sessions) {
                    Ok(result) => ok_response(id, result),
                    Err(e) => error_response(id, INVALID_PARAMS, &e),
                }
            }
            "resources/list" => {
                ok_response(id, serde_json::json!({ "resources": served_resources() }))
            }
            "resources/read" => match read_resource(&params, profile) {
                Ok(result) => ok_response(id, result),
                Err((code, message, uri)) => {
                    resource_error_response(id, code, &message, uri.as_deref())
                }
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

/// [MCP 2025-06-18 lifecycle §Version Negotiation] 클라이언트가 요청한 개정판을
/// 지원하면 **같은 값**으로, 아니면 서버가 지원하는 다른 개정판으로 응답한다(둘 다 MUST).
///
/// 후자가 핵심이다: 클라이언트는 서버가 제시한 개정판을 자기가 못 하면 연결을 끊게
/// 되어 있는데, 요청값을 되비추면 그 검사가 **항상 통과**해 버려 끊을 기회 자체가
/// 사라진다. 버전이 없거나 문자열이 아닌 경우도 "요청한 개정판이 목록에 없다"와 같은
/// 갈래로 접어 서버 기준판을 제시한다.
fn negotiate_protocol_version(params: &serde_json::Value) -> &'static str {
    let Some(requested) = params.get("protocolVersion").and_then(|v| v.as_str()) else {
        return PROTOCOL_VERSION;
    };
    SUPPORTED_PROTOCOL_VERSIONS
        .iter()
        .find(|supported| **supported == requested)
        .copied()
        .unwrap_or(PROTOCOL_VERSION)
}

// ── [#3627] resources 표면 ─────────────────────────────────────────────────

/// 서버가 내는 문서 리소스 하나. 필드 이름은 MCP `resources/list` 의 Resource
/// 객체(uri/name/title/description/mimeType)와 1:1 로 대응한다.
struct DocResource {
    uri: &'static str,
    name: &'static str,
    title: &'static str,
    description: &'static str,
    mime_type: &'static str,
    text: &'static str,
}

/// 프로필과 무관하게 항상 노출되는 자기서술 매니페스트의 URI.
const CAPABILITIES_URI: &str = "rhwp://capabilities/mcp";

/// [#3627] 저장소의 canonical 문서를 `include_str!` 로 **컴파일 시점에** 안는다.
///
/// rhwp 는 단일 실행 파일로 배포된다 — 저장소 밖에 설치된 exe 옆에는 `mydocs/` 가
/// 없으므로 런타임 디스크 읽기는 정작 리소스가 필요한 설치 환경에서 그대로 실패한다
/// (개발 트리에서만 되는 리소스는 계약이 아니다). 원본 파일을 그대로 가리키므로
/// 복제본은 생기지 않고(문서를 고치면 다음 빌드가 따라온다), 템플릿 XML 을
/// `include_str!` 로 안는 `serializer::hwpx::static_assets` 선례와 같은 방식이다.
///
/// URI 는 커스텀 `rhwp://` 스킴이다. 본문이 바이너리 안에 있으므로 `file://` 은
/// 설치본에 존재하지 않는 경로를 광고하게 되고, `https://` 는 스펙상 클라이언트가
/// 직접 가져올 수 있을 때만 쓴다.
const DOC_RESOURCES: &[DocResource] = &[
    DocResource {
        uri: "rhwp://docs/llms.txt",
        name: "llms.txt",
        title: "rhwp 문서 지도 (llms.txt)",
        description: "에이전트 진입점 — 계약·실무·문제 해결 문서로 가는 링크 목록.",
        mime_type: "text/plain",
        text: include_str!("../llms.txt"),
    },
    DocResource {
        uri: "rhwp://docs/agent_knowledge_map.md",
        name: "agent_knowledge_map.md",
        title: "에이전트 지식 지도",
        description: "작업별 명령 결정 표·봉투 필드 사전·주소 어휘. 첫 문서로 읽는다.",
        mime_type: "text/markdown",
        text: include_str!("../mydocs/manual/agent_knowledge_map.md"),
    },
    DocResource {
        uri: "rhwp://docs/agent_troubleshooting_guide.md",
        name: "agent_troubleshooting_guide.md",
        title: "에이전트 실패 사전",
        description: "오류 문자열 그대로 검색되는 증상별 원인·처방.",
        mime_type: "text/markdown",
        text: include_str!("../mydocs/manual/agent_troubleshooting_guide.md"),
    },
    // [#3627 잔여 / R7·R34] 레시피 6편 — 목표에서 시작하는 완주 서사. 지식 지도가
    // "무엇을 부르나"라면 레시피는 "어떤 순서로 목표까지 가나"다. MCP 클라이언트가
    // 프로토콜 표준 경로로 읽을 수 있어야 CLI·저장소 없이도 서사가 닿는다.
    DocResource {
        uri: "rhwp://recipes/01_fill_form_and_submit.md",
        name: "recipe-01-fill-form",
        title: "레시피 1 — 서식 문서를 채워서 제출용으로 만들기",
        description: "필드 조회→채움→검증→산출 완주 서사 (실측 출력 인용).",
        mime_type: "text/markdown",
        text: include_str!("../mydocs/manual/recipes/01_fill_form_and_submit.md"),
    },
    DocResource {
        uri: "rhwp://recipes/02_table_csv_roundtrip.md",
        name: "recipe-02-table-csv",
        title: "레시피 2 — 표 데이터를 CSV 로 뽑아 고치고 되돌리기",
        description: "export-tables→스프레드시트 편집→csv-to-table 왕복 서사.",
        mime_type: "text/markdown",
        text: include_str!("../mydocs/manual/recipes/02_table_csv_roundtrip.md"),
    },
    DocResource {
        uri: "rhwp://recipes/03_redact_before_sharing.md",
        name: "recipe-03-redact",
        title: "레시피 3 — 배포 전 개인정보 마스킹",
        description: "redact --dry-run 검토→실행→재검사 서사 (--no-raw 기본).",
        mime_type: "text/markdown",
        text: include_str!("../mydocs/manual/recipes/03_redact_before_sharing.md"),
    },
    DocResource {
        uri: "rhwp://recipes/04_safety_check_untrusted_doc.md",
        name: "recipe-04-safety-check",
        title: "레시피 4 — 출처를 모르는 문서를 처음 열 때",
        description: "inspect 3축(은닉·주입·유니코드) 선검사 서사.",
        mime_type: "text/markdown",
        text: include_str!("../mydocs/manual/recipes/04_safety_check_untrusted_doc.md"),
    },
    DocResource {
        uri: "rhwp://recipes/05_mail_merge_batch_fill.md",
        name: "recipe-05-mail-merge",
        title: "레시피 5 — 서식 하나에 여러 사람 데이터를 한 번에 채우기",
        description: "batch fill 메일머지 서사 (행 파일→산출물 N).",
        mime_type: "text/markdown",
        text: include_str!("../mydocs/manual/recipes/05_mail_merge_batch_fill.md"),
    },
    DocResource {
        uri: "rhwp://recipes/06_visual_regression_before_after.md",
        name: "recipe-06-visual-regression",
        title: "레시피 6 — 편집 전후를 눈이 아니라 숫자로 비교하기",
        description: "render-diff 픽셀 판정 서사 (bbox 불변 증명).",
        mime_type: "text/markdown",
        text: include_str!("../mydocs/manual/recipes/06_visual_regression_before_after.md"),
    },
];

/// [#3627 잔여] 스키마 리소스 — 본문이 파일이 아니라 **생성기**다.
///
/// export-ir-schema · export-plan-schema · export-capabilities-schema 가 내는 것과
/// 같은 lib 함수를 부른다 — CLI 봉투와 이 리소스가 한 원천에서 같은 값을 낸다.
/// 파일로 얼리지 않는 이유: 스키마는 코드에서 파생되므로 얼린 사본은 첫 변경부터
/// 낡는다(DOC_RESOURCES 가 원본 문서를 직접 안는 것과 같은 원리의 생성기 판).
struct SchemaResource {
    uri: &'static str,
    name: &'static str,
    title: &'static str,
    description: &'static str,
    generate: fn() -> serde_json::Value,
}

const SCHEMA_RESOURCES: &[SchemaResource] = &[
    SchemaResource {
        uri: "rhwp://schemas/ir",
        name: "ir-schema",
        title: "공개 IR JSON Schema",
        description: "Document IR 의 JSON Schema 2020-12 — 바인딩·외부 검증기의 단일 출처 (export-ir-schema 동일).",
        generate: rhwp::ir_schema::ir_schema,
    },
    SchemaResource {
        uri: "rhwp://schemas/plan",
        name: "plan-schema",
        title: "편집 계획서(run) JSON Schema",
        description: "run 계획서 문법의 JSON Schema — 계획 생성의 단일 출처 (export-plan-schema 동일).",
        generate: rhwp::plan_schema::plan_schema,
    },
    SchemaResource {
        uri: "rhwp://schemas/capabilities",
        name: "capabilities-schema",
        title: "capabilities 봉투 JSON Schema",
        description: "capabilities 자기서술 봉투의 JSON Schema (export-capabilities-schema 동일).",
        generate: rhwp::capabilities_schema::capabilities_schema,
    },
];

/// resources/list 응답 본문.
///
/// 프로필은 리소스 **목록**을 필터하지 않는다 — 지식 지도·실패 사전은 특정 도구의
/// 사용설명서가 아니라 봉투 어휘·판정 규칙 같은 전 표면 공통 문서라, 가리면 그
/// 프로필이 실제로 가진 도구를 쓰는 능력만 깎인다. 대신 계약 문서인 매니페스트는
/// **내용**이 프로필로 렌더된다(read_resource) — tools/list 에 없는 도구를
/// 자기서술이 광고하면 에이전트가 "알 수 없는 도구" 를 밟는다.
fn served_resources() -> Vec<serde_json::Value> {
    let mut resources = vec![serde_json::json!({
        "uri": CAPABILITIES_URI,
        "name": "capabilities-mcp",
        "title": "rhwp MCP 자기서술 매니페스트",
        "description": "이 서버가 제공하는 도구의 이름·설명·입력 스키마·CLI 배선. \
                        --profile 로 띄운 서버는 tools/list 와 같은 필터된 목록을 낸다.",
        "mimeType": "application/json",
    })];
    resources.extend(SCHEMA_RESOURCES.iter().map(|r| {
        serde_json::json!({
            "uri": r.uri,
            "name": r.name,
            "title": r.title,
            "description": r.description,
            "mimeType": "application/json",
            // size 없음 — 본문이 생성기라 목록 시점에 길이를 약속하지 않는다.
        })
    }));
    resources.extend(DOC_RESOURCES.iter().map(|r| {
        serde_json::json!({
            "uri": r.uri,
            "name": r.name,
            "title": r.title,
            "description": r.description,
            "mimeType": r.mime_type,
            "size": r.text.len(),
        })
    }));
    resources
}

/// resources/read 본체. Err 는 (코드, 메시지, uri) — 미지의 URI 는 스펙이 정한
/// -32002, 잘못된 요청 구조는 -32602 로 가른다.
fn read_resource(
    params: &serde_json::Value,
    profile: Option<&'static crate::agent_profiles::AgentProfile>,
) -> Result<serde_json::Value, (i64, String, Option<String>)> {
    let Some(uri) = params.get("uri").and_then(|u| u.as_str()) else {
        return Err((INVALID_PARAMS, "params.uri 가 필요합니다".into(), None));
    };
    let (mime_type, text) = if uri == CAPABILITIES_URI {
        // 단일 출처: `capabilities --mcp` 의 stdout 과 같은 함수가 낸 값이다.
        (
            "application/json",
            crate::mcp_manifest_value(profile).to_string(),
        )
    } else if let Some(r) = SCHEMA_RESOURCES.iter().find(|r| r.uri == uri) {
        ("application/json", (r.generate)().to_string())
    } else {
        match DOC_RESOURCES.iter().find(|r| r.uri == uri) {
            Some(r) => (r.mime_type, r.text.to_string()),
            None => {
                return Err((
                    RESOURCE_NOT_FOUND,
                    format!("알 수 없는 리소스: {uri}"),
                    Some(uri.to_string()),
                ))
            }
        }
    };
    // contents 는 배열이다 — 한 URI 가 여러 조각을 낼 수 있다는 스펙 형태를 지킨다.
    Ok(serde_json::json!({
        "contents": [{ "uri": uri, "mimeType": mime_type, "text": text }]
    }))
}

/// 리소스 오류는 스펙 예시대로 `data.uri` 로 어떤 URI 가 문제였는지 되돌려준다.
fn resource_error_response(
    id: serde_json::Value,
    code: i64,
    message: &str,
    uri: Option<&str>,
) -> serde_json::Value {
    let mut response = error_response(id, code, message);
    if let Some(uri) = uri {
        response["error"]["data"] = serde_json::json!({ "uri": uri });
    }
    response
}

/// tools/list 응답: 선언 도구(MCP 필수 3종만 노출) + 세션 도구 3종.
fn served_tools(
    tool_defs: &[serde_json::Value],
    session_allows: &dyn Fn(&str) -> bool,
) -> Vec<serde_json::Value> {
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
    // 세션 도구는 이름 단위로 걸러 내보낸다 — tools/list 와 tools/call 이 같은
    // 판정 함수를 쓰므로 목록에서 뺀 도구를 호출로 우회할 수 없다.
    let mut session: Vec<serde_json::Value> = Vec::new();
    session.push(serde_json::json!({
        "name": "hwp_open",
        "description": "문서를 파싱해 세션 핸들(docId)을 연다. 대형 문서를 여러 번 조회할 때 재파싱을 피한다. 암호 문서는 선택 password를 쓴다. 조회가 끝나면 hwp_close 로 닫는다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "HWP/HWPX/HML 문서 경로" },
                "password": {
                    "type": "string",
                    "writeOnly": true,
                    "description": "암호 문서 비밀번호. 서버는 응답과 세션 상태에 보존하지 않는다."
                }
            },
            "required": ["path"]
        }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_text",
        "description": "hwp_open 으로 연 핸들에서 페이지 텍스트를 재파싱 없이 읽는다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string", "description": "hwp_open 이 돌려준 핸들" },
                "page": { "type": "integer", "minimum": 0, "description": "0부터 시작하는 페이지 번호. 생략하면 전체" },
                "maxChars": { "type": "integer", "minimum": 1, "description": "[#3787 S7] 본문 전체의 문자 상한. 넘으면 truncated:true 와 omittedCount(생략 문자 수)를 봉투에 남긴다. 생략하면 무제한" }
            },
            "required": ["docId"]
        }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_info",
        "description": "[#3609] 핸들의 메타(형식·페이지/문단 수·폰트)를 재파싱 없이 조회한다. 편집 후 페이지 수 변화를 추적할 때 쓴다. 봉투는 hwp_info 와 동형.",
        "inputSchema": { "type": "object", "properties": { "docId": { "type": "string" } }, "required": ["docId"] }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_fields",
        "description": "[#3609] 핸들의 누름틀을 재파싱 없이 조사한다. hwp_doc_fill_fields 직후 반영값 확인에 쓴다. 봉투는 hwp_fields 와 동형.",
        "inputSchema": { "type": "object", "properties": { "docId": { "type": "string" } }, "required": ["docId"] }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_tables",
        "description": "[#3609] 핸들의 표 격자를 재파싱 없이 추출한다. 봉투는 hwp_export_tables 와 동형.",
        "inputSchema": { "type": "object", "properties": { "docId": { "type": "string" } }, "required": ["docId"] }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_render_page",
        "description": "[#3609] 핸들에서 해당 쪽을 SVG 로 렌더해 저장한다 — 편집 직후 눈검증(VLM) 루프가 세션 안에서 닫힌다.",
        "inputSchema": { "type": "object", "properties": { "docId": { "type": "string" }, "page": { "type": "integer", "minimum": 0 }, "output": { "type": "string", "description": "출력 SVG 경로" } }, "required": ["docId", "page", "output"] }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_search",
        "description": "[#3601] hwp_open 으로 연 핸들에서 재파싱 없이 검색한다. 주소 어휘(matches[].section/paragraph/page/context)는 hwp_search 와 동형 — 대형 문서에서 '어디를 고칠까'를 반복 탐색할 때 쓴다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string", "description": "hwp_open 이 돌려준 핸들" },
                "query": { "type": "string", "minLength": 1, "description": "검색어" },
                "caseSensitive": { "type": "boolean", "description": "대소문자 구분. 기본 true" },
                "maxMatches": { "type": "integer", "minimum": 1, "description": "[#3787 S7] 반환 매치 상한. 절단되면 totalMatchCount·truncated:true·omittedCount 가 총량을 알린다. 생략하면 무제한" }
            },
            "required": ["docId", "query"]
        }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_replace_text",
        "description": "[#3601] 핸들의 IR 에 문자열 일괄 치환을 누적한다(디스크 미기록 — hwp_doc_save 가 기록 지점). replacedCount 0 은 오류가 아니라 계수 보고다. hwp_doc_fill_fields 와 조합해 '채우고 다듬고 한 번에 저장'하는 흐름을 만든다. [#3719] 봉투의 changedPages:[n,…]|null 은 재조판 후 0 기준 쪽 번호 — 그 쪽만 hwp_doc_render_page 로 렌더하면 눈검증이 끝난다(null 이면 확정 불가이니 전체를 보라).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string", "description": "hwp_open 이 돌려준 핸들" },
                "find": { "type": "string", "minLength": 1, "description": "찾을 문자열" },
                "replace": { "type": "string", "description": "바꿀 문자열 (빈 문자열이면 삭제)" },
                "caseSensitive": { "type": "boolean", "description": "대소문자 구분. 기본 true" }
            },
            "required": ["docId", "find", "replace"]
        }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_set_cell",
        "description": "[#3603] 핸들의 표 격자 좌표(hwp_doc_tables 와 동일)에 값을 기록한다 — 디스크 미기록, hwp_doc_save 가 기록 지점. 병합으로 덮인 칸은 앵커 좌표를 안내하며 실패하고, 칸 넘침은 overflow 로 보고한다(무상태 hwp_set_cell 과 동형). [#3719] 봉투의 changedPages:[n,…]|null 은 재조판 후 0 기준 쪽 번호로, 분할된 표는 걸친 쪽을 전부 담는다 — 그 쪽만 hwp_doc_render_page 로 렌더하면 눈검증이 끝난다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string" },
                "table": { "type": "integer", "minimum": 0, "description": "본문 최상위 표 번호" },
                "row": { "type": "integer", "minimum": 0 },
                "col": { "type": "integer", "minimum": 0 },
                "text": { "type": "string", "description": "셀에 넣을 값 (빈 문자열이면 비우기)" },
                "keepStyle": { "type": "boolean", "description": "true 면 셀 스타일 상속 유지 (기본: 검정·비이탤릭 정규화)" }
            },
            "required": ["docId", "table", "row", "col", "text"]
        }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_fill_fields",
        "description": "[#3598] hwp_open 으로 연 핸들의 IR 에 누름틀 값을 직접 채운다(디스크 미기록 — hwp_doc_save 가 유일한 기록 지점). 여러 번 호출하면 누적된다. 판정 필드(filledCount/notFound/ambiguous)는 hwp_fill_fields 와 동형이고, 반복 필드는 '이름[N]' 으로 지목한다. [#3719] 봉투의 changedPages:[n,…]|null 은 재조판 후 0 기준 쪽 번호 — 그 쪽만 hwp_doc_render_page 로 렌더하면 눈검증 루프가 세션 안에서 상수 비용으로 닫힌다(null 이면 확정 불가이니 전체를 보라).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string", "description": "hwp_open 이 돌려준 핸들" },
                "data": { "type": "object", "description": "{\"필드이름\":\"값\"} 객체. 반복 필드는 \"이름[N]\"(0 기준)" }
            },
            "required": ["docId", "data"]
        }
    }));
    session.push(serde_json::json!({
        "name": "hwp_doc_save",
        "description": "[#3598] 핸들에 누적된 편집을 형식 보존(HWPX→HWPX, 그 외→HWP5, #3383 규약)으로 저장한다. 핸들은 저장 후에도 열려 있다 — 이어서 편집·재저장할 수 있다.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "docId": { "type": "string", "description": "hwp_open 이 돌려준 핸들" },
                "output": { "type": "string", "description": "출력 파일 경로" },
                "verify": { "type": "boolean", "description": "true 면 저장본 재파싱 IR 자기검증(verify 필드)" }
            },
            "required": ["docId", "output"]
        }
    }));
    session.push(serde_json::json!({
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
    tools.extend(
        session
            .into_iter()
            .filter(|t| session_allows(t["name"].as_str().unwrap_or_default())),
    );
    tools
}

/// tools/call 본체. Err 는 JSON-RPC 오류(잘못된 요청 구조), Ok(isError=true) 는
/// 도구 실행 실패(MCP 규약: 실행 실패는 프로토콜 오류가 아니라 도구 결과다).
fn handle_tool_call(
    params: &serde_json::Value,
    tool_defs: &[serde_json::Value],
    session_allows: &dyn Fn(&str) -> bool,
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

    // tools/list에서 제거한 세션 도구는 호출로 우회할 수도 없어야 한다. 프로필은
    // 추천 목록이 아니라 서버가 실제로 제공하는 도구 집합의 경계다.
    // 목록 필터와 **같은 판정 함수**를 쓴다 — 둘이 갈라지면 경계가 뚫린다.
    if is_session_tool(name) && !session_allows(name) {
        return Ok(tool_error(format!(
            "현재 프로필에서는 세션 도구를 제공하지 않습니다: {name}"
        )));
    }

    match name {
        "hwp_open" => Ok(session_open(&args, sessions)),
        "hwp_doc_text" => Ok(session_doc_text(&args, sessions)),
        "hwp_doc_info" => Ok(session_info(&args, sessions)),
        "hwp_doc_fields" => Ok(session_fields(&args, sessions)),
        "hwp_doc_tables" => Ok(session_tables(&args, sessions)),
        "hwp_doc_render_page" => Ok(session_render_page(&args, sessions)),
        "hwp_doc_search" => Ok(session_search(&args, sessions)),
        "hwp_doc_replace_text" => Ok(session_replace_text(&args, sessions)),
        "hwp_doc_set_cell" => Ok(session_set_cell(&args, sessions)),
        "hwp_doc_fill_fields" => Ok(session_fill_fields(&args, sessions)),
        "hwp_doc_save" => Ok(session_save(&args, sessions)),
        "hwp_close" => Ok(session_close(&args, sessions)),
        _ => {
            let Some(def) = tool_defs.iter().find(|t| t["name"] == name) else {
                // [#3694] didYouMean — error 필드가 기존 원문을 담아 하위호환.
                let error = format!("알 수 없는 도구: {name}");
                let candidates: Vec<&str> = tool_defs
                    .iter()
                    .filter_map(|t| t["name"].as_str())
                    .collect();
                let did_you_mean: Vec<String> = crate::closest_name(name, candidates.into_iter())
                    .into_iter()
                    .collect();
                let mut body = serde_json::json!({ "error": error, "didYouMean": did_you_mean });
                if let Some(best) = body["didYouMean"][0].as_str() {
                    body["nextCall"] = serde_json::json!({
                        "name": best,
                        "arguments": {},
                        "why": "요청한 이름이 없음 — 가장 가까운 실존 도구로 교정"
                    });
                }
                return Ok(tool_error(body.to_string()));
            };
            Ok(run_cli_tool(def, &args))
        }
    }
}

fn is_session_tool(name: &str) -> bool {
    matches!(
        name,
        "hwp_open"
            | "hwp_doc_text"
            | "hwp_doc_info"
            | "hwp_doc_fields"
            | "hwp_doc_tables"
            | "hwp_doc_render_page"
            | "hwp_doc_search"
            | "hwp_doc_replace_text"
            | "hwp_doc_set_cell"
            | "hwp_doc_fill_fields"
            | "hwp_doc_save"
            | "hwp_close"
    )
}

/// [#3699] 교정 호출 동봉 오류 — error 필드가 기존 원문을 담아 하위호환.
/// nextCall.name 은 반드시 실존 도구(호출부 책임, 계약 테스트가 고정).
fn tool_error_with_next(
    message: String,
    next_name: &str,
    next_args: serde_json::Value,
    why: &str,
) -> serde_json::Value {
    tool_error(
        serde_json::json!({
            "error": message,
            "nextCall": { "name": next_name, "arguments": next_args, "why": why }
        })
        .to_string(),
    )
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
    let password = match mcp_password(args) {
        Ok(password) => password,
        Err(message) => return tool_error(message),
    };
    let document_result = match password.as_deref() {
        Some(password) => HwpDocument::from_bytes_with_password(&data, password.as_bytes()),
        None => HwpDocument::from_bytes(&data),
    };
    let doc = match document_result {
        Ok(d) => d,
        Err(e) => return tool_error(format!("{path} 파싱 실패: {e}")),
    };
    // [#3598] save 의 형식 보존을 위해 원본 형식을 핸들에 함께 기억한다.
    let detected_format = rhwp::parser::detect_format(&data);
    let source_is_hwpx = matches!(detected_format, rhwp::parser::FileFormat::Hwpx);
    let size_bytes = data.len();
    let page_count = doc.page_count();
    let doc_id = format!("doc-{}", sessions.next_id);
    sessions.next_id += 1;
    sessions.docs.insert(
        doc_id.clone(),
        SessionDoc {
            doc,
            source_is_hwpx,
            size_bytes,
            detected_format,
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

/// MCP password는 자식 CLI stdin의 첫 줄과 같은 계약으로 제한한다. 값은 호출 범위를
/// 벗어나 저장하지 않으며, 오류 문자열에도 포함하지 않는다.
fn mcp_password(args: &serde_json::Value) -> Result<Option<String>, String> {
    let Some(value) = args.get("password") else {
        return Ok(None);
    };
    let Some(password) = value.as_str() else {
        return Err("password는 문자열이어야 합니다".into());
    };
    if password.contains(['\r', '\n']) {
        return Err("password에는 줄바꿈을 포함할 수 없습니다".into());
    }
    Ok(Some(password.to_string()))
}

/// MCP 인자 강제변환의 단일 계약 — **없음**과 **있는데 형식이 틀림**을 가른다.
///
/// `args.get(k).and_then(|v| v.as_u64())` 는 두 경우를 모두 `None` 으로 뭉갠다. 그러면
/// `page: -1` 같은 오타가 "page 생략"과 구별되지 않아, 한 쪽만 달라던 요청이 문서 전체를
/// **성공 응답으로** 받아 간다. 호출자(에이전트)는 isError 도 경고도 못 보므로 오타를
/// 알아챌 방법이 없다 — 조용히 틀린 답을 주는 부류라 반드시 거부해야 한다.
///
/// `null` 만 관용적으로 "생략"으로 읽는다. 다수 MCP 호스트가 미지정 선택 인자를 `null` 로
/// 직렬화하므로, 이를 오류로 만들면 멀쩡한 호출이 깨진다.
fn opt_u64(args: &serde_json::Value, key: &str) -> Result<Option<u64>, String> {
    let Some(v) = args.get(key) else {
        return Ok(None);
    };
    if v.is_null() {
        return Ok(None);
    }
    if let Some(n) = v.as_u64() {
        return Ok(Some(n));
    }
    // 파이썬 계열 호스트는 정수를 `3.0` 으로 직렬화하고 JSON Schema 도 이를 integer 로
    // 인정한다. 소수부 없는 음이 아닌 값만 받아 준다 — `2.5`·`-1`·`"3"`·`true` 는 거부.
    if let Some(f) = v.as_f64() {
        if f.fract() == 0.0 && f >= 0.0 && f <= u64::MAX as f64 {
            return Ok(Some(f as u64));
        }
    }
    Err(format!("{key} 는 0 이상의 정수여야 합니다 (받은 값: {v})"))
}

/// `opt_u64` 의 불리언 판. `"true"`/`1` 같은 근사값도 거부한다 — 선언이 `boolean` 인데
/// 실행이 관용 변환을 하면 선언과 실행이 어긋나고, 어긋난 쪽은 늘 조용하다.
fn opt_bool(args: &serde_json::Value, key: &str) -> Result<Option<bool>, String> {
    match args.get(key) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Bool(b)) => Ok(Some(*b)),
        Some(v) => Err(format!(
            "{key} 는 true 또는 false 여야 합니다 (받은 값: {v})"
        )),
    }
}

/// [#3787 S7] 자원 상한(1 이상) 인자. 생략은 **무제한**이고, `0`·음수·소수·문자열은
/// 거부한다 — `0` 을 "무제한"으로 뭉개면 "아무것도 주지 마라"는 요청이 "전부 달라"가
/// 되어 정반대로 실행된다.
fn opt_limit(args: &serde_json::Value, key: &str) -> Result<Option<usize>, String> {
    match opt_u64(args, key)? {
        None => Ok(None),
        Some(0) => Err(format!("{key} 는 1 이상이어야 합니다 (생략하면 무제한)")),
        Some(n) => usize::try_from(n)
            .map(Some)
            .map_err(|_| format!("{key} 범위 초과: {n}")),
    }
}

/// 필수 정수. "생략"과 "형식 오류"를 서로 다른 문구로 보고한다 — 같은 문구로 뭉개면
/// 호출자가 값이 아니라 호출 형태를 의심하며 헛수고한다.
fn req_u64(args: &serde_json::Value, key: &str) -> Result<u64, String> {
    match opt_u64(args, key)? {
        Some(n) => Ok(n),
        None => Err(format!("{key} 가 필요합니다")),
    }
}

fn session_doc_text(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return tool_error("docId 가 필요합니다".into());
    };
    let Some(sd) = sessions.docs.get_mut(doc_id) else {
        return tool_error_with_next(
            format!("열려 있지 않은 핸들: {doc_id} (hwp_open 먼저)"),
            "hwp_open",
            serde_json::json!({ "path": "<열 문서 경로>" }),
            "핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도",
        );
    };
    let doc = &mut sd.doc;
    let page_count = doc.page_count();
    // page 오타(-1·2.5·"3")를 "생략"과 갈라 낸다. 뭉개면 아래 `None` 갈래로
    // 떨어져 **문서 전체**가 성공 응답으로 나간다 — 한 쪽만 달라던 호출과 구별 불가.
    let page_arg = match opt_u64(args, "page") {
        Ok(v) => v,
        Err(e) => return tool_error(e),
    };
    // [#3787 S7] 컨텍스트 범람 방어 상한. 생략하면 무제한(종전 동작).
    let max_chars = match opt_limit(args, "maxChars") {
        Ok(v) => v,
        Err(e) => return tool_error(e),
    };
    let pages: Vec<u32> = match page_arg {
        Some(raw_page) => {
            let p = match u32::try_from(raw_page) {
                Ok(p) => p,
                Err(_) => return tool_error(format!("페이지 번호 범위 초과: {raw_page}")),
            };
            if p >= page_count {
                return tool_error(format!("페이지 범위 초과: {p} (0~{})", page_count - 1));
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };
    let mut extracted = Vec::with_capacity(pages.len());
    for p in pages {
        match doc.extract_page_text_native(p) {
            Ok(text) => extracted.push((p, text)),
            Err(e) => return tool_error(format!("페이지 {p} 텍스트 추출 실패: {e:?}")),
        }
    }
    // [#3787 S7] 무상태 `export-text --json --max-chars` 와 같은 helper 를 쓴다 —
    // 절단 어휘(truncated·omittedCount)가 두 표면에서 갈라지지 않게 한다.
    let (page_objs, omitted_count) = crate::truncate_page_texts(&extracted, max_chars);
    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": doc_id,
            "pageCount": page_objs.len(),
            "truncated": omitted_count > 0,
            "omittedCount": omitted_count,
            "pages": page_objs,
        })
        .to_string(),
    )
}

/// [#3609] 세션 조회 4종 — 전부 무상태 봉투 helper 재사용(동형 보장).
fn with_doc<'a>(
    args: &serde_json::Value,
    sessions: &'a mut Sessions,
) -> Result<(&'a mut SessionDoc, String), serde_json::Value> {
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return Err(tool_error("docId 가 필요합니다".into()));
    };
    let id = doc_id.to_string();
    match sessions.docs.get_mut(&id) {
        Some(sd) => Ok((sd, id)),
        None => Err(tool_error_with_next(
            format!("열려 있지 않은 핸들: {id} (hwp_open 먼저)"),
            "hwp_open",
            serde_json::json!({ "path": "<열 문서 경로>" }),
            "핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도",
        )),
    }
}

fn session_info(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let (sd, id) = match with_doc(args, sessions) {
        Ok(v) => v,
        Err(e) => return e,
    };
    tool_ok_text(
        crate::info_json_value(&id, sd.size_bytes, sd.detected_format, &sd.doc).to_string(),
    )
}

fn session_fields(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let (sd, id) = match with_doc(args, sessions) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let fields = crate::collect_field_records(&sd.doc);
    tool_ok_text(crate::fields_json_value(&id, &fields).to_string())
}

fn session_tables(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let (sd, id) = match with_doc(args, sessions) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let tables = rhwp::document_core::queries::table_extract::extract_tables(sd.doc.document());
    tool_ok_text(crate::tables_json_value(&id, &tables).to_string())
}

fn session_render_page(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    // page 는 필수 인자다. as_u64() 로 뭉개면 `page: -1` 도 "page 가 필요합니다"
    // 로 보고돼, 보냈는데 없다고 하는 오진이 된다.
    let raw_page = match req_u64(args, "page") {
        Ok(v) => v,
        Err(e) => return tool_error(e),
    };
    let page = match u32::try_from(raw_page) {
        Ok(page) => page,
        Err(_) => return tool_error(format!("페이지 번호 범위 초과: {raw_page}")),
    };
    let Some(output) = args
        .get("output")
        .and_then(|o| o.as_str())
        .map(String::from)
    else {
        return tool_error("output 이 필요합니다".into());
    };
    let (sd, id) = match with_doc(args, sessions) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let page_count = sd.doc.page_count();
    if page_count == 0 {
        return tool_error("렌더 가능한 페이지가 없습니다".into());
    }
    if page >= page_count {
        return tool_error(format!("페이지 범위 초과: {page} (0~{})", page_count - 1));
    }
    let svg = match sd.doc.render_page_svg(page) {
        Ok(s) => s,
        Err(e) => return tool_error(format!("페이지 {page} 렌더 실패: {e:?}")),
    };
    if let Err(e) = std::fs::write(&output, &svg) {
        return tool_error(format!("{output} 쓰기 실패: {e}"));
    }
    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": id,
            "page": page,
            "format": "svg",
            "output": output,
            "bytes": svg.len(),
        })
        .to_string(),
    )
}

/// [#3601] 열린 핸들에서 재파싱 없이 검색한다. 봉투는 무상태 `search --json` 과
/// 같은 helper(`crate::search_json_value`)를 재사용해 주소 어휘 동형을 보장한다
/// (`source` 자리에는 경로 대신 핸들 docId 가 들어간다).
fn session_search(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return tool_error("docId 가 필요합니다".into());
    };
    let Some(query) = args.get("query").and_then(|q| q.as_str()) else {
        return tool_error("query 가 필요합니다".into());
    };
    if query.is_empty() {
        return tool_error("query 는 빈 문자열일 수 없습니다".into());
    }
    // `"false"`·`0` 은 as_bool() 에서 None → 기본값 true 로 되돌아간다. 축을
    // 끄라고 보낸 요청이 켠 채 실행되고, 봉투는 caseSensitive:true 를 **성공**으로
    // 보고한다. 검색 결과가 조용히 달라지므로 거부가 유일하게 안전한 처리다.
    let case_sensitive = match opt_bool(args, "caseSensitive") {
        Ok(v) => v.unwrap_or(true),
        Err(e) => return tool_error(e),
    };
    // [#3787 S7] 컨텍스트 범람 방어 상한. 생략하면 무제한(종전 동작).
    let max_matches = match opt_limit(args, "maxMatches") {
        Ok(v) => v,
        Err(e) => return tool_error(e),
    };
    let Some(sd) = sessions.docs.get_mut(doc_id) else {
        return tool_error_with_next(
            format!("열려 있지 않은 핸들: {doc_id} (hwp_open 먼저)"),
            "hwp_open",
            serde_json::json!({ "path": "<열 문서 경로>" }),
            "핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도",
        );
    };
    // [#3787 S7] 총량은 전수 grep 으로 세고 **표시만** 자른다 — 무상태 `search
    // --max-matches` 와 같은 규칙이라 totalMatchCount 가 두 표면에서 같은 뜻이다.
    let all = sd.doc.grep(query, case_sensitive, None);
    let total = all.len();
    let shown: Vec<_> = match max_matches {
        Some(n) => all.into_iter().take(n).collect(),
        None => all,
    };
    tool_ok_text(crate::search_json_value(doc_id, query, case_sensitive, &shown, total).to_string())
}

/// [#3719 §6-1] 세션 편집 봉투의 `changedPages` — 무상태 판(#3712)과 **같은** 코어
/// 질의(`DocumentCore::pages_covering_paragraphs`)를 재사용한다. 새 계산은 없다.
///
/// 호출 시점이 계약의 절반이다. 세션은 편집 후에도 같은 인스턴스가 살아 있어서
/// **재조판 전에 쪽을 계산하면 편집 전 레이아웃을 보고한다**(#3704 가 조회 4종에서
/// 고친 바로 그 스테일). 질의가 진입에서 `paginate_if_needed()` 를 부르므로 편집 →
/// 질의 순서만 지키면 되고, 이미 조판이 맞았다면 dirty 구역이 없어 사실상 무비용이다.
///
/// 대상 문단이 하나라도 조판 커버리지 밖이면 부분 목록 대신 `null` — 빠뜨린 쪽이
/// 거짓 통과를 만들기 때문이다(#3630 P3, 원칙 5).
fn changed_pages_value(doc: &mut HwpDocument, targets: &[(usize, usize)]) -> serde_json::Value {
    match doc.pages_covering_paragraphs(targets) {
        Some(pages) => serde_json::json!(pages),
        None => serde_json::Value::Null,
    }
}

/// [#3601] 핸들의 IR 에 문자열 일괄 치환을 누적한다 — 디스크 미기록, save 가 기록 지점.
/// 무상태 `edit replace-text` 와 같은 코어 경로(`replace_all_native`)를 재사용한다.
fn session_replace_text(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return tool_error("docId 가 필요합니다".into());
    };
    let Some(find) = args.get("find").and_then(|f| f.as_str()) else {
        return tool_error("find 가 필요합니다".into());
    };
    if find.is_empty() {
        return tool_error("find 는 빈 문자열일 수 없습니다".into());
    }
    let Some(replace) = args.get("replace").and_then(|r| r.as_str()) else {
        return tool_error("replace 가 필요합니다".into());
    };
    // 검색과 달리 여기서는 **문서가 바뀐다**. caseSensitive 오타가 조용히
    // true 로 되돌아가면 치환 대상 집합이 달라진 채 IR 에 누적되고, save 가 그대로
    // 디스크에 굳힌다 — 되돌릴 수 없는 축이라 더더욱 거부해야 한다.
    let case_sensitive = match opt_bool(args, "caseSensitive") {
        Ok(v) => v.unwrap_or(true),
        Err(e) => return tool_error(e),
    };
    let Some(sd) = sessions.docs.get_mut(doc_id) else {
        return tool_error_with_next(
            format!("열려 있지 않은 핸들: {doc_id} (hwp_open 먼저)"),
            "hwp_open",
            serde_json::json!({ "path": "<열 문서 경로>" }),
            "핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도",
        );
    };
    // [#3719 §6-1] 치환 **전** 매치 주소를 붙잡는다 — 문자열 치환은 문단을 새로 만들지
    // 않아 인덱스를 밀지 않으므로, 이 주소가 치환 후에도 그대로 유효하다. 무상태
    // `edit replace-text`(#3712)가 쓰는 근거와 같은 것이라 두 봉투가 같은 쪽을 답한다.
    let changed_paras: Vec<(usize, usize)> = sd
        .doc
        .grep(find, case_sensitive, None)
        .iter()
        .map(|m| (m.section, m.paragraph))
        .collect();
    let result = match sd.doc.replace_all_native(find, replace, case_sensitive) {
        Ok(r) => r,
        Err(e) => return tool_error(format!("치환 실패: {e}")),
    };
    // replace_all_native 는 {"ok":true,"count":N} 문자열을 낸다 — 계수만 뽑아
    // 세션 봉투 어휘(replacedCount)로 정규화한다.
    let count = serde_json::from_str::<serde_json::Value>(&result)
        .ok()
        .and_then(|v| v["count"].as_u64())
        .unwrap_or(0);
    // 치환이 실제로 일어났다면 핸들의 페이지 어휘를 즉시 갱신한다 — 코어는
    // recompose 로 dirty 만 남기므로, 여기서 재페이지네이션하지 않으면 이후
    // hwp_doc_info/text/render/search 가 편집 전 레이아웃을 서빙한다.
    if count > 0 {
        sd.doc.repaginate_if_needed();
    }
    // [#3719 §6-1] 눈검증 대상 쪽 — 위 재조판 **뒤**라야 편집 후 레이아웃을 보고한다.
    // 0건 치환은 IR 이 그대로다: 볼 쪽이 없으니 빈 목록이 정확하다("전체를 보라"는
    // null 로 내리면 무변경 호출마다 전수 렌더를 유도하게 된다).
    let changed_pages = if count > 0 {
        changed_pages_value(&mut sd.doc, &changed_paras)
    } else {
        serde_json::json!([])
    };
    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": doc_id,
            "find": find,
            "replace": replace,
            "caseSensitive": case_sensitive,
            "changedPages": changed_pages,
            "replacedCount": count,
        })
        .to_string(),
    )
}

/// [#3603] 핸들의 표 격자 좌표에 값을 기록한다 — resolve_table_cell(CLI 와 공유)로
/// 좌표를 해석하고, overflow 판정·검정 정규화까지 무상태 edit set-cell 과 동형이다.
fn session_set_cell(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    // 셋 다 보냈는데 하나가 음수여도 종전에는 "table/row/col 이 필요합니다" 였다.
    // 있는 인자를 없다고 말하는 오진이라, 호출자는 값이 아니라 호출 형태를 의심하며
    // 같은 실수를 반복한다. 축별로 따로 검사해 어느 축이 왜 틀렸는지 지목한다.
    let table_no = match req_u64(args, "table") {
        Ok(v) => v,
        Err(e) => return tool_error(e),
    };
    let row = match req_u64(args, "row") {
        Ok(v) => v,
        Err(e) => return tool_error(e),
    };
    let col = match req_u64(args, "col") {
        Ok(v) => v,
        Err(e) => return tool_error(e),
    };
    let Some(new_text) = args.get("text").and_then(|t| t.as_str()).map(String::from) else {
        return tool_error("text 가 필요합니다".into());
    };
    // keepStyle 오타는 "스타일 상속 유지" 요청을 조용히 검정 정규화로 되돌린다 —
    // 서식지 셀 서식이 말없이 지워지는 경로라 관용 변환을 두면 안 된다.
    let keep_style = match opt_bool(args, "keepStyle") {
        Ok(v) => v.unwrap_or(false),
        Err(e) => return tool_error(e),
    };
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return tool_error("docId 가 필요합니다".into());
    };
    // [#3603] 필수 인자가 모두 갖춰진 뒤, 핸들을 건드리기 전에 거부한다 — 무상태 CLI 는
    // 파일을 읽기도 전에 EXIT_USAGE 로 끊는다. 여기만 통과시키면 셀 문단 하나에 raw 개행이
    // 박힌 채 IR 에 누적되고, 그 핸들은 hwp_close 전까지 되돌릴 방법이 없다.
    if let Some(message) = crate::set_cell_control_char_rejection(&new_text) {
        return tool_error(message.to_string());
    }
    let id = doc_id.to_string();
    let Some(sd) = sessions.docs.get_mut(&id) else {
        return tool_error_with_next(
            format!("열려 있지 않은 핸들: {id} (hwp_open 먼저)"),
            "hwp_open",
            serde_json::json!({ "path": "<열 문서 경로>" }),
            "핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도",
        );
    };
    let table_no = match usize::try_from(table_no) {
        Ok(value) => value,
        Err(_) => return tool_error("table 값이 이 플랫폼의 범위를 벗어났습니다".into()),
    };
    let row = match u16::try_from(row) {
        Ok(value) => value,
        Err(_) => return tool_error("row 값은 0~65535 범위여야 합니다".into()),
    };
    let col = match u16::try_from(col) {
        Ok(value) => value,
        Err(_) => return tool_error("col 값은 0~65535 범위여야 합니다".into()),
    };
    let (sec, para, ctrl, cell_idx, para_lens, old_text) =
        match crate::resolve_table_cell(sd.doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(crate::CellResolveError::Usage(m)) | Err(crate::CellResolveError::Runtime(m)) => {
                return tool_error(m)
            }
        };
    let overflow = crate::measure_cell_overflow(&sd.doc, sec, para, ctrl, cell_idx, &new_text).map(
        |(cell_w, text_w, lines)| {
            serde_json::json!({
                "target": format!("table{}[{},{}]", table_no, row, col),
                // CLI 의 overflow 항목과 키 집합을 맞춘다 — 넘친 값이 무엇이었는지 없으면
                // 여러 칸을 연달아 채운 에이전트가 어느 값이 넘쳤는지 되짚을 수 없다.
                "text": new_text,
                "cellWidthPx": (cell_w * 100.0).round() / 100.0,
                "textWidthPx": (text_w * 100.0).round() / 100.0,
                "lines": lines,
            })
        },
    );
    for (pi, len) in para_lens.iter().enumerate() {
        if *len == 0 {
            continue;
        }
        if let Err(e) = sd.doc.delete_text_in_cell(
            sec as u32,
            para as u32,
            ctrl as u32,
            cell_idx as u32,
            pi as u32,
            0,
            *len as u32,
        ) {
            return tool_error(format!("셀 비우기 실패(문단 {pi}): {e:?}"));
        }
    }
    if !new_text.is_empty() {
        if let Err(e) = sd.doc.insert_text_in_cell(
            sec as u32,
            para as u32,
            ctrl as u32,
            cell_idx as u32,
            0,
            0,
            &new_text,
        ) {
            return tool_error(format!("셀 쓰기 실패: {e:?}"));
        }
        if !keep_style
            && !crate::recolor_cell_text_black(sd.doc.document_mut(), sec, para, ctrl, cell_idx)
        {
            // 경고 수준 — 봉투에 남기지 않고 진행 (CLI 와 동일한 관용).
        }
    }
    // [#3719 §6-1] 눈검증 대상 쪽 — 표 호스트 문단이 걸친 쪽 **전부**(분할 표 포함).
    // 근거 주소는 무상태 `edit set-cell`(#3712)과 같은 `resolve_table_cell` 의 호스트
    // 문단이다. 셀 편집 코어가 이미 재조판했으므로 이 질의는 편집 후 조판을 본다.
    let changed_pages = changed_pages_value(&mut sd.doc, &[(sec, para)]);
    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": id,
            "table": table_no, "row": row, "col": col,
            "oldText": old_text,
            "newText": new_text,
            "changedPages": changed_pages,
            // CLI 봉투와 같은 키 — 검정 정규화를 건너뛰었는지는 제출 서식에서 결과가
            // 달라지는 판단 재료라, 무엇이 적용됐는지 봉투만 보고 알 수 있어야 한다.
            "keepStyle": keep_style,
            "overflow": overflow.map(|o| vec![o]).unwrap_or_default(),
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
        return tool_error_with_next(
            format!("열려 있지 않은 핸들: {doc_id} (hwp_open 먼저)"),
            "hwp_open",
            serde_json::json!({ "path": "<열 문서 경로>" }),
            "핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도",
        );
    };
    let doc = &mut sd.doc;

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    // [#3719 §6-1] 같은 순회에서 문단 주소도 담는다 — changedPages 산출 근거를 무상태
    // `edit fill-fields`(#3712)와 같은 출처(FieldLocation)로 맞춘다.
    let mut name_locs: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for fi in doc.collect_all_fields().iter() {
        if let Some(n) = fi.field.field_name() {
            *name_counts.entry(n.to_string()).or_insert(0) += 1;
            name_locs
                .entry(n.to_string())
                .or_default()
                .push((fi.location.section_index, fi.location.para_index));
        }
    }
    let mut changed_paras: Vec<(usize, usize)> = Vec::new();

    let mut filled: Vec<serde_json::Value> = Vec::new();
    let mut not_found: Vec<String> = Vec::new();
    let mut ambiguous: Vec<serde_json::Value> = Vec::new();
    // [#3707] 개수 판정을 통과하지만 화면상 구별되지 않는 이름 쌍 — 무상태 경로와
    // 같은 코어(text_security)를 재사용해 판정 어휘를 동형으로 유지한다.
    let all_names: Vec<String> = name_counts.keys().cloned().collect();
    let confusable_groups = rhwp::document_core::text_security::confusable_collisions(&all_names);
    let mut confusable: Vec<serde_json::Value> = Vec::new();

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
        if let Some((_, group)) = confusable_groups
            .iter()
            .find(|(_, g)| g.iter().any(|n| n == name))
        {
            let others: Vec<&String> = group.iter().filter(|n| *n != name).collect();
            confusable.push(serde_json::json!({
                "name": name,
                "lookalikes": others,
                "note": "화면상 구별되지 않는 이름의 누름틀이 이 문서에 함께 있습니다 — 채운 칸이 의도한 칸인지 확인하세요.",
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
        if let Some(loc) = name_locs.get(name).and_then(|l| l.get(*occurrence)) {
            changed_paras.push(*loc);
        }
        filled.push(serde_json::json!({
            "name": name, "occurrence": occurrence, "value": value_str,
        }));
    }
    // 채움이 실제로 반영됐다면 핸들의 페이지 어휘를 즉시 갱신한다 — 코어의
    // set_field_value_by_name_at 는 recompose 로 dirty 만 남기므로, 여기서
    // 재페이지네이션하지 않으면 hwp_doc_info 의 pageCount("편집 후 페이지 수
    // 변화를 추적" 약속)와 text/render/search 의 page 주소가 전부 편집 전
    // 레이아웃에 머문다. 도구 호출당 1회 — 필드 수만큼 반복하지 않는다.
    if !apply.is_empty() {
        doc.repaginate_if_needed();
    }
    // [#3719 §6-1] 눈검증 대상 쪽 — 위 재조판 **뒤**라야 편집 후 레이아웃을 보고한다.
    // 채운 것이 없으면 빈 목록(볼 쪽 없음)이고, 채웠는데 문단→쪽 매핑을 확정할 수
    // 없으면 null 이다.
    let changed_pages = changed_pages_value(doc, &changed_paras);

    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": doc_id,
            "changedPages": changed_pages,
            "filledCount": filled.len(),
            "filled": filled,
            "notFound": not_found,
            "ambiguous": ambiguous,
            "confusable": confusable,
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
    // 저장은 스냅숏이다 — immutable borrow로 잡아 저장이 live handle IR을 바꾸지
    // 못하게 한다. 닫힌 핸들에는 기존의 재시도 안내도 유지한다.
    let Some(sd) = sessions.docs.get(doc_id) else {
        return tool_error_with_next(
            format!("열려 있지 않은 핸들: {doc_id} (hwp_open 먼저)"),
            "hwp_open",
            serde_json::json!({ "path": "<열 문서 경로>" }),
            "핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도",
        );
    };

    // [버그] `output` 경로의 확장자를 무시하고 원본 포맷(source_is_hwpx)만으로 직렬화
    // 형식을 정했다 — HWPX 핸들을 `.hwp` 경로로 저장해도 zip(HWPX) 바이트를 그대로
    // 써 버려 확장자와 실제 내용이 어긋났다. CLI의 `edit_output_format`(main.rs)은
    // 명시적 출력 확장자를 우선하는데, MCP 세션 경로만 비동형이었다. 같은 규칙을 쓴다.
    let explicit_ext = std::path::Path::new(output)
        .extension()
        .map(|ext| ext.to_string_lossy().to_ascii_lowercase());
    let format = match (sd.source_is_hwpx, explicit_ext.as_deref()) {
        (true, Some("hwp")) => crate::EditOutputFormat::Hwp,
        (true, _) => crate::EditOutputFormat::Hwpx,
        (false, _) => crate::EditOutputFormat::Hwp,
    };
    // HWP5 산출 경로의 어댑터(`convert_if_hwpx_source`)는 `Hwpx | Hwp3` 양쪽에서 돌며
    // 살아 있는 IR 을 제자리에서 고친다. 도구 계약이 "핸들은 저장 후에도 열려 있다"
    // 이므로 세션은 복제본에 어댑터를 태우는 스냅숏 경로를 쓴다.
    let bytes = match crate::edit_serialize_snapshot(&sd.doc, format) {
        Ok(b) => b,
        Err(e) => return tool_error(format!("직렬화 실패: {e}")),
    };
    if let Err(e) = std::fs::write(output, &bytes) {
        return tool_error(format!("{output} 쓰기 실패: {e}"));
    }
    // [#3702] verify:true — 저장본 재파싱 IR 자기검증. 세션은 exit 가 없으므로
    // isError:false 를 유지하고 판정은 verify 필드로 낸다(판정은 데이터).
    let verify = if args
        .get("verify")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let (report, _failed) = crate::edit_verify_report(&sd.doc, &bytes, false);
        report
    } else {
        serde_json::Value::Null
    };
    tool_ok_text(
        serde_json::json!({
            "schemaVersion": "1.0",
            "docId": doc_id,
            "output": output,
            "outputFormat": format.label(),
            "bytes": bytes.len(),
            "verify": verify,
        })
        .to_string(),
    )
}

fn session_close(args: &serde_json::Value, sessions: &mut Sessions) -> serde_json::Value {
    let Some(doc_id) = args.get("docId").and_then(|d| d.as_str()) else {
        return tool_error("docId 가 필요합니다".into());
    };
    if sessions.docs.remove(doc_id).is_none() {
        return tool_error_with_next(
            format!("열려 있지 않은 핸들: {doc_id}"),
            "hwp_open",
            serde_json::json!({ "path": "<열 문서 경로>" }),
            "핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도",
        );
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
    let mut cli_args = match substitute_args(&template, args) {
        Ok(a) => a,
        Err(e) => return tool_error(e),
    };
    if let Some(optional_args) = def["cli"]["optionalArgs"].as_array() {
        for optional in optional_args {
            let Some(key) = optional.get("when").and_then(|v| v.as_str()) else {
                return tool_error("MCP optionalArgs.when 정의가 올바르지 않습니다".into());
            };
            // 존재 여부만으로는 부족하다. `--dry-run` 같은 presence 플래그는 값이 없어
            // "있으면 켜짐" 이므로, `dryRun: false` 를 존재로 세면 **끄라고 보낸 요청이
            // 켜는 요청이 된다**. JSON 의 false/null 은 "그 축을 쓰지 않음" 으로 읽는다.
            match args.get(key) {
                None | Some(serde_json::Value::Null) | Some(serde_json::Value::Bool(false)) => {
                    continue;
                }
                Some(_) => {}
            }
            let Some(template) = optional.get("args").and_then(|v| v.as_array()) else {
                return tool_error(format!(
                    "MCP optionalArgs.{key}.args 정의가 올바르지 않습니다"
                ));
            };
            match substitute_args(template, args) {
                // [#3835] `cli.args` 에 POSIX `--` 옵션 종결자가 있으면(예: hwp_search 의
                // `{query}` 앞) 선택 인자를 그 **앞**에 끼워 넣는다. 뒤에 붙이면 이미
                // 닫힌 옵션 파싱 구간(위치 인자만 허용)에 플래그가 섞여 "인자가 너무
                // 많습니다" 가 된다. `--` 가 없는 배선은 종전처럼 끝에 붙인다.
                Ok(extra) => match cli_args.iter().position(|a| a == "--") {
                    Some(dash_pos) => {
                        cli_args.splice(dash_pos..dash_pos, extra);
                    }
                    None => cli_args.extend(extra),
                },
                Err(e) => return tool_error(e),
            }
        }
    }

    let password = match mcp_password(args) {
        Ok(password) => password,
        Err(message) => return tool_error(message),
    };

    // stdin 도구(hwp_batch 계열): paths 배열을 한 줄에 하나씩 흘려 넣는다.
    //
    // paths 가 없거나 형태가 틀린 채 자식을 띄우면 자식이 서버의 stdin — 즉 MCP
    // 프로토콜 스트림 자체 — 을 상속한다. 그 순간부터 클라이언트가 보내는 JSON-RPC
    // 프레임을 자식 batch 가 "파일 경로"로 읽어가고(응답 없는 요청), 서버는 자식이
    // EOF 를 볼 때까지 wait_with_output 에서 멈춘다. 그래서 stdin 도구는 자식을
    // 띄우기 전에 paths 를 선검증해 즉시 도구 오류로 돌려준다.
    let stdin_paths: Option<String> =
        if crate::MCP_STDIN_TOOLS.contains(&def["name"].as_str().unwrap_or_default()) {
            let Some(arr) = args.get("paths").and_then(|p| p.as_array()) else {
                return tool_error(
                    "paths 는 문자열 배열이어야 합니다 (예: {\"paths\":[\"a.hwp\"]})".into(),
                );
            };
            let mut paths = Vec::with_capacity(arr.len());
            for v in arr {
                match v.as_str() {
                    Some(s) => paths.push(s),
                    // 비문자열을 조용히 걸러내면 "3건을 보냈는데 0건 스윕"이 성공처럼
                    // 보인다 — 형태 오류는 실행 전에 그대로 알려준다.
                    None => {
                        return tool_error(format!("paths 항목은 문자열이어야 합니다: {v}"));
                    }
                }
            }
            if paths.is_empty() {
                return tool_error(
                    "paths 가 비어 있습니다 — 대상 문서 경로를 1개 이상 넣어 주세요".into(),
                );
            }
            Some(paths.join("\n"))
        } else {
            None
        };

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return tool_error(format!("실행 파일 경로 조회 실패: {e}")),
    };
    if password.is_some() && stdin_paths.is_some() {
        return tool_error(
            "batch MCP 도구는 경로 목록 stdin과 password를 함께 받을 수 없습니다".into(),
        );
    }
    if password.is_some() && def["cli"]["passwordStdin"].is_null() {
        return tool_error("이 MCP 도구는 password 입력을 지원하지 않습니다".into());
    }
    if password.is_some() {
        // 민감값은 argv가 아니라 기존 CLI의 stdin 계약으로만 전달한다.
        cli_args.push("--password-stdin".to_string());
    }

    let stdin_payload = password
        .map(|password| format!("{password}\n"))
        .or_else(|| stdin_paths.map(|paths| format!("{paths}\n")));
    let mut cmd = std::process::Command::new(exe);
    cmd.args(&cli_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    // 자식 stdin 은 payload(password 또는 paths)를 흘릴 때만 파이프, 그 외에는
    // 항상 닫는다(null) — 어떤 자식도 서버의 프로토콜 stdin 을 상속해서는 안 된다.
    if stdin_payload.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    } else {
        cmd.stdin(std::process::Stdio::null());
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return tool_error(format!("CLI 실행 실패: {e}")),
    };
    if let (Some(payload), Some(mut stdin)) = (stdin_payload, child.stdin.take()) {
        let _ = stdin.write_all(payload.as_bytes());
        // drop 으로 stdin 닫힘 — password reader와 batch가 EOF를 본다.
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
