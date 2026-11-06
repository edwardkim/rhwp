//! [#8686] 관계형·고정형 회귀 — 칸 안 문단 기준 자리차지(글 위아래) 표는 흐름을 전진시키지
//! 않는다. 세로 오프셋 위 공간에 저장된 뒤 문단은 그 자리에 놓이고, 칸 첫 문단의 저장 앞
//! 여백도 중첩 표가 있는 칸에서 흐름에 들어간다.
//!
//! 주석 갱신: 2026-11-07 05:40 (KST) — 측정값은 아래 표
//!
//! - 입력: `samples/issue8686/cell_float_offset_space.hwp` — 속초시 장사시설 사용신청서
//!   (공개 조례 서식, 한/글 2018 저장 HWP, 1쪽) 그대로. 큰 표 마지막 칸(r=12, 12문단, 세로
//!   가운데 정렬)의 문단 6 「※ 개인정보는 …」 에 1×2 표(글자처럼 취급 아님, 글 위아래, 문단
//!   기준, 위 정렬, 세로 오프셋 3902)가 있다. 문단 7 「이용ㆍ제공되지 않습니다.」 는 문단 6
//!   바로 뒤(12744 = 11052 + 1300 + 392)에 저장돼 있어 표 띠(11052 + 3902 = 14954 부터) 위
//!   오프셋 공간에 든다. 칸 첫 문단의 저장 첫 줄은 vpos 600(앞 여백)이다.
//! - 독립 기준: 같은 입력의 한/글 2022 Print PDF `pdf/issue8686_cell_float_offset_space-2020.pdf`
//!   (1쪽). 96dpi 래스터에서 「이용ㆍ제공되지 않습니다.」 줄이 첫 동의 표 위에 있고, 「※」 줄과의
//!   간격이 저장 줄 간격(1692HU = 22.56px)이다.
//! - 수정 전 rhwp(devel a7feb849e1): 표가 흐름을 표 아래까지 전진시켜 문단 7 을 첫 동의 표
//!   아래에 그렸다. 중첩 표가 있는 칸이라 첫 문단 앞 여백(8px)을 흐름에서 빼, 칸 글 전체가
//!   8px 위였다.
//!
//! 측정(2026-11-07, 코드 c75d2f07de, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣):
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | 쪽 | Native | fresh WASM | 수정 전 Native |
//! |---|---|---|---|---|---|
//! | cell_float_offset_space | 1 | 1 | 99.95 | 99.97 | 78.32 |
//!
//! 두 경로 전쪽 최저: Native 99.95, fresh WASM 99.97.
//!
//! 증거: `mydocs/pr/assets/issue_8686/{native,native_base,wasm}/*_silhouette.tsv`,
//! `review/*.png`. 검사는 저장 사다리가 뒤 문단을 표 띠 위 공간에 둔다는
//! 원본 관계(전제), 그린 뒤 문단 줄이 표 위끝보다 위이고 앞 줄과의 간격이 저장 줄 간격인 관계
//! (관계형), 독립 PDF 전체 쪽수(고정형)다. 절대 px 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8686/cell_float_offset_space.hwp";

fn px(hu: i64) -> f64 {
    hu as f64 * 96.0 / 7200.0
}

fn node_text(node: &RenderNode) -> String {
    let mut out = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        out.push_str(&node_text(child));
    }
    out
}

fn text_lines(node: &RenderNode, out: &mut Vec<(String, BoundingBox)>) {
    if let RenderNodeType::TextLine(_) = &node.node_type {
        out.push((node_text(node), node.bbox.clone()));
    }
    for child in &node.children {
        text_lines(child, out);
    }
}

fn tables(node: &RenderNode, out: &mut Vec<(String, BoundingBox)>) {
    if let RenderNodeType::Table(_) = &node.node_type {
        out.push((node_text(node), node.bbox.clone()));
    }
    for child in &node.children {
        tables(child, out);
    }
}

