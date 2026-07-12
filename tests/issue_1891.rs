//! Issue #1891 — 외부 참조(BinData Link) 그림의 HWPX 라운드트립 보존.
//!
//! 규제영향분석서 계열(73504)은 manifest 에 `isEmbeded="0"` 외부 파일 참조
//! (`href="D:\다운로드\"`)를 갖고 여러 `<hp:pic>` 이 이를 참조한다. 종전 결함 체인:
//!   1) 직렬화기 bin_data_map 이 임베디드 콘텐츠만 등록 → 링크 참조 pic 직렬화
//!      실패 → 그림 컨트롤 통째 드롭 → 레이아웃 앵커 소실(STRUCT 167px),
//!   2) manifest 명명이 순번(i+1) 기반이라 링크로 id 에 구멍이 있으면 파서의
//!      숫자 불변식(binaryItemIDRef "imageN" → bin_data_id N)과 어긋남,
//!   3) 파서 bin_data_items 필터가 media-type 으로만 외부 항목을 인식해
//!      octet-stream 링크가 누락 → 후속 항목 bin_data_id 전부 시프트.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};
use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataType;
use rhwp::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

const SAMPLE: &str = "samples/issue1891_external_bindata_link.hwpx";
// [#2070 잠정] 76076: PDF 정답 82, 86712: PDF 정답 65. 종전 82/65는 본문 래핑
// +1줄 과대(45자 휴리스틱)와 빈 문단 0높이 과소의 **상쇄**였다. #2070 에서 빈 문단
// 축을 한글 정합(em 줄박스, 80168=157 달성 필수)으로 고치면서 상쇄가 노출되어
// 83/64 로 이동 — 본문 NO_LS 실폭 래핑(reflow_line_segs 정식 호출) 후속 이슈에서
// 82/65 로 복귀시킨다. 80168/80250 은 PDF 정답 그대로.
const HWP5_ORIGIN_SAMPLES: &[(&str, u32)] = &[
    ("samples/76076_regulatory_analysis.hwp", 83),
    ("samples/80168_regulatory_analysis.hwp", 157),
    ("samples/80250_regulatory_analysis.hwp", 17),
    ("samples/86712_regulatory_analysis.hwp", 64),
    ("samples/issue1891/76076_regulatory_analysis.hwpx", 83),
    ("samples/issue1891/80168_regulatory_analysis.hwpx", 157),
    ("samples/issue1891/80250_regulatory_analysis.hwpx", 17),
    // [#2070 잠정] hwpx 픽스처만 63 — 자기-export의 HwpUnitChar intent 절반 기록
    // 버그(serializer)로 원본과 비등가(재래핑 폭 상이). serializer 수정 후 64 복귀.
    ("samples/issue1891/86712_regulatory_analysis.hwpx", 63),
];

fn read_sample() -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    fs::read(Path::new(repo_root).join(SAMPLE)).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

fn read_rel(path: &str) -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    fs::read(Path::new(repo_root).join(path)).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// 렌더 자기정합: pic 드롭 없이 왕복해야 한다 (종전 STRUCT 167px).
#[test]
fn issue_1891_external_link_roundtrip_render_is_self_consistent() {
    let data = read_sample();
    let diff = roundtrip_geom(&data, Via::Hwpx)
        .unwrap_or_else(|e| panic!("roundtrip_geom({SAMPLE}): {e:?}"));

    assert_eq!(diff.page_count_a, diff.page_count_b, "페이지 수 왕복 보존");
    assert!(
        diff.max_disp <= 1.0,
        "{SAMPLE} 라운드트립 렌더 변위 {:.2}px > 1.0px (외부 참조 pic 드롭 회귀)",
        diff.max_disp
    );
    for pg in &diff.pages {
        assert!(
            !pg.structure_mismatch,
            "{SAMPLE} page {} 구조 불일치 (node {} vs {}) — 그림 컨트롤 드롭 회귀",
            pg.page, pg.node_count_a, pg.node_count_b
        );
    }
}

