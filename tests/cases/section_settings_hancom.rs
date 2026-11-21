//! 검사 유형: 관계형(한컴 12.30 Mac PDF로 확인한 구역 설정 계약).
//!
//! 시험 문서 `two-sections.hwpx`는 구역마다 두 쪽이고, 쪽마다 바탕쪽·머리말·꼬리말과 가운데 아래 쪽
//! 번호가 있다. 첫 문단의 컨트롤 차례는 [구역 정의][쪽 번호 위치][단 정의]다.
//! 한컴 한글 12.30(Mac, 2026-11-18) PDF로 저장해 본 결과:
//! - 시작 쪽 번호 홀수·짝수는 빈 쪽을 넣지 않고, 이어 받을 번호가 그 홀짝이 아니면 하나 건너뛴다.
//!   둘째 구역 짝수: 1, 2, 4, 5. 첫 구역이 2부터인 문서의 둘째 구역 홀수: 2, 3, 5, 6.
//!   첫 구역 짝수: 2, 3, 4, 5.
//! - 구역의 표 시작 번호 7은 그 구역 첫 표를 표 7로 매기고 뒤 표는 8로 잇는다.
//! - HWP 첫 문단에서 구역 정의 바로 뒤에 단 정의가 없으면 한컴은 첫 구역의 시작 쪽 번호와 기본 탭
//!   간격을 무시한다(우리 HWP를 [구역 정의][단 정의][쪽 번호 위치]로만 바꾸면 10, 11, 12, 13).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const SAMPLE: &str = "tests/fixtures/section_settings_hancom/two-sections.hwpx";

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

fn page_numbers(core: &DocumentCore) -> Vec<u64> {
    (0..core.page_count())
        .map(|page| {
            let info: serde_json::Value =
                serde_json::from_str(&core.get_page_info_native(page).unwrap()).unwrap();
            info["pageNumber"].as_u64().expect("pageNumber")
        })
        .collect()
}

/// 쪽 SVG의 글자 노드를 이어 붙인다(글자마다 따로 놓여도 찾을 수 있게).
fn page_text(core: &DocumentCore, page: u32) -> String {
    let svg = core.render_page_svg_native(page).unwrap();
    svg.split("</text>")
        .filter_map(|chunk| chunk.rsplit_once('>').map(|(_, text)| text))
        .collect()
}

