//! 관계형 회귀: 머리말·꼬리말 표의 셀 최소 높이·내용 포함·소속 보존과 정상 비인라인 대조. 과거의 2배 임계값·200px 핀은 사용하지 않는다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! header-stale.hwpx (api-8571-height-contract-header-stale): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-near.hwpx (api-8571-height-contract-header-near): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-grown.hwpx (api-8571-height-contract-header-grown): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! footer-stale.hwpx (api-8571-height-contract-footer-stale): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! footer-near-complete.hwpx (api-8571-height-contract-footer-near-complete): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! footer-grown.hwpx (api-8571-height-contract-footer-grown): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! non-inline-header-table.hwp (api-8571-non-inline-header-table-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! non-inline-header-table.hwpx (api-8571-non-inline-header-table-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형: 머리말·꼬리말 표의 셀 최소 높이·내용 포함·원본 소속 보존.
//! 주석 갱신: 2026-11-06T02:33:40+09:00. 직전 파일 커밋: 2026-10-19T20:25:11+09:00,
//! 007cd428c7821f91bb82631fac659fda5ae3a93f. 측정 시각과 Git 변경 시각을 구분한다.
//! 측정: 2026-11-06 KST, 독립 한컴 2020 Print, 96dpi PDFium, 보정 전 2px raw 실루엣.
//! header/footer stale·near·grown 여섯 HWPX: 각각 전 1쪽 Native/fresh WASM 모두 100.00%.
//! footer-near-complete는 전체 표를 직접 판독할 수 있게 아래 여백을 명시한 입력이다.
//! 정상 non-inline-header-table.hwp/.hwpx: 각각 전 1쪽 양쪽 100.00%, 독립 PDF 쪽수 일치.
//! 위 수치는 원본별 전쪽 최저값이며 제품 source a314d7a24d3234305abff43149f575853db31105다.
//! 정식 근거: mydocs/pr/assets/semanticist21_20261105/hf-height-contract/README.md.
//! 최소 글꼴 5개 환경에서 실행하며 개인 OTF 설치에 의존하지 않는다.
//! 기존 2배 임계값·200px/357.24px 핀과 불완전한 합성 줄 메타데이터를 재사용하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

fn open(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/semanticist21_integration_review/pr8571")
        .join(name);
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn source_table(core: &DocumentCore, footer: bool) -> &Table {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Header(header) if !footer => Some(&header.paragraphs),
            Control::Footer(value) if footer => Some(&value.paragraphs),
            _ => None,
        })
        .flatten()
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("원본 머리말·꼬리말 표")
}

fn find(
    node: &RenderNode,
    predicate: impl Fn(&RenderNodeType) -> bool + Copy,
) -> Option<&RenderNode> {
    if predicate(&node.node_type) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find(child, predicate))
}

fn text(node: &RenderNode) -> String {
    match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => node.children.iter().map(text).collect(),
    }
}

fn assert_content_contained(table: &RenderNode) {
    let cells: Vec<_> = table
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert!(!cells.is_empty(), "내용을 소유하는 셀이 존재한다");
    for cell in cells {
        assert!(cell.bbox.y >= table.bbox.y, "셀은 표 위로 벗어나지 않는다");
        assert!(
            cell.bbox.y + cell.bbox.height <= table.bbox.y + table.bbox.height,
            "셀은 표 아래로 벗어나지 않는다"
        );
        for line in cell
            .children
            .iter()
            .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        {
            assert!(
                line.bbox.y >= cell.bbox.y,
                "실제 글줄은 소유 셀 안에서 시작한다"
            );
            assert!(
                line.bbox.y + line.bbox.height <= cell.bbox.y + cell.bbox.height,
                "마지막 글줄까지 소유 셀에 포함된다"
            );
        }
    }
}

fn check_inline(case: &str, grows_by_content: bool) {
    for footer in [false, true] {
        let region = if footer { "footer" } else { "header" };
        let suffix = if footer && case == "near" {
            "near-complete"
        } else {
            case
        };
        let core = open(&format!("height-contract/{region}-{suffix}.hwpx"));
        let source = source_table(&core, footer);
        assert!(source.common.treat_as_char);
        let tree = core.build_page_render_tree(0).unwrap();
        let owner = find(&tree.root, |kind| {
            if footer {
                matches!(kind, RenderNodeType::Footer)
            } else {
                matches!(kind, RenderNodeType::Header)
            }
        })
        .expect("원본 머리말·꼬리말 영역");
        let table = find(owner, |kind| matches!(kind, RenderNodeType::Table(_)))
            .expect("영역이 소유한 실제 표");
        let minimum = f64::from(source.cells[0].height) * 96.0 / 7200.0;
        let object_height = f64::from(source.common.height) * 96.0 / 7200.0;
        assert_content_contained(table);
        if grows_by_content {
            assert!(
                table.bbox.height > minimum,
                "최소 셀 선언보다 큰 내용을 수용한다"
            );
            assert!(
                table.bbox.height < object_height,
                "낡은 개체 선언까지 빈 높이를 늘리지 않는다"
            );
            assert_eq!(text(table), "가가가", "명시적 세 줄의 순서·누락·중복 보존");
        } else {
            // 절대 배치 핀이 아니라 원본 최소 크기와 실제 표 사이의 비율 계약이다.
            assert!(
                (table.bbox.height / minimum - 1.0).abs() < 1e-9,
                "내용을 담는 셀 선언은 개체 선언의 상대 격차와 관계없이 유지한다"
            );
            assert!(table.bbox.height < object_height);
            assert_eq!(text(table), "가", "원본 내용 보존");
        }
        assert!(
            table.bbox.y >= tree.root.bbox.y
                && table.bbox.y + table.bbox.height <= tree.root.bbox.y + tree.root.bbox.height,
            "독립 Print에서 확인한 완전한 표가 같은 페이지에 포함된다"
        );
    }
}

#[test]
fn inline_table_keeps_cell_declared_height_when_object_height_is_stale() {
    check_inline("stale", false);
}

#[test]
fn inline_table_keeps_cell_minimum_when_object_height_is_only_slightly_taller() {
    check_inline("near", false);
}

#[test]
fn inline_table_grows_to_contain_all_content_without_following_stale_object_height() {
    check_inline("grown", true);
}

#[test]
fn normal_non_inline_header_tables_keep_their_owner_and_complete_cell_contents() {
    for extension in ["hwp", "hwpx"] {
        let core = open(&format!("non-inline-header-table.{extension}"));
        assert!(!source_table(&core, false).common.treat_as_char);
        let tree = core.build_page_render_tree(0).unwrap();
        let header = find(&tree.root, |kind| matches!(kind, RenderNodeType::Header)).unwrap();
        let table = find(header, |kind| matches!(kind, RenderNodeType::Table(_))).unwrap();
        assert_content_contained(table);
        assert_eq!(text(table), "머리말 자리 차지 표");
    }
}
