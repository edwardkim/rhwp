//! [#7232] scaffold 표 셀 문단의 가로 정렬을 지정할 수 있다.
//!
//! ## 무엇이 문제였나
//!
//! scaffold 가 만드는 셀 문단은 본문과 같은 `PS_NORMAL`(양쪽 정렬) 하나뿐이었고, 명세에
//! 정렬을 지정할 방법이 없었다. 한/글은 양쪽 정렬에서 한 줄에 단어가 하나뿐이면 폭을
//! 채우려 글자 사이를 벌리므로, 좁은 열에 긴 영문 토큰(코드명·경로·SQL)이 오면
//! `I N S E R T …` 처럼 벌어져 읽기 나빠진다.
//!
//! ## 기대값의 출처 — 이 수정과 독립적인 두 근거
//!
//! 1. **편집 경로의 기존 계약**: `DocumentCore::create_table_native` 는
//!    `options.column_alignments`(CLI `edit insert-table --alignments left,center,right`)를
//!    `find_or_create_para_shape` 로 옮겨 **열 단위**로 셀 문단 모양을 만든다. 지정이 없으면
//!    커서 문단의 정렬을 물려받는다. scaffold 도 같은 낱말·같은 축·같은 기본값을 쓴다.
//! 2. **한/글 정본 대조**: `samples/issue7232/` 의 두 입력을 한/글 2020(11.0.0.9136)으로
//!    PDF 출력해 글자 origin 차를 쟀다. 같은 셀·같은 글자에서
//!    `justify` 는 `[3.48, 7.80, 6.72, 6.72, 6.96, 7.08, …]pt`,
//!    `left` 는 `[3.12, 7.32, 6.24, 6.36, 6.60, 6.60, …]pt` 로
//!    **양쪽 정렬 쪽이 글자마다 0.36~0.48pt 벌어진다**. 곧 이슈가 신고한 벌어짐이고,
//!    `left` 지정이 그것을 없앤다.
//!
//! 기본값은 바꾸지 않는다. 코퍼스 실측(저장소 `samples/` HWPX 518종의 셀 문단 148,442개)에서
//! 셀 정렬은 CENTER 44.9% · JUSTIFY 29.8% · RIGHT 19.0% · LEFT 5.5% 로 갈리므로
//! "한 값이 옳다"가 성립하지 않는다. 종전 산출(양쪽 정렬)을 그대로 두고 선택지를 연다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::style::Alignment;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::scaffold::{build_scaffold, parse_scaffold_str};
use rhwp::serializer::serialize_hwpx;

fn spec_json(cell_align: &str) -> String {
    format!(
        r#"{{"version":"1","title":"정렬","blocks":[
            {{"type":"paragraph","text":"표 앞 문단"}},
            {{"type":"table"{cell_align},"rows":[
                ["No","항목","설명"],
                ["1","INSERT…VALUES APPEND","설명 텍스트"]
            ]}}
        ]}}"#
    )
}

fn doc_of(cell_align: &str) -> Document {
    let spec = parse_scaffold_str(&spec_json(cell_align)).expect("scaffold 명세");
    build_scaffold(&spec)
}

fn only_table(doc: &Document) -> &Table {
    let tables: Vec<&Table> = doc.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(tables.len(), 1, "표는 하나여야 한다");
    tables[0]
}

/// 셀 문단이 실제로 참조하는 문단 모양의 정렬 — 열 우선(행 0)으로 읽는다.
fn cell_alignments(doc: &Document) -> Vec<Alignment> {
    let table = only_table(doc);
    (0..table.col_count)
        .map(|c| {
            let cell = table
                .cells
                .iter()
                .find(|cell| cell.col == c && cell.row == 0)
                .expect("행 0 의 칸");
            let ps_id = cell.paragraphs[0].para_shape_id;
            doc.doc_info.para_shapes[ps_id as usize].alignment
        })
        .collect()
}

/// 본문 문단("표 앞 문단")과 표를 담은 문단(host)의 정렬 — 셀 정렬 지정이 이쪽을 건드리면
/// 안 된다. 제목 문단은 원래 가운데 정렬(`PS_TITLE`)이라 이 대조에서 뺀다.
fn body_and_host_alignments(doc: &Document) -> Vec<Alignment> {
    doc.sections[0]
        .paragraphs
        .iter()
        .filter(|p| {
            p.text == "표 앞 문단" || p.controls.iter().any(|c| matches!(c, Control::Table(_)))
        })
        .map(|p| doc.doc_info.para_shapes[p.para_shape_id as usize].alignment)
        .collect()
}

/// 지정이 없으면 종전대로 양쪽 정렬이다.
#[test]
fn default_stays_justify() {
    let doc = doc_of("");
    assert_eq!(
        cell_alignments(&doc),
        vec![Alignment::Justify; 3],
        "cell_align 생략 시 종전 산출(양쪽 정렬)이 그대로여야 한다"
    );
}

