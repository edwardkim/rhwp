//! 검사 유형: 본문 배치는 관계형(줄·표/그림 겹침, 줄 순서), 문서 쪽수만 고정형이다.
//! 2단 쪽의 여러 단 걸침 개체는 앵커 없는 단의 저장 글줄에도 같은 배제 프레임을 제공한다.
//! 주석 갱신: 2026-11-11 00:05 KST. 아래 초기 검증 이력과 최신 통합 보정 측정을 구분한다.
//! 실제 측정: 2026-11-08 KST, base = upstream/devel 5077b216cb, head = base + 이 PR 의 src 변경. 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw.
//! 원본 `samples/issue7282/wide_paper_table_two_columns.hwp`, 전체 2쪽=2쪽.
//! 1쪽 수정 전 Native 74.70 / fresh WASM 73.59 → 수정 후 Native 93.45 / fresh WASM 92.47. 2쪽 Native 96.35 / WASM 95.98 그대로.
//! 수정 후 전쪽 최저 Native 93.45·fresh WASM 92.47(90% 미만 0쪽). 98% 미만이라 px 좌표는 고정하지 않는다.
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_7282/{base,head}/{native,wasm}/wide_paper_table_two_columns_silhouette.tsv`, `png/`.
//! 확대 판독: 1쪽 표와 두 단의 표 위·아래 줄이 한/글과 같은 자리(overlay 일치). 수정 전에는 왼쪽 단 줄이 표 위에 겹쳐 그려졌다. 오른쪽 단 마지막 문단이 약 5px 위에 놓이는 차이는 수정 전부터 있던 것이다.
//! 같은 assertion 이 수정 전 FAIL(왼쪽 단 문단 0.6 줄이 표와 겹침) / 수정 후 PASS.
//! 독립 기대값: 한/글 Print `pdf/issue7282/wide_paper_table_two_columns-2020.pdf`(2쪽). 1쪽 두 단 모두 표 위 줄과 표 아래 줄 사이에 표가 비켜 있고, 글자가 표와 겹치지 않는다.
//! 저장 근거: 문단 0.6(왼쪽 단)·0.12(오른쪽 단)의 줄 사다리 19200 → 32008 점프 = 줄 피치 1600 + 표 높이 8374 + 바깥 위·아래 1417×2. 표는 문단 0.13 의 종이 기준 자리차지 표다.
//! 독립 절대 px 좌표 대신 표 상자와 줄 상자의 겹침·순서 관계를 쓴다. 설치 글꼴을 가정하지 않는다.
//!
//! 2026-11-10 23:28 KST 추가 판독: 통합 코드 009b3cf42b의 Native/fresh WASM은
//! 1쪽 93.08%, 2쪽 96.18%다. 2쪽은 그림 아래 글줄을 그림 뒤에 그려 가리는 실제 결함이 있다.
//! 새 `spanning_square_picture_*` 검사는 이 글줄과 실제 그림 상자의 포함·순서 관계를 검사한다.
//! 위치 변경 반례는 저장 줄 점프와 현재 그림 프레임이 일치하지 않을 때 일반 줄 흐름을 유지한다.
//! 2026-11-11 보정 후 Native/fresh WASM은 1쪽 93.08/93.08%, 2쪽 100.00/100.00%다.
//! 2쪽 review·overlay에서 양 단의 모든 후속 글줄이 그림 아래에 남는 것을 직접 판독했다.
//! 절대 px를 고정하지 않는다. 원본 줄 소속·그림 배제 프레임과의 관계 및 위치 변경 반례를 검사한다.
//! 정식 전후 증거: mydocs/pr/assets/planet6897_clean19_20261110/pr8752/maintainer/spanning-square/.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue7282/wide_paper_table_two_columns.hwp";
const PDF_PAGES: u32 = 2;
const TABLE_PARA: usize = 13;
/// 저장 사다리에서 표 틀 아래로 점프하는 문단: 왼쪽 단(앵커 없음), 오른쪽 단.
const PUSHED_PARAS: [usize; 2] = [6, 12];

fn table_frame(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(TABLE_PARA) && table.cell_context.is_none() {
            return Some(node.bbox);
        }
    }
    node.children.iter().find_map(table_frame)
}

fn para_lines(node: &RenderNode, para: usize, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para) {
            out.push(node.bbox);
            return;
        }
    }
    for child in &node.children {
        para_lines(child, para, out);
    }
}

#[test]
fn spanning_paper_table_pushes_lines_in_both_columns() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), PDF_PAGES, "독립 Print 와 같은 쪽수");

    let tree = core.build_page_render_tree(0).expect("1쪽 배치");
    let table = table_frame(&tree.root).expect("1쪽 두 단 걸침 표");
    for para in PUSHED_PARAS {
        let mut lines = Vec::new();
        para_lines(&tree.root, para, &mut lines);
        assert!(lines.len() >= 3, "문단 0.{para} 의 줄: {lines:?}");
        assert!(
            lines.windows(2).all(|w| w[0].y < w[1].y),
            "문단 0.{para} 의 줄은 저장 순서대로 아래로 간다: {lines:?}"
        );
        for line in &lines {
            let overlaps =
                line.y + line.height > table.y + 0.5 && line.y < table.y + table.height - 0.5;
            assert!(
                !overlaps,
                "문단 0.{para} 의 줄이 표와 겹친다: line={line:?} table={table:?}"
            );
        }
        assert!(
            lines.iter().any(|l| l.y + l.height <= table.y + 0.5),
            "문단 0.{para} 의 앞 줄은 표 위에 남는다: {lines:?} table={table:?}"
        );
        assert!(
            lines.iter().any(|l| l.y >= table.y + table.height - 0.5),
            "문단 0.{para} 의 나머지 줄은 표 아래로 간다: {lines:?} table={table:?}"
        );
    }
}

fn square_picture_frame(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(&node.node_type, RenderNodeType::Image(image) if image.para_index == Some(18)) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(square_picture_frame)
}

fn source_picture_jump(core: &DocumentCore, para: usize) -> usize {
    let segments = &core.document().sections[0].paragraphs[para].line_segs;
    segments
        .windows(2)
        .position(|pair| {
            pair[1].vertical_pos > pair[0].vertical_pos + pair[0].line_height + pair[0].line_spacing
        })
        .expect("독립 Print의 그림 전후를 나누는 원본 저장 줄 점프")
        + 1
}

#[test]
fn spanning_square_picture_keeps_both_columns_successors_outside_its_frame() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).unwrap()).unwrap();
    let tree = core.build_page_render_tree(1).unwrap();
    let picture = square_picture_frame(&tree.root).expect("2쪽 실제 그림을 보존한다");
    for para in [16, 22] {
        let mut lines = Vec::new();
        para_lines(&tree.root, para, &mut lines);
        let source = &core.document().sections[0].paragraphs[para];
        assert_eq!(
            lines.len(),
            source.line_segs.len(),
            "원본 모든 줄을 보존한다"
        );
        let split = source_picture_jump(&core, para);
        assert!(lines[..split]
            .iter()
            .all(|line| line.y + line.height <= picture.y));
        assert!(
            lines[split..]
                .iter()
                .all(|line| line.y >= picture.y + picture.height),
            "문단 {para}의 그림 아래 소유 글줄이 그림 뒤로 들어갔다: {lines:?}, 그림 {picture:?}"
        );
        assert!(lines.windows(2).all(|pair| pair[0].y < pair[1].y));
    }
}

fn picture_jump_after_geometry_change(change: impl FnOnce(&mut rhwp::model::image::Picture)) {
    let mut core = DocumentCore::from_bytes(&std::fs::read(SOURCE).unwrap()).unwrap();
    let mut document = core.document().clone();
    let rhwp::model::control::Control::Picture(picture) =
        &mut document.sections[0].paragraphs[18].controls[0]
    else {
        panic!("원본 그림");
    };
    change(picture);
    core.set_document(document);
    let tree = core.build_page_render_tree(1).unwrap();
    for para in [16, 22] {
        let split = source_picture_jump(&core, para);
        let source = &core.document().sections[0].paragraphs[para].line_segs[split - 1];
        let mut lines = Vec::new();
        para_lines(&tree.root, para, &mut lines);
        let pitch = f64::from(source.line_height + source.line_spacing) * 96.0 / 7200.0;
        assert!(
            ((lines[split].y - lines[split - 1].y) - pitch).abs() < 0.01,
            "현재 그림이 증명하지 않는 저장 점프는 적용하지 않는다: {lines:?}"
        );
    }
}

#[test]
fn spanning_square_picture_requires_the_saved_jump_to_close_its_current_frame() {
    picture_jump_after_geometry_change(|picture| {
        picture.common.vertical_offset += 1600;
    });
}

#[test]
fn spanning_square_picture_outside_the_columns_does_not_push_their_lines() {
    picture_jump_after_geometry_change(|picture| {
        picture.common.horizontal_offset = 59528;
    });
}
