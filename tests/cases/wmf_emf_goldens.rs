//! M09x: 현행 WMF/EMF 변환을 골든으로 잠근다.
//!
//! 엔진을 고치거나 새 파서를 만들지 않는다. 한컴 정답 오라클이 아니라
//! `WMFConverter`/`parse_emf`/`convert_to_svg` 의 현재 출력을 고정한다.
//!
//! 골든 갱신: `UPDATE_WMF_EMF_GOLDENS=1 cargo test --test <suite> wmf_emf_goldens`

use std::path::{Path, PathBuf};

use rhwp::emf::Record;
use rhwp::wmf::converter::{SVGPlayer, WMFConverter};
use rhwp::wmf::parser::MetafileHeader;

const FIXTURE_DIR: &str = "tests/fixtures/m09x_wmf_emf";
const WMF_CORPUS_DIR: &str = "fuzz/corpus/parse_wmf";

struct GoldenCase {
    id: &'static str,
    kind: &'static str,
    /// `parse_wmf` 코퍼스에 같은 바이트를 둘 파일명. EMF 는 하네스가 없어 None.
    seed_name: Option<&'static str>,
    bytes: fn() -> Vec<u8>,
}

const CASES: &[GoldenCase] = &[
    GoldenCase {
        id: "wmf_minimal_placeable",
        kind: "wmf",
        seed_name: Some("minimal_placeable.wmf"),
        bytes: wmf_minimal_placeable,
    },
    GoldenCase {
        id: "wmf_placeable_rect",
        kind: "wmf",
        seed_name: Some("m09x_placeable_rect.wmf"),
        bytes: wmf_placeable_rect,
    },
    GoldenCase {
        id: "wmf_placeable_ellipse",
        kind: "wmf",
        seed_name: Some("m09x_placeable_ellipse.wmf"),
        bytes: wmf_placeable_ellipse,
    },
    GoldenCase {
        id: "wmf_placeable_line",
        kind: "wmf",
        seed_name: Some("m09x_placeable_line.wmf"),
        bytes: wmf_placeable_line,
    },
    GoldenCase {
        id: "wmf_standard_header_eof",
        kind: "wmf",
        seed_name: Some("m09x_standard_header.wmf"),
        bytes: wmf_standard_header_eof,
    },
    GoldenCase {
        id: "emf_header_eof",
        kind: "emf",
        seed_name: None,
        bytes: emf_header_eof,
    },
    GoldenCase {
        id: "emf_rectangle",
        kind: "emf",
        seed_name: None,
        bytes: emf_rectangle,
    },
    GoldenCase {
        id: "emf_ellipse",
        kind: "emf",
        seed_name: None,
        bytes: emf_ellipse,
    },
    GoldenCase {
        id: "emf_line",
        kind: "emf",
        seed_name: None,
        bytes: emf_line,
    },
];

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn fixture_dir() -> PathBuf {
    repo_path(FIXTURE_DIR)
}

fn golden_path(id: &str) -> PathBuf {
    fixture_dir().join(format!("{id}.golden"))
}

fn corpus_path(name: &str) -> PathBuf {
    repo_path(WMF_CORPUS_DIR).join(name)
}

fn update_requested() -> bool {
    matches!(
        std::env::var("UPDATE_WMF_EMF_GOLDENS").as_deref(),
        Ok("1") | Ok("true")
    )
}

fn wmf_placeable_prefix(left: i16, top: i16, right: i16, bottom: i16) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&0x9AC6_CDD7u32.to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
    b.extend_from_slice(&left.to_le_bytes());
    b.extend_from_slice(&top.to_le_bytes());
    b.extend_from_slice(&right.to_le_bytes());
    b.extend_from_slice(&bottom.to_le_bytes());
    b.extend_from_slice(&1440u16.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
    push_wmf_standard_header(&mut b);
    b
}

fn push_wmf_standard_header(b: &mut Vec<u8>) {
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&9u16.to_le_bytes());
    b.extend_from_slice(&0x0300u16.to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
}

