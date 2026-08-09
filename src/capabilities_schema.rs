//! [#3776] `export-capabilities-schema` — capabilities 자체의 JSON Schema 를 기계 산출한다.
//!
//! IR 스키마(`ir_schema`, #3762)가 *문서 모델*의 자기서술이라면, 이것은 **명령 표면의
//! 자기서술**이다. 외부 바인딩(파이썬·Node·C#)의 타입 생성기가 `capabilities` 를 읽어
//! 명령 래퍼를 찍어내려면 `commands[].recordFields`·`flags`·`exitCodes` 의 **모양이
//! 고정**돼 있어야 한다. 고정돼 있지 않으면 생성기는 매 릴리스마다 추측을 다시 한다.
//!
//! ## 왜 손으로 쓰는가
//!
//! `capabilities` 출력은 `serde_json::json!` 리터럴로 조립된 값이라 파생할 타입 자체가
//! 없다. 실제 출력에서 역추론하면 "지금 마침 비어 있는 필드"가 계약에서 통째로 빠진다
//! (`commands[].requiresFeature` 는 native-skia 항목 하나에만 붙는다). 여기 적은 목록이
//! 곧 "우리가 외부에 약속하는 명령 표면"이다.
//!
//! ## 두 개의 스키마
//!
//! `capabilities` 와 `capabilities --mcp` 는 **서로 다른 봉투**다. 하나로 합치면 소비자가
//! 어느 필드가 어느 출력의 것인지 구분할 수 없으므로 `schema`(명령 표면)와
//! `mcpSchema`(도구 매니페스트)를 나란히 싣는다.
//!
//! ## 버저닝
//!
//! `capabilitiesSchemaVersion` 은 봉투 `schemaVersion`(명령별)과 **분리**된 전역 버전이다.
//! 필드 추가 = minor, 의미 변경·삭제 = major. major 는 분기 회고 승인 없이 금지.

use serde_json::{json, Value};

use crate::schema_registry::ENVELOPE_SCHEMA_VERSION;

/// capabilities 스키마 버전 — 단일 출처·판올림 이력은 [`crate::schema_registry`]
/// (#4329). 여기서는 재수출만 해 기존 호출부 경로를 보존한다.
pub use crate::schema_registry::CAPABILITIES_SCHEMA_VERSION;

/// JSON Schema draft — 소비자(코드 생성기)가 파서를 고를 수 있게 명시한다.
const SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// `$defs` 참조 하나를 만든다.
fn r(name: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{name}") })
}

/// 설명이 달린 원시 타입.
fn prim(ty: &str, description: &str) -> Value {
    json!({ "type": ty, "description": description })
}

/// 배열 타입.
fn array_of(items: Value, description: &str) -> Value {
    json!({ "type": "array", "items": items, "description": description })
}

/// 객체 타입 — 필수 필드를 명시하고 추가 필드를 허용한다.
///
/// `additionalProperties: true` 는 의도적이다. capabilities 는 **추가-전용 진화** 계약
/// 이므로(`jsonContract.schemaPolicy` 가 같은 말을 한다) 새 필드가 붙어도 기존 소비자가
/// 깨지지 않아야 한다. false 로 두면 명령을 하나 늘릴 때마다 모든 바인딩이 동시에 실패한다.
fn object(properties: Value, required: &[&str], description: &str) -> Value {
    json!({
        "type": "object",
        "description": description,
        "properties": properties,
        "required": required,
        "additionalProperties": true,
    })
}

/// 열거형 — 허용 값과 각 값의 뜻.
fn enum_of(values: &[(&str, &str)], description: &str) -> Value {
    let names: Vec<&str> = values.iter().map(|(n, _)| *n).collect();
    let doc = values
        .iter()
        .map(|(n, d)| format!("{n}={d}"))
        .collect::<Vec<_>>()
        .join(", ");
    json!({
        "type": "string",
        "enum": names,
        "description": format!("{description} ({doc})"),
    })
}

// ── capabilities 봉투 ────────────────────────────────────────────────────

