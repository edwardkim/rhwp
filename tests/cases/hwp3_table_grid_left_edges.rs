//! 고정형·관계형: HWP3 표의 독립 한컴 격자와 셀 내용·소속 보존.
//! 주석 갱신: 2026-11-09T18:16:58+09:00. 직전 파일 변경: 383f66acca8c966da9a198fd046c3da0ed94c3b6 2026-11-08T23:47:32+09:00.
//! 시각 측정: 2026-11-09 KST, 독립 한컴 Print PDF, 96 DPI, print profile, compat 2022.
//! 색상 경계 보정 전 2px silhouette_raw_match_percent: sample16 전체 64쪽의
//! Native Skia 최저 99.57001%, fresh WASM Canvas2D 최저 99.62920%.
//! table-cell-order 전체 1쪽: Native 100.00000%, fresh WASM 100.00000%.
//! 전쪽 TSV·producer·직접 판독: mydocs/pr/assets/chrisryugj-20261109/README.md.
//! 행열·셀 주소·병합 기대값은 같은 문서의 독립 한컴 변환 XML이며 px 배치를 고정하지 않는다.
//! 대형 합성 입력은 HWP3 10.6의 84-byte 정보·word 셀 개수·27-byte 셀 구조를 사용한다.
//! 합성 검사는 parser 내용·소속 계약이며 한컴 Print 증거나 실제 페이지 배치를 주장하지 않는다.

use std::collections::BTreeMap;
use std::io::Read;

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::Table;

type CellGrid = BTreeMap<(u16, u16), (u16, u16)>;

fn walk_tables(paragraphs: &[Paragraph], out: &mut Vec<Table>) {
    for p in paragraphs {
        for c in &p.controls {
            if let Control::Table(t) = c {
                out.push(t.as_ref().clone());
                for cell in &t.cells {
                    walk_tables(&cell.paragraphs, out);
                }
            }
        }
    }
}

fn tables(name: &str) -> Vec<Table> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    let raw = std::fs::read(path).expect("read sample");
    parsed_tables(&raw)
}

fn parsed_tables(raw: &[u8]) -> Vec<Table> {
    let doc = rhwp::parser::hwp3::parse_hwp3(raw).expect("HWP3 파싱");
    let mut out = Vec::new();
    for section in &doc.sections {
        walk_tables(&section.paragraphs, &mut out);
    }
    out
}

fn cell_text(t: &Table) -> String {
    t.cells
        .iter()
        .flat_map(|c| c.paragraphs.iter().map(|p| p.text.as_str()))
        .collect::<Vec<_>>()
        .join("|")
}

fn table_of<'a>(all: &'a [Table], key: &str) -> &'a Table {
    let found: Vec<_> = all.iter().filter(|t| cell_text(t).contains(key)).collect();
    assert_eq!(found.len(), 1, "'{key}' 를 담은 표가 하나여야 한다");
    found[0]
}

#[test]
fn sample16_tables_match_the_hancom_grid() {
    let all = tables("hwp3-sample16.hwp");
    for (key, rows, cols) in [
        ("접 수 번 호", 15, 5),
        ("사업책임자(주계약자)", 23, 17),
        ("투입인력 개인별 경력사항 확인서", 15, 17),
    ] {
        let table = table_of(&all, key);
        assert_eq!((table.row_count, table.col_count), (rows, cols));
    }
}

#[test]
fn cell_order_table_matches_the_hancom_grid() {
    let all = tables("hwp3-table-cell-order.hwp");
    let t = all
        .iter()
        .max_by_key(|t| u32::from(t.row_count) * u32::from(t.col_count))
        .expect("표");
    assert_eq!((t.row_count, t.col_count), (29, 12));
}