/// 값 하나는 표 전체에 적용된다.
#[test]
fn single_value_applies_to_every_cell() {
    let doc = doc_of(r#","cell_align":"left""#);
    assert_eq!(cell_alignments(&doc), vec![Alignment::Left; 3]);
    let outside = body_and_host_alignments(&doc);
    assert_eq!(
        outside,
        vec![Alignment::Justify; 2],
        "셀 정렬 지정이 본문 문단과 표 host 문단까지 바꾸면 안 된다"
    );
}

/// 목록은 열 단위다 — 편집 경로 `column_alignments` 와 같은 축.
#[test]
fn list_applies_per_column() {
    let doc = doc_of(r#","cell_align":["center","left","right"]"#);
    assert_eq!(
        cell_alignments(&doc),
        vec![Alignment::Center, Alignment::Left, Alignment::Right]
    );

    // 같은 열의 모든 행이 같은 정렬이어야 한다.
    let table = only_table(&doc);
    for cell in &table.cells {
        let expected = [Alignment::Center, Alignment::Left, Alignment::Right][cell.col as usize];
        let got = doc.doc_info.para_shapes[cell.paragraphs[0].para_shape_id as usize].alignment;
        assert_eq!(got, expected, "{}행 {}열", cell.row, cell.col);
    }
}

/// HWPX 왕복이 열별 정렬을 보존한다.
#[test]
fn hwpx_round_trip_keeps_per_column_alignment() {
    let doc = doc_of(r#","cell_align":["center","left","right"]"#);
    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let reparsed = rhwp::parser::parse_document(&hwpx).expect("HWPX 재파싱");
    assert_eq!(
        cell_alignments(&reparsed),
        vec![Alignment::Center, Alignment::Left, Alignment::Right]
    );
}

/// 잘못된 입력은 무엇이 왜 틀렸는지 알려 주며 즉시 실패한다.
#[test]
fn invalid_cell_align_is_rejected_with_a_hint() {
    let short = parse_scaffold_str(&spec_json(r#","cell_align":["left","center"]"#))
        .expect_err("열 수와 다른 목록은 거부해야 한다");
    let short = short.to_string();
    assert!(
        short.contains("cell_align") && short.contains("열 수"),
        "길이 불일치 오류에 이유가 있어야 한다: {short}"
    );

    let typo = parse_scaffold_str(&spec_json(r#","cell_align":"middle""#))
        .expect_err("지원하지 않는 값은 거부해야 한다");
    let typo = typo.to_string();
    assert!(
        typo.contains("middle") && typo.contains("justify|left|center|right"),
        "오타 오류에 지원 값 목록이 있어야 한다: {typo}"
    );

    let misplaced = parse_scaffold_str(
        r#"{"version":"1","blocks":[{"type":"paragraph","text":"x","cell_align":"left"}]}"#,
    )
    .expect_err("paragraph 블록의 cell_align 은 거부해야 한다");
    assert!(
        misplaced.to_string().contains("table 블록 전용"),
        "잘못 쓴 자리를 알려 줘야 한다: {misplaced}"
    );
}

/// 주장의 의미 확인 — 좁은 열에서 양쪽 정렬은 글자를 벌리고 `left` 는 벌리지 않는다.
///
/// `samples/issue7232/` 의 두 입력은 같은 명세에서 `cell_align` 만 다르게 만든 뒤 같은
/// 열 폭(4000,8000,24000,5950)을 적용한 것이다. 한/글 2020 PDF 로는 글자마다
/// 0.36~0.48pt 차이가 났다(모듈 주석). 여기서는 rhwp 조판의 같은 방향을 고정한다.
#[test]
fn justify_spreads_glyphs_and_left_does_not() {
    fn line_width(rel: &str) -> f64 {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
        let core =
            rhwp::document_core::DocumentCore::from_bytes(&std::fs::read(path).expect("픽스처"))
                .expect("문서 로드");
        let tree = core.build_page_render_tree(0).expect("render tree");
        fn walk(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                if run.display_or_text().starts_with("INSERT") {
                    out.push((node.bbox.x, node.bbox.width));
                }
            }
            for child in &node.children {
                walk(child, out);
            }
        }
        let mut found = Vec::new();
        walk(&tree.root, &mut found);
        assert!(!found.is_empty(), "{rel}: 'INSERT…' 런을 못 찾았다");
        found.iter().map(|(_, w)| *w).fold(0.0_f64, f64::max)
    }

    let justify = line_width("samples/issue7232/cell_align_justify.hwpx");
    let left = line_width("samples/issue7232/cell_align_left.hwpx");
    assert!(
        justify > left + 1.0,
        "같은 글자·같은 열 폭에서 양쪽 정렬이 더 넓어야 한다 (justify {justify:.2}px, left {left:.2}px)"
    );
}
