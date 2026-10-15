#![cfg(not(target_arch = "wasm32"))]
//! 그러데이션·그림 채우기는 속성 조회에서 제 종류로 보이고, 읽은 속성을 다시 써도 남는다.
//!
//! 테두리/배경 조회가 그러데이션·그림 채우기를 `fillType: "none"` 으로 돌려주면, 대화상자처럼
//! 읽은 속성에서 한 값만 바꿔 다시 쓰는 호출이 채우기를 지운다. 바꾼 문서를 HWP·HWPX로
//! 저장해 다시 열어도 손대지 않은 문서를 저장해 다시 연 것과 채우기가 같아야 한다.
use rhwp::model::control::Control;
use rhwp::model::style::{Fill, FillType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

/// 본문 표 칸에 그러데이션·그림 채우기가 있는 HWP·HWPX 표본.
const CELL_SAMPLES: [&str; 3] = [
    "samples/issue7235/156086935_none_image_fill.hwp",
    "samples/hwpx/mel-001.hwpx",
    "samples/hwpx/el-school-001.hwpx",
];

/// 쪽 배경이 그러데이션·그림인 HWP·HWPX 표본.
const PAGE_SAMPLES: [&str; 2] = [
    "samples/basic/NewYear_s_Day.hwp",
    "samples/issue2816/imgbrush_total_page_fill.hwpx",
];

/// (구역, 문단, 컨트롤, 칸).
type CellAt = (usize, usize, usize, usize);

fn open(path: &str) -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

/// HWP·HWPX로 저장해 다시 연 문서.
fn reopened(doc: &HwpDocument) -> [(&'static str, HwpDocument); 2] {
    let open = |bytes: Vec<u8>| HwpDocument::from_bytes(&bytes).unwrap();
    [
        ("HWP", open(doc.export_hwp().unwrap())),
        ("HWPX", open(doc.export_hwpx().unwrap())),
    ]
}

fn fill_of(doc: &HwpDocument, border_fill_id: u16) -> Fill {
    let border_fills = &doc.document().doc_info.border_fills;
    border_fills[usize::from(border_fill_id) - 1].fill.clone()
}

/// 그러데이션·그림이면 조회가 돌려줄 `fillType`.
fn kind(fill: &Fill) -> Option<&'static str> {
    match fill.fill_type {
        FillType::Gradient => Some("gradient"),
        FillType::Image => Some("image"),
        _ => None,
    }
}

fn cell_fill(doc: &HwpDocument, (s, p, c, i): CellAt) -> Fill {
    match &doc.document().sections[s].paragraphs[p].controls[c] {
        Control::Table(table) => fill_of(doc, table.cells[i].border_fill_id),
        _ => panic!("표가 아니다"),
    }
}

fn page_fill(doc: &HwpDocument) -> Fill {
    let page_border_fill = &doc.document().sections[0].section_def.page_border_fill;
    fill_of(doc, page_border_fill.border_fill_id)
}

/// 영역 테두리/배경이 없는 본문 표에서 그러데이션·그림 채우기 칸을 찾는다.
fn filled_cells(doc: &HwpDocument) -> Vec<CellAt> {
    let mut cells = Vec::new();
    for (s, section) in doc.document().sections.iter().enumerate() {
        for (p, para) in section.paragraphs.iter().enumerate() {
            for (c, control) in para.controls.iter().enumerate() {
                let Control::Table(table) = control else {
                    continue;
                };
                if !table.zones.is_empty() {
                    continue;
                }
                for (i, cell) in table.cells.iter().enumerate() {
                    let id = cell.border_fill_id;
                    if id > 0 && kind(&fill_of(doc, id)).is_some() {
                        cells.push((s, p, c, i));
                    }
                }
            }
        }
    }
    cells
}

#[test]
fn rewriting_read_cell_properties_keeps_gradient_and_image_fills() {
    let mut problems = Vec::new();
    let mut kinds = Vec::new();
    for path in CELL_SAMPLES {
        let mut doc = open(path);
        let cells = filled_cells(&doc);
        let before: Vec<Fill> = cells.iter().map(|&at| cell_fill(&doc, at)).collect();
        let saved_before = reopened(&doc);

        for (&at, fill) in cells.iter().zip(&before) {
            let (s, p, c, i) = (at.0 as u32, at.1 as u32, at.2 as u32, at.3 as u32);
            let mut props: Value =
                serde_json::from_str(&doc.get_cell_properties(s, p, c, i).unwrap()).unwrap();
            kinds.push(kind(fill));
            if props["fillType"].as_str() != kind(fill) {
                problems.push(format!("{path} {at:?} 조회 {}", props["fillType"]));
            }
            // 셀 속성 대화상자처럼 읽은 값에서 안쪽 여백 하나만 바꿔 다시 쓴다.
            props["paddingLeft"] = (props["paddingLeft"].as_i64().unwrap() + 100).into();
            doc.set_cell_properties(s, p, c, i, &props.to_string())
                .unwrap();
        }

        for (&at, fill) in cells.iter().zip(&before) {
            let now = cell_fill(&doc, at);
            if now != *fill {
                problems.push(format!("{path} {at:?} 편집 직후 {:?}", now.fill_type));
            }
        }
        for ((format, saved), (_, saved_before)) in reopened(&doc).iter().zip(&saved_before) {
            for &at in &cells {
                let now = cell_fill(saved, at);
                if now != cell_fill(saved_before, at) {
                    problems.push(format!(
                        "{path} {at:?} {format} 다시 열기 {:?}",
                        now.fill_type
                    ));
                }
            }
        }
    }
    assert!(
        kinds.contains(&Some("gradient")) && kinds.contains(&Some("image")),
        "그러데이션·그림 칸을 모두 찾아야 한다: {kinds:?}"
    );
    assert!(problems.is_empty(), "채우기를 잃었다: {problems:#?}");
}

#[test]
fn rewriting_read_page_border_fill_keeps_gradient_and_image_fills() {
    let mut problems = Vec::new();
    for path in PAGE_SAMPLES {
        let mut doc = open(path);
        let before = page_fill(&doc);
        let saved_before = reopened(&doc);

        let json = doc.get_page_border_fill(0).unwrap();
        let props: Value = serde_json::from_str(&json).unwrap();
        if props["fillType"].as_str() != kind(&before) {
            problems.push(format!("{path} 조회 {}", props["fillType"]));
        }
        doc.set_page_border_fill(0, &json).unwrap();

        let now = page_fill(&doc);
        if now != before {
            problems.push(format!("{path} 편집 직후 {:?}", now.fill_type));
        }
        for ((format, saved), (_, saved_before)) in reopened(&doc).iter().zip(&saved_before) {
            let now = page_fill(saved);
            if now != page_fill(saved_before) {
                problems.push(format!("{path} {format} 다시 열기 {:?}", now.fill_type));
            }
        }
    }
    assert!(problems.is_empty(), "쪽 배경을 잃었다: {problems:#?}");
}
