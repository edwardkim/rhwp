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
