//! Issue #1771: 중첩 그룹 복합 벡터가 HWP5 라운드트립에서 보존되어야 한다.
//!
//! Regression shape (samples/task1771/nested_group_vectors.hwpx, 노면요철포장 지침):
//! - 수정 전: 중첩 그룹을 CONTAINER(0x56) 레코드만으로 직렬화 → 파서의 자식 경계
//!   (SHAPE_COMPONENT @ child_level) 미인식 → 재파스 children 710 → 12 대량 소실
//!   (렌더 Path 335→0, Group 215→3).
//! - 수정 후: 중첩 그룹도 SHAPE_COMPONENT('$con') 경계로 방출 → 710 전량 보존,
//!   render-diff --via hwp PASS (변위 0.00px).

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::parse_document;
use rhwp::serializer::serialize_hwp;

const SAMPLE: &str = "samples/task1771/nested_group_vectors.hwpx";

fn count_shapes(shape: &ShapeObject) -> usize {
    match shape {
        ShapeObject::Group(g) => 1 + g.children.iter().map(count_shapes).sum::<usize>(),
        _ => 1,
    }
}

fn doc_shape_count(doc: &rhwp::model::document::Document) -> usize {
    let mut n = 0;
    for section in &doc.sections {
        for para in &section.paragraphs {
            for c in &para.controls {
                if let Control::Shape(s) = c {
                    n += count_shapes(s);
                }
            }
        }
    }
    n
}

#[test]
fn issue_1771_nested_group_children_survive_roundtrip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = parse_document(&bytes).expect("parse hwpx");
    let original = doc_shape_count(&doc);
    assert!(
        original > 500,
        "재현 전제: 복합 벡터 문서(도형 {original}개 > 500)"
    );

    let hwp = serialize_hwp(&doc).expect("serialize hwp");
    let reloaded = parse_document(&hwp).expect("reload hwp");
    let roundtripped = doc_shape_count(&reloaded);

    assert_eq!(
        roundtripped, original,
        "중첩 그룹 자식이 라운드트립에서 전량 보존되어야 한다"
    );
}
