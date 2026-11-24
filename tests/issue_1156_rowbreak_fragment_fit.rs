//! 관계형·고정형 회귀 — #7769 국민임대 안내문의 금융자산 행 이어받기와 동의서 순서를 보호한다.
//! 주석 갱신 2026-11-23 22:35 KST. 직전 파일 변경 2026-11-23T21:00:34+09:00
//! `eea27261915ad7a99c1cf8df80ab4a252bedca57`.
//! 기준 검증 2026-11-23 KST, 96 DPI Print PDFium과 보정 전 2px raw 실루엣.
//! 35쪽 최저 Native/fresh WASM 98.99/99.00%, 5쪽 98.99/99.00%, 6쪽 99.99/100.00%.
//! 30쪽 99.82/99.81%. 근거는 `mydocs/pr/assets/issue7769/synam/`의 TSV와 확대 비교다.
//!
//! RowBreak 표의 행 내부 조각 때문에 전체 조각이 쪽 경계를 넘으면
//! 그 조각을 현재 쪽에 남기지 않는다.
//!
//! `samples/kps-ai.hwp` 37쪽의 329문단은 32행 2열 RowBreak 표다.
//! 독립 Print의37쪽은 성능 요구사항으로 끝나고38쪽은 데이터 요구사항부터 잇는다.
//! 고정 행 번호 대신 원본 소유·쪽별 내용·본문 포함 관계를 검사한다.
//! v448 Native/fresh 전77쪽 최저90.85010%, 누락·미달0, 전쪽 RGBA 동일.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

fn load_doc(rel_path: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

fn page_dump(rel_path: &str, page_idx: u32) -> String {
    let doc = load_doc(rel_path);
    doc.dump_page_items(Some(page_idx))
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn owned_table(root: &RenderNode, pi: usize, ci: usize) -> &RenderNode {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    nodes
        .into_iter()
        .find(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
        if t.para_index == Some(pi) && t.control_index == Some(ci))
        })
        .expect("원본 표의 실제 쪽 조각")
}

fn subtree_text(node: &RenderNode) -> String {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
        _ => String::new(),
    };
    node.children.iter().fold(own, |mut text, child| {
        text.push_str(&subtree_text(child));
        text
    })
}

#[test]
fn kps_ai_page37_defers_overflowing_split_row_slice() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/kps-ai.hwp");
    let core = DocumentCore::from_bytes(&fs::read(path).expect("KPS 원본")).unwrap();
    let (pi, ci, original) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, paragraph)| {
            paragraph
                .controls
                .iter()
                .enumerate()
                .find_map(|(ci, control)| {
                    let Control::Table(table) = control else {
                        return None;
                    };
                    table
                        .cells
                        .iter()
                        .any(|c| {
                            c.paragraphs
                                .iter()
                                .any(|p| compact(&p.text) == "4.성능요구사항")
                        })
                        .then_some((pi, ci, table))
                        .filter(|(_, _, table)| {
                            table.cells.iter().any(|c| {
                                c.paragraphs.iter().any(|p| compact(&p.text) == "작성방법")
                            })
                        })
                })
        })
        .expect("제안서 작성방법 표의 원본 소유자");
    let boundary = original
        .cells
        .iter()
        .find(|c| {
            c.paragraphs
                .iter()
                .any(|p| compact(&p.text) == "5.데이터요구사항")
        })
        .expect("독립 Print의 다음 쪽 시작 항목")
        .row;
    let first = core.build_page_render_tree(36).unwrap();
    let next = core.build_page_render_tree(37).unwrap();
    let first_table = owned_table(&first.root, pi, ci);
    let next_table = owned_table(&next.root, pi, ci);
    assert!(
        first_table
            .children
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::TableCell(c) => Some(c),
                _ => None,
            })
            .all(|c| c.row < boundary),
        "데이터 항목의 일부를 앞 조각에 남기지 않는다"
    );
    assert!(next_table.children.iter().any(|n| matches!(&n.node_type,
        RenderNodeType::TableCell(c) if c.row == boundary)));
    // 두 조각은 원본의 모든 행을 보존하고 반복 머리행만 공유한다.
    let left_rows = |table: &RenderNode| {
        table
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TableCell(cell) if cell.col == 0 => Some(cell.row),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>()
    };
    let rows = left_rows(first_table);
    let following = left_rows(next_table);
    let headers = first_table
        .children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell)
                if cell.col == 0 && compact(&subtree_text(node)) == "항목" =>
            {
                Some(cell.row)
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        rows.intersection(&following)
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        headers,
        "반복 머리행만 두 조각에 속한다"
    );
    assert_eq!(
        rows.union(&following).count(),
        usize::from(original.row_count),
        "두 조각은 원본의 모든 행을 보존한다"
    );
    for (tree, table) in [(&first, first_table), (&next, next_table)] {
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        let column = nodes
            .into_iter()
            .find(|n| {
                matches!(n.node_type, RenderNodeType::Column(_))
                    && n.children.iter().any(|c| c.id == table.id)
            })
            .expect("조각의 소유 단");
        assert!(
            table.bbox.y >= column.bbox.y - 1e-8
                && table.bbox.y + table.bbox.height <= column.bbox.y + column.bbox.height + 1e-8,
            "표 조각 전체는 소유 본문 단 안에 있어야 한다"
        );
    }
    let first_text = compact(&core.extract_page_text_native(36).unwrap());
    let next_text = compact(&core.extract_page_text_native(37).unwrap());
    assert!(first_text.contains("4.성능요구사항") && !first_text.contains("5.데이터요구사항"));
    assert!(!next_text.contains("4.성능요구사항") && next_text.contains("5.데이터요구사항"));
    assert_kps_ai_source_cell_ownership();
}

