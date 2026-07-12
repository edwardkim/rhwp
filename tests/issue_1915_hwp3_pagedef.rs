//! Issue #1915: HWP3-origin 문서를 HWP5 로 저장 시 page_def(용지·여백) 소실 회귀 가드.
//!
//! HWP3 파서는 `section.section_def` 만 채우고 첫 문단에 `Control::SectionDef` 를
//! 넣지 않는다. HWP5 직렬화기는 이 컨트롤을 만나야 secd/PAGE_DEF 레코드를 출력하므로
//! 종전에는 재조립본 재로드 시 용지·여백이 전부 0 이 됐다 (hwpdocs 10k 서베이 41건,
//! 전부 v3.0.0.0). 수정: serialize_section 이 첫 문단에 SectionDef 컨트롤이 없으면
//! section_def 로 보강해 직렬화한다 (hwpx_to_hwp 어댑터의 기존 보강과 동일 계약).
//!
//! fixture: `samples/hwp3-pagedef-1915.hwp` (행정정보공개목록 별지 서식, 2.4KB,
//! `HWP Document File V3.00` 시그니처).

use rhwp::parser::parse_document;
use rhwp::serializer::serialize_document;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/hwp3-pagedef-1915.hwp";

#[test]
fn hwp3_origin_hwp5_save_preserves_page_def() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&p).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc1 = parse_document(&bytes).unwrap_or_else(|e| panic!("HWP3 파싱: {e:?}"));

    let pd1 = &doc1.sections[0].section_def.page_def;
    assert!(
        pd1.width > 0 && pd1.height > 0,
        "전제: HWP3 파서가 page_def 를 채워야 한다 (width={}, height={})",
        pd1.width,
        pd1.height,
    );

    let out = serialize_document(&doc1).unwrap_or_else(|e| panic!("HWP5 직렬화: {e:?}"));
    let doc2 = parse_document(&out).unwrap_or_else(|e| panic!("재파싱: {e:?}"));
    let pd2 = &doc2.sections[0].section_def.page_def;

    // 수정 전: pd2 전 필드 0 (용지 0×0) 으로 확실히 구분된다.
    assert_eq!(pd1.width, pd2.width, "#1915: page_def.width 소실");
    assert_eq!(pd1.height, pd2.height, "#1915: page_def.height 소실");
    assert_eq!(pd1.margin_left, pd2.margin_left, "#1915: margin_left 소실");
    assert_eq!(
        pd1.margin_right, pd2.margin_right,
        "#1915: margin_right 소실"
    );
    assert_eq!(pd1.margin_top, pd2.margin_top, "#1915: margin_top 소실");
    assert_eq!(
        pd1.margin_bottom, pd2.margin_bottom,
        "#1915: margin_bottom 소실"
    );
    assert_eq!(
        pd1.margin_header, pd2.margin_header,
        "#1915: margin_header 소실"
    );
    assert_eq!(
        pd1.margin_footer, pd2.margin_footer,
        "#1915: margin_footer 소실"
    );
    assert_eq!(pd1.landscape, pd2.landscape, "#1915: landscape 소실");
}