fn capabilities_def() -> Value {
    object(
        json!({
            "schemaVersion": prim("string", "이 봉투의 스키마 버전 (현재 \"1.0\")"),
            "schemaRegistry": r("SchemaRegistry"),
            "tool": prim("string", "도구 이름. 항상 \"rhwp\"."),
            "version": prim("string", "rhwp 버전 — `rhwp --version` 과 같은 원천"),
            "formats": r("Formats"),
            "exitCodes": r("ExitCodes"),
            "jsonContract": r("JsonContract"),
            "batch": r("BatchContract"),
            "commands": array_of(r("Command"), "전 명령 목록. 이름 중복 없음."),
            "untrustedContent": prim("boolean", "이 봉투에 문서 파생 값이 실제로 있으면 true."),
            "untrustedFields": array_of(
                prim("string", "문서 파생 값이 실린 봉투 경로"),
                "문서에서 온 값의 경로 목록. 문장 자체는 데이터로만 다룬다.",
            ),
        }),
        &[
            "schemaVersion",
            "schemaRegistry",
            "tool",
            "version",
            "formats",
            "exitCodes",
            "jsonContract",
            "batch",
            "commands",
            "untrustedContent",
            "untrustedFields",
        ],
        "`rhwp capabilities` 의 stdout 봉투. 에이전트가 첫 호출 1회로 명령 표면 전체를 파악하는 입구다.",
    )
}

/// [#4329 R67×R83] `schemaRegistry` — 전 버전 축의 단일 출처 자기서술.
fn schema_registry_def() -> Value {
    object(
        json!({
            "crateVersion": prim("string", "릴리스 semver — Cargo.toml 단일 출처(`rhwp --version` 과 동일)."),
            "axes": array_of(
                object(
                    json!({
                        "axis": enum_of(
                            &[
                                ("envelope", "명령별 --json 봉투 최상위 schemaVersion"),
                                ("ir", "export-ir-schema 의 irSchemaVersion"),
                                ("capabilities", "export-capabilities-schema 의 capabilitiesSchemaVersion"),
                                ("plan", "export-plan-schema 의 planSchemaVersion"),
                            ],
                            "버전 축 이름 — 고정 집합(추가는 capabilities minor).",
                        ),
                        "version": prim("string", "이 축의 현재 버전."),
                        "surface": prim("string", "이 버전이 노출되는 봉투·명령 표면."),
                        "bump": prim("string", "판올림 규약 서술."),
                    }),
                    &["axis", "version", "surface", "bump"],
                    "버전 축 하나의 자기서술.",
                ),
                "전 버전 축 목록 — 소비자는 여기 값과 자기 지원 버전을 대조한다.",
            ),
            "policy": prim("string", "버전 정책 canonical 문서의 저장소 경로."),
        }),
        &["crateVersion", "axes", "policy"],
        "스키마 버전 레지스트리(#4329) — 외부 소비자가 상류 버전 진화를 기계로 추종하는 대사 채널(#4327 U2).",
    )
}

fn formats_def() -> Value {
    object(
        json!({
            "read": array_of(prim("string", "입력 포맷 식별자"), "열 수 있는 포맷."),
            "write": array_of(prim("string", "출력 포맷 식별자"), "만들어 낼 수 있는 포맷."),
        }),
        &["read", "write"],
        "포맷 표면 — 읽기·쓰기 축을 따로 선언한다 (읽을 수 있다고 쓸 수 있는 것이 아니다).",
    )
}

fn exit_codes_def() -> Value {
    object(
        json!({
            "0": prim("string", "성공"),
            "1": prim("string", "런타임 실패 (읽기·파싱·렌더·쓰기)"),
            "2": prim("string", "사용법 오류 (인자 없음, 알 수 없는 옵션/명령)"),
            "3": prim("string", "검증 단언 실패 (--verify IR 차이, run 계획 assertions)"),
            "4": prim("string", "--verify-pages 페이지 수 불일치"),
        }),
        &["0", "1", "2"],
        "종료 코드 사전 — 키는 십진 코드 문자열, 값은 그 코드의 뜻. 3·4 는 검증 게이트를 가진 명령에만 나타난다.",
    )
}

