//! Issue #3466: 문단에 `autoNum` 이 있으면 인라인 컨트롤이 한 칸씩 뒤로 밀린다.
//!
//! 자동번호/새번호(HWP 제어문자 `0x12`)는 **8 코드 유닛을 점유하면서 가시 placeholder 를
//! 한 글자 남긴다**(파서 두 경로 공통: `body_text.rs`, `hwpx/section.rs`). 그래서 그 글자
//! 뒤에는 8 갭이 남지 않고(stride 8 − 글자폭 1 = 7, 7/8 = 0), 갭을 순서대로 나눠 주던
//! `control_text_positions()` 에서 자동번호가 **다음 컨트롤의 갭을 가져갔다**.
//!
//! 결과적으로 수식 k 가 수식 k+1 의 자리에 배치되고 마지막 수식은 문단 끝으로 밀려,
//! 표시 흔들림이 아니라 **읽는 뜻이 달라지는** 어순 오류가 났다.

use rhwp::model::control::{AutoNumber, Control, Equation};
use rhwp::model::paragraph::Paragraph;

/// `<autoNum/> " (1)" <equation/> " (2)"` — 제보자가 올린 최소 재현.
#[test]
fn autonum_does_not_steal_following_equation_gap() {
    let para = Paragraph {
        text: "  (1) (2)".to_string(),
        char_offsets: vec![0, 8, 9, 10, 11, 20, 21, 22, 23],
        controls: vec![
            Control::AutoNumber(AutoNumber::default()),
            Control::Equation(Box::<Equation>::default()),
        ],
        ..Default::default()
    };

    assert_eq!(
        para.control_text_positions(),
        vec![0, 5],
        "자동번호는 자기 placeholder 자리(0), 수식은 자기 갭 자리(5)에 놓여야 한다"
    );
}

/// 자동번호 없는 문단은 종전과 동일해야 한다(무회귀).
#[test]
fn plain_inline_equation_positions_unchanged() {
    // "가" <equation/> "나" — 수식은 첫 글자 뒤 갭 8 로 나타난다.
    let para = Paragraph {
        text: "가나".to_string(),
        char_offsets: vec![0, 9],
        controls: vec![Control::Equation(Box::<Equation>::default())],
        ..Default::default()
    };

    assert_eq!(
        para.control_text_positions(),
        vec![1],
        "일반 인라인 수식 위치는 변하지 않아야 한다"
    );
}

/// 탭은 `controls[]` 에 없지만 가시 글자를 남기며 8 코드 유닛을 점유한다(HWP5 `0x09`,
/// HWPX `'\t'` 폭 8). stride 만 보고 판정하면 탭이 컨트롤 자리를 가로채 **반대 방향으로**
/// 어순이 깨지므로, placeholder 문자와 대기 컨트롤 variant 까지 함께 요구해야 한다.
#[test]
fn tab_does_not_consume_a_control_slot() {
    // <tab> "가" <equation/> "나"
    //   idx 0: '\t'  off 0  stride 8   ← 컨트롤이 아니다
    //   idx 1: '가'  off 8  stride 9 → 갭 8 (수식)
    //   idx 2: '나'  off 17
    let para = Paragraph {
        text: "\t가나".to_string(),
        char_offsets: vec![0, 8, 17],
        controls: vec![Control::Equation(Box::<Equation>::default())],
        ..Default::default()
    };

    assert_eq!(
        para.control_text_positions(),
        vec![2],
        "수식은 '가' 뒤(2)여야 한다 — 탭이 자리를 가져가면 0 이 된다"
    );
}

/// 자동번호 + 수식 2개 — 밀림이 누적되지 않는지 확인한다.
#[test]
fn autonum_with_two_equations_keeps_document_order() {
    // <autoNum/> "변 " <eq A/> "와 " <eq B/> "는"
    //   idx 0: autoNum placeholder(' ')  off 0  stride 8
    //   idx 1: '변'                      off 8
    //   idx 2: ' '                       off 9  stride 9 → 갭 8 (eq A)
    //   idx 3: '와'                      off 18
    //   idx 4: ' '                       off 19 stride 9 → 갭 8 (eq B)
    //   idx 5: '는'                      off 28
    let para = Paragraph {
        text: " 변 와 는".to_string(),
        char_offsets: vec![0, 8, 9, 18, 19, 28],
        controls: vec![
            Control::AutoNumber(AutoNumber::default()),
            Control::Equation(Box::<Equation>::default()),
            Control::Equation(Box::<Equation>::default()),
        ],
        ..Default::default()
    };

    assert_eq!(
        para.control_text_positions(),
        vec![0, 3, 5],
        "자동번호 0, 수식 A 는 '변 ' 뒤(3), 수식 B 는 '와 ' 뒤(5)"
    );
}
