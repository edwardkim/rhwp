//! [#3907 O1] `export-ontology` — 자기서술 유도 온톨로지의 계약.
//!
//! 여기서 검증하는 것은 "JSON-LD 가 나오나"가 아니라 **유도가 전수인가**다.
//! 온톨로지의 논지는 손 나열 상수 0(원천 선언이 바뀌면 함께 바뀐다)이므로,
//! 계약도 손 목록을 두지 않는다 — 같은 바이너리의 `export-ir-schema`·
//! `capabilities`·`export-provenance-map` 산출을 원천으로 삼아 기계 대조한다.
//! 하나라도 빠지면 빠진 이름을 열거하며 red 다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::{HashMap, HashSet};
use std::process::{Command, Output};

use serde_json::Value;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], o: &Output) -> String {
    format!(
        "명령: rhwp {}\nexit: {:?}\nstderr:\n{}",
        args.join(" "),
        o.status.code(),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn json_of(args: &[&str]) -> Value {
    let out = run(args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(args, &out));
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, &out)
        )
    })
}

fn envelope() -> Value {
    json_of(&["export-ontology", "--json"])
}

fn graph_of(envelope: &Value) -> &Vec<Value> {
    envelope["ontology"]["@graph"]
        .as_array()
        .expect("ontology.@graph 배열")
}

/// `@type` 에 해당 타입이 있는가 (문자열 하나든 배열이든).
fn typed(node: &Value, ty: &str) -> bool {
    match &node["@type"] {
        Value::String(s) => s == ty,
        Value::Array(list) => list.iter().any(|t| t == ty),
        _ => false,
    }
}

// ── ① 전수 포섭 — 클래스 ⊇ IR 정의 전부, 행위 ⊇ json 명령 전부 ─────────────

