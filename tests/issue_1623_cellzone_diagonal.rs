use std::path::{Path, PathBuf};

use rhwp::model::control::Control;
use rhwp::model::style::CenterLine;
use rhwp::{parse_document, wasm_api::HwpDocument, DocumentCore};
use serde_json::Value;

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
    let bf5 = &doc.doc_info.border_fills[4];
    assert_eq!((bf5.attr >> 8) & 0x03, 2, "bf5 slash Crooked=2 보존");
    assert_eq!((bf5.attr >> 5) & 0x07, 0b010, "bf5 backSlash CENTER");
    assert_eq!((bf9.attr >> 2) & 0x07, 0b010, "zone #9 slash");
    assert_eq!((bf9.attr >> 5) & 0x07, 0b010, "zone #9 backSlash");
    assert_eq!(bf10.center_line, CenterLine::Vertical);
    assert_eq!((bf11.attr >> 2) & 0x07, 0b010, "zone #11 slash");
    assert_eq!((bf11.attr >> 5) & 0x07, 0b010, "zone #11 backSlash");
}

#[test]
fn issue_1633_sample3_separates_diagonal_and_centerline() {
    for sample in ["samples/대각선샘플3.hwpx", "samples/대각선샘플3.hwp"] {
        let bytes = read_sample(sample);
        let doc = parse_document(&bytes).unwrap_or_else(|err| panic!("{sample}: {err}"));
        let table = first_table(&doc);

        assert_eq!(table.row_count, 10, "{sample}: row count");
        assert_eq!(table.col_count, 10, "{sample}: col count");

        let mut has_diagonal_only = false;
        let mut has_centerline_only = false;
        for bf in &doc.doc_info.border_fills {
            let slash = (bf.attr >> 2) & 0x07;
            let backslash = (bf.attr >> 5) & 0x07;
            let center_line = if bf.center_line != CenterLine::None {
                bf.center_line
            } else {
                CenterLine::from_hwp_attr(bf.attr)
            };

            if slash != 0 || backslash != 0 {
                has_diagonal_only = true;
                assert_eq!(
                    center_line,
                    CenterLine::None,
                    "{sample}: 대각선 BorderFill은 중심선을 함께 갖지 않아야 함 attr=0x{:04x}",
                    bf.attr
                );
            }
            if center_line != CenterLine::None {
                has_centerline_only = true;
                assert_eq!(slash, 0, "{sample}: 중심선 BorderFill slash 방향");
                assert_eq!(backslash, 0, "{sample}: 중심선 BorderFill backSlash 방향");
            }
        }

        assert!(has_diagonal_only, "{sample}: 대각선-only BorderFill");
        assert!(has_centerline_only, "{sample}: 중심선-only BorderFill");
    }
}

#[test]
fn issue_1633_cell_border_edit_preserves_table_object_height() {
    let bytes = read_sample("samples/대각선샘플3.hwp");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("대각선샘플3 HWP 파싱");
    let before_height = first_table(doc.document()).common.height;
    let before_raw_row_sum = first_table(doc.document())
        .get_row_heights()
        .iter()
        .sum::<u32>();
    assert!(
        before_height > before_raw_row_sum,
        "회귀 가드는 셀 높이 합보다 큰 표 객체 높이가 있는 한컴 저장본이어야 함"
    );

    let centerline_props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":0,
        "diagonalBackSlash":0,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"CROSS"
    }"##;
    doc.set_cell_properties(0, 0, 2, 0, centerline_props)
        .expect("크기 변경 없는 중심선 적용");

    let after_table = first_table(doc.document());
    assert_eq!(
        after_table.common.height, before_height,
        "크기 변경 없는 테두리/대각선 편집은 표 객체 높이를 cell.height 합계로 덮어쓰면 안 됨"
    );
    let exported = doc.export_hwp_with_adapter().expect("HWP export");
    let reparsed = parse_document(&exported).expect("exported HWP 재파싱");
    assert_eq!(
        first_table(&reparsed).common.height,
        before_height,
        "HWP 저장본도 표 객체 높이를 보존해야 함"
    );
}

