use super::*;
use crate::model::control::Bookmark;
use crate::model::table::Table;

#[test]
fn test_paragraph_default() {
    let para = Paragraph::default();
    assert_eq!(para.char_count, 0);
    assert!(para.text.is_empty());
    assert!(para.controls.is_empty());
}

#[test]
fn test_line_seg_flags() {
    let seg = LineSeg { tag: 0x03, ..Default::default() };
    assert!(seg.is_first_line_of_page());
    assert!(seg.is_first_line_of_column());
}

#[test]
fn test_column_break_type() {
    assert_eq!(ColumnBreakType::default(), ColumnBreakType::None);
}

#[test]
fn test_insert_text_at_middle() {
    let mut para = Paragraph {
        text: "안녕세계".to_string(),
        char_count: 4,
        char_offsets: vec![0, 1, 2, 3],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
        ],
        ..Default::default()
    };
    para.insert_text_at(2, "하");
    assert_eq!(para.text, "안녕하세계");
    assert_eq!(para.char_count, 5);
    assert_eq!(para.char_offsets, vec![0, 1, 2, 3, 4]);
}

#[test]
fn test_insert_text_at_beginning() {
    let mut para = Paragraph {
        text: "세계".to_string(),
        char_count: 2,
        char_offsets: vec![0, 1],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
        ],
        ..Default::default()
    };
    para.insert_text_at(0, "안녕");
    assert_eq!(para.text, "안녕세계");
    assert_eq!(para.char_count, 4);
    assert_eq!(para.char_offsets, vec![0, 1, 2, 3]);
}

#[test]
fn test_insert_text_at_end() {
    let mut para = Paragraph {
        text: "안녕".to_string(),
        char_count: 2,
        char_offsets: vec![0, 1],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
        ],
        ..Default::default()
    };
    para.insert_text_at(2, "세계");
    assert_eq!(para.text, "안녕세계");
    assert_eq!(para.char_count, 4);
    assert_eq!(para.char_offsets, vec![0, 1, 2, 3]);
}

#[test]
fn test_insert_text_char_shapes_shift() {
    let mut para = Paragraph {
        text: "AB".to_string(),
        char_count: 2,
        char_offsets: vec![0, 1],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
            CharShapeRef { start_pos: 1, char_shape_id: 2 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
        ],
        ..Default::default()
    };
    // 'A'와 'B' 사이에 'X' 삽입
    para.insert_text_at(1, "X");
    assert_eq!(para.text, "AXB");
    // start_pos=0은 변경 없음, start_pos=1은 2로 시프트
    assert_eq!(para.char_shapes[0].start_pos, 0);
    assert_eq!(para.char_shapes[1].start_pos, 2);
}

#[test]
fn test_insert_text_line_segs_shift() {
    let mut para = Paragraph {
        text: "Hello\nWorld".to_string(),
        char_count: 11,
        char_offsets: (0..11).collect(),
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
            LineSeg { text_start: 6, ..Default::default() },
        ],
        ..Default::default()
    };
    // 위치 3에 "XX" 삽입 → "HelXXlo\nWorld"
    para.insert_text_at(3, "XX");
    assert_eq!(para.text, "HelXXlo\nWorld");
    // 두 번째 줄의 text_start=6이 8로 시프트
    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(para.line_segs[1].text_start, 8);
}

#[test]
fn test_insert_text_empty() {
    let mut para = Paragraph {
        text: "AB".to_string(),
        char_count: 2,
        char_offsets: vec![0, 1],
        ..Default::default()
    };
    para.insert_text_at(1, "");
    assert_eq!(para.text, "AB");
    assert_eq!(para.char_count, 2);
}

#[test]
fn test_insert_line_break() {
    let mut para = Paragraph {
        text: "가나다".to_string(),
        char_count: 3,
        char_offsets: vec![0, 1, 2],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
        ],
        ..Default::default()
    };
    // Shift+Enter: 줄바꿈 문자 '\n' 삽입
    para.insert_text_at(1, "\n");
    assert_eq!(para.text, "가\n나다");
    assert_eq!(para.char_count, 4);
    // 줄바꿈은 문단 분리가 아니므로 같은 문단에 남아야 함
    assert!(para.text.contains('\n'));
}

#[test]
fn test_delete_text_at_middle() {
    let mut para = Paragraph {
        text: "안녕하세계".to_string(),
        char_count: 5,
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
        ],
        ..Default::default()
    };
    let deleted = para.delete_text_at(2, 1); // '하' 삭제
    assert_eq!(deleted, 1);
    assert_eq!(para.text, "안녕세계");
    assert_eq!(para.char_count, 4);
    assert_eq!(para.char_offsets, vec![0, 1, 2, 3]);
}

