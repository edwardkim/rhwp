//! 관계형: 표 조각의 cellzone 포함·도색 순서, 글상자 목차 쪽번호 소속,
//! 원 HFT 부재 시 표시 대체 글꼴의 공급 순서.
//! 고정형: 독립 Print의 전체 64쪽과 2쪽 목차 쪽번호 x, 8쪽 zone 배경 위쪽 y(96 DPI, ±2px).
//! 주석 갱신: 2026-11-21 19:04 KST.
//! 입력 `samples/issue886/1660000-201400009-kobe-earthquake.hwp`.
//! 기준 `pdf/issue886/1660000-201400009-kobe-earthquake-hancom-2020.pdf`(Hancom PDF Print 64쪽).
//! 측정(2026-11-21, 통합 코드 `cef9ff9acc`, Visual Sweep 96 DPI·compat 2022, 원 HFT 공급):
//! Native/fresh WASM 2쪽 raw 99.87/99.87%, 8쪽 99.99/99.99%, 전쪽 최저 98.12/98.15%.
//! 1쪽 원 HFT 부재 조건의 남은 차이는 표시 대체 HY견고딕의 자형으로, 글꼴 공급 관계로 검사한다.
//! 증거: `mydocs/pr/assets/planet6897_9240_9282_20261121/coordinate-evidence/886/`.
//!
//! 원인: 구역 2 문단 49의 26행×3열 표는 7~8쪽에 걸친다. 18행 `1995년 1월 18일 수요일` 칸(병합
//! 3열)의 고유 테두리/배경(bf=8)은 채우기가 없고, 표의 cellzone (18,0)~(18,0) 이 bf=36
//! (면색 #FAFABF)을 준다. 한컴 2020 Print 8쪽은 이 행을 노랗게 칠한다. rhwp 는 비분할 표
//! 경로(`table_layout` 4-2)에서만 zone 배경을 칠하고 조각 표(`table_partial`)는 zone 테두리만
//! 덮어써, 이어진 조각에 놓인 zone 배경이 통째로 빠졌다.
//!
//! 검사: zone 끝 칸까지의 span 을 쓰는 같은 규칙으로 zone 칸을 찾고, 그 칸이 놓인 조각에서
//! zone 채우기 색의 사각형이 그 칸 상자를 덮는지(포함), 칸보다 먼저(아래에) 칠해지는지(순서),
//! 그 조각이 표의 첫 조각이 아닌지(이어받은 조각)를 본다. 색은 원본 BorderFill 에서 읽는다.
//!
//! 1쪽 원인: 제목 글자(charPr 35)의 한글 face 는 HFT `한양견고딕` 이다. 원 프로그램이 없으면
//! 렌더러 사슬(#6171)이 HY견고딕을 표시 대체로 고르고 fresh WASM 도 그렇게 그리는데, Native
//! 글꼴 공급(`native_webfonts::prepare`)과 Visual Sweep 의 Native 준비는 공급 원장에 없는 원본을
//! 곧장 공개 Noto 로 치환해 Native 만 가는 Regular 고딕으로 그렸다. 글꼴 계획 요청에 표시 대체
//! (`displayAliases`)를 싣고, 설치된 표시 대체가 있으면 공개 Noto 로 치환하지 않는지 본다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue886/1660000-201400009-kobe-earthquake.hwp";
const SECTION: usize = 2;
const PARA: usize = 49;
/// 독립 한컴 2020 Print의 쪽수.
const PRINT_PAGES: u32 = 64;
const PRINT_COORDINATE_TOLERANCE_PX: f64 = 2.0;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("required fixture"))
        .expect("HWP parse");
    core.set_hangul2024_compat(false);
    core
}

fn assert_print_coordinate(pixel: f64, point: f64, label: &str) {
    let expected = point * 96.0 / 72.0;
    assert!(
        (pixel - expected).abs() <= PRINT_COORDINATE_TOLERANCE_PX,
        "{label}: {pixel:.2}px, 독립 Print {expected:.2}px"
    );
}

fn table(core: &DocumentCore) -> &Table {
    core.document().sections[SECTION].paragraphs[PARA]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("구역 2 문단 49 의 표")
}

/// 이 표(구역·문단)의 표 노드들.
fn table_nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = Vec::new();
    if let RenderNodeType::Table(t) = &node.node_type {
        if t.section_index == Some(SECTION) && t.para_index == Some(PARA) {
            out.push(node);
        }
    }
    for child in &node.children {
        out.extend(table_nodes(child));
    }
    out
}

fn covers(outer: &BoundingBox, inner: &BoundingBox) -> bool {
    outer.x <= inner.x + 0.5
        && outer.y <= inner.y + 0.5
        && outer.x + outer.width >= inner.x + inner.width - 0.5
        && outer.y + outer.height >= inner.y + inner.height - 0.5
}

