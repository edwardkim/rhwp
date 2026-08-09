//! Issue #4388 — `ArcShape.arc_type` 이 HWPX 경로에서 경고 없이 유실되는 결함의 회귀 테스트.
//!
//! `src/renderer/layout/shape_layout.rs` 는 `arc.arc_type` 값에 따라 완전히 다른 경로를
//! 그린다 (0: 열린 호, 1: 부채꼴/PIE, 2: 활/CHORD — `PathCommand` 시퀀스 자체가 갈린다).
//! 그런데 HWPX 파서는 `<hp:arc>` 요소의 `type` 속성을 읽지 않아(주석: "arc_type 은
//! 태그속성(추후)") 항상 0(NORMAL/열린 호)으로 고정됐고, 직렬화기도 이 속성 자체를
//! 방출하지 않았다 — 왕복마다 원본이 PIE/CHORD 였어도 조용히 열린 호로 바뀐다.
//!
//! 실제 OWPML 모델(hancom-io/hwpx-owpml-model, Apache-2.0, `Class/Para/ArcType.cpp`
//! `CArcType::WriteElement`)을 확인한 결과 속성명은 `arcType` 이 아니라 `type` 이다
//! (`arcType` 은 `<hp:ellipse>` 자신의 동명 속성으로, 서로 다른 요소의 별개 속성이다).
//! `g_ArcTypeList`: NORMAL=0, PIE=1, CHORD=2 — `ArcShape.arc_type` (0: Arc, 1:
//! CircularSector, 2: Bow) 와 1:1 대응.
//!
//! samples/ 안의 279개 실제 HWPX 표본에는 `<hp:arc>` 요소가 하나도 없어(전부 `<hp:ellipse
//! arcType="NORMAL">` 뿐) 실측 fixture 로 회귀를 걸 수 없다 — 합성 IR 로 왕복을 검증한다.

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::model::shape::{ArcShape, CommonObjAttr, DrawingObjAttr, ShapeObject};
use rhwp::model::style::CharShape;
use rhwp::model::Point;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

fn doc_with_arc(arc_type: u8) -> Document {
    let arc = ArcShape {
        common: CommonObjAttr {
            width: 4000,
            height: 3000,
            ..Default::default()
        },
        drawing: DrawingObjAttr::default(),
        arc_type,
        center: Point { x: 2000, y: 1500 },
        axis1: Point { x: 4000, y: 1500 },
        axis2: Point { x: 2000, y: 0 },
    };
    let mut para = Paragraph::default();
    para.text = "\u{fffc}".to_string();
    para.char_offsets = vec![0];
    para.char_count = 9; // 8유닛 오브젝트 슬롯 + null 종단 관례(다른 shape 테스트와 동형)
                         // char_shape_id=0 을 명시 등록한다 — 그렇지 않으면 1차 직렬화는 암묵적 fallback
                         // 으로 통과하지만(#1592 관례), 재파싱된 문단이 char_shapes=[(0,0)] 를 얻은 뒤
                         // 2차 직렬화에서 "미등록 ID 참조" 로 실패해 왕복 안정성 확인이 막힌다.
    para.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    para.controls
        .push(Control::Shape(Box::new(ShapeObject::Arc(arc))));

    let mut section = Section::default();
    section.paragraphs.push(para);
    let mut doc = Document::default();
    doc.doc_info.char_shapes.push(CharShape::default());
    doc.sections.push(section);
    doc
}

/// 첫 `<hp:arc ...>` 여는 태그의 `type` 속성값 추출 (section*.xml 전수 검색).
fn extract_arc_type_attr(hwpx_bytes: &[u8]) -> Option<String> {
    let reader = std::io::Cursor::new(hwpx_bytes);
    let mut zip = zip::ZipArchive::new(reader).ok()?;
    use std::io::Read;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).ok()?;
        if !f.name().ends_with(".xml") || !f.name().contains("section") {
            continue;
        }
        let mut xml = String::new();
        f.read_to_string(&mut xml).ok()?;
        if let Some(pos) = xml.find("<hp:arc ") {
            let tag_end = xml[pos..].find('>').map(|e| pos + e)?;
            let tag = &xml[pos..tag_end];
            let key = "type=\"";
            let s = tag.find(key)? + key.len();
            let rest = &tag[s..];
            let e = rest.find('"')?;
            return Some(rest[..e].to_string());
        }
    }
    None
}