#[test]
fn issue_1633_sample4_preserves_diagonal_without_centerline() {
    for sample in ["samples/대각선샘플4.hwpx", "samples/대각선샘플4.hwp"] {
        let bytes = read_sample(sample);
        let doc = parse_document(&bytes).unwrap_or_else(|err| panic!("{sample}: {err}"));
        let table = first_table(&doc);

        assert_eq!(table.row_count, 10, "{sample}: row count");
        assert_eq!(table.col_count, 10, "{sample}: col count");
        assert!(
            table.zones.iter().any(|zone| (
                zone.start_row,
                zone.start_col,
                zone.end_row,
                zone.end_col
            ) == (0, 0, 9, 9)),
            "{sample}: 전체 표 cellzone"
        );

        let mut diagonal_only_count = 0;
        for bf in &doc.doc_info.border_fills {
            let slash = (bf.attr >> 2) & 0x07;
            let backslash = (bf.attr >> 5) & 0x07;
            let has_diagonal = slash != 0 || backslash != 0;
            let has_centerline = bf.center_line != CenterLine::None
                || CenterLine::from_hwp_attr(bf.attr) != CenterLine::None;
            if has_diagonal {
                diagonal_only_count += 1;
                assert!(
                    !has_centerline,
                    "{sample}: 대각선-only 비교군에는 중심선이 없어야 함 attr=0x{:04x}",
                    bf.attr
                );
            }
        }
        assert!(
            diagonal_only_count >= 1,
            "{sample}: 대각선 BorderFill을 찾지 못함"
        );

        let wasm_doc =
            HwpDocument::from_bytes(&bytes).unwrap_or_else(|err| panic!("{sample}: {err:?}"));
        let props = wasm_doc
            .get_cell_properties(0, 0, 2, 0)
            .unwrap_or_else(|err| panic!("{sample}: getCellProperties 실패: {err:?}"));
        let props: Value = serde_json::from_str(&props)
            .unwrap_or_else(|err| panic!("{sample}: JSON 파싱 실패: {err}"));
        assert_eq!(props["diagonalSlash"].as_u64(), Some(2), "{sample}");
        assert_eq!(props["diagonalBackSlash"].as_u64(), Some(2), "{sample}");
        assert_eq!(props["centerLine"].as_str(), Some("NONE"), "{sample}");
    }
}

#[test]
fn issue_1633_sample5_keeps_cellzone_and_cell_borderfills_separate() {
    for sample in ["samples/대각선샘플5.hwpx", "samples/대각선샘플5.hwp"] {
        let bytes = read_sample(sample);
        let doc = parse_document(&bytes).unwrap_or_else(|err| panic!("{sample}: {err}"));
        let table = first_table(&doc);

        assert_eq!(table.row_count, 10, "{sample}: row count");
        assert_eq!(table.col_count, 10, "{sample}: col count");
        let zone = table
            .zones
            .iter()
            .find(|zone| {
                (zone.start_row, zone.start_col, zone.end_row, zone.end_col) == (0, 0, 9, 9)
            })
            .unwrap_or_else(|| panic!("{sample}: 전체 cellzone 없음"));
        let cell = table.cells.get(1).expect("1행 2열 셀");
        assert_ne!(
            cell.border_fill_id, zone.border_fill_id,
            "{sample}: 1행 2열 셀 BF는 cellzone BF와 분리되어야 함"
        );

        let zone_bf = &doc.doc_info.border_fills[(zone.border_fill_id - 1) as usize];
        let cell_bf = &doc.doc_info.border_fills[(cell.border_fill_id - 1) as usize];
        assert_eq!(
            zone_bf.borders[0].line_type as u8, 0,
            "{sample}: zone BF는 셀 외곽 테두리를 갖지 않아야 함"
        );
        assert_ne!(
            cell_bf.borders[0].line_type as u8, 0,
            "{sample}: 1행 2열 셀 BF는 셀 테두리를 보존해야 함"
        );
        assert_eq!((cell_bf.attr >> 2) & 0x07, 2, "{sample}: slash");
        assert_eq!((cell_bf.attr >> 5) & 0x07, 2, "{sample}: backSlash");
    }
}