fn json_contract_def() -> Value {
    object(
        json!({
            "stdout": prim("string", "성공 시 stdout 규약 (데이터만, 진단은 stderr)"),
            "failure": prim("string", "실패 시 stdout 규약 (단건은 0바이트)"),
            "schemaPolicy": prim("string", "필드 진화 규약 (추가 허용, 변경·삭제는 schemaVersion 범프)"),
            "textSecurity": r("TextSecurityContract"),
            "provenance": r("ProvenanceContract"),
        }),
        &["stdout", "failure", "schemaPolicy"],
        "`--json` 계약의 전역 규약 — 개별 명령 봉투보다 상위에서 stdout·실패·진화를 고정한다.",
    )
}

fn provenance_contract_def() -> Value {
    object(
        json!({
            "fields": array_of(
                prim("string", "출처 표지 필드 이름"),
                "모든 JSON 봉투에 붙는 문서 파생값 표지 필드.",
            ),
            "meaning": prim("string", "표지 필드의 보안 의미"),
            "map": prim("string", "명령별 출처 지도 조회 방법"),
            "policy": prim("string", "표지 부착 정책"),
        }),
        &["fields", "meaning", "map", "policy"],
        "문서에서 온 값과 엔진이 만든 값을 구분하는 출처 표지 계약.",
    )
}

fn text_security_def() -> Value {
    object(
        json!({
            "field": prim("string", "봉투에서 이 진단이 실리는 필드 이름"),
            "kinds": array_of(prim("string", "탐지 종류 식별자"), "탐지하는 문자열 위험 종류."),
            "policy": prim("string", "처리 방침 (보고 전용 — 문서 문자열을 수정하지 않는다)"),
            "status": array_of(prim("string", "상태 값"), "보고 상태 값 목록."),
            "surfaces": array_of(prim("string", "노출 지점"), "이 진단이 실리는 명령·플래그 목록."),
        }),
        &["field", "kinds", "policy"],
        "문자열 보안 진단 계약 — 혼용 문자·양방향 제어문자 등을 보고만 하고 고치지 않는다는 선언.",
    )
}

fn batch_contract_def() -> Value {
    object(
        json!({
            "subcommands": array_of(prim("string", "batch 하위 명령 이름"), "batch 로 돌릴 수 있는 축."),
            "flags": array_of(prim("string", "플래그 이름"), "batch 축 전체가 받는 플래그."),
            "input": prim("string", "stdin 입력 규약 (한 줄당 경로 하나)"),
            "output": prim("string", "산출물 규약 (파일을 쓰는 축과 목적지)"),
            "ordering": prim("string", "출력 순서 규약 (stdin 입력 순서 보존)"),
            "exitAggregation": prim("string", "전건 처리 결과를 단일 종료 코드로 접는 규칙"),
            "authentication": prim("string", "암호 문서 자격증명 전달 지원 여부"),
            "mcp": r("BatchMcpContract"),
        }),
        &["subcommands", "flags"],
        "batch 축 계약 — 명령별 항목(commands[batch])과 달리 축 전체의 입출력·순서·집계 규약을 담는다.",
    )
}

fn batch_mcp_contract_def() -> Value {
    object(
        json!({
            "available": array_of(prim("string", "MCP 로 노출되는 batch 축"), "hwp_batch 계열이 받는 축."),
            "excluded": object(
                json!({}),
                &[],
                "MCP 에서 제외한 축 → 제외 사유. 키는 축 이름이라 고정 목록이 없다.",
            ),
        }),
        &["available"],
        "batch 축의 MCP 노출 범위 — CLI 에만 있는 축을 사유와 함께 밝힌다.",
    )
}