#[test]
fn test_delete_text_at_beginning() {
    let mut para = Paragraph {
        text: "안녕세계".to_string(),
        char_count: 4,
        char_offsets: vec![0, 1, 2, 3],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
        ],
        ..Default::default()
    };
    let deleted = para.delete_text_at(0, 2); // '안녕' 삭제
    assert_eq!(deleted, 2);
    assert_eq!(para.text, "세계");
    assert_eq!(para.char_count, 2);
    assert_eq!(para.char_offsets, vec![0, 1]);
}

#[test]
fn test_delete_text_at_end() {
    let mut para = Paragraph {
        text: "안녕세계".to_string(),
        char_count: 4,
        char_offsets: vec![0, 1, 2, 3],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
        ],
        ..Default::default()
    };
    let deleted = para.delete_text_at(3, 1); // '계' 삭제
    assert_eq!(deleted, 1);
    assert_eq!(para.text, "안녕세");
    assert_eq!(para.char_count, 3);
    assert_eq!(para.char_offsets, vec![0, 1, 2]);
}

#[test]
fn test_delete_text_char_shapes_shift() {
    let mut para = Paragraph {
        text: "AXB".to_string(),
        char_count: 3,
        char_offsets: vec![0, 1, 2],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
            CharShapeRef { start_pos: 2, char_shape_id: 2 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
        ],
        ..Default::default()
    };
    // 'X' 삭제 (위치 1)
    para.delete_text_at(1, 1);
    assert_eq!(para.text, "AB");
    // start_pos=0은 변경 없음, start_pos=2는 1로 시프트
    assert_eq!(para.char_shapes[0].start_pos, 0);
    assert_eq!(para.char_shapes[1].start_pos, 1);
}

#[test]
fn test_delete_text_line_segs_shift() {
    let mut para = Paragraph {
        text: "HelXXlo\nWorld".to_string(),
        char_count: 13,
        char_offsets: (0..13).collect(),
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![
            LineSeg { text_start: 0, ..Default::default() },
            LineSeg { text_start: 8, ..Default::default() },
        ],
        ..Default::default()
    };
    // 위치 3에서 2글자("XX") 삭제 → "Hello\nWorld"
    para.delete_text_at(3, 2);
    assert_eq!(para.text, "Hello\nWorld");
    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(para.line_segs[1].text_start, 6);
}

#[test]
fn test_delete_text_empty_range() {
    let mut para = Paragraph {
        text: "AB".to_string(),
        char_count: 2,
        char_offsets: vec![0, 1],
        ..Default::default()
    };
    let deleted = para.delete_text_at(1, 0);
    assert_eq!(deleted, 0);
    assert_eq!(para.text, "AB");
    assert_eq!(para.char_count, 2);
}

// === split_at 테스트 ===

