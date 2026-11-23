//! 관계형·고정형(쪽별 내용 소속, 원본 판형 및 인쇄 쪽 번호).
//! 주석 갱신 2026-11-24T03:35:00+09:00. 직전 변경 a0b8b59d4e4c710dff36d5ef78a13296f907b003 (2026-11-24T01:57:23+09:00).
//! 2026-11-24, Windows 한컴 Print 대비 96 DPI 2px raw 실루엣. B5 대조군 4종·한컴 재저장 HWP/HWPX·HWP 재생성본은 각 4쪽 모두 Native/fresh WASM 100.00%다.
//! 전쪽 측정과 원본은 `mydocs/pr/assets/semanticist21-20261124-rework/final/`에 기록한다.
//!
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

/// 시험 문서의 첫 구역만 고친 HWPX 바이트.
fn with_first_section(edit: impl Fn(&str) -> String) -> Vec<u8> {
    use std::io::{Read, Write};
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut source = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        let name = entry.name().unwrap().to_string();
        if name == "Contents/section0.xml" {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).unwrap();
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            out.start_file(name, options).unwrap();
            out.write_all(edit(&xml).as_bytes()).unwrap();
        } else {
            out.raw_copy_file(entry).unwrap();
        }
    }
    out.finish().unwrap().into_inner()
}

/// 시험 문서 첫 구역 첫 문단은 [구역 정의][쪽 번호 위치][단 정의] 차례이고, 두 구역이 같은
/// `masterpage0`을 참조한다. 한컴 12.30(Mac)은 이 차례면 첫 구역의 구역 정의를 버리고 기본값으로
/// 연다. 그래서 바탕쪽은 3·4쪽에만 보이고, 첫 구역만 B5 용지·시작 쪽 번호 5로 바꿔도 1쪽은 A4·1쪽이다.
/// 단 정의를 구역 정의 바로 뒤로 옮기면 첫 구역 설정이 살아 바탕쪽이 1·2쪽에도 보인다.
#[test]
fn first_section_def_split_from_column_def_falls_back_like_hancom() {
    let marked = |core: &DocumentCore| -> Vec<bool> {
        (0..core.page_count())
            .map(|page| page_text(core, page).contains("바탕쪽표식"))
            .collect()
    };
    let core = open();
    assert_eq!(marked(&core), [false, false, true, true]);
    let saved = rhwp::serializer::hwpx::serialize_hwpx(core.document()).unwrap();
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    assert_eq!(marked(&reopened), [false, false, true, true]);

    let b5_from_five = with_first_section(|xml| {
        xml.replace(
            r#"width="59528" height="84186""#,
            r#"width="51592" height="72852""#,
        )
        .replace(
            r#"pageStartsOn="BOTH" page="0""#,
            r#"pageStartsOn="BOTH" page="5""#,
        )
    });
    let core = DocumentCore::from_bytes(&b5_from_five).unwrap();
    assert_eq!(
        core.document().sections[0].section_def.page_def.width,
        59528
    );
    assert_eq!(page_numbers(&core), [1, 2, 3, 4]);
    assert_eq!(marked(&core), [false, false, true, true]);

    let page_number =
        r#"<hp:ctrl><hp:pageNum pos="BOTTOM_CENTER" formatType="DIGIT" sideChar="-"/></hp:ctrl>"#;
    let column = r#"<hp:ctrl><hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="1" sameSz="1" sameGap="0"/></hp:ctrl>"#;
    let reordered = with_first_section(|xml| {
        let swapped = xml.replace(
            &format!("{page_number}{column}"),
            &format!("{column}{page_number}"),
        );
        assert_ne!(swapped, xml);
        swapped
    });
    let core = DocumentCore::from_bytes(&reordered).unwrap();
    assert_eq!(marked(&core), [true; 4]);
}