#[test]
fn issue_1633_sample4_renders_cellzone_diagonal_without_cell_overlay() {
    for sample in ["samples/대각선샘플4.hwpx", "samples/대각선샘플4.hwp"] {
        let core = DocumentCore::from_bytes(&read_sample(sample))
            .unwrap_or_else(|err| panic!("{sample}: {err}"));
        let svg = core
            .render_page_svg_native(0)
            .unwrap_or_else(|err| panic!("{sample} SVG 렌더 실패: {err}"));
        let lines = rendered_lines(&svg);

        assert!(
            lines
                .iter()
                .any(|(x1, y1, x2, y2)| { (x1 - x2).abs() > 500.0 && (y1 - y2).abs() > 150.0 }),
            "{sample}: 전체 cellzone X 대각선이 렌더되지 않음"
        );

        let short_cell_diagonals: Vec<_> = lines
            .iter()
            .filter(|(x1, y1, x2, y2)| {
                let dx = (x1 - x2).abs();
                let dy = (y1 - y2).abs();
                (40.0..80.0).contains(&dx) && (10.0..25.0).contains(&dy)
            })
            .collect();
        assert!(
            short_cell_diagonals.is_empty(),
            "{sample}: cellzone 대각선 위에 셀 고유 대각선이 중복 렌더됨: {short_cell_diagonals:?}"
        );
    }
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

#[test]
fn issue_1633_crooked_diagonal_renders_middle_segment() {
    for sample in ["samples/대각선샘플.hwpx", "samples/대각선샘플.hwp"] {
        let core = DocumentCore::from_bytes(&read_sample(sample))
            .unwrap_or_else(|err| panic!("{sample}: {err}"));
        let svg = core
            .render_page_svg_native(0)
            .unwrap_or_else(|err| panic!("{sample} SVG 렌더 실패: {err}"));
        let lines = rendered_lines(&svg);

        assert!(
            lines.iter().any(|(x1, y1, x2, y2)| {
                let len = (x2 - x1).abs();
                (y1 - y2).abs() < 0.01 && (20.0..=35.0).contains(&len) && *y1 < 350.0
            }),
            "{sample}: 첫 줄 두 번째 칸의 꺾은 대각선 가운데 수평 선분이 없음"
        );
    }
}

#[test]
fn issue_1633_get_cell_properties_reflects_cellzone_diagonal() {
    for sample in ["samples/대각선샘플.hwpx", "samples/대각선샘플.hwp"] {
        let bytes = read_sample(sample);
        let parsed = parse_document(&bytes).unwrap_or_else(|err| panic!("{sample}: {err}"));
        let table = first_table(&parsed);
        let cell_idx = table
            .cells
            .iter()
            .position(|cell| cell.row == 2 && cell.col == 2)
            .unwrap_or_else(|| panic!("{sample}: row=2 col=2 셀을 찾지 못함"));
        assert_ne!(
            table.cells[cell_idx].border_fill_id, 11,
            "{sample}: 회귀 가드는 cellzone overlay 조회를 검증해야 함"
        );

        let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|err| panic!("{sample}: {err}"));
        let json = doc
            .get_cell_properties(0, 0, 2, cell_idx as u32)
            .unwrap_or_else(|err| panic!("{sample}: getCellProperties 실패: {err:?}"));
        let props: Value = serde_json::from_str(&json).unwrap_or_else(|err| {
            panic!("{sample}: getCellProperties JSON 파싱 실패: {err}: {json}")
        });

        assert_eq!(props["borderFillId"].as_u64(), Some(11), "{sample}: {json}");
        assert_eq!(props["diagonalLine"].as_u64(), Some(10), "{sample}: {json}");
        assert_eq!(props["diagonalSlash"].as_u64(), Some(2), "{sample}: {json}");
        assert_eq!(
            props["diagonalBackSlash"].as_u64(),
            Some(2),
            "{sample}: {json}"
        );
        assert_eq!(
            props["diagonalWidth"].as_u64(),
            Some(13),
            "{sample}: {json}"
        );
        assert_eq!(
            props["diagonalColor"].as_str(),
            Some("#000000"),
            "{sample}: {json}"
        );
    }
}

