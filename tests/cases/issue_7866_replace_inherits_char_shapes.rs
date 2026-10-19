#![cfg(not(target_arch = "wasm32"))]

//! [#7866] 찾아 바꾸기가 바뀐 글자의 글자 모양을 지운 글자에서 잇는 계약.
//!
//! 종전 치환은 지운 뒤 넣는 두 단계였다. 낱말이 글자 모양 구간을 통째로 차지하면 지우는
//! 순간 그 구간이 사라지고, 새로 넣은 글자는 앞 글자의 모양에 붙었다(진하게·빨강 소실).
//!
//! 기대값은 한/글 2024 의 실제 찾아 바꾸기(COM 자동화, `samples/issue7866/*_before.hwpx`
//! 를 같은 검색어·바꿀 말로 바꾼 저장본이 `*_after.hwpx`)다. 규칙: i번째 새 글자는 i번째
//! 지운 글자의 모양을, 지운 글자 수를 넘는 꼬리는 마지막 지운 글자의 모양을 받는다.
//! 「굵게」의 첫 글자만 굵으면 「새말」의 「새」만 굵고(`b_first_char_bold`), 「굵(굵게)게(빨강)」
//! → 「일이삼」은 「일」 굵게·「이삼」 빨강이다(`k_two_shapes_then_plain`). 앞 글자가 굵고
//! 검색어가 보통이면 새 말도 보통이다(`h_prev_bold_match_plain`).
//!
//! 수정 전에는 `hancom_pairs_after_replace_all` 이 `a_whole_bold` 부터 실패한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

const CASES: &[(&str, &str, &str, bool)] = &[
    ("a_whole_bold", "굵게", "새말", false),
    ("b_first_char_bold", "굵게", "새말", false),
    ("c_second_char_bold", "굵게", "새말", false),
    ("d_red_longer", "굵게", "길어진새말", false),
    ("e_para_start", "굵게", "새말", false),
    ("f_para_end", "굵게", "새말", false),
    ("g_table_cell", "굵게", "새말", true),
    ("h_prev_bold_match_plain", "굵게", "새말", false),
    ("i_shorter", "굵게표시", "짧", false),
    ("j_longer_partial", "굵게", "길어진새말", false),
    ("k_two_shapes_then_plain", "굵게", "일이삼", false),
];

fn load(name: &str, stage: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/issue7866/{name}_{stage}.hwpx"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("한/글 저장본 파싱")
}

fn target_paragraph(core: &DocumentCore, in_table: bool) -> &Paragraph {
    let paragraphs = &core.document().sections[0].paragraphs;
    if in_table {
        for para in paragraphs {
            for control in &para.controls {
                if let Control::Table(table) = control {
                    return &table.cells[0].paragraphs[0];
                }
            }
        }
        panic!("표가 없다");
    }
    paragraphs
        .iter()
        .find(|p| !p.text.is_empty())
        .expect("글자가 있는 본문 문단")
}

/// 글자마다 (글자, 진하게, 글자 색). 모양 ID 가 아니라 속성으로 비교한다 —
/// 한/글 저장본과 rhwp 편집본은 ID 배정이 다를 수 있다.
fn signature(core: &DocumentCore, in_table: bool) -> Vec<(char, bool, String)> {
    let para = target_paragraph(core, in_table);
    let shapes = &core.document().doc_info.char_shapes;
    para.text
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            let id = para.char_shape_id_at(i).unwrap_or(0) as usize;
            let shape = shapes.get(id).expect("글자 모양 ID 범위");
            (ch, shape.bold, format!("{:?}", shape.text_color))
        })
        .collect()
}

fn reopened(core: &DocumentCore) -> DocumentCore {
    let bytes = core.export_hwpx_native().expect("HWPX 저장");
    DocumentCore::from_bytes(&bytes).expect("저장본 재파싱")
}

