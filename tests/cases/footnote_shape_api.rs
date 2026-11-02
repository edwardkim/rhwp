//! 각주 모양(주석 모양 > 각주 모양 탭)을 읽고 바꾼다.
//!
//! 시험 문서는 한/글 12.30(Mac)에서 각주 모양을 ①,②,③·앞뒤 장식 문자 [ ]·구분선 파선
//! 0.4 mm·여백(구분선 위 4.0, 각주 사이 1.5, 구분선 아래 2.5 mm)·쪽마다 새로 시작·작게·
//! 본문 아래·오른쪽 단 아래로 정한 뒤, 1쪽에 각주 둘을 넣고 쪽을 나눠 2쪽에 각주 하나를 넣어
//! 저장했다. 한/글은 새 각주에도 이 모양을 쓰고 번호를 1·2·1로 저장하며, 인쇄
//! PDF(`pdf/footnote_shape/`)에 1쪽 [①] [②], 2쪽 [①]을 찍는다. 같은 문서를 HWPX로 저장한
//! `footNotePr`의 요소는 아래 `HANCOM_FOOTNOTE_PR`이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::{json, Value};

const SAMPLE: &str = "tests/fixtures/footnote_shape/restart-page.hwp";
/// 구분선 종류는 뺐다. 이 문서의 HWP 저장값(3)을 한/글은 HWPX에 `DASH`로 쓰지만 rhwp HWPX
/// 저장은 이 모양 API와 별개로 3을 `DOT`으로 쓴다.
const HANCOM_FOOTNOTE_PR: [&str; 4] = [
    r#"<hp:autoNumFormat type="CIRCLED_DIGIT" userChar="" prefixChar="[" suffixChar="]" supscript="1"/>"#,
    r#"<hp:noteSpacing betweenNotes="425" belowLine="708" aboveLine="1133"/>"#,
    r#"<hp:numbering type="ON_PAGE" newNum="1"/>"#,
    r#"<hp:placement place="RIGHT_MOST_COLUMN" beneathText="1"/>"#,
];

fn open(bytes: &[u8]) -> DocumentCore {
    DocumentCore::from_bytes(bytes).expect("open")
}

fn sample() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    open(&std::fs::read(path).expect("read sample"))
}

fn shape(core: &DocumentCore) -> Value {
    serde_json::from_str(&core.get_footnote_shape_native(0).expect("footnote shape")).unwrap()
}

fn numbers(core: &DocumentCore) -> Vec<u16> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| &para.controls)
        .filter_map(|control| match control {
            Control::Footnote(note) => Some(note.number),
            _ => None,
        })
        .collect()
}

/// 쪽마다 본문 각주 표시를 차례로 모은다.
fn markers(core: &DocumentCore) -> Vec<Vec<String>> {
    fn collect(node: &RenderNode, out: &mut Vec<String>) {
        if let RenderNodeType::FootnoteMarker(marker) = &node.node_type {
            out.push(marker.text.clone());
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("render tree");
            let mut out = Vec::new();
            collect(&tree.root, &mut out);
            out
        })
        .collect()
}

fn footnote_pr(hwpx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("hwpx zip");
    let mut xml = String::new();
    std::io::Read::read_to_string(
        &mut zip.by_name("Contents/section0.xml").expect("section0"),
        &mut xml,
    )
    .unwrap();
    let start = xml.find("<hp:footNotePr>").expect("footNotePr");
    let end = xml[start..].find("</hp:footNotePr>").unwrap() + start;
    xml[start..end + "</hp:footNotePr>".len()].to_string()
}

#[test]
fn reads_the_footnote_shape_hancom_saved() {
    let core = sample();
    let shape = shape(&core);
    for (key, expected) in [
        ("numberFormat", json!("circledDigit")),
        ("prefixChar", json!("[")),
        ("suffixChar", json!("]")),
        ("separatorEnabled", json!(true)),
        ("separatorLength", json!(-1)),
        ("separatorMarginTop", json!(1133)),
        ("separatorMarginBottom", json!(708)),
        ("noteSpacing", json!(425)),
        ("numberCodeSuperscript", json!(true)),
        ("printInlineAfterText", json!(true)),
        ("numbering", json!("restartPage")),
        ("placement", json!("rightColumn")),
    ] {
        assert_eq!(shape[key], expected, "{key}: {shape}");
    }
    assert_eq!(numbers(&core), [1, 2, 1]);
    assert_eq!(markers(&core), [vec!["[①]", "[②]"], vec!["[①]"]]);
}

