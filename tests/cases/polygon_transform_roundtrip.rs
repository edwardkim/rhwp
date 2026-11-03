//! 관계형·고정형: polygon_transform_roundtrip의 공개 입력·편집·저장 계약.
//! 주석 갱신: 2026-11-03T21:22:49+09:00; 직전 파일 변경 커밋: 2026-10-26T23:40:05+09:00 92d19ea0e3690446106a126b747e0e5f611a0837.
//! 측정: 2026-11-03, 96 DPI·색상 경계 보정 전 2px raw 실루엣. 한컴 rotated HWP/HWPX 각 1쪽 및 공개 API 5개 변환 상태 각 1쪽, Native/fresh WASM 각각 전쪽 최저 100.00%. 독립 Print 확대 판독 후 90도 쪽의 절대 꼭짓점만 ±1px, 저장 전후 관계는 별도로 비교한다.
//! 정식 증거: mydocs/pr/assets/semanticist21-20261103/visual-results.json 및 전쪽 TSV·확대 PNG.
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
        let original_snapshot = core.save_snapshot_native();
        core.set_shape_properties_native(0, 0, control, props)
            .unwrap();
        let live = polygon(&core);
        // Getter는 회전 외곽 폭을 내보낸다. 실제 속성 봉지의 되먹임은 no-op이다.
        let applied = core.get_shape_properties_native(0, 0, control).unwrap();
        core.set_shape_properties_native(0, 0, control, &applied)
            .unwrap();
        assert_close(&polygon(&core), &live);
        // 회전·대칭은 대응 속성으로 역연산한다. 크기와 회전을 함께 바꾼 경우는
        // Studio의 실제 undo처럼 위치·로컬 크기·행렬까지 snapshot으로 복원한다.
        if props.contains("width") {
            core.restore_snapshot_native(original_snapshot).unwrap();
        } else {
            let inverse = if props.contains("rotationAngle") {
                r#"{"rotationAngle":0}"#
            } else if props.contains("horzFlip") {
                r#"{"horzFlip":false}"#
            } else {
                r#"{"vertFlip":false}"#
            };
            core.set_shape_properties_native(0, 0, control, inverse)
                .unwrap();
        }
        core.set_shape_properties_native(0, 0, control, props)
            .unwrap();
        assert_close(&polygon(&core), &live);
        if props == r#"{"rotationAngle":90}"# {
            // 120×60px 상자(200,200)를 가운데(260,230) 기준으로 90도 돌린 자리다.
            for (actual, expected) in
                live.iter()
                    .zip([(290.0, 170.0), (290.0, 290.0), (230.0, 170.0)])
            {
                assert!(
                    (actual.0 - expected.0).abs() <= 1.0 && (actual.1 - expected.1).abs() <= 1.0
                );
            }
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

/// 한컴 2024가 30도 돌려 저장한 직사각형·열린 다각형. 회전 행렬은 개체 가운데 기준이고,
/// 위치는 회전 외접 상자의 왼쪽 위(직사각형 13864×14400, 크기 15824×13683)다.
#[test]
fn hancom_rotated_shapes_keep_their_centre_after_save_and_reopen() {
    for bytes in [
        &include_bytes!("../fixtures/hancom_rotated/rotated.hwp")[..],
        &include_bytes!("../fixtures/hancom_rotated/rotated.hwpx")[..],
    ] {
        let core = DocumentCore::from_bytes(bytes).unwrap();
        let svg = core.render_page_svg_native(0).unwrap();
        // 직사각형은 외접 상자 가운데(위치 + 크기/2)를 기준으로 돈다.
        let px = |hu: f64| hu / 75.0;
        let centre = (px(13864.0 + 15824.0 / 2.0), px(14400.0 + 13683.0 / 2.0));
        let rotate = svg.find("rotate(30,").expect("직사각형 회전");
        let args: Vec<f64> = svg[rotate + "rotate(30,".len()..]
            .split(')')
            .next()
            .unwrap()
            .split(',')
            .map(|v| v.parse().unwrap())
            .collect();
        assert!((args[0] - centre.0).abs() < 0.05 && (args[1] - centre.1).abs() < 0.05);
        let live = polygon(&core);
        for saved in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&saved).unwrap();
            assert_close(&polygon(&reopened), &live);
            assert!(reopened
                .render_page_svg_native(0)
                .unwrap()
                .contains(&format!("rotate(30,{},{})", args[0], args[1])));
        }
    }
}
