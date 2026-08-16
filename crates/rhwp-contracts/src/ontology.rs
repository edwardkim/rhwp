//! [#3907 O1] `export-ontology` — 살아있는 자기서술에서 JSON-LD 온톨로지를 기계 유도한다.
//!
//! ## 왜 유도인가
//!
//! 손으로 쓰는 Ontology-as-Code 는 낡는다 — 명령이 하나 늘거나 IR 필드가 하나
//! 바뀌면 온톨로지 문서는 조용히 거짓말이 된다. rhwp 에는 이미 기계가 읽는
//! 자기서술 네 축이 있다:
//!
//! - `ir_schema()` — 문서 모델(타입·필드)의 자기서술
//! - `capabilities` 봉투 — 명령 표면(category·flags·recordFields)의 자기서술
//! - MCP 도구 정의 — 도구 → CLI 배선의 자기서술
//! - `provenance::MAP` — 봉투 필드의 신뢰 경계(문서 파생 = 데이터, 지시 아님)
//!
//! 본 모듈은 이 네 원천을 **실행 시점에** JSON-LD `@graph` 로 접는다. 여기에는
//! 타입 이름도 명령 이름도 손으로 나열하지 않는다 — 원천 선언이 바뀌면 온톨로지가
//! 함께 바뀌므로 드리프트가 구조적으로 불가능하다. 드리프트 가드는
//! `tests/ontology_contract.rs` 가 전수 포섭으로 잡는다.
//!
//! ## 스코프 (O1)
//!
//! 이번 층은 **스키마 모드**만이다 — rhwp 라는 도구 자신(타입·명령·신뢰 경계)의
//! 온톨로지. 특정 문서 한 부를 개체(instance)로 서술하는 문서 인스턴스 모드(O2)는
//! 후속이다.
//!
//! ## 어휘 규약
//!
//! 자체 어휘는 `rhwp:` 접두어(`https://github.com/edwardkim/rhwp/ontology#`) 아래
//! 둔다. 표준 어휘 대응은 **대응이 실제로 성립하는 것만** 쓴다: 클래스는
//! `rdfs:Class`, 속성은 `rdf:Property`, 명령·도구는 `schema:Action` — 그 이상의
//! schema.org 대응(예: Document 를 특정 schema.org 타입에 강제 매핑)은 하지 않는다.
//! 계층(`rdfs:subClassOf`)도 IR 스키마의 구조(순수 `oneOf` 유니온)에서 유도되는
//! 만큼만 싣는다 — 억지 계층 금지.

use serde_json::{json, Map, Value};

use crate::ir_schema;
use crate::provenance;
use crate::schema_registry::ENVELOPE_SCHEMA_VERSION;

/// 온톨로지 산출물 버전. 봉투 `schemaVersion` 과 독립적으로 진화한다.
pub const ONTOLOGY_VERSION: &str = "1.0";

/// 자체 어휘의 IRI 뿌리.
const VOCAB: &str = "https://github.com/edwardkim/rhwp/ontology#";

// ── @id 규약 ─────────────────────────────────────────────────────────────

fn class_id(name: &str) -> String {
    format!("rhwp:ir/{name}")
}

fn property_id(class: &str, field: &str) -> String {
    format!("rhwp:ir/{class}#{field}")
}

fn command_id(name: &str) -> String {
    format!("rhwp:cmd/{name}")
}

fn tool_id(name: &str) -> String {
    format!("rhwp:tool/{name}")
}

/// IRI 참조 값 — JSON-LD 에서 문자열(리터럴)과 구별되도록 `@id` 객체로 싼다.
fn iri(id: String) -> Value {
    json!({ "@id": id })
}

// ── @context ─────────────────────────────────────────────────────────────

/// 접두어 선언만 담는 문맥 — 술어는 전부 접두어 형태(`rdfs:label` 등)로 쓴다.
fn context() -> Value {
    json!({
        "rhwp": VOCAB,
        "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
        "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
        "xsd": "http://www.w3.org/2001/XMLSchema#",
        "schema": "https://schema.org/",
    })
}

// ── IR 스키마 → 클래스·속성 유도 ─────────────────────────────────────────