#[test]
fn issue_886_partial_table_paints_cellzone_fill() {
    let core = core();
    assert_eq!(
        core.page_count(),
        PRINT_PAGES,
        "독립 한컴 Print 와 같은 쪽 수"
    );

    let table = table(&core);
    let zone = table
        .zones
        .iter()
        .find(|zone| {
            zone.border_fill_id > 0
                && core.document().doc_info.border_fills[usize::from(zone.border_fill_id) - 1]
                    .fill
                    .solid
                    .is_some()
        })
        .expect("면색 채우기가 있는 cellzone");
    let color = core.document().doc_info.border_fills[usize::from(zone.border_fill_id) - 1]
        .fill
        .solid
        .as_ref()
        .unwrap()
        .background_color;
    let zone_cell = table
        .cells
        .iter()
        .find(|cell| cell.row == zone.start_row && cell.col == zone.start_col)
        .expect("zone 시작 칸");
    let end_cell = table
        .cells
        .iter()
        .find(|cell| cell.row == zone.end_row && cell.col == zone.end_col)
        .expect("zone 끝 칸");
    // 표본의 전제: zone 칸 자신은 채우기가 없어 배경은 zone 만이 준다.
    assert!(
        core.document().doc_info.border_fills[usize::from(zone_cell.border_fill_id) - 1]
            .fill
            .solid
            .is_none(),
        "zone 칸의 고유 BorderFill 에는 면색이 없다"
    );
    let end_row = end_cell.row + end_cell.row_span.max(1);
    let end_col = end_cell.col + end_cell.col_span.max(1);

    let mut first_page = None;
    let mut checked = false;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render tree");
        for node in table_nodes(&tree.root) {
            first_page.get_or_insert(page);
            let cells: Vec<(usize, &RenderNode)> = node
                .children
                .iter()
                .enumerate()
                .filter(|(_, child)| match &child.node_type {
                    RenderNodeType::TableCell(meta) => {
                        meta.row >= zone.start_row
                            && meta.col >= zone.start_col
                            && meta.row + meta.row_span.max(1) <= end_row
                            && meta.col + meta.col_span.max(1) <= end_col
                    }
                    _ => false,
                })
                .collect();
            if cells.is_empty() {
                continue;
            }
            assert!(
                first_page.is_some_and(|first| first < page),
                "zone 칸이 표의 이어받은 조각({}쪽)에 놓인다",
                page + 1
            );
            assert_eq!(page + 1, 8, "1월 18일의 zone은 Print 8쪽에 놓인다");
            for (cell_index, cell) in cells {
                let fill = node.children.iter().enumerate().find(|(_, child)| {
                    matches!(&child.node_type, RenderNodeType::Rectangle(rect)
                        if rect.style.fill_color == Some(color))
                        && covers(&child.bbox, &cell.bbox)
                });
                let Some((fill_index, fill)) = fill else {
                    panic!(
                        "{}쪽 zone 칸 {:?} 을 덮는 zone 면색 사각형이 없다",
                        page + 1,
                        cell.bbox
                    );
                };
                assert!(
                    fill_index < cell_index,
                    "zone 배경이 칸보다 먼저(아래에) 칠해진다"
                );
                assert!(
                    covers(&cell.bbox, &fill.bbox),
                    "zone 배경 {:?} 이 zone 칸 {:?} 밖으로 넘치지 않는다",
                    fill.bbox,
                    cell.bbox
                );
                assert_print_coordinate(fill.bbox.y, 215.284, "8쪽 zone 배경 위");
                checked = true;
            }
        }
    }
    assert!(checked, "zone 칸을 그린 조각을 찾았다");
}

fn collect_chapter_three_page_number(node: &RenderNode, numbers: &mut Vec<BoundingBox>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.para_index == Some(8) && run.text == "16"
            && run.cell_context.as_ref().is_some_and(|context| context.in_textbox))
    {
        numbers.push(node.bbox);
    }
    for child in &node.children {
        collect_chapter_three_page_number(child, numbers);
    }
}

#[test]
fn issue_886_textbox_toc_leader_keeps_the_print_page_number_origin() {
    let core = core();
    let tree = core.build_page_render_tree(1).expect("2쪽 목차");
    let mut numbers = Vec::new();
    collect_chapter_three_page_number(&tree.root, &mut numbers);
    assert_eq!(
        numbers.len(),
        1,
        "제3장 쪽번호는 목차 글상자에 한 번 그린다"
    );
    assert_print_coordinate(numbers[0].x, 471.12, "2쪽 제3장 쪽번호 원점");
}