#[test]
fn test_split_at_middle() {
    let mut para = Paragraph {
        text: "안녕하세요".to_string(),
        char_count: 6, // 5 + 1
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    let new_para = para.split_at(2);

    // 원래 문단: "안녕"
    assert_eq!(para.text, "안녕");
    assert_eq!(para.char_offsets, vec![0, 1]);
    assert_eq!(para.char_shapes[0].start_pos, 0);
    assert_eq!(para.char_shapes[0].char_shape_id, 1);

    // 새 문단: "하세요"
    assert_eq!(new_para.text, "하세요");
    assert_eq!(new_para.char_offsets, vec![0, 1, 2]);
    assert_eq!(new_para.char_shapes[0].start_pos, 0);
    assert_eq!(new_para.char_shapes[0].char_shape_id, 1);
}

#[test]
fn test_split_at_beginning() {
    let mut para = Paragraph {
        text: "ABC".to_string(),
        char_count: 4,
        char_offsets: vec![0, 1, 2],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    let new_para = para.split_at(0);

    assert_eq!(para.text, "");
    assert_eq!(para.char_offsets.len(), 0);

    assert_eq!(new_para.text, "ABC");
    assert_eq!(new_para.char_offsets, vec![0, 1, 2]);
}

#[test]
fn test_split_at_end() {
    let mut para = Paragraph {
        text: "ABC".to_string(),
        char_count: 4,
        char_offsets: vec![0, 1, 2],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    let new_para = para.split_at(3);

    assert_eq!(para.text, "ABC");
    assert_eq!(new_para.text, "");
}

#[test]
fn test_split_at_preserves_char_shapes() {
    let mut para = Paragraph {
        text: "AABBB".to_string(),
        char_count: 6,
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
            CharShapeRef { start_pos: 2, char_shape_id: 2 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    // 위치 2에서 분리 → "AA" / "BBB"
    let new_para = para.split_at(2);

    // 원래: style 1만 유지
    assert_eq!(para.char_shapes.len(), 1);
    assert_eq!(para.char_shapes[0].char_shape_id, 1);

    // 새 문단: style 2 (pos 0)
    assert_eq!(new_para.char_shapes.len(), 1);
    assert_eq!(new_para.char_shapes[0].start_pos, 0);
    assert_eq!(new_para.char_shapes[0].char_shape_id, 2);
}

#[test]
fn test_split_at_preserves_para_shape() {
    let mut para = Paragraph {
        text: "ABCD".to_string(),
        char_count: 5,
        char_offsets: vec![0, 1, 2, 3],
        char_shapes: vec![CharShapeRef { start_pos: 0, char_shape_id: 1 }],
        para_shape_id: 42,
        style_id: 3,
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    let new_para = para.split_at(2);
    assert_eq!(new_para.para_shape_id, 42);
    assert_eq!(new_para.style_id, 3);
}

// === merge_from 테스트 ===

#[test]
fn test_merge_from_basic() {
    let mut para1 = Paragraph {
        text: "안녕".to_string(),
        char_count: 3,
        char_offsets: vec![0, 1],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    let para2 = Paragraph {
        text: "하세요".to_string(),
        char_count: 4,
        char_offsets: vec![0, 1, 2],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    let merge_pos = para1.merge_from(&para2);

    assert_eq!(merge_pos, 2); // 원래 "안녕"의 길이
    assert_eq!(para1.text, "안녕하세요");
    assert_eq!(para1.char_offsets, vec![0, 1, 2, 3, 4]);
}

#[test]
fn test_merge_from_different_styles() {
    let mut para1 = Paragraph {
        text: "AA".to_string(),
        char_count: 3,
        char_offsets: vec![0, 1],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    let para2 = Paragraph {
        text: "BB".to_string(),
        char_count: 3,
        char_offsets: vec![0, 1],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 2 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    para1.merge_from(&para2);

    assert_eq!(para1.text, "AABB");
    // char_shapes: style 1 at 0, style 2 at 2
    assert_eq!(para1.char_shapes.len(), 2);
    assert_eq!(para1.char_shapes[0].char_shape_id, 1);
    assert_eq!(para1.char_shapes[1].start_pos, 2);
    assert_eq!(para1.char_shapes[1].char_shape_id, 2);
}

#[test]
fn test_merge_from_empty() {
    let mut para1 = Paragraph {
        text: "안녕".to_string(),
        char_count: 3,
        char_offsets: vec![0, 1],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    let para2 = Paragraph::default();

    let merge_pos = para1.merge_from(&para2);
    assert_eq!(merge_pos, 2);
    assert_eq!(para1.text, "안녕");
}

#[test]
fn test_split_and_merge_roundtrip() {
    let mut para = Paragraph {
        text: "안녕하세요".to_string(),
        char_count: 6,
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg { text_start: 0, ..Default::default() }],
        ..Default::default()
    };

    // 분리 후 재병합 → 원본 텍스트 복원
    let new_para = para.split_at(2);
    assert_eq!(para.text, "안녕");
    assert_eq!(new_para.text, "하세요");

    para.merge_from(&new_para);
    assert_eq!(para.text, "안녕하세요");
    assert_eq!(para.char_offsets, vec![0, 1, 2, 3, 4]);
}

#[test]
fn test_char_shape_id_at() {
    let para = Paragraph {
        text: "ABCDE".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 10 },
            CharShapeRef { start_pos: 2, char_shape_id: 20 },
        ],
        ..Default::default()
    };
    assert_eq!(para.char_shape_id_at(0), Some(10));
    assert_eq!(para.char_shape_id_at(1), Some(10));
    assert_eq!(para.char_shape_id_at(2), Some(20));
    assert_eq!(para.char_shape_id_at(4), Some(20));
}

#[test]
fn test_apply_char_shape_range_full() {
    // 전체 범위 적용
    let mut para = Paragraph {
        text: "ABCDE".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 10 },
        ],
        ..Default::default()
    };
    para.apply_char_shape_range(0, 5, 99);
    assert_eq!(para.char_shapes.len(), 1);
    assert_eq!(para.char_shapes[0].char_shape_id, 99);
    assert_eq!(para.char_shapes[0].start_pos, 0);
}

#[test]
fn test_apply_char_shape_range_left_partial() {
    // 왼쪽 부분만 변경: [0,2) → 99, [2,5) → 10 유지
    let mut para = Paragraph {
        text: "ABCDE".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 10 },
        ],
        ..Default::default()
    };
    para.apply_char_shape_range(0, 2, 99);
    assert_eq!(para.char_shapes.len(), 2);
    assert_eq!(para.char_shapes[0].char_shape_id, 99);
    assert_eq!(para.char_shapes[0].start_pos, 0);
    assert_eq!(para.char_shapes[1].char_shape_id, 10);
    assert_eq!(para.char_shapes[1].start_pos, 2);
}

#[test]
fn test_apply_char_shape_range_right_partial() {
    // 오른쪽 부분만 변경: [0,2) → 10 유지, [2,5) → 99
    let mut para = Paragraph {
        text: "ABCDE".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 10 },
        ],
        ..Default::default()
    };
    para.apply_char_shape_range(2, 5, 99);
    assert_eq!(para.char_shapes.len(), 2);
    assert_eq!(para.char_shapes[0].char_shape_id, 10);
    assert_eq!(para.char_shapes[0].start_pos, 0);
    assert_eq!(para.char_shapes[1].char_shape_id, 99);
    assert_eq!(para.char_shapes[1].start_pos, 2);
}

#[test]
fn test_apply_char_shape_range_middle() {
    // 중간 부분 변경: [1,3) → 99
    let mut para = Paragraph {
        text: "ABCDE".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 10 },
        ],
        ..Default::default()
    };
    para.apply_char_shape_range(1, 3, 99);
    assert_eq!(para.char_shapes.len(), 3);
    assert_eq!(para.char_shapes[0].char_shape_id, 10);
    assert_eq!(para.char_shapes[0].start_pos, 0);
    assert_eq!(para.char_shapes[1].char_shape_id, 99);
    assert_eq!(para.char_shapes[1].start_pos, 1);
    assert_eq!(para.char_shapes[2].char_shape_id, 10);
    assert_eq!(para.char_shapes[2].start_pos, 3);
}