fn first_arc(doc: &Document) -> Option<&ArcShape> {
    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Shape(s) = ctrl {
                    if let ShapeObject::Arc(a) = s.as_ref() {
                        return Some(a);
                    }
                }
            }
        }
    }
    None
}

#[test]
fn arc_type_normal_roundtrips_and_uses_type_attribute() {
    let doc = doc_with_arc(0);
    let bytes = serialize_hwpx(&doc).expect("serialize arc(NORMAL)");
    let attr = extract_arc_type_attr(&bytes);
    assert_eq!(
        attr.as_deref(),
        Some("NORMAL"),
        "<hp:arc type=\"NORMAL\"> 이 방출되어야 함 (수정 전엔 type 속성 자체가 없었음)"
    );
    let doc2 = parse_hwpx(&bytes).expect("reparse");
    let arc = first_arc(&doc2).expect("arc control survives");
    assert_eq!(arc.arc_type, 0, "NORMAL(0) 왕복 보존");
}

#[test]
fn arc_type_pie_roundtrips_and_uses_type_attribute() {
    // arc_type=1 (CircularSector/부채꼴) — OWPML PIE.
    let doc = doc_with_arc(1);
    let bytes = serialize_hwpx(&doc).expect("serialize arc(PIE)");
    let attr = extract_arc_type_attr(&bytes);
    assert_eq!(
        attr.as_deref(),
        Some("PIE"),
        "<hp:arc type=\"PIE\"> 로 방출되어야 함 — arcType 이 아니라 type 속성"
    );
    let doc2 = parse_hwpx(&bytes).expect("reparse");
    let arc = first_arc(&doc2).expect("arc control survives");
    assert_eq!(
        arc.arc_type, 1,
        "PIE(1) 왕복 보존 — 수정 전엔 파서가 type 속성을 읽지 않아 항상 0 으로 되돌아왔음"
    );
}

#[test]
fn arc_type_chord_roundtrips_and_uses_type_attribute() {
    // arc_type=2 (Bow/활) — OWPML CHORD.
    let doc = doc_with_arc(2);
    let bytes = serialize_hwpx(&doc).expect("serialize arc(CHORD)");
    let attr = extract_arc_type_attr(&bytes);
    assert_eq!(
        attr.as_deref(),
        Some("CHORD"),
        "<hp:arc type=\"CHORD\"> 로 방출되어야 함"
    );
    let doc2 = parse_hwpx(&bytes).expect("reparse");
    let arc = first_arc(&doc2).expect("arc control survives");
    assert_eq!(arc.arc_type, 2, "CHORD(2) 왕복 보존");

    // 2-round 안정성 (geometry roundtrip 테스트(#1598)와 동형 관례).
    let bytes2 = serialize_hwpx(&doc2).expect("serialize r2");
    let doc3 = parse_hwpx(&bytes2).expect("reparse r2");
    let arc3 = first_arc(&doc3).expect("arc control survives r2");
    assert_eq!(arc3.arc_type, 2, "2-round CHORD 안정");
}

/// center/axis 지오메트리는 (#1598 로) 이미 왕복하지만, arc_type 이 항상 0 으로
/// 뭉개지는 게 이 이슈의 핵심 증상이었다 — geometry 는 그대로인데 렌더 결과(열린
/// 호/부채꼴/활)만 달라지는 조용한 손실임을 명시적으로 못 박는다.
#[test]
fn arc_geometry_unaffected_by_arc_type_fix() {
    let doc = doc_with_arc(1);
    let bytes = serialize_hwpx(&doc).expect("serialize");
    let doc2 = parse_hwpx(&bytes).expect("reparse");
    let arc = first_arc(&doc2).expect("arc control survives");
    assert_eq!((arc.center.x, arc.center.y), (2000, 1500));
    assert_eq!((arc.axis1.x, arc.axis1.y), (4000, 1500));
    assert_eq!((arc.axis2.x, arc.axis2.y), (2000, 0));
}
