//! 단계 10 단위 테스트 — EMR_HEADER 파싱 + 레코드 시퀀스 읽기.

use super::parser::records::Record;
use super::*;

/// 최소 88바이트 EMR_HEADER + EMR_EOF(0x14바이트) 조합. ext 없음.
fn fixture_minimal_header_eof() -> Vec<u8> {
    let mut b = Vec::with_capacity(88 + 20);

    // EMR_HEADER (88바이트)
    b.extend_from_slice(&1u32.to_le_bytes()); // Type=1
    b.extend_from_slice(&88u32.to_le_bytes()); // Size=88
                                               // Bounds RECTL (16B)
    b.extend_from_slice(&0i32.to_le_bytes());
    b.extend_from_slice(&0i32.to_le_bytes());
    b.extend_from_slice(&1000i32.to_le_bytes());
    b.extend_from_slice(&500i32.to_le_bytes());
    // Frame RECTL (16B) — 0.01mm 단위
    b.extend_from_slice(&0i32.to_le_bytes());
    b.extend_from_slice(&0i32.to_le_bytes());
    b.extend_from_slice(&10000i32.to_le_bytes());
    b.extend_from_slice(&5000i32.to_le_bytes());
    // Signature " EMF"
    b.extend_from_slice(&0x464D4520u32.to_le_bytes());
    // Version
    b.extend_from_slice(&0x00010000u32.to_le_bytes());
    // Bytes (전체 파일 크기) / Records / Handles / Reserved
    b.extend_from_slice(&108u32.to_le_bytes()); // 88 + 20 EOF
    b.extend_from_slice(&2u32.to_le_bytes()); // Records
    b.extend_from_slice(&1u16.to_le_bytes()); // Handles
    b.extend_from_slice(&0u16.to_le_bytes()); // Reserved
                                              // nDescription / offDescription / nPalEntries
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    // Device SIZEL (8B)
    b.extend_from_slice(&1920i32.to_le_bytes());
    b.extend_from_slice(&1080i32.to_le_bytes());
    // Millimeters SIZEL (8B)
    b.extend_from_slice(&508i32.to_le_bytes());
    b.extend_from_slice(&286i32.to_le_bytes());
    assert_eq!(b.len(), 88);

    // EMR_EOF (최소 20바이트: type+size + nPalEntries(4) + offPalEntries(4) + SizeLast(4))
    b.extend_from_slice(&14u32.to_le_bytes()); // Type=14
    b.extend_from_slice(&20u32.to_le_bytes()); // Size=20
    b.extend_from_slice(&0u32.to_le_bytes()); // nPalEntries
    b.extend_from_slice(&0u32.to_le_bytes()); // offPalEntries
    b.extend_from_slice(&20u32.to_le_bytes()); // SizeLast

    b
}

#[test]
fn parses_minimal_header_and_eof() {
    let bytes = fixture_minimal_header_eof();
    let records = parse_emf(&bytes).expect("parse");
    assert_eq!(records.len(), 2);

    match &records[0] {
        Record::Header(h) => {
            assert_eq!(h.signature, 0x464D4520);
            assert_eq!(h.bounds.right, 1000);
            assert_eq!(h.bounds.bottom, 500);
            assert_eq!(h.device.cx, 1920);
            assert_eq!(h.handles, 1);
            assert!(h.ext1.is_none());
            assert!(h.ext2.is_none());
        }
        other => panic!("first record not Header: {other:?}"),
    }
    matches!(records[1], Record::Eof);
}

#[test]
fn rejects_bad_signature() {
    let mut bytes = fixture_minimal_header_eof();
    // signature는 EMR_HEADER 시작 + 40 = offset 40에 위치. Type/Size 제외하면 [40..44].
    bytes[40] = 0xAA;
    bytes[41] = 0xBB;
    bytes[42] = 0xCC;
    bytes[43] = 0xDD;
    match parse_emf(&bytes) {
        Err(Error::InvalidSignature { got }) => {
            assert_eq!(got, 0xDDCCBBAA);
        }
        other => panic!("expected InvalidSignature, got {other:?}"),
    }
}

#[test]
fn rejects_non_header_first_record() {
    // type=14(EOF)를 선두에 둔 손상된 EMF.
    let mut b = Vec::new();
    b.extend_from_slice(&14u32.to_le_bytes());
    b.extend_from_slice(&20u32.to_le_bytes());
    b.extend_from_slice(&[0u8; 12]);
    match parse_emf(&b) {
        Err(Error::InvalidFirstRecord { got }) => assert_eq!(got, 14),
        other => panic!("expected InvalidFirstRecord, got {other:?}"),
    }
}

#[test]
fn rejects_misaligned_record() {
    let mut b = Vec::new();
    b.extend_from_slice(&1u32.to_le_bytes());
    b.extend_from_slice(&87u32.to_le_bytes()); // size not multiple of 4
    b.extend_from_slice(&[0u8; 79]);
    match parse_emf(&b) {
        Err(Error::MisalignedRecord { size, .. }) => assert_eq!(size, 87),
        other => panic!("expected MisalignedRecord, got {other:?}"),
    }
}

