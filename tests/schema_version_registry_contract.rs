//! [#3907 R83] 스키마 버전 레지스트리 — 공개 스키마 3축의 단일 대사 지점.
//!
//! 공개 스키마 버전 상수는 세 곳에 흩어져 산다: `src/ir_schema.rs` 의
//! `IR_SCHEMA_VERSION`, `src/plan_schema.rs` 의 `PLAN_SCHEMA_VERSION`,
//! `src/capabilities_schema.rs` 의 `CAPABILITIES_SCHEMA_VERSION`. 축별 계약
//! 테스트(ir_schema_contract·plan_schema_contract·capabilities_schema_contract)는
//! 각자 자기 봉투만 보므로, "세 축이 함께 움직여야 하는 지점"의 드리프트 —
//! 상수와 `$id` 꼬리의 어긋남, 생성물 바인딩의 잔류, 봉투 계약 버전의 축간
//! 분화 — 는 어디에서도 잡히지 않았다. 이 파일이 그 교차 대사의 단일 지점이다.
//!
//! 설계 원칙 두 가지:
//!
//! 1. **레지스트리는 코드 표면이 아니라 실패 메시지다.** 새 모듈·새 명령을
//!    만들지 않는다. 각 검사가 실패할 때 "버전을 올리면 함께 올려야 하는 지점"
//!    목록을 출력하는 것으로 레지스트리 역할을 수행한다 — 선언은 이 파일의
//!    `AXES` 표 하나뿐이고, 소스 쪽 수정은 0 이다.
//!
//! 2. **정책 중립.** 여기 있는 것은 선언(어떤 축·어떤 필드·어떤 상수)과
//!    대사(실물 바이너리 출력 ↔ 소스 상수)뿐이다. 버전을 **어떻게** 올리는가 —
//!    semver 이중 규약, major/minor 판정 — 는 로드맵 R67(버전 정책, [가설])의
//!    영역이라 여기서 규정하지 않는다. R67 이 착지하면 이 표는 그대로 두고
//!    정책 검사만 얹으면 된다.

#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

use rhwp::capabilities_schema::CAPABILITIES_SCHEMA_VERSION;
use rhwp::ir_schema::IR_SCHEMA_VERSION;
use rhwp::plan_schema::PLAN_SCHEMA_VERSION;

/// nextest 가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp 를 우선한다
/// (cli_json_contract.rs 와 같은 처리).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
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

/// 공개 스키마 한 축의 레지스트리 항목 — 선언과 대사 지점만 담는다.
struct SchemaAxis {
    /// 봉투를 내는 실물 명령.
    command: &'static str,
    /// 봉투·스키마 본문이 싣는 축 고유 버전 필드.
    version_field: &'static str,
    /// `src/` 단일 출처 상수의 실값 (lib 경유).
    source_version: &'static str,
    /// 실패 메시지에 쓸 상수 이름.
    constant_name: &'static str,
    /// 봉투 안에서 스키마 본문이 실리는 키들 — 각 본문은 버전 필드와
    /// 버전 꼬리를 가진 `$id` 를 싣는다 (capabilities 는 본문이 둘이다).
    body_keys: &'static [&'static str],
    /// 버전을 올릴 때 **함께 올려야 하는 지점** — 실패 메시지가 이 목록을
    /// 출력하는 것으로 레지스트리 역할을 한다.
    bump_together: &'static [&'static str],
}

/// 스키마 버전 레지스트리 본체. 새 공개 스키마 축이 생기면 여기 한 줄을 얹는다.
const AXES: &[SchemaAxis] = &[
    SchemaAxis {
        command: "export-ir-schema",
        version_field: "irSchemaVersion",
        source_version: IR_SCHEMA_VERSION,
        constant_name: "IR_SCHEMA_VERSION",
        body_keys: &["schema"],
        bump_together: &[
            "src/ir_schema.rs — pub const IR_SCHEMA_VERSION",
            "src/ir_schema.rs ir_schema() — \"$id\" 꼬리 …/schema/ir/<버전> (리터럴)",
        ],
    },
    SchemaAxis {
        command: "export-plan-schema",
        version_field: "planSchemaVersion",
        source_version: PLAN_SCHEMA_VERSION,
        constant_name: "PLAN_SCHEMA_VERSION",
        body_keys: &["schema"],
        bump_together: &[
            "src/plan_schema.rs — pub const PLAN_SCHEMA_VERSION",
            "src/plan_schema.rs plan_schema() — \"$id\" 꼬리 …/schema/plan/<버전> (리터럴)",
            "tests/plan_schema_contract.rs — planSchemaVersion 하드코딩 대조",
        ],
    },
    SchemaAxis {
        command: "export-capabilities-schema",
        version_field: "capabilitiesSchemaVersion",
        source_version: CAPABILITIES_SCHEMA_VERSION,
        constant_name: "CAPABILITIES_SCHEMA_VERSION",
        // capabilities 봉투는 본문이 둘이다 — 명령 표면 스키마와 MCP 매니페스트
        // 스키마가 한 상수를 공유한다.
        body_keys: &["schema", "mcpSchema"],
        bump_together: &[
            "src/capabilities_schema.rs — pub const CAPABILITIES_SCHEMA_VERSION",
            "src/capabilities_schema.rs capabilities_schema() — \"$id\" 꼬리 …/schema/capabilities/<버전>",
            "src/capabilities_schema.rs mcp_manifest_schema() — \"$id\" 꼬리 …/schema/capabilities-mcp/<버전>",
        ],
    },
];