#[test]
fn issue_1633_get_cell_own_properties_ignores_cellzone_diagonal() {
    for sample in ["samples/대각선샘플.hwpx", "samples/대각선샘플.hwp"] {
        let bytes = read_sample(sample);
        let parsed = parse_document(&bytes).unwrap_or_else(|err| panic!("{sample}: {err}"));
        let table = first_table(&parsed);
        let zone = table
            .zones
            .iter()
            .find(|zone| zone.border_fill_id == 11)
            .unwrap_or_else(|| panic!("{sample}: BF 11 cellzone을 찾지 못함"));
        let cell_idx = table
            .cells
            .iter()
            .position(|cell| {
                if cell.row < zone.start_row
                    || cell.row > zone.end_row
                    || cell.col < zone.start_col
                    || cell.col > zone.end_col
                    || cell.border_fill_id == zone.border_fill_id
                {
                    return false;
                }
                let Some(bf) = parsed
                    .doc_info
                    .border_fills
                    .get((cell.border_fill_id.saturating_sub(1)) as usize)
                else {
                    return false;
                };
                ((bf.attr >> 2) & 0x07) == 0 && ((bf.attr >> 5) & 0x07) == 0
            })
            .unwrap_or_else(|| panic!("{sample}: cellzone 내부의 개별 대각선 없는 셀을 찾지 못함"));
        assert_ne!(
            table.cells[cell_idx].border_fill_id, 11,
            "{sample}: 회귀 가드는 cellzone과 개별 셀 BF 분리를 검증해야 함"
        );

        let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|err| panic!("{sample}: {err}"));
        let effective_props = doc
            .get_cell_properties(0, 0, 2, cell_idx as u32)
            .unwrap_or_else(|err| panic!("{sample}: effective 셀 속성 조회 실패: {err:?}"));
        let effective_props: Value = serde_json::from_str(&effective_props)
            .unwrap_or_else(|err| panic!("{sample}: effective JSON 파싱 실패: {err}"));
        assert_eq!(effective_props["borderFillId"].as_u64(), Some(11));
        assert_eq!(effective_props["diagonalLine"].as_u64(), Some(10));
        assert_eq!(effective_props["diagonalSlash"].as_u64(), Some(2));
        assert_eq!(effective_props["diagonalBackSlash"].as_u64(), Some(2));

        let own_props = doc
            .get_cell_own_properties(0, 0, 2, cell_idx as u32)
            .unwrap_or_else(|err| panic!("{sample}: 고유 셀 속성 조회 실패: {err:?}"));
        let own_props: Value = serde_json::from_str(&own_props)
            .unwrap_or_else(|err| panic!("{sample}: own JSON 파싱 실패: {err}"));
        assert_ne!(own_props["borderFillId"].as_u64(), Some(11), "{sample}");
        assert_eq!(own_props["diagonalSlash"].as_u64(), Some(0), "{sample}");
        assert_eq!(own_props["diagonalBackSlash"].as_u64(), Some(0), "{sample}");
    }
}

#[test]
fn issue_1633_as_one_cell_diagonal_uses_cellzone_range() {
    let mut doc = HwpDocument::create_empty();
    let created = doc
        .create_table_ex_native(
            0,
            0,
            0,
            3,
            3,
            true,
            Some(&[10000, 10000, 10000]),
            Some(&[7200, 7200, 7200]),
        )
        .expect("표 생성");
    let created: Value = serde_json::from_str(&created).expect("createTable JSON");
    let ppi = created["paraIdx"].as_u64().expect("paraIdx") as u32;
    let ci = created["controlIdx"].as_u64().expect("controlIdx") as u32;
    let props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":2,
        "diagonalBackSlash":2,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"NONE"
    }"##;

    doc.set_cell_zone_properties(0, ppi, ci, 0, 0, 1, 1, props)
        .expect("cellzone 적용");

    let table = first_table(doc.document());
    let zone = table.zones.last().expect("asOne cellzone");
    assert_eq!(
        (zone.start_row, zone.start_col, zone.end_row, zone.end_col),
        (0, 0, 1, 1)
    );

    let cell_props = doc
        .get_cell_properties(0, ppi, ci, 4)
        .expect("zone 내부 셀 속성 조회");
    let cell_props: Value = serde_json::from_str(&cell_props).expect("cell props JSON");
    assert_eq!(cell_props["diagonalLine"].as_u64(), Some(1));
    assert_eq!(cell_props["diagonalSlash"].as_u64(), Some(2));
    assert_eq!(cell_props["diagonalBackSlash"].as_u64(), Some(2));
    assert_eq!(cell_props["centerLine"].as_str(), Some("NONE"));

    let svg = doc.render_page_svg_native(0).expect("SVG 렌더");
    let lines = rendered_lines(&svg);
    assert!(
        lines
            .iter()
            .any(|(x1, y1, x2, y2)| (x1 - x2).abs() > 200.0 && (y1 - y2).abs() > 150.0),
        "asOne 대각선은 첫 셀이 아니라 선택 영역 전체 cellzone bbox를 가로질러야 함"
    );
}

#[test]
fn issue_1633_as_one_cellzone_survives_hwp_export() {
    let mut doc = HwpDocument::create_empty();
    let created = doc
        .create_table_ex_native(0, 0, 0, 8, 10, true, Some(&[4195; 10]), Some(&[1282; 8]))
        .expect("표 생성");
    let created: Value = serde_json::from_str(&created).expect("createTable JSON");
    let ppi = created["paraIdx"].as_u64().expect("paraIdx") as u32;
    let ci = created["controlIdx"].as_u64().expect("controlIdx") as u32;
    let props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":2,
        "diagonalBackSlash":2,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"NONE"
    }"##;

    doc.set_cell_zone_properties(0, ppi, ci, 0, 0, 7, 9, props)
        .expect("cellzone 적용");
    let exported = doc.export_hwp_with_adapter().expect("HWP export");
    let reparsed = parse_document(&exported).expect("exported HWP 재파싱");
    let table = first_table(&reparsed);
    let zone = table.zones.first().expect("HWP TABLE record cellzone");

    assert_eq!(table.row_count, 8);
    assert_eq!(table.col_count, 10);
    assert_eq!(
        (zone.start_row, zone.start_col, zone.end_row, zone.end_col),
        (0, 0, 7, 9),
        "다른 이름 저장 HWP도 10열 8줄 전체 cellzone을 보존해야 함"
    );
    let bf = &reparsed.doc_info.border_fills[(zone.border_fill_id - 1) as usize];
    assert_eq!(bf.diagonal.diagonal_type, 1);
    assert_eq!((bf.attr >> 2) & 0x07, 2);
    assert_eq!((bf.attr >> 5) & 0x07, 2);
}

