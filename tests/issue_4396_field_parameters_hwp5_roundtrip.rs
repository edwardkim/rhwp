//! Issue #4396 — HWPX 필드 `<hp:parameters>` 가 HWP5 왕복(HWPX→HWP→HWPX) 후
//! `Command` 하나로 축소되던 손실의 회귀 가드.
//!
//! 원인: `parser/hwpx/section.rs`(`parse_field_parameters`)는 HWPX 를 파싱할 때
//! `Command`/(MEMO 의) `Number` 만 영구 모델 필드로 뽑고 나머지(`Prop`/`Direction`/
//! `HelpState`/`Path`/`Category`/`TargetType`/`DocOpenType` 등)는 `raw_parameters_xml`
//! verbatim 문자열에만 남았다. 이 필드는 HWPX 파서 전용(HWP5 경로엔 없음)이라, 문서가
//! HWP5 를 한 번이라도 거치면 `None` 이 되고, `serializer/hwpx/section.rs`
//! (`generated_field_parameters`)는 `Command` 하나만 담은 최소
//! `<hp:parameters cnt="1">` 를 조용히 합성했다 — 경고 없는 손실.
//!
//! 수정: HWPX 파서가 `<hp:parameters>` 전체를 `Field::parameters`
//! (`ParameterList` — OWPML `hp:ParameterList` 5종 그대로) 트리로도 채우고, HWP5
//! 직렬화기(`serializer/control.rs`)가 CTRL_DATA 확장 아이템(`CTRL_DATA_ITEM_PARAMS_XML`)
//! 으로 그 트리를 함께 실어 보내며, HWP5 파서(`parser/body_text.rs`)가 그 아이템에서
//! 다시 트리를 복원한다. 직렬화기는 `raw_parameters_xml` 이 없어도 이 트리가 있으면
//! 그걸로 `<hp:parameters>` 를 재조립한다.
//!
//! 이 테스트는 **의도적으로 신규 API(`Field::parameters`/`Parameter`) 를 쓰지 않는다**
//! — 수정 전 코드에는 그 타입 자체가 없어 컴파일이 안 되므로, "컴파일 실패"가 아니라
//! "돌아가지만 데이터가 축소된다"는 실제 버그를 보이려면 수정 전에도 존재하던
//! `raw_parameters_xml`(String) 하나만으로 판정해야 한다. 이 파일은 수정 전 커밋에
//! 그대로 얹어도 컴파일되고, 실행하면 실패한다(수동 확인 완료 — 아래 각 테스트의
//! 주석 참고).
//!
//! fixture: `samples/누름틀-2024.hwpx` — 이슈에 실린 정확한 재현 샘플
//! (section 0 paragraph 0 필드가 Prop(integerParam)/Command(stringParam)/
//! Direction(stringParam) 3개 파라미터를 가진 `<hp:parameters cnt="3">`).

use std::fs;
use std::path::Path;

use rhwp::model::control::{Control, Field};
use rhwp::model::document::Document;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/누름틀-2024.hwpx";

fn sample_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// 문서 순회 순서로 필드 컨트롤을 전부 모은다(섹션→문단→컨트롤 순, 안정적인 순서 —
/// 다른 회귀 테스트도 이 순서에 기대어 doc1/doc3 을 위치로 비교한다).
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

/// HWPX 원본 → HWP5(convert) → HWPX(export-hwpx) 왕복 후 실제 파일을 다시 파싱해
/// 원본과 비교한다. `rhwp convert` + `rhwp export-hwpx` + `rhwp ir-diff` 조합의
/// 이슈 재현 절차를 그대로 코드로 옮긴 것이다.
fn roundtrip_via_hwp5(bytes: &[u8]) -> Document {
    let mut hwpx_doc = HwpDocument::from_bytes(bytes).expect("HWPX 파싱(convert 준비)");
    let hwp_bytes = hwpx_doc
        .export_hwp_with_adapter()
        .expect("HWPX→HWP5 변환(rhwp convert)");
    let hwp_doc = HwpDocument::from_bytes(&hwp_bytes).expect("HWP5 재파싱");
    let final_bytes = hwp_doc
        .export_hwpx_native()
        .expect("HWP5→HWPX 변환(rhwp export-hwpx)");
    parse_hwpx(&final_bytes).expect("최종 HWPX 재파싱(rhwp ir-diff 가 하는 것과 동일)")
}

