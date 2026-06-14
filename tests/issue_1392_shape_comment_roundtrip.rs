//! Task #1392 통합 — hp:shapeComment(객체 설명) roundtrip 보존.
//!
//! 4경로(pic/equation/rect/container) 전부 parse→serialize→재parse 후 description
//! 보존 + 2-round 안정. rect 는 #1392 이전부터 정상이었고, pic/equation/container 가
//! 본 타스크 해소 대상.

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

/// 문서 전체에서 (경로종류, description) 멀티셋을 수집한다.
fn collect_descriptions(doc: &rhwp::model::document::Document) -> Vec<(String, String)> {
    fn shape_kind_desc(s: &ShapeObject) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let (kind, desc) = match s {
            ShapeObject::Line(x) => ("line", &x.common.description),
            ShapeObject::Rectangle(x) => ("rect", &x.common.description),
            ShapeObject::Ellipse(x) => ("ellipse", &x.common.description),
            ShapeObject::Arc(x) => ("arc", &x.common.description),
            ShapeObject::Polygon(x) => ("polygon", &x.common.description),
            ShapeObject::Curve(x) => ("curve", &x.common.description),
            ShapeObject::Chart(x) => ("chart", &x.common.description),
            ShapeObject::Ole(x) => ("ole", &x.common.description),
            ShapeObject::Group(x) => ("group", &x.common.description),
            ShapeObject::Picture(x) => ("pic", &x.common.description),
        };
        if !desc.is_empty() {
            out.push((kind.to_string(), desc.clone()));
        }
        if let ShapeObject::Group(g) = s {
            for c in &g.children {
                out.extend(shape_kind_desc(c));
            }
        }
        out
    }
    fn visit(p: &rhwp::model::paragraph::Paragraph, out: &mut Vec<(String, String)>) {
        for c in &p.controls {
            match c {
                Control::Picture(pic) if !pic.common.description.is_empty() => {
                    out.push(("pic".into(), pic.common.description.clone()));
                }
                Control::Equation(eq) if !eq.common.description.is_empty() => {
                    out.push(("eq".into(), eq.common.description.clone()));
                }
                Control::Shape(s) => out.extend(shape_kind_desc(s)),
                Control::Table(t) => {
                    for cell in &t.cells {
                        for q in &cell.paragraphs {
                            visit(q, out);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    for s in &doc.sections {
        for p in &s.paragraphs {
            visit(p, &mut out);
        }
    }
    out
}

fn sorted(mut v: Vec<(String, String)>) -> Vec<(String, String)> {
    v.sort();
    v
}

fn assert_roundtrip_preserves(path: &str, min_count: usize) {
    let bytes = std::fs::read(path).expect("샘플 읽기");
    let doc1 = parse_hwpx(&bytes).expect("parse 원본");
    let d1 = collect_descriptions(&doc1);
    assert!(
        d1.len() >= min_count,
        "{path}: 원본 description {}건 (>= {min_count} 기대)",
        d1.len()
    );

    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");
    assert_eq!(
        sorted(d1.clone()),
        sorted(collect_descriptions(&doc2)),
        "{path}: round1 description 멀티셋 보존"
    );

    let out2 = serialize_hwpx(&doc2).expect("serialize r2");
    let doc3 = parse_hwpx(&out2).expect("reparse r2");
    assert_eq!(
        sorted(collect_descriptions(&doc2)),
        sorted(collect_descriptions(&doc3)),
        "{path}: 2-round 안정"
    );
}

#[test]
fn aift_pic_container_comment_roundtrips() {
    // pic 13 + container 2 = 15 (1단계 정밀 계수)
    assert_roundtrip_preserves("samples/hwpx/aift.hwpx", 13);
}

#[test]
fn math_001_equation_comment_roundtrips() {
    // equation 44 + pic 1
    assert_roundtrip_preserves("samples/hwpx/math-001.hwpx", 40);
}

#[test]
fn onsemiro_all_paths_comment_roundtrips() {
    // pic 31 + container 1 + rect 1 = 33
    assert_roundtrip_preserves("samples/hwpx/[2027] 온새미로 1 본교재.hwpx", 30);
}
