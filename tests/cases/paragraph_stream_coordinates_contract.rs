//! HWP5 `PARA_TEXT` 좌표 계약.
//!
//! 공개 스펙의 확장 문자는 8 UTF-16 code unit 슬롯을 차지한다. 탭은 Rust 문자열에서
//! 한 문자지만 HWP5 스트림에서는 확장 문자이므로, 문자 오프셋과 `char_count` 계산에
//! 슬롯 전체 폭이 반영되어야 한다.

use rhwp::model::paragraph::Paragraph;

#[test]
fn insert_split_merge_preserves_hwp_stream_widths() {
    let mut paragraph = Paragraph::new_empty();
    paragraph.insert_text_at(0, "A🙂\tB");

    assert_eq!(paragraph.char_offsets, vec![0, 1, 3, 11]);
    assert_eq!(paragraph.char_count, 13);

    let tail = paragraph.split_at(3);
    assert_eq!(paragraph.text, "A🙂\t");
    assert_eq!(paragraph.char_offsets, vec![0, 1, 3]);
    assert_eq!(paragraph.char_count, 12);
    assert_eq!(tail.text, "B");
    assert_eq!(tail.char_offsets, vec![0]);
    assert_eq!(tail.char_count, 2);

    paragraph.merge_from(&tail);
    assert_eq!(paragraph.text, "A🙂\tB");
    assert_eq!(paragraph.char_offsets, vec![0, 1, 3, 11]);
    assert_eq!(paragraph.char_count, 13);

    paragraph.delete_text_at(1, 2);
    assert_eq!(paragraph.text, "AB");
    assert_eq!(paragraph.char_offsets, vec![0, 1]);
    assert_eq!(paragraph.char_count, 3);
}

#[test]
fn split_clears_new_paragraph_instance_id_and_preserves_suffix() {
    let mut paragraph = Paragraph {
        text: "AB".to_string(),
        char_count: 3,
        char_offsets: vec![0, 1],
        // counts(6) + instanceId(4) + change-tracking suffix(2)
        raw_header_extra: vec![1, 2, 3, 4, 5, 6, 0xD2, 0x94, 0x09, 0xBA, 7, 8],
        ..Paragraph::new_empty()
    };

    let tail = paragraph.split_at(1);

    assert_eq!(
        &paragraph.raw_header_extra[6..10],
        &[0xD2, 0x94, 0x09, 0xBA]
    );
    assert_eq!(&tail.raw_header_extra[6..10], &[0, 0, 0, 0]);
    assert_eq!(&tail.raw_header_extra[10..12], &[7, 8]);
}