#[test]
fn hancom_pairs_after_replace_all() {
    for &(name, src, dst, in_table) in CASES {
        let mut core = load(name, "before");
        let expected = signature(&load(name, "after"), in_table);
        let json = core.replace_all_native(src, dst, true).expect("치환");
        assert_eq!(json, r#"{"ok":true,"count":1}"#, "{name}");
        assert_eq!(signature(&core, in_table), expected, "{name}: 치환 직후");
        assert_eq!(
            signature(&reopened(&core), in_table),
            expected,
            "{name}: 저장 뒤 다시 열기"
        );
    }
}

#[test]
fn single_replace_paths_inherit_the_same_shapes() {
    let expected = signature(&load("k_two_shapes_then_plain", "after"), false);

    let mut core = load("k_two_shapes_then_plain", "before");
    let json = core
        .replace_one_native("굵게", "일이삼", true)
        .expect("치환");
    assert!(json.starts_with(r#"{"ok":true"#), "{json}");
    assert_eq!(signature(&core, false), expected, "replace_one");

    let mut core = load("k_two_shapes_then_plain", "before");
    core.replace_nth_native("굵게", "일이삼", true, 0)
        .expect("치환");
    assert_eq!(signature(&core, false), expected, "replace_nth");

    let mut core = load("k_two_shapes_then_plain", "before");
    let (sec, para_idx) = (0, {
        let paragraphs = &core.document().sections[0].paragraphs;
        paragraphs.iter().position(|p| !p.text.is_empty()).unwrap()
    });
    core.replace_text_native(sec, para_idx, 2, 2, "일이삼")
        .expect("치환");
    assert_eq!(signature(&core, false), expected, "replace_text");
    assert_eq!(
        signature(&reopened(&core), false),
        expected,
        "replace_text 저장 뒤"
    );
}

#[test]
fn issue_report_synthetic_document() {
    // 이슈 본문의 재현 그대로: 빈 문서, 「굵게」만 진하게, replaceAll.
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞 굵게 뒤").unwrap();
    core.apply_char_format_native(0, 0, 2, 4, r#"{"bold":true}"#)
        .unwrap();
    let bold_id = core.document().sections[0].paragraphs[0]
        .char_shape_id_at(2)
        .unwrap();
    assert!(core.document().doc_info.char_shapes[bold_id as usize].bold);

    core.replace_all_native("굵게", "새말", true).unwrap();
    let para = &core.document().sections[0].paragraphs[0];
    assert_eq!(para.text, "앞 새말 뒤");
    assert_eq!(para.char_shape_id_at(2), Some(bold_id), "새");
    assert_eq!(para.char_shape_id_at(3), Some(bold_id), "말");
    assert_eq!(para.char_shape_id_at(1), Some(0), "앞 공백은 보통");
    assert_eq!(para.char_shape_id_at(4), Some(0), "뒤 공백은 보통");
}

#[test]
fn plain_insert_without_deletion_is_unchanged() {
    // 지울 글자가 없으면 종전 넣기와 같다 — 넣은 글자는 앞 글자의 모양을 따른다.
    let mut core = load("a_whole_bold", "before");
    let para_idx = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| !p.text.is_empty())
        .unwrap();
    core.insert_text_native(0, para_idx, 2, "X").unwrap();
    let sig = signature(&core, false);
    assert_eq!(sig[2].0, 'X');
    assert!(!sig[2].1, "앞 글자(공백, 보통)의 모양: {sig:?}");
    assert!(sig[3].1 && sig[4].1, "원래 굵은 글자는 그대로: {sig:?}");
}

#[test]
fn native_style_inheriting_replace_normalizes_newlines_before_counting_offsets() {
    // #7979의 devel 병합 해소 계약: 공용 Native 진입점도 CRLF/CR을 LF로
    // 정규화한 뒤 글자 수와 이벤트 위치를 정한다. 화면 좌표 검사가 아니다.
    for replacement in ["가\r\n나\r다", "가\n나\n다"] {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "앞굵게뒤").unwrap();
        core.apply_char_format_native(0, 0, 1, 3, r#"{"bold":true}"#)
            .unwrap();
        let bold_id = core.document().sections[0].paragraphs[0]
            .char_shape_id_at(1)
            .unwrap();
        let result = core
            .replace_text_inheriting_char_shapes_native(0, 0, 1, 2, replacement)
            .unwrap();
        let paragraph = &core.document().sections[0].paragraphs[0];
        assert_eq!(paragraph.text, "앞가\n나\n다뒤", "입력 {replacement:?}");
        assert_eq!(result, r#"{"ok":true,"charOffset":6}"#);
        for offset in 1..6 {
            assert_eq!(paragraph.char_shape_id_at(offset), Some(bold_id));
        }
        assert_eq!(paragraph.char_shape_id_at(6), Some(0), "뒤 글자 모양 보존");
    }
}
