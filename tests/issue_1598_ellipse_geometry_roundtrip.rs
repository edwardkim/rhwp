//! Task #1598 — HWPX ellipse/arc 전용 지오메트리(center/축/시작끝점) roundtrip 보존.
//!
//! HWPX 파서가 `<hc:center>/<hc:ax1>/...` 를 읽지 않고 직렬화도 드롭하던 결함
//! (#1589 잔여 페이지 붕괴의 근본)을 회귀 차단한다. IR diff 게이트는 ellipse 지오메트리를
//! 비교하지 않으므로(IR-invisible) 본 전용 테스트가 유일한 자동 게이트다.
//!
//! 통제 검증(한글 오라클): 36385226 은 지오메트리 미방출 시 3→2 붕괴, 방출 시 3 유지.

use rhwp::model::shape::ShapeObject;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

/// 문서 전체 ellipse 의 (center, ax1, ax2, start1, end1, start2, end2) 좌표를 순서대로 수집.
/// Point 는 PartialEq 미구현이라 (x, y) 튜플 배열로 환산.
fn collect_ellipse_geoms(doc: &rhwp::model::document::Document) -> Vec<[(i32, i32); 7]> {
    fn visit(p: &rhwp::model::paragraph::Paragraph, out: &mut Vec<[(i32, i32); 7]>) {
        for c in &p.controls {
            match c {
                rhwp::model::control::Control::Shape(s) => {
                    if let ShapeObject::Ellipse(e) = s.as_ref() {
                        out.push([
                            (e.center.x, e.center.y),
                            (e.axis1.x, e.axis1.y),
                            (e.axis2.x, e.axis2.y),
                            (e.start1.x, e.start1.y),
                            (e.end1.x, e.end1.y),
                            (e.start2.x, e.start2.y),
                            (e.end2.x, e.end2.y),
                        ]);
                    }
                }
                rhwp::model::control::Control::Table(t) => {
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

#[test]
fn ellipse_geometry_roundtrips() {
    let path = "samples/hwpx/opengov/36385226_결재문서본문_제2처리장 슬러지인발용 에어리프트 브로워 2호기 소모품 교체 보고.hwpx";
    let bytes = std::fs::read(path).expect("샘플 읽기");

    let doc1 = parse_hwpx(&bytes).expect("parse 원본");
    let g1 = collect_ellipse_geoms(&doc1);
    assert!(
        g1.len() >= 9,
        "원본 ellipse {}개 (>=9 기대) — 파서가 ellipse 를 적재해야 함",
        g1.len()
    );
    // 파서가 전용 지오메트리를 실제로 읽었는가 (모두 0 이면 드롭된 것 = RED).
    let nonzero = g1
        .iter()
        .any(|geo| geo.iter().any(|&(x, y)| x != 0 || y != 0));
    assert!(
        nonzero,
        "ellipse 전용 지오메트리(center/축/시작끝점)가 전부 0 — 파서 드롭(#1598 미수정)"
    );

    // round 1: serialize → reparse → 지오메트리 보존.
    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");
    let g2 = collect_ellipse_geoms(&doc2);
    assert_eq!(g1, g2, "round1 ellipse 지오메트리 보존 실패(직렬화 드롭)");

    // 2-round 안정.
    let out2 = serialize_hwpx(&doc2).expect("serialize r2");
    let doc3 = parse_hwpx(&out2).expect("reparse r2");
    let g3 = collect_ellipse_geoms(&doc3);
    assert_eq!(g2, g3, "2-round ellipse 지오메트리 안정 실패");
}
