//! [Issue #8303] HWP3 출처 HWPX 를 다시 저장해도 줄 시작(`textpos`)이 흘러내리지 않는다.
//!
//! 종전 저장기는 HWP3 원본의 `textpos` 를 구역·단 정의 자리(`secd`·`cold`)만큼 내려 썼고
//! (#5943), HWP3 출처 마커의 제어 축 복원이 원본 IR 의 보정폭(0)으로 파서가 센 값을 덮어써
//! 다시 저장할 때마다 같은 폭을 또 뺐다(`hwp3-table-caption` 구역 0 문단 0 줄 1:
//! 62 → 46 → 30 → 14 → 0).
//!
//! 기대값은 한컴이다. 한컴 2020(11.0.0.9136)·2024(13.0.0.3901)가 같은 HWP3 을 HWPX 로
//! 저장하면 이 줄을 `textpos="62"` 로 쓰고, 한컴 2020 Print 는 그 파일을 원본 HWP3 과 같은
//! 자리에 인쇄한다. 46 으로 내린 파일은 표를 한 줄(1692HU) 아래에 인쇄한다. 그래서 HWP3
//! 출처 HWPX 는 HWP5 출처와 같은 문단 축으로 낸다(표지 `rhwp-hwp3-paragraph-axis`).

use rhwp::document_core::DocumentCore;
use rhwp::model::document::Document;
use rhwp::serializer::hwpx::roundtrip::diff_documents;
use std::io::{Read, Write};

/// 코퍼스의 HWP3 문서 전부(암호 문서는 비밀번호와 함께).
const HWP3_SAMPLES: &[(&str, Option<&str>)] = &[
    ("samples/hwp3-curve.hwp", None),
    ("samples/hwp3-ellipse-empty-textbox.hwp", None),
    ("samples/hwp3-empty-cell.hwp", None),
    ("samples/hwp3-pagedef-1915.hwp", None),
    ("samples/HWP3-password-123456.hwp", Some("123456")),
    ("samples/hwp3-sample.hwp", None),
    ("samples/hwp3-sample10.hwp", None),
    ("samples/hwp3-sample11.hwp", None),
    ("samples/hwp3-sample13.hwp", None),
    ("samples/hwp3-sample14.hwp", None),
    ("samples/hwp3-sample16.hwp", None),
    ("samples/hwp3-sample19.hwp", None),
    ("samples/hwp3-sample4.hwp", None),
    ("samples/hwp3-sample5.hwp", None),
    ("samples/hwp3-table-caption.hwp", None),
    ("samples/hwp3-table-cell-order.hwp", None),
    ("samples/hwp3-table-cell-overlap.hwp", None),
    ("samples/hwp3-table-grid-gap.hwp", None),
    ("samples/issue_265.hwp", None),
    ("samples/issue1892_hwp3_drawing_group_roundtrip.hwp", None),
    ("samples/issue1892_hwp3_tab_roundtrip.hwp", None),
    ("samples/issue1950_hwp3_tab_charoffset.hwp", None),
    (
        "samples/issue4680/20117321_gijang_handover_bond_ledger_form.hwp",
        None,
    ),
    ("samples/issue5699/20099369_yeongwol_forms.hwp", None),
    ("samples/issue5916/05434_vehicle_log_form.hwp", None),
    ("samples/issue6266/seizure_list_form_button.hwp", None),
    ("samples/SO-SUEOP.hwp", None),
];

const AXIS_MARKER: &str = "META-INF/rhwp-hwp3-paragraph-axis";

fn open(path: &str, password: Option<&str>) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    match password {
        Some(p) => DocumentCore::from_bytes_with_password(&bytes, p.as_bytes()),
        None => DocumentCore::from_bytes(&bytes),
    }
    .unwrap_or_else(|e| panic!("{path} 파싱 실패: {e}"))
}

fn resave_as_hwpx(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX 저장"))
        .expect("HWPX 다시 열기")
}