/// 봉투 계약 버전(`schemaVersion`)의 대사 지점 — 세 봉투가 각자 리터럴로 들고
/// 있어, 한 곳만 올리면 축간 분화가 조용히 일어난다.
const ENVELOPE_VERSION_BUMP_TOGETHER: &[&str] = &[
    "src/ir_schema.rs envelope() — \"schemaVersion\" 리터럴",
    "src/plan_schema.rs envelope() — \"schemaVersion\" 리터럴",
    "src/capabilities_schema.rs envelope() — \"schemaVersion\" 리터럴",
    "src/main.rs — export-*-schema 명령 -o 저널의 \"schemaVersion\" 리터럴 3곳",
    "tests/ir_schema_contract.rs·plan_schema_contract.rs·capabilities_schema_contract.rs — 하드코딩 대조",
];

fn registry_note(constant_name: &str, bump_together: &[&str]) -> String {
    format!(
        "\n[레지스트리] {} 을(를) 올릴 때 함께 올려야 하는 지점:\n  - {}",
        constant_name,
        bump_together.join("\n  - ")
    )
}

/// 실물 바이너리를 돌려 봉투를 얻는다 — 상수 대 상수 비교로는 lib↔bin 경로의
/// 드리프트(예: main.rs 쪽 하드코딩)를 잡을 수 없어, 반드시 실행 출력으로 대사한다.
fn envelope_of(command: &str) -> serde_json::Value {
    let args = [command];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    serde_json::from_slice(&output.stdout).expect("봉투 JSON 파싱 실패")
}

/// `$id` 의 마지막 경로 조각 — 버전 꼬리.
fn id_tail(id: &str) -> &str {
    id.rsplit('/').next().unwrap_or("")
}

/// 축별 대사 1 — 실물 봉투·스키마 본문의 버전 필드가 소스 상수와 일치한다.
#[test]
fn envelope_version_field_matches_source_constant() {
    for axis in AXES {
        let v = envelope_of(axis.command);
        assert_eq!(
            v[axis.version_field],
            axis.source_version,
            "{} 봉투의 {} 이 소스 상수 {} = {:?} 와 어긋난다.{}",
            axis.command,
            axis.version_field,
            axis.constant_name,
            axis.source_version,
            registry_note(axis.constant_name, axis.bump_together)
        );
        for key in axis.body_keys {
            assert_eq!(
                v[key][axis.version_field],
                axis.source_version,
                "{} 의 {} 본문 버전이 소스 상수 {} = {:?} 와 어긋난다.{}",
                axis.command,
                key,
                axis.constant_name,
                axis.source_version,
                registry_note(axis.constant_name, axis.bump_together)
            );
        }
    }
}

/// 축별 대사 2 — 스키마 본문 `$id` 의 버전 꼬리가 소스 상수와 일치한다.
///
/// `$id` 는 상수를 참조하지 않는 리터럴이라, 상수만 올리면 조용히 낡는다 —
/// 이 검사가 있기 전까지 어떤 테스트도 이 꼬리를 보지 않았다.
#[test]
fn schema_id_version_tail_matches_source_constant() {
    for axis in AXES {
        let v = envelope_of(axis.command);
        for key in axis.body_keys {
            let id = v[key]["$id"].as_str().unwrap_or_else(|| {
                panic!("{} 의 {} 본문에 $id 가 없다: {}", axis.command, key, v[key])
            });
            assert_eq!(
                id_tail(id),
                axis.source_version,
                "{} 의 {} 본문 $id({}) 버전 꼬리가 소스 상수 {} = {:?} 와 어긋난다.{}",
                axis.command,
                key,
                id,
                axis.constant_name,
                axis.source_version,
                registry_note(axis.constant_name, axis.bump_together)
            );
        }
    }
}

/// 교차 대사 — 세 봉투의 계약 버전(`schemaVersion`)은 한 값으로 움직인다.
///
/// 값 자체(현재 "1.0")는 축별 계약 테스트가 각자 고정한다. 여기서는 값을
/// 규정하지 않고 **축간 동일성**만 대사한다 — 한 축만 올라간 채 머지되는
/// 조용한 분화가 이 검사의 표적이다.
#[test]
fn envelope_contract_version_is_uniform_across_axes() {
    let versions: Vec<(&str, serde_json::Value)> = AXES
        .iter()
        .map(|axis| {
            (
                axis.command,
                envelope_of(axis.command)["schemaVersion"].clone(),
            )
        })
        .collect();
    let (reference_command, reference) = &versions[0];
    assert!(
        reference.is_string(),
        "{} 봉투에 schemaVersion 이 없다: {:?}",
        reference_command,
        reference
    );
    for (command, version) in &versions[1..] {
        assert_eq!(
            version,
            reference,
            "봉투 계약 버전이 축마다 다르다 — {} = {:?}, {} = {:?}. 함께 올리거나, \
             분화가 의도라면 이 검사를 그 결정과 함께 갱신하라.{}",
            reference_command,
            reference,
            command,
            version,
            registry_note("봉투 schemaVersion", ENVELOPE_VERSION_BUMP_TOGETHER)
        );
    }
}