fn command_def() -> Value {
    object(
        json!({
            "name": prim("string", "명령 이름. `rhwp <name>` 으로 호출한다."),
            "category": r("CommandCategory"),
            "summary": prim("string", "한 줄 요약 (사람·에이전트 공용)"),
            "json": prim("boolean", "`--json` 기계 계약을 가지면 true. 없으면 키 자체가 없다."),
            "batch": prim("boolean", "`batch` 축으로도 돌릴 수 있으면 true."),
            "flags": array_of(prim("string", "플래그 이름"), "이 명령이 받는 플래그. json 명령에만 붙는다."),
            "recordFields": array_of(
                prim("string", "봉투 최상위 필드 이름"),
                "`--json` 봉투가 싣는 필드 목록. 코드 생성기의 반환 타입 원천이다.",
            ),
            // [#3884 G4] 하위 명령 자기서술 — 부모 명령(edit·inspect)에만 붙는다.
            "subcommands": array_of(
                object(
                    json!({
                        "name": prim("string", "하위 명령 이름. `rhwp <부모> <이름>` 으로 호출한다."),
                        "summary": prim("string", "하위 한 줄 요약 — `--search` 매칭 대상"),
                    }),
                    &["name", "summary"],
                    "하위 명령 하나의 자기서술.",
                ),
                "하위 명령 목록. 선언 ↔ 디스패치 대조는 tests/capabilities_subcommands_contract.rs.",
            ),
            "requiresFeature": prim("string", "이 명령이 요구하는 빌드 feature. 게이트된 명령에만 붙는다."),
            "available": prim("boolean", "현재 바이너리에서 실제로 쓸 수 있는가 (게이트된 명령에만 붙는다)."),
        }),
        &["name", "category", "summary"],
        "명령 하나의 자기서술. json·batch·flags·recordFields 는 기계 계약을 가진 명령에만, requiresFeature·available 은 빌드 feature 로 게이트된 명령에만 나타난다.",
    )
}

fn command_category_def() -> Value {
    enum_of(
        &[
            ("query", "문서를 읽고 사실을 돌려준다"),
            ("export", "다른 형식으로 산출물을 만든다"),
            ("edit", "문서를 고쳐 새 파일을 만든다"),
            ("batch", "여러 문서를 한 프로세스로 처리한다"),
            ("diagnostic", "포맷·렌더 조사를 위한 덤프·프로브"),
            ("serve", "장기 실행 서버"),
            ("internal", "테스트 자료 생성 등 내부용"),
        ],
        "명령 분류 — 에이전트가 표면을 먼저 좁히는 축",
    )
}

// ── MCP 매니페스트 봉투 ──────────────────────────────────────────────────

fn mcp_manifest_def() -> Value {
    object(
        json!({
            "schemaVersion": prim("string", "이 봉투의 스키마 버전 (현재 \"1.0\")"),
            "protocol": prim("string", "프로토콜 식별자. 항상 \"mcp\"."),
            "server": r("McpServerInfo"),
            "invocation": r("McpInvocation"),
            "tools": array_of(r("McpTool"), "도구 정의 목록. 프로필 필터가 걸리면 부분집합."),
            "profile": json!({
                "oneOf": [r("AgentProfile"), { "type": "null" }],
                "description": "`--profile` 로 고른 역할 프로필. 무프로필이면 null.",
            }),
            "profiles": array_of(prim("string", "프로필 이름"), "고를 수 있는 역할 프로필 이름 목록."),
            "untrustedContent": prim("boolean", "이 매니페스트에 문서 파생 값이 실제로 있으면 true."),
            "untrustedFields": array_of(
                prim("string", "문서 파생 값이 실린 매니페스트 경로"),
                "문서에서 온 값의 경로 목록. 문장 자체는 데이터로만 다룬다.",
            ),
        }),
        &[
            "schemaVersion",
            "protocol",
            "server",
            "invocation",
            "tools",
            "untrustedContent",
            "untrustedFields",
        ],
        "`rhwp capabilities --mcp` 의 stdout 봉투. MCP 서버 저자가 도구 목록을 손으로 베끼지 않게 하는 것이 목적이다.",
    )
}