fn push_wmf_eof(b: &mut Vec<u8>) {
    b.extend_from_slice(&3u32.to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
}

fn push_wmf_rect_like(
    b: &mut Vec<u8>,
    function: u16,
    left: i16,
    top: i16,
    right: i16,
    bottom: i16,
) {
    b.extend_from_slice(&7u32.to_le_bytes());
    b.extend_from_slice(&function.to_le_bytes());
    b.extend_from_slice(&bottom.to_le_bytes());
    b.extend_from_slice(&right.to_le_bytes());
    b.extend_from_slice(&top.to_le_bytes());
    b.extend_from_slice(&left.to_le_bytes());
}

fn push_wmf_point(b: &mut Vec<u8>, function: u16, x: i16, y: i16) {
    b.extend_from_slice(&5u32.to_le_bytes());
    b.extend_from_slice(&function.to_le_bytes());
    b.extend_from_slice(&y.to_le_bytes());
    b.extend_from_slice(&x.to_le_bytes());
}

fn wmf_minimal_placeable() -> Vec<u8> {
    let mut b = wmf_placeable_prefix(0, 0, 100, 100);
    push_wmf_eof(&mut b);
    b
}

fn wmf_placeable_rect() -> Vec<u8> {
    let mut b = wmf_placeable_prefix(0, 0, 100, 100);
    push_wmf_rect_like(&mut b, 0x041B, 10, 10, 90, 80);
    push_wmf_eof(&mut b);
    b
}

fn wmf_placeable_ellipse() -> Vec<u8> {
    let mut b = wmf_placeable_prefix(0, 0, 100, 100);
    push_wmf_rect_like(&mut b, 0x0418, 10, 10, 90, 80);
    push_wmf_eof(&mut b);
    b
}

fn wmf_placeable_line() -> Vec<u8> {
    let mut b = wmf_placeable_prefix(0, 0, 100, 100);
    push_wmf_point(&mut b, 0x0214, 10, 10);
    push_wmf_point(&mut b, 0x0213, 90, 80);
    push_wmf_eof(&mut b);
    b
}

fn wmf_standard_header_eof() -> Vec<u8> {
    let mut b = Vec::new();
    push_wmf_standard_header(&mut b);
    push_wmf_eof(&mut b);
    b
}

fn emf_header(
    bounds: (i32, i32, i32, i32),
    frame: (i32, i32, i32, i32),
    bytes: u32,
    records: u32,
) -> Vec<u8> {
    let mut b = Vec::with_capacity(88);
    b.extend_from_slice(&1u32.to_le_bytes());
    b.extend_from_slice(&88u32.to_le_bytes());
    for v in [bounds.0, bounds.1, bounds.2, bounds.3] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    for v in [frame.0, frame.1, frame.2, frame.3] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b.extend_from_slice(&0x464D_4520u32.to_le_bytes());
    b.extend_from_slice(&0x0001_0000u32.to_le_bytes());
    b.extend_from_slice(&bytes.to_le_bytes());
    b.extend_from_slice(&records.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&1920i32.to_le_bytes());
    b.extend_from_slice(&1080i32.to_le_bytes());
    b.extend_from_slice(&508i32.to_le_bytes());
    b.extend_from_slice(&286i32.to_le_bytes());
    debug_assert_eq!(b.len(), 88);
    b
}

fn push_emf_eof(b: &mut Vec<u8>) {
    b.extend_from_slice(&14u32.to_le_bytes());
    b.extend_from_slice(&20u32.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&20u32.to_le_bytes());
}

fn push_emf_rect_like(
    b: &mut Vec<u8>,
    record_type: u32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) {
    b.extend_from_slice(&record_type.to_le_bytes());
    b.extend_from_slice(&24u32.to_le_bytes());
    b.extend_from_slice(&left.to_le_bytes());
    b.extend_from_slice(&top.to_le_bytes());
    b.extend_from_slice(&right.to_le_bytes());
    b.extend_from_slice(&bottom.to_le_bytes());
}

fn push_emf_point(b: &mut Vec<u8>, record_type: u32, x: i32, y: i32) {
    b.extend_from_slice(&record_type.to_le_bytes());
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&x.to_le_bytes());
    b.extend_from_slice(&y.to_le_bytes());
}

fn emf_header_eof() -> Vec<u8> {
    let mut b = emf_header((0, 0, 1000, 500), (0, 0, 10000, 5000), 108, 2);
    push_emf_eof(&mut b);
    b
}

fn emf_rectangle() -> Vec<u8> {
    let mut b = emf_header((0, 0, 100, 100), (0, 0, 2540, 2540), 132, 3);
    push_emf_rect_like(&mut b, 0x2B, 10, 10, 90, 80);
    push_emf_eof(&mut b);
    b
}

fn emf_ellipse() -> Vec<u8> {
    let mut b = emf_header((0, 0, 100, 100), (0, 0, 2540, 2540), 132, 3);
    push_emf_rect_like(&mut b, 0x2A, 10, 10, 90, 80);
    push_emf_eof(&mut b);
    b
}

