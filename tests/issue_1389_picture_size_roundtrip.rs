//! Task #1389 통합 — 그림 크기 요소(curSz/imgRect/imgDim) roundtrip 보존.

use rhwp::model::control::Control;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

fn collect_pics(
    doc: &rhwp::model::document::Document,
) -> Vec<((u32, u32), [i32; 4], [i32; 4], (u32, u32))> {
    // (curSz, border_x, border_y, img_dim)
    let mut out = Vec::new();
    fn visit(c: &Control, out: &mut Vec<((u32, u32), [i32; 4], [i32; 4], (u32, u32))>) {
        match c {
            Control::Picture(p) => out.push((
                (p.shape_attr.current_width, p.shape_attr.current_height),
                p.border_x,
                p.border_y,
                p.img_dim,
            )),
            Control::Table(t) => {
                for cell in &t.cells {
                    for q in &cell.paragraphs {
                        for cc in &q.controls {
                            visit(cc, out);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    for s in &doc.sections {
        for p in &s.paragraphs {
            for c in &p.controls {
                visit(c, &mut out);
            }
        }
    }
    out
}

#[test]
fn ta_pic_size_elements_roundtrip() {
    let bytes = std::fs::read("samples/hwpx/ta-pic-001-r.hwpx").expect("샘플 읽기");
    let doc1 = parse_hwpx(&bytes).expect("parse 원본");
    let pics = collect_pics(&doc1);
    assert!(!pics.is_empty(), "ta-pic 그림 존재");
    // pic0 셀 내 그림: curSz 13668×12686, imgDim 49380×45840 (1단계 실측)
    assert!(
        pics.iter()
            .any(|(cs, _, _, dim)| *cs == (13668, 12686) && *dim == (49380, 45840)),
        "원본 pic0 크기 적재: {pics:?}"
    );

    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");
    assert_eq!(pics, collect_pics(&doc2), "round1 그림 크기 요소 보존");

    let out2 = serialize_hwpx(&doc2).expect("serialize r2");
    let doc3 = parse_hwpx(&out2).expect("reparse r2");
    assert_eq!(collect_pics(&doc2), collect_pics(&doc3), "2-round 안정");
}
