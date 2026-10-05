//! [#7523] 단 정의를 바꾼 뒤 HWP 로 저장하고 다시 열면 바꾼 값이 그대로 나와야 한다.
//!
//! 결함 형태: HWP 파서는 단 정의 속성 u16 을 `raw_attr` 에 통째로 보관하고, 직렬화기는
//! `raw_attr` 가 0 이 아니면 구조화 필드 대신 그 값을 쓴다. `set_column_def_native` 는
//! 단 수·종류·너비 동일 필드만 바꾸고 `raw_attr` 는 그대로 두어, 저장본에는 원래
//! 단 수가 실렸다(1단 → 2단 편집이 다시 열면 1단).
//!
//! 판정은 다시 연 문서가 편집한 문서와 같은 단 정의·같은 단 영역을 갖는가다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::page::{ColumnDef, ColumnType};

fn load(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let bytes = std::fs::read(path).expect("표본 로드");
    DocumentCore::from_bytes(&bytes).expect("파싱")
}

fn reopen(core: &DocumentCore) -> DocumentCore {
    let bytes = core.export_hwp_native().expect("HWP 저장");
    DocumentCore::from_bytes(&bytes).expect("저장본 다시 열기")
}

fn first_column_def(core: &DocumentCore) -> ColumnDef {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .find_map(|c| match c {
            Control::ColumnDef(cd) => Some(cd.clone()),
            _ => None,
        })
        .expect("구역 0 에 단 정의가 있어야 한다")
}

/// 첫 쪽의 단 영역 (x, 너비).
fn page_columns(core: &DocumentCore) -> serde_json::Value {
    let info = core.get_page_info_native(0).expect("쪽 정보");
    serde_json::from_str::<serde_json::Value>(&info).expect("쪽 정보 JSON")["columns"].clone()
}

#[test]
fn edited_column_count_type_and_spacing_survive_hwp_save() {
    for (count, column_type, expected_type, spacing) in [
        (2, 0, ColumnType::Normal, 2268),
        (3, 1, ColumnType::Distribute, 1134),
    ] {
        let mut core = load("samples/field-01.hwp");
        assert_eq!(
            first_column_def(&core).column_count,
            1,
            "표본 전제 위반: 1단 문서여야 한다"
        );
        core.set_column_def_native(0, count, column_type, true, spacing)
            .expect("단 정의 변경");

        let reopened = reopen(&core);
        let cd = first_column_def(&reopened);
        assert_eq!(cd.column_count, count, "다시 연 문서의 단 수");
        assert_eq!(cd.column_type, expected_type, "다시 연 문서의 단 종류");
        assert!(cd.same_width, "다시 연 문서의 단 너비 동일");
        assert_eq!(cd.spacing, spacing, "다시 연 문서의 단 간격");
        assert_eq!(
            page_columns(&reopened),
            page_columns(&core),
            "다시 연 문서의 단 영역이 편집한 문서와 같아야 한다"
        );
    }
}

/// 너비를 따로 주지 않은 채 '너비 동일'을 끄면(편집기의 왼쪽/오른쪽 다단 프리셋) 단별
/// 너비 목록이 비어 있다. 저장본은 이 상태를 같은 단 배치로 다시 열어야 하며, 비어 있는
/// 단별 너비 자리에 구분선 바이트를 읽어 들이면 안 된다.
#[test]
fn mixed_width_without_column_widths_survives_hwp_save() {
    let mut core = load("samples/field-01.hwp");
    core.set_column_def_native(0, 2, 0, false, 2268)
        .expect("단 정의 변경");
    let edited = first_column_def(&core);

    let reopened = reopen(&core);
    let cd = first_column_def(&reopened);
    assert_eq!(cd.column_count, 2, "다시 연 문서의 단 수");
    assert_eq!(cd.spacing, 2268, "다시 연 문서의 단 간격");
    assert_eq!(
        (cd.separator_type, cd.separator_width, cd.separator_color),
        (
            edited.separator_type,
            edited.separator_width,
            edited.separator_color
        ),
        "다시 연 문서의 단 구분선"
    );
    assert_eq!(
        page_columns(&reopened),
        page_columns(&core),
        "다시 연 문서의 단 영역이 편집한 문서와 같아야 한다"
    );
}

