//! 구역·단 정의는 문단 맨 앞을 지킨다.
//!
//! 개체나 각주로 시작하는 구역 첫 문단의 맨 앞에 글을 넣으면, 글은 정의 뒤와 개체 앞에
//! 놓여야 한다. 그 자리에서 문단을 나누면 개체는 뒷문단 맨 앞에 남아야 한다. 한컴
//! 산출물은 언제나 정의 제어문자가 글자보다 앞이고, 한글 2022는 정의가 글 뒤에 놓인
//! 문서를 열다 멈춘다(#4680).

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::{Paragraph, TitleMark};

/// 문단 하나의 글과 (컨트롤 종류, 글자 위치) 목록. 누름틀 범위는 `field`·`field-end` 로 싣는다.
type Para = (String, Vec<(&'static str, usize)>);

/// 문단 맨 앞에 둘 개체: 각주, 미주, 글자처럼 취급한 수식·표, 떠 있는 사각형.
const OBJECTS: [&str; 5] = ["fn", "en", "eqed", "tbl", "gso"];

fn kind(control: &Control) -> &'static str {
    match control {
        Control::SectionDef(_) => "secd",
        Control::ColumnDef(_) => "cold",
        Control::Footnote(_) => "fn",
        Control::Endnote(_) => "en",
        Control::Equation(_) => "eqed",
        Control::Table(_) => "tbl",
        Control::Shape(_) => "gso",
        Control::Field(_) => "clk",
        _ => "other",
    }
}

fn layout(core: &DocumentCore) -> Vec<Para> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .map(|para| {
            let mut controls: Vec<_> = para
                .controls
                .iter()
                .zip(para.control_text_positions())
                .map(|(control, at)| (kind(control), at))
                .collect();
            for range in &para.field_ranges {
                controls.push(("field", range.start_char_idx));
                controls.push(("field-end", range.end_char_idx));
            }
            (para.text.clone(), controls)
        })
        .collect()
}

