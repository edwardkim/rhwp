//! 관계형·고정형: #8863 원본 중복 글자 모양의 편집 전후 유효 값 보존.
//! 확인: 2026-11-10 00:14:37 UTC. 근거: mydocs/report/task_m100_8863_report.md.
//! 입력: 공개 정관 HWP5(`samples/issue8863/README.md`), 최소 모델 입력.
//! 원본부터 같은 위치인 경계와 삭제 때문에 모인 경계를 구분한다.
//! API·모델·저장 왕복 계약이며 화면 시각 일치율은 측정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::{CharShapeRef, Paragraph};

type Signature = Vec<Vec<Vec<(char, bool, String)>>>;

fn load() -> DocumentCore {
    DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue8863/articles-of-incorporation.hwp"
    ))
    .unwrap()
}

fn signature(core: &DocumentCore) -> Signature {
    core.document()
        .sections
        .iter()
        .map(|section| {
            section
                .paragraphs
                .iter()
                .map(|para| {
                    para.text
                        .chars()
                        .enumerate()
                        .map(|(i, ch)| {
                            let id = para.char_shape_id_at(i).unwrap() as usize;
                            let shape = &core.document().doc_info.char_shapes[id];
                            (ch, shape.bold, format!("{:?}", shape.text_color))
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

fn assert_signature(actual: &Signature, expected: &Signature, stage: &str) {
    assert_eq!(actual.len(), expected.len(), "{stage}: 구역 수");
    for (s, (actual_section, expected_section)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual_section.len(),
            expected_section.len(),
            "{stage}: s{s} 문단 수"
        );
        for (p, (actual_para, expected_para)) in
            actual_section.iter().zip(expected_section).enumerate()
        {
            assert_eq!(actual_para, expected_para, "{stage}: s{s} p{p}");
        }
    }
}

fn assert_roundtrips(core: &DocumentCore, expected: &Signature) {
    assert_signature(&signature(core), expected, "편집 직후");
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_signature(&signature(&reopened), expected, "저장 후 다시 열기");
    }
}

#[test]
fn real_hwp_delete_preserves_every_surviving_character() {
    let mut core = load();
    let runs: Vec<_> = core.document().sections[0].paragraphs[4]
        .char_shapes
        .iter()
        .map(|s| (s.start_pos, s.char_shape_id))
        .collect();
    assert_eq!(runs, [(0, 7), (0, 6), (0, 12), (0, 6)]);
    let mut expected = signature(&core);
    expected[0][4].drain(11..13);
    core.delete_text_native(0, 4, 11, 2).unwrap();
    assert_roundtrips(&core, &expected);
}

#[test]
fn real_hwp_equal_length_replace_preserves_other_runs() {
    let mut core = load();
    let mut expected = signature(&core);
    expected[0][4][11].0 = '법';
    expected[0][4][12].0 = '인';
    core.replace_text_native(0, 4, 11, 2, "법인").unwrap();
    assert_roundtrips(&core, &expected);
}

#[test]
fn real_hwp_replace_all_preserves_both_plain_and_bold_paragraphs() {
    let mut core = load();
    let mut expected = signature(&core);
    let mut count = 0;
    let mut matched_bold = false;
    let mut matched_plain = false;
    for section in &mut expected {
        for para in section {
            for i in 0..para.len().saturating_sub(1) {
                if para[i].0 == '회' && para[i + 1].0 == '사' {
                    matched_bold |= para[i].1;
                    matched_plain |= !para[i].1;
                    para[i].0 = '법';
                    para[i + 1].0 = '인';
                    count += 1;
                }
            }
        }
    }
    assert!(count > 1 && matched_bold && matched_plain);
    let result: serde_json::Value =
        serde_json::from_str(&core.replace_all_native("회사", "법인", true).unwrap()).unwrap();
    assert_eq!(result["count"], count);
    assert_roundtrips(&core, &expected);
}

fn paragraph(runs: &[(u32, u32)]) -> Paragraph {
    // insert_text_at이 emoji의 UTF-16 offset까지 실제 제품 규약대로 만든다.
    let mut para = Paragraph::default();
    para.insert_text_at(0, "A😀BCDE");
    para.char_shapes = runs
        .iter()
        .map(|&(start_pos, char_shape_id)| CharShapeRef {
            start_pos,
            char_shape_id,
        })
        .collect();
    para
}

fn shape_ids(para: &Paragraph) -> Vec<u32> {
    (0..para.text.chars().count())
        .map(|i| para.char_shape_id_at(i).unwrap())
        .collect()
}

#[test]
fn every_delete_range_preserves_original_last_wins_shapes() {
    // 중복 위치가 삭제 앞·시작·범위 안·끝·뒤에 오는 경우, BMP/astral 혼합.
    for runs in [
        vec![(0, 7), (0, 6), (0, 12), (0, 6)],
        vec![(0, 7), (0, 6), (3, 20), (3, 21), (6, 30), (6, 31)],
        vec![(0, 6), (1, 8), (1, 9), (4, 20), (4, 21)],
        vec![(3, 7), (3, 6), (6, 20)],
    ] {
        let original = paragraph(&runs);
        let ids = shape_ids(&original);
        let chars: Vec<_> = original.text.chars().collect();
        for start in 0..chars.len() {
            for count in 1..=chars.len() - start {
                let mut para = original.clone();
                let mut expected_ids = ids.clone();
                expected_ids.drain(start..start + count);
                let mut expected_chars = chars.clone();
                expected_chars.drain(start..start + count);
                assert_eq!(para.delete_text_at(start, count), count);
                assert_eq!(para.text, expected_chars.iter().collect::<String>());
                assert_eq!(shape_ids(&para), expected_ids, "{runs:?}, {start}/{count}");
                assert!(para
                    .char_shapes
                    .windows(2)
                    .all(|w| w[0].start_pos < w[1].start_pos));
            }
        }
    }
}

#[test]
fn full_delete_and_refill_keep_first_effective_shape_not_first_record() {
    let mut para = paragraph(&[(0, 7), (0, 6), (3, 12), (3, 13)]);
    let first_effective = para.char_shape_id_at(0).unwrap();
    assert_eq!(first_effective, 6);
    let len = para.text.chars().count();
    para.delete_text_at(0, len);
    assert_eq!(para.char_shapes.len(), 1);
    assert_eq!(para.char_shapes[0].char_shape_id, first_effective);
    para.insert_text_at(0, "새 날짜");
    assert!(shape_ids(&para).iter().all(|&id| id == first_effective));
}

#[test]
fn suffix_delete_keeps_first_effective_collapsed_shape() {
    let mut para = paragraph(&[(0, 6), (3, 12), (3, 13), (6, 20)]);
    para.delete_text_at(2, 4);
    assert_eq!(para.text, "A😀");
    assert_eq!(para.char_shapes.last().unwrap().char_shape_id, 13);
    assert_eq!(para.char_shapes.last().unwrap().start_pos, 3);
}

#[test]
fn no_op_delete_does_not_normalize_source_records() {
    let original = paragraph(&[(0, 7), (0, 6), (3, 12), (3, 13)]);
    for (start, count) in [(2, 0), (99, 2)] {
        let mut para = original.clone();
        assert_eq!(para.delete_text_at(start, count), 0);
        let runs: Vec<_> = para
            .char_shapes
            .iter()
            .map(|s| (s.start_pos, s.char_shape_id))
            .collect();
        assert_eq!(runs, [(0, 7), (0, 6), (3, 12), (3, 13)]);
        assert_eq!(para.text, original.text);
    }
}

#[test]
fn delayed_first_duplicate_keeps_implicit_prefix_during_full_delete() {
    let mut para = paragraph(&[(3, 7), (3, 6), (6, 20)]);
    assert_eq!(shape_ids(&para), [7, 7, 6, 6, 6, 20]);
    para.delete_text_at(0, 6);
    para.insert_text_at(0, "새 날짜");
    assert_eq!(shape_ids(&para), [7, 7, 7, 7]);
}