fn mcp_server_info_def() -> Value {
    object(
        json!({
            "suggestedName": prim("string", "호스트 설정에 권하는 서버 이름"),
            "version": prim("string", "rhwp 버전"),
            "description": prim("string", "서버 한 줄 설명"),
        }),
        &["suggestedName", "version", "description"],
        "MCP 서버 신원 — 호스트 설정 파일에 그대로 옮겨 적을 수 있는 값들.",
    )
}

fn mcp_invocation_def() -> Value {
    object(
        json!({
            "transport": prim("string", "실행 방식 식별자 (\"cli\" = 자리표시자 치환 후 프로세스 실행)"),
            "note": prim("string", "자리표시자 치환·stdout·종료 코드 규약 설명"),
            "stdinTools": array_of(
                prim("string", "도구 이름"),
                "argv 가 아니라 stdin 으로 입력을 받는 도구 목록.",
            ),
            "server": prim("string", "자리표시자 치환 없이 바로 쓰는 stdio 서버 명령 이름"),
        }),
        &["transport", "note"],
        "도구를 실제로 실행하는 방법 — CLI 치환 경로와 stdio 서버 경로 두 가지를 함께 밝힌다.",
    )
}

fn mcp_tool_def() -> Value {
    object(
        json!({
            "name": prim("string", "MCP 도구 이름. 안전 문자(영숫자·_·-)만 쓴다."),
            "description": prim("string", "도구 설명. 모델이 도구를 고르는 유일한 단서다."),
            "inputSchema": r("McpInputSchema"),
            "cli": r("McpCliBinding"),
            "outputFields": array_of(
                prim("string", "봉투 필드 이름"),
                "이 도구가 돌려주는 JSON 봉투의 최상위 필드 목록.",
            ),
            "annotations": r("McpToolAnnotations"),
        }),
        &["name", "description", "inputSchema", "cli"],
        "MCP 도구 하나의 정의. 호스트가 tools/list 에 그대로 등록할 수 있는 모양이다.",
    )
}

/// [#4220 T3] MCP 표준 ToolAnnotations (2025-03-26 개정판 신설) — 호스트가 실행 전에
/// 도구 성격을 판정하는 힌트. rhwp 는 스펙 기본값에 기대지 않고 4필드를 전부 명시한다.
fn mcp_tool_annotations_def() -> Value {
    object(
        json!({
            "readOnlyHint": prim(
                "boolean",
                "true 면 환경을 바꾸지 않는다 — 파일을 쓰지 않는 조회·stdout 전용 도구. 유도 근거: outputFields 에 산출 경로 필드(output/outputDir)가 없음.",
            ),
            "destructiveHint": prim(
                "boolean",
                "true 면 파괴적 갱신이 가능하다 — 원본을 덮어쓰는 --in-place 축이 있는 도구만. 산출 분리(-o) 도구는 추가형이라 false. readOnlyHint=false 일 때만 의미가 있다.",
            ),
            "idempotentHint": prim(
                "boolean",
                "true 면 같은 인자 재실행이 추가 효과를 내지 않는다 — 무상태 도구는 매번 원본에서 다시 계산하는 결정론 변환이라 전부 true.",
            ),
            "openWorldHint": prim(
                "boolean",
                "true 면 외부 개방 세계(네트워크 등)와 상호작용한다. rhwp 도구는 로컬 파일만 다루므로 전부 false.",
            ),
        }),
        &[
            "readOnlyHint",
            "destructiveHint",
            "idempotentHint",
            "openWorldHint",
        ],
        "MCP 표준 tool annotations — tools/list 에 그대로 실리는 도구 성격 힌트. MCP 규약상 신뢰할 수 없는 서버의 힌트는 참고용이다(클라이언트 확인 대체 불가).",
    )
}

