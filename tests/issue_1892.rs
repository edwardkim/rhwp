//! Issue #1892 — 대법원 서식 hwp(HWP3) 계열 HWP5 라운드트립 렌더 위치 자기정합.
//!
//! 20k 라운드트립 위치 검사에서 대법원 행정예규 서식 계열이 페이지 내 초대형
//! 변위(최대 5449px)로 표면화했다. HWP3 파스 → export_hwp_with_adapter → 재파스
//! 경로의 3중 결함 체인 + 탭 확장 결함:
//!   1) 자식이 없는 '$con' SHAPE_COMPONENT(빈 묶음)를 파서가 사각형 폴백으로
//!      오분류 → 재파스 렌더 트리 Group→Rect 구조 분기,
//!   2) 직렬화기 rendering matrix 폴백이 offset_x/y 를 translation 으로 승격 —
//!      그룹 자식 위치의 단일 권위는 render_tx/ty 인데 메타데이터 offset 이
//!      행렬로 살아나 자식이 relative_pos 만큼 이동(최대 10485px),
//!   3) HWP3 build_common_obj_attr 가 크기 기준 비트(15-17/18-19)를 누락 —
//!      0(Paper 퍼센트)으로 저장되어 재파스 시 개체가 종이×(크기/10000)배 팽창,
//!   4) 직렬화기의 탭 "데이터 없음" 마커([0,...,0,0x0009])를 파서가 tab_extended
//!      로 실어 레이아웃이 ext[0]=0 을 탭 결과 위치로 해석 → 탭 무폭화.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};
use rhwp::model::control::Control;
use rhwp::model::shape::{ShapeObject, SizeCriterion};
use rhwp::parser::parse_document;
use rhwp::serializer::serialize_document;

const SAMPLE_GROUP: &str = "samples/issue1892_hwp3_drawing_group_roundtrip.hwp";
const SAMPLE_TAB: &str = "samples/issue1892_hwp3_tab_roundtrip.hwp";

fn read_sample(rel: &str) -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn assert_roundtrip_render_self_consistent(rel: &str) {
    let data = read_sample(rel);
    let diff =
        roundtrip_geom(&data, Via::Hwp).unwrap_or_else(|e| panic!("roundtrip_geom({rel}): {e:?}"));

    assert_eq!(
        diff.page_count_a, diff.page_count_b,
        "{rel} 라운드트립 페이지 수 불일치: A={} B={}",
        diff.page_count_a, diff.page_count_b
    );
    assert!(
        diff.max_disp <= 1.0,
        "{rel} 라운드트립 렌더 변위 {:.2}px > 1.0px (HWP3 그리기/탭 왕복 회귀)",
        diff.max_disp
    );
    for pg in &diff.pages {
        assert!(
            !pg.structure_mismatch,
            "{rel} page {} 구조 불일치 (node {} vs {}) — 빈 묶음 Group→Rect 왕복 회귀",
            pg.page, pg.node_count_a, pg.node_count_b
        );
    }
}

/// 대표 2912309 (제적등본2 서식): 빈 묶음 + rendering matrix + 크기 기준 축.
/// 수정 전 STRUCT_MISMATCH 5449px.
#[test]
fn issue_1892_hwp3_drawing_group_roundtrip_render_is_self_consistent() {
    assert_roundtrip_render_self_consistent(SAMPLE_GROUP);
}

/// 대표 2952505 (문서건명부 서식): 탭 확장 "데이터 없음" 마커 축.
/// 수정 전 OVER 130px.
#[test]
fn issue_1892_hwp3_tab_roundtrip_render_is_self_consistent() {
    assert_roundtrip_render_self_consistent(SAMPLE_TAB);
}

/// IR 핀: 빈 묶음('$con', children=0)이 직렬화→재파스에서 Group 으로 보존되고
/// (사각형 폴백 금지), 그룹 자식 rendering matrix 는 identity(translation 미승격),
/// 최상위 개체 크기 기준은 Absolute 로 왕복한다.
#[test]
fn issue_1892_empty_container_and_size_criterion_roundtrip() {
    let data = read_sample(SAMPLE_GROUP);
    let doc_a = parse_document(&data).expect("parse A");
    let out = serialize_document(&doc_a).expect("serialize");
    let doc_b = parse_document(&out).expect("parse B");

    // 문단 0.1 의 최상위 묶음: A 구조 = Group { children: [Group { children: [] }] }
    let find_group =
        |doc: &rhwp::model::document::Document| -> Option<(bool, SizeCriterion, SizeCriterion)> {
            for para in &doc.sections[0].paragraphs {
                for ctrl in &para.controls {
                    if let Control::Shape(shape) = ctrl {
                        if let ShapeObject::Group(g) = shape.as_ref() {
                            if let Some(ShapeObject::Group(child)) = g.children.first() {
                                let child_matrix_is_identity = child.shape_attr.render_tx == 0.0
                                    && child.shape_attr.render_ty == 0.0;
                                return Some((
                                    child_matrix_is_identity,
                                    g.common.width_criterion,
                                    g.common.height_criterion,
                                ));
                            }
                        }
                    }
                }
            }
            None
        };

    let a = find_group(&doc_a).expect("A: 중첩 빈 묶음을 가진 최상위 묶음이 있어야 함");
    let b = find_group(&doc_b)
        .expect("B: 재파스에서 중첩 빈 묶음이 Group 으로 보존되어야 함 (사각형 폴백 회귀)");

    assert!(a.0, "A: HWP3 그룹 자식 rendering matrix 는 identity");
    assert!(
        b.0,
        "B: 재파스 그룹 자식 rendering matrix 가 translation 으로 오염 (offset 승격 회귀)"
    );
    assert_eq!(
        (b.1, b.2),
        (SizeCriterion::Absolute, SizeCriterion::Absolute),
        "B: 재파스 크기 기준이 Absolute 가 아님 (criterion 비트 누락 → Paper 퍼센트 팽창 회귀)"
    );
}

/// IR 핀: tab_extended 없는 탭이 직렬화→재파스 후에도 tab_extended 를 만들지
/// 않는다 (null 마커를 IR 로 실으면 레이아웃이 탭 무폭으로 해석).
#[test]
fn issue_1892_null_tab_extension_marker_not_parsed_into_ir() {
    let data = read_sample(SAMPLE_TAB);
    let doc_a = parse_document(&data).expect("parse A");
    let out = serialize_document(&doc_a).expect("serialize");
    let doc_b = parse_document(&out).expect("parse B");

    let count = |doc: &rhwp::model::document::Document| -> (usize, usize) {
        let mut tabs = 0usize;
        let mut exts = 0usize;
        for para in &doc.sections[0].paragraphs {
            tabs += para.text.matches('\t').count();
            exts += para.tab_extended.len();
        }
        (tabs, exts)
    };

    let (tabs_a, exts_a) = count(&doc_a);
    let (tabs_b, exts_b) = count(&doc_b);
    assert!(tabs_a > 0, "픽스처에 탭이 있어야 함");
    assert_eq!(tabs_a, tabs_b, "탭 문자 수 왕복 보존");
    assert_eq!(
        exts_a, exts_b,
        "tab_extended 수 왕복 보존 (null 마커가 IR 로 유입되는 회귀: A={exts_a} B={exts_b})"
    );
}
