//! [#8924] 관계형·고정형 회귀 — 저장 줄 정보가 없는 문단에서 글자처럼 취급 표가 문단 가용 폭보다
//! 넓어 줄이 이미 넘쳤어도, 표 뒤 전진폭 없는 꼬리(줄 끝 공백·문단 끝)는 표 줄에 남는다.
//!
//! 주석 갱신: 2026-11-11 07:20 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8924/wide_tac_trailing_tail.hwp` — 공개 조례 시행규칙 서식 13620707
//!   (HWP 5.0.2.5, 1쪽). 본문 문단에 저장 줄 정보가 없다. 문단 5·7·9·16 은 왼쪽 2000·오른쪽
//!   1200 HU 여백 문단에 가용 폭보다 넓은 글자처럼 취급 표 하나와 공백 꼬리를 담는다.
//! - 독립 기준: `pdf/issue8924/wide_tac_trailing_tail-2020.pdf`(한/글 2022 12.0.0.4547 Print, 1쪽).
//! - 수정 전 rhwp(devel `e118de8aba`)는 넘친 표 줄 뒤 폭 0 꼬리로 줄을 넘겨, 표마다 아래에 빈
//!   글줄(14.4px)을 하나씩 더 만들고 문단 18~21 을 2쪽으로 밀었다(전체 2쪽).
//!
//! 측정: 2026-11-11, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 devel `e118de8aba` + 이 수정.
//! 1쪽 Native 100.00 / fresh WASM 100.00(수정 전 Native 은 2쪽이라 쪽 대응 불가). 전체 1쪽 = PDF 1쪽,
//! 전쪽 최저 Native 100.00(1쪽)·fresh WASM 100.00(1쪽).
//! 증거: `mydocs/pr/assets/issue_8924/{native,wasm}/`(TSV·review·overlay PNG).
//!
//! 검사: 원본 계약(문단 5·7·9·16 저장 줄 없음, 글자처럼 취급 표 폭 > 문단 가용 폭), 쪽수 1 =
//! 한/글(고정형), 표 문단은 글줄 하나뿐이고 그 줄이 표를 담으며 다음 문단 첫 줄은 표 아래 끝과
//! 겹치지 않고 꼬리 글줄 높이 안에서 이어진다(관계형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8924/wide_tac_trailing_tail.hwp";
const TABLE_PARAS: [usize; 4] = [5, 7, 9, 16];

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

/// 본문 흐름(표 칸 안 제외)의 글줄·표 상자를 소유 문단과 함께 모은다.
#[derive(Default)]
struct Flow {
    tables: Vec<(usize, BoundingBox)>,
    lines: Vec<(usize, BoundingBox)>,
}

fn collect(node: &RenderNode, flow: &mut Flow) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let Some(para) = table.para_index {
                flow.tables.push((para, node.bbox));
            }
            return;
        }
        RenderNodeType::TextLine(line) => {
            if let Some(para) = line.para_index {
                flow.lines.push((para, node.bbox));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, flow);
    }
}

#[test]
fn zero_width_tail_after_overflowing_tac_table_stays_on_the_table_line() {
    let core = load();
    let doc = core.document();
    let section = &doc.sections[0];
    let paragraphs = &section.paragraphs;
    let page_def = &section.section_def.page_def;
    let body_width = i64::from(page_def.width)
        - i64::from(page_def.margin_left)
        - i64::from(page_def.margin_right);

    // 원본 계약: 표 문단은 저장 줄이 없고, 글자처럼 취급 표의 폭이 문단 가용 폭보다 넓다.
    for &index in &TABLE_PARAS {
        let para = &paragraphs[index];
        assert!(
            para.line_segs.iter().all(|seg| seg.tag & (1 << 31) != 0),
            "문단 {index} 은 저장 줄 정보가 없다(합성 줄만)"
        );
        let shape = &doc.doc_info.para_shapes[para.para_shape_id as usize];
        let available = body_width - i64::from(shape.margin_left) - i64::from(shape.margin_right);
        let table_width = para
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Table(table) if table.common.treat_as_char => {
                    Some(i64::from(table.common.width))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("문단 {index} 글자처럼 취급 표"));
        assert!(
            table_width > available,
            "문단 {index} 표 폭 {table_width} > 가용 폭 {available}"
        );
        assert!(!para.text.is_empty(), "문단 {index} 표 뒤 꼬리 글자");
    }

    assert_eq!(core.page_count(), 1, "전체 쪽수 = 한/글 1쪽");

    let page = core.build_page_render_tree(0).expect("1쪽");
    let mut flow = Flow::default();
    collect(&page.root, &mut flow);

    // 모든 문단이 1쪽에 한 번 이상 나온다(누락 없음).
    for index in 0..paragraphs.len() {
        assert!(
            flow.lines.iter().any(|(owner, _)| *owner == index),
            "문단 {index} 이 1쪽에 있다"
        );
    }

    for &index in &TABLE_PARAS {
        let lines: Vec<_> = flow
            .lines
            .iter()
            .filter(|(owner, _)| *owner == index)
            .collect();
        assert_eq!(
            lines.len(),
            1,
            "문단 {index} 은 표 줄 하나뿐(꼬리가 새 줄을 만들지 않음)"
        );
        let line = lines[0].1;
        let table = flow
            .tables
            .iter()
            .find(|(owner, _)| *owner == index)
            .map(|(_, bbox)| *bbox)
            .unwrap_or_else(|| panic!("문단 {index} 표"));
        assert!(
            line.y <= table.y + 0.5 && table.y + table.height <= line.y + line.height + 0.5,
            "문단 {index} 표가 자기 줄 안에 있다: 줄 {line:?}, 표 {table:?}"
        );

        let next = flow
            .lines
            .iter()
            .find(|(owner, _)| *owner == index + 1)
            .map(|(_, bbox)| *bbox)
            .unwrap_or_else(|| panic!("문단 {} 첫 줄", index + 1));
        let gap = next.y - (table.y + table.height);
        assert!(
            gap >= -0.5,
            "문단 {} 첫 줄이 표와 겹치지 않는다: {gap}",
            index + 1
        );
        // 수정 전에는 표 아래에 꼬리 글줄(14.4px)과 그 줄간격이 끼어 이 간격이 그만큼 더 컸다.
        assert!(
            gap < line.y + line.height - (table.y + table.height) + 14.4,
            "문단 {} 첫 줄이 표 줄 바로 뒤에 온다: {gap}",
            index + 1
        );
    }
}
