//! #7899: 본문 편집은 독립된 구역 끝 목록의 저장 소유를 무효화하지 않는다.
//! 기대값은 한컴 저장 원본의 tag/level/payload와 목록 순서다. 조판 golden은 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::parser::cfb_reader::CfbReader;
use rhwp::serializer::{serialize_hwp, SerializeError};

type RecordTuple = (u16, u16, Vec<u8>);

fn sample(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join(name),
    )
    .expect("공개 한컴 저장 원본")
}

// 구현의 tail 모델·경계 helper를 사용하지 않고 파일 레코드 헤더를 직접 해독한다.
fn section_records(bytes: &[u8], section: usize) -> Vec<RecordTuple> {
    let mut reader = CfbReader::open(bytes).expect("CFB");
    let header = reader.read_file_header().unwrap();
    let compressed = u32::from_le_bytes(header[36..40].try_into().unwrap()) & 1 != 0;
    let stream = reader
        .read_body_text_section(section as u32, compressed, false)
        .unwrap();
    let mut cursor = 0;
    let mut records = Vec::new();
    while cursor < stream.len() {
        let head = u32::from_le_bytes(stream[cursor..cursor + 4].try_into().unwrap());
        cursor += 4;
        let mut size = (head >> 20) as usize;
        if size == 4095 {
            size = u32::from_le_bytes(stream[cursor..cursor + 4].try_into().unwrap()) as usize;
            cursor += 4;
        }
        records.push((
            (head & 1023) as u16,
            ((head >> 10) & 1023) as u16,
            stream[cursor..cursor + size].to_vec(),
        ));
        cursor += size;
    }
    records
}

fn tail(bytes: &[u8], section: usize) -> Vec<RecordTuple> {
    let records = section_records(bytes, section);
    let last = records.iter().rposition(|r| r.0 == 66 && r.1 == 0).unwrap();
    let start = records
        .iter()
        .enumerate()
        .skip(last + 1)
        .find(|(_, r)| r.1 == 1 && matches!(r.0, 72 | 93))
        .map(|(i, _)| i);
    start.map_or_else(Vec::new, |i| records[i..].to_vec())
}

fn edit(bytes: &[u8], section: usize, text: &str) -> Vec<u8> {
    let mut core = DocumentCore::from_bytes(bytes).unwrap();
    core.insert_text_native(section, 0, 0, text).unwrap();
    let saved = core.export_hwp_native().expect("본문 편집 저장");
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    assert!(
        reopened.document().sections[section].paragraphs[0]
            .text
            .starts_with(text),
        "원본 스트림을 반환해 본문 편집을 버리면 안 된다"
    );
    saved
}

#[test]
fn extension_master_lists_survive_body_edit_in_source_order() {
    for name in ["exam_math_8.hwp", "exam_science.hwp"] {
        let original = sample(name);
        let expected = tail(&original, 0);
        assert!(!expected.is_empty());
        let kinds: Vec<_> = expected
            .iter()
            .filter(|r| r.0 == 72 && r.1 == 1)
            .map(|r| u16::from_le_bytes(r.2[18..20].try_into().unwrap()))
            .collect();
        assert!(kinds.contains(&4), "독립 원본에 유실된 종류 4가 있다");
        if name == "exam_science.hwp" {
            assert!(kinds.contains(&5) && kinds.contains(&6) && kinds.contains(&7));
        }
        assert_eq!(
            tail(&edit(&original, 0, "A"), 0),
            expected,
            "{name}: 목록 본문·미해석 레코드·소속·순서를 모두 보존"
        );
    }
}

#[test]
fn native_memo_numbers_and_contents_survive_body_edit() {
    for name in ["aift.hwp", "issue5866/memo_field_hwp5.hwp"] {
        let original = sample(name);
        let core = DocumentCore::from_bytes(&original).unwrap();
        let section = core.document().sections.len() - 1;
        let expected = tail(&original, section);
        let numbers: Vec<_> = expected
            .iter()
            .filter(|r| r.0 == 93)
            .map(|r| u32::from_le_bytes(r.2[..4].try_into().unwrap()))
            .collect();
        assert_eq!(
            numbers,
            if name == "aift.hwp" {
                vec![1, 2]
            } else {
                vec![1]
            }
        );
        assert_eq!(
            tail(&edit(&original, section, "A"), section),
            expected,
            "메모 번호·목록 본문이 각각 한 번만 저장된다"
        );
    }
}

