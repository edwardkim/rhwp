//! 관계형: 원본과 편집·저장본의 CTRL_DATA 소유 경로·순서·바이트를 대조한다.
//! 주석 갱신 2026-11-22T02:48:47+09:00. 직전 파일 변경 2026-10-31T23:10:00+07:00, 1feecd87284777a1ddcfe4eeafd745abe9cc1894.
//! HWP5 바이너리 보존 검사다. 측정 대상은 구역 레코드이며 조판 좌표와 시각 일치율은 비해당이다.
//! 실물 비교 근거는 mydocs/report/task_m100_8225_report.md와 mydocs/pr/archives/pr_8233_review.md에 있다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::Path;

use rhwp::parser::{record::Record, tags};

#[derive(Debug, PartialEq, Eq, Clone)]
struct Owner {
    tag: u16,
    // 같은 종류의 그림이 같은 구역에 여럿 있을 수 있다.
    object: Option<(u32, usize)>,
}

#[derive(Debug, PartialEq, Eq)]
struct CtrlData {
    section: String,
    owners: Vec<Owner>,
    data: Vec<u8>,
}

fn ctrl_data_records(hwp: &[u8]) -> Vec<CtrlData> {
    let mut cfb = cfb::CompoundFile::open(Cursor::new(hwp)).expect("CFB 열기");
    let mut header = Vec::new();
    cfb.open_stream("/FileHeader")
        .unwrap()
        .read_to_end(&mut header)
        .unwrap();
    let compressed = u32::from_le_bytes(header[36..40].try_into().unwrap()) & 1 != 0;
    let mut paths: Vec<_> = cfb
        .walk()
        .filter(|e| e.is_stream() && e.path().parent() == Some(Path::new("/BodyText")))
        .map(|e| e.path().to_owned())
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let section = path.file_name().unwrap().to_str().unwrap();
        if !section.starts_with("Section") {
            continue;
        }
        let mut stream = cfb.open_stream(&path).unwrap();
        let mut body = Vec::new();
        if compressed {
            flate2::read::DeflateDecoder::new(stream)
                .read_to_end(&mut body)
                .unwrap();
        } else {
            stream.read_to_end(&mut body).unwrap();
        }
        let mut owners = Vec::new();
        let mut ids = HashMap::new();
        for record in Record::read_all(&body).unwrap() {
            owners.truncate(usize::from(record.level));
            let tag = record.tag_id;
            let object = if matches!(tag, tags::HWPTAG_CTRL_HEADER | tags::HWPTAG_SHAPE_COMPONENT) {
                let id = u32::from_le_bytes(record.data[..4].try_into().unwrap());
                let index = ids.entry((tag, id)).or_insert(0);
                *index += 1;
                Some((id, *index))
            } else {
                None
            };
            if tag == tags::HWPTAG_CTRL_DATA {
                out.push(CtrlData {
                    section: section.to_owned(),
                    owners: owners.clone(),
                    data: record.data,
                });
            }
            owners.push(Owner { tag, object });
        }
    }
    out
}

fn group_child_ctrl_data(hwp: &[u8]) -> Vec<Vec<u8>> {
    ctrl_data_records(hwp).into_iter()
        .filter(|r| matches!(r.owners.as_slice(), [.., parent, child]
            if parent.tag == tags::HWPTAG_SHAPE_COMPONENT && child.tag == tags::HWPTAG_SHAPE_COMPONENT))
        .map(|r| r.data).collect()
}

/// 구역을 모두 다시 직렬화한 HWP 바이트 (편집기에서 글을 고친 뒤 저장한 것과 같은 경로)
fn rewritten(bytes: &[u8]) -> Vec<u8> {
    let mut doc = rhwp::parse_document(bytes).expect("문서 읽기");
    for section in &mut doc.sections {
        section.raw_stream = None;
    }
    rhwp::serializer::serialize_document(&doc).expect("저장")
}

