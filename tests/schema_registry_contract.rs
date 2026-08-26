//! [#4329] 스키마 버전 레지스트리(R83) × 버전 정책(R67) 계약.
//!
//! 이 테스트가 고정하는 것은 세 가지다.
//!
//! 1. **산개 금지(소스 스캔)** — 버전 문자열의 단일 출처는 `src/schema_registry.rs`
//!    다. 봉투 `"schemaVersion": "…"` 리터럴, `*_SCHEMA_VERSION` 상수 정의, `$id`
//!    버전 조각이 레지스트리 밖에 다시 생기면 여기서 red 가 된다 — #4329 이전의
//!    상태(8개 파일 ~67사이트 산개)로 되돌아가는 회귀를 구조로 막는다.
//! 2. **실행 봉투 일치** — `capabilities` 의 `schemaRegistry` 와 각
//!    `export-*-schema` 봉투의 축 버전이 레지스트리 상수와 항상 같다. 선언과
//!    실물이 갈리면 소비자는 갈린 쪽을 믿고 깨진다(#4327 §3 의 스테일 고정이
//!    바로 그 실물 사례).
//! 3. **소비자 대사 표면** — 외부 소비자가 `capabilities` 한 번으로 전 축 버전과
//!    정책 문서 경로를 기계 대조할 수 있다(#4327 U2). 축 집합은 고정
//!    {envelope, ir, capabilities, plan} 이고, 정책 경로는 실물 문서로 이어진다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::schema_registry::{
    CAPABILITIES_SCHEMA_VERSION, ENVELOPE_SCHEMA_VERSION, IR_SCHEMA_VERSION, PLAN_SCHEMA_VERSION,
};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn describe(args: &[&str], o: &Output) -> String {
    format!(
        "명령: rhwp {}\nexit: {:?}\nstderr:\n{}",
        args.join(" "),
        o.status.code(),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn envelope_of(args: &[&str]) -> serde_json::Value {
    let o = run(args);
    assert_eq!(o.status.code(), Some(0), "{}", describe(args, &o));
    serde_json::from_slice(&o.stdout).expect("stdout 순수 JSON 봉투")
}

// ── 1. 산개 금지 — 소스 스캔 ────────────────────────────────────────────

fn rs_files_under(dir: &Path, acc: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("src 순회") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rs_files_under(&path, acc);
        } else if path.extension().is_some_and(|e| e == "rs") {
            acc.push(path);
        }
    }
}