/// [#4396] 이슈에 실린 정확한 사례: section 0 paragraph 0 필드가 원본에서
/// `cnt="3"`(Prop/Command/Direction) 인데, HWP5 왕복 후 재파싱한 `raw_parameters_xml`
/// 이 `cnt="1"`(Command 하나) 로 조용히 축소되면 안 된다.
///
/// **수정 전 실패 확인**: 이 테스트 파일을 수정 전 커밋(8개 소스 파일 원복)에 얹고
/// 돌리면 `actual` 이 `Some("<hp:parameters cnt=\"1\" name=\"\">...")` 로 나와
/// `contains("Prop")`/`contains("Direction")` 단언이 실패한다 — 수동으로 확인했다
/// (patch 를 되돌리고 실행 → panic, 되살리고 실행 → pass).
#[test]
fn field_parameters_not_collapsed_to_command_only_after_hwp5_roundtrip() {
    let bytes = fs::read(sample_path()).expect("샘플 읽기");
    let doc1 = parse_hwpx(&bytes).expect("원본 파싱");
    let fields1 = collect_fields(&doc1);
    assert!(!fields1.is_empty(), "샘플에 필드가 있어야 함");

    // 전제 확인: 원본에 cnt>1(=Command 하나로 축소되면 정보를 잃는) 필드가 실제로
    // 있다 — 이슈가 보고한 정확한 상황. cnt 속성 문자열 검사로 신규 API 없이 확인.
    let multi_param_field_idx = fields1
        .iter()
        .position(|f| {
            f.raw_parameters_xml
                .as_deref()
                .is_some_and(|xml| !xml.contains("cnt=\"1\""))
        })
        .unwrap_or_else(|| panic!("전제 실패: {SAMPLE} 에 다중 파라미터 필드가 없음"));
    let original_xml = fields1[multi_param_field_idx]
        .raw_parameters_xml
        .clone()
        .unwrap();
    assert!(
        original_xml.contains("name=\"Prop\"") && original_xml.contains("name=\"Direction\""),
        "전제 실패: 원본 필드[{multi_param_field_idx}] 에 Prop/Direction 이 없음: {original_xml}"
    );

    let doc3 = roundtrip_via_hwp5(&bytes);
    let fields3 = collect_fields(&doc3);
    assert_eq!(
        fields1.len(),
        fields3.len(),
        "HWP5 왕복 전후 필드 개수가 달라짐"
    );

    let roundtripped_xml = fields3[multi_param_field_idx].raw_parameters_xml.clone();
    assert!(
        roundtripped_xml
            .as_deref()
            .is_some_and(|xml| xml.contains("name=\"Prop\"")),
        "필드[{multi_param_field_idx}]: HWP5 왕복 후 Prop 파라미터가 사라짐(Command 하나로 \
         축소됨) — 원본={original_xml:?} 왕복후={roundtripped_xml:?}"
    );
    assert!(
        roundtripped_xml
            .as_deref()
            .is_some_and(|xml| xml.contains("name=\"Direction\"")),
        "필드[{multi_param_field_idx}]: HWP5 왕복 후 Direction 파라미터가 사라짐(Command \
         하나로 축소됨) — 원본={original_xml:?} 왕복후={roundtripped_xml:?}"
    );
}

/// [#4396] 위 테스트의 일반화 — 샘플 전체에서 원본이 다중 파라미터를 가진 필드
/// **개수**를 세고, HWP5 왕복 후에도 (재파싱된) `raw_parameters_xml` 이 여전히
/// `cnt="1"` 하나로 뭉개지지 않은 필드 개수가 그 이하로 줄지 않아야 한다.
#[test]
fn multi_param_field_count_does_not_shrink_after_hwp5_roundtrip() {
    let bytes = fs::read(sample_path()).expect("샘플 읽기");
    let doc1 = parse_hwpx(&bytes).expect("원본 파싱");
    let fields1 = collect_fields(&doc1);

    fn is_multi_param(f: &Field) -> bool {
        f.raw_parameters_xml
            .as_deref()
            .is_some_and(|xml| !xml.contains("cnt=\"1\""))
    }

    let original_multi_count = fields1.iter().filter(|f| is_multi_param(f)).count();
    assert!(
        original_multi_count > 0,
        "전제 실패: {SAMPLE} 에 다중 파라미터 필드가 없음"
    );

    let doc3 = roundtrip_via_hwp5(&bytes);
    let fields3 = collect_fields(&doc3);
    let roundtripped_multi_count = fields3.iter().filter(|f| is_multi_param(f)).count();

    assert_eq!(
        original_multi_count, roundtripped_multi_count,
        "HWP5 왕복 후 다중 파라미터 필드 개수가 {original_multi_count} → \
         {roundtripped_multi_count} 로 줄었음(#4396 축소 재발)"
    );
}