/// JSON Schema 원시 타입 → XSD 레인지. 매핑이 성립하지 않으면 None(생략).
fn scalar_range(ty: &str) -> Option<&'static str> {
    match ty {
        "string" => Some("xsd:string"),
        "integer" => Some("xsd:integer"),
        "number" => Some("xsd:decimal"),
        "boolean" => Some("xsd:boolean"),
        // 익명 객체(인라인 정의) — 이름 있는 IR 타입이 아니므로 일반 자원으로 둔다.
        "object" => Some("rdfs:Resource"),
        _ => None,
    }
}

fn ref_name(spec: &Value) -> Option<&str> {
    spec.get("$ref")?.as_str()?.strip_prefix("#/$defs/")
}

/// 필드 스키마에서 유도한 레인지.
struct FieldRange {
    /// 레인지 IRI (`rhwp:ir/…` 또는 `xsd:…`). 유도가 성립하지 않으면 None(생략).
    range: Option<String>,
    /// 배열 필드인가 (`type: array`).
    multi: bool,
    /// null 허용인가 (`oneOf: [X, null]`).
    nullable: bool,
}

/// 필드 스키마 하나에서 레인지를 유도한다 — 확정할 수 없는 모양은 지어내지 않고
/// 생략한다(`range: None`).
fn field_range(spec: &Value) -> FieldRange {
    // 참조 — 이름 있는 IR 타입.
    if let Some(name) = ref_name(spec) {
        return FieldRange {
            range: Some(class_id(name)),
            multi: false,
            nullable: false,
        };
    }
    // 배열 — 원소 타입으로 재귀하고 다값 표시만 남긴다.
    if spec.get("type").and_then(Value::as_str) == Some("array") {
        if let Some(items) = spec.get("items") {
            let inner = field_range(items);
            return FieldRange {
                range: inner.range,
                multi: true,
                nullable: false,
            };
        }
    }
    // oneOf — [X, null] 꼴이면 X 로 확정하고 null 허용을 표시한다.
    if let Some(variants) = spec.get("oneOf").and_then(Value::as_array) {
        let mut nullable = false;
        let mut refs: Vec<&str> = Vec::new();
        for v in variants {
            if v.get("type").and_then(Value::as_str) == Some("null") {
                nullable = true;
            } else if let Some(name) = ref_name(v) {
                refs.push(name);
            }
        }
        if let [only] = refs.as_slice() {
            return FieldRange {
                range: Some(class_id(only)),
                multi: false,
                nullable,
            };
        }
        // 다지선다 유니온은 한 레인지로 접을 수 없다 — 지어내지 않는다.
        return FieldRange {
            range: None,
            multi: false,
            nullable,
        };
    }
    // 판별자 상수(`const`) — 문자열 리터럴.
    if spec.get("const").map(Value::is_string) == Some(true) {
        return FieldRange {
            range: Some("xsd:string".to_string()),
            multi: false,
            nullable: false,
        };
    }
    // 원시 타입.
    let range = spec
        .get("type")
        .and_then(Value::as_str)
        .and_then(scalar_range)
        .map(str::to_string);
    FieldRange {
        range,
        multi: false,
        nullable: false,
    }
}

/// 정의가 순수 `oneOf` 유니온(태그 유니온)이면 변형 이름들을 돌려준다.
///
/// `Control` 이 이 모양이다 — 변형들은 유니온의 하위 클래스로 유도할 수 있다.
/// `properties` 나 `type` 이 섞인 정의는 유니온이 아니므로 계층을 만들지 않는다.
fn union_variants(def: &Value) -> Option<Vec<String>> {
    if def.get("properties").is_some() || def.get("type").is_some() {
        return None;
    }
    let one_of = def.get("oneOf")?.as_array()?;
    let mut names = Vec::new();
    for item in one_of {
        names.push(ref_name(item)?.to_string());
    }
    (!names.is_empty()).then_some(names)
}

/// 설명이 비어 있지 않으면 `rdfs:comment` 로 싣는다 — **없으면 생략, 지어내기 금지.**
fn attach_comment(node: &mut Map<String, Value>, description: Option<&Value>) {
    if let Some(text) = description.and_then(Value::as_str) {
        if !text.trim().is_empty() {
            node.insert("rdfs:comment".to_string(), json!(text));
        }
    }
}

