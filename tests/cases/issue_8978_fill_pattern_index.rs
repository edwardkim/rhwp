//! 관계형·고정형: 칸 배경 무늬의 위→아래 순서(관계), 한컴 출력·저장에서 정한 무늬 번호와 `hatchStyle` 목록(고정).
//! 주석 갱신: 2026-11-12T14:49:09+09:00.
//! 2026-11-12 측정 raw(96dpi, pypdfium2): fill-patterns-0-6.hwp 1쪽 Native 99.97% / fresh WASM 99.99%,
//! fill-patterns-0-6.hancom2024.hwpx 1쪽 Native 99.97% / fresh WASM 99.99% (각 전1쪽);
//! hwpspec.hwp 23쪽 Native 99.98762% / fresh WASM 99.99085%, 전178쪽 최저 97.08% / 97.08%.
//! 입력·기준: `samples/issue8978/fill-patterns-0-6.hwp`와 한글 2024 Print `pdf/issue8978/fill-patterns-0-6-2024.pdf`,
//! 같은 한글이 저장한 `samples/issue8978/fill-patterns-0-6.hancom2024.hwpx`, `samples/hwpspec.hwp`와 `pdf/hwpspec-2024.pdf`.
//! 근거: mydocs/pr/assets/issue_8978/README.md.
#![cfg(not(target_arch = "wasm32"))]
//! HWP 채우기 무늬 종류는 0~5다(#8978). 0 ─, 1 │, 2 ＼, 3 ／, 4 ＋, 5 ×이고 -1과 6은 무늬가 없다.
//! HWPX `hatchStyle`은 한글 저장 결과를 따라 2 = `SLASH`, 3 = `BACK_SLASH`다.
use std::io::Read;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const HWP: &str = "samples/issue8978/fill-patterns-0-6.hwp";
const HANCOM_HWPX: &str = "samples/issue8978/fill-patterns-0-6.hancom2024.hwpx";

