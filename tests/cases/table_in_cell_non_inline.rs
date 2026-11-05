//! 관계형 회귀: 셀 내부 비인라인 표의 소속·포함·후속 문단 순서·원본 속성. 절대 배치 핀은 없다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! non-inline-cell-table.hwp (api-8570-non-inline-cell-table-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! non-inline-cell-table.hwpx (api-8570-non-inline-cell-table-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형 및 고정형 회귀: 표 칸 안에 글자처럼 취급하지 않는 표를 만든다.
//! 주석 갱신: 2026-11-03T13:40:00+09:00 (Asia/Seoul). 절대 px 위치를 고정하지 않는다.
//!
//! 기대값은 한/글 12.30(Mac)이 만든 문서에서 정했다. 표 칸에 캐럿을 두고 표 > 표 만들기에서
//! 글자처럼 취급을 끈 채 2×3 표를 만들어 HWPX로 저장했다. 안쪽 표는 `treatAsChar="0"`,
//! `textWrap="TOP_AND_BOTTOM"`, `vertRelTo="PARA"`, `horzRelTo="COLUMN"`, `flowWithText="1"`,
//! 바깥 여백 283, `pageBreak="CELL"`, `repeatHeader="1"`이고, 폭은 칸 안쪽 폭(10,487 − 510 − 510)
//! − 바깥 여백 566 = 8,901이었다. 표 문단에는 표만 있고, 뒤에 빈 문단이 하나 생겼다.
//! 글자처럼 취급을 켜고 만든 표는 `issue_8262_table_in_cell`이 본다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{HorzRelTo, TextWrap, VertRelTo};
use rhwp::model::table::{Table, TablePageBreak};
use rhwp::wasm_api::HwpDocument;

/// 한/글이 만든 자리 차지 표의 `common.attr`(세로 문단·가로 단·본문 흐름·위아래·번호 종류 표).
const HNC_FLOATING_TABLE_ATTR: u32 = 0x082a_2210;

/// 빈 문서 첫 문단에 3×3 글자처럼 취급 표를 만든다. 표는 SectionDef·ColumnDef 뒤 컨트롤 2.
fn doc_with_body_table() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.create_table_ex_native(0, 0, 0, 3, 3, true, None, None)
        .unwrap();
    doc
}

fn body_table(doc: &HwpDocument) -> &Table {
    match &doc.document().sections[0].paragraphs[0].controls[2] {
        Control::Table(t) => t,
        _ => panic!("문단 0 컨트롤 2 는 표여야 한다"),
    }
}

fn only_table(para: &Paragraph) -> &Table {
    assert_eq!(para.controls.len(), 1, "표 문단에는 표만 있다");
    match &para.controls[0] {
        Control::Table(t) => t,
        _ => panic!("표 문단의 컨트롤은 표여야 한다"),
    }
}

fn assert_hancom_floating_shape(inner: &Table, col_w: u32) {
    assert!(!inner.common.treat_as_char, "글자처럼 취급하지 않는다");
    assert_eq!(inner.common.text_wrap, TextWrap::TopAndBottom);
    assert_eq!(inner.common.vert_rel_to, VertRelTo::Para);
    assert_eq!(inner.common.horz_rel_to, HorzRelTo::Column);
    assert!(inner.common.flow_with_text, "쪽 영역 안으로 제한");
    let m = &inner.common.margin;
    assert_eq!((m.left, m.right, m.top, m.bottom), (283, 283, 283, 283));
    assert_eq!(inner.page_break, TablePageBreak::RowBreak, "pageBreak CELL");
    assert!(inner.repeat_header, "제목 줄 반복");
    assert_eq!(inner.common.width, col_w * 2, "표 폭");
}

#[test]
fn create_non_inline_table_in_cell_matches_hancom() {
    // 바깥 칸 폭 13,984 → 안쪽 12,964 → −566 = 12,398 → 칸당 6,199
    let mut doc = doc_with_body_table();
    let json = doc
        .create_table_in_cell_by_path_native(0, 0, &[(2, 4, 0)], 0, 2, 2, None, None, false)
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["cellParaIdx"], 0);

    let cell = &body_table(&doc).cells[4];
    assert_eq!(cell.paragraphs.len(), 2, "표 문단 + 뒤 빈 문단");
    let host = &cell.paragraphs[0];
    assert_eq!(host.char_count, 9, "컨트롤 8 + 문단 끝 1");
    let inner = only_table(host);
    assert_hancom_floating_shape(inner, 6_199);
    assert_eq!(inner.common.attr, HNC_FLOATING_TABLE_ATTR, "common.attr");
    assert_eq!(inner.attr, HNC_FLOATING_TABLE_ATTR, "attr = common.attr");
    let tail = &cell.paragraphs[1];
    assert!(tail.text.is_empty() && tail.controls.is_empty());
}

#[test]
fn non_inline_table_in_cell_is_placed_inside_its_cell() {
    use serde_json::Value;

    fn find<'a>(node: &'a Value, kind: &str) -> Option<&'a Value> {
        if node["type"] == kind {
            return Some(node);
        }
        node["children"]
            .as_array()?
            .iter()
            .find_map(|n| find(n, kind))
    }

    let mut doc = doc_with_body_table();
    doc.create_table_in_cell_by_path_native(0, 0, &[(2, 4, 0)], 0, 2, 2, None, None, false)
        .unwrap();
    doc.insert_text_in_cell_native(0, 0, 2, 4, 1, 0, "뒤")
        .unwrap();
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let outer = find(&tree, "Table").expect("본문 표");
    let cell = outer["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Cell" && n["row"] == 1 && n["col"] == 1)
        .expect("가운데 칸");
    let nested = cell["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Table")
        .expect("칸 소유의 중첩 표");
    let b = |n: &Value, k: &str| n["bbox"][k].as_f64().unwrap();
    let eps = 0.2;
    assert!(b(nested, "x") + eps >= b(cell, "x"), "왼쪽이 칸 안");
    assert!(b(nested, "y") + eps >= b(cell, "y"), "위쪽이 칸 안");
    assert!(
        b(nested, "x") + b(nested, "w") <= b(cell, "x") + b(cell, "w") + eps,
        "오른쪽이 칸 안"
    );
    let bottom = b(nested, "y") + b(nested, "h");
    assert!(
        bottom <= b(cell, "y") + b(cell, "h") + eps,
        "아래쪽이 칸 안"
    );
    let tail = cell["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "TextLine" && n["pi"] == 1)
        .expect("표 뒤 문단의 줄");
    assert!(b(tail, "y") + eps >= bottom, "뒤 문단은 표 아래에 놓인다");
}

#[test]
fn non_inline_table_in_cell_survives_hwp_and_hwpx_roundtrip() {
    let mut doc = doc_with_body_table();
    doc.create_table_in_cell_by_path_native(0, 0, &[(2, 4, 0)], 0, 2, 2, None, None, false)
        .unwrap();
    for (label, bytes) in [
        ("hwp", doc.export_hwp_native().unwrap()),
        ("hwpx", doc.export_hwpx_native().unwrap()),
    ] {
        let reloaded =
            HwpDocument::new(&bytes).unwrap_or_else(|_| panic!("{label} 다시 읽기 실패"));
        let cell = &body_table(&reloaded).cells[4];
        assert_eq!(cell.paragraphs.len(), 2, "{label}: 표 문단 + 뒤 빈 문단");
        assert_hancom_floating_shape(only_table(&cell.paragraphs[0]), 6_199);
    }
}