/// IR 스키마의 `$defs` 를 클래스·속성 노드로 편다.
fn push_ir_nodes(graph: &mut Vec<Value>) {
    let schema = ir_schema::ir_schema();
    let Some(defs) = schema.get("$defs").and_then(Value::as_object) else {
        return;
    };

    // 계층: 순수 oneOf 유니온의 변형 → 유니온의 하위 클래스.
    let mut super_of: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for (name, def) in defs {
        if let Some(variants) = union_variants(def) {
            for variant in variants {
                // 유니온이 여럿이면 마지막 선언이 이긴다 — 현 IR 은 Control 하나다.
                super_of.insert(variant, class_id(name));
            }
        }
    }

    for (name, def) in defs {
        // 클래스 노드.
        let mut class = Map::new();
        class.insert("@id".to_string(), json!(class_id(name)));
        class.insert("@type".to_string(), json!("rdfs:Class"));
        class.insert("rdfs:label".to_string(), json!(name));
        attach_comment(&mut class, def.get("description"));
        if let Some(sup) = super_of.get(name.as_str()) {
            class.insert("rdfs:subClassOf".to_string(), iri(sup.clone()));
        }
        // 열거형 정의 — 허용 값을 데이터로 싣는다 (스키마의 enum 이 원천).
        if let Some(values) = def.get("enum").and_then(Value::as_array) {
            class.insert("rhwp:enumValues".to_string(), json!(values));
        }
        graph.push(Value::Object(class));

        // 속성 노드 — 필드마다 도메인(소속 타입)·레인지(필드 타입)를 유도한다.
        let Some(properties) = def.get("properties").and_then(Value::as_object) else {
            continue;
        };
        let required: Vec<&str> = def
            .get("required")
            .and_then(Value::as_array)
            .map(|r| r.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        for (field, spec) in properties {
            let mut prop = Map::new();
            prop.insert("@id".to_string(), json!(property_id(name, field)));
            prop.insert("@type".to_string(), json!("rdf:Property"));
            prop.insert("rdfs:label".to_string(), json!(field));
            prop.insert("rdfs:domain".to_string(), iri(class_id(name)));
            let derived = field_range(spec);
            if let Some(range) = derived.range {
                prop.insert("rdfs:range".to_string(), iri(range));
            }
            if derived.multi {
                prop.insert("rhwp:multiValued".to_string(), json!(true));
            }
            if derived.nullable {
                prop.insert("rhwp:nullable".to_string(), json!(true));
            }
            if required.contains(&field.as_str()) {
                prop.insert("rhwp:required".to_string(), json!(true));
            }
            if let Some(values) = spec.get("enum").and_then(Value::as_array) {
                prop.insert("rhwp:enumValues".to_string(), json!(values));
            }
            attach_comment(&mut prop, spec.get("description"));
            graph.push(Value::Object(prop));
        }
    }
}

// ── capabilities → 명령 행위 유도 ────────────────────────────────────────

/// `capabilities` 봉투의 `commands[]` 를 행위 노드로 편다.
fn push_command_nodes(graph: &mut Vec<Value>, capabilities: &Value) {
    let Some(commands) = capabilities.get("commands").and_then(Value::as_array) else {
        return;
    };
    for command in commands {
        let Some(name) = command.get("name").and_then(Value::as_str) else {
            continue;
        };
        let mut node = Map::new();
        node.insert("@id".to_string(), json!(command_id(name)));
        // 명령은 실행 가능한 행위다 — schema.org 대응이 실제로 성립하는 지점.
        node.insert("@type".to_string(), json!(["rhwp:Action", "schema:Action"]));
        node.insert("rdfs:label".to_string(), json!(name));
        attach_comment(&mut node, command.get("summary"));
        // capabilities 가 선언한 축을 그대로 싣는다 — 있으면 싣고 없으면 생략.
        for (source, predicate) in [
            ("category", "rhwp:category"),
            ("json", "rhwp:json"),
            ("batch", "rhwp:batch"),
            ("flags", "rhwp:flags"),
            ("recordFields", "rhwp:recordFields"),
            ("subcommands", "rhwp:subcommands"),
            ("requiresFeature", "rhwp:requiresFeature"),
            ("available", "rhwp:available"),
        ] {
            if let Some(value) = command.get(source) {
                node.insert(predicate.to_string(), value.clone());
            }
        }
        // 신뢰 술어 — 출처 지도의 선언(문서 파생 필드 경로)을 행위에 단다.
        if let Some(entry) = provenance::entry(name) {
            let paths: Vec<&str> = entry.untrusted.iter().map(|f| f.path).collect();
            node.insert("rhwp:untrustedFields".to_string(), json!(paths));
            node.insert("rhwp:provenanceNote".to_string(), json!(entry.note));
        }
        graph.push(Value::Object(node));
    }
}

// ── MCP 도구 → 행위 유도 ─────────────────────────────────────────────────

/// MCP 도구 정의를 행위 노드로 펴고, 내려가는 CLI 명령 노드와 연결한다.
fn push_tool_nodes(graph: &mut Vec<Value>, tools: &[Value]) {
    for tool in tools {
        let Some(name) = tool.get("name").and_then(Value::as_str) else {
            continue;
        };
        let mut node = Map::new();
        node.insert("@id".to_string(), json!(tool_id(name)));
        node.insert("@type".to_string(), json!(["rhwp:Action", "schema:Action"]));
        node.insert("rdfs:label".to_string(), json!(name));
        attach_comment(&mut node, tool.get("description"));
        if let Some(command) = tool
            .get("cli")
            .and_then(|c| c.get("command"))
            .and_then(Value::as_str)
        {
            node.insert(
                "rhwp:implementsCommand".to_string(),
                iri(command_id(command)),
            );
        }
        if let Some(properties) = tool
            .get("inputSchema")
            .and_then(|s| s.get("properties"))
            .and_then(Value::as_object)
        {
            let inputs: Vec<&String> = properties.keys().collect();
            node.insert("rhwp:inputProperties".to_string(), json!(inputs));
        }
        if let Some(required) = tool.get("inputSchema").and_then(|s| s.get("required")) {
            node.insert("rhwp:requiredInputs".to_string(), required.clone());
        }
        if let Some(output_fields) = tool.get("outputFields") {
            node.insert("rhwp:outputFields".to_string(), output_fields.clone());
        }
        // [#4226 접합] annotations(readOnly/destructive/idempotent) — 도구 정의에
        // 있으면 싣고 없으면 생략한다. #4226 미머지 상태의 devel 에서도 유도가
        // 성립해야 하므로 모양을 가정하지 않고 통째로 옮긴다.
        if let Some(annotations) = tool.get("annotations") {
            node.insert("rhwp:annotations".to_string(), annotations.clone());
        }
        graph.push(Value::Object(node));
    }
}

// ── 조립 ─────────────────────────────────────────────────────────────────

/// 온톨로지 메타 노드 — 무엇에서 유도됐는지의 자기 기술.
fn meta_node(capabilities: &Value) -> Value {
    let mut node = Map::new();
    node.insert("@id".to_string(), json!("rhwp:ontology"));
    node.insert("@type".to_string(), json!("rhwp:Ontology"));
    node.insert("rdfs:label".to_string(), json!("rhwp ontology"));
    node.insert(
        "rdfs:comment".to_string(),
        json!(
            "rhwp 의 자기서술(IR 스키마·capabilities·MCP 도구 정의·봉투 출처 지도)에서 \
             실행 시점에 기계 유도한 온톨로지. 손 나열 상수가 없으므로 원천 선언이 \
             바뀌면 함께 바뀐다 — 드리프트 불가."
        ),
    );
    node.insert("rhwp:ontologyVersion".to_string(), json!(ONTOLOGY_VERSION));
    node.insert(
        "rhwp:irSchemaVersion".to_string(),
        json!(ir_schema::IR_SCHEMA_VERSION),
    );
    if let Some(version) = capabilities.get("version") {
        node.insert("rhwp:toolVersion".to_string(), version.clone());
    }
    Value::Object(node)
}

/// JSON-LD 온톨로지 본문 (`--bare` 가 내는 것).
///
/// `capabilities` 와 MCP 도구 목록은 바이너리 쪽 단일 출처 함수의 산출을 그대로
/// 받는다 — 프로세스 재호출 없이 같은 크레이트의 값을 직접 접는다.
pub fn ontology(capabilities: &Value, mcp_tools: &[Value]) -> Value {
    let mut graph: Vec<Value> = vec![meta_node(capabilities)];
    push_ir_nodes(&mut graph);
    push_command_nodes(&mut graph, capabilities);
    push_tool_nodes(&mut graph, mcp_tools);
    json!({
        "@context": context(),
        "@graph": graph,
    })
}

/// `@graph` 에서 (클래스, 속성, 행위) 수를 센다.
fn counts(body: &Value) -> (usize, usize, usize) {
    let mut classes = 0usize;
    let mut properties = 0usize;
    let mut actions = 0usize;
    if let Some(graph) = body.get("@graph").and_then(Value::as_array) {
        for node in graph {
            match node.get("@type") {
                Some(Value::String(t)) if t == "rdfs:Class" => classes += 1,
                Some(Value::String(t)) if t == "rdf:Property" => properties += 1,
                Some(Value::Array(types)) if types.iter().any(|t| t == "rhwp:Action") => {
                    actions += 1;
                }
                _ => {}
            }
        }
    }
    (classes, properties, actions)
}

/// `export-ontology` 봉투 — 본문과 유도 규모(클래스·속성·행위 수)를 함께 싣는다.
///
/// 규모 필드는 소비자가 파싱 전에 산출 크기를 가늠하게 하고, 유도가 통째로 비는
/// 회귀를 숫자 하나로 잡는다.
pub fn envelope(capabilities: &Value, mcp_tools: &[Value]) -> Value {
    let body = ontology(capabilities, mcp_tools);
    let (class_count, property_count, action_count) = counts(&body);
    json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "ontology": body,
        "classCount": class_count,
        "propertyCount": property_count,
        "actionCount": action_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// 단위 테스트용 소형 capabilities/도구 표본 — 실물 전수 대조는
    /// tests/ontology_contract.rs 가 바이너리 실행으로 한다.
    fn sample_capabilities() -> Value {
        json!({
            "version": "0.0.0-test",
            "commands": [
                { "name": "info", "category": "query", "summary": "문서 메타", "json": true,
                  "flags": ["--json"], "recordFields": ["schemaVersion", "title"] },
                { "name": "dump", "category": "diagnostic", "summary": "덤프" },
            ],
        })
    }

    fn sample_tools() -> Vec<Value> {
        vec![json!({
            "name": "hwp_info",
            "description": "문서 메타를 돌려준다",
            "inputSchema": { "type": "object", "properties": { "path": {} }, "required": ["path"] },
            "cli": { "command": "info", "args": ["info", "{path}", "--json"] },
            "outputFields": ["schemaVersion", "title"],
        })]
    }

    #[test]
    fn graph_covers_every_ir_definition_as_class() {
        let body = ontology(&sample_capabilities(), &sample_tools());
        let labels: HashSet<&str> = body["@graph"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["@type"] == "rdfs:Class")
            .filter_map(|n| n["rdfs:label"].as_str())
            .collect();
        let schema = ir_schema::ir_schema();
        for name in schema["$defs"].as_object().unwrap().keys() {
            assert!(labels.contains(name.as_str()), "클래스 누락: {name}");
        }
    }

    #[test]
    fn property_domains_point_at_existing_classes() {
        let body = ontology(&sample_capabilities(), &sample_tools());
        let graph = body["@graph"].as_array().unwrap();
        let class_ids: HashSet<&str> = graph
            .iter()
            .filter(|n| n["@type"] == "rdfs:Class")
            .filter_map(|n| n["@id"].as_str())
            .collect();
        let mut checked = 0usize;
        for node in graph {
            if node["@type"] != "rdf:Property" {
                continue;
            }
            let domain = node["rdfs:domain"]["@id"].as_str().expect("domain @id");
            assert!(class_ids.contains(domain), "고아 도메인: {domain}");
            checked += 1;
        }
        assert!(checked > 100, "속성 유도가 통째로 비었다: {checked}");
    }

    #[test]
    fn control_variants_are_subclasses_of_control() {
        let body = ontology(&sample_capabilities(), &sample_tools());
        let graph = body["@graph"].as_array().unwrap();
        let table = graph
            .iter()
            .find(|n| n["@id"] == "rhwp:ir/TableControl")
            .expect("TableControl");
        assert_eq!(table["rdfs:subClassOf"]["@id"], "rhwp:ir/Control");
        // 억지 계층 금지 — 유니온 밖 타입에는 계층이 없어야 한다.
        let paragraph = graph
            .iter()
            .find(|n| n["@id"] == "rhwp:ir/Paragraph")
            .expect("Paragraph");
        assert!(paragraph.get("rdfs:subClassOf").is_none());
    }

    #[test]
    fn commands_become_actions_with_trust_predicates() {
        let body = ontology(&sample_capabilities(), &sample_tools());
        let graph = body["@graph"].as_array().unwrap();
        let info = graph
            .iter()
            .find(|n| n["@id"] == "rhwp:cmd/info")
            .expect("info 행위");
        assert!(info["@type"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t == "schema:Action"));
        assert_eq!(info["rhwp:category"], "query");
        // provenance::MAP 의 info 선언(title·fonts[])이 신뢰 술어로 실린다.
        let untrusted = info["rhwp:untrustedFields"].as_array().expect("신뢰 술어");
        assert!(untrusted.iter().any(|p| p == "title"), "{untrusted:?}");
    }

    #[test]
    fn tools_become_actions_linked_to_commands() {
        let body = ontology(&sample_capabilities(), &sample_tools());
        let graph = body["@graph"].as_array().unwrap();
        let tool = graph
            .iter()
            .find(|n| n["@id"] == "rhwp:tool/hwp_info")
            .expect("hwp_info 행위");
        assert_eq!(tool["rhwp:implementsCommand"]["@id"], "rhwp:cmd/info");
        assert_eq!(tool["rhwp:requiredInputs"], json!(["path"]));
        // #4226 미머지 상태 — annotations 가 없으면 술어도 없어야 한다(지어내기 금지).
        assert!(tool.get("rhwp:annotations").is_none());
    }

    #[test]
    fn tool_annotations_are_carried_when_present() {
        let mut tools = sample_tools();
        tools[0]["annotations"] = json!({ "readOnlyHint": true });
        let body = ontology(&sample_capabilities(), &tools);
        let tool = body["@graph"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["@id"] == "rhwp:tool/hwp_info")
            .expect("hwp_info 행위")
            .clone();
        assert_eq!(tool["rhwp:annotations"]["readOnlyHint"], true);
    }

    #[test]
    fn node_ids_are_unique() {
        let body = ontology(&sample_capabilities(), &sample_tools());
        let graph = body["@graph"].as_array().unwrap();
        let mut seen = HashSet::new();
        for node in graph {
            let id = node["@id"].as_str().expect("@id");
            assert!(seen.insert(id.to_string()), "@id 중복: {id}");
        }
    }

    #[test]
    fn envelope_counts_match_graph() {
        let env = envelope(&sample_capabilities(), &sample_tools());
        assert_eq!(env["schemaVersion"], "1.0");
        let graph = env["ontology"]["@graph"].as_array().unwrap();
        let classes = graph.iter().filter(|n| n["@type"] == "rdfs:Class").count();
        let actions = graph
            .iter()
            .filter(|n| {
                n["@type"]
                    .as_array()
                    .is_some_and(|t| t.iter().any(|x| x == "rhwp:Action"))
            })
            .count();
        assert_eq!(env["classCount"].as_u64().unwrap() as usize, classes);
        assert_eq!(env["actionCount"].as_u64().unwrap() as usize, actions);
        assert!(env["propertyCount"].as_u64().unwrap() > 100);
        // IR 타입 41정의가 전부 실려야 한다.
        assert!(classes >= 41, "클래스가 너무 적다: {classes}");
    }
}
