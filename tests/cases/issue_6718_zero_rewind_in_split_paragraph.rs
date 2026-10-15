//! [Issue #6718 잔여] `vpos == 0` 되감김 승격의 둘째 겹(`ladder_page_is_full`)이
//! **옳은 자리까지 걷어내던** 것을 되살린다.
//!
//! 주축은 `issue_6718_native_hwp5_zero_vpos_rewind` 가 잠근다. 그 시험 주석이 남긴
//! 미해결 축이 이 시험의 대상이다 — `27469` 의 `pi=47`(7쪽)은 따라야 하는 자리인데
//! 예산이 1.39줄 남아서 `ladder_page_is_full` 에 걸린다. `#2070` 시장구조조사의
//! `pi=343`(1.48줄)·`pi=1088`(1.10줄)과 슬랙이 줄 단위로 뒤섞여 **슬랙·시작위치·
//! 낙폭·직전 항목·문단 수·태그** 어느 축으로도 갈리지 않았다.
//!
//! ## 갈리는 축 — 이 승격이 쪽 경계를 새로 만드는가
//!
//! 문단이 이 조각으로 끝나지 않으면(`end_line < line_count`) 남은 줄은 어차피 다음
//! 쪽으로 넘어간다. 경계는 이미 서 있고 사다리는 그 경계를 **어디에 둘지**만 말한다.
//! 그때는 예산에 여유가 남아도 파일이 적어 둔 자리를 따르는 것이 옳다.
//!
//! ```text
//!   27469 pi=47   end_line 5 < line_count 8   이미 쪼개진다      → 따른다
//!   #2070 pi=343  end_line 5 = line_count 5   여기서 새로 끊는다 → 안 따른다
//!   #2070 pi=1088 end_line 5 = line_count 5   같음
//! ```
//!
//! ## 실측
//!
//! ```text
//!                                   devel        이 수정
//!   27469        overflow             5             3      (pi=47 닫힘)
//!                8쪽 마지막 줄     용지 밖        본문 안
//!   issue5966    off-canvas           3             0
//!                overflow            17             8
//!   #2070        pageCount          315           315      (불변)
//! ```
//!
//! 이 시험은 **`issue5966`** 을 쓴다. `27469` 는 주축 시험이 이미 잡고 있고, 이쪽은
//! 같은 결함이 **다른 문서에서 용지 밖 출력으로 나타나는 것**을 잠근다 — 143쪽 문서의
//! 60쪽에서 본문 세 줄이 용지(1122.5px) 아래 1216.8px 에 그려져 인쇄에서 사라졌다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue5966/1130000-202100008_franchise_review_report.hwp";
/// 0-based — 되감김 문단이 놓이는 물리 60쪽.
const PAGE_INDEX: u32 = 59;

/// 꼬리말(쪽번호)은 본문 하한 아래에 있는 것이 정상이다 — `Body` 아래만 모은다.
fn collect_body_runs<'a>(node: &'a RenderNode, in_body: bool, out: &mut Vec<&'a RenderNode>) {
    let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    if in_body {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if !run.text.trim().is_empty() {
                out.push(node);
            }
        }
    }
    for child in &node.children {
        collect_body_runs(child, in_body, out);
    }
}

/// 본문 표의 원본 셀 내용과 실제 셀 소유를 대조해, 빈 출력으로 포함 검사를 통과하지 못하게 한다.
fn collect_owner_tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(569)) {
        out.push(node);
    }
    for child in &node.children {
        collect_owner_tables(child, out);
    }
}
fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn split_paragraph_follows_the_stored_rewind_even_with_budget_left() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue5966 sample");
    assert_eq!(
        document.page_count(),
        // 과거 rhwp 실측 143쪽 대신 동일 원본의 독립 한컴 2020 Print PDF를 따른다.
        // 현재 전142쪽 Native/fresh WASM 최저93.16816%와 60쪽 본문/셀 소속을 확인했다.
        // 증거: mydocs/pr/assets/semanticist21-20261008/franchise-whole-dual-current-proof-v453/README.md.
        142,
        "독립 Print PDF의 142쪽이어야 한다 — 되감김 승격이 쪽을 새로 만들면 안 된다"
    );

    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p60");
    let page_bottom = tree.root.bbox.y + tree.root.bbox.height;
    let body = tree
        .root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("60쪽 본문 영역");
    let body_bottom = body.bbox.y + body.bbox.height;
    let mut owner_tables = Vec::new();
    collect_owner_tables(body, &mut owner_tables);
    assert_eq!(
        owner_tables.len(),
        1,
        "독립 Print 60쪽의 소유 표는 한 번만 나타난다"
    );
    let owner = owner_tables[0];
    let RenderNodeType::Table(meta) = &owner.node_type else {
        unreachable!()
    };
    let source = &document.document().sections[meta.section_index.expect("표의 구역 소유")]
        .paragraphs[meta.para_index.expect("표의 문단 소유")];
    let Control::Table(source_table) =
        &source.controls[meta.control_index.expect("표의 컨트롤 소유")]
    else {
        panic!("원본 소유 개체는 표다")
    };
    let cells: Vec<_> = owner
        .children
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert_eq!(
        cells.len(),
        source_table.cells.len(),
        "소유 셀 누락이나 중복이 없어야 한다"
    );
    for source_cell in &source_table.cells {
        let selected: Vec<_> = cells.iter().filter(|node| matches!(&node.node_type,
            RenderNodeType::TableCell(cell) if cell.row == source_cell.row && cell.col == source_cell.col)).collect();
        assert_eq!(selected.len(), 1, "원본 행·열의 소유 셀은 하나다");
        let mut runs = Vec::new();
        collect_body_runs(selected[0], true, &mut runs);
        let actual: String = runs
            .iter()
            .filter_map(|node| {
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    Some(run.text.as_str())
                } else {
                    None
                }
            })
            .collect();
        let expected: String = source_cell
            .paragraphs
            .iter()
            .map(|para| para.text.as_str())
            .collect();
        assert_eq!(
            compact(&actual),
            compact(&expected),
            "원본 셀의 내용 순서·보존을 확인한다"
        );
    }
    let mut nodes = Vec::new();
    collect_body_runs(&tree.root, false, &mut nodes);

    let worst = nodes
        .iter()
        .map(|node| node.bbox.y + node.bbox.height)
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        worst.is_finite(),
        "60쪽에 글자가 있어야 한다 — 표본이나 쪽 인덱스가 어긋났다"
    );
    assert!(
        worst <= page_bottom,
        "60쪽 글자가 **용지 밖**으로 나갔다 — 최하단 {worst:.1}px, 용지 {page_bottom:.1}px \
         (회귀 시 1216.8px = +94.3px, 인쇄에서 소실된다)"
    );
    assert!(
        worst <= body_bottom,
        "60쪽 글자가 본문 하한을 넘었다 — 최하단 {worst:.1}px, 하한 {body_bottom:.1}px"
    );
}