#[test]
fn every_ir_definition_appears_as_a_class() {
    let env = envelope();
    let classes: HashSet<&str> = graph_of(&env)
        .iter()
        .filter(|n| typed(n, "rdfs:Class"))
        .filter_map(|n| n["rdfs:label"].as_str())
        .collect();

    let ir = json_of(&["export-ir-schema", "--bare"]);
    let defs: Vec<&String> = ir["$defs"].as_object().expect("$defs").keys().collect();
    assert!(
        defs.len() >= 41,
        "IR 정의 파싱이 거의 0건이면 이 가드는 공허하게 통과한다: {}",
        defs.len()
    );

    let missing: Vec<&&String> = defs
        .iter()
        .filter(|name| !classes.contains(name.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "IR 스키마 정의인데 온톨로지 클래스에 없는 것: {missing:?}\n\
         유도(src/ontology.rs push_ir_nodes)가 $defs 전수를 돌지 않고 있습니다."
    );
}

#[test]
fn every_json_command_appears_as_an_action() {
    let env = envelope();
    let actions: HashSet<&str> = graph_of(&env)
        .iter()
        .filter(|n| typed(n, "rhwp:Action"))
        .filter(|n| {
            n["@id"]
                .as_str()
                .is_some_and(|id| id.starts_with("rhwp:cmd/"))
        })
        .filter_map(|n| n["rdfs:label"].as_str())
        .collect();

    let cap = json_of(&["capabilities"]);
    let declared: Vec<&str> = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter(|c| c["json"] == true)
        .filter_map(|c| c["name"].as_str())
        .collect();
    assert!(
        declared.len() >= 20,
        "capabilities 파싱이 거의 0건이면 이 가드는 공허하게 통과한다: {declared:?}"
    );

    let missing: Vec<&&str> = declared
        .iter()
        .filter(|name| !actions.contains(**name))
        .collect();
    assert!(
        missing.is_empty(),
        "--json 계약 명령인데 온톨로지 행위에 없는 것: {missing:?}\n\
         유도(src/ontology.rs push_command_nodes)가 commands[] 전수를 돌지 않고 있습니다."
    );

    // 자기 포섭 — 이 명령 자신도 유도 결과에 나타나야 한다(자기서술의 완결성).
    assert!(actions.contains("export-ontology"), "{actions:?}");
}

#[test]
fn every_mcp_tool_appears_as_an_action_linked_to_a_real_command() {
    let env = envelope();
    let graph = graph_of(&env);
    let tool_nodes: HashMap<&str, &Value> = graph
        .iter()
        .filter(|n| {
            n["@id"]
                .as_str()
                .is_some_and(|id| id.starts_with("rhwp:tool/"))
        })
        .filter_map(|n| n["rdfs:label"].as_str().map(|l| (l, n)))
        .collect();
    let command_ids: HashSet<&str> = graph
        .iter()
        .filter(|n| {
            n["@id"]
                .as_str()
                .is_some_and(|id| id.starts_with("rhwp:cmd/"))
        })
        .filter_map(|n| n["@id"].as_str())
        .collect();

    let manifest = json_of(&["capabilities", "--mcp"]);
    let tools = manifest["tools"].as_array().expect("tools");
    assert!(
        !tools.is_empty(),
        "도구가 0건이면 이 가드는 공허하게 통과한다"
    );
    for tool in tools {
        let name = tool["name"].as_str().expect("도구 이름");
        let node = tool_nodes
            .get(name)
            .unwrap_or_else(|| panic!("MCP 도구 {name} 이 온톨로지 행위에 없습니다"));
        assert!(typed(node, "rhwp:Action"), "{name}: {node}");
        assert!(typed(node, "schema:Action"), "{name}: {node}");
        // 배선 유도 — 도구가 내려가는 CLI 명령 노드가 실제로 그래프에 있어야 한다.
        let implements = node["rhwp:implementsCommand"]["@id"]
            .as_str()
            .unwrap_or_else(|| panic!("{name}: implementsCommand 누락: {node}"));
        assert!(
            command_ids.contains(implements),
            "{name} 이 그래프에 없는 명령 {implements} 를 가리킵니다"
        );
    }
}

// ── ② 유도 정합 — 속성의 도메인이 실제 IR 정의를 가리킨다 ──────────────────

#[test]
fn property_domains_and_ranges_resolve_to_real_definitions() {
    let env = envelope();
    let graph = graph_of(&env);
    let ir = json_of(&["export-ir-schema", "--bare"]);
    let defs: HashSet<String> = ir["$defs"]
        .as_object()
        .expect("$defs")
        .keys()
        .map(|k| format!("rhwp:ir/{k}"))
        .collect();

    let mut checked = 0usize;
    for node in graph {
        if !typed(node, "rdf:Property") {
            continue;
        }
        let id = node["@id"].as_str().expect("@id");
        let domain = node["rdfs:domain"]["@id"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: rdfs:domain 누락"));
        assert!(
            defs.contains(domain),
            "{id} 의 도메인 {domain} 이 실제 IR 정의에 없습니다"
        );
        // 레인지가 IR 클래스를 가리키면 그 클래스도 실존해야 한다 (xsd:*·rdfs:Resource 는 제외).
        if let Some(range) = node["rdfs:range"]["@id"].as_str() {
            if range.starts_with("rhwp:ir/") {
                assert!(
                    defs.contains(range),
                    "{id} 의 레인지 {range} 가 실제 IR 정의에 없습니다"
                );
            }
        }
        checked += 1;
    }
    assert!(
        checked >= 100,
        "검사한 속성이 {checked}건뿐입니다 — 유도가 비었습니다"
    );
}

// ── ③ 신뢰 술어 ⊇ 출처 지도의 untrusted 경로 전수 ─────────────────────────

#[test]
fn trust_predicates_cover_every_untrusted_path_in_the_provenance_map() {
    let env = envelope();
    let actions: HashMap<&str, &Value> = graph_of(&env)
        .iter()
        .filter(|n| {
            n["@id"]
                .as_str()
                .is_some_and(|id| id.starts_with("rhwp:cmd/"))
        })
        .filter_map(|n| n["rdfs:label"].as_str().map(|l| (l, n)))
        .collect();

    let map = json_of(&["export-provenance-map", "--json"]);
    let commands = map["commands"].as_object().expect("commands");
    let mut checked_paths = 0usize;
    for (name, entry) in commands {
        let node = actions
            .get(name.as_str())
            .unwrap_or_else(|| panic!("출처 지도의 명령 {name} 이 온톨로지 행위에 없습니다"));
        let declared: Vec<&str> = node["rhwp:untrustedFields"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: rhwp:untrustedFields 누락: {node}"))
            .iter()
            .filter_map(Value::as_str)
            .collect();
        for path in entry["untrusted"].as_array().expect("untrusted") {
            let path = path.as_str().expect("경로 문자열");
            assert!(
                declared.contains(&path),
                "{name} 의 문서 파생 경로 {path} 가 온톨로지 신뢰 술어에 없습니다: {declared:?}"
            );
            checked_paths += 1;
        }
    }
    assert!(
        checked_paths >= 30,
        "대조한 신뢰 경로가 {checked_paths}건뿐입니다 — 지도 파싱이 비었습니다"
    );
}

// ── ④ JSON-LD 형식 계약 — @context·@graph 존재, @id 유일 ──────────────────

#[test]
fn jsonld_shape_context_graph_and_unique_ids() {
    let env = envelope();
    assert_eq!(env["schemaVersion"], "1.0", "{env}");
    let context = env["ontology"]["@context"]
        .as_object()
        .expect("@context 객체");
    // 자체 어휘 접두어와 표준 어휘 접두어가 선언돼 있어야 한다 — 접두어 없는
    // `rhwp:`·`rdfs:` 키는 JSON-LD 소비자에게 무의미한 문자열이 된다.
    for prefix in ["rhwp", "rdf", "rdfs", "xsd", "schema"] {
        assert!(
            context.get(prefix).is_some_and(Value::is_string),
            "@context 에 {prefix} 접두어가 없습니다: {context:?}"
        );
    }

    let graph = graph_of(&env);
    assert!(!graph.is_empty(), "@graph 가 비었습니다");
    let mut seen: HashSet<&str> = HashSet::new();
    for node in graph.iter() {
        let id = node["@id"]
            .as_str()
            .unwrap_or_else(|| panic!("@id 없는 노드: {node}"));
        assert!(seen.insert(id), "@id 중복: {id}");
    }

    // 봉투의 유도 통계가 그래프 실물과 일치해야 한다.
    let classes = graph.iter().filter(|n| typed(n, "rdfs:Class")).count();
    let properties = graph.iter().filter(|n| typed(n, "rdf:Property")).count();
    let actions = graph.iter().filter(|n| typed(n, "rhwp:Action")).count();
    assert_eq!(env["classCount"].as_u64(), Some(classes as u64), "{env}");
    assert_eq!(
        env["propertyCount"].as_u64(),
        Some(properties as u64),
        "{env}"
    );
    assert_eq!(env["actionCount"].as_u64(), Some(actions as u64), "{env}");
    assert!(classes >= 41, "IR 타입 41정의보다 적습니다: {classes}");

    // 문서를 열지 않는 명령 — 출처 표지는 false 를 명시한다.
    assert_eq!(env["untrustedContent"], false, "{env}");
    assert_eq!(env["untrustedFields"], serde_json::json!([]), "{env}");
}

#[test]
fn bare_flag_emits_jsonld_body_only() {
    let args = ["export-ontology", "--bare"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v: Value = serde_json::from_slice(&out.stdout).expect("JSON-LD");
    assert!(v["@context"].is_object(), "{v}");
    assert!(v["@graph"].is_array(), "{v}");
    // 봉투 필드가 섞이면 안 된다 — --bare 는 RDF 도구에 바로 먹이는 본문이다.
    assert!(v["schemaVersion"].is_null(), "봉투가 섞였다: {v}");
    assert!(v["classCount"].is_null(), "봉투가 섞였다: {v}");
    assert!(v["untrustedContent"].is_null(), "봉투가 섞였다: {v}");
}

// ── ⑤ 실패 규약 — 사용법 오류 exit 2·stdout 0B, 쓰기 실패 exit 1 ──────────

#[test]
fn unknown_option_is_usage_error_with_empty_stdout() {
    let args = ["export-ontology", "--없는옵션"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(2), "{}", describe(&args, &out));
    assert!(out.stdout.is_empty(), "{}", describe(&args, &out));
}

#[test]
fn missing_output_path_is_usage_error_with_empty_stdout() {
    let args = ["export-ontology", "-o"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(2), "{}", describe(&args, &out));
    assert!(out.stdout.is_empty(), "{}", describe(&args, &out));
}

#[test]
fn unwritable_output_is_runtime_error() {
    let bad = std::path::PathBuf::from("없는폴더-ontology")
        .join("깊은")
        .join("경로.jsonld");
    let bad_str = bad.to_str().unwrap();
    let args = ["export-ontology", "-o", bad_str];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&args, &out));
}

#[test]
fn output_file_keeps_stdout_machine_readable() {
    let target = std::env::temp_dir().join(format!(
        "rhwp-ontology-{}-{}.jsonld",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let target_str = target.to_str().unwrap().to_string();
    let args = ["export-ontology", "-o", &target_str, "--json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));

    let v: Value = serde_json::from_slice(&out.stdout).expect("봉투");
    assert_eq!(v["output"], target_str, "{v}");
    assert!(v["bytes"].as_u64().unwrap_or(0) > 0, "{v}");
    assert!(v["ontologyVersion"].is_string(), "{v}");

    let written = std::fs::read_to_string(&target).expect("파일");
    let parsed: Value = serde_json::from_str(&written).expect("파일 JSON");
    assert!(parsed["ontology"]["@graph"].is_array(), "{parsed}");

    let _ = std::fs::remove_file(&target);
}

// ── 표면 배선 — capabilities·MCP 등재가 실물과 정합한다 ────────────────────

#[test]
fn capabilities_declares_export_ontology() {
    let cap = json_of(&["capabilities"]);
    let cmd = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "export-ontology")
        .expect("export-ontology 수록");
    assert_eq!(cmd["json"], true, "{cmd}");
    let fields = cmd["recordFields"].as_array().expect("recordFields");
    for want in [
        "schemaVersion",
        "ontology",
        "classCount",
        "propertyCount",
        "actionCount",
    ] {
        assert!(fields.iter().any(|f| f == want), "{want} 누락: {cmd}");
    }
    // 선언한 플래그는 실제로 수용돼야 한다 — 선언만 있고 안 받으면 거짓 계약이다.
    let sink = std::env::temp_dir().join(format!("rhwp-ontflag-{}.jsonld", std::process::id()));
    let sink_str = sink.to_str().unwrap().to_string();
    for flag in cmd["flags"].as_array().expect("flags") {
        let flag = flag.as_str().expect("flag 문자열");
        let args: Vec<&str> = if flag == "-o" {
            vec!["export-ontology", "-o", &sink_str]
        } else {
            vec!["export-ontology", flag]
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

#[test]
fn mcp_manifest_registers_the_tool() {
    let manifest = json_of(&["capabilities", "--mcp"]);
    let tool = manifest["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_export_ontology")
        .expect("hwp_export_ontology 등재");
    assert_eq!(tool["cli"]["command"], "export-ontology", "{tool}");
    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    // 필수 인자가 없어도 빈 배열을 선언해야 한다 — 부재와 "필수 없음"은 다르다.
    assert_eq!(
        tool["inputSchema"]["required"].as_array().map(Vec::len),
        Some(0),
        "{tool}"
    );
    // 선언한 입력은 전부 CLI 에 배선돼야 한다 (optionalArgs 규약).
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