#[test]
fn applying_a_shape_renumbers_and_restyles_existing_footnotes() {
    let mut core = sample();
    core.apply_footnote_shape_native(
        0,
        r#"{"numberFormat":"digit","prefixChar":"","suffixChar":")","numbering":"continue","numberCodeSuperscript":false}"#,
    )
    .expect("digit shape");
    assert_eq!(numbers(&core), [1, 2, 3]);
    assert_eq!(markers(&core), [vec!["1)", "2)"], vec!["3)"]]);

    core.apply_footnote_shape_native(0, r#"{"numbering":"restartSection","startNumber":5}"#)
        .expect("start number");
    assert_eq!(numbers(&core), [5, 6, 7]);
    assert_eq!(shape(&core)["startNumber"], 5);

    // 한/글에서 정한 모양으로 되돌리면 번호·저장본이 한/글과 같다.
    core.apply_footnote_shape_native(
        0,
        r#"{"numberFormat":"circledDigit","prefixChar":"[","suffixChar":"]","numbering":"restartPage","startNumber":1,"numberCodeSuperscript":true}"#,
    )
    .expect("hancom shape");
    assert_eq!(numbers(&core), [1, 2, 1]);
    assert_eq!(markers(&core), [vec!["[①]", "[②]"], vec!["[①]"]]);

    let hwpx = core.export_hwpx_native().expect("hwpx");
    let saved = footnote_pr(&hwpx);
    for part in HANCOM_FOOTNOTE_PR {
        assert!(saved.contains(part), "{part} 없음: {saved}");
    }
    let expected = shape(&core);
    for bytes in [hwpx, core.export_hwp_native().expect("hwp")] {
        let reopened = open(&bytes);
        assert_eq!(shape(&reopened), expected);
        assert_eq!(numbers(&reopened), [1, 2, 1]);
    }
}

#[test]
fn separator_spacing_and_placement_round_trip() {
    let mut core = sample();
    core.apply_footnote_shape_native(
        0,
        r##"{"separatorLength":5669,"separatorLineType":1,"separatorLineWidth":1,"separatorColor":"#ff0000","separatorMarginTop":850,"separatorMarginBottom":567,"noteSpacing":283,"printInlineAfterText":false,"placement":"mergedColumn"}"##,
    )
    .expect("separator");
    let applied = shape(&core);
    for (key, expected) in [
        ("separatorLength", json!(5669)),
        ("separatorColor", json!("#ff0000")),
        ("separatorMarginTop", json!(850)),
        ("separatorMarginBottom", json!(567)),
        ("noteSpacing", json!(283)),
        ("printInlineAfterText", json!(false)),
        ("placement", json!("mergedColumn")),
    ] {
        assert_eq!(applied[key], expected, "{key}: {applied}");
    }
    for bytes in [
        core.export_hwpx_native().expect("hwpx"),
        core.export_hwp_native().expect("hwp"),
    ] {
        assert_eq!(shape(&open(&bytes)), applied);
    }
}

#[test]
fn new_footnotes_follow_the_section_shape() {
    let mut core = sample();
    // 2쪽 문단 끝에 각주를 하나 더 넣으면 그 쪽의 두 번째 각주 [②]가 된다.
    let para = core.document().sections[0].paragraphs.len() - 1;
    let end = core.document().sections[0].paragraphs[para]
        .text
        .chars()
        .count();
    core.insert_footnote_native(0, para, end).expect("insert");
    assert_eq!(numbers(&core), [1, 2, 1, 2]);
    assert_eq!(markers(&core), [vec!["[①]", "[②]"], vec!["[①]", "[②]"]]);
}

#[test]
fn invalid_input_leaves_the_shape_unchanged() {
    let mut core = sample();
    let before = shape(&core);
    for props in [
        r#"{"numbering":"weekly"}"#,
        r#"{"numberFormat":"emoji","prefixChar":"("}"#,
        r#"{"placement":"documentEnd"}"#,
        r#"{"separatorColor":"red"}"#,
        "not json",
    ] {
        assert!(
            core.apply_footnote_shape_native(0, props).is_err(),
            "{props}"
        );
        assert_eq!(shape(&core), before, "{props}");
    }
    assert!(core.get_footnote_shape_native(9).is_err());
}
