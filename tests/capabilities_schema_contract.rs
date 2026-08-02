//! [#3776] `export-capabilities-schema` — 명령 표면 코드 생성의 단일 출처 계약 (M19).
//!
//! 이 스키마가 깨지면 바인딩 세대가 통째로 잘못된 래퍼를 만든다. 그래서 여기서
//! 검증하는 것은 "JSON 이 나오나"가 아니라 **스키마로서 성립하나**, 그리고 **실제
//! capabilities 출력과 어긋나지 않나**다 — 끊어진 참조·고아 정의·닫힌 객체·미선언
//! 필드는 전부 실패다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp")
}

fn describe(args: &[&str], o: &Output) -> String {
    format!(
        "명령: rhwp {}\nexit: {:?}\nstderr:\n{}",
        args.join(" "),
        o.status.code(),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn schema_envelope() -> serde_json::Value {
    let args = ["export-capabilities-schema"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    serde_json::from_slice(&output.stdout).expect("봉투 JSON")
}

/// 봉투에 실린 두 스키마 본문 — 명령 표면과 MCP 매니페스트는 서로 다른 봉투다.
fn both_schemas(envelope: &serde_json::Value) -> [(&'static str, serde_json::Value); 2] {
    [
        ("schema", envelope["schema"].clone()),
        ("mcpSchema", envelope["mcpSchema"].clone()),
    ]
}

/// 봉투 스키마 — 소비자가 버전과 방언을 먼저 확인할 수 있어야 한다.
#[test]
fn envelope_declares_version_and_dialect() {
    let v = schema_envelope();
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["capabilitiesSchemaVersion"].is_string(), "{v}");
    assert!(
        v["dialect"]
            .as_str()
            .unwrap_or("")
            .contains("json-schema.org"),
        "방언을 명시해야 소비자가 파서를 고를 수 있다: {v}"
    );
    assert!(v["definitionCount"].as_u64().unwrap_or(0) >= 12, "{v}");
    assert!(v["schema"].is_object(), "{v}");
    assert!(v["mcpSchema"].is_object(), "{v}");
}

/// capabilities 스키마 버전은 봉투 버전과 **분리**돼 있다 — 명령별 봉투와 전역 표면
/// 계약은 따로 진화한다.
#[test]
fn capabilities_schema_version_is_independent_of_envelope_version() {
    let v = schema_envelope();
    for (label, schema) in both_schemas(&v) {
        assert_eq!(
            schema["capabilitiesSchemaVersion"], v["capabilitiesSchemaVersion"],
            "{label} 본문과 봉투의 스키마 버전이 어긋난다: {v}"
        );
    }
}

/// 두 스키마의 루트가 각자의 봉투 정의를 가리킨다.
#[test]
fn schema_roots_point_at_their_envelopes() {
    let v = schema_envelope();
    assert_eq!(v["schema"]["$ref"], "#/$defs/Capabilities", "{v}");
    assert!(v["schema"]["$defs"]["Capabilities"].is_object(), "{v}");
    assert_eq!(v["mcpSchema"]["$ref"], "#/$defs/McpManifest", "{v}");
    assert!(v["mcpSchema"]["$defs"]["McpTool"].is_object(), "{v}");
}

/// 끊어진 `$ref` 는 코드 생성기를 즉시 망가뜨린다.
#[test]
fn every_reference_resolves() {
    let v = schema_envelope();
    for (label, schema) in both_schemas(&v) {
        let defs = schema["$defs"].as_object().expect("$defs");
        let mut missing = Vec::new();
        collect_refs(&schema, &mut |name| {
            if !defs.contains_key(name) {
                missing.push(name.to_string());
            }
        });
        assert!(
            missing.is_empty(),
            "{label}: 정의되지 않은 참조: {missing:?}"
        );
    }
}

/// 아무도 가리키지 않는 정의는 죽은 계약이다.
#[test]
fn no_orphan_definitions() {
    let v = schema_envelope();
    for (label, root) in [("schema", "Capabilities"), ("mcpSchema", "McpManifest")] {
        let schema = &v[label];
        let defs = schema["$defs"].as_object().expect("$defs");
        let mut referenced: HashSet<String> = HashSet::new();
        referenced.insert(root.to_string());
        collect_refs(schema, &mut |name| {
            referenced.insert(name.to_string());
        });
        let orphans: Vec<&String> = defs.keys().filter(|k| !referenced.contains(*k)).collect();
        assert!(
            orphans.is_empty(),
            "{label}: 아무도 참조하지 않는 정의: {orphans:?}"
        );
    }
}

/// capabilities 는 **추가-전용 진화**다 — 객체가 추가 필드를 막으면 명령 하나 늘 때마다
/// 모든 바인딩이 동시에 깨진다.
#[test]
fn objects_permit_additional_properties() {
    let v = schema_envelope();
    for (label, schema) in both_schemas(&v) {
        let defs = schema["$defs"].as_object().expect("$defs");
        for (name, def) in defs {
            if def["type"] == "object" {
                assert_eq!(
                    def["additionalProperties"], true,
                    "{label}.{name} 이 추가 필드를 막고 있다"
                );
            }
        }
    }
}

/// 바인딩 생성기가 반드시 읽어야 하는 표면 타입이 스키마에 있어야 한다.
#[test]
fn schema_covers_command_surface_types() {
    let v = schema_envelope();
    let defs = v["schema"]["$defs"].as_object().expect("$defs");
    for required in [
        "Capabilities",
        "Command",
        "CommandCategory",
        "ExitCodes",
        "JsonContract",
        "BatchContract",
        "Formats",
    ] {
        assert!(
            defs.contains_key(required),
            "{required} 정의 누락: {defs:?}"
        );
    }
    let mcp_defs = v["mcpSchema"]["$defs"].as_object().expect("$defs");
    for required in [
        "McpManifest",
        "McpTool",
        "McpInputSchema",
        "McpCliBinding",
        "McpOptionalArg",
    ] {
        assert!(
            mcp_defs.contains_key(required),
            "{required} 정의 누락: {mcp_defs:?}"
        );
    }
}

/// 모든 정의에 설명이 달려 있어야 한다 — 생성된 바인딩의 docstring 원천이다.
#[test]
fn every_definition_is_documented() {
    let v = schema_envelope();
    for (label, schema) in both_schemas(&v) {
        let defs = schema["$defs"].as_object().expect("$defs");
        let undocumented: Vec<&String> = defs
            .iter()
            .filter(|(_, d)| d["description"].as_str().unwrap_or("").trim().is_empty())
            .map(|(k, _)| k)
            .collect();
        assert!(
            undocumented.is_empty(),
            "{label}: 설명 없는 정의: {undocumented:?}"
        );
    }
}

/// 스키마가 **실제 출력과 어긋나지 않아야** 한다 — 선언만 맞고 현실과 다르면
/// 생성된 바인딩은 컴파일되지만 런타임에 필드를 못 찾는다.
#[test]
fn schema_matches_live_capabilities_output() {
    let v = schema_envelope();
    let defs = v["schema"]["$defs"].as_object().expect("$defs");
    let caps: serde_json::Value =
        serde_json::from_slice(&run(&["capabilities"]).stdout).expect("capabilities");

    let root = &defs["Capabilities"];
    let declared = root["properties"].as_object().expect("properties");
    let live = caps.as_object().expect("capabilities 객체");
    let undeclared: Vec<&String> = live.keys().filter(|k| !declared.contains_key(*k)).collect();
    assert!(
        undeclared.is_empty(),
        "실제 capabilities 에는 있는데 스키마가 선언하지 않은 최상위 키: {undeclared:?}"
    );
    for required in root["required"].as_array().expect("required") {
        let name = required.as_str().expect("required 항목");
        assert!(
            live.contains_key(name),
            "필수 선언 {name} 이 실제 출력에 없다"
        );
    }

    // 명령 항목: 실제 키가 전부 선언돼 있고, category 는 열거값 안에 있어야 한다.
    let cmd_props = defs["Command"]["properties"]
        .as_object()
        .expect("Command.properties");
    let categories: Vec<&str> = defs["CommandCategory"]["enum"]
        .as_array()
        .expect("CommandCategory.enum")
        .iter()
        .filter_map(|c| c.as_str())
        .collect();
    let commands = caps["commands"].as_array().expect("commands");
    assert!(commands.len() >= 20, "명령이 너무 적다: {}", commands.len());
    for command in commands {
        let obj = command.as_object().expect("명령 객체");
        for key in obj.keys() {
            assert!(
                cmd_props.contains_key(key),
                "Command 스키마가 선언하지 않은 필드 {key}: {command}"
            );
        }
        let category = command["category"].as_str().expect("category");
        assert!(
            categories.contains(&category),
            "CommandCategory 에 없는 분류 {category}: {command}"
        );
    }
}

/// MCP 매니페스트 스키마도 실제 `capabilities --mcp` 출력과 맞아야 한다.
#[test]
fn mcp_schema_matches_live_manifest_output() {
    let v = schema_envelope();
    let defs = v["mcpSchema"]["$defs"].as_object().expect("$defs");
    let manifest: serde_json::Value =
        serde_json::from_slice(&run(&["capabilities", "--mcp"]).stdout).expect("manifest");

    let declared = defs["McpManifest"]["properties"]
        .as_object()
        .expect("properties");
    let live = manifest.as_object().expect("매니페스트 객체");
    let undeclared: Vec<&String> = live.keys().filter(|k| !declared.contains_key(*k)).collect();
    assert!(
        undeclared.is_empty(),
        "실제 매니페스트에는 있는데 스키마가 선언하지 않은 최상위 키: {undeclared:?}"
    );

    let tool_props = defs["McpTool"]["properties"]
        .as_object()
        .expect("McpTool.properties");
    let cli_props = defs["McpCliBinding"]["properties"]
        .as_object()
        .expect("McpCliBinding.properties");
    let tools = manifest["tools"].as_array().expect("tools");
    assert!(
        !tools.is_empty(),
        "도구가 0건이면 이 가드는 공허하게 통과한다"
    );
    for t in tools {
        for key in t.as_object().expect("도구 객체").keys() {
            assert!(
                tool_props.contains_key(key),
                "McpTool 스키마가 선언하지 않은 필드 {key}: {t}"
            );
        }
        for key in t["cli"].as_object().expect("cli 객체").keys() {
            assert!(
                cli_props.contains_key(key),
                "McpCliBinding 스키마가 선언하지 않은 필드 {key}: {t}"
            );
        }
    }
}

/// `--bare` 는 봉투 없이 스키마 본문만 — JSON Schema 도구에 바로 먹인다.
#[test]
fn bare_flag_emits_schema_body_only() {
    let args = ["export-capabilities-schema", "--bare"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("스키마");
    assert!(v["$schema"].is_string(), "{v}");
    assert!(v["$defs"]["Capabilities"].is_object(), "{v}");
    // 봉투 필드가 섞이면 안 된다.
    assert!(v["definitionCount"].is_null(), "봉투가 섞였다: {v}");
    assert!(v["mcpSchema"].is_null(), "봉투가 섞였다: {v}");
}

/// `-o` 로 파일에 쓰고, `--json` 이면 stdout 은 기계 계약을 유지한다.
#[test]
fn output_file_keeps_stdout_machine_readable() {
    let target = std::env::temp_dir().join(format!(
        "rhwp-capschema-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let target_str = target.to_str().unwrap().to_string();
    let args = ["export-capabilities-schema", "-o", &target_str, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("봉투");
    assert_eq!(v["output"], target_str, "{v}");
    assert!(v["bytes"].as_u64().unwrap_or(0) > 0, "{v}");
    assert!(v["capabilitiesSchemaVersion"].is_string(), "{v}");

    let written = std::fs::read_to_string(&target).expect("파일");
    let parsed: serde_json::Value = serde_json::from_str(&written).expect("파일 JSON");
    assert!(parsed["schema"]["$defs"].is_object(), "{parsed}");
    assert!(parsed["mcpSchema"]["$defs"].is_object(), "{parsed}");

    let _ = std::fs::remove_file(&target);
}

/// 알 수 없는 옵션은 사용법 오류(2), stdout 0바이트.
#[test]
fn unknown_option_is_usage_error() {
    let args = ["export-capabilities-schema", "--없는옵션"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

/// `-o` 뒤에 경로가 없으면 사용법 오류.
#[test]
fn missing_output_path_is_usage_error() {
    let args = ["export-capabilities-schema", "-o"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty());
}

/// 쓸 수 없는 경로는 런타임 오류(1) — 사용법 오류와 구분한다.
#[test]
fn unwritable_output_is_runtime_error() {
    let bad: PathBuf = PathBuf::from("없는폴더-capschema")
        .join("깊은")
        .join("경로.json");
    let bad_str = bad.to_str().unwrap().to_string();
    let args = ["export-capabilities-schema", "-o", &bad_str];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
}

/// capabilities 가 이 명령을 json 명령으로 선언한다 (드리프트 가드와 정합).
#[test]
fn capabilities_declares_export_capabilities_schema() {
    let output = run(&["capabilities"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("capabilities");
    let cmd = v["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "export-capabilities-schema")
        .expect("export-capabilities-schema 수록");
    assert_eq!(cmd["json"], true, "{cmd}");
    let fields = cmd["recordFields"].as_array().expect("recordFields");
    for want in [
        "capabilitiesSchemaVersion",
        "definitionCount",
        "schema",
        "mcpSchema",
    ] {
        assert!(fields.iter().any(|f| f == want), "{want} 누락: {cmd}");
    }
    // 선언한 플래그는 실제로 수용돼야 한다 — 선언만 있고 안 받으면 거짓 계약이다.
    let sink = std::env::temp_dir().join(format!("rhwp-capflag-{}.json", std::process::id()));
    let sink_str = sink.to_str().unwrap().to_string();
    for flag in cmd["flags"].as_array().expect("flags") {
        let flag = flag.as_str().expect("flag 문자열");
        let args: Vec<&str> = if flag == "-o" {
            vec!["export-capabilities-schema", "-o", &sink_str]
        } else {
            vec!["export-capabilities-schema", flag]
        };
        let out = run(&args);
        assert_ne!(
            out.status.code(),
            Some(2),
            "선언한 플래그 {flag} 를 실제로 받지 않는다: {}",
            describe(&args, &out)
        );
    }
    let _ = std::fs::remove_file(&sink);
}

/// MCP 도구로도 노출된다 — `--json` 계약 명령은 도구를 가져야 한다는 저장소 규약.
#[test]
fn mcp_manifest_registers_the_tool() {
    let output = run(&["capabilities", "--mcp"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("manifest");
    let tool = v["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_export_capabilities_schema")
        .expect("hwp_export_capabilities_schema 등재");
    assert_eq!(
        tool["cli"]["command"], "export-capabilities-schema",
        "{tool}"
    );
    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    // 필수 인자가 없어도 빈 배열을 선언해야 한다 — 부재와 "필수 없음"은 다르다.
    assert_eq!(
        tool["inputSchema"]["required"].as_array().map(Vec::len),
        Some(0),
        "{tool}"
    );
    // 선언한 입력은 전부 CLI 에 배선돼야 한다.
    let wired: Vec<&str> = tool["cli"]["optionalArgs"]
        .as_array()
        .expect("optionalArgs")
        .iter()
        .filter_map(|o| o["when"].as_str())
        .collect();
    for key in tool["inputSchema"]["properties"]
        .as_object()
        .expect("properties")
        .keys()
    {
        assert!(
            wired.contains(&key.as_str()),
            "{key} 가 배선되지 않았다: {tool}"
        );
    }
}

/// `$ref` 를 재귀 수집한다.
fn collect_refs(value: &serde_json::Value, sink: &mut impl FnMut(&str)) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, item) in map {
                if key == "$ref" {
                    if let Some(name) = item.as_str().and_then(|p| p.strip_prefix("#/$defs/")) {
                        sink(name);
                    }
                } else {
                    collect_refs(item, sink);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_refs(item, sink);
            }
        }
        _ => {}
    }
}