fn emf_line() -> Vec<u8> {
    let mut b = emf_header((0, 0, 100, 100), (0, 0, 2540, 2540), 140, 4);
    push_emf_point(&mut b, 0x1B, 10, 10);
    push_emf_point(&mut b, 0x36, 90, 80);
    push_emf_eof(&mut b);
    b
}

fn emf_record_kind(record: &Record) -> String {
    match record {
        Record::Header(_) => "Header".to_string(),
        Record::Eof => "Eof".to_string(),
        Record::CreatePen { .. } => "CreatePen".to_string(),
        Record::CreateBrushIndirect { .. } => "CreateBrushIndirect".to_string(),
        Record::ExtCreateFontIndirectW { .. } => "ExtCreateFontIndirectW".to_string(),
        Record::SelectObject { .. } => "SelectObject".to_string(),
        Record::DeleteObject { .. } => "DeleteObject".to_string(),
        Record::SaveDC => "SaveDC".to_string(),
        Record::RestoreDC { .. } => "RestoreDC".to_string(),
        Record::SetWorldTransform(_) => "SetWorldTransform".to_string(),
        Record::ModifyWorldTransform { .. } => "ModifyWorldTransform".to_string(),
        Record::SetMapMode(_) => "SetMapMode".to_string(),
        Record::SetWindowExtEx(_) => "SetWindowExtEx".to_string(),
        Record::SetWindowOrgEx(_) => "SetWindowOrgEx".to_string(),
        Record::SetViewportExtEx(_) => "SetViewportExtEx".to_string(),
        Record::SetViewportOrgEx(_) => "SetViewportOrgEx".to_string(),
        Record::SetBkMode(_) => "SetBkMode".to_string(),
        Record::SetTextAlign(_) => "SetTextAlign".to_string(),
        Record::SetTextColor(_) => "SetTextColor".to_string(),
        Record::SetBkColor(_) => "SetBkColor".to_string(),
        Record::MoveToEx(_) => "MoveToEx".to_string(),
        Record::LineTo(_) => "LineTo".to_string(),
        Record::Rectangle(_) => "Rectangle".to_string(),
        Record::RoundRect { .. } => "RoundRect".to_string(),
        Record::Ellipse(_) => "Ellipse".to_string(),
        Record::Arc { .. } => "Arc".to_string(),
        Record::Chord { .. } => "Chord".to_string(),
        Record::Pie { .. } => "Pie".to_string(),
        Record::Polyline16 { .. } => "Polyline16".to_string(),
        Record::Polygon16 { .. } => "Polygon16".to_string(),
        Record::PolyBezier16 { .. } => "PolyBezier16".to_string(),
        Record::BeginPath => "BeginPath".to_string(),
        Record::EndPath => "EndPath".to_string(),
        Record::CloseFigure => "CloseFigure".to_string(),
        Record::FillPath(_) => "FillPath".to_string(),
        Record::StrokePath(_) => "StrokePath".to_string(),
        Record::StrokeAndFillPath(_) => "StrokeAndFillPath".to_string(),
        Record::ExtTextOutW(_) => "ExtTextOutW".to_string(),
        Record::StretchDIBits(_) => "StretchDIBits".to_string(),
        Record::Unknown { record_type, .. } => format!("Unknown(0x{record_type:08X})"),
    }
}

fn snapshot_wmf(bytes: &[u8]) -> String {
    let header = match MetafileHeader::parse(&mut &*bytes) {
        Ok((MetafileHeader::StartsWithPlaceable(placeable, header), _)) => format!(
            "placeable bounds=({},{},{},{}) inch={} type={:?} version={:?} objects={}",
            placeable.bounding_box.left,
            placeable.bounding_box.top,
            placeable.bounding_box.right,
            placeable.bounding_box.bottom,
            placeable.inch,
            header.typ,
            header.version,
            header.number_of_objects
        ),
        Ok((MetafileHeader::StartsWithHeader(header), _)) => format!(
            "standard type={:?} version={:?} objects={}",
            header.typ, header.version, header.number_of_objects
        ),
        Err(err) => format!("err: {err}"),
    };
    let svg = match WMFConverter::new(bytes, SVGPlayer::new()).run() {
        Ok(out) => String::from_utf8_lossy(&out).into_owned(),
        Err(err) => format!("err: {err}"),
    };
    format!(
        "# M09x wmf golden — current-engine lock\n\
         kind: wmf\n\
         bytes: {}\n\
         \n\
         === header ===\n\
         {header}\n\
         \n\
         === svg ===\n\
         {svg}\n",
        bytes.len()
    )
}

