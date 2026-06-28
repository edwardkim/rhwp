use std::path::{Path, PathBuf};

use rhwp::model::control::Control;
use rhwp::model::style::CenterLine;
use rhwp::{parse_document, DocumentCore};

fn sample_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(name)
}

fn read_sample(name: &str) -> Vec<u8> {
    std::fs::read(sample_path(name)).unwrap_or_else(|err| panic!("{name} 읽기 실패: {err}"))
}

fn first_table(doc: &rhwp::model::document::Document) -> &rhwp::model::table::Table {
    doc.sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|para| &para.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("대각선샘플 표를 찾지 못함")
}

fn line_attr(line: &str, key: &str) -> Option<f64> {
    let needle = format!("{key}=\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}

fn rendered_lines(svg: &str) -> Vec<(f64, f64, f64, f64)> {
    svg.split("<line")
        .skip(1)
        .filter_map(|part| {
            let tag = part.split('>').next()?;
            Some((
                line_attr(tag, "x1")?,
                line_attr(tag, "y1")?,
                line_attr(tag, "x2")?,
                line_attr(tag, "y2")?,
            ))
        })
        .collect()
}

#[test]
fn issue_1623_sample_preserves_centerline_and_cellzones() {
    let doc = parse_document(&read_sample("samples/대각선샘플.hwpx")).expect("HWPX 파싱 실패");
    let table = first_table(&doc);

    assert_eq!(table.zones.len(), 2, "cellzoneList 보존");
    assert_eq!(table.zones[0].border_fill_id, 9);
    assert_eq!(table.zones[1].border_fill_id, 11);
    assert!(
        table.cells.iter().any(|cell| cell.border_fill_id == 10),
        "개별 셀 중심선 BorderFill 참조 보존"
    );

    let bf9 = &doc.doc_info.border_fills[8];
    let bf10 = &doc.doc_info.border_fills[9];
    let bf11 = &doc.doc_info.border_fills[10];
    assert_eq!((bf9.attr >> 2) & 0x07, 0b010, "zone #9 slash");
    assert_eq!((bf9.attr >> 5) & 0x07, 0b010, "zone #9 backSlash");
    assert_eq!(bf10.center_line, CenterLine::Vertical);
    assert_eq!((bf11.attr >> 2) & 0x07, 0b010, "zone #11 slash");
    assert_eq!((bf11.attr >> 5) & 0x07, 0b010, "zone #11 backSlash");
}

#[test]
fn issue_1623_cellzone_diagonal_renders_across_zone_bbox() {
    for sample in ["samples/대각선샘플.hwpx", "samples/대각선샘플.hwp"] {
        let core = DocumentCore::from_bytes(&read_sample(sample))
            .unwrap_or_else(|err| panic!("{sample}: {err}"));
        let svg = core
            .render_page_svg_native(0)
            .unwrap_or_else(|err| panic!("{sample} SVG 렌더 실패: {err}"));
        let lines = rendered_lines(&svg);

        assert!(
            lines
                .iter()
                .any(|(x1, y1, x2, y2)| { (x1 - x2).abs() > 250.0 && (y1 - y2).abs() > 250.0 }),
            "{sample}: cellzone 전체 bbox를 가로지르는 대각선이 없음"
        );
    }
}