#[test]
fn issue_1633_centerline_is_disabled_for_as_one_cellzone_apply() {
    let mut doc = HwpDocument::create_empty();
    let created = doc
        .create_table_ex_native(0, 0, 0, 8, 10, true, Some(&[4195; 10]), Some(&[1282; 8]))
        .expect("표 생성");
    let created: Value = serde_json::from_str(&created).expect("createTable JSON");
    let ppi = created["paraIdx"].as_u64().expect("paraIdx") as u32;
    let ci = created["controlIdx"].as_u64().expect("controlIdx") as u32;
    let diagonal_props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":2,
        "diagonalBackSlash":2,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"NONE"
    }"##;
    let centerline_props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":0,
        "diagonalBackSlash":0,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"CROSS"
    }"##;

    doc.set_cell_zone_properties(0, ppi, ci, 0, 0, 7, 9, diagonal_props)
        .expect("대각선 cellzone 적용");
    doc.set_cell_zone_properties(0, ppi, ci, 0, 0, 7, 9, centerline_props)
        .expect("중심선 cellzone 입력 무시");

    let props = doc
        .get_cell_properties(0, ppi, ci, 0)
        .expect("zone 내부 셀 속성 조회");
    let props: Value = serde_json::from_str(&props).expect("cell props JSON");
    assert_eq!(props["centerLine"].as_str(), Some("NONE"));
    assert_eq!(props["diagonalSlash"].as_u64(), Some(0));
    assert_eq!(props["diagonalBackSlash"].as_u64(), Some(0));

    let svg = doc.render_page_svg_native(0).expect("SVG 렌더");
    let lines = rendered_lines(&svg);
    assert!(
        !lines
            .iter()
            .any(|(x1, y1, x2, y2)| (x1 - x2).abs() > 500.0 && (y1 - y2).abs() > 100.0),
        "중심선이 비활성화된 cellzone 입력 후 기존 전체 X 대각선이 남아 있으면 안 됨"
    );
}

