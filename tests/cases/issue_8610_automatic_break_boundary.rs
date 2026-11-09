//! 관계형·고정형 — 자동 분할 경계의 열린 변과 실제 종료선을 독립 Print에 맞춰 보호한다.
//! 주석 갱신: 2026-11-07T06:49:20+09:00; 새 파일, 직전 파일 커밋 없음.
//! 2026-11-07 측정: 96 DPI, 색상 경계 보정 전 2px 내용 실루엣, Print 프로필·compat 2022.
//! NARS 2021 원본 전3쪽 Native 99.85/100/100%, fresh WASM 99.87/100/100%; 최저99.85/99.87%.
//! 독립 PDF 전3쪽과 일치하고 양쪽 전3쪽 review·overlay에서 추가 컷 선 제거·실제 종료선·내용을 판독했다.
//! Native R26b·fresh WASM R26c의 source/producer SHA 및 원본/PDF 해시:
//! mydocs/pr/assets/planet6897_20261106/pr8615/r26/{native,wasm}-provenance.json.
//! 같은 폴더의 native.tsv·wasm.tsv와 PNG, ../break-border의 원시 attr·독립 PDF vector가 근거다.
//! 실제 한컴 저장 HWPX의 breakCellSeparateLine=1과 DiagonalType=NONE을 따른다.
//! 2쪽에는 표 전체 폭의 수평선이 없고 3쪽에는 실제 종료선이 하나 있다. 절대 화면 좌표 핀은 없다.
//! 실제 글꼴 설치 여부를 분기하거나 skip하지 않으며 저장 표의 변·본문 순서로 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::style::{BorderFill, CenterLine};
use rhwp::renderer::render_tree::{LineNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue8610/36849-nars-press-release-20211116.hwp";

fn bytes() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("검증한 NARS 원본")
}

fn outer_table(node: &RenderNode) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(0) && table.control_index == Some(2) {
            return Some(node);
        }
    }
    node.children.iter().find_map(outer_table)
}

fn full_width_edges(table: &RenderNode) -> Vec<&LineNode> {
    table
        .children
        .iter()
        .filter_map(|node| {
            let RenderNodeType::Line(line) = &node.node_type else {
                return None;
            };
            let horizontal = (line.y2 - line.y1).abs() <= f64::EPSILON;
            let full_width = ((line.x2 - line.x1).abs() - table.bbox.width).abs() < 1e-6;
            (horizontal && full_width).then_some(line)
        })
        .collect()
}

fn text(node: &RenderNode, output: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        output.push_str(&run.text);
    }
    for child in &node.children {
        text(child, output);
    }
}

#[test]
fn split_fragments_keep_open_cuts_and_the_real_terminal_border() {
    let core = DocumentCore::from_bytes(&bytes()).expect("원본 조판");
    assert_eq!(core.page_count(), 3, "독립 한컴 Print 전3쪽");
    let mut pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("실제 render tree");
        let table = outer_table(&tree.root).expect("본문을 소유한 바깥 표");
        let edges = full_width_edges(table);
        if page == 0 {
            assert!(!edges.is_empty(), "첫 조각의 머리 영역 괘선은 보존한다");
            assert!(
                edges
                    .iter()
                    .all(|line| line.y1 < table.bbox.y + table.bbox.height - line.style.width),
                "첫 조각의 컷을 셀 종료선으로 닫지 않는다"
            );
        } else if page == 1 {
            assert!(edges.is_empty(), "독립 Print 2쪽은 위·아래 컷 모두 열린다");
        } else {
            assert_eq!(
                edges.len(),
                1,
                "독립 Print 3쪽의 실제 종료선 하나는 보존한다"
            );
            assert!(edges[0].y1 > table.bbox.y, "이어받기 시작 컷을 닫지 않는다");
        }
        let mut content = String::new();
        text(table, &mut content);
        pages.push(content);
    }
    assert!(pages[0].contains("순환경제 구축"));
    assert!(pages[1].contains("폐기물은"));
    assert!(
        pages[2].contains("장철민") && pages[2].contains("담당자"),
        "마지막 본문·연락처를 잃지 않는다"
    );
    assert!(
        !pages[0].contains("담당자: 환경노동팀"),
        "끝 내용이 첫 조각으로 중복되지 않는다"
    );
}

#[test]
fn explicit_break_line_survives_both_document_serializers() {
    let mut document = rhwp::parser::parse_document(&bytes()).expect("원본 파싱");
    let original = &document.doc_info.border_fills[14];
    assert!(original.break_cell_separate_line);
    assert_eq!(original.center_line, CenterLine::None);
    assert_eq!(original.diagonal.diagonal_type, 0, "한컴의 별도 컷 선 NONE");
    document.doc_info.border_fills[14].raw_data = None;
    for encoded in [
        rhwp::serializer::serialize_document(&document).expect("HWP5 모델 직렬화"),
        rhwp::serializer::serialize_hwpx(&document).expect("HWPX 모델 직렬화"),
    ] {
        let reopened = DocumentCore::from_bytes(&encoded).expect("저장본 파싱");
        let saved = &reopened.document().doc_info.border_fills[14];
        assert!(
            saved.break_cell_separate_line,
            "명시적 표 경계 설정을 저장한다"
        );
        assert_eq!(saved.diagonal.diagonal_type, 0);
        assert_eq!(saved.center_line, CenterLine::None);
    }
}

#[test]
fn hwp_serializer_keeps_backslash_and_center_line_contexts_distinct() {
    // 이 검사는 문서 배치가 아닌 합성 BORDER_FILL의 사양 비트 계약이다.
    let encoded_attr = |fill: BorderFill| {
        let bytes = rhwp::serializer::doc_info::serialize_border_fill(&fill);
        u16::from_le_bytes([bytes[0], bytes[1]])
    };
    let backslash = (1 << 5) | (1 << 10);
    assert_eq!(
        encoded_attr(BorderFill {
            attr: backslash,
            ..Default::default()
        }),
        backslash,
        "표 별도 설정 false가 기존 꺾인 BackSlash를 지우지 않는다"
    );
    assert_eq!(
        encoded_attr(BorderFill {
            attr: 1 << 10,
            ..Default::default()
        }) & (1 << 10),
        0,
        "일반 표 문맥에서 해제한 별도 설정은 저장에도 반영한다"
    );
    for center_line in [
        CenterLine::Vertical,
        CenterLine::Horizontal,
        CenterLine::Cross,
    ] {
        let attr = encoded_attr(BorderFill {
            center_line,
            ..Default::default()
        });
        assert_eq!(
            CenterLine::from_hwp_attr(attr),
            center_line,
            "중심선 방향 비트는 별도 계약이다"
        );
    }
}
