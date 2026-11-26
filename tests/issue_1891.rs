//! 관계형·고정형: 기존 계약을 유지하고 #7754~#7765의 수용한 원본 범위를 복원한다.
//! 주석 갱신: 2026-11-22T03:21:21+09:00; 직전 파일 변경: 2026-10-05T01:58:07+09:00 127e931d1f821acd9203d41ed58d2b9d27d54af1.
//! 복원 원본 측정: 2026-11-22 KST,96DPI·Print·PDFium·보정 전2px raw, 각 독립 Print와 같은 전체 쪽수.
//! Native/fresh WASM 전쪽 최저: 7754 평문24쪽98.29/98.22%,7755 315쪽98.40/98.40%,
//! 7756 1쪽99.00/98.99%,7757 3쪽99.78/99.80%,7759 1쪽99.94/99.94%,7760 19쪽99.18/99.18%,
//! 7761 HWP46쪽99.98/99.98%,7762 HWPX46쪽99.98/99.98%,7765 HWP64쪽98.08/98.11%·HWPX64쪽98.69/98.69%.
//! 근거: mydocs/pr/assets/issues7754_7765/source32/README.md; 시장구조조사의 선행 전쪽 교차 증거는 같은 상위 market-source7/README.md.
//! 이번 복원 범위 밖 입력의 기존 계약·근거는 아래 기록을 유지한다. 시스템 글꼴을 요구하지 않는다.
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
// 시각 기준을 유지하는 원본과 대응 PDF의 쪽수.
// 86712 두 원본은 source32의 전64쪽 Native/fresh WASM 98% 이상 근거로 복원한다.
// 80168의 이관 범위는 그대로 유지한다.
const HWP5_ORIGIN_SAMPLES: &[(&str, u32)] = &[
    ("samples/86712_regulatory_analysis.hwp", 64),
    ("samples/issue1891/86712_regulatory_analysis.hwpx", 64),
    ("samples/76076_regulatory_analysis.hwp", 82),
    ("samples/issue1891/76076_regulatory_analysis.hwpx", 82),
];

/// 정상 원본의 저장 줄을 테스트 입력 교체 과정에서 다시 잃지 않게 한다.
/// 잘못된 이전 입력은 원시 파싱에서 1810개 문단의 LineSeg가 비어 있었다.
/// DocumentCore의 합성 결과가 아닌 parser 출력을 검사해야 소실을 검출한다.
#[test]
fn issue_1891_regulatory_fixture_preserves_authentic_saved_lines() {
    fn check(paras: &[rhwp::model::paragraph::Paragraph]) -> usize {
        use rhwp::model::control::Control;
        let mut count = 0;
        for para in paras {
            assert!(
                !para.line_segs.is_empty(),
                "정상 원본의 저장 줄이 없는 문단: {:?}",
                para.text
            );
            assert!(para.line_segs.iter().all(|seg| seg.tag
                & rhwp::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                == 0));
            count += 1;
            for control in &para.controls {
                if let Control::Table(table) = control {
                    for cell in &table.cells {
                        count += check(&cell.paragraphs);
                    }
                }
            }
        }
        count
    }
    for sample in [
        "samples/86712_regulatory_analysis.hwp",
        "samples/issue1891/86712_regulatory_analysis.hwpx",
    ] {
        let doc = rhwp::parser::parse_document(&read_rel(sample)).expect("정상 저장본 원시 파싱");
        let paragraphs: usize = doc
            .sections
            .iter()
            .map(|section| check(&section.paragraphs))
            .sum();
        assert_eq!(paragraphs, 2101, "본문·표 재귀 문단 보존: {sample}");
    }
}

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
/// HWP5 저장 줄/페이지 분할 시멘틱을 유지해야 한다. 줄 정보 유무는 개별 입력에 따른다.
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

        if sample.contains("86712_regulatory_analysis") {
            // HWP5-origin 재저장은 저장 줄 계약으로 재배치할 수 있다.
            // 원본별 Print 쪽 소유는 #2279/#7765가 보호하며, 왕복에서는 모든 원본 소유자의
            // 글자 순서·내용·중복을 검사한다. 쪽 경계를 문자열의 일부로 가정하지 않는다.
            let public_owners = |doc: &DocumentCore| {
                let mut owners = std::collections::BTreeMap::<String, String>::new();
                for page in 0..doc.page_count() {
                    let layout: serde_json::Value =
                        serde_json::from_str(&doc.get_page_text_layout_native(page).unwrap())
                            .unwrap();
                    for run in layout["runs"].as_array().unwrap() {
                        let owner = serde_json::json!([
                            run["secIdx"],
                            run["paraIdx"],
                            run["parentParaIdx"],
                            run["cellPath"]
                        ])
                        .to_string();
                        owners
                            .entry(owner)
                            .or_default()
                            .push_str(run["text"].as_str().unwrap());
                    }
                }
                owners
            };
            let original = public_owners(&source);
            let roundtrip = public_owners(&reparsed);
            assert_eq!(
                original.keys().collect::<Vec<_>>(),
                roundtrip.keys().collect::<Vec<_>>(),
                "{sample}: 왕복의 원본 소유 주소 보존"
            );
            for (owner, text) in original {
                assert_eq!(
                    text, roundtrip[&owner],
                    "{sample} {owner}: 전64쪽 원본 소유별 내용·순서·누락·중복 보존"
                );
            }
        }

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

/// [#3820 Stage 53] direct HWPX의 저장 reset 보정은 reset 없는 일반 중첩 표의
/// legacy scalar 측정까지 바꾸면 안 된다. 이 문서의 깊은 중첩 표는 그 과확장 시
/// 마지막 쪽에서 보이지 않는 줄이 34→56으로 증가한다.
///
/// [#5875] 상한 34→50: 중첩 표 글자 캡션을 그리기 시작하면서 69쪽의 복원된 캡션
/// (`<장애인건강검진기관(5개소) 탈의실 및 공용면적 현황>`)이 제 띠를 차지하자, 같은
/// 쪽 하단의 쪼갤 수 없는 중첩 표(#4915 계열)가 캡션 띠만큼 내려가 쪽 경계를 넘는다
/// (+17줄, 전부 69쪽). #3820 의 legacy scalar 과확장(마지막 쪽 +22줄)과는 위치·기전이
/// 다르므로 그 축의 가드 역할은 유지된다.
#[test]
fn issue_1891_external_link_overflow_cell_lines_do_not_grow() {
    let data = read_sample();
    let doc = DocumentCore::from_bytes(&data).expect("parse issue1891 sample");
    let _ = doc.take_overflow_cell_lines();

    let mut total = 0u64;
    for page in 0..doc.page_count() {
        doc.render_page_svg_native(page)
            .unwrap_or_else(|e| panic!("render page {}: {e:?}", page + 1));
        total += u64::from(doc.take_overflow_cell_lines());
    }

    assert!(
        total <= 50,
        "direct HWPX 일반 중첩 표의 쪽 밖 소실 줄이 증가함: {total} > 50"
    );
}
