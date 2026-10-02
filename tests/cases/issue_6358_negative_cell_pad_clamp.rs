//! [Issue #6358] 깨진 음수 셀 pad 가 Center 정렬 텍스트를 셀 밖 +130px 로 보낸다.
//!
//! 활성 셀 여백의 음수(-19215 HU)는 결측으로 보고 표 기본(0)으로 폴백한다.
//! 정상 양수 셀 여백을 유지하려면 hasMargin이 켜진 입력이어야 한다.
//! 비활성 보존값은 표 기본 0을 덮어쓰지 않으며 #1785 대조 검사가 확인한다.
//! 사양과 #6101 정상 문서 전체 11쪽의 Native 시각 최저92.26525%가 독립 근거다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::table::Cell;
use rhwp::model::Padding;

#[test]
fn issue_6358_negative_vertical_pad_falls_back_to_table_zero() {
    let cell = Cell {
        padding: Padding {
            left: -13888,
            right: -14867,
            top: 32,
            bottom: -19215,
        },
        apply_inner_margin: true,
        ..Default::default()
    };
    let paint = cell.effective_padding(&Padding::default());
    assert_eq!(
        (paint.left, paint.right, paint.top, paint.bottom),
        (0, 0, 32, 0),
        "활성 셀의 음수 수직 pad 는 표 기본 0으로 폴백해야 한다"
    );
}

#[test]
fn issue_6358_small_positive_vertical_pad_is_kept() {
    let cell = Cell {
        padding: Padding {
            left: 0,
            right: 0,
            top: 141,
            bottom: 141,
        },
        apply_inner_margin: true,
        ..Default::default()
    };
    let paint = cell.effective_padding(&Padding::default());
    assert_eq!(
        (paint.left, paint.right, paint.top, paint.bottom),
        (0, 0, 141, 141),
        "활성 셀의 정상 수직 pad 141은 표 기본 0보다 우선해야 한다"
    );
}
