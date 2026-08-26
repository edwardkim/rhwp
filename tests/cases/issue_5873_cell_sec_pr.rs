//! [Issue #5873] 표 셀 안 구역 나누기(`secd`)를 HWPX 저장에서 통째로 버린다.
//!
//! `render_runs` 는 `SectionDef` 슬롯을 hidden 처리해 XML 을 내지 않는다. 본문 최상위
//! 문단은 #4056 이 `write_section` 루프에서 `build_secpr_run` 으로 보완해 왔지만
//! subList(셀) 경로에는 그 보완이 없어, 셀 안 구역이 `hp:secPr` 없이 뒤따르는
//! `hp:colPr` 만 남았다.
//!
//! 실물 피해: 코퍼스 HWP5 6,491문서를 레코드 단위로 전수해 보면 BodyText 스트림 수보다
//! `secd` 가 많은 문서가 3건이고 그 17개가 전부 표 셀 안(level 3)이다. 그중 06874 는
//! 셀 안 `secd` 15개를 잃어 한글이 `부록 4-④` 문단부터 문서 끝까지 폐기했다
//! (204→159쪽, −57,525자). 한글 2022 오라클 주입 검정에서 그 자리 **한 곳에만**
//! `secPr` 를 되살리면 쪽수·글자수가 원본과 정확히 같아졌다.
//!
//! 계약: 셀 문단이 `Control::SectionDef` 를 들고 있으면 그 셀의 `hp:subList` 안에
//! `hp:secPr` 가 나와야 한다. 본문 최상위 구역(#4056)은 종전대로 유지된다.

use std::io::Read;

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section, SectionDef};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{CharShape, ParaShape};
use rhwp::model::table::{Cell, Table};

fn secdef() -> SectionDef {
    SectionDef {
        page_def: PageDef {
            width: 59528,
            height: 84188,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn text_para(text: &str) -> Paragraph {
    Paragraph {
        text: text.to_string(),
        char_count: text.chars().count() as u32,
        ..Default::default()
    }
}

/// 표 한 개, 둘째 셀의 첫 문단이 구역 나누기를 든 최소 문서.
fn cell_sec_pr_document() -> Document {
    let head = Cell {
        col: 0,
        row: 0,
        col_span: 1,
        row_span: 1,
        width: 20000,
        paragraphs: vec![text_para("검토 의견")],
        ..Default::default()
    };

    let mut body_para = text_para("체계적으로 잘 정돈되어 있습니다.");
    body_para
        .controls
        .insert(0, Control::SectionDef(Box::new(secdef())));
    let body = Cell {
        col: 0,
        row: 1,
        col_span: 1,
        row_span: 1,
        width: 20000,
        paragraphs: vec![body_para],
        ..Default::default()
    };

    let table = Table {
        col_count: 1,
        row_count: 2,
        cells: vec![head, body],
        ..Default::default()
    };

    let mut owner = text_para("부록");
    owner.controls.push(Control::Table(Box::new(table)));

    let mut section = Section::default();
    section.section_def = secdef();
    section.paragraphs.push(text_para("본문 첫 문단"));
    section.paragraphs.push(owner);

    let mut doc = Document::default();
    // 직렬화기가 참조 무결성을 검사한다 — 문단/글자 모양 0번을 등록한다.
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.doc_info.char_shapes = vec![CharShape::default()];
    doc.doc_properties.section_count = 1;
    doc.sections.push(section);
    doc
}

/// 저장된 HWPX 의 `Contents/section*.xml` 을 모두 이어 붙인다.
fn section_xml(doc: &Document) -> String {
    let bytes = rhwp::serializer::hwpx::serialize_hwpx(doc).expect("serialize hwpx");
    let mut zin = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip 열기");
    let mut out = String::new();
    for i in 0..zin.len() {
        let mut f = zin.by_index(i).expect("zip 항목");
        let name = f.name().to_string();
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            let mut s = String::new();
            f.read_to_string(&mut s).expect("section xml 읽기");
            out.push_str(&s);
        }
    }
    out
}

/// 셀 안 구역 나누기가 `hp:subList` 안의 `hp:secPr` 로 나와야 한다.
#[test]
fn cell_section_def_is_emitted_as_sec_pr() {
    let xml = section_xml(&cell_sec_pr_document());

    // 구역 정의는 두 개다 — 구역 첫 문단(템플릿)과 셀 안 하나.
    assert_eq!(
        xml.matches("<hp:secPr ").count(),
        2,
        "셀 안 SectionDef 가 secPr 로 나오지 않았다 (#5873 회귀)\n{xml}"
    );

    // 그 두 번째 secPr 은 subList 안에 있어야 한다.
    let second = xml
        .match_indices("<hp:secPr ")
        .nth(1)
        .map(|(i, _)| i)
        .expect("두 번째 secPr");
    let before = &xml[..second];
    let depth = before.matches("<hp:subList").count() - before.matches("</hp:subList>").count();
    assert!(
        depth > 0,
        "셀 안 secPr 이 subList 밖으로 나갔다 (depth={depth}) — 셀 구역이 본문 구역이 된다"
    );
}

/// 셀에 구역 나누기가 없으면 종전대로 구역 정의는 하나뿐이다.
#[test]
fn plain_cell_keeps_a_single_sec_pr() {
    let mut doc = cell_sec_pr_document();
    for section in doc.sections.iter_mut() {
        for para in section.paragraphs.iter_mut() {
            for ctrl in para.controls.iter_mut() {
                if let Control::Table(t) = ctrl {
                    for cell in t.cells.iter_mut() {
                        for p in cell.paragraphs.iter_mut() {
                            p.controls.retain(|c| !matches!(c, Control::SectionDef(_)));
                        }
                    }
                }
            }
        }
    }
    let xml = section_xml(&doc);
    assert_eq!(
        xml.matches("<hp:secPr ").count(),
        1,
        "구역 나누기가 없는 셀에서 secPr 이 늘었다"
    );
}
