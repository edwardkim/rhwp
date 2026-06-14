//! #1403 회귀 — 그림/도형/묶음 캡션(hp:caption)이 HWPX roundtrip 에서 보존되는지 검증.
//!
//! aift.hwpx section2 실측(코퍼스 전수 중 비표 캡션 보유 유일 샘플):
//! pic 캡션 9건 + container(묶음) 캡션 2건. 수정 전에는 파서가 캡션을 적재하지
//! 않고 serializer 도 방출하지 않아 roundtrip 후 전량 소실되었다.

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{Caption, ShapeObject};
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

fn caption_text(cap: &Caption) -> String {
    cap.paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("")
}

/// (종류, 캡션 텍스트) 수집 — 본문/표 셀/글상자/묶음 자식까지 재귀.
fn collect_object_captions(doc: &Document) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    for section in &doc.sections {
        for para in &section.paragraphs {
            walk_para(para, &mut out);
        }
    }
    out
}

fn walk_para(para: &Paragraph, out: &mut Vec<(&'static str, String)>) {
    for ctrl in &para.controls {
        match ctrl {
            Control::Table(t) => {
                for cell in &t.cells {
                    for p in &cell.paragraphs {
                        walk_para(p, out);
                    }
                }
            }
            Control::Picture(pic) => {
                if let Some(cap) = &pic.caption {
                    out.push(("pic", caption_text(cap)));
                }
            }
            Control::Shape(shape) => walk_shape(shape, out),
            _ => {}
        }
    }
}

fn walk_shape(shape: &ShapeObject, out: &mut Vec<(&'static str, String)>) {
    let (kind, cap): (&'static str, &Option<Caption>) = match shape {
        ShapeObject::Line(x) => ("shape", &x.drawing.caption),
        ShapeObject::Rectangle(x) => ("shape", &x.drawing.caption),
        ShapeObject::Ellipse(x) => ("shape", &x.drawing.caption),
        ShapeObject::Arc(x) => ("shape", &x.drawing.caption),
        ShapeObject::Polygon(x) => ("shape", &x.drawing.caption),
        ShapeObject::Curve(x) => ("shape", &x.drawing.caption),
        ShapeObject::Chart(x) => ("shape", &x.caption),
        ShapeObject::Ole(x) => ("shape", &x.caption),
        ShapeObject::Group(x) => ("group", &x.caption),
        ShapeObject::Picture(x) => ("pic", &x.caption),
    };
    if let Some(c) = cap {
        out.push((kind, caption_text(c)));
    }
    if let Some(tb) = shape_text_box(shape) {
        for p in &tb.paragraphs {
            walk_para(p, out);
        }
    }
    if let ShapeObject::Group(g) = shape {
        for child in &g.children {
            walk_shape(child, out);
        }
    }
}

fn shape_text_box(s: &ShapeObject) -> Option<&rhwp::model::shape::TextBox> {
    match s {
        ShapeObject::Line(x) => x.drawing.text_box.as_ref(),
        ShapeObject::Rectangle(x) => x.drawing.text_box.as_ref(),
        ShapeObject::Ellipse(x) => x.drawing.text_box.as_ref(),
        ShapeObject::Arc(x) => x.drawing.text_box.as_ref(),
        ShapeObject::Polygon(x) => x.drawing.text_box.as_ref(),
        ShapeObject::Curve(x) => x.drawing.text_box.as_ref(),
        ShapeObject::Chart(x) => x.drawing.text_box.as_ref(),
        ShapeObject::Ole(x) => x.drawing.text_box.as_ref(),
        ShapeObject::Group(_) | ShapeObject::Picture(_) => None,
    }
}

fn count_kind(caps: &[(&'static str, String)], kind: &str) -> usize {
    caps.iter().filter(|(k, _)| *k == kind).count()
}

#[test]
fn issue_1403_captions_roundtrip_aift() {
    let bytes = std::fs::read("samples/hwpx/aift.hwpx").expect("샘플 읽기");
    let doc1 = parse_hwpx(&bytes).expect("parse 원본");

    // 1) 파서 적재 — section2 실측 분포: pic 9 + container 2
    let caps1 = collect_object_captions(&doc1);
    assert_eq!(count_kind(&caps1, "pic"), 9, "pic 캡션 적재: {caps1:?}");
    assert_eq!(count_kind(&caps1, "group"), 2, "묶음 캡션 적재: {caps1:?}");
    let texts1: Vec<&str> = caps1.iter().map(|(_, t)| t.as_str()).collect();
    assert!(
        texts1
            .iter()
            .any(|t| t.contains("주관기관이 보유한 오케스트레이션 아키텍처")),
        "실물 pic 캡션 텍스트 적재: {texts1:?}"
    );
    assert!(
        texts1
            .iter()
            .any(|t| t.contains("서울소방재난본부 인공지능 건축허가 시스템 화면")),
        "실물 묶음 캡션 텍스트 적재: {texts1:?}"
    );

    // 2) roundtrip 보존 — serialize → reparse 후 종류별 개수 + 텍스트 multiset 동일
    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");
    let caps2 = collect_object_captions(&doc2);
    assert_eq!(count_kind(&caps2, "pic"), 9, "pic 캡션 보존: {caps2:?}");
    assert_eq!(count_kind(&caps2, "group"), 2, "묶음 캡션 보존: {caps2:?}");
    let mut t1: Vec<String> = caps1.iter().map(|(_, t)| t.clone()).collect();
    let mut t2: Vec<String> = caps2.iter().map(|(_, t)| t.clone()).collect();
    t1.sort();
    t2.sort();
    assert_eq!(t1, t2, "캡션 텍스트 보존");

    // 3) 2-round 안정성 — 재직렬화 후에도 동일
    let out2 = serialize_hwpx(&doc2).expect("2-round serialize");
    let doc3 = parse_hwpx(&out2).expect("2-round reparse");
    let caps3 = collect_object_captions(&doc3);
    assert_eq!(caps2.len(), caps3.len(), "2-round 캡션 개수 안정");
}