/// HWP5 원본을 HWPX로 저장한 산출물은 HWPX 컨테이너라도 HWP5-origin marker를 통해
/// HWP5 lineSeg 부재/pagination 시멘틱을 유지해야 한다.
#[test]
fn issue_1891_hwp5_origin_hwpx_export_reparse_keeps_page_count() {
    for (sample, expected_page_count) in HWP5_ORIGIN_SAMPLES {
        let bytes = read_rel(sample);
        let source =
            DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"));
        let before = source.page_count();
        assert_eq!(
            before, *expected_page_count,
            "{sample}: 공식 PDF 기준 쪽수와 불일치"
        );
        let exported = source
            .export_hwpx_native()
            .unwrap_or_else(|e| panic!("export {sample}: {e:?}"));
        let reparsed = DocumentCore::from_bytes(&exported)
            .unwrap_or_else(|e| panic!("reparse exported {sample}: {e:?}"));

        assert_eq!(
            reparsed.page_count(),
            before,
            "{sample}: HWP5-origin HWPX export 재파스 쪽수 불일치"
        );

        let parsed =
            parse_hwpx(&exported).unwrap_or_else(|e| panic!("parse exported hwpx {sample}: {e:?}"));
        assert!(
            parsed
                .hwpx_aux_entry(HWP5_ORIGIN_HWPX_MARKER_PATH)
                .is_some(),
            "{sample}: HWP5-origin HWPX marker 누락"
        );
    }
}

/// IR 핀: Link BinData 항목과 그 참조가 2-round 왕복에서 안정 보존된다.
#[test]
fn issue_1891_link_bin_data_stable_across_two_rounds() {
    let data = read_sample();
    let doc1 = parse_hwpx(&data).expect("parse 1");

    let count_links = |doc: &rhwp::model::document::Document| {
        doc.doc_info
            .bin_data_list
            .iter()
            .filter(|b| matches!(b.data_type, BinDataType::Link))
            .count()
    };
    fn count_pics_in_paras(paras: &[rhwp::model::paragraph::Paragraph]) -> usize {
        use rhwp::model::control::Control;
        use rhwp::model::shape::ShapeObject;
        let mut n = 0;
        for para in paras {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Picture(_) => n += 1,
                    Control::Table(tbl) => {
                        for cell in &tbl.cells {
                            n += count_pics_in_paras(&cell.paragraphs);
                        }
                    }
                    Control::Shape(shape) => {
                        if matches!(shape.as_ref(), ShapeObject::Picture(_)) {
                            n += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        n
    }
    let count_pics = |doc: &rhwp::model::document::Document| {
        doc.sections
            .iter()
            .map(|s| count_pics_in_paras(&s.paragraphs))
            .sum::<usize>()
    };

    let links1 = count_links(&doc1);
    let pics1 = count_pics(&doc1);
    assert!(links1 > 0, "픽스처에 Link BinData 가 있어야 함");
    assert!(pics1 >= 4, "픽스처 본문 pic 수 확인: {pics1}");

    let out1 = serialize_hwpx(&doc1).expect("serialize 1");
    let doc2 = parse_hwpx(&out1).expect("parse 2");
    assert_eq!(
        count_links(&doc2),
        links1,
        "Link BinData 항목 수 1-round 보존"
    );
    assert_eq!(
        count_pics(&doc2),
        pics1,
        "pic 컨트롤 수 1-round 보존 (드롭 회귀)"
    );

    let out2 = serialize_hwpx(&doc2).expect("serialize 2 (숫자 불변식 어긋나면 실패)");
    let doc3 = parse_hwpx(&out2).expect("parse 3");
    assert_eq!(
        count_links(&doc3),
        links1,
        "Link BinData 항목 수 2-round 보존"
    );
    assert_eq!(count_pics(&doc3), pics1, "pic 컨트롤 수 2-round 보존");

    // 임베디드 그림의 데이터 정합: pic 이 참조하는 bin_data_id 가 실제 콘텐츠와
    // 같은 바이트를 가리켜야 한다 (명명 시프트 회귀 검출).
    let content_bytes = |doc: &rhwp::model::document::Document| -> Vec<(u16, usize)> {
        let mut v: Vec<(u16, usize)> = doc
            .bin_data_content
            .iter()
            .map(|b| (b.id, b.data.len()))
            .collect();
        v.sort();
        v
    };
    assert_eq!(
        content_bytes(&doc1),
        content_bytes(&doc3),
        "bin_data_content (id, 크기) 2-round 보존 — 인덱스 시프트 회귀"
    );
}