#[test]
fn test_apply_char_shape_range_multi_segment() {
    // 여러 세그먼트에 걸쳐 적용
    let mut para = Paragraph {
        text: "ABCDE".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 10 },
            CharShapeRef { start_pos: 2, char_shape_id: 20 },
            CharShapeRef { start_pos: 4, char_shape_id: 30 },
        ],
        ..Default::default()
    };
    // [1,4) 범위에 99 적용 → 10 + 20 일부 + 30 앞부분
    para.apply_char_shape_range(1, 4, 99);
    assert_eq!(para.char_shapes.len(), 3);
    assert_eq!(para.char_shapes[0].char_shape_id, 10);
    assert_eq!(para.char_shapes[0].start_pos, 0);
    assert_eq!(para.char_shapes[1].char_shape_id, 99);
    assert_eq!(para.char_shapes[1].start_pos, 1);
    assert_eq!(para.char_shapes[2].char_shape_id, 30);
    assert_eq!(para.char_shapes[2].start_pos, 4);
}

#[test]
fn test_apply_char_shape_range_merge_same_id() {
    // 같은 ID로 적용하면 병합됨
    let mut para = Paragraph {
        text: "ABCDE".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 10 },
            CharShapeRef { start_pos: 2, char_shape_id: 20 },
        ],
        ..Default::default()
    };
    // [2,5) 범위에 10 적용 → 전체가 10으로 병합
    para.apply_char_shape_range(2, 5, 10);
    assert_eq!(para.char_shapes.len(), 1);
    assert_eq!(para.char_shapes[0].char_shape_id, 10);
}

