//! MCP 자기서술 메타데이터의 조립 경계.
//!
//! 기능군 파일은 원래 선언 순서를 유지해 도구를 추가하고, 이 모듈만 공통 계약과
//! 후처리(암호 stdin, annotations, catalog 완전성)를 소유한다.

mod advanced;
mod edit_content;
mod edit_format;
mod edit_structure;
mod exchange;
mod protocol;
mod read;

/// [#3263] `capabilities --mcp` — MCP 도구 정의 생성.
///
/// MCP 서버 저자(및 함수 호출 클라이언트)가 도구 이름·설명·입력 JSON Schema·실행 배선을
/// 손으로 옮겨 적지 않게 한다. `--json` 계약을 가진 명령이 늘면
/// `capabilities_mcp_covers_every_json_command` 가 누락을 잡는다.
pub(crate) fn show_mcp_tools(profile: Option<&'static crate::agent_profiles::AgentProfile>) -> i32 {
    println!("{}", mcp_manifest_value(profile));
    crate::EXIT_OK
}

/// [#3627] 매니페스트 **값** — `capabilities --mcp` 의 stdout 과 `mcp-serve` 의
/// `rhwp://capabilities/mcp` 리소스가 같은 함수를 쓴다. 프로필 필터가 두 곳에
/// 복제되면 자기서술이 tools/list 에 없는 도구를 광고하게 된다.
pub(crate) fn mcp_manifest_value(
    profile: Option<&'static crate::agent_profiles::AgentProfile>,
) -> serde_json::Value {
    let mut tools = mcp_tool_definitions();
    if let Some(p) = profile {
        tools.retain(|t| {
            t["name"]
                .as_str()
                .map(|n| crate::agent_profiles::allows_tool(p, n))
                .unwrap_or(false)
        });
    }

    crate::provenance::marked(
        serde_json::json!({
        "schemaVersion": crate::ENVELOPE_SCHEMA_VERSION,
        "protocol": "mcp",
        "server": {
            "suggestedName": "rhwp",
            "version": rhwp::version(),
            "description": "HWP/HWPX 한국어 문서를 읽고 편집하는 도구 모음",
        },
        "invocation": {
            "transport": "cli",
            "note": "각 도구의 cli.args 에서 {name} 자리표시자를 inputSchema 의 같은 이름 값으로 치환해 실행한다. stdout 은 순수 JSON, 진단은 stderr, 종료 코드는 0/1/2(+ir-diff 차이 3). 자리표시자 치환 없이 바로 쓰려면 `rhwp mcp-serve`(stdio JSON-RPC 서버, #3140)를 실행한다.",
            "stdinTools": MCP_STDIN_TOOLS,
            "server": "mcp-serve",
        },
        "tools": tools,
        "profile": profile.map(|p| serde_json::json!({
            "name": p.name,
            "summary": p.summary,
            "session": crate::agent_profiles::opens_session(p),
            "sessionTools": p.session_tools.map(|t| if t.is_empty() { crate::agent_profiles::ALL_SESSION_TOOLS.to_vec() } else { t.to_vec() }),
            "recipe": p.recipe,
        })),
        "profiles": crate::agent_profiles::names(),
        }),
        "capabilities",
    )
}

/// stdin 으로 경로 목록을 받는 MCP 도구 — `capabilities --mcp` 의 `invocation.stdinTools`
/// 선언과 `mcp-serve` 의 자식 stdin 배선(`run_cli_tool`)이 이 목록 하나를 공유한다.
/// 이 도구들은 `paths` 없이 자식을 띄우면 자식이 서버의 프로토콜 stdin 을 상속해
/// 이후 JSON-RPC 프레임을 파일 경로로 소비하므로, 서버 쪽에서 반드시 선검증한다.
pub(crate) const MCP_STDIN_TOOLS: [&str; 3] =
    ["hwp_batch", "hwp_batch_search", "hwp_batch_extract_data"];

/// [#3787 S4] `inspect unicode --kind` 의 허용값 — 탐지 코어가 단일 출처다.
fn inspect_unicode_kind_enum() -> Vec<String> {
    rhwp::document_core::text_security::DeceptionKind::ALL
        .iter()
        .map(|kind| kind.filter_name().to_string())
        .chain(std::iter::once("all".to_string()))
        .collect()
}

/// `inspect watermark --kind` 의 허용값 — 탐지 코어(MarkKind)가 단일 출처다.
fn inspect_watermark_kind_enum() -> Vec<String> {
    rhwp::document_core::queries::stego_scan::MarkKind::ALL
        .iter()
        .map(|kind| kind.filter_name().to_string())
        .chain(std::iter::once("all".to_string()))
        .collect()
}