fn has_envelope_version_literal(statement: &str) -> bool {
    let compact = statement
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let object_literal = compact.contains(r#""schemaVersion":""#);
    let insert_without_registry = compact.contains(r#"insert("schemaVersion""#)
        && !compact.contains("ENVELOPE_SCHEMA_VERSION");
    let assignment_without_registry = (compact.contains(r#"["schemaVersion"]="#)
        || compact.contains(r#"['schemaVersion']="#))
        && !compact.contains("ENVELOPE_SCHEMA_VERSION");
    object_literal || insert_without_registry || assignment_without_registry
}

#[test]
fn source_scanner_recognizes_object_insert_and_assignment_literals() {
    assert!(has_envelope_version_literal(r#""schemaVersion": "1.0","#));
    assert!(has_envelope_version_literal(
        r#"map.insert(
            "schemaVersion".into(),
            json!("9.9"),
        );"#
    ));
    assert!(has_envelope_version_literal(
        r#"value["schemaVersion"] = serde_json::json!("1.0");"#
    ));
    assert!(!has_envelope_version_literal(
        r#""schemaVersion": ENVELOPE_SCHEMA_VERSION,"#
    ));
    assert!(!has_envelope_version_literal(
        r#"map.insert(
            "schemaVersion".into(),
            json!(ENVELOPE_SCHEMA_VERSION),
        );"#
    ));
}

/// 레지스트리 밖의 버전 리터럴·상수 정의를 전수 스캔으로 금지한다.
///
/// 스캔 대상은 `src/` 전체(.rs)다. tests/ 는 대상이 아니다 — 테스트가 기대값을
/// 리터럴로 고정하는 것(예: `assert_eq!(env["schemaVersion"], "1.0")`)은 이중
/// 장부로서 유익하다.
#[test]
fn no_version_literals_outside_registry() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rs_files_under(&src, &mut files);
    assert!(
        files.len() > 100,
        "src 스캔이 {}개 파일뿐 — 순회가 깨졌다",
        files.len()
    );

    let mut violations = Vec::new();
    for path in files {
        if path.file_name().is_some_and(|n| n == "schema_registry.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("소스 읽기");
        let mut statement_start = 1;
        for statement in text.split_inclusive(';') {
            // ① 봉투 버전 리터럴 — 객체, Map::insert, 인덱스 대입 형태를
            // 공백·개행과 무관하게 잡는다.
            if has_envelope_version_literal(statement) {
                violations.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    statement_start,
                    statement.split_whitespace().collect::<Vec<_>>().join(" ")
                ));
            }
            statement_start += statement.bytes().filter(|b| *b == b'\n').count();
        }
        for (i, line) in text.lines().enumerate() {
            // ② 축 상수 재정의 — 재수출(pub use)은 허용, 값 정의는 금지.
            let const_definition = line.contains(r#"SCHEMA_VERSION: &str = ""#);
            // ③ $id 버전 조각 리터럴 — format!(…{상수}) 파생만 허용.
            let id_literal = line.contains("rhwp/schema/") && line.contains("/1.");
            if const_definition || id_literal {
                violations.push(format!("{}:{}: {}", path.display(), i + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "버전 문자열은 src/schema_registry.rs 단일 출처에서만 정의한다(#4329). 산개 재발:\n{}",
        violations.join("\n")
    );
}

// ── 2·3. 실행 봉투 일치 + 소비자 대사 표면 ──────────────────────────────

/// `capabilities` 봉투의 `schemaRegistry` 가 레지스트리 상수와 일치하고, 축 집합이
/// 고정돼 있으며, 정책 경로가 실물 문서로 이어진다.
#[test]
fn capabilities_schema_registry_matches_constants() {
    let env = envelope_of(&["capabilities"]);
    assert_eq!(env["schemaVersion"], ENVELOPE_SCHEMA_VERSION);

    let reg = &env["schemaRegistry"];
    assert_eq!(
        reg["crateVersion"],
        env!("CARGO_PKG_VERSION"),
        "crateVersion 은 Cargo.toml 단일 출처와 같아야 한다"
    );
    assert_eq!(
        reg["policy"], "mydocs/tech/agent_runtime/version_policy.md",
        "정책 문서 경로가 계약과 다르다"
    );
    let policy_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(reg["policy"].as_str().unwrap());
    assert!(
        policy_path.exists(),
        "봉투가 광고하는 정책 문서가 실물에 없다: {}",
        policy_path.display()
    );

    let axes = reg["axes"].as_array().expect("axes 배열");
    let expected = [
        ("envelope", ENVELOPE_SCHEMA_VERSION),
        ("ir", IR_SCHEMA_VERSION),
        ("capabilities", CAPABILITIES_SCHEMA_VERSION),
        ("plan", PLAN_SCHEMA_VERSION),
    ];
    assert_eq!(axes.len(), expected.len(), "축 집합은 고정이다: {reg}");
    for (axis, version) in expected {
        let found = axes
            .iter()
            .find(|a| a["axis"] == axis)
            .unwrap_or_else(|| panic!("축 {axis} 이 봉투에 없다"));
        assert_eq!(found["version"], version, "축 {axis} 버전 불일치");
        assert!(
            found["surface"].as_str().is_some_and(|s| !s.is_empty()),
            "축 {axis} 의 surface 서술이 비었다"
        );
    }
}

/// 각 `export-*-schema` 봉투의 축 버전·`$id` 버전 조각이 레지스트리 상수에서
/// 파생된다 — 셋 중 하나라도 리터럴로 되돌아가면 값이 갈리는 순간 여기서 잡힌다.
#[test]
fn export_schema_envelopes_derive_from_registry() {
    let caps = envelope_of(&["export-capabilities-schema"]);
    assert_eq!(caps["schemaVersion"], ENVELOPE_SCHEMA_VERSION);
    assert_eq!(
        caps["capabilitiesSchemaVersion"],
        CAPABILITIES_SCHEMA_VERSION
    );
    let caps_id = caps["schema"]["$id"].as_str().expect("$id");
    assert!(
        caps_id.ends_with(&format!("/capabilities/{CAPABILITIES_SCHEMA_VERSION}")),
        "$id 버전 조각이 레지스트리와 다르다: {caps_id}"
    );
    let mcp_id = caps["mcpSchema"]["$id"].as_str().expect("mcp $id");
    assert!(
        mcp_id.ends_with(&format!("/capabilities-mcp/{CAPABILITIES_SCHEMA_VERSION}")),
        "mcp $id 버전 조각이 레지스트리와 다르다: {mcp_id}"
    );

    let ir = envelope_of(&["export-ir-schema"]);
    assert_eq!(ir["schemaVersion"], ENVELOPE_SCHEMA_VERSION);
    assert_eq!(ir["irSchemaVersion"], IR_SCHEMA_VERSION);
    let ir_id = ir["schema"]["$id"].as_str().expect("ir $id");
    assert!(
        ir_id.ends_with(&format!("/ir/{IR_SCHEMA_VERSION}")),
        "ir $id 버전 조각이 레지스트리와 다르다: {ir_id}"
    );

    let plan = envelope_of(&["export-plan-schema"]);
    assert_eq!(plan["schemaVersion"], ENVELOPE_SCHEMA_VERSION);
    assert_eq!(plan["planSchemaVersion"], PLAN_SCHEMA_VERSION);
    let plan_id = plan["schema"]["$id"].as_str().expect("plan $id");
    assert!(
        plan_id.ends_with(&format!("/plan/{PLAN_SCHEMA_VERSION}")),
        "plan $id 버전 조각이 레지스트리와 다르다: {plan_id}"
    );
}

/// capabilities 스키마(자기서술의 자기서술)도 새 표면을 계약에 실었다 —
/// `$defs.SchemaRegistry` 가 존재하고 Capabilities 봉투가 `schemaRegistry` 를
/// 필수로 선언한다. 스키마에서 빠지면 코드 생성기가 이 축을 통째로 모른다.
#[test]
fn capabilities_schema_declares_schema_registry() {
    let caps = envelope_of(&["export-capabilities-schema"]);
    let schema = &caps["schema"];
    assert!(
        schema["$defs"]["SchemaRegistry"].is_object(),
        "$defs.SchemaRegistry 부재"
    );
    let required: Vec<&str> = schema["$defs"]["Capabilities"]["required"]
        .as_array()
        .expect("Capabilities.required")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(
        required.contains(&"schemaRegistry"),
        "Capabilities.required 에 schemaRegistry 가 없다: {required:?}"
    );
}