/// 본문 문단마다 줄 시작 목록. `lift` 면 HWP5 축으로 올린 값이다.
fn body_line_starts(doc: &Document, lift: bool) -> Vec<Vec<Vec<u32>>> {
    doc.sections
        .iter()
        .map(|section| {
            section
                .paragraphs
                .iter()
                .map(|para| {
                    (0..para.line_segs.len())
                        .map(|i| {
                            if lift {
                                para.line_seg_text_start(i)
                            } else {
                                para.line_segs[i].text_start
                            }
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

/// 본문 문단 0 줄 1(표 줄, `vertpos="1692"`)의 textpos. 문단 0 의 `linesegarray` 는 표 안의
/// 캡션·셀 문단 뒤에 오므로 vertpos 로 그 줄을 고른다.
fn table_line_textpos(hwpx: &[u8]) -> Vec<u32> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("HWPX ZIP");
    let mut xml = String::new();
    zip.by_name("Contents/section0.xml")
        .expect("section0.xml")
        .read_to_string(&mut xml)
        .expect("section0.xml 읽기");
    xml.split("<hp:lineseg ")
        .skip(1)
        .filter(|tag| tag.contains(r#"vertpos="1692" vertsize="67616""#))
        .map(|tag| {
            tag.split("textpos=\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .and_then(|value| value.parse().ok())
                .expect("textpos")
        })
        .collect()
}

/// 이슈의 재현: 다시 저장할 때마다 16씩 줄던 줄 시작이 한컴 저장본과 같은 62 에 머문다.
#[test]
fn hwp3_table_caption_line_start_survives_repeated_hwpx_save() {
    let source = open("samples/hwp3-table-caption.hwp", None);
    assert_eq!(
        source.document().sections[0].paragraphs[0].line_segs[1].text_start,
        62,
        "원본 HWP3"
    );
    let mut bytes = source.export_hwpx_native().expect("HWPX 저장");
    for generation in 1..=4 {
        assert_eq!(
            table_line_textpos(&bytes),
            [62],
            "{generation}세대 파일: 한컴 2020·2024 저장본의 textpos"
        );
        let reopened = DocumentCore::from_bytes(&bytes).expect("HWPX 다시 열기");
        assert_eq!(
            reopened.document().sections[0].paragraphs[0].line_seg_text_start(1),
            62,
            "{generation}세대를 다시 연 HWP5 축 줄 시작"
        );
        bytes = reopened.export_hwpx_native().expect("HWPX 다시 저장");
    }
}

/// 호환: 축 표지가 없는 이전 산출물(`textpos` 를 구역·단 정의 자리만큼 내려 쓴 파일)은
/// 종전처럼 그 폭을 올려 읽고, 다시 저장하면 새 축으로 옮긴다.
#[test]
fn legacy_hwp3_origin_hwpx_without_axis_marker_is_lifted_once() {
    let current = open("samples/hwp3-table-caption.hwp", None)
        .export_hwpx_native()
        .expect("HWPX 저장");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&current)).expect("HWPX ZIP");
    let mut legacy = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let mut found_marker = false;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).expect("엔트리");
        let name = entry.name().expect("엔트리 이름").to_string();
        if name == AXIS_MARKER {
            found_marker = true;
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("엔트리 읽기");
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).expect("UTF-8");
            assert_eq!(xml.matches(r#"textpos="62""#).count(), 1);
            data = xml
                .replace(r#"textpos="62""#, r#"textpos="46""#)
                .into_bytes();
        }
        let method = if name == "mimetype" {
            zip::CompressionMethod::Stored
        } else {
            zip::CompressionMethod::Deflated
        };
        legacy
            .start_file(
                name,
                zip::write::SimpleFileOptions::default().compression_method(method),
            )
            .expect("엔트리 쓰기");
        legacy.write_all(&data).expect("엔트리 쓰기");
    }
    assert!(found_marker, "새 산출물은 축 표지를 싣는다");
    let legacy = legacy.finish().expect("ZIP 닫기").into_inner();

    let reopened = DocumentCore::from_bytes(&legacy).expect("이전 산출물 열기");
    let para = &reopened.document().sections[0].paragraphs[0];
    assert_eq!(
        (para.line_segs[1].text_start, para.line_seg_text_start(1)),
        (46, 62)
    );
    let resaved = reopened.export_hwpx_native().expect("다시 저장");
    assert_eq!(table_line_textpos(&resaved), [62], "새 축으로 저장");
}

/// 코퍼스의 HWP3 문서 전부: 1세대 파일은 원본 HWP3 의 줄 시작을 그대로 싣고, 다시 열거나
/// HWP5 로 옮겨도 원본과 같으며, 2세대는 1세대와 같다.
#[test]
fn hwp3_origin_hwpx_resave_is_idempotent_and_keeps_source_axis() {
    let mut compared_lines = 0usize;
    for &(path, password) in HWP3_SAMPLES {
        let source = open(path, password);
        let gen1 = resave_as_hwpx(&source);
        let gen2 = resave_as_hwpx(&gen1);

        let diff = diff_documents(gen1.document(), gen2.document());
        assert!(
            diff.differences.is_empty(),
            "{path}: 1세대 → 2세대 IR 차이 {:?}",
            diff.differences
                .iter()
                .map(|d| d.to_string())
                .take(5)
                .collect::<Vec<_>>()
        );
        // `diff_documents` 는 HWP5↔HWPX 축 교차 비교를 위해 `textpos` 이동을 일부 허용하므로
        // 파일 값 그대로도 비교한다.
        assert_eq!(
            body_line_starts(gen1.document(), false),
            body_line_starts(gen2.document(), false),
            "{path}: 1세대 → 2세대 파일 줄 시작"
        );

        // 같은 축 값을 소비하는 경로: 다시 읽은 IR 의 HWP5 축 투영, 그리고 그 IR 을 HWP5 로
        // 저장할 때 `textpos` 를 HWP5 축으로 쓰는 저장기(`body_text.rs`).
        let hwp5 = DocumentCore::from_bytes(&gen1.export_hwp_with_adapter().expect("HWP5 저장"))
            .expect("HWP5 다시 열기");
        let original = body_line_starts(source.document(), false);
        for (route, observed) in [
            ("HWPX 다시 열기", body_line_starts(gen1.document(), true)),
            ("HWPX → HWP5", body_line_starts(hwp5.document(), false)),
        ] {
            assert_eq!(original.len(), observed.len(), "{path} {route}: 구역 수");
            for (si, (a, b)) in original.iter().zip(&observed).enumerate() {
                assert_eq!(a.len(), b.len(), "{path} {route}: 구역 {si} 문단 수");
                for (pi, (la, lb)) in a.iter().zip(b).enumerate() {
                    if la.len() != lb.len() {
                        continue;
                    }
                    compared_lines += la.len();
                    assert_eq!(
                        la, lb,
                        "{path} {route}: 구역 {si} 문단 {pi} 의 HWP5 축 줄 시작"
                    );
                }
            }
        }
    }
    assert!(compared_lines > 2000, "비교한 줄 {compared_lines}");
}