fn read(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Hatch {
    Horizontal,
    Vertical,
    Falling,
    Rising,
    Cross,
    DiagonalCross,
}

/// 한글 2024 Print에서 위 행부터 보이는 무늬. 7행(무늬 6)은 무늬가 없다.
const PRINT_ORDER: [Hatch; 6] = [
    Hatch::Horizontal,
    Hatch::Vertical,
    Hatch::Falling,
    Hatch::Rising,
    Hatch::Cross,
    Hatch::DiagonalCross,
];

/// SVG `<pattern>` 정의의 선 모양.
fn hatch_of(def: &str) -> Hatch {
    let paths: Vec<&str> = def
        .split("<path d=\"")
        .skip(1)
        .map(|rest| &rest[..rest.find('"').unwrap()])
        .collect();
    let horizontal = paths.iter().any(|d| d.starts_with("M0 0H"));
    let vertical = paths.iter().any(|d| d.starts_with("M0 0V"));
    let falling = paths.iter().any(|d| d.starts_with("M0 0L"));
    let rising = paths
        .iter()
        .any(|d| !d.starts_with("M0 0") && d.contains("L0 "));
    match (horizontal, vertical, falling, rising) {
        (true, false, false, false) => Hatch::Horizontal,
        (false, true, false, false) => Hatch::Vertical,
        (false, false, true, false) => Hatch::Falling,
        (false, false, false, true) => Hatch::Rising,
        (true, true, false, false) => Hatch::Cross,
        (false, false, true, true) => Hatch::DiagonalCross,
        other => panic!("알 수 없는 무늬 {other:?}: {def}"),
    }
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

/// 채운 도형마다 `(x, y, 무늬)`. 무늬가 아닌 채우기는 `None`이다.
fn filled_shapes(svg: &str) -> Vec<(String, f64, Option<Hatch>)> {
    let mut defs = std::collections::HashMap::new();
    for chunk in svg.split("<pattern ").skip(1) {
        let def = &chunk[..chunk.find("</pattern>").unwrap()];
        let id = def
            .strip_prefix("id=\"")
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        defs.insert(id.to_string(), hatch_of(def));
    }
    svg.split('<')
        .filter(|tag| tag.starts_with("rect ") || tag.starts_with("path "))
        .filter_map(|tag| {
            let fill = attr(tag, "fill").filter(|f| *f != "none")?;
            let hatch = fill
                .strip_prefix("url(#")
                .and_then(|id| id.strip_suffix(')'))
                .and_then(|id| defs.get(id).copied());
            let (x, y) = match (attr(tag, "x"), attr(tag, "y")) {
                (Some(x), Some(y)) => (x.to_string(), y.parse().unwrap()),
                _ => (String::new(), first_y_of_path(attr(tag, "d")?)),
            };
            Some((x, y, hatch))
        })
        .collect()
}

/// 무늬로 칠한 도형의 무늬를 위에서 아래 순서로 돌려준다.
fn hatches_top_to_bottom(svg: &str) -> Vec<Hatch> {
    let mut filled: Vec<(f64, Hatch)> = filled_shapes(svg)
        .into_iter()
        .filter_map(|(_, y, hatch)| Some((y, hatch?)))
        .collect();
    filled.sort_by(|a, b| a.0.total_cmp(&b.0));
    filled.into_iter().map(|(_, hatch)| hatch).collect()
}

fn first_y_of_path(d: &str) -> f64 {
    let mut numbers = d
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .filter(|s| !s.is_empty());
    numbers.next().unwrap();
    numbers.next().unwrap().parse().unwrap()
}

fn first_table(document: &Document) -> &Table {
    document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .expect("표")
}

/// 오른쪽 칸의 무늬 종류를 위 행부터 돌려준다.
fn right_column_pattern_types(document: &Document) -> Vec<i32> {
    let table = first_table(document);
    let mut cells: Vec<_> = table.cells.iter().filter(|c| c.col == 1).collect();
    cells.sort_by_key(|c| c.row);
    cells
        .iter()
        .map(|c| {
            let fill = &document.doc_info.border_fills[c.border_fill_id as usize - 1].fill;
            fill.solid.map_or(-1, |s| s.pattern_type)
        })
        .collect()
}

/// 오른쪽 칸 배경의 무늬를 위 행부터 돌려준다. 무늬 없는 단색 칸은 `None`이다.
fn right_column_hatches(svg: &str) -> Vec<Option<Hatch>> {
    let shapes = filled_shapes(svg);
    let column = shapes
        .iter()
        .find(|(_, _, hatch)| hatch.is_some())
        .map(|(x, _, _)| x.clone())
        .expect("무늬 칸");
    let mut cells: Vec<_> = shapes
        .into_iter()
        .filter(|(x, _, _)| *x == column)
        .collect();
    cells.sort_by(|a, b| a.1.total_cmp(&b.1));
    cells.into_iter().map(|(_, _, hatch)| hatch).collect()
}

#[test]
fn hwp_pattern_types_draw_in_hancom_print_order() {
    let core = DocumentCore::from_bytes(&read(HWP)).unwrap();
    let svg = core.render_page_svg_native(0).unwrap();
    let mut expected: Vec<_> = PRINT_ORDER.iter().copied().map(Some).collect();
    expected.push(None);
    assert_eq!(right_column_hatches(&svg), expected);
}

#[test]
fn hancom_hwpx_hatch_styles_read_as_the_same_pattern_types() {
    let hwp = rhwp::parser::parse_document(&read(HWP)).unwrap();
    let hwpx = rhwp::parser::parse_document(&read(HANCOM_HWPX)).unwrap();
    assert_eq!(right_column_pattern_types(&hwp), [0, 1, 2, 3, 4, 5, 6]);
    // 한글은 무늬 6을 HWPX에 쓰지 않는다.
    assert_eq!(right_column_pattern_types(&hwpx), [0, 1, 2, 3, 4, 5, -1]);

    let core = DocumentCore::from_bytes(&read(HANCOM_HWPX)).unwrap();
    let svg = core.render_page_svg_native(0).unwrap();
    let mut expected: Vec<_> = PRINT_ORDER.iter().copied().map(Some).collect();
    expected.push(None);
    assert_eq!(right_column_hatches(&svg), expected);
}

/// HWPX의 `borderFillIDRef`를 따라 오른쪽 칸의 `hatchStyle`을 위 행부터 돌려준다.
fn right_column_hatch_styles(hwpx: &[u8]) -> Vec<Option<String>> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).unwrap();
    let mut text = |name: &str| {
        let mut out = String::new();
        zip.by_name(name).unwrap().read_to_string(&mut out).unwrap();
        out
    };
    let header = text("Contents/header.xml");
    let section = text("Contents/section0.xml");
    let hatch_of_fill = |id: &str| -> Option<String> {
        let open = format!("<hh:borderFill id=\"{id}\"");
        let start = header.find(&open).unwrap();
        let body = &header[start..start + header[start..].find("</hh:borderFill>").unwrap()];
        let brush = &body[body.find("<hc:winBrush")?..];
        let brush = &brush[..brush.find("/>").unwrap()];
        attr(brush, "hatchStyle").map(str::to_string)
    };
    let mut rows: Vec<(u32, Option<String>)> = section
        .split("<hp:tc ")
        .skip(1)
        .filter_map(|tc| {
            let addr = &tc[tc.find("<hp:cellAddr").unwrap()..];
            if attr(addr, "colAddr") != Some("1") {
                return None;
            }
            let row = attr(addr, "rowAddr").unwrap().parse().unwrap();
            Some((row, hatch_of_fill(attr(tc, "borderFillIDRef").unwrap())))
        })
        .collect();
    rows.sort_by_key(|r| r.0);
    rows.into_iter().map(|r| r.1).collect()
}

