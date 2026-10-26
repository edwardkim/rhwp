//! 회전·대칭한 다각형은 편집 중 그림과 HWP/HWPX 저장 뒤 다시 연 그림의 위치가 같다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::Point;

/// 쪽 SVG에서 다각형 경로의 꼭짓점 좌표를 읽는다.
fn polygon(core: &DocumentCore) -> Vec<(f64, f64)> {
    let svg = core.render_page_svg_native(0).unwrap();
    let start = svg.find("<path d=\"M").unwrap() + "<path d=\"".len();
    let end = start + svg[start..].find('"').unwrap();
    svg[start..end]
        .split(['M', 'L'])
        .filter(|part| !part.trim().is_empty())
        .map(|part| {
            let mut xy = part.split_whitespace().map(|v| v.parse::<f64>().unwrap());
            (xy.next().unwrap(), xy.next().unwrap())
        })
        .collect()
}

fn assert_close(actual: &[(f64, f64)], expected: &[(f64, f64)]) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert!(
            (a.0 - e.0).abs() < 0.01 && (a.1 - e.1).abs() < 0.01,
            "{actual:?} != {expected:?}"
        );
    }
}

#[test]
fn transformed_polygon_keeps_live_position_after_save_and_reopen() {
    for props in [
        r#"{"rotationAngle":90}"#,
        r#"{"rotationAngle":270}"#,
        r#"{"horzFlip":true}"#,
        r#"{"vertFlip":true}"#,
        r#"{"rotationAngle":30,"width":12000}"#,
    ] {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "본문").unwrap();
        let points = [
            Point { x: 0, y: 0 },
            Point { x: 9000, y: 0 },
            Point { x: 0, y: 4500 },
        ];
        let made: serde_json::Value = serde_json::from_str(
            &core
                .create_shape_control_native(
                    0,
                    0,
                    0,
                    9000,
                    4500,
                    15000,
                    15000,
                    false,
                    "InFrontOfText",
                    "polygon",
                    false,
                    false,
                    &points,
                )
                .unwrap(),
        )
        .unwrap();
        let control = made["controlIdx"].as_u64().unwrap() as usize;
        core.set_shape_properties_native(0, 0, control, props)
            .unwrap();
        let live = polygon(&core);
        if props == r#"{"rotationAngle":90}"# {
            // 120×60px 상자(200,200)를 가운데(260,230) 기준으로 90도 돌린 자리다.
            assert_close(&live, &[(290.0, 170.0), (290.0, 290.0), (230.0, 170.0)]);
        }
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_close(&polygon(&reopened), &live);
        }
    }
}