#[test]
fn parses_header_with_extensions() {
    // Size=108 (ext1+ext2 포함)로 빌드.
    let mut b = Vec::new();
    b.extend_from_slice(&1u32.to_le_bytes());
    b.extend_from_slice(&108u32.to_le_bytes()); // Size=108
                                                // 80B 본체
    for _ in 0..20 {
        b.extend_from_slice(&0u32.to_le_bytes());
    }
    // signature는 offset 40에 있어야 함 → b[40..44]. 지금까지 8(type/size)+80 = 88.
    // 이미 0으로 채워져 있으니 offset 40의 4바이트를 signature로 교체.
    b[40..44].copy_from_slice(&0x464D4520u32.to_le_bytes());

    // ext1 (12B): cbPixelFormat, offPixelFormat, bOpenGL
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&1u32.to_le_bytes());
    // ext2 (8B)
    b.extend_from_slice(&12345u32.to_le_bytes());
    b.extend_from_slice(&6789u32.to_le_bytes());
    assert_eq!(b.len(), 108);

    // EOF
    b.extend_from_slice(&14u32.to_le_bytes());
    b.extend_from_slice(&20u32.to_le_bytes());
    b.extend_from_slice(&[0u8; 12]);

    let records = parse_emf(&b).expect("parse");
    let Record::Header(h) = &records[0] else {
        panic!()
    };
    assert_eq!(h.ext1.unwrap().b_open_gl, 1);
    assert_eq!(h.ext2.unwrap().micrometers_x, 12345);
    assert_eq!(h.ext2.unwrap().micrometers_y, 6789);
}

/// 단계 11 — 객체/상태 레코드 파싱 테스트용 픽스처 빌더.
fn header_prefix() -> Vec<u8> {
    // 최소 88B 헤더. fixture_minimal_header_eof에서 EOF 제외.
    let mut b = fixture_minimal_header_eof();
    b.truncate(88);
    b
}

fn push_record(b: &mut Vec<u8>, rt: u32, payload: &[u8]) {
    let size = 8u32 + payload.len() as u32;
    assert!(size.is_multiple_of(4), "record size must be 4-aligned");
    b.extend_from_slice(&rt.to_le_bytes());
    b.extend_from_slice(&size.to_le_bytes());
    b.extend_from_slice(payload);
}

fn push_eof(b: &mut Vec<u8>) {
    push_record(b, 14, &[0u8; 12]);
}

#[test]
fn parses_create_pen_and_select_and_delete() {
    let mut b = header_prefix();

    // EMR_CREATEPEN (0x26): handle=1, style=0(PS_SOLID), width=2, reserved=0, color=0x00FF0000
    let mut pen_payload = Vec::new();
    pen_payload.extend_from_slice(&1u32.to_le_bytes()); // handle
    pen_payload.extend_from_slice(&0u32.to_le_bytes()); // style
    pen_payload.extend_from_slice(&2i32.to_le_bytes()); // width.x
    pen_payload.extend_from_slice(&0i32.to_le_bytes()); // width.y
    pen_payload.extend_from_slice(&0x00FF0000u32.to_le_bytes()); // color
    push_record(&mut b, 0x26, &pen_payload);

    // EMR_SELECTOBJECT (0x25): handle=1
    push_record(&mut b, 0x25, &1u32.to_le_bytes());

    // EMR_DELETEOBJECT (0x28): handle=1
    push_record(&mut b, 0x28, &1u32.to_le_bytes());

    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    assert_eq!(recs.len(), 5);

    match &recs[1] {
        Record::CreatePen { handle, pen } => {
            assert_eq!(*handle, 1);
            assert_eq!(pen.style, 0);
            assert_eq!(pen.width, 2);
            assert_eq!(pen.color, 0x00FF0000);
        }
        other => panic!("expected CreatePen, got {other:?}"),
    }
    assert!(matches!(recs[2], Record::SelectObject { handle: 1 }));
    assert!(matches!(recs[3], Record::DeleteObject { handle: 1 }));
    assert!(matches!(recs[4], Record::Eof));
}