/// 독립 PDF의 성능/데이터 요구사항 경계를 내용과 원본 셀 소속으로 검사한다.
/// 절대 쪽 번호·행 컷 숫자와 dump 문자열 형식은 기대값으로 사용하지 않는다.
fn assert_kps_ai_source_cell_ownership() {
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn compact(text: &str) -> String {
        text.chars()
            .filter(|ch| !ch.is_whitespace() && !ch.is_control())
            .collect()
    }
    fn text(node: &RenderNode) -> String {
        if !node.visible || node.editor_only {
            return String::new();
        }
        let mut result = match &node.node_type {
            RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
            _ => String::new(),
        };
        for child in &node.children {
            result.push_str(&text(child));
        }
        result
    }
    fn tables<'a>(
        node: &'a RenderNode,
        body: Option<(f64, f64)>,
        owner: (usize, usize, usize),
        found: &mut Vec<(&'a RenderNode, (f64, f64))>,
    ) {
        if !node.visible || node.editor_only {
            return;
        }
        let body = if matches!(node.node_type, RenderNodeType::Body { .. }) {
            Some((node.bbox.y, node.bbox.y + node.bbox.height))
        } else {
            body
        };
        if let RenderNodeType::Table(meta) = &node.node_type {
            if (meta.section_index, meta.para_index, meta.control_index)
                == (Some(owner.0), Some(owner.1), Some(owner.2))
                && meta.cell_context.is_none()
            {
                found.push((node, body.expect("대상 표는 본문 소속이다")));
            }
        }
        for child in &node.children {
            tables(child, body, owner, found);
        }
    }
    fn cells<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
        if !node.visible || node.editor_only {
            return;
        }
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            found.push(node);
            return;
        }
        for child in &node.children {
            cells(child, found);
        }
    }
    fn lines_inside_cell(node: &RenderNode, top: f64, bottom: f64) {
        if !node.visible || node.editor_only {
            return;
        }
        if matches!(node.node_type, RenderNodeType::TextLine(_)) && !compact(&text(node)).is_empty()
        {
            assert!(node.bbox.y >= top - 0.5, "셀 위로 벗어난 글줄");
            assert!(
                node.bbox.y + node.bbox.height <= bottom + 0.5,
                "셀 아래로 벗어난 글줄"
            );
        }
        for child in &node.children {
            lines_inside_cell(child, top, bottom);
        }
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/kps-ai.hwp");
    let core = rhwp::document_core::DocumentCore::from_bytes(&fs::read(path).unwrap())
        .expect("KPS 원본 로드");
    let mut owners = Vec::new();
    for (section, value) in core.document().sections.iter().enumerate() {
        for (paragraph, host) in value.paragraphs.iter().enumerate() {
            if !compact(&host.text).contains("제안서세부작성지침") {
                continue;
            }
            for (control, value) in host.controls.iter().enumerate() {
                if let Control::Table(table) = value {
                    owners.push((section, paragraph, control, table));
                }
            }
        }
    }
    assert_eq!(owners.len(), 1, "제안서 세부작성지침 표의 원본 소속");
    let &(section, paragraph, control, source) = &owners[0];
    let owner = (section, paragraph, control);
    let mut fragments = Vec::new();
    let mut page_texts = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("본문 렌더");
        let mut found = Vec::new();
        tables(&tree.root, None, owner, &mut found);
        for (table, (top, bottom)) in found {
            assert!(table.bbox.y >= top - 0.5, "표가 본문 위를 넘음");
            assert!(
                table.bbox.y + table.bbox.height <= bottom + 0.5,
                "표가 본문 아래를 넘음"
            );
            fragments.push((page, table.clone()));
            page_texts.push(compact(&text(&tree.root)));
        }
    }
    assert_eq!(fragments.len(), 2, "독립 PDF의 앞 표와 이어받은 표");
    assert_eq!(
        fragments[1].0,
        fragments[0].0 + 1,
        "표 조각 사이에 빈 쪽이 없음"
    );
    assert!(page_texts[0].contains("제안서세부작성지침"));
    let front = compact(&text(&fragments[0].1));
    let next = compact(&text(&fragments[1].1));
    assert!(front.contains("성능요구사항"));
    assert!(
        !front.contains("데이터요구사항"),
        "PDF의 다음 조각 항목을 앞쪽에 남기지 않음"
    );
    assert!(next.contains("데이터요구사항"));
    assert!(
        !next.contains("성능요구사항"),
        "완료한 앞 항목을 이어받기에서 반복하지 않음"
    );

    let headers = source.leading_header_rows();
    let mut rendered_cells = Vec::new();
    for (_, table) in &fragments {
        cells(table, &mut rendered_cells);
    }
    for (index, source_cell) in source.cells.iter().enumerate() {
        let copies: Vec<_> = rendered_cells
            .iter()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::TableCell(meta)
                if meta.model_cell_index == Some(index as u32))
            })
            .collect();
        let expected = if headers.contains(&(source_cell.row as usize)) {
            fragments.len()
        } else {
            1
        };
        assert_eq!(
            copies.len(),
            expected,
            "원본 셀 {index}의 누락/중복; 반복 제목행만 예외"
        );
        let expected_text = compact(
            &source_cell
                .paragraphs
                .iter()
                .map(|paragraph| paragraph.text.as_str())
                .collect::<String>(),
        );
        for cell in copies {
            assert!(
                compact(&text(cell)).contains(&expected_text),
                "원본 셀 {index} 내용 보존"
            );
            lines_inside_cell(cell, cell.bbox.y, cell.bbox.y + cell.bbox.height);
        }
    }
}