#[test]
fn test_apply_char_shape_after_sequential_insert() {
    // 새 문서에서 문자를 하나씩 입력한 후 중간에 숫자를 삽입하고 윗첨자 적용하는 시나리오
    let mut para = Paragraph {
        text: String::new(),
        char_offsets: vec![],
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 10 },
        ],
        ..Default::default()
    };

    // 한글 문자 7개 순차 입력: "가나다라마바사"
    for (i, ch) in "가나다라마바사".chars().enumerate() {
        para.insert_text_at(i, &ch.to_string());
    }
    assert_eq!(para.text, "가나다라마바사");
    assert_eq!(para.char_offsets.len(), 7);

    // 위치 2에 "123" 삽입 → "가나123다라마바사"
    para.insert_text_at(2, "1");
    para.insert_text_at(3, "2");
    para.insert_text_at(4, "3");
    assert_eq!(para.text, "가나123다라마바사");
    assert_eq!(para.char_offsets.len(), 10);

    // "123" (chars 2-5)에 윗첨자 적용
    para.apply_char_shape_range(2, 5, 99);

    // 결과: [원본(0), 윗첨자(2), 원본(5)] 3개 세그먼트
    eprintln!("char_shapes count: {}", para.char_shapes.len());
    for (i, cs) in para.char_shapes.iter().enumerate() {
        eprintln!("  [{}] pos={} id={}", i, cs.start_pos, cs.char_shape_id);
    }
    assert_eq!(para.char_shapes.len(), 3, "should have 3 segments: original, superscript, original");
    assert_eq!(para.char_shapes[0].char_shape_id, 10); // 가나
    assert_eq!(para.char_shapes[0].start_pos, 0);
    assert_eq!(para.char_shapes[1].char_shape_id, 99); // 123
    assert_eq!(para.char_shapes[1].start_pos, 2);
    assert_eq!(para.char_shapes[2].char_shape_id, 10); // 다라마바사
    assert_eq!(para.char_shapes[2].start_pos, 5);
}

#[test]
fn test_control_text_positions_empty() {
    let para = Paragraph::default();
    assert_eq!(para.control_text_positions(), Vec::<usize>::new());
}

#[test]
fn test_control_text_positions_no_offsets_inline_sequential() {
    let para = Paragraph {
        text: String::new(),
        char_offsets: vec![],
        controls: vec![
            Control::Table(Box::<Table>::default()),
            Control::Table(Box::<Table>::default()),
        ],
        ..Default::default()
    };
    // 인라인 컨트롤 2개: 첫 번째 push 0 후 pos += 1, 두 번째 push 1
    assert_eq!(para.control_text_positions(), vec![0, 1]);
}

#[test]
fn test_control_text_positions_gap_between_chars() {
    // text = "AB", char_offsets = [0, 9] → 'A'(width 1) 와 'B' 사이 8 unit 갭 = inline ctrl 1개
    let para = Paragraph {
        text: "AB".to_string(),
        char_offsets: vec![0, 9],
        controls: vec![Control::Table(Box::<Table>::default())],
        ..Default::default()
    };
    // controls[0] = 'A' 다음 character index 1
    assert_eq!(para.control_text_positions(), vec![1]);
}

#[test]
fn test_control_text_positions_gap_before() {
    // text = "A", char_offsets = [8] → 'A' 앞에 8 unit 갭 = inline ctrl 1개
    let para = Paragraph {
        text: "A".to_string(),
        char_offsets: vec![8],
        controls: vec![Control::Table(Box::<Table>::default())],
        ..Default::default()
    };
    // controls[0] = 'A' 이전 character index 0
    assert_eq!(para.control_text_positions(), vec![0]);
}

#[test]
fn test_control_text_positions_surrogate_pair_char_width() {
    // text = "🎉A": '🎉'(U+1F389, surrogate pair, UTF-16 width=2) + 'A'(width=1)
    // char_offsets = [0, 17] → 사이 gap = 17 - 0 - 2(surrogate width) = 15 = inline ctrl 1개
    // controls 2 개로 width 분기 boundary 검증:
    //   - width=2 정상: gap=15, n_ctrls=1, push position 1, fill last position chars.len()=2 → [1, 2]
    //   - width=1 버그: gap=16, n_ctrls=2, push position 1 두 번 → [1, 1] (다른 결과)
    let para = Paragraph {
        text: "🎉A".to_string(),
        char_offsets: vec![0, 17],
        controls: vec![
            Control::Table(Box::<Table>::default()),
            Control::Table(Box::<Table>::default()),
        ],
        ..Default::default()
    };
    assert_eq!(para.control_text_positions(), vec![1, 2]);
}

#[test]
fn test_control_text_positions_no_offsets_non_inline_skipped() {
    // `char_offsets` 비어있는 폴백 경로에서 비인라인 컨트롤 (Bookmark) 은 pos 증가시키지 않음.
    // [Bookmark, Table, Bookmark] →
    //   - Bookmark: push pos=0, 비인라인이라 pos 유지 (0)
    //   - Table:    push pos=0, 인라인이라 pos += 1 (= 1)
    //   - Bookmark: push pos=1, 비인라인이라 pos 유지
    // 결과: [0, 0, 1]
    let para = Paragraph {
        text: String::new(),
        char_offsets: vec![],
        controls: vec![
            Control::Bookmark(Bookmark::default()),
            Control::Table(Box::<Table>::default()),
            Control::Bookmark(Bookmark::default()),
        ],
        ..Default::default()
    };
    assert_eq!(para.control_text_positions(), vec![0, 0, 1]);
}