#[test]
fn odd_even_page_start_skips_one_number_without_a_blank_page() {
    let mut core = open();
    assert_eq!(page_numbers(&core), [1, 2, 3, 4]);
    core.set_section_def_native(1, r#"{"pageNumType":2}"#)
        .unwrap();
    assert_eq!(page_numbers(&core), [1, 2, 4, 5]);
    core.set_section_def_native(1, r#"{"pageNumType":1}"#)
        .unwrap();
    assert_eq!(page_numbers(&core), [1, 2, 3, 4]);
    core.set_section_def_native(0, r#"{"pageNum":2}"#).unwrap();
    assert_eq!(page_numbers(&core), [2, 3, 5, 6]);
    core.set_section_def_native(0, r#"{"pageNum":0,"pageNumType":2}"#)
        .unwrap();
    core.set_section_def_native(1, r#"{"pageNumType":0}"#)
        .unwrap();
    assert_eq!(page_numbers(&core), [2, 3, 4, 5]);
    // 사용자 시작 번호가 있으면 홀짝 종류보다 앞선다.
    core.set_section_def_native(1, r#"{"pageNum":10,"pageNumType":1}"#)
        .unwrap();
    assert_eq!(page_numbers(&core), [2, 3, 10, 11]);

    for bytes in [
        core.export_hwp_with_adapter_snapshot().unwrap(),
        rhwp::serializer::hwpx::serialize_hwpx(core.document()).unwrap(),
    ] {
        let mut reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(page_numbers(&reopened), [2, 3, 10, 11]);
        reopened
            .set_section_def_native(1, r#"{"pageNum":0,"pageNumType":2}"#)
            .unwrap();
        assert_eq!(page_numbers(&reopened), [2, 3, 4, 5]);
    }
}

#[test]
fn section_object_start_numbers_renumber_captions_on_edit() {
    let mut core = open();
    for section in [1, 0] {
        for _ in 0..2 {
            let created: serde_json::Value =
                serde_json::from_str(&core.create_table_native(section, 0, 0, 1, 1).unwrap())
                    .unwrap();
            let para = created["paraIdx"].as_u64().unwrap() as usize;
            let control = created["controlIdx"].as_u64().unwrap() as usize;
            core.set_table_properties_native(
                section,
                para,
                control,
                r#"{"hasCaption":true,"captionDirection":3}"#,
            )
            .unwrap();
        }
    }
    let captions = |core: &DocumentCore| {
        (0..core.page_count())
            .map(|page| page_text(core, page))
            .collect::<String>()
    };
    let before = captions(&core);
    assert!(before.contains("표1") && before.contains("표4"), "{before}");

    core.set_section_def_native(1, r#"{"tableNum":7}"#).unwrap();
    let after = captions(&core);
    for label in ["표1", "표2", "표7", "표8"] {
        assert!(after.contains(label), "{label} 없음: {after}");
    }
    assert!(!after.contains("표3"), "{after}");

    core.set_section_def_all_native(r#"{"tableNum":0}"#)
        .unwrap();
    core.set_section_def_native(0, r#"{"tableNum":3}"#).unwrap();
    let continued = captions(&core);
    for label in ["표3", "표4", "표5", "표6"] {
        assert!(continued.contains(label), "{label} 없음: {continued}");
    }
}

#[test]
fn hwp_save_puts_column_def_right_after_section_def() {
    let mut core = open();
    let leading = |core: &DocumentCore, section: usize| -> Vec<&'static str> {
        core.document().sections[section].paragraphs[0]
            .controls
            .iter()
            .take(3)
            .map(|control| match control {
                Control::SectionDef(_) => "secd",
                Control::ColumnDef(_) => "cold",
                Control::PageNumberPos(_) => "pgnp",
                _ => "other",
            })
            .collect()
    };
    assert_eq!(leading(&core, 0), ["secd", "pgnp", "cold"]);
    core.set_section_def_all_native(r#"{"pageNum":10,"defaultTabSpacing":4000}"#)
        .unwrap();
    let saved = core.export_hwp_with_adapter_snapshot().unwrap();
    // 저장은 살아 있는 문서를 바꾸지 않는다.
    assert_eq!(leading(&core, 0), ["secd", "pgnp", "cold"]);

    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    for section in 0..2 {
        assert_eq!(leading(&reopened, section), ["secd", "cold", "pgnp"]);
        let def: serde_json::Value =
            serde_json::from_str(&reopened.get_section_def_native(section).unwrap()).unwrap();
        assert_eq!(def["pageNum"], 10);
        assert_eq!(def["defaultTabSpacing"], 4000);
    }
    assert_eq!(page_numbers(&reopened), [10, 11, 10, 11]);
    assert_eq!(
        reopened.document().sections[0].paragraphs[0].text,
        core.document().sections[0].paragraphs[0].text
    );
}

/// `hancom-even-start.hwp`: 둘째 구역을 짝수 시작으로 둔 시험 문서를 한컴 12.30이 HWP로 저장한 것.
/// 한컴은 HWP 구역 정의 속성 bit 20-21에 짝수를 1, 홀수를 2로 쓴다.
#[test]
fn hwp_page_start_bits_follow_hancom() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/section_settings_hancom/hancom-even-start.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let kind = |core: &DocumentCore, section: usize| {
        let def: serde_json::Value =
            serde_json::from_str(&core.get_section_def_native(section).unwrap()).unwrap();
        def["pageNumType"].as_u64().unwrap()
    };
    assert_eq!(kind(&core, 1), 2);
    assert_eq!(page_numbers(&core), [1, 2, 4, 5]);

    core.set_section_def_native(1, r#"{"pageNumType":1}"#)
        .unwrap();
    let saved = core.export_hwp_with_adapter_snapshot().unwrap();
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    assert_eq!(kind(&reopened, 1), 1);
    assert_eq!(
        (reopened.document().sections[1].section_def.flags >> 20) & 0x03,
        2
    );
    assert_eq!(page_numbers(&reopened), [1, 2, 3, 4]);
}