/// Windows 한컴 Print의 네 쪽에서 머리말·꼬리말·본문·바탕쪽의 소속을 함께 확인한다.
#[test]
fn first_section_uses_the_character_order_and_requires_a_column_definition() {
    let samples =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/semanticist21-20261124-rework");
    for (file, adjacent) in [
        ("b5-adjacent.hwpx", true),
        ("b5-text.hwpx", false),
        ("b5-page-number.hwpx", false),
        ("b5-no-column.hwpx", false),
    ] {
        let input = std::fs::read(samples.join(file)).unwrap();
        let original = DocumentCore::from_bytes(&input).unwrap();
        let check = |doc: &DocumentCore| {
            let width = doc.document().sections[0].section_def.page_def.width;
            assert_eq!(width, if adjacent { 51592 } else { 59528 }, "{file}");
            assert_eq!(
                page_numbers(doc),
                if adjacent { [5, 6, 7, 8] } else { [1, 2, 3, 4] },
                "{file}"
            );
            for (page, body) in ["첫째", "둘째", "셋째", "넷째"].iter().enumerate() {
                let text = page_text(doc, page as u32);
                assert!(
                    text.contains(body)
                        && text.contains("머리말확인")
                        && text.contains("꼬리말확인"),
                    "{file} {page}: {text}"
                );
                assert_eq!(
                    text.contains("바탕쪽표식"),
                    adjacent || page >= 2,
                    "{file} {page}"
                );
                assert_eq!(
                    text.contains('A'),
                    file == "b5-text.hwpx" && page == 0,
                    "{file} {page}"
                );
            }
        };
        check(&original);
        for hwpx in [true, false] {
            let mut document = DocumentCore::from_bytes(&input).unwrap();
            for _ in 0..2 {
                let saved = if hwpx {
                    document.export_hwpx_native()
                } else {
                    document.export_hwp_native()
                }
                .unwrap();
                document = DocumentCore::from_bytes(&saved).unwrap();
                check(&document);
            }
        }
    }
}

#[test]
fn printed_master_text_overlaps_only_its_source_body_line() {
    use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};

    let samples =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/semanticist21-20261124-rework");
    for file in [
        "b5-adjacent.hwpx",
        "b5-no-column.hwpx",
        "b5-page-number.hwpx",
        "b5-text.hwpx",
        "two-sections-2020.hwp",
        "two-sections-2020.hwpx",
    ] {
        let doc = DocumentCore::from_bytes(&std::fs::read(samples.join(file)).unwrap()).unwrap();
        assert_eq!(doc.page_count(), 4, "{file}");
        for page in 0..doc.page_count() {
            let root = doc.build_page_render_tree(page).unwrap().root;
            let scan = scan_page(page, &root, doc.page_count(), &AnomalyOptions::default());
            // 한컴 PDF에서 바탕쪽표식과 본문의 첫 단어는 같은 원점에서 인쇄된다.
            let printed_master = file == "b5-adjacent.hwpx" || page >= 2;
            assert_eq!(
                scan.text_overlap.len(),
                usize::from(printed_master),
                "{file} {page}"
            );
            for pair in scan.text_overlap {
                let paths = [&pair.path_a, &pair.path_b];
                assert!(
                    paths.iter().any(|p| p.starts_with("Page/Body/")),
                    "{paths:?}"
                );
                assert!(
                    paths.iter().any(|p| p.starts_with("Page/MasterPage")),
                    "{paths:?}"
                );
            }
        }
    }
}

#[test]
fn rebuilding_hancom_saved_section_preserves_the_normalized_controls_and_all_fields() {
    use rhwp::diagnostics::ir_field_sweep::sweep_documents;
    use rhwp::parser::parse_document;
    use rhwp::serializer::serialize_document;

    let input = include_bytes!("../../samples/semanticist21-20261124-rework/two-sections-2020.hwp");
    let original = parse_document(input).unwrap();
    let mut edited = original.clone();
    edited.doc_info.raw_stream_dirty = true;
    for section in &mut edited.sections {
        section.raw_stream = None;
    }
    let saved = serialize_document(&edited).unwrap();
    let reopened = parse_document(&saved).unwrap();
    let mut expected = original;
    let controls = &mut expected.sections[1].paragraphs[0].controls;
    assert!(matches!(
        controls.as_slice(),
        [
            Control::SectionDef(_),
            Control::PageNumberPos(_),
            Control::ColumnDef(_),
            ..
        ]
    ));
    // 한컴 저장본의 둘째 구역에서만 쪽 번호 위치와 단 정의가 순서를 바꾼다.
    controls.swap(1, 2);
    let differences = sweep_documents(&expected, &reopened).unwrap();
    assert!(differences.is_empty(), "{differences:?}");
}