fn memo_begin_markers(bytes: &[u8], section: usize) -> Vec<Vec<u8>> {
    section_records(bytes, section)
        .into_iter()
        .filter(|r| r.0 == 67)
        .flat_map(|(_, _, payload)| {
            (0..payload.len().saturating_sub(15))
                .step_by(2)
                .filter(|&i| payload[i..i + 6] == [3, 0, 0x65, 0x6d, 0x25, 0x25])
                .map(|i| payload[i..i + 16].to_vec())
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn unknown_header_memo_keeps_distinct_begin_slot_identity() {
    for name in ["aift.hwp", "issue5866/memo_field_hwp5.hwp"] {
        let original = sample(name);
        let count = DocumentCore::from_bytes(&original)
            .unwrap()
            .document()
            .sections
            .len();
        let section = count - 1;
        let saved = edit(&original, section, "A");
        let expected: Vec<_> = (0..count)
            .flat_map(|s| memo_begin_markers(&original, s))
            .collect();
        assert!(!expected.is_empty(), "원본 본문에는 %%me 시작 슬롯이 있다");
        let actual: Vec<_> = (0..count)
            .flat_map(|s| memo_begin_markers(&saved, s))
            .collect();
        assert_eq!(
            actual, expected,
            "%unk CTRL_HEADER를 본문 %%me 슬롯의 대용으로 쓰면 메모 참조가 끊긴다"
        );
        // command·번호·ID가 담긴 %unk header도 동시에 보존한다.
        let headers = |bytes: &[u8]| -> Vec<_> {
            (0..count)
                .flat_map(|s| section_records(bytes, s))
                .filter(|r| r.0 == 71 && r.2.windows(8).any(|w| w == b"M\0E\0M\0O\0"))
                .collect()
        };
        assert_eq!(headers(&saved), headers(&original));
    }
}

#[test]
fn repeated_save_does_not_duplicate_tail_and_preserves_both_edits() {
    let original = sample("exam_math_8.hwp");
    let first = edit(&original, 0, "A");
    let second = edit(&first, 0, "B");
    assert_eq!(tail(&second, 0), tail(&original, 0));
    assert!(DocumentCore::from_bytes(&second)
        .unwrap()
        .document()
        .sections[0]
        .paragraphs[0]
        .text
        .starts_with("BA"));
}

#[test]
fn editing_other_section_keeps_memo_owner_and_ordinary_master_controls() {
    let original = sample("aift.hwp");
    let count = DocumentCore::from_bytes(&original)
        .unwrap()
        .document()
        .sections
        .len();
    let saved = edit(&original, 0, "A");
    for section in 1..count {
        assert_eq!(
            section_records(&saved, section),
            section_records(&original, section),
            "다른 구역의 저장 소유를 옮기거나 재작성하면 안 된다"
        );
    }
    for name in ["exam_kor.hwp", "exam_eng.hwp"] {
        let original = sample(name);
        let before = DocumentCore::from_bytes(&original).unwrap();
        let saved = edit(&original, 0, "A");
        let after = DocumentCore::from_bytes(&saved).unwrap();
        assert_eq!(
            serde_json::to_value(&before.document().sections[0].section_def.master_pages).unwrap(),
            serde_json::to_value(&after.document().sections[0].section_def.master_pages).unwrap(),
            "정상 바탕쪽의 적용 의미와 본문 보존: {name}"
        );
        assert_eq!(tail(&saved, 0), tail(&original, 0));
    }
}

#[test]
fn changed_master_model_cannot_replay_stale_tail() {
    let core = DocumentCore::from_bytes(&sample("exam_math_8.hwp")).unwrap();
    let mut model = core.document().clone();
    let masters = &mut model.sections[0].section_def.master_pages;
    assert!(!masters.is_empty());
    masters[0].overlap = !masters[0].overlap;
    assert!(
        matches!(
            serialize_hwp(&model),
            Err(SerializeError::UnsupportedInput(_))
        ),
        "지원하지 않는 목록 자체 수정은 낡은 원본을 강제로 저장하지 않는다"
    );
}

#[test]
fn removed_memo_anchor_cannot_replay_orphan_tail() {
    let core = DocumentCore::from_bytes(&sample("issue5866/memo_field_hwp5.hwp")).unwrap();
    let mut model = core.document().clone();
    let mut removed = false;
    for para in &mut model.sections[0].paragraphs {
        para.controls.retain(|c| {
            let memo = matches!(c, Control::Field(f) if f.command.starts_with("MEMO/"));
            removed |= memo;
            !memo
        });
    }
    assert!(removed, "원본의 실제 본문 MEMO anchor를 제거한다");
    assert!(
        matches!(
            serialize_hwp(&model),
            Err(SerializeError::UnsupportedInput(_))
        ),
        "소유 anchor가 사라진 목록을 원본 그대로 저장하면 안 된다"
    );
}

#[test]
fn synthesized_section_definition_does_not_take_tail_owned_masters() {
    let core = DocumentCore::from_bytes(&sample("exam_math_8.hwp")).unwrap();
    let mut model = core.document().clone();
    let expected = serde_json::to_value(&model.sections[0].section_def.master_pages).unwrap();
    model.sections[0].paragraphs[0]
        .controls
        .retain(|c| !matches!(c, Control::SectionDef(_)));
    let saved = serialize_hwp(&model).expect("구역 정의 보강 경로");
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened.document().sections[0].section_def.master_pages).unwrap(),
        expected,
        "독립 tail 저장 소유를 secd 안에 다시 생성하면 중복된다"
    );
}

#[test]
fn hwpx_without_native_tail_keeps_existing_memo_materialization() {
    let original = sample("hwpx/aift.hwpx");
    let core = DocumentCore::from_bytes(&original).unwrap();
    let saved = core
        .export_hwp_with_adapter_snapshot()
        .expect("기존 HWPX→HWP 생성 경로");
    let reader = CfbReader::open(&saved).unwrap();
    let count = reader.section_count() as usize;
    let numbers: Vec<_> = (0..count)
        .flat_map(|s| section_records(&saved, s))
        .filter(|r| r.0 == 93)
        .map(|r| u32::from_le_bytes(r.2[..4].try_into().unwrap()))
        .collect();
    assert_eq!(
        numbers,
        vec![1, 2],
        "원본 없는 HWPX 메모 목록 생성은 유지한다"
    );
    let markers: Vec<_> = (0..count)
        .flat_map(|s| memo_begin_markers(&saved, s))
        .collect();
    assert_eq!(
        markers.len(),
        2,
        "목록과 연결되는 시작 슬롯도 각각 존재한다"
    );
    assert!(DocumentCore::from_bytes(&saved).is_ok());
}