#[test]
fn group_child_ctrl_data_survives_section_rewrite() {
    let original =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/sungeo.hwp"))
            .expect("samples/basic/sungeo.hwp");
    let expected = group_child_ctrl_data(&original);
    assert_eq!(
        expected.len(),
        24,
        "시험 문서의 묶음 자식 CTRL_DATA 수가 바뀌었다"
    );
    let text: String = expected
        .iter()
        .map(|d| {
            String::from_utf16_lossy(
                &d.chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert!(
        text.contains("항 내용"),
        "자식 CTRL_DATA 에 개체 설명 글이 있어야 한다"
    );

    let once = rewritten(&original);
    assert_eq!(
        group_child_ctrl_data(&once),
        expected,
        "다시 쓴 구역에서 자식 도형 CTRL_DATA 가 달라졌다"
    );

    // 저장본을 다시 열어 또 저장해도 그대로 (모델에 남는다)
    let twice = rewritten(&once);
    assert_eq!(group_child_ctrl_data(&twice), expected);
}

/// 반례: 묶음 자신에게는 CTRL_DATA 가 없고 첫 자식 그림에만 있다(하이퍼링크). 컨트롤 몫으로
/// 자식 것을 가져가면 묶음 아래와 자식 아래에 두 번 쓰인다.
#[test]
fn child_only_ctrl_data_is_not_written_twice() {
    let original = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue6542/156678235_mid_para_vpos_rewind.hwp"),
    )
    .expect("samples/issue6542/156678235_mid_para_vpos_rewind.hwp");
    assert_eq!(
        group_child_ctrl_data(&original).len(),
        1,
        "자식 그림 하나에만 CTRL_DATA 가 있는 문서여야 한다"
    );
    let out = rewritten(&original);
    assert_eq!(
        ctrl_data_records(&out).len(),
        ctrl_data_records(&original).len(),
        "CTRL_DATA 수가 원본과 달라졌다"
    );
    assert_eq!(
        group_child_ctrl_data(&out),
        group_child_ctrl_data(&original)
    );
}

/// 다른 한컴 문서에서도 묶음 자식 CTRL_DATA 가 남는다 (이슈에 적힌 1개 손실 문서)
#[test]
fn group_child_ctrl_data_survives_in_multi_section_document() {
    let original =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp-multi-001.hwp"))
            .expect("samples/hwp-multi-001.hwp");
    let expected = group_child_ctrl_data(&original);
    assert!(
        !expected.is_empty(),
        "묶음 자식 CTRL_DATA 가 있는 문서여야 한다"
    );
    let out = rewritten(&original);
    assert_eq!(group_child_ctrl_data(&out), expected);
    assert_eq!(
        ctrl_data_records(&out).len(),
        ctrl_data_records(&original).len()
    );
}

#[test]
fn every_ctrl_data_keeps_its_owner_across_two_edits() {
    for (name, count) in [
        ("samples/basic/sungeo.hwp", 60),
        ("samples/20250130-hongbo.hwp", 2),
        ("samples/task2137/156618554_petfood_press.hwp", 2),
        (
            "samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp",
            2,
        ),
    ] {
        let original = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap();
        let expected = ctrl_data_records(&original);
        assert_eq!(expected.len(), count, "{name}");
        let mut editor = rhwp::DocumentCore::from_bytes(&original).unwrap();
        let mut text = editor.document().sections[0].paragraphs[0].text.clone();
        for round in 1..=2 {
            editor.insert_text_native(0, 0, 0, "가").unwrap();
            text.insert(0, '가');
            let saved = editor.export_hwp_native().unwrap();
            assert_eq!(
                ctrl_data_records(&saved),
                expected,
                "{name}, 저장 {round}회"
            );
            editor = rhwp::DocumentCore::from_bytes(&saved).unwrap();
            assert_eq!(
                editor.document().sections[0].paragraphs[0].text,
                text,
                "{name}, 재열기 {round}회"
            );
        }
    }
}

#[test]
fn header_and_component_slots_are_independent_of_caption_controls() {
    use rhwp::model::{
        control::{Bookmark, Control},
        image::Picture,
        paragraph::Paragraph,
        shape::Caption,
    };
    use rhwp::parser::body_text::parse_paragraph;
    use rhwp::serializer::control::serialize_control;

    let picture = Control::Picture(Box::new(Picture {
        caption: Some(Caption {
            paragraphs: vec![Paragraph {
                controls: vec![Control::Bookmark(Bookmark {
                    name: "캡션".into(),
                })],
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }));
    let header_packet: &[u8] = &[0x34, 0x12, 0, 0, 0, 0];
    let component_packet: &[u8] = &[0x35, 0x12, 0, 0, 0, 0];
    for header_data in [None, Some(header_packet)] {
        for component_data in [None, Some(component_packet)] {
            // 그림의 캡션에도 별도의 CTRL_HEADER와 책갈피 데이터가 있다.
            let mut source = vec![Record {
                tag_id: tags::HWPTAG_PARA_HEADER,
                level: 0,
                size: 24,
                data: vec![0; 24],
            }];
            serialize_control(&picture, 1, None, &mut source);
            for (owner_tag, owner_level, packet) in [
                (tags::HWPTAG_CTRL_HEADER, 1, header_data),
                (tags::HWPTAG_SHAPE_COMPONENT, 2, component_data),
            ] {
                if let Some(data) = packet {
                    let owner = source
                        .iter()
                        .position(|r| r.tag_id == owner_tag && r.level == owner_level)
                        .unwrap();
                    source.insert(
                        owner + 1,
                        Record {
                            tag_id: tags::HWPTAG_CTRL_DATA,
                            level: owner_level + 1,
                            size: data.len() as u32,
                            data: data.to_vec(),
                        },
                    );
                }
            }
            let packets = |records: &[Record]| -> Vec<_> {
                records
                    .iter()
                    .filter(|r| r.tag_id == tags::HWPTAG_CTRL_DATA)
                    .map(|r| (r.level, r.data.clone()))
                    .collect()
            };
            let expected = packets(&source);
            assert_eq!(
                expected.len(),
                1 + usize::from(header_data.is_some()) + usize::from(component_data.is_some())
            );
            let paragraph = parse_paragraph(&source).unwrap();
            let mut saved = Vec::new();
            serialize_control(
                &paragraph.controls[0],
                1,
                paragraph.ctrl_data_records[0].as_deref(),
                &mut saved,
            );
            assert_eq!(packets(&saved), expected);
        }
    }
}
