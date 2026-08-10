//! Issue #4396 — HWPX 필드 `<hp:parameters>` 가 HWP5 왕복(HWPX→HWP→HWPX) 후
//! `Command` 하나로 축소되던 손실.
//!
//! ## 범위 (2026-08 리뷰로 확정)
//!
//! - **HWPX↔HWPX**: `Field::parameters`(`ParameterList` — OWPML `hp:ParameterList` 5종
//!   그대로) 로 완전 보존한다. OWPML 이 `<hp:parameters>` 구조를 규정하므로 옳은 수정.
//! - **HWPX→HWP5→HWPX**: 여전히 손실이 남는다 — `command`/`memo_index`(HWP5
//!   CTRL_HEADER 에 실제 슬롯이 있는 항목)를 뺀 나머지(`Prop`/`Direction`/`HelpState`/
//!   `Path`/`Category`/`TargetType`/`DocOpenType` 등)는 HWP5 CTRL_DATA 에 담을 스펙
//!   규정 슬롯이 없다(`pdf/hwpspec-2024.pdf` §4.2.8/§4.2.10.11/§4.2.10.15 확인 —
//!   `src/serializer/control.rs` 의 `field_parameter_loss_warning` 문서 주석 참고).
//!   스펙에 없는 `item_id` 를 임의로 정해 채우는 시도(`0x4010`)는 리뷰에서
//!   `document_core::converters::hwpx_to_hwp` 의 `0x4000+idx` 순차 할당과 충돌
//!   가능성이 지적되어 되돌렸다. 대신 `serializer::control::field_parameter_loss_warning`
//!   이 이 손실을 조용히 넘기지 않고 경고(`eprintln!`)로 낸다 — 그 판정 로직의 단위
//!   테스트는 `src/serializer/control/tests.rs` 에 있다(`field_parameter_loss_warning_*`).
//!
//! 이 파일은 **HWPX↔HWPX 보존**(GREEN, 새로 확보된 계약)과 **HWP5 경유 손실이 여전히
//! 현재 상태임을 문서화**(회귀 시 이 테스트가 깨지도록) 하는 두 가지를 검사한다.
//!
//! fixture: `samples/누름틀-2024.hwpx` — 이슈에 실린 정확한 재현 샘플
//! (section 0 paragraph 0 필드가 Prop(integerParam)/Command(stringParam)/
//! Direction(stringParam) 3개 파라미터를 가진 `<hp:parameters cnt="3">`).

use std::fs;
use std::path::Path;

use rhwp::model::control::{Control, Field, Parameter};
use rhwp::model::document::Document;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/누름틀-2024.hwpx";

fn sample_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// 문서 순회 순서로 필드 컨트롤을 전부 모은다(섹션→문단→컨트롤 순, 안정적인 순서).
fn collect_fields(doc: &Document) -> Vec<&Field> {
    doc.sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            Control::Field(f) => Some(f),
            _ => None,
        })
        .collect()
}

fn param_name(p: &Parameter) -> Option<&str> {
    match p {
        Parameter::Boolean { name, .. }
        | Parameter::Integer { name, .. }
        | Parameter::Float { name, .. }
        | Parameter::String { name, .. } => name.as_deref(),
        Parameter::List(list) => list.name.as_deref(),
    }
}

/// [#4396] HWPX→HWPX 재직렬화 후에도 `Field::parameters` 트리가 완전히 보존된다 —
/// OWPML `hp:ParameterList` 가 규정하는 구조 그대로(이름·타입·값, Prop/Direction 등
/// 전부). 이 계약은 `raw_parameters_xml` 바이트 정확 캐시와 별개로 구조화된 트리에도
/// 성립해야 한다.
#[test]
fn field_parameters_tree_fully_preserved_across_pure_hwpx_roundtrip() {
    let bytes = fs::read(sample_path()).expect("샘플 읽기");
    let doc1 = parse_hwpx(&bytes).expect("원본 파싱");
    let fields1 = collect_fields(&doc1);
    assert!(!fields1.is_empty(), "샘플에 필드가 있어야 함");

    let multi_param_idx = fields1
        .iter()
        .position(|f| f.parameters.items.len() > 1)
        .unwrap_or_else(|| panic!("전제 실패: {SAMPLE} 에 다중 파라미터 필드가 없음"));
    let original_names: Vec<&str> = fields1[multi_param_idx]
        .parameters
        .items
        .iter()
        .filter_map(param_name)
        .collect();
    assert!(
        original_names.contains(&"Prop") && original_names.contains(&"Direction"),
        "전제 실패: 필드[{multi_param_idx}] 에 Prop/Direction 이 없음: {original_names:?}"
    );

    let out = serialize_hwpx(&doc1).expect("HWPX 재직렬화");
    let doc2 = parse_hwpx(&out).expect("HWPX 재파싱");
    let fields2 = collect_fields(&doc2);
    assert_eq!(
        fields1.len(),
        fields2.len(),
        "HWPX 왕복 후 필드 개수가 달라짐"
    );

    let f1 = &fields1[multi_param_idx];
    let f2 = &fields2[multi_param_idx];
    assert_eq!(
        f1.parameters.items.len(),
        f2.parameters.items.len(),
        "HWPX↔HWPX 왕복인데 파라미터 개수가 달라짐 — 필드[{multi_param_idx}]: \
         원본={:?} 왕복후={:?}",
        f1.parameters.items,
        f2.parameters.items
    );
    let names2: Vec<&str> = f2.parameters.items.iter().filter_map(param_name).collect();
    for name in &original_names {
        assert!(
            names2.contains(name),
            "HWPX↔HWPX 왕복 후 파라미터 '{name}' 이 사라짐 — 왕복후={names2:?}"
        );
    }
}

