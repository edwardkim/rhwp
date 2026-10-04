//! Issue #4898 ③: HWPX 원본이 `hp:switch` 없이 평문으로 적은 문단 여백·줄간격은 그 표기 그대로
//! 되쓴다.
//!
//! 한글은 `hp:switch` 가 있으면 `hp:case`(HwpUnitChar) 를 우선 읽는다. 평문 원본을 switch 형태로
//! 바꿔 쓰면 고정 줄간격의 단위 해석이 달라져 조판이 밀릴 수 있다. 여백은 평문 HWPUNIT과
//! 패키지 xmlVersion1.4의 평문과 HwpUnitChar case는 공통 IR의 절반으로 출력한다.
//! 이전 패키지의 평문은 저장 IR 값을 그대로 출력한다.
//!
//! 한글 2022 오라클 10k 전수(x2x) 실측: 산출이 실제로 바뀌는 문서 290건을 전수 측정해 쪽수 결함
//! 23건이 원본 쪽수로 복귀했고 새로 깨진 문서는 0건이었다(당시 파서는 무변경)。
//! 후속 보정은 한컴 저장본으로 확인한 평문 여백 단위를 공통 IR로 정규화하고 재저장 값을 보존한다.

use rhwp::model::document::Document;
use rhwp::model::style::{LineSpacingType, ParaShape};
use rhwp::serializer::hwpx::serialize_hwpx;

const MARGIN_LEFT: i32 = 3000;
const LINE_SPACING_FIXED: i32 = 3560;

fn document_with_para_shape(plain: bool) -> Document {
    let mut doc = Document::default();
    let shape = ParaShape {
        margin_left: MARGIN_LEFT,
        line_spacing: LINE_SPACING_FIXED,
        line_spacing_type: LineSpacingType::Fixed,
        hwpx_plain_para_margin: plain,
        hwpx_plain_para_margin_physical: plain,
        ..Default::default()
    };
    doc.doc_info.para_shapes = vec![shape];
    doc
}

fn header_xml(hwpx: &[u8]) -> String {
    use std::io::Read;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("zip 열기 실패");
    let mut out = String::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).expect("zip 항목");
        if f.name() == "Contents/header.xml" {
            f.read_to_string(&mut out).expect("header.xml 읽기");
            break;
        }
    }
    assert!(!out.is_empty(), "header.xml 이 없다");
    out
}

fn para_pr_block(header: &str) -> String {
    let start = header.find("<hh:paraPr").expect("paraPr 없음");
    let end = header[start..]
        .find("</hh:paraPr>")
        .expect("paraPr 닫힘 없음");
    header[start..start + end].to_string()
}

#[test]
fn issue_4898_plain_source_keeps_plain_margin_notation() {
    for (version, physical) in [("1.2", false), ("1.3", false), ("1.4", true)] {
        let mut doc = document_with_para_shape(true);
        doc.doc_info.para_shapes[0].hwpx_plain_para_margin_physical = physical;
        // 같은 IR을 각 판본으로 저장할 때 실제 읽기 계약에 맞는 평문 값을 쓴다.
        doc.hwpx_aux_entries.push((
            "version.xml".into(),
            format!("<hv:HCFVersion xmlns:hv=\"http://www.hancom.co.kr/hwpml/2011/version\" xmlVersion=\"{version}\"/>").into_bytes(),
        ));
        let bytes = serialize_hwpx(&doc).expect("HWPX 직렬화 실패");
        let block = para_pr_block(&header_xml(&bytes));
        assert!(
            !block.contains("<hp:switch>"),
            "평문 원본의 고정 줄간격 계약은 switch 없이 보존해야 한다"
        );
        let stored_margin = if physical {
            MARGIN_LEFT / 2
        } else {
            MARGIN_LEFT
        };
        assert!(
            block.contains(&format!("<hc:left value=\"{stored_margin}\"")),
            "패키지 {version}의 평문 여백 단위 보존: {block}"
        );
        assert!(
            block.contains(&format!("value=\"{LINE_SPACING_FIXED}\"")),
            "고정 줄간격도 저장값 그대로여야 한다: {block}"
        );
        let parsed = rhwp::parser::parse_document(&bytes).expect("평문 여백 왕복 파싱");
        assert_eq!(parsed.doc_info.para_shapes[0].margin_left, MARGIN_LEFT);
        assert_eq!(
            parsed.doc_info.para_shapes[0].line_spacing,
            LINE_SPACING_FIXED
        );
    }
}

#[test]
fn issue_4898_switch_source_keeps_switch_notation() {
    let bytes = serialize_hwpx(&document_with_para_shape(false)).expect("HWPX 직렬화 실패");
    let block = para_pr_block(&header_xml(&bytes));

    assert!(
        block.contains("<hp:switch>"),
        "switch 원본은 종전대로 case/default 두 갈래로 써야 한다"
    );
    assert!(
        block.contains(&format!("<hc:left value=\"{}\"", MARGIN_LEFT / 2)),
        "case 갈래는 HwpUnitChar 1× 스케일(저장값의 절반)이다: {block}"
    );
    assert!(
        block.contains(&format!("<hc:left value=\"{MARGIN_LEFT}\"")),
        "default 갈래는 저장값 그대로다: {block}"
    );
}
