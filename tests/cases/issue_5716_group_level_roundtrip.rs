//! Issue #5716: 중첩 그룹 컨테이너·그룹 멤버 OLE 의 `groupLevel` 이 HWPX 저장
//! 왕복에서 보존되어야 한다.
//!
//! Regression shape (samples/task1771/nested_group_vectors.hwpx, 컨테이너 223개·
//! 중첩 깊이 8):
//! - 수정 전: `write_container_open`/`write_ole` 이 groupLevel 을 "0" 으로
//!   하드코딩 → 컨테이너 groupLevel 분포(0~7)가 저장 시 전건 0 으로 붕괴
//!   (리프 도형 경로는 #2746 에서 이미 보존). hp:ole 은 파서 arm 도 없어
//!   HWPX 출신 그룹 멤버 OLE 는 파싱 단계에서부터 유실.
//! - 수정 후: 컨테이너·OLE 모두 IR(shape_attr.group_level) 보존 값을 되쓴다.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

const SAMPLE: &str = "samples/task1771/nested_group_vectors.hwpx";

/// 도형 트리를 돌며 (컨테이너 여부, group_level) 를 평탄화 수집한다.
fn collect_levels(shape: &ShapeObject, out: &mut Vec<(bool, u16)>) {
    let is_group = matches!(shape, ShapeObject::Group(_));
    out.push((is_group, shape.shape_attr().group_level));
    if let ShapeObject::Group(g) = shape {
        for child in &g.children {
            collect_levels(child, out);
        }
    }
}

fn doc_levels(doc: &Document) -> Vec<(bool, u16)> {
    let mut out = Vec::new();
    for section in &doc.sections {
        for para in &section.paragraphs {
            for c in &para.controls {
                if let Control::Shape(s) = c {
                    collect_levels(s, &mut out);
                }
            }
        }
    }
    out
}

fn load_sample() -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    parse_hwpx(&bytes).expect("parse hwpx")
}

#[test]
fn issue_5716_container_group_level_survives_hwpx_roundtrip() {
    let doc = load_sample();
    let original = doc_levels(&doc);
    let containers_nonzero = original
        .iter()
        .filter(|(is_group, lv)| *is_group && *lv > 0)
        .count();
    assert!(
        containers_nonzero > 100,
        "재현 전제: 중첩 컨테이너(groupLevel>0)가 다수여야 한다: {containers_nonzero}"
    );

    let out = serialize_hwpx(&doc).expect("serialize hwpx");
    let reparsed = parse_hwpx(&out).expect("reparse hwpx");
    let roundtripped = doc_levels(&reparsed);

    assert_eq!(
        roundtripped, original,
        "컨테이너·리프의 groupLevel 분포가 왕복에서 그대로 보존되어야 한다"
    );
}

#[test]
fn issue_5716_ole_group_level_survives_hwpx_roundtrip() {
    // 샘플의 hp:ole 은 최상위(groupLevel=0)뿐이라, 그룹 멤버 OLE 의 하드코딩·
    // 파서 arm 부재는 IR 에 비영 값을 실어 왕복시켜 검출한다.
    let mut doc = load_sample();
    let mut seeded = 0usize;
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            for c in &mut para.controls {
                if let Control::Shape(s) = c {
                    if let ShapeObject::Ole(ole) = s.as_mut() {
                        ole.drawing.shape_attr.group_level = 2;
                        seeded += 1;
                    }
                }
            }
        }
    }
    assert!(seeded > 0, "재현 전제: 샘플에 hp:ole 이 있어야 한다");

    let out = serialize_hwpx(&doc).expect("serialize hwpx");
    let reparsed = parse_hwpx(&out).expect("reparse hwpx");
    let mut preserved = 0usize;
    for section in &reparsed.sections {
        for para in &section.paragraphs {
            for c in &para.controls {
                if let Control::Shape(s) = c {
                    if let ShapeObject::Ole(ole) = s.as_ref() {
                        assert_eq!(
                            ole.drawing.shape_attr.group_level, 2,
                            "hp:ole groupLevel 이 왕복에서 보존되어야 한다"
                        );
                        preserved += 1;
                    }
                }
            }
        }
    }
    assert_eq!(preserved, seeded, "OLE 개수 보존");
}