fn mcp_input_schema_def() -> Value {
    object(
        json!({
            "type": prim("string", "항상 \"object\" — MCP 도구 입력은 이름 있는 인자 묶음이다."),
            "properties": object(
                json!({}),
                &[],
                "인자 이름 → JSON Schema 조각. 키가 인자 이름이라 고정 목록이 없다.",
            ),
            "required": array_of(
                prim("string", "인자 이름"),
                "필수 인자 이름 목록. 필수가 없어도 빈 배열로 반드시 선언한다 — 소비자가 \"필수 없음\"과 \"선언 누락\"을 구분할 수 있어야 한다.",
            ),
        }),
        &["type", "properties", "required"],
        "도구 입력의 JSON Schema. 이 안의 모든 인자는 cli.args 또는 cli.optionalArgs 에 배선돼 있다.",
    )
}

fn mcp_cli_binding_def() -> Value {
    object(
        json!({
            "command": prim("string", "내려가는 rhwp 명령 이름 (capabilities.commands[].name 과 같은 값)"),
            "args": array_of(
                prim("string", "argv 토큰 또는 {인자이름} 자리표시자"),
                "필수 argv 템플릿. `{name}` 은 inputSchema 의 같은 이름 값으로 치환한다.",
            ),
            "optionalArgs": array_of(r("McpOptionalArg"), "값이 있을 때만 덧붙이는 인자."),
            "passwordStdin": r("McpPasswordStdin"),
        }),
        &["command", "args"],
        "도구 → CLI 배선. 서버는 이 메타데이터만 보고 자식 프로세스를 띄운다.",
    )
}

fn mcp_optional_arg_def() -> Value {
    object(
        json!({
            "when": prim("string", "이 인자를 켜는 inputSchema 속성 이름. false·null 이면 덧붙이지 않는다."),
            "args": array_of(
                prim("string", "argv 토큰 또는 {인자이름} 자리표시자"),
                "덧붙일 argv 조각. 값 없는 불리언 플래그면 플래그 하나만 들어간다.",
            ),
        }),
        &["when", "args"],
        "선택 인자 배선 하나 — presence 플래그와 값 있는 옵션을 같은 모양으로 다룬다.",
    )
}

fn mcp_password_stdin_def() -> Value {
    object(
        json!({
            "argument": prim("string", "비밀번호를 담는 inputSchema 속성 이름"),
            "flag": prim("string", "자식 CLI 에 넘기는 플래그"),
            "format": prim("string", "stdin 전달 형식 (utf8-first-line)"),
        }),
        &["argument", "flag", "format"],
        "비밀번호 전달 계약 — 민감값은 argv 금지, stdin 으로만 넘긴다.",
    )
}

fn agent_profile_def() -> Value {
    object(
        json!({
            "name": prim("string", "프로필 이름"),
            "summary": prim("string", "이 역할이 무엇을 하는지 한 줄 요약"),
            "session": prim("boolean", "세션 도구(문서 열어두기) 표면을 여는가"),
            "sessionTools": json!({
                "oneOf": [
                    array_of(prim("string", "세션 도구 이름"), "허용하는 세션 도구."),
                    { "type": "null" },
                ],
                "description": "허용 세션 도구 목록. 세션을 열지 않는 프로필이면 null.",
            }),
            "recipe": array_of(prim("string", "권장 절차 한 줄"), "이 역할의 권장 호출 순서."),
        }),
        &["name", "summary", "session"],
        "역할 프로필 — 도구 표면을 역할별로 좁혀 모델의 선택지를 줄인다.",
    )
}

// ── 조립 ─────────────────────────────────────────────────────────────────

/// `$defs` 의 정의 수 — 봉투가 규모를 미리 알리는 데 쓴다.
fn definition_count(schema: &Value) -> usize {
    schema
        .get("$defs")
        .and_then(|d| d.as_object())
        .map(|o| o.len())
        .unwrap_or(0)
}