fn snapshot_emf(bytes: &[u8]) -> String {
    let (parse, kinds) = match rhwp::emf::parse_emf(bytes) {
        Ok(records) => {
            let kinds = records
                .iter()
                .map(emf_record_kind)
                .collect::<Vec<_>>()
                .join(", ");
            let header = records.iter().find_map(|record| match record {
                Record::Header(header) => Some(format!(
                    "bounds=({},{},{},{}) frame=({},{},{},{}) signature=0x{:08X} records_field={}",
                    header.bounds.left,
                    header.bounds.top,
                    header.bounds.right,
                    header.bounds.bottom,
                    header.frame.left,
                    header.frame.top,
                    header.frame.right,
                    header.frame.bottom,
                    header.signature,
                    header.records
                )),
                _ => None,
            });
            (
                format!(
                    "ok count={} kinds=[{kinds}]\n{}",
                    records.len(),
                    header.unwrap_or_else(|| "header: missing".to_string())
                ),
                kinds,
            )
        }
        Err(err) => (format!("err: {err}"), String::new()),
    };
    let _ = kinds;
    let fragment = match rhwp::emf::convert_to_svg(bytes, (0.0, 0.0, 100.0, 100.0)) {
        Ok(svg) => svg,
        Err(err) => format!("err: {err}"),
    };
    let standalone = match rhwp::emf::convert_to_standalone_svg(bytes) {
        Some(svg) => String::from_utf8_lossy(&svg).into_owned(),
        None => "none".to_string(),
    };
    format!(
        "# M09x emf golden — current-engine lock\n\
         kind: emf\n\
         bytes: {}\n\
         \n\
         === parse ===\n\
         {parse}\n\
         \n\
         === svg_fragment ===\n\
         {fragment}\n\
         \n\
         === standalone ===\n\
         {standalone}\n",
        bytes.len()
    )
}

fn snapshot_of(case: &GoldenCase) -> String {
    let bytes = (case.bytes)();
    match case.kind {
        "wmf" => snapshot_wmf(&bytes),
        "emf" => snapshot_emf(&bytes),
        other => panic!("unknown golden kind: {other}"),
    }
}

#[test]
fn wmf_emf_goldens_lock_current_engine() {
    assert_eq!(CASES.len(), 9, "M09x 골든 개수가 카탈로그와 어긋난다");
    assert!(CASES.iter().any(|c| c.kind == "wmf"));
    assert!(CASES.iter().any(|c| c.kind == "emf"));
    let mut ids = CASES.iter().map(|c| c.id).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), CASES.len(), "골든 id 가 중복된다");

    std::fs::create_dir_all(fixture_dir()).expect("create fixture dir");
    std::fs::create_dir_all(repo_path(WMF_CORPUS_DIR)).expect("create parse_wmf corpus dir");
    let update = update_requested();
    let mut mismatches = Vec::new();

    for case in CASES {
        let bytes = (case.bytes)();
        if let Some(seed_name) = case.seed_name {
            let seed_path = corpus_path(seed_name);
            if update || !seed_path.exists() {
                if !update && !seed_path.exists() {
                    mismatches.push(format!(
                        "{}: parse_wmf 시드 없음 ({}) — UPDATE_WMF_EMF_GOLDENS=1 로 생성",
                        case.id,
                        seed_path.display()
                    ));
                } else {
                    std::fs::write(&seed_path, &bytes)
                        .unwrap_or_else(|e| panic!("write {}: {e}", seed_path.display()));
                }
            } else {
                let on_disk = std::fs::read(&seed_path)
                    .unwrap_or_else(|e| panic!("read {}: {e}", seed_path.display()));
                if on_disk != bytes {
                    mismatches.push(format!(
                        "{}: parse_wmf 시드가 빌더와 다르다 ({})",
                        case.id,
                        seed_path.display()
                    ));
                }
            }
        }

        let actual = snapshot_of(case);
        let path = golden_path(case.id);
        if update || !path.exists() {
            if !update && !path.exists() {
                mismatches.push(format!(
                    "{}: 골든 파일 없음 ({}) — UPDATE_WMF_EMF_GOLDENS=1 로 생성",
                    case.id,
                    path.display()
                ));
                continue;
            }
            std::fs::write(&path, actual.as_bytes())
                .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
            continue;
        }
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        if expected != actual {
            mismatches.push(format!(
                "{} ({})\n--- expected ---\n{expected}\n--- actual ---\n{actual}",
                case.id,
                path.display()
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "WMF/EMF 골든 불일치 {}건 (의도된 엔진 변경이면 UPDATE_WMF_EMF_GOLDENS=1):\n{}",
        mismatches.len(),
        mismatches.join("\n\n")
    );
}