#[test]
fn issue_1633_cellzone_origin_centerline_renders_after_each_cell_apply() {
    let mut doc = HwpDocument::create_empty();
    let created = doc
        .create_table_ex_native(0, 0, 0, 8, 10, true, Some(&[4195; 10]), Some(&[1282; 8]))
        .expect("표 생성");
    let created: Value = serde_json::from_str(&created).expect("createTable JSON");
    let ppi = created["paraIdx"].as_u64().expect("paraIdx") as u32;
    let ci = created["controlIdx"].as_u64().expect("controlIdx") as u32;
    let diagonal_props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":2,
        "diagonalBackSlash":2,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"NONE"
    }"##;
    let centerline_props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":0,
        "diagonalBackSlash":0,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"CROSS"
    }"##;

    doc.set_cell_zone_properties(0, ppi, ci, 0, 0, 7, 9, diagonal_props)
        .expect("대각선 cellzone 적용");

    doc.set_cell_properties(0, ppi, ci, 0, centerline_props)
        .expect("선택 셀 중심선 적용");

    let own_props = doc
        .get_cell_own_properties(0, ppi, ci, 0)
        .expect("고유 셀 속성 조회");
    let own_props: Value = serde_json::from_str(&own_props).expect("own props JSON");
    assert_eq!(own_props["centerLine"].as_str(), Some("CROSS"));
    assert_eq!(own_props["diagonalSlash"].as_u64(), Some(0));
    assert_eq!(own_props["diagonalBackSlash"].as_u64(), Some(0));

    let table = first_table(doc.document());
    assert!(
        !table.zones.iter().any(|zone| {
            (zone.start_row, zone.start_col, zone.end_row, zone.end_col) == (0, 0, 0, 0)
        }),
        "중심선은 한컴 저장 호환성을 위해 1x1 cellzone이 아니라 셀 BF로 보존해야 함"
    );

    let after_svg = doc.render_page_svg_native(0).expect("SVG 렌더");
    let after_lines = rendered_lines(&after_svg);
    assert!(
        after_lines.iter().any(|(x1, y1, x2, y2)| {
            (x1 - x2).abs() < 0.01
                && (130.0..155.0).contains(x1)
                && (10.0..25.0).contains(&(y1 - y2).abs())
        }),
        "전체 cellzone 대각선 시작 셀에 나중에 적용한 세로 중심선이 렌더되어야 함"
    );
    assert!(
        after_lines.iter().any(|(x1, y1, x2, y2)| {
            (y1 - y2).abs() < 0.01
                && (140.0..150.0).contains(y1)
                && (40.0..80.0).contains(&(x1 - x2).abs())
        }),
        "전체 cellzone 대각선 시작 셀에 나중에 적용한 가로 중심선이 렌더되어야 함"
    );

    let exported = doc.export_hwp_with_adapter().expect("HWP export");
    let reparsed = parse_document(&exported).expect("exported HWP 재파싱");
    let reparsed_table = first_table(&reparsed);
    assert!(
        !reparsed_table.zones.iter().any(|zone| {
            (zone.start_row, zone.start_col, zone.end_row, zone.end_col) == (0, 0, 0, 0)
        }),
        "HWP 저장본도 대각선샘플3처럼 1x1 중심선 cellzone을 만들지 않아야 함"
    );
    let reparsed_cell = reparsed_table.cells.first().expect("첫 셀");
    let reparsed_bf = &reparsed.doc_info.border_fills[(reparsed_cell.border_fill_id - 1) as usize];
    assert_eq!(reparsed_bf.center_line, CenterLine::Cross);
    assert_eq!(reparsed_cell.list_header_width_ref, 0x0400);
    assert_eq!(reparsed_cell.raw_list_extra.len(), 13);
    assert_eq!(
        u32::from_le_bytes(reparsed_cell.raw_list_extra[0..4].try_into().unwrap()),
        reparsed_cell.width,
        "한컴 저장본 셀 LIST_HEADER처럼 raw 확장 폭 참조를 보강해야 함"
    );
}

#[test]
fn issue_1633_cell_diagonal_renders_over_existing_cellzone_diagonal() {
    let mut doc = HwpDocument::create_empty();
    let created = doc
        .create_table_ex_native(0, 0, 0, 8, 10, true, Some(&[4195; 10]), Some(&[1282; 8]))
        .expect("표 생성");
    let created: Value = serde_json::from_str(&created).expect("createTable JSON");
    let ppi = created["paraIdx"].as_u64().expect("paraIdx") as u32;
    let ci = created["controlIdx"].as_u64().expect("controlIdx") as u32;
    let diagonal_props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":2,
        "diagonalBackSlash":2,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"NONE"
    }"##;

    doc.set_cell_zone_properties(0, ppi, ci, 0, 0, 7, 9, diagonal_props)
        .expect("대각선 cellzone 적용");
    doc.set_cell_properties(0, ppi, ci, 1, diagonal_props)
        .expect("1행 2열 선택 셀 대각선 적용");

    let own_props = doc
        .get_cell_own_properties(0, ppi, ci, 1)
        .expect("고유 셀 속성 조회");
    let own_props: Value = serde_json::from_str(&own_props).expect("own props JSON");
    assert_eq!(own_props["diagonalSlash"].as_u64(), Some(2));
    assert_eq!(own_props["diagonalBackSlash"].as_u64(), Some(2));

    let svg = doc.render_page_svg_native(0).expect("SVG 렌더");
    let lines = rendered_lines(&svg);
    let short_cell_diagonals: Vec<_> = lines
        .iter()
        .filter(|(x1, y1, x2, y2)| {
            let dx = (x1 - x2).abs();
            let dy = (y1 - y2).abs();
            (40.0..80.0).contains(&dx) && (10.0..30.0).contains(&dy)
        })
        .collect();
    assert!(
        short_cell_diagonals.len() >= 2,
        "기존 cellzone 대각선 위에 나중에 적용한 1행 2열 셀 대각선이 렌더되어야 함: {short_cell_diagonals:?}"
    );
}