fn hancom_cell_grid(reference: &str, key: &str) -> CellGrid {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(reference);
    let mut zip =
        zip::ZipArchive::new(std::fs::File::open(path).expect("독립 변환본")).expect("HWPX ZIP");
    let mut found = Vec::new();
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).expect("ZIP entry");
        let name = file.name().expect("ZIP entry name").into_owned();
        if !name.starts_with("Contents/section") || !name.ends_with(".xml") {
            continue;
        }
        let mut xml = String::new();
        file.read_to_string(&mut xml).expect("section XML");
        let doc = roxmltree::Document::parse(&xml).expect("독립 XML");
        for table in doc.descendants().filter(|n| n.has_tag_name("tbl")) {
            let text = table
                .descendants()
                .filter(|n| n.has_tag_name("t"))
                .filter_map(|n| n.text())
                .collect::<String>();
            if !text.contains(key) {
                continue;
            }
            let mut cells = CellGrid::new();
            for cell in table
                .children()
                .filter(|n| n.has_tag_name("tr"))
                .flat_map(|n| n.children())
                .filter(|n| n.has_tag_name("tc"))
            {
                let addr = cell
                    .children()
                    .find(|n| n.has_tag_name("cellAddr"))
                    .expect("cellAddr");
                let span = cell
                    .children()
                    .find(|n| n.has_tag_name("cellSpan"))
                    .expect("cellSpan");
                let value = |node: roxmltree::Node<'_, '_>, name: &str| {
                    node.attribute(name)
                        .expect("attribute")
                        .parse::<u16>()
                        .expect("u16")
                };
                assert!(cells
                    .insert(
                        (value(addr, "rowAddr"), value(addr, "colAddr")),
                        (value(span, "rowSpan"), value(span, "colSpan"))
                    )
                    .is_none());
            }
            found.push(cells);
        }
    }
    assert_eq!(found.len(), 1, "독립 원본의 대상 표");
    found.pop().expect("격자")
}

#[test]
fn real_tables_keep_hancom_cell_addresses_and_spans() {
    for (sample, reference, keys) in [
        (
            "hwp3-sample16.hwp",
            "samples/hwp3-sample16-hwp5.hwpx",
            &[
                "접 수 번 호",
                "사업책임자(주계약자)",
                "투입인력 개인별 경력사항 확인서",
            ][..],
        ),
        (
            "hwp3-table-cell-order.hwp",
            "pdf/issue7642-hwp3-realign-controls-20261012/hwp3-table-cell-order-hancom-2020.hwpx",
            &["민사신청사건기록"][..],
        ),
    ] {
        let all = tables(sample);
        for key in keys {
            let t = table_of(&all, key);
            let actual: CellGrid = t
                .cells
                .iter()
                .map(|c| ((c.row, c.col), (c.row_span, c.col_span)))
                .collect();
            assert_eq!(actual.len(), t.cells.len(), "셀 원점 중복: {key}");
            assert_eq!(
                actual,
                hancom_cell_grid(reference, key),
                "셀 주소·병합: {key}"
            );
        }
    }
}