/// 문서 경로 하나를 받는 도구의 표준 입력 스키마.
fn path_schema(extra: serde_json::Value) -> serde_json::Value {
    let mut props = serde_json::json!({
        "path": { "type": "string", "description": "HWP/HWPX/HML 문서 경로" }
    });
    if let (Some(p), Some(e)) = (props.as_object_mut(), extra.as_object()) {
        for (k, v) in e {
            p.insert(k.clone(), v.clone());
        }
    }
    serde_json::json!({
        "type": "object",
        "properties": props,
        "required": ["path"],
    })
}

fn tool(
    name: &str,
    description: &str,
    input_schema: serde_json::Value,
    command: &str,
    args_template: serde_json::Value,
    output_fields: &[&str],
) -> serde_json::Value {
    let spec = crate::cli::catalog::find(command)
        .unwrap_or_else(|| panic!("MCP CLI 배선이 catalog에 없습니다: {command}"));
    assert!(spec.mcp, "MCP 비참여 명령을 도구에 배선했습니다: {command}");
    serde_json::json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
        "cli": { "command": spec.name, "args": args_template },
        "outputFields": output_fields,
    })
}

/// 선택 인자는 기본 `cli.args` 뒤에만 덧붙인다. MCP 서버는 이 메타데이터를
/// 해석해 실제 CLI flag를 전달하고, capability 소비자는 생략 가능 여부를 안다.
fn tool_with_optional_args(
    name: &str,
    description: &str,
    input_schema: serde_json::Value,
    command: &str,
    args_template: serde_json::Value,
    optional_args: serde_json::Value,
    output_fields: &[&str],
) -> serde_json::Value {
    let mut definition = tool(
        name,
        description,
        input_schema,
        command,
        args_template,
        output_fields,
    );
    definition["cli"]["optionalArgs"] = optional_args;
    definition
}

fn supports_password_stdin(name: &str) -> bool {
    matches!(
        name,
        "hwp_info"
            | "hwp_digest"
            | "hwp_export_text"
            | "hwp_export_structure"
            | "hwp_ir_diff"
            | "hwp_export_svg"
            | "hwp_export_pdf"
            | "hwp_export_markdown"
            | "hwp_convert_hwpx"
            | "hwp_convert_hwp5"
            | "hwp_split_document"
            | "hwp_export_tables"
            | "hwp_search"
            | "hwp_extract_data"
            | "hwp_fields"
            | "hwp_explain"
            | "hwp_explore"
            | "hwp_inspect_hidden_text"
            | "hwp_inspect_injection"
            | "hwp_inspect_unicode"
            | "hwp_inspect_watermark"
            | "hwp_fill_fields"
            | "hwp_replace_text"
            | "hwp_set_checkbox"
            | "hwp_set_cell"
            | "hwp_set_cell_props"
            | "hwp_set_table_props"
            | "hwp_insert_text_in_cell"
            | "hwp_delete_text_in_cell"
            | "hwp_split_table"
            | "hwp_fit_table"
            | "hwp_resize_table"
            | "hwp_resize_table_cell"
            | "hwp_move_table"
            | "hwp_merge_table"
            | "hwp_apply_para_format_in_cell"
            | "hwp_apply_char_format_in_cell"
            | "hwp_set_column_widths"
            | "hwp_delete_equation"
            | "hwp_set_numbering_restart"
            | "hwp_set_page_def"
            | "hwp_set_section_def"
    )
}

fn add_password_stdin_contract(definition: &mut serde_json::Value) {
    let Some(properties) = definition["inputSchema"]["properties"].as_object_mut() else {
        return;
    };
    properties.insert(
        "password".to_string(),
        serde_json::json!({
            "type": "string",
            "writeOnly": true,
            "description": "암호 문서 비밀번호. MCP 서버는 응답·세션에 저장하지 않고, 무상태 도구에서는 자식 CLI stdin으로만 전달한다."
        }),
    );
    definition["cli"]["passwordStdin"] = serde_json::json!({
        "argument": "password",
        "flag": "--password-stdin",
        "format": "utf8-first-line"
    });
}

fn add_edit_tool(
    tools: &mut Vec<serde_json::Value>,
    name: &str,
    description: &str,
    properties: serde_json::Value,
    required: &[&str],
    args: serde_json::Value,
    optional_args: serde_json::Value,
    output_fields: &[&str],
) {
    tools.push(tool_with_optional_args(
        name,
        description,
        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
        }),
        "edit",
        args,
        optional_args,
        output_fields,
    ));
}