#[test]
fn hwp_to_hwpx_writes_the_hatch_styles_hancom_writes() {
    let hancom = right_column_hatch_styles(&read(HANCOM_HWPX));
    assert_eq!(
        hancom.iter().map(Option::as_deref).collect::<Vec<_>>(),
        [
            Some("HORIZONTAL"),
            Some("VERTICAL"),
            Some("SLASH"),
            Some("BACK_SLASH"),
            Some("CROSS"),
            Some("CROSS_DIAGONAL"),
            None,
        ]
    );
    let doc = HwpDocument::new(&read(HWP)).unwrap();
    assert_eq!(
        right_column_hatch_styles(&doc.export_hwpx().unwrap()),
        hancom
    );
}

/// 한컴이 저장한 실제 문서: 13쪽 「RECORD_HEADER」 행은 무늬 3(／), 「RECORD_DATA (RD)」 행은
/// 무늬 2(＼)다. 한글 2024 Print도 위 행이 ／, 아래 행이 ＼다.
#[test]
fn hwpspec_record_header_and_data_rows_keep_their_diagonals() {
    let core = DocumentCore::from_bytes(&read("samples/hwpspec.hwp")).unwrap();
    let svg = core.render_page_svg_native(22).unwrap();
    assert_eq!(hatches_top_to_bottom(&svg), [Hatch::Rising, Hatch::Falling]);
}

fn cell_props(doc: &HwpDocument, at: (u32, u32)) -> Value {
    serde_json::from_str(&doc.get_cell_properties(0, at.0, at.1, 0).unwrap()).unwrap()
}

/// 채우기가 없는 칸의 조회값을 되쓰며 면 색만 주면 무늬 없는 단색이다.
/// 조회가 무늬 없음을 0으로 알리면 이 호출이 가로줄을 저장한다.
#[test]
fn reading_back_an_unfilled_cell_does_not_store_a_horizontal_hatch() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    let created: Value = serde_json::from_str(&doc.create_table(0, 0, 0, 1, 1).unwrap()).unwrap();
    let at = (
        created["paraIdx"].as_u64().unwrap() as u32,
        created["controlIdx"].as_u64().unwrap() as u32,
    );
    let mut props = cell_props(&doc, at);
    assert_eq!(props["fillType"], "none");
    assert_eq!(props["patternType"], -1);

    props["fillType"] = "solid".into();
    props["fillColor"] = "#f5f5f5".into();
    doc.set_cell_properties(0, at.0, at.1, 0, &props.to_string())
        .unwrap();
    for (format, bytes) in [
        ("HWP", doc.export_hwp().unwrap()),
        ("HWPX", doc.export_hwpx().unwrap()),
    ] {
        let reopened = HwpDocument::new(&bytes).unwrap();
        let props = cell_props(&reopened, at);
        assert_eq!(props["fillType"], "solid", "{format}");
        assert_eq!(props["patternType"], -1, "{format}");
    }
}