fn word(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn paragraph_header(chars: u16) -> Vec<u8> {
    let mut result = vec![0; 230];
    word(&mut result, 1, chars);
    word(&mut result, 3, 1);
    word(&mut result, 12, 250);
    result[21..28].fill(100);
    word(&mut result, 49, 160);
    result[215] = 1;
    result
}
fn line_info(y: u16) -> Vec<u8> {
    [0, 0, 400, y, 0, 0, 0]
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect()
}

/// HWP3 공식 10.6: (행,열) byte는 sheet serial이고 위치·크기는 별도 word다.
/// 좌표가 u16 안에 있는 2×N 또는 N×2 입력이다. source 마커 하나를 셀 하나에 넣는다.
fn dense_grid(length: u16, transpose: bool) -> Vec<u8> {
    let mut fonts = Vec::new();
    for _ in 0..7 {
        fonts.extend_from_slice(&1u16.to_le_bytes());
        let mut name = [0; 40];
        name[..4].copy_from_slice(&[0xb9, 0xd9, 0xc5, 0xc1]);
        fonts.extend_from_slice(&name);
    }
    fonts.extend_from_slice(&0u16.to_le_bytes());
    let mut info = [0; 84];
    word(&mut info, 16, 10);
    word(&mut info, 42, if transpose { length * 20 } else { 2000 });
    word(&mut info, 44, if transpose { 2000 } else { length * 20 });
    word(&mut info, 80, length * 2);
    let mut cells = Vec::new();
    let mut paragraphs = Vec::new();
    for row in 0..length {
        for col in 0..2u16 {
            let mut cell = [0; 27];
            cell[0] = (if transpose { col } else { row }) as u8;
            cell[1] = (if transpose { row } else { col }) as u8;
            let (x, y, width, height) = if transpose {
                (row * 20, col * 1000, 20, 1000)
            } else {
                (col * 1000, row * 20, 1000, 20)
            };
            for (offset, value) in [(4, x), (6, y), (8, width), (10, height)] {
                word(&mut cell, offset, value);
            }
            cell[18] = 1;
            cells.extend_from_slice(&cell);
            let text = format!("R{row:04}C{col}");
            paragraphs.extend_from_slice(&paragraph_header(text.len() as u16 + 1));
            paragraphs.extend_from_slice(&line_info(0));
            for byte in text.bytes() {
                paragraphs.extend_from_slice(&u16::from(byte).to_le_bytes());
            }
            paragraphs.extend_from_slice(&13u16.to_le_bytes());
            paragraphs.extend_from_slice(&[0; 43]);
        }
    }
    let mut outer = paragraph_header(5);
    outer.extend_from_slice(&line_info(100));
    outer.extend_from_slice(&10u16.to_le_bytes());
    outer.extend_from_slice(&0u32.to_le_bytes());
    outer.extend_from_slice(&10u16.to_le_bytes());
    outer.extend_from_slice(&info);
    outer.extend_from_slice(&cells);
    outer.extend_from_slice(&paragraphs);
    outer.extend_from_slice(&[0; 43]);
    outer.extend_from_slice(&13u16.to_le_bytes());
    let mut raw = b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05".to_vec();
    raw.extend_from_slice(&[0; 128]);
    raw.extend_from_slice(&[0; 1008]);
    raw.extend_from_slice(&fonts);
    raw.extend_from_slice(&outer);
    raw.extend_from_slice(&[0; 43]);
    raw
}

fn assert_dense_grid(length: u16, transpose: bool) {
    let all = parsed_tables(&dense_grid(length, transpose));
    assert_eq!(all.len(), 1);
    let t = &all[0];
    let expected_dimensions = if transpose { (2, length) } else { (length, 2) };
    assert_eq!((t.row_count, t.col_count), expected_dimensions);
    assert_eq!(t.cells.len(), usize::from(length) * 2);
    let mut seen = BTreeMap::new();
    for c in &t.cells {
        let text = c
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<String>();
        assert!(
            seen.insert(text, (c.row, c.col, c.row_span, c.col_span))
                .is_none(),
            "내용 중복"
        );
    }
    for row in 0..length {
        for col in 0..2u16 {
            let (r, c) = if transpose { (col, row) } else { (row, col) };
            assert_eq!(
                seen.get(&format!("R{row:04}C{col}")),
                Some(&(r, c, 1, 1)),
                "내용 누락·소속·스팬: {row},{col}"
            );
        }
    }
}

#[test]
fn row_neighbors_beyond_the_old_scan_limit_keep_every_cell() {
    for rows in [2, 2048, 2049, 2050] {
        assert_dense_grid(rows, false);
    }
}

#[test]
fn column_neighbors_beyond_the_old_scan_limit_keep_every_cell() {
    for columns in [2, 2048, 2049, 2050] {
        assert_dense_grid(columns, true);
    }
}
