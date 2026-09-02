//! [Issue #6358] 깨진 음수 셀 pad 가 Center 정렬 텍스트를 셀 밖 +130px 로 보낸다.
//!
//! 전축0 표의 수직 미지정 폴백은 `cell.pad < 2500` 이면 셀 저장값을 쓴다.
//! 음수(-19215 HU)도 그 한도를 통과해 `inner_height` 가 부풀고, valign=Center
//! 가 `" □ 비용 : "` 런을 자기 셀(y≈236) 밖 y≈366 에 놓는다.
//! 음수는 결측 센티널로 보고 표 기본(0)으로 폴백한다.

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
        apply_inner_margin: false,
        ..Default::default()
    };
    let paint = cell.effective_padding(&Padding::default());
    assert_eq!(
        (paint.left, paint.right, paint.top, paint.bottom),
        (0, 0, 32, 0),
        "음수 수직 pad 는 전축0 폴백에서 표 기본 0 이어야 한다"
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
        apply_inner_margin: false,
        ..Default::default()
    };
    let paint = cell.effective_padding(&Padding::default());
    assert_eq!(
        (paint.left, paint.right, paint.top, paint.bottom),
        (0, 0, 141, 141),
        "정상 수직 pad 141 은 #2195 전축0 폴백을 유지해야 한다"
    );
}