#[test]
fn synam_001_page14_uses_row_budget_after_repeated_header() {
    let sample = "samples/synam-001.hwp";

    let page13 = page_dump(sample, 12);
    assert!(
        page13.contains("PartialTable   pi=140 ci=0  rows=2..7"),
        "page 13 should include the first visible slice of row 6 after repeated header rows:\n{page13}"
    );
    assert!(
        page13.contains("start_cut=[2, 19] end_cut=[2, 13]"),
        "page 13 should continue row 2 and cut into row 6 instead of deferring row 6:\n{page13}"
    );

    let page14 = page_dump(sample, 13);
    assert!(
        page14.contains("start_cut=[2, 13]"),
        "page 14 should continue from the row 6 cut already started on page 13:\n{page14}"
    );
}

#[test]
fn synam_001_financial_assets_cell_continues_between_pages_five_and_six() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/synam-001.hwp");
    let core = DocumentCore::from_bytes(&fs::read(path).expect("synam-001 원본")).unwrap();
    let (para, control, row, column) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, paragraph)| {
            paragraph
                .controls
                .iter()
                .enumerate()
                .find_map(|(ci, ctrl)| {
                    let Control::Table(table) = ctrl else {
                        return None;
                    };
                    let cell = table.cells.iter().find(|cell| {
                        cell.paragraphs
                            .iter()
                            .any(|p| p.text.contains("비상장주식의 평가"))
                            && cell.paragraphs.iter().any(|p| p.text.contains("보통예금"))
                    })?;
                    Some((pi, ci, cell.row, cell.col))
                })
        })
        .expect("금융자산 설명의 원본 셀");
    assert_eq!(core.page_count(), 35);

    let split_cell = |page| {
        let tree = core
            .build_page_render_tree(page)
            .expect("원본 표가 나뉜 쪽");
        let table = owned_table(&tree.root, para, control);
        let mut cells = table.children.iter().filter(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell)
                if cell.row == row && cell.col == column)
        });
        let cell = cells.next().expect("해당 쪽의 금융자산 설명 셀");
        assert!(
            cells.next().is_none(),
            "같은 셀 조각이 한 쪽에 중복되지 않는다"
        );
        compact(&subtree_text(cell))
    };
    let first = split_cell(4);
    let second = split_cell(5);
    assert!(first.contains("보통예금") && first.ends_with("비상장주"));
    assert!(second.starts_with("식의평가에관하여는") && second.contains("연금보험"));
    assert_eq!(
        format!("{first}{second}")
            .matches("비상장주식의평가에관하여는")
            .count(),
        1
    );
}

#[test]
fn synam_visible_float_title_precedes_its_cell_and_the_signature_line() {
    let document = DocumentCore::from_bytes(include_bytes!("../samples/synam-001.hwp"))
        .expect("국민임대 안내문 원본");
    let tree = document
        .build_page_render_tree(29)
        .expect("30쪽 개인정보 동의");
    let form = owned_table(&tree.root, 229, 0);
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let column = nodes
        .iter()
        .copied()
        .find(|node| {
            matches!(node.node_type, RenderNodeType::Column(_))
                && node.children.iter().any(|child| child.id == form.id)
        })
        .expect("동의서 표를 담는 본문 단");
    let title = column
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(229)
            && compact(&subtree_text(node)) == "7.[필수]")
        })
        .expect("표와 같은 문단에 속한 제목");
    let signature = column
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(231)
            && compact(&subtree_text(node)).starts_with("20"))
        })
        .expect("표 뒤의 서명 날짜");
    let cell = form
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .expect("동의서 설명 셀");
    assert!(compact(&subtree_text(cell)).starts_with("본인은위1~6호"));
    assert!(title.bbox.y + title.bbox.height <= form.bbox.y);
    assert!(form.bbox.y + form.bbox.height <= signature.bbox.y);
}
