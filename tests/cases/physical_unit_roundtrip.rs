//! 정수 문서 단위의 왕복과 연속 좌표의 절삭을 검사하는 수학적 계약.
//! 실제 문서의 쪽·좌표나 렌더링 baseline을 고정하는 검사가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::{hwpunit_to_px, px_to_hwpunit};

#[test]
fn integer_document_units_survive_pixel_roundtrip() {
    let units = [
        i32::MIN,
        -10604,
        -2253,
        -1,
        0,
        1,
        141,
        7200,
        10604,
        25088,
        25089,
        59528,
        84188,
        i32::MAX,
    ];
    for dpi in [72.0, 96.0, 120.0, 144.0, 192.0, 300.0] {
        for unit in units {
            let pixel = hwpunit_to_px(unit, dpi);
            assert_eq!(px_to_hwpunit(pixel, dpi), unit, "unit={unit}, dpi={dpi}");
        }
    }
}

#[test]
fn fractional_coordinates_keep_truncation_without_integer_source() {
    for (pixel, expected) in [(0.25, 18), (-0.25, -18), (1.1, 82), (-1.1, -82)] {
        assert_eq!(px_to_hwpunit(pixel, 96.0), expected);
    }
    let pixel = hwpunit_to_px(10604, 96.0);
    assert_eq!(px_to_hwpunit(pixel.next_down(), 96.0), 10603);
    assert_eq!(px_to_hwpunit(pixel.next_up(), 96.0), 10604);
}