#[test]
fn issue_1633_cellzone_origin_cell_diagonal_renders_after_each_cell_apply() {
    let mut doc = HwpDocument::create_empty();
    let created = doc
        .create_table_ex_native(0, 0, 0, 8, 10, true, Some(&[4195; 10]), Some(&[1282; 8]))
        .expect("표 생성");
    let created: Value = serde_json::from_str(&created).expect("createTable JSON");
    let ppi = created["paraIdx"].as_u64().expect("paraIdx") as u32;
    let ci = created["controlIdx"].as_u64().expect("controlIdx") as u32;
    let diagonal_props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":2,
        "diagonalBackSlash":2,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"NONE"
    }"##;

    doc.set_cell_zone_properties(0, ppi, ci, 0, 0, 7, 9, diagonal_props)
        .expect("대각선 cellzone 적용");
    doc.set_cell_properties(0, ppi, ci, 0, diagonal_props)
        .expect("시작 셀 대각선 적용");

    let table = first_table(doc.document());
    assert!(
        table.zones.iter().any(|zone| {
            (zone.start_row, zone.start_col, zone.end_row, zone.end_col) == (0, 0, 0, 0)
        }),
        "cellzone 시작 셀에 명시적으로 적용한 대각선은 1x1 override zone으로 분리되어야 함"
    );

    let svg = doc.render_page_svg_native(0).expect("SVG 렌더");
    let lines = rendered_lines(&svg);
    let short_cell_diagonals: Vec<_> = lines
        .iter()
        .filter(|(x1, y1, x2, y2)| {
            let dx = (x1 - x2).abs();
            let dy = (y1 - y2).abs();
            (40.0..80.0).contains(&dx) && (10.0..30.0).contains(&dy)
        })
        .collect();
    assert!(
        short_cell_diagonals.len() >= 2,
        "cellzone 시작 셀에 나중에 적용한 개별 셀 대각선이 렌더되어야 함: {short_cell_diagonals:?}"
    );
}

#[test]
fn issue_1633_cell_edit_does_not_store_cellzone_borderfill_as_cell_borderfill() {
    let bytes = read_sample("samples/대각선샘플4.hwp");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("대각선샘플4 HWP 파싱");
    let effective_props = doc
        .get_cell_properties(0, 0, 2, 1)
        .expect("1행 2열 effective 속성 조회");
    let effective_props: Value =
        serde_json::from_str(&effective_props).expect("effective props JSON");
    let zone_bf_id = effective_props["borderFillId"]
        .as_u64()
        .expect("zone borderFillId") as u16;

    doc.set_cell_properties(0, 0, 2, 1, &effective_props.to_string())
        .expect("cellzone effective props 기반 셀 편집");

    let table = first_table(doc.document());
    let zone = table
        .zones
        .iter()
        .find(|zone| zone.start_row == 0 && zone.start_col == 0)
        .expect("전체 cellzone");
    let cell = table.cells.get(1).expect("1행 2열 셀");
    assert_eq!(zone.border_fill_id, zone_bf_id);
    assert_ne!(
        cell.border_fill_id, zone.border_fill_id,
        "각 셀마다 적용은 cellzone BF를 셀 BF로 저장하면 안 됨"
    );

    let zone_bf = &doc.document().doc_info.border_fills[(zone.border_fill_id - 1) as usize];
    let cell_bf = &doc.document().doc_info.border_fills[(cell.border_fill_id - 1) as usize];
    assert_eq!(
        zone_bf.borders[0].line_type as u8, 0,
        "zone BF는 셀 테두리 없음"
    );
    assert_ne!(
        cell_bf.borders[0].line_type as u8, 0,
        "셀 BF는 기존 셀 테두리를 보존해야 함"
    );
    assert_eq!((cell_bf.attr >> 2) & 0x07, 2);
    assert_eq!((cell_bf.attr >> 5) & 0x07, 2);
}

#[test]
fn issue_1633_all_cells_centerline_suppresses_old_cellzone_diagonal_render() {
    let mut doc = HwpDocument::create_empty();
    let created = doc
        .create_table_ex_native(0, 0, 0, 8, 10, true, Some(&[4195; 10]), Some(&[1282; 8]))
        .expect("표 생성");
    let created: Value = serde_json::from_str(&created).expect("createTable JSON");
    let ppi = created["paraIdx"].as_u64().expect("paraIdx") as u32;
    let ci = created["controlIdx"].as_u64().expect("controlIdx") as u32;
    let diagonal_props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":2,
        "diagonalBackSlash":2,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"NONE"
    }"##;
    let centerline_props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":0,
        "diagonalBackSlash":0,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"CROSS"
    }"##;

    doc.set_cell_zone_properties(0, ppi, ci, 0, 0, 7, 9, diagonal_props)
        .expect("대각선 cellzone 적용");
    for cell_idx in 0..80 {
        doc.set_cell_properties(0, ppi, ci, cell_idx, centerline_props)
            .unwrap_or_else(|err| panic!("cell {cell_idx} 중심선 적용 실패: {err:?}"));
    }

    let svg = doc.render_page_svg_native(0).expect("SVG 렌더");
    let lines = rendered_lines(&svg);
    assert!(
        !lines
            .iter()
            .any(|(x1, y1, x2, y2)| (x1 - x2).abs() > 500.0 && (y1 - y2).abs() > 100.0),
        "모든 셀이 중심선으로 덮인 뒤에는 기존 전체 cellzone X가 렌더되면 안 됨"
    );
    assert!(
        lines
            .iter()
            .filter(|(x1, y1, x2, y2)| (x1 - x2).abs() < 0.01
                && (10.0..30.0).contains(&(y1 - y2).abs()))
            .count()
            >= 10,
        "각 셀 중심선의 세로 선분이 렌더되어야 함"
    );
}

