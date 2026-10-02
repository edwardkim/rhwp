//! 비-글자취급 개체(자리차지·어울림 표·그림·도형)의 **기준 문자도 그 줄의 글자**다.
//!
//! 개체가 글자처럼 흐르지 않아도 기준 문자는 문단 텍스트 안의 한 글자이고 자기 글자모양을
//! 가진다. 한/글은 그 글자모양을 줄 높이에 넣는다. 한/글이 저장한 줄(`LINE_SEG`)에서 문단
//! 첫 줄의 개체 기준 문자 글자모양이 그 줄의 다른 글자보다 클 때 저장 줄 높이는 **언제나
//! 기준 문자 쪽**이다(`samples/` 와 10k 코퍼스 HWP 1,500건 표본 스캔: 11/11, 반례 0).
//!
//! rhwp 의 줄 재계산(`reflow_line_segs`)은 텍스트 토큰의 글꼴만 보고 기준 문자를 빼서, 편집
//! 뒤나 `linesegarray` 가 없는 HWPX 문단에서 줄이 작은 글자 크기로 줄었다. 공문 HWPX
//! 표본에서 표 아래로 밀린 host 줄이 8pt(800/480)로 합성돼 뒤 본문 전체가 한/글 정본보다
//! 8.5px 위로 올라붙었다(정본과 생성기 저장 사다리는 모두 12pt 줄 1200/720).
//!
//! 기대값은 각 표본의 **한/글 저장 첫 줄 높이**다(구현이 낸 값이 아니다). 문단 끝에 글자를
//! 하나 넣어 줄을 다시 계산시킨 뒤 첫 줄이 그 높이를 유지하는지 본다. 기준 문자가 더 작은
//! 문단은 텍스트 크기를 유지해야 한다(대조군).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

struct Case {
    sample: &'static str,
    section: usize,
    para: usize,
    /// 한/글이 저장한 첫 줄 `text_height`(HU) — 독립 기준.
    stored_first_line_height: i32,
    /// 첫 줄 텍스트 글자의 최대 글자모양 크기(HU) — 기준 문자를 빼면 나오는 값.
    text_only_height: i32,
}

const LARGER_ANCHOR: &[Case] = &[
    // 자리차지 표, 기준 문자 16pt · 텍스트 12pt
    Case {
        sample: "samples/task2287/1342000_edu_curriculum_map.hwp",
        section: 4,
        para: 12,
        stored_first_line_height: 1600,
        text_only_height: 1200,
    },
    // 자리차지 표(단 기준 가로), 기준 문자 15pt · 텍스트 14pt
    Case {
        sample: "samples/issue7198/156403546_negative_spacing_host_after_table.hwp",
        section: 0,
        para: 22,
        stored_first_line_height: 1500,
        text_only_height: 1400,
    },
    // 자리차지 그림, 기준 문자 12pt · 텍스트 10pt
    Case {
        sample: "samples/task1725/text_footnote_tail_overpagination.hwp",
        section: 0,
        para: 4701,
        stored_first_line_height: 1200,
        text_only_height: 1000,
    },
    // 도형 4개 + 그림, 기준 문자 20pt · 텍스트 14pt
    Case {
        sample: "samples/issue5699/16758113_pruning_forms.hwp",
        section: 1,
        para: 2,
        stored_first_line_height: 2000,
        text_only_height: 1400,
    },
];

/// 대조군: 기준 문자(어울림 표, 10.5pt)가 텍스트(11.5pt)보다 작다 — 줄은 텍스트 크기다.
const SMALLER_ANCHOR: Case = Case {
    sample: "samples/exam_kor.hwp",
    section: 1,
    para: 33,
    stored_first_line_height: 1150,
    text_only_height: 1150,
};

fn first_line_height_after_edit(case: &Case) -> (i32, i32) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(case.sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", case.sample));
    let mut core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let stored = core.document().sections[case.section].paragraphs[case.para]
        .line_segs
        .first()
        .map(|seg| seg.text_height)
        .expect("저장 첫 줄");
    let end = core.document().sections[case.section].paragraphs[case.para]
        .text
        .chars()
        .count();
    core.insert_text_native(case.section, case.para, end, "가")
        .expect("문단 끝 글자 삽입");
    let reflowed = core.document().sections[case.section].paragraphs[case.para]
        .line_segs
        .first()
        .map(|seg| seg.text_height)
        .expect("재계산 첫 줄");
    (stored, reflowed)
}

#[test]
fn reflowed_first_line_keeps_larger_floating_anchor_char_height() {
    for case in LARGER_ANCHOR {
        assert!(case.stored_first_line_height > case.text_only_height);
        let (stored, reflowed) = first_line_height_after_edit(case);
        assert_eq!(
            stored, case.stored_first_line_height,
            "{} {}.{}: 표본의 한/글 저장 첫 줄 높이가 기록과 다르다",
            case.sample, case.section, case.para
        );
        assert_eq!(
            reflowed,
            case.stored_first_line_height,
            "{} {}.{}: 다시 계산한 첫 줄이 개체 기준 문자 크기({})가 아니라 {} 이다 \
             (텍스트만 본 값 {})",
            case.sample,
            case.section,
            case.para,
            case.stored_first_line_height,
            reflowed,
            case.text_only_height
        );
    }
}

#[test]
fn reflowed_first_line_ignores_smaller_floating_anchor_char() {
    let (stored, reflowed) = first_line_height_after_edit(&SMALLER_ANCHOR);
    assert_eq!(stored, SMALLER_ANCHOR.stored_first_line_height);
    assert_eq!(
        reflowed, SMALLER_ANCHOR.text_only_height,
        "기준 문자가 텍스트보다 작으면 줄 높이는 텍스트 크기여야 한다"
    );
}