/// `rhwp capabilities` 출력 전체의 JSON Schema.
///
/// 최상위에 `capabilitiesSchemaVersion` 을 덧붙여 소비자가 스키마 자체의 버전을 알 수
/// 있게 한다 (스키마의 스키마 문제를 피하려고 `$defs` 밖 최상위 키로 둔다).
pub fn capabilities_schema() -> Value {
    // 정의가 늘면 json! 매크로 재귀 한도에 걸린다 — 맵으로 조립한다.
    let defs: serde_json::Map<String, Value> = [
        ("Capabilities", capabilities_def()),
        ("SchemaRegistry", schema_registry_def()),
        ("Formats", formats_def()),
        ("ExitCodes", exit_codes_def()),
        ("JsonContract", json_contract_def()),
        ("TextSecurityContract", text_security_def()),
        ("ProvenanceContract", provenance_contract_def()),
        ("BatchContract", batch_contract_def()),
        ("BatchMcpContract", batch_mcp_contract_def()),
        ("Command", command_def()),
        ("CommandCategory", command_category_def()),
    ]
    .into_iter()
    .map(|(name, def)| (name.to_string(), def))
    .collect();

    json!({
        "$schema": SCHEMA_DIALECT,
        // [#4329] $id 의 버전 조각도 레지스트리 상수에서 파생 — 리터럴 산개 금지.
        "$id": format!("https://github.com/edwardkim/rhwp/schema/capabilities/{CAPABILITIES_SCHEMA_VERSION}"),
        "title": "rhwp capabilities",
        "capabilitiesSchemaVersion": CAPABILITIES_SCHEMA_VERSION,
        "description":
            "`rhwp capabilities` 가 내는 명령 표면 자기서술의 공개 계약. \
             진화 규약: 필드 추가 = minor, 의미 변경·삭제 = major.",
        "$ref": "#/$defs/Capabilities",
        "$defs": defs,
    })
}

/// `rhwp capabilities --mcp` 출력 전체의 JSON Schema.
///
/// 명령 표면과 **다른 봉투**라 스키마도 따로 낸다. MCP 호스트 어댑터를 코드 생성하는
/// 소비자는 이쪽만 읽으면 된다.
pub fn mcp_manifest_schema() -> Value {
    let defs: serde_json::Map<String, Value> = [
        ("McpManifest", mcp_manifest_def()),
        ("McpServerInfo", mcp_server_info_def()),
        ("McpInvocation", mcp_invocation_def()),
        ("McpTool", mcp_tool_def()),
        ("McpToolAnnotations", mcp_tool_annotations_def()),
        ("McpInputSchema", mcp_input_schema_def()),
        ("McpCliBinding", mcp_cli_binding_def()),
        ("McpOptionalArg", mcp_optional_arg_def()),
        ("McpPasswordStdin", mcp_password_stdin_def()),
        ("AgentProfile", agent_profile_def()),
    ]
    .into_iter()
    .map(|(name, def)| (name.to_string(), def))
    .collect();

    json!({
        "$schema": SCHEMA_DIALECT,
        // [#4329] $id 의 버전 조각도 레지스트리 상수에서 파생 — 리터럴 산개 금지.
        "$id": format!("https://github.com/edwardkim/rhwp/schema/capabilities-mcp/{CAPABILITIES_SCHEMA_VERSION}"),
        "title": "rhwp MCP tool manifest",
        "capabilitiesSchemaVersion": CAPABILITIES_SCHEMA_VERSION,
        "description":
            "`rhwp capabilities --mcp` 가 내는 MCP 도구 매니페스트의 공개 계약. \
             inputSchema 의 모든 속성은 cli.args 또는 cli.optionalArgs 에 배선돼 있다.",
        "$ref": "#/$defs/McpManifest",
        "$defs": defs,
    })
}