#[test]
fn issue_1633_centerline_excludes_diagonal_on_hwp_export() {
    let mut doc = HwpDocument::create_empty();
    let created = doc
        .create_table_ex_native(0, 0, 0, 8, 10, true, Some(&[4195; 10]), Some(&[1282; 8]))
        .expect("표 생성");
    let created: Value = serde_json::from_str(&created).expect("createTable JSON");
    let ppi = created["paraIdx"].as_u64().expect("paraIdx") as u32;
    let ci = created["controlIdx"].as_u64().expect("controlIdx") as u32;
    let props = r##"{
        "borderFillId":1,
        "borderLeft":{"type":1,"width":0,"color":"#000000"},
        "borderRight":{"type":1,"width":0,"color":"#000000"},
        "borderTop":{"type":1,"width":0,"color":"#000000"},
        "borderBottom":{"type":1,"width":0,"color":"#000000"},
        "fillType":"none",
        "diagonalLine":1,
        "diagonalSlash":2,
        "diagonalBackSlash":2,
        "diagonalWidth":0,
        "diagonalColor":"#000000",
        "centerLine":"CROSS"
    }"##;

    doc.set_cell_properties(0, ppi, ci, 0, props)
        .expect("선택 셀 중심선 적용");
    let cell_props = doc
        .get_cell_own_properties(0, ppi, ci, 0)
        .expect("고유 셀 속성 조회");
    let cell_props: Value = serde_json::from_str(&cell_props).expect("cell props JSON");
    assert_eq!(cell_props["centerLine"].as_str(), Some("CROSS"));
    assert_eq!(
        cell_props["diagonalSlash"].as_u64(),
        Some(0),
        "중심선을 선택하면 대각선 방향은 한컴처럼 해제되어야 함"
    );
    assert_eq!(
        cell_props["diagonalBackSlash"].as_u64(),
        Some(0),
        "중심선을 선택하면 역대각선 방향은 한컴처럼 해제되어야 함"
    );

    let exported = doc.export_hwp_with_adapter().expect("HWP export");
    let reparsed = parse_document(&exported).expect("exported HWP 재파싱");
    let table = first_table(&reparsed);
    let cell = table.cells.first().expect("HWP TABLE 첫 셀");
    let bf = &reparsed.doc_info.border_fills[(cell.border_fill_id - 1) as usize];

    assert_ne!(bf.attr & (1 << 13), 0, "HWP 중심선 bit 13 보존");
    assert_eq!(
        (bf.attr >> 8) & 0x03,
        3,
        "CROSS 중심선은 한컴식 slash 중심선 보조 비트를 저장해야 함"
    );
    assert_eq!(
        bf.attr & (1 << 10),
        1 << 10,
        "CROSS 중심선은 한컴식 backSlash 중심선 보조 비트를 저장해야 함"
    );
    assert_eq!(bf.center_line, CenterLine::Cross);
    assert_eq!(bf.diagonal.diagonal_type, 1);
    assert_eq!(
        (bf.attr >> 2) & 0x07,
        0,
        "HWP 저장본도 중심선과 대각선 방향을 동시에 저장하지 않음"
    );
    assert_eq!(
        (bf.attr >> 5) & 0x07,
        0,
        "HWP 저장본도 중심선과 역대각선 방향을 동시에 저장하지 않음"
    );
    assert_eq!(cell.list_header_width_ref, 0x0400);
    assert_eq!(cell.raw_list_extra.len(), 13);
    assert_eq!(
        u32::from_le_bytes(cell.raw_list_extra[0..4].try_into().unwrap()),
        cell.width,
        "새로 만든 표 셀도 한컴식 47바이트 LIST_HEADER로 저장해야 함"
    );
}