#[test]
fn parses_create_brush_indirect() {
    let mut b = header_prefix();
    // EMR_CREATEBRUSHINDIRECT (0x27): handle=2, style=0, color=0x00112233, hatch=0
    let mut p = Vec::new();
    p.extend_from_slice(&2u32.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    p.extend_from_slice(&0x00112233u32.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    push_record(&mut b, 0x27, &p);
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    match &recs[1] {
        Record::CreateBrushIndirect { handle, brush } => {
            assert_eq!(*handle, 2);
            assert_eq!(brush.color, 0x00112233);
        }
        other => panic!("expected CreateBrushIndirect, got {other:?}"),
    }
}

#[test]
fn parses_ext_create_font_indirect_w() {
    let mut b = header_prefix();
    // EMR_EXTCREATEFONTINDIRECTW (0x52): handle(4) + LogFontW(92) = 96B payload.
    let mut p = Vec::new();
    p.extend_from_slice(&3u32.to_le_bytes()); // handle
    p.extend_from_slice(&(-12i32).to_le_bytes()); // height
    p.extend_from_slice(&0i32.to_le_bytes()); // width
    p.extend_from_slice(&0i32.to_le_bytes()); // escapement
    p.extend_from_slice(&0i32.to_le_bytes()); // orientation
    p.extend_from_slice(&700i32.to_le_bytes()); // weight (bold)
    p.extend_from_slice(&[1u8, 0, 0, 1, 0, 0, 0, 0]); // italic/underline/strikeout/charset + precisions
                                                      // FaceName "Arial" + null padding
    let face: Vec<u16> = "Arial".encode_utf16().collect();
    for w in &face {
        p.extend_from_slice(&w.to_le_bytes());
    }
    for _ in face.len()..32 {
        p.extend_from_slice(&0u16.to_le_bytes());
    }
    assert_eq!(p.len(), 96);
    push_record(&mut b, 0x52, &p);
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    match &recs[1] {
        Record::ExtCreateFontIndirectW { handle, font } => {
            assert_eq!(*handle, 3);
            assert_eq!(font.weight, 700);
            assert_eq!(font.italic, 1);
            assert_eq!(font.face_name, "Arial");
            assert_eq!(font.height, -12);
        }
        other => panic!("expected ExtCreateFontIndirectW, got {other:?}"),
    }
}

#[test]
fn parses_dc_stack_and_world_transform() {
    let mut b = header_prefix();
    push_record(&mut b, 0x21, &[]); // EMR_SAVEDC
                                    // EMR_SETWORLDTRANSFORM (0x23): XForm(24B)
    let mut p = Vec::new();
    for v in [2.0_f32, 0.0, 0.0, 3.0, 10.0, 20.0] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    push_record(&mut b, 0x23, &p);
    // EMR_RESTOREDC (0x22): relative=-1
    push_record(&mut b, 0x22, &(-1i32).to_le_bytes());
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    assert!(matches!(recs[1], Record::SaveDC));
    match &recs[2] {
        Record::SetWorldTransform(x) => {
            assert!((x.m11 - 2.0).abs() < 1e-6);
            assert!((x.m22 - 3.0).abs() < 1e-6);
            assert!((x.dx - 10.0).abs() < 1e-6);
            assert!((x.dy - 20.0).abs() < 1e-6);
        }
        other => panic!("expected SetWorldTransform, got {other:?}"),
    }
    assert!(matches!(recs[3], Record::RestoreDC { relative: -1 }));
}

#[test]
fn parses_window_viewport_and_colors() {
    let mut b = header_prefix();
    // EMR_SETWINDOWEXTEX (0x09): SizeL (cx=100, cy=200)
    let mut p = Vec::new();
    p.extend_from_slice(&100i32.to_le_bytes());
    p.extend_from_slice(&200i32.to_le_bytes());
    push_record(&mut b, 0x09, &p);
    // EMR_SETVIEWPORTORGEX (0x0C): PointL (x=5, y=6)
    let mut p = Vec::new();
    p.extend_from_slice(&5i32.to_le_bytes());
    p.extend_from_slice(&6i32.to_le_bytes());
    push_record(&mut b, 0x0C, &p);
    // EMR_SETTEXTCOLOR (0x18): 0x00ABCDEF
    push_record(&mut b, 0x18, &0x00ABCDEFu32.to_le_bytes());
    // EMR_SETBKMODE (0x12): 1=transparent
    push_record(&mut b, 0x12, &1u32.to_le_bytes());
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    match &recs[1] {
        Record::SetWindowExtEx(s) => {
            assert_eq!(s.cx, 100);
            assert_eq!(s.cy, 200);
        }
        other => panic!("expected SetWindowExtEx, got {other:?}"),
    }
    match &recs[2] {
        Record::SetViewportOrgEx(p) => {
            assert_eq!(p.x, 5);
            assert_eq!(p.y, 6);
        }
        other => panic!("expected SetViewportOrgEx, got {other:?}"),
    }
    assert!(matches!(recs[3], Record::SetTextColor(0x00ABCDEF)));
    assert!(matches!(recs[4], Record::SetBkMode(1)));
}

#[test]
fn dc_stack_save_restore_round_trip() {
    use super::converter::DcStack;

    let mut dc = DcStack::new();
    assert_eq!(dc.depth(), 0);
    dc.current_mut().text_color = 0x111111;
    dc.save();
    assert_eq!(dc.depth(), 1);
    dc.current_mut().text_color = 0x222222;
    dc.save();
    dc.current_mut().text_color = 0x333333;
    // Pop 1 — returns to state after first save (text_color=0x222222)
    assert!(dc.restore(-1));
    assert_eq!(dc.current().text_color, 0x222222);
    assert!(dc.restore(-1));
    assert_eq!(dc.current().text_color, 0x111111);
    assert!(!dc.restore(-1)); // 스택 비었으므로 실패
}

#[test]
fn object_table_insert_get_remove() {
    use super::converter::{GraphicsObject, ObjectTable};
    use super::parser::objects::LogPen;

    let mut table = ObjectTable::new();
    table.insert(
        1,
        GraphicsObject::Pen(LogPen {
            style: 0,
            width: 2,
            _reserved: 0,
            color: 0x00FF0000,
        }),
    );
    assert!(table.get(1).is_some());
    assert_eq!(table.len(), 1);
    table.remove(1);
    assert!(table.get(1).is_none());
    assert!(table.is_empty());
}

// ───── 단계 12: 드로잉 레코드 + SVG 컨버터 ─────

#[test]
fn parses_rectangle_and_ellipse() {
    let mut b = header_prefix();
    // EMR_RECTANGLE (0x2B): RectL(16B)
    let mut p = Vec::new();
    for v in [10i32, 20, 110, 120] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    push_record(&mut b, 0x2B, &p);
    // EMR_ELLIPSE (0x2A): RectL
    let mut p2 = Vec::new();
    for v in [0i32, 0, 50, 30] {
        p2.extend_from_slice(&v.to_le_bytes());
    }
    push_record(&mut b, 0x2A, &p2);
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    match &recs[1] {
        Record::Rectangle(r) => {
            assert_eq!((r.left, r.top, r.right, r.bottom), (10, 20, 110, 120));
        }
        other => panic!("expected Rectangle, got {other:?}"),
    }
    assert!(matches!(recs[2], Record::Ellipse(_)));
}

#[test]
fn parses_polyline16_with_points() {
    let mut b = header_prefix();
    // EMR_POLYLINE16 (0x56): RectL(16) + count(4) + POINTS[3] (4B each) = 32B
    let mut p = Vec::new();
    for v in [0i32, 0, 100, 100] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    p.extend_from_slice(&3u32.to_le_bytes());
    for (x, y) in [(0i16, 0i16), (50, 50), (100, 0)] {
        p.extend_from_slice(&x.to_le_bytes());
        p.extend_from_slice(&y.to_le_bytes());
    }
    push_record(&mut b, 0x56, &p);
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    match &recs[1] {
        Record::Polyline16 { points, .. } => {
            assert_eq!(points, &vec![(0, 0), (50, 50), (100, 0)]);
        }
        other => panic!("expected Polyline16, got {other:?}"),
    }
}

#[test]
fn polyline16_rejects_unbounded_point_count() {
    // EMR_POLYLINE16 레코드에 악의적인 count(u32::MAX)를 넣고 실제 포인트
    // 데이터는 0바이트만 둔다. 검증 없이 Vec::with_capacity(count)를
    // 호출하면 ~17GB 예약을 시도해 OOM abort로 이어진다 — 남은 바이트 기준
    // 상한이 걸려 빈 포인트 목록으로 정상 반환해야 한다.
    let mut b = header_prefix();
    let mut p = Vec::new();
    for v in [0i32, 0, 100, 100] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    p.extend_from_slice(&u32::MAX.to_le_bytes()); // count (조작된 값)
    push_record(&mut b, 0x56, &p);
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("count가 남은 바이트 기준으로 상한이 걸려 정상 파싱돼야 함");
    match &recs[1] {
        Record::Polyline16 { points, .. } => {
            assert!(
                points.is_empty(),
                "남은 바이트가 없으므로 포인트는 비어야 함"
            );
        }
        other => panic!("expected Polyline16, got {other:?}"),
    }
}

#[test]
fn parses_path_begin_end_fill() {
    let mut b = header_prefix();
    push_record(&mut b, 0x3B, &[]); // BeginPath
    push_record(&mut b, 0x3C, &[]); // EndPath
                                    // FillPath: RectL
    let mut p = Vec::new();
    for v in [0i32, 0, 10, 10] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    push_record(&mut b, 0x3E, &p);
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    assert!(matches!(recs[1], Record::BeginPath));
    assert!(matches!(recs[2], Record::EndPath));
    assert!(matches!(recs[3], Record::FillPath(_)));
}

#[test]
fn convert_to_svg_emits_rect_with_stroke_and_fill() {
    // 헤더 + CreatePen(검정 2px) + CreateBrush(빨강) + SelectObject(pen) + SelectObject(brush) + Rectangle + EOF
    let mut b = header_prefix();

    // pen handle=1, style=0, width=2, color=#000000
    let mut p = Vec::new();
    p.extend_from_slice(&1u32.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    p.extend_from_slice(&2i32.to_le_bytes());
    p.extend_from_slice(&0i32.to_le_bytes());
    p.extend_from_slice(&0x00000000u32.to_le_bytes());
    push_record(&mut b, 0x26, &p);

    // brush handle=2, style=0 solid, color=#0000FF(RGB red in COLORREF is 0x000000FF)
    let mut p = Vec::new();
    p.extend_from_slice(&2u32.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    p.extend_from_slice(&0x000000FFu32.to_le_bytes()); // R=255 → rgb(255,0,0)
    p.extend_from_slice(&0u32.to_le_bytes());
    push_record(&mut b, 0x27, &p);

    push_record(&mut b, 0x25, &1u32.to_le_bytes()); // SelectObject pen
    push_record(&mut b, 0x25, &2u32.to_le_bytes()); // SelectObject brush

    // Rectangle (10,20)-(110,120)
    let mut p = Vec::new();
    for v in [10i32, 20, 110, 120] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    push_record(&mut b, 0x2B, &p);
    push_eof(&mut b);

    let svg = convert_to_svg(&b, (0.0, 0.0, 1000.0, 500.0)).expect("convert");
    assert!(
        svg.starts_with("<g transform=\"matrix("),
        "svg must start with group transform: {svg}"
    );
    assert!(svg.contains("<rect "));
    assert!(svg.contains("fill=\"rgb(255,0,0)\""));
    assert!(svg.contains("stroke=\"rgb(0,0,0)\""));
    assert!(svg.contains("width=\"100\"")); // 110-10
    assert!(svg.contains("height=\"100\"")); // 120-20
    assert!(svg.ends_with("</g>"));
}

#[test]
fn convert_to_svg_polyline_and_ellipse() {
    let mut b = header_prefix();

    // pen handle=1, style=0, width=1, color=#000000
    let mut p = Vec::new();
    p.extend_from_slice(&1u32.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    p.extend_from_slice(&1i32.to_le_bytes());
    p.extend_from_slice(&0i32.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    push_record(&mut b, 0x26, &p);
    push_record(&mut b, 0x25, &1u32.to_le_bytes());

    // Polyline16: 3점
    let mut p = Vec::new();
    for v in [0i32, 0, 100, 100] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    p.extend_from_slice(&3u32.to_le_bytes());
    for (x, y) in [(0i16, 0i16), (50, 50), (100, 0)] {
        p.extend_from_slice(&x.to_le_bytes());
        p.extend_from_slice(&y.to_le_bytes());
    }
    push_record(&mut b, 0x56, &p);

    // Ellipse 0,0-40,40
    let mut p = Vec::new();
    for v in [0i32, 0, 40, 40] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    push_record(&mut b, 0x2A, &p);
    push_eof(&mut b);

    let svg = convert_to_svg(&b, (0.0, 0.0, 100.0, 50.0)).expect("convert");
    assert!(
        svg.contains("<polyline points=\"0,0 50,50 100,0\""),
        "polyline missing: {svg}"
    );
    assert!(
        svg.contains("<ellipse cx=\"20\" cy=\"20\" rx=\"20\" ry=\"20\""),
        "ellipse missing: {svg}"
    );
}

#[test]
fn convert_to_svg_polygon_closes_shape() {
    let mut b = header_prefix();
    let mut p = Vec::new();
    p.extend_from_slice(&1u32.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    p.extend_from_slice(&1i32.to_le_bytes());
    p.extend_from_slice(&0i32.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    push_record(&mut b, 0x26, &p);
    push_record(&mut b, 0x25, &1u32.to_le_bytes());
    // Polygon16
    let mut p = Vec::new();
    for v in [0i32, 0, 10, 10] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    p.extend_from_slice(&3u32.to_le_bytes());
    for (x, y) in [(0i16, 0i16), (10, 0), (5, 10)] {
        p.extend_from_slice(&x.to_le_bytes());
        p.extend_from_slice(&y.to_le_bytes());
    }
    push_record(&mut b, 0x57, &p);
    push_eof(&mut b);

    let svg = convert_to_svg(&b, (0.0, 0.0, 10.0, 10.0)).expect("convert");
    assert!(
        svg.contains("<polygon points=\"0,0 10,0 5,10\""),
        "polygon missing: {svg}"
    );
}

#[test]
fn colorref_conversion_low_byte_is_red() {
    use super::converter::colorref_to_rgb;
    // COLORREF 0x00BBGGRR → R=lowest byte
    assert_eq!(colorref_to_rgb(0x000000FF), "rgb(255,0,0)");
    assert_eq!(colorref_to_rgb(0x0000FF00), "rgb(0,255,0)");
    assert_eq!(colorref_to_rgb(0x00FF0000), "rgb(0,0,255)");
}

// ───── 단계 13: 텍스트·비트맵 레코드 ─────

/// EMR_EXTTEXTOUTW 페이로드 빌더. 텍스트만 담고 offDx는 0.
fn build_ext_text_out_w_payload(text: &str, ref_x: i32, ref_y: i32) -> Vec<u8> {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    let n_chars = utf16.len() as u32;
    // 고정부 68B (payload 기준) + 문자열. offString은 record 기준이므로 68 + 8 = 76.
    let off_string: u32 = 76;
    let mut p = Vec::new();
    // Bounds RECTL (16)
    for v in [0i32, 0, 100, 20] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    // iGraphicsMode, exScale, eyScale
    p.extend_from_slice(&1u32.to_le_bytes());
    p.extend_from_slice(&1.0f32.to_le_bytes());
    p.extend_from_slice(&1.0f32.to_le_bytes());
    // Reference PointL
    p.extend_from_slice(&ref_x.to_le_bytes());
    p.extend_from_slice(&ref_y.to_le_bytes());
    // Chars, offString, Options
    p.extend_from_slice(&n_chars.to_le_bytes());
    p.extend_from_slice(&off_string.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes());
    // Rectangle RECTL (16)
    for v in [0i32, 0, 100, 20] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    // offDx
    p.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(p.len(), 68);
    // OutputString
    for w in &utf16 {
        p.extend_from_slice(&w.to_le_bytes());
    }
    // 4 정렬 패딩
    while p.len() % 4 != 0 {
        p.push(0);
    }
    p
}

#[test]
fn parses_ext_text_out_w_ascii() {
    let mut b = header_prefix();
    let payload = build_ext_text_out_w_payload("Hello", 10, 20);
    push_record(&mut b, 0x54, &payload);
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    match &recs[1] {
        Record::ExtTextOutW(t) => {
            assert_eq!(t.text, "Hello");
            assert_eq!(t.reference.x, 10);
            assert_eq!(t.reference.y, 20);
        }
        other => panic!("expected ExtTextOutW, got {other:?}"),
    }
}

#[test]
fn parses_ext_text_out_w_korean() {
    let mut b = header_prefix();
    let payload = build_ext_text_out_w_payload("가나다", 5, 15);
    push_record(&mut b, 0x54, &payload);
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    match &recs[1] {
        Record::ExtTextOutW(t) => assert_eq!(t.text, "가나다"),
        other => panic!("expected ExtTextOutW, got {other:?}"),
    }
}

#[test]
fn convert_to_svg_text_uses_font_face_name() {
    // CreateFont(Arial, height=12, weight=700) + SelectObject + SetTextColor(빨강) + ExtTextOutW
    let mut b = header_prefix();

    // ExtCreateFontIndirectW: handle=1, LogFontW
    let mut font_payload = Vec::new();
    font_payload.extend_from_slice(&1u32.to_le_bytes()); // handle
    font_payload.extend_from_slice(&(-12i32).to_le_bytes());
    font_payload.extend_from_slice(&0i32.to_le_bytes());
    font_payload.extend_from_slice(&0i32.to_le_bytes());
    font_payload.extend_from_slice(&0i32.to_le_bytes());
    font_payload.extend_from_slice(&700i32.to_le_bytes()); // bold
    font_payload.extend_from_slice(&[0u8; 8]);
    let face: Vec<u16> = "Arial".encode_utf16().collect();
    for w in &face {
        font_payload.extend_from_slice(&w.to_le_bytes());
    }
    for _ in face.len()..32 {
        font_payload.extend_from_slice(&0u16.to_le_bytes());
    }
    assert_eq!(font_payload.len(), 96);
    push_record(&mut b, 0x52, &font_payload);

    // SelectObject handle=1
    push_record(&mut b, 0x25, &1u32.to_le_bytes());
    // SetTextColor 빨강
    push_record(&mut b, 0x18, &0x000000FFu32.to_le_bytes());
    // ExtTextOutW "Hi"
    let payload = build_ext_text_out_w_payload("Hi", 5, 20);
    push_record(&mut b, 0x54, &payload);
    push_eof(&mut b);

    let svg = convert_to_svg(&b, (0.0, 0.0, 100.0, 50.0)).expect("convert");
    assert!(svg.contains("<text "), "no text element: {svg}");
    assert!(svg.contains("font-family=\"Arial\""), "no Arial: {svg}");
    assert!(svg.contains("font-weight=\"bold\""), "no bold: {svg}");
    assert!(svg.contains("fill=\"rgb(255,0,0)\""), "no red fill: {svg}");
    assert!(svg.contains(">Hi</text>"), "no Hi text: {svg}");
}

#[test]
fn parses_stretch_di_bits_and_emits_image() {
    // 최소 DIB: BITMAPINFOHEADER(40B) + bits(4B).
    let mut bmi = Vec::new();
    bmi.extend_from_slice(&40u32.to_le_bytes()); // biSize
    bmi.extend_from_slice(&2i32.to_le_bytes()); // biWidth
    bmi.extend_from_slice(&2i32.to_le_bytes()); // biHeight
    bmi.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    bmi.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    bmi.extend_from_slice(&0u32.to_le_bytes()); // biCompression
    bmi.extend_from_slice(&16u32.to_le_bytes()); // biSizeImage
    bmi.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    bmi.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    bmi.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    bmi.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant
    assert_eq!(bmi.len(), 40);
    let bits: Vec<u8> = (0..16).collect();

    // STRETCHDIBITS 페이로드 (72B 고정 + BMI + bits)
    let off_bmi = 72u32 + 8; // record 기준
    let off_bits = off_bmi + bmi.len() as u32;
    let mut p = Vec::new();
    // Bounds
    for v in [0i32, 0, 2, 2] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    // xDest, yDest
    p.extend_from_slice(&10i32.to_le_bytes());
    p.extend_from_slice(&20i32.to_le_bytes());
    // xSrc, ySrc, cxSrc, cySrc
    for _ in 0..4 {
        p.extend_from_slice(&0i32.to_le_bytes());
    }
    // offBmi, cbBmi
    p.extend_from_slice(&off_bmi.to_le_bytes());
    p.extend_from_slice(&(bmi.len() as u32).to_le_bytes());
    // offBits, cbBits
    p.extend_from_slice(&off_bits.to_le_bytes());
    p.extend_from_slice(&(bits.len() as u32).to_le_bytes());
    // UsageSrc, RasterOp
    p.extend_from_slice(&0u32.to_le_bytes());
    p.extend_from_slice(&0x00CC0020u32.to_le_bytes()); // SRCCOPY
                                                       // cxDest, cyDest
    p.extend_from_slice(&40i32.to_le_bytes());
    p.extend_from_slice(&50i32.to_le_bytes());
    assert_eq!(p.len(), 72);
    p.extend_from_slice(&bmi);
    p.extend_from_slice(&bits);
    while p.len() % 4 != 0 {
        p.push(0);
    }

    let mut b = header_prefix();
    push_record(&mut b, 0x51, &p);
    push_eof(&mut b);

    let recs = parse_emf(&b).expect("parse");
    match &recs[1] {
        Record::StretchDIBits(s) => {
            assert_eq!(s.x_dest, 10);
            assert_eq!(s.y_dest, 20);
            assert_eq!(s.cx_dest, 40);
            assert_eq!(s.cy_dest, 50);
            assert_eq!(s.bmi.len(), 40);
            assert_eq!(s.bits.len(), 16);
        }
        other => panic!("expected StretchDIBits, got {other:?}"),
    }

    // SVG 출력 검증
    let svg = convert_to_svg(&b, (0.0, 0.0, 100.0, 100.0)).expect("convert");
    assert!(svg.contains("<image "));
    assert!(svg.contains("x=\"10\" y=\"20\" width=\"40\" height=\"50\""));
    // [Task #860] BMP → PNG 변환 후 embed (SVG renderer 의 BMP URI 미지원).
    // BMP decode 실패 시 fallback 으로 BMP URI 유지 (graceful degradation).
    assert!(
        svg.contains("href=\"data:image/png;base64,")
            || svg.contains("href=\"data:image/bmp;base64,"),
        "expected PNG (or BMP fallback) data URI"
    );
}

#[test]
fn text_xml_special_chars_are_escaped() {
    let mut b = header_prefix();
    let payload = build_ext_text_out_w_payload("a<b&c", 0, 0);
    push_record(&mut b, 0x54, &payload);
    push_eof(&mut b);

    let svg = convert_to_svg(&b, (0.0, 0.0, 10.0, 10.0)).expect("convert");
    assert!(svg.contains("&lt;b&amp;c"), "xml not escaped: {svg}");
    assert!(!svg.contains(">a<b&c<"), "raw chars present: {svg}");
}

#[test]
fn preserves_unknown_records_as_payload() {
    // Header + Unknown(type=0x00000054 = ExtTextOutW, 단계 13에서 분기) + EOF
    let mut b = fixture_minimal_header_eof();
    // Insert unknown BEFORE eof. fixture_minimal_header_eof는 88 + 20 = 108 바이트.
    // EOF 구간(마지막 20바이트)을 잘라내고, Unknown(16B) + EOF(20B) 재조립.
    let eof = b.split_off(88);
    b.extend_from_slice(&0x00000070u32.to_le_bytes()); // Type=0x70 (미정의, 1차 범위 외)
    b.extend_from_slice(&16u32.to_le_bytes()); // Size=16
    b.extend_from_slice(&0xDEADBEEFu32.to_le_bytes()); // 페이로드
    b.extend_from_slice(&0xCAFEBABEu32.to_le_bytes());
    b.extend_from_slice(&eof);

    let records = parse_emf(&b).expect("parse");
    assert_eq!(records.len(), 3);
    match &records[1] {
        Record::Unknown {
            record_type,
            payload,
        } => {
            assert_eq!(*record_type, 0x00000070);
            assert_eq!(payload.len(), 8);
        }
        other => panic!("expected Unknown, got {other:?}"),
    }
}

// ── [#5637] EMF+ 이중 스트림 재동기 ────────────────────────────────────────

/// EMF+ 시그니처 코멘트 페이로드: [DataSize][b"EMF+"][filler].
fn emfplus_comment_payload(filler: usize) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&((4 + filler) as u32).to_le_bytes());
    p.extend_from_slice(b"EMF+");
    p.resize(p.len() + filler, 0u8);
    p
}

/// 자기-일관 STRETCHDIBITS 페이로드 (고정 72B + BMI 40B + bits 16B).
fn stretch_dibits_payload() -> Vec<u8> {
    let mut bmi = Vec::new();
    bmi.extend_from_slice(&40u32.to_le_bytes()); // biSize
    bmi.extend_from_slice(&2i32.to_le_bytes()); // biWidth
    bmi.extend_from_slice(&2i32.to_le_bytes()); // biHeight
    bmi.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    bmi.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    bmi.extend_from_slice(&[0u8; 24]); // 나머지 필드 0
    let bits = [0xAAu8; 16];

    let mut p = Vec::new();
    for v in [0i32, 0, 2, 2] {
        p.extend_from_slice(&v.to_le_bytes()); // Bounds
    }
    p.extend_from_slice(&0i32.to_le_bytes()); // xDest
    p.extend_from_slice(&0i32.to_le_bytes()); // yDest
    p.extend_from_slice(&[0u8; 16]); // xSrc..cySrc
    p.extend_from_slice(&80u32.to_le_bytes()); // offBmi (record 기준)
    p.extend_from_slice(&40u32.to_le_bytes()); // cbBmi
    p.extend_from_slice(&120u32.to_le_bytes()); // offBits
    p.extend_from_slice(&16u32.to_le_bytes()); // cbBits
    p.extend_from_slice(&0u32.to_le_bytes()); // UsageSrc
    p.extend_from_slice(&0x00CC_0020u32.to_le_bytes()); // SRCCOPY
    p.extend_from_slice(&2i32.to_le_bytes()); // cxDest
    p.extend_from_slice(&2i32.to_le_bytes()); // cyDest
    p.extend_from_slice(&bmi);
    p.extend_from_slice(&bits);
    p
}

#[test]
fn resyncs_after_lying_emfplus_comment() {
    // EMF+ 코멘트 뒤에 프레이밍 밖 바이트(코멘트 Size 필드 거짓말 상황) →
    // 4바이트 정렬 재동기로 뒤쪽 GDI 레코드(SaveDC + EOF)를 살린다.
    let mut b = header_prefix();
    push_record(&mut b, 0x46, &emfplus_comment_payload(8));
    for _ in 0..6 {
        b.extend_from_slice(&0xBF00_0000u32.to_le_bytes()); // 레코드로 안 읽히는 잔여
    }
    push_record(&mut b, 0x21, &[]); // SaveDC
    push_eof(&mut b);

    let records = parse_emf(&b).expect("resync parse");
    assert!(
        records.iter().any(|r| matches!(r, Record::SaveDC)),
        "재동기로 SaveDC 를 살려야 함: {records:?}"
    );
    assert!(
        records.iter().any(|r| matches!(r, Record::Eof)),
        "EOF 까지 도달해야 함"
    );
}

#[test]
fn no_resync_without_emfplus_comment() {
    // 음성 대조: EMF+ 코멘트가 없는 스트림의 같은 손상은 종전대로 오류.
    let mut b = header_prefix();
    let mut plain = Vec::new();
    plain.extend_from_slice(&8u32.to_le_bytes());
    plain.extend_from_slice(b"GDIC"); // EMF+ 아님
    plain.extend_from_slice(&[0u8; 4]);
    push_record(&mut b, 0x46, &plain);
    for _ in 0..6 {
        b.extend_from_slice(&0xBF00_0000u32.to_le_bytes());
    }
    push_record(&mut b, 0x21, &[]);
    push_eof(&mut b);

    assert!(parse_emf(&b).is_err(), "EMF+ 없는 손상 스트림은 오류 유지");
}

#[test]
fn resync_accepts_self_consistent_stretch_dibits_without_chain() {
    // 폴백 비트맵 레코드 뒤가 다시 깨진 스트림 — 연쇄 검증은 실패하지만
    // STRETCHDIBITS 내부 필드 자기-일관성으로 단독 수용해야 한다.
    let mut b = header_prefix();
    push_record(&mut b, 0x46, &emfplus_comment_payload(8));
    for _ in 0..6 {
        b.extend_from_slice(&0xBF00_0000u32.to_le_bytes());
    }
    push_record(&mut b, 0x51, &stretch_dibits_payload());
    for _ in 0..6 {
        b.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // 뒤쪽도 파단 — EOF 없음
    }

    let records = parse_emf(&b).expect("salvage parse");
    assert!(
        records
            .iter()
            .any(|r| matches!(r, Record::StretchDIBits(_))),
        "자기-일관 STRETCHDIBITS 를 살려야 함: {records:?}"
    );
}

#[test]
fn wellformed_stream_with_emfplus_comment_is_unchanged() {
    // EMF+ 코멘트가 있어도 구조가 온전하면 재동기가 개입하지 않는다.
    let mut b = header_prefix();
    push_record(&mut b, 0x46, &emfplus_comment_payload(8));
    push_record(&mut b, 0x21, &[]); // SaveDC
    push_eof(&mut b);

    let records = parse_emf(&b).expect("parse");
    assert_eq!(records.len(), 4, "Header+Comment+SaveDC+EOF: {records:?}");
    assert!(matches!(records[2], Record::SaveDC));
    assert!(matches!(records[3], Record::Eof));
}

#[test]
fn resync_gives_up_gracefully_and_keeps_prefix() {
    // 재동기 지점이 없으면 파단 전까지 모은 **그릴 내용 있는** 프리픽스로 Ok 마감한다.
    let mut b = header_prefix();
    push_record(&mut b, 0x46, &emfplus_comment_payload(8));
    let mut rect = Vec::new();
    for v in [1i32, 2, 30, 40] {
        rect.extend_from_slice(&v.to_le_bytes());
    }
    push_record(&mut b, 0x2B, &rect); // Rectangle — 파단 전 내용
    for _ in 0..8 {
        b.extend_from_slice(&0xBF00_0000u32.to_le_bytes()); // 이후 전부 잔해
    }

    let records = parse_emf(&b).expect("salvage parse");
    assert!(records.iter().any(|r| matches!(r, Record::Rectangle(_))));
    assert!(
        !records.iter().any(|r| matches!(r, Record::Eof)),
        "EOF 는 없어야 함"
    );
}

#[test]
fn unpaintable_prefix_stays_error() {
    // 그릴 내용이 전혀 없는 프리픽스는 EMF+ 여부와 무관하게 오류 유지 —
    // 빈 SVG 보다 placeholder(오류 경로)가 정보량이 많다.
    let mut b = header_prefix();
    push_record(&mut b, 0x46, &emfplus_comment_payload(8));
    push_record(&mut b, 0x21, &[]); // SaveDC — 상태 레코드만
    for _ in 0..8 {
        b.extend_from_slice(&0xBF00_0000u32.to_le_bytes());
    }

    assert!(parse_emf(&b).is_err());
}

#[test]
fn salvages_paintable_prefix_without_emfplus_comment() {
    // EMF+ 무관 대형 손상 스트림(코퍼스 특허청 계열): 파단 전에 그릴 내용이
    // 있으면 프리픽스로 Ok 마감한다. (내용 없는 손상은 여전히 오류 — 위
    // no_resync_without_emfplus_comment 가 그 대조군)
    let mut b = header_prefix();
    let mut p = Vec::new();
    for v in [1i32, 2, 30, 40] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    push_record(&mut b, 0x2B, &p); // Rectangle
    for _ in 0..6 {
        b.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    }

    let records = parse_emf(&b).expect("salvage parse");
    assert!(records.iter().any(|r| matches!(r, Record::Rectangle(_))));
}