/// `export-capabilities-schema` 봉투 — 두 스키마 본문과 메타를 함께 싣는다.
///
/// `definitionCount` 는 **두 스키마의 정의 수 합계**다. 소비자가 코드 생성 규모를 먼저
/// 가늠하고, 스키마가 통째로 비는 회귀를 한 숫자로 잡는다.
pub fn envelope() -> Value {
    let schema = capabilities_schema();
    let mcp_schema = mcp_manifest_schema();
    let def_count = definition_count(&schema) + definition_count(&mcp_schema);
    json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "capabilitiesSchemaVersion": CAPABILITIES_SCHEMA_VERSION,
        "dialect": SCHEMA_DIALECT,
        "definitionCount": def_count,
        "schema": schema,
        "mcpSchema": mcp_schema,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schemas_have_roots_and_defs() {
        let schema = capabilities_schema();
        assert_eq!(schema["$ref"], "#/$defs/Capabilities");
        assert!(schema["$defs"]["Capabilities"].is_object());
        assert!(schema["$defs"]["Capabilities"]["properties"]["untrustedContent"].is_object());
        assert!(schema["$defs"]["Capabilities"]["properties"]["untrustedFields"].is_object());
        assert!(schema["$defs"]["JsonContract"]["properties"]["provenance"].is_object());
        let mcp = mcp_manifest_schema();
        assert_eq!(mcp["$ref"], "#/$defs/McpManifest");
        assert!(mcp["$defs"]["McpTool"].is_object());
        assert!(mcp["$defs"]["McpManifest"]["properties"]["untrustedContent"].is_object());
        assert!(mcp["$defs"]["McpManifest"]["properties"]["untrustedFields"].is_object());
    }

    #[test]
    fn every_ref_resolves_to_a_definition() {
        // 끊어진 $ref 는 코드 생성기를 즉시 망가뜨린다 — 스키마의 최소 건전성이다.
        for schema in [capabilities_schema(), mcp_manifest_schema()] {
            let defs = schema["$defs"].as_object().expect("$defs");
            let mut missing = Vec::new();
            collect_refs(&schema, &mut |name| {
                if !defs.contains_key(name) {
                    missing.push(name.to_string());
                }
            });
            assert!(missing.is_empty(), "정의되지 않은 참조: {missing:?}");
        }
    }

    #[test]
    fn definitions_are_reachable_from_root() {
        // 아무도 가리키지 않는 정의는 죽은 계약이다.
        for (schema, root) in [
            (capabilities_schema(), "Capabilities"),
            (mcp_manifest_schema(), "McpManifest"),
        ] {
            let defs = schema["$defs"].as_object().expect("$defs");
            let mut referenced = std::collections::HashSet::new();
            referenced.insert(root.to_string());
            collect_refs(&schema, &mut |name| {
                referenced.insert(name.to_string());
            });
            let orphans: Vec<&String> = defs.keys().filter(|k| !referenced.contains(*k)).collect();
            assert!(orphans.is_empty(), "아무도 참조하지 않는 정의: {orphans:?}");
        }
    }

    #[test]
    fn objects_allow_additional_properties() {
        // 추가-전용 진화 계약: 명령이 하나 늘어도 기존 소비자가 깨지면 안 된다.
        for schema in [capabilities_schema(), mcp_manifest_schema()] {
            let defs = schema["$defs"].as_object().expect("$defs");
            for (name, def) in defs {
                if def["type"] == "object" {
                    assert_eq!(
                        def["additionalProperties"], true,
                        "{name} 이 추가 필드를 막고 있다 — capabilities 는 추가-전용 진화다"
                    );
                }
            }
        }
    }

    #[test]
    fn envelope_reports_definition_count() {
        let env = envelope();
        assert_eq!(env["schemaVersion"], "1.0");
        assert_eq!(
            env["capabilitiesSchemaVersion"],
            CAPABILITIES_SCHEMA_VERSION
        );
        let count = env["definitionCount"].as_u64().expect("definitionCount");
        assert!(count >= 12, "정의가 너무 적다: {count}");
        assert!(env["mcpSchema"]["$defs"].is_object());
    }

    /// `$ref` 를 재귀 수집한다.
    fn collect_refs(value: &Value, sink: &mut impl FnMut(&str)) {
        match value {
            Value::Object(map) => {
                for (key, item) in map {
                    if key == "$ref" {
                        if let Some(path) = item.as_str() {
                            if let Some(name) = path.strip_prefix("#/$defs/") {
                                sink(name);
                            }
                        }
                    } else {
                        collect_refs(item, sink);
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    collect_refs(item, sink);
                }
            }
            _ => {}
        }
    }
}
