//! Issue #1916: raw_ctrl_data 미보존 HWP5 표의 CommonObjAttr 왕복 소실 회귀 가드.
//!
//! 일부 HWP5 원본(정책연구·보도자료 생성기 계열)은 표 CTRL_HEADER 의 raw 캡처가
//! 비는데, 종전 직렬화기는 이때 빈 ctrl data 를 써서 재로드 시 treat_as_char/
//! flowWithText/wrap 등 개체 공통속성 전체가 디폴트로 소실됐다 (hwpdocs 10k 서베이
//! 12건 — IR diff 는 flowWithText 축으로 발현). 수정: raw 부재 시
//! serialize_common_obj_attr(&table.common) 로 재구성 (hwpx_to_hwp 어댑터 Stage 2
//! 합성과 동일 계약).
//!
//! fixture: `samples/hwp5-tbl-attr-1916.hwp` (관세청 별지 서식, 20KB) —
//! s0 p2 c0 표가 flow_with_text=true + treat_as_char=true, raw_ctrl_data 비어 있음.

use rhwp::model::control::Control;
use rhwp::parser::parse_document;
use rhwp::serializer::serialize_document;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/hwp5-tbl-attr-1916.hwp";

fn collect_tables(doc: &rhwp::model::document::Document) -> Vec<(usize, usize, bool, bool, u32)> {
    let mut v = Vec::new();
    for (pi, para) in doc.sections[0].paragraphs.iter().enumerate() {
        for (ci, c) in para.controls.iter().enumerate() {
            if let Control::Table(t) = c {
                v.push((
                    pi,
                    ci,
                    t.common.flow_with_text,
                    t.common.treat_as_char,
                    t.common.attr,
                ));
            }
        }
    }
    v
}

#[test]
fn table_common_attr_survives_hwp5_roundtrip_without_raw() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&p).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc1 = parse_document(&bytes).unwrap_or_else(|e| panic!("파싱: {e:?}"));
    let t1 = collect_tables(&doc1);
    assert!(
        t1.iter().any(|&(_, _, flow, tac, _)| flow && tac),
        "전제: flow_with_text=true + treat_as_char=true 표 존재 (원본 파스): {t1:?}",
    );

    let out = serialize_document(&doc1).unwrap_or_else(|e| panic!("직렬화: {e:?}"));
    let doc2 = parse_document(&out).unwrap_or_else(|e| panic!("재파싱: {e:?}"));
    let t2 = collect_tables(&doc2);

    assert_eq!(t1.len(), t2.len(), "#1916: 표 개수 변동");
    for (a, b) in t1.iter().zip(t2.iter()) {
        // 수정 전: raw 부재 표는 재파스에서 flow=false, tac=false, attr=0 으로 확실히 구분.
        assert_eq!(a.2, b.2, "#1916: p{} c{} flow_with_text 소실", a.0, a.1);
        assert_eq!(a.3, b.3, "#1916: p{} c{} treat_as_char 소실", a.0, a.1);
    }
}