/// 공개 Noto 프로그램의 `name` 표만 바꿔 다른 family 로 등록되는 합성 글꼴을 만든다.
/// 글꼴 공급 결정만 검사하므로 윤곽·전진폭은 원래 Noto 그대로다.
fn renamed_font(font: &[u8], family: &str, postscript: &str) -> Vec<u8> {
    let be16 = |b: &[u8], at: usize| u16::from_be_bytes([b[at], b[at + 1]]);
    let tables = usize::from(be16(font, 4));
    let mut out = font.to_vec();
    // 새 name 표: format 0, Windows/Unicode BMP/en-US 의 family(1)·full(4)·PostScript(6).
    let records: Vec<(u16, Vec<u8>)> = [(1u16, family), (4, family), (6, postscript)]
        .into_iter()
        .map(|(id, text)| (id, text.encode_utf16().flat_map(u16::to_be_bytes).collect()))
        .collect();
    let mut name = Vec::new();
    name.extend(0u16.to_be_bytes());
    name.extend((records.len() as u16).to_be_bytes());
    name.extend(((6 + 12 * records.len()) as u16).to_be_bytes());
    let mut offset = 0u16;
    for (id, text) in &records {
        for value in [3u16, 1, 0x0409, *id, text.len() as u16, offset] {
            name.extend(value.to_be_bytes());
        }
        offset += text.len() as u16;
    }
    for (_, text) in &records {
        name.extend(text);
    }
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
    let name_offset = out.len() as u32;
    out.extend(&name);
    let entry = (0..tables)
        .map(|i| 12 + 16 * i)
        .find(|&at| &font[at..at + 4] == b"name")
        .expect("name 표");
    out[entry + 8..entry + 12].copy_from_slice(&name_offset.to_be_bytes());
    out[entry + 12..entry + 16].copy_from_slice(&(name.len() as u32).to_be_bytes());
    out
}

/// 원 HFT `한양견고딕` 은 공급되지 않고, 렌더러가 표시 대체로 선언한 `HY견고딕`(#6171)만
/// 설치된 환경. 고베대지진 1쪽 제목(charPr 35, 한글 face 4 = 한양견고딕)이 이 경우다.
/// fresh WASM(SVG/Canvas 사슬)은 HY견고딕으로 그리는데 Native 공급만 공개 Noto 로 바꿔
/// 가는 Regular 고딕이 됐다. 선언 대체가 설치돼 있으면 Noto 로 치환하지 않아야 한다.
/// (선언 대체가 없는 원본의 Noto 치환은 `issue_8533_native_webfont_supply` 가 지킨다.)
#[test]
fn issue_886_native_supply_keeps_installed_display_alias_over_generic_noto() {
    let source = "한양견고딕";
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    for faces in &mut document.doc_info.font_faces {
        for face in faces {
            face.name = source.into();
            face.is_embedded = false;
            face.subst_font = None;
        }
    }
    // 공급자(Native·Visual Sweep)는 렌더러 사슬의 표시 대체를 글꼴 계획에서 읽는다.
    core.set_document(document.clone());
    let plan: serde_json::Value =
        serde_json::from_str(&core.get_document_font_selection_plan()).unwrap();
    let request = plan["requests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|request| request["source"] == source)
        .expect("한양견고딕 요청");
    assert!(
        request["displayAliases"]
            .as_array()
            .is_some_and(|aliases| aliases.iter().any(|alias| alias == "HY견고딕")),
        "글꼴 계획이 표시 대체 HY견고딕을 알린다: {request}"
    );
    assert!(
        !request["primary"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "HY견고딕"),
        "표시 대체를 원 프로그램 신원(primary)에 섞지 않는다"
    );

    let noto = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("ttfs/opensource/NotoSansKR-Regular.ttf"),
    )
    .expect("공개 Noto");
    let dir = std::env::temp_dir().join(format!("rhwp-issue886-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let alias = dir.join("issue886-hy-gyeongothic.ttf");
    std::fs::write(
        &alias,
        renamed_font(&noto, "HY견고딕", "RHWPIssue886Gyeongothic"),
    )
    .unwrap();

    let supply =
        rhwp::renderer::native_webfonts::prepare(&document.doc_info, std::slice::from_ref(&alias));
    let _ = std::fs::remove_dir_all(&dir);
    let supply = supply.expect("Native 글꼴 공급");
    assert!(
        supply.unavailable.iter().any(|name| name == source),
        "원 프로그램이 없다는 진단은 유지한다"
    );
    assert!(
        !supply.mappings.contains_key(source),
        "설치된 표시 대체(HY견고딕)를 공개 Noto 로 덮지 않는다: {:?}",
        supply.mappings
    );
    assert!(
        !supply.generic_fallbacks.iter().any(|name| name == source),
        "일반 Noto 대체로 기록하지 않는다"
    );
}