/// [#3263→#3140] MCP 도구 정의의 단일 출처. 기능군 모듈은 기존 순서로 이어 붙인다.
pub(crate) fn mcp_tool_definitions() -> Vec<serde_json::Value> {
    let mut tools = Vec::new();
    read::extend(&mut tools);
    exchange::extend(&mut tools);
    edit_content::extend(&mut tools);
    edit_structure::extend(&mut tools);
    edit_format::extend(&mut tools);
    protocol::extend(&mut tools);
    advanced::extend(&mut tools);

    for definition in &mut tools {
        if definition["name"]
            .as_str()
            .is_some_and(supports_password_stdin)
        {
            add_password_stdin_contract(definition);
        }
    }
    // [#4220 T3] 기존 선언에서 MCP 표준 tool annotations를 유도한다.
    for definition in &mut tools {
        definition["annotations"] = derive_mcp_tool_annotations(definition);
    }
    let wired_commands: std::collections::BTreeSet<&str> = tools
        .iter()
        .filter_map(|definition| definition["cli"]["command"].as_str())
        .collect();
    for spec in crate::cli::catalog::commands()
        .iter()
        .filter(|spec| spec.mcp)
    {
        assert!(
            wired_commands.contains(spec.name),
            "catalog MCP 참여 명령에 도구가 없습니다: {}",
            spec.name
        );
    }
    tools
}

/// [#4220 T3] MCP 표준 `annotations` 값 하나 (2025-03-26 개정판 신설 ToolAnnotations,
/// 2025-06-18 유지 — schema.ts 의 readOnlyHint/destructiveHint/idempotentHint/openWorldHint).
///
/// 스펙 기본값(readOnlyHint=false, destructiveHint=true, idempotentHint=false,
/// openWorldHint=true)에 기대지 않고 네 필드를 전부 명시한다 — inputSchema.required 를
/// 빈 배열이라도 반드시 선언하는 것과 같은 이유로, 소비자가 "선언 누락"과 "기본값
/// 의도"를 구분할 수 있어야 한다.
///
/// `openWorldHint` 는 전 도구 공통 false 다: rhwp 도구는 로컬 파일만 다루며
/// 네트워크 등 외부 개방 세계에 닿는 축이 없다.
pub(crate) fn mcp_annotations(
    read_only: bool,
    destructive: bool,
    idempotent: bool,
) -> serde_json::Value {
    serde_json::json!({
        "readOnlyHint": read_only,
        "destructiveHint": destructive,
        "idempotentHint": idempotent,
        "openWorldHint": false,
    })
}

/// [#4220 T3] 무상태 도구 하나의 annotations 유도 — 근거는 그 도구 자신의 선언이다.
///
/// - `readOnlyHint`: 봉투 `outputFields` 에 산출 경로 필드(`output`/`outputDir`)가
///   없으면 true. 파일을 쓰지 않는 도구는 환경을 바꾸지 않는다 — 조회(query)와
///   stdout 전용 export(hwp_export_text·hwp_export_tables 등)가 여기 속한다.
///   `hwp_table_to_csv` 처럼 출력이 선택인 도구는 "쓸 수 있다"는 이유로 false 다
///   (힌트는 안전 방향으로 보수적이어야 한다).
/// - `destructiveHint`: cli 배선에 `--in-place` 축이 있을 때만 true. 그 밖의 쓰기는
///   전부 산출 분리(-o) 원칙의 추가형(additive)이다 — 원본 문서를 덮지 않는다
///   (redact 의 원본 보호 exit 2, export 계열의 같은 경로 거부가 그 증거다).
/// - `idempotentHint`: 무상태 도구는 전부 true — 매 호출이 같은 원본에서 다시
///   계산하는 결정론 변환이라, 같은 인자 재실행은 같은 산출을 다시 쓸 뿐 추가
///   효과가 없다(세션 편집 누적과 대비되는 성질이다 — mcp_serve 참고).
fn derive_mcp_tool_annotations(definition: &serde_json::Value) -> serde_json::Value {
    let writes_files = definition["outputFields"].as_array().is_some_and(|fields| {
        fields
            .iter()
            .any(|f| matches!(f.as_str(), Some("output" | "outputDir")))
    });
    let in_place = cli_wiring_has_flag(&definition["cli"], "--in-place");
    mcp_annotations(!writes_files, in_place, true)
}

/// cli 배선(필수 `args` + `optionalArgs[].args`)에 특정 플래그가 있는가.
fn cli_wiring_has_flag(cli: &serde_json::Value, flag: &str) -> bool {
    let args_contain = |args: &serde_json::Value| {
        args.as_array()
            .is_some_and(|a| a.iter().any(|t| t.as_str() == Some(flag)))
    };
    args_contain(&cli["args"])
        || cli["optionalArgs"]
            .as_array()
            .is_some_and(|opts| opts.iter().any(|o| args_contain(&o["args"])))
}

// [#3694] capabilities 명령 목록의 단일 출처 — 자기서술과 did-you-mean 이 공유한다.
