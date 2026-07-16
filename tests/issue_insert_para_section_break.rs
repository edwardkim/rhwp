//! 문단 삽입 시 구역 나누기 표식 이관 회귀 테스트.
//!
//! 구역의 첫 문단은 `column_type=Section` / `raw_break_type=0x03` 을 지닌다 — "여기서 구역이
//! 시작한다". 이 표식은 문단 **내용**이 아니라 **자리**에 딸린 속성이다.
//!
//! `insert_paragraph_native(sec, 0)` 은 새 빈 문단을 0번에 넣고 기존 문단을 1번으로 민다.
//! 표식을 옮기지 않으면 밀려난 문단이 1번에서 계속 구역 시작을 주장해 거기서 쪽이 끊기고,
//! 새 빈 문단만 홀로 남은 쪽이 앞에 생긴다.
//!
//! `split_paragraph_native` 는 원본 객체가 0번에 남아 표식이 딸려 있으므로 이 문제가 없다.
//! 두 경로가 같은 결과를 내야 한다 — 함께 잠근다.

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::ColumnBreakType;

/// "제목" 한 줄짜리 새 문서.
fn one_line_document() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("blank2010 템플릿 로드");
    core.insert_text_native(0, 0, 0, "제목")
        .expect("본문 문단에 텍스트");
    core
}

/// 전제: 구역 첫 문단이 구역 시작 표식을 지닌다. 이게 깨지면 아래 테스트들의 의미가 없다.
#[test]
fn section_first_paragraph_carries_section_break() {
    let core = one_line_document();
    let first = &core.document().sections[0].paragraphs[0];
    assert_eq!(
        first.column_type,
        ColumnBreakType::Section,
        "구역 첫 문단은 구역 시작 표식을 지녀야 한다"
    );
}

/// 0번에 문단을 끼워도 쪽수가 늘지 않는다.
///
/// 한 줄짜리 제목 + 빈 문단이 2쪽이 될 이유는 없다.
#[test]
fn insert_paragraph_at_zero_keeps_page_count() {
    let mut core = one_line_document();
    assert_eq!(core.page_count(), 1, "삽입 전 1쪽이어야 한다");

    core.insert_paragraph_native(0, 0).expect("0번에 문단 삽입");

    assert_eq!(
        core.page_count(),
        1,
        "빈 문단 하나를 앞에 끼웠을 뿐인데 쪽이 늘었다 — 밀려난 문단이 구역 시작 표식을 \
         쥔 채 1번으로 가서 거기서 쪽을 끊었다"
    );
}

/// 표식은 밀려난 문단이 아니라 새 첫 문단이 지녀야 한다.
///
/// 쪽수만 보면 표식이 양쪽에 다 있거나 다 없는 경우를 놓친다.
#[test]
fn insert_paragraph_at_zero_moves_section_break_to_new_first() {
    let mut core = one_line_document();
    core.insert_paragraph_native(0, 0).expect("0번에 문단 삽입");

    let paras = &core.document().sections[0].paragraphs;
    assert_eq!(
        paras[0].column_type,
        ColumnBreakType::Section,
        "새 첫 문단이 구역 시작 표식을 이어받아야 한다"
    );
    assert_eq!(
        paras[1].column_type,
        ColumnBreakType::None,
        "밀려난 문단은 표식을 놓아야 한다 — 쥐고 있으면 거기서 쪽이 끊긴다"
    );
    assert_eq!(paras[1].text, "제목", "밀려난 문단이 제목이어야 한다");
    assert_eq!(
        paras[0].raw_break_type, 0x03,
        "raw_break_type 도 함께 이관되어야 한다 (직렬화에 쓰인다)"
    );
    assert_eq!(paras[1].raw_break_type, 0x00, "밀려난 문단의 raw_break_type 은 비어야 한다");
}

/// 0번이 아닌 자리의 표식은 문단을 따라간다.
///
/// 중간 문단의 쪽/단 나누기는 사용자가 그 문단에 직접 넣은 것이므로 옮기면 안 된다.
#[test]
fn insert_paragraph_in_middle_leaves_break_flags_alone() {
    let mut core = one_line_document();
    core.insert_paragraph_native(0, 1).expect("1번에 문단 삽입");

    let paras = &core.document().sections[0].paragraphs;
    assert_eq!(
        paras[0].column_type,
        ColumnBreakType::Section,
        "0번 문단의 표식은 그대로 있어야 한다"
    );
    assert_eq!(core.page_count(), 1, "뒤에 빈 문단을 붙였을 뿐이니 1쪽");
}

/// 두 삽입 경로가 같은 결과를 낸다.
///
/// `split_paragraph_native(0, 0, 0)` 과 `insert_paragraph_native(0, 0)` 은 둘 다
/// "앞에 빈 문단을 만든다"는 같은 일을 한다. 쪽수가 갈리면 하나는 틀린 것이다.
#[test]
fn insert_and_split_agree_on_page_count() {
    let mut inserted = one_line_document();
    inserted.insert_paragraph_native(0, 0).expect("insert");

    let mut split = one_line_document();
    split.split_paragraph_native(0, 0, 0).expect("split");

    assert_eq!(
        inserted.page_count(),
        split.page_count(),
        "같은 결과를 만드는 두 경로가 쪽수를 다르게 낸다 — insert={}쪽 split={}쪽",
        inserted.page_count(),
        split.page_count()
    );
}