/// [#4396] HWPX→HWP5→HWPX 왕복 — 현재 **알려진 한계**: HWP5 CTRL_DATA 에 스펙이
/// 규정한 슬롯이 없어 `Prop`/`Direction` 등이 손실된다(위 모듈 문서 참고). 이 테스트는
/// "손실이 없어야 한다"가 아니라 "이 손실이 지금 상태이고, 만약 이게 갑자기
/// **악화**되면(예: 필드 개수 자체가 어긋나거나 `command`/`memo_index` 처럼 실제
/// 슬롯이 있는 항목까지 사라지면) 잡아낸다"는 가드다. 손실 자체를 없애려면
/// `field_parameter_loss_warning` 문서 주석에 적힌 스펙 근거를 다시 검토해 실제
/// 규정된 슬롯을 찾아야 한다 — 그 전까지는 이 상태가 의도된 것이다.
#[test]
fn field_parameters_beyond_command_are_lost_after_hwp5_roundtrip_known_limitation() {
    let bytes = fs::read(sample_path()).expect("샘플 읽기");
    let doc1 = parse_hwpx(&bytes).expect("원본 파싱");
    let fields1 = collect_fields(&doc1);
    let multi_param_idx = fields1
        .iter()
        .position(|f| f.parameters.items.len() > 1)
        .unwrap_or_else(|| panic!("전제 실패: {SAMPLE} 에 다중 파라미터 필드가 없음"));
    let original_command = fields1[multi_param_idx].command.clone();
    assert!(
        !original_command.is_empty(),
        "전제 실패: command 가 비어있음"
    );

    let mut hwpx_doc = HwpDocument::from_bytes(&bytes).expect("HWPX 파싱(convert 준비)");
    let hwp_bytes = hwpx_doc
        .export_hwp_with_adapter()
        .expect("HWPX→HWP5 변환(rhwp convert)");
    let hwp_doc = HwpDocument::from_bytes(&hwp_bytes).expect("HWP5 재파싱");
    let final_bytes = hwp_doc
        .export_hwpx_native()
        .expect("HWP5→HWPX 변환(rhwp export-hwpx)");
    let doc3 = parse_hwpx(&final_bytes).expect("최종 HWPX 재파싱");
    let fields3 = collect_fields(&doc3);

    assert_eq!(
        fields1.len(),
        fields3.len(),
        "HWP5 왕복 후 필드 '개수' 는 어긋나면 안 됨(파라미터 손실과 별개 계약)"
    );

    let f3 = &fields3[multi_param_idx];
    // 실제 슬롯이 있는 command 는 반드시 살아남아야 한다 — 이게 깨지면 손실 범위가
    // 확대된 것(악화), #4396 이 고치려던 것보다 더 나쁜 회귀다.
    assert_eq!(
        f3.command, original_command,
        "command 자체가 손실됨 — HWP5 CTRL_HEADER 슬롯까지 깨진 심각한 회귀"
    );

    // 현재 상태(알려진 한계): Prop/Direction 처럼 슬롯이 없는 항목은 사라진다.
    // 이 assert 가 실패하면(=Prop/Direction 이 살아남으면) 스펙에 규정된 슬롯을
    // 새로 찾았다는 뜻이니 이 테스트와 모듈 문서를 함께 업데이트해야 한다.
    let names3: Vec<&str> = f3.parameters.items.iter().filter_map(param_name).collect();
    assert!(
        !names3.contains(&"Prop") && !names3.contains(&"Direction"),
        "Prop/Direction 이 HWP5 왕복 후에도 남아있음 — 스펙에 슬롯을 새로 찾았다면 \
         이 테스트를 GREEN 기대(보존)로 뒤집고 관련 문서를 갱신할 것: {names3:?}"
    );
}
