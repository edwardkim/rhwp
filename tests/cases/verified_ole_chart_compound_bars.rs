//! 공식 배포 HWPX의 차트 막대와 MS-EMF 복합 다각형·장치 브러시 계약을 보호한다.

use rhwp::emf::converter::Player;
use rhwp::emf::parser::objects::{LogBrush, RectL, XForm};
use rhwp::emf::parser::records::{DibPatternBrush, Record};

fn svg(records: &[Record]) -> String {
    let mut player = Player::new((0.0, 0.0, 100.0, 100.0));
    player.play(records).unwrap();
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\">{}</svg>",
        player.svg.into_string()
    )
}

fn raster(svg: &str) -> resvg::tiny_skia::Pixmap {
    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default()).unwrap();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(100, 100).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    pixmap
}

fn compound() -> Record {
    Record::PolyPolygon16 {
        bounds: RectL {
            left: 10,
            top: 10,
            right: 80,
            bottom: 80,
        },
        polygons: vec![
            vec![(10, 10), (80, 10), (80, 80), (10, 80)],
            vec![(20, 20), (40, 20), (40, 40), (20, 40)],
        ],
    }
}

fn solid() -> Vec<Record> {
    vec![
        Record::CreateBrushIndirect {
            handle: 1,
            brush: LogBrush {
                style: 0,
                color: 0x0000_ff00,
                hatch: 0,
            },
        },
        Record::SelectObject { handle: 1 },
        Record::SelectObject {
            handle: 0x8000_0008,
        },
    ]
}

#[test]
fn compound_polygon_hole_obeys_fill_mode_and_saved_dc() {
    let mut records = solid();
    records.extend([
        Record::SaveDC,
        Record::SetPolyFillMode(2),
        Record::RestoreDC { relative: -1 },
        compound(),
    ]);
    let alternate = raster(&svg(&records));
    assert_eq!(
        alternate.pixel(30, 30).unwrap().alpha(),
        0,
        "기본 alternate의 안쪽 윤곽은 구멍"
    );
    assert!(
        alternate.pixel(15, 15).unwrap().alpha() > 0,
        "바깥 영역은 채움"
    );

    let mut winding = solid();
    winding.extend([Record::SetPolyFillMode(2), compound()]);
    let winding = raster(&svg(&winding));
    assert!(
        winding.pixel(30, 30).unwrap().alpha() > 0,
        "같은 방향 윤곽은 winding에서 채움"
    );
}

#[test]
fn polybezier_inside_path_contributes_to_later_fill() {
    let mut records = solid();
    records.push(Record::Rectangle(RectL {
        left: 0,
        top: 0,
        right: 50,
        bottom: 50,
    }));
    records.push(Record::CreateBrushIndirect {
        handle: 2,
        brush: LogBrush {
            style: 0,
            color: 0x00ff_ffff,
            hatch: 0,
        },
    });
    records.extend([
        Record::SelectObject { handle: 2 },
        Record::BeginPath,
        Record::PolyBezier16 {
            bounds: RectL {
                left: 10,
                top: 10,
                right: 30,
                bottom: 30,
            },
            points: vec![
                (10, 20),
                (10, 10),
                (30, 10),
                (30, 20),
                (30, 30),
                (10, 30),
                (10, 20),
            ],
        },
        Record::EndPath,
        Record::FillPath(RectL {
            left: 10,
            top: 10,
            right: 30,
            bottom: 30,
        }),
    ]);
    let image = raster(&svg(&records));
    let center = image.pixel(20, 20).unwrap();
    let background = image.pixel(5, 5).unwrap();
    assert!(
        center.red() > background.red() && center.blue() > background.blue(),
        "곡선으로 감싼 영역은 뒤의 FillPath에서 흰색으로 채움"
    );
}

fn striped_brush() -> DibPatternBrush {
    let mut bmi = vec![0u8; 40];
    bmi[..4].copy_from_slice(&40u32.to_le_bytes());
    bmi[4..8].copy_from_slice(&2i32.to_le_bytes());
    bmi[8..12].copy_from_slice(&2i32.to_le_bytes());
    bmi[12..14].copy_from_slice(&1u16.to_le_bytes());
    bmi[14..16].copy_from_slice(&24u16.to_le_bytes());
    // 두 행 모두 빨강·파랑. 24bpp DIB 행은 4바이트 경계로 채운다.
    let bits = [0, 0, 255, 255, 0, 0, 0, 0].repeat(2);
    DibPatternBrush {
        usage: 0,
        bmi,
        bits,
    }
}

#[test]
fn bitmap_brush_keeps_device_period_under_world_transform() {
    let records = [
        Record::CreateDibPatternBrush {
            handle: 1,
            brush: striped_brush(),
        },
        Record::SelectObject { handle: 1 },
        Record::SelectObject {
            handle: 0x8000_0008,
        },
        Record::SetWorldTransform(XForm {
            m11: 2.0,
            m12: 0.0,
            m21: 0.0,
            m22: 2.0,
            dx: 0.0,
            dy: 0.0,
        }),
        Record::Rectangle(RectL {
            left: 0,
            top: 0,
            right: 20,
            bottom: 20,
        }),
    ];
    let image = raster(&svg(&records));
    assert_eq!(
        image.pixel(6, 6),
        image.pixel(8, 6),
        "브러시는 장치의 2픽셀 주기 유지"
    );
    assert_ne!(
        image.pixel(6, 6),
        image.pixel(7, 6),
        "서로 다른 두 색을 실제로 재생"
    );
}

#[test]
fn compound_polygon_rejects_unbounded_or_inconsistent_counts() {
    use rhwp::emf::parser::{records::drawing::parse_poly_polygon16, Cursor};
    let mut payload = vec![0u8; 16];
    payload.extend_from_slice(&u32::MAX.to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    assert!(parse_poly_polygon16(&mut Cursor::new(&payload)).is_err());

    payload.truncate(16);
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(&4u32.to_le_bytes());
    assert!(matches!(
        parse_poly_polygon16(&mut Cursor::new(&payload)),
        Err(rhwp::emf::Error::InvalidPolygonPointCount {
            declared: 0,
            actual: 4
        })
    ));
}