/// 「보호되며」 문단을 품은 칸의 문단들.
fn notice_cell(core: &DocumentCore) -> &[Paragraph] {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|control| match control {
            Control::Table(table) => table.cells.iter().find(|cell| {
                cell.paragraphs
                    .iter()
                    .any(|para| para.text.contains("보호되며"))
            }),
            _ => None,
        })
        .map(|cell| cell.paragraphs.as_slice())
        .expect("안내 칸")
}

fn line_with<'a>(lines: &'a [(String, BoundingBox)], needle: &str) -> &'a BoundingBox {
    let found: Vec<_> = lines
        .iter()
        .filter(|(text, _)| text.contains(needle))
        .collect();
    assert_eq!(found.len(), 1, "「{needle}」 줄은 한 번 그린다: {found:?}");
    &found[0].1
}

#[test]
fn cell_float_table_leaves_offset_space_to_following_paragraph() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    assert_eq!(core.page_count(), 1, "한컴 Print PDF 1쪽");

    // 전제: 호스트 문단의 자리차지 표 띠 위 오프셋 공간에 다음 문단이 저장돼 있다.
    let paras = notice_cell(&core);
    let host_index = paras
        .iter()
        .position(|para| para.text.contains("보호되며"))
        .expect("호스트 문단");
    let host = &paras[host_index];
    let next = &paras[host_index + 1];
    let table = host
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("호스트 문단의 표");
    assert!(!table.common.treat_as_char, "글자처럼 취급하지 않는다");
    assert_eq!(table.common.text_wrap, TextWrap::TopAndBottom, "글 위아래");
    assert_eq!(table.common.vert_rel_to, VertRelTo::Para, "문단 기준");
    let host_line = &host.line_segs[0];
    let next_line = &next.line_segs[0];
    let band_top =
        i64::from(host_line.vertical_pos) + i64::from(table.common.vertical_offset as i32);
    let stored_pitch = i64::from(next_line.vertical_pos - host_line.vertical_pos);
    assert_eq!(
        stored_pitch,
        i64::from(host_line.line_height + host_line.line_spacing),
        "다음 문단은 호스트 줄 바로 뒤에 저장된다"
    );
    assert!(
        i64::from(next_line.vertical_pos + next_line.line_height) <= band_top,
        "다음 문단은 표 띠 위끝 {band_top} 위 공간에 저장된다"
    );

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("1쪽 렌더 트리: {e:?}"));
    let mut lines = Vec::new();
    text_lines(&tree.root, &mut lines);
    let mut drawn_tables = Vec::new();
    tables(&tree.root, &mut drawn_tables);

    let host_box = line_with(&lines, "보호되며");
    let next_box = line_with(&lines, "제공되지");
    let consent = drawn_tables
        .iter()
        .filter(|(text, _)| text.contains("개인정보 수집, 이용에 동의"))
        .map(|(_, bbox)| bbox)
        .min_by(|a, b| a.height.total_cmp(&b.height))
        .expect("첫 동의 표");

    // 뒤 문단 줄은 표 위 오프셋 공간에, 호스트 줄 바로 뒤 저장 줄 간격에 놓인다.
    assert!(
        next_box.y + next_box.height <= consent.y + 0.5,
        "「제공되지」 줄 아래끝 {:.2} 은 첫 동의 표 위끝 {:.2} 위",
        next_box.y + next_box.height,
        consent.y
    );
    let pitch = next_box.y - host_box.y;
    assert!(
        (pitch - px(stored_pitch)).abs() <= 0.5,
        "「※」 줄→「제공되지」 줄 {pitch:.2} = 저장 줄 간격 {:.2}",
        px(stored_pitch)
    );
    // 표는 칸 안에서 다음 문단보다 아래에 그대로 남는다.
    assert!(
        consent.y > host_box.y + host_box.height,
        "표는 호스트 줄 아래"
    );
}