fn para(text: &str, controls: &[(&'static str, usize)]) -> Para {
    (text.to_string(), controls.to_vec())
}

fn insert_object(core: &mut DocumentCore, object: &str, para_idx: usize, offset: usize) {
    match object {
        "fn" => core.insert_footnote_native(0, para_idx, offset),
        "en" => core.insert_endnote_native(0, para_idx, offset),
        "eqed" => core.insert_equation_native(0, para_idx, offset, "x+1", 1000, 0),
        "tbl" => core.create_table_ex_native(0, para_idx, offset, 1, 1, true, None, None),
        "gso" => core.create_shape_control_native(
            0,
            para_idx,
            offset,
            3000,
            3000,
            0,
            0,
            false,
            "Square",
            "rectangle",
            false,
            false,
            &[],
        ),
        _ => unreachable!(),
    }
    .unwrap();
}

/// 첫 문단이 `{secd}{cold}{object}가나다`, 둘째·셋째 문단이 "라마"·"바사"인 문서.
fn fixture(object: &'static str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나다라마바사").unwrap();
    core.split_paragraph_native(0, 0, 5, None).unwrap();
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    insert_object(&mut core, object, 0, 0);
    assert_eq!(
        layout(&core)[..1],
        [para("가나다", &[("secd", 0), ("cold", 0), (object, 0)])],
        "{object}: 준비한 첫 문단"
    );
    core
}

/// 어긋난 경우를 모아 한꺼번에 보여 준다.
#[derive(Default)]
struct Failures(Vec<String>);

impl Failures {
    /// 편집 직후 상태와 HWP·HWPX로 저장했다가 다시 연 상태를 `expected` 와 견준다.
    fn check(&mut self, case: String, core: &DocumentCore, expected: &[Para]) {
        let reopen = |bytes: Vec<u8>| layout(&DocumentCore::from_bytes(&bytes).unwrap());
        let states = [
            ("편집 직후", layout(core)),
            ("HWP 다시 열기", reopen(core.export_hwp_native().unwrap())),
            ("HWPX 다시 열기", reopen(core.export_hwpx_native().unwrap())),
        ];
        for (stage, actual) in states {
            if actual != expected {
                self.0.push(format!("{case} / {stage}: {actual:?}"));
            }
        }
    }

    fn assert_none(self) {
        assert!(
            self.0.is_empty(),
            "{}건 어긋남:\n{}",
            self.0.len(),
            self.0.join("\n")
        );
    }
}

/// 첫 문단 뒤에 그대로 남는 "라마"·"바사" 문단.
fn rest() -> [Para; 2] {
    [para("라마", &[]), para("바사", &[])]
}

#[test]
fn typing_before_a_leading_object_keeps_definitions_first() {
    let mut failures = Failures::default();
    for object in OBJECTS {
        let mut core = fixture(object);
        core.insert_text_native(0, 0, 0, "제목").unwrap();
        let [p1, p2] = rest();
        let expected = [
            para("제목가나다", &[("secd", 0), ("cold", 0), (object, 2)]),
            p1,
            p2,
        ];
        failures.check(format!("{object}: 맨 앞 입력"), &core, &expected);
    }
    failures.assert_none();
}

#[test]
fn composing_before_a_leading_object_keeps_definitions_first() {
    let mut failures = Failures::default();
    for object in OBJECTS {
        let mut core = fixture(object);
        // IME 조합: 조합 중인 글자를 지우고 새 글자를 넣는다.
        core.replace_body_text_local_native(0, 0, 0, 0, "ㅎ")
            .unwrap();
        core.replace_body_text_local_native(0, 0, 0, 1, "하")
            .unwrap();
        core.replace_body_text_local_native(0, 0, 0, 1, "한")
            .unwrap();
        let [p1, p2] = rest();
        let expected = [
            para("한가나다", &[("secd", 0), ("cold", 0), (object, 1)]),
            p1,
            p2,
        ];
        failures.check(format!("{object}: 맨 앞 조합"), &core, &expected);
    }
    failures.assert_none();
}

#[test]
fn splitting_before_a_leading_object_keeps_it_first_in_the_new_paragraph() {
    type Split = fn(&mut DocumentCore) -> Result<String, rhwp::error::HwpError>;
    let splits: [(&str, Split); 3] = [
        ("Enter", |core| core.split_paragraph_native(0, 0, 0, None)),
        ("쪽 나누기", |core| {
            core.insert_page_break_native(0, 0, 0)
        }),
        ("단 나누기", |core| {
            core.insert_column_break_native(0, 0, 0)
        }),
    ];
    let mut failures = Failures::default();
    for object in OBJECTS {
        for (name, split) in splits {
            let mut core = fixture(object);
            split(&mut core).unwrap();
            let [p1, p2] = rest();
            let expected = [
                para("", &[("secd", 0), ("cold", 0)]),
                para("가나다", &[(object, 0)]),
                p1,
                p2,
            ];
            failures.check(format!("{object}: 맨 앞 {name}"), &core, &expected);
        }
    }
    failures.assert_none();
}

#[test]
fn pasting_before_a_leading_object_keeps_definitions_first() {
    let mut failures = Failures::default();
    for object in OBJECTS {
        // 한 문단 조각은 정의 뒤, 개체 앞에 들어간다.
        let mut core = fixture(object);
        core.copy_selection_native(0, 1, 0, 1, 2).unwrap();
        core.paste_internal_native(0, 0, 0).unwrap();
        let [p1, p2] = rest();
        let expected = [
            para("라마가나다", &[("secd", 0), ("cold", 0), (object, 2)]),
            p1,
            p2,
        ];
        failures.check(format!("{object}: 한 문단 붙이기"), &core, &expected);

        // 여러 문단 조각은 첫 조각이 정의 뒤에 붙고, 개체는 마지막 조각 뒤에 놓인다.
        let mut core = fixture(object);
        core.copy_selection_native(0, 1, 0, 2, 2).unwrap();
        core.paste_internal_native(0, 0, 0).unwrap();
        let [p1, p2] = rest();
        let expected = [
            para("라마", &[("secd", 0), ("cold", 0)]),
            para("바사가나다", &[(object, 2)]),
            p1,
            p2,
        ];
        failures.check(format!("{object}: 여러 문단 붙이기"), &core, &expected);

        let mut core = fixture(object);
        core.paste_html_native(0, 0, 0, "<p>X</p>").unwrap();
        let [p1, p2] = rest();
        let expected = [
            para("X가나다", &[("secd", 0), ("cold", 0), (object, 1)]),
            p1,
            p2,
        ];
        failures.check(format!("{object}: HTML 한 문단 붙이기"), &core, &expected);

        let mut core = fixture(object);
        core.paste_html_native(0, 0, 0, "<p>X</p><p>Y</p>").unwrap();
        let [p1, p2] = rest();
        let expected = [
            para("X", &[("secd", 0), ("cold", 0)]),
            para("Y가나다", &[(object, 1)]),
            p1,
            p2,
        ];
        failures.check(format!("{object}: HTML 여러 문단 붙이기"), &core, &expected);

        // 표가 든 HTML은 문단째 들어간다. 정의만 남은 앞 문단은 표 문단을 합쳐 정의를 지키고,
        // 개체는 뒷문단 맨 앞에 남는다.
        let mut core = fixture(object);
        core.paste_html_native(0, 0, 0, "<table><tr><td>A</td></tr></table>")
            .unwrap();
        let [p1, p2] = rest();
        let expected = [
            para("", &[("secd", 0), ("cold", 0), ("tbl", 0)]),
            para("가나다", &[(object, 0)]),
            p1,
            p2,
        ];
        failures.check(format!("{object}: HTML 표 붙이기"), &core, &expected);
    }
    failures.assert_none();
}

#[test]
fn splitting_before_an_inline_object_mid_paragraph_keeps_it_first() {
    // 떠 있는 개체는 캐럿 자리를 차지하지 않아 나누는 축이 따로 어긋난다. 여기서는 뺀다.
    let mut failures = Failures::default();
    for object in ["fn", "en", "eqed", "tbl"] {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "가나다앞라마").unwrap();
        core.split_paragraph_native(0, 0, 3, None).unwrap();
        insert_object(&mut core, object, 1, 1);
        let joined = [
            para("가나다", &[("secd", 0), ("cold", 0)]),
            para("앞라마", &[(object, 1)]),
        ];
        assert_eq!(layout(&core), joined, "{object}: 준비한 문서");

        core.split_paragraph_native(0, 1, 1, None).unwrap();
        let split = [
            para("가나다", &[("secd", 0), ("cold", 0)]),
            para("앞", &[]),
            para("라마", &[(object, 0)]),
        ];
        failures.check(format!("{object}: 개체 바로 앞 Enter"), &core, &split);

        // 지우개로 합쳤다가 되돌리면 합친 자리에서 다시 나눈다.
        core.merge_paragraph_native(0, 2).unwrap();
        failures.check(
            format!("{object}: 개체로 시작하는 문단 합치기"),
            &core,
            &joined,
        );
        core.split_paragraph_native(0, 1, 1, None).unwrap();
        failures.check(format!("{object}: 합치기 되돌리기"), &core, &split);
    }
    failures.assert_none();
}

#[test]
fn splitting_right_before_a_click_here_field_keeps_it_first() {
    // 누름틀 시작과 빈 누름틀의 끝 표지도 나눈 자리 갭의 8유닛 자리다. 새 문단 첫 글자 앞에
    // 그 자리가 없으면 누름틀이 문단 끝으로 밀린다.
    type Split = fn(&mut DocumentCore) -> Result<String, rhwp::error::HwpError>;
    let splits: [(&str, Split); 3] = [
        ("Enter", |core| core.split_paragraph_native(0, 0, 2, None)),
        ("쪽 나누기", |core| {
            core.insert_page_break_native(0, 0, 2)
        }),
        ("단 나누기", |core| {
            core.insert_column_break_native(0, 0, 2)
        }),
    ];
    let mut failures = Failures::default();
    // 채운 누름틀은 `가나[다라]마`, 빈 누름틀은 `가나[]다라마`.
    for (value, end) in [("다라", 2), ("", 0)] {
        for (name, split) in splits {
            let mut core = DocumentCore::new_empty();
            core.create_blank_document_native().unwrap();
            core.insert_text_native(
                0,
                0,
                0,
                if value.is_empty() {
                    "가나다라마"
                } else {
                    "가나마"
                },
            )
            .unwrap();
            core.insert_click_here_field_at(0, 0, 2, "안내문", "", "칸", true)
                .unwrap();
            if !value.is_empty() {
                core.set_field_value_by_name("칸", value).unwrap();
            }
            assert_eq!(
                layout(&core),
                [para(
                    "가나다라마",
                    &[
                        ("secd", 0),
                        ("cold", 0),
                        ("clk", 2),
                        ("field", 2),
                        ("field-end", 2 + end)
                    ]
                )],
                "{value:?}: 준비한 문서"
            );
            split(&mut core).unwrap();
            let expected = [
                para("가나", &[("secd", 0), ("cold", 0)]),
                para("다라마", &[("clk", 0), ("field", 0), ("field-end", end)]),
            ];
            failures.check(format!("{value:?} 누름틀 바로 앞 {name}"), &core, &expected);
        }
    }
    failures.assert_none();
}

#[test]
fn title_marks_in_the_gap_keep_their_slots() {
    // 제목 차례 표시는 컨트롤 없는 8유닛 자리다. 세지 않으면 같은 갭의 컨트롤이 한 칸씩 밀린다.
    let footnote = || Control::Footnote(Box::default());
    let mark = |char_idx| TitleMark {
        char_idx,
        ignore: false,
    };

    // `{제목 차례}{cold}{fn}가` 맨 앞 입력: 표지와 단 정의는 글 앞, 각주는 글 뒤.
    let mut para = Paragraph {
        text: "가".into(),
        char_offsets: vec![24],
        controls: vec![Control::ColumnDef(Default::default()), footnote()],
        title_marks: vec![mark(0)],
        ..Paragraph::new_empty()
    };
    para.insert_text_at(0, "제목");
    assert_eq!(para.char_offsets, [16, 17, 26]);
    assert_eq!(para.control_text_positions(), [0, 2]);

    // `가{제목 차례}{fn}나`를 각주 바로 앞에서 나누면 표지와 각주가 새 문단 맨 앞에 간다.
    let mut para = Paragraph {
        text: "가나".into(),
        char_offsets: vec![0, 17],
        controls: vec![footnote()],
        title_marks: vec![mark(1)],
        ..Paragraph::new_empty()
    };
    let tail = para.split_at(1);
    assert_eq!(tail.text, "나");
    assert_eq!(tail.title_marks, [mark(0)]);
    assert_eq!(tail.char_offsets, [16]);
    assert_eq!(tail.control_text_positions(), [0]);
}

#[test]
fn typing_at_the_start_of_an_ordinary_first_paragraph_is_unchanged() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나다").unwrap();
    core.insert_text_native(0, 0, 0, "제목").unwrap();
    core.replace_body_text_local_native(0, 0, 0, 0, "한")
        .unwrap();
    let mut failures = Failures::default();
    let expected = [para("한제목가나다", &[("secd", 0), ("cold", 0)])];
    failures.check("개체 없는 첫 문단".to_string(), &core, &expected);
    failures.assert_none();
}