/// 단별 너비가 있는 문서(KTX.hwp: 2단, 117.7mm/158.3mm)에서 '너비 동일'을 끈 채 단 수를
/// 바꾸면 옛 단 수의 너비 목록은 새 단에 맞지 않는다. 저장본은 편집한 문서와 같은 단
/// 배치로 다시 열려야 한다.
#[test]
fn mixed_width_count_change_survives_hwp_save() {
    let mut core = load("samples/basic/KTX.hwp");
    let original = first_column_def(&core);
    assert!(
        original.column_count == 2 && !original.same_width && original.widths.len() == 2,
        "표본 전제 위반: 단별 너비가 있는 2단 문서여야 한다"
    );
    core.set_column_def_native(0, 3, 0, false, 1134)
        .expect("단 정의 변경");

    let reopened = reopen(&core);
    let cd = first_column_def(&reopened);
    assert_eq!(cd.column_count, 3, "다시 연 문서의 단 수");
    assert_eq!(
        (cd.separator_type, cd.separator_width, cd.separator_color),
        (
            original.separator_type,
            original.separator_width,
            original.separator_color
        ),
        "다시 연 문서의 단 구분선"
    );
    assert_eq!(
        page_columns(&reopened),
        page_columns(&core),
        "다시 연 문서의 단 영역이 편집한 문서와 같아야 한다"
    );
}

fn collect_column_text<'a>(
    node: &'a serde_json::Value,
    column: Option<u64>,
    runs: &mut Vec<(u64, &'a serde_json::Value)>,
) {
    let column = if node["type"] == "Column" {
        node["col"].as_u64()
    } else {
        column
    };
    if node["type"] == "TextRun" {
        if let Some(column) = column {
            runs.push((column, node));
        }
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect_column_text(child, column, runs);
        }
    }
}

/// 실제 단 폭 편집·HWP 저장본의 문단 내부 재조판 좌표를 저장 단나누기로 해석하면
/// 회사 정보가 오른쪽 단으로 넘어간다. 독립 한컴 Print의 3쪽을 Native/fresh WASM으로
/// 확인한 입력이며, 절대 위치 대신 제목 뒤 순서·첫 단 소속·내용 보존을 검사한다.
#[test]
fn reflowed_paragraph_local_positions_do_not_advance_column() {
    let core = load("mydocs/pr/assets/semanticist21-20261005/pr7527/pr7527-two-columns.hwp");
    assert_eq!(core.page_count(), 3, "독립 Print와 전체 쪽수 일치");
    let trees: Vec<serde_json::Value> = (0..core.page_count())
        .map(|page| {
            serde_json::from_str(&core.build_page_render_tree(page).unwrap().root.to_json())
                .unwrap()
        })
        .collect();
    let mut first_runs = Vec::new();
    collect_column_text(&trees[0], None, &mut first_runs);
    let body = trees[0]["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["type"] == "Body")
        .unwrap();
    let column = body["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["type"] == "Column" && node["col"] == 0)
        .unwrap();
    let left = column["bbox"]["x"].as_f64().unwrap();
    let right = left + column["bbox"]["w"].as_f64().unwrap();
    let title_end = first_runs
        .iter()
        .filter(|(_, run)| {
            ["마케팅", "전략", "기획서"]
                .iter()
                .any(|word| run["text"].as_str().unwrap_or("").starts_with(word))
        })
        .map(|(_, run)| run["bbox"]["y"].as_f64().unwrap() + run["bbox"]["h"].as_f64().unwrap())
        .reduce(f64::max)
        .expect("제목 보존");
    let mut previous_end = title_end;
    for label in ["회사명", "작성자", "부서명", "Tel", "E-Mail"] {
        let matches: Vec<_> = first_runs
            .iter()
            .filter(|(_, run)| run["text"].as_str().unwrap_or("").starts_with(label))
            .collect();
        assert_eq!(matches.len(), 1, "첫 쪽의 {label} 누락·중복 금지");
        let (owner, run) = matches[0];
        assert_eq!(
            *owner, 0,
            "문단 내부 VPOS 되감김은 단 경계가 아니다: {label}"
        );
        let box_ = &run["bbox"];
        let x = box_["x"].as_f64().unwrap();
        let y = box_["y"].as_f64().unwrap();
        assert!(x >= left && x + box_["w"].as_f64().unwrap() <= right);
        assert!(
            y >= previous_end,
            "제목·회사 정보의 내용 순서 보존: {label}"
        );
        previous_end = y + box_["h"].as_f64().unwrap();
        let mut all_runs = Vec::new();
        for tree in &trees {
            collect_column_text(tree, None, &mut all_runs);
        }
        assert_eq!(
            all_runs
                .iter()
                .filter(|(_, run)| run["text"].as_str().unwrap_or("").starts_with(label))
                .count(),
            1,
            "뒤 쪽에 {label}을 중복 배치하지 않는다"
        );
    }
}
