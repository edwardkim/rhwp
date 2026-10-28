//! #7512 — 글자 모양 복원 뮤테이터의 배치 지연 계약.
//!
//! `begin_batch`~`end_batch` 사이에서 `setCharShapeId`·`setCharShapeIdInCell` 은
//! 재구성·재페이지네이션을 `end_batch` 의 paginate() 1회로 미룬다(깊이 2 이상 셀의
//! `ByPath` 경로와 같다, #4118). 같은 편집을 호출마다 rebuild 한 결과와 배치로 묶은
//! 결과는 쪽 수·모든 쪽의 지오메트리와 SVG·저장 바이트까지 같아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;

const BODY_PARAS: usize = 30;
const CELL_TEXT: &str = "셀 문단의 글자 모양을 여러 번 나눠 되돌리는 실행 취소 경로";

/// 본문 문단 30개와 그 뒤 1×1 표 하나. 30pt 글자 모양 id 도 함께 돌려준다.
fn document() -> (DocumentCore, usize, u32) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    for i in 0..BODY_PARAS {
        if i > 0 {
            core.insert_paragraph_native(0, i).unwrap();
        }
        core.insert_text_native(0, i, 0, &format!("본문 문단 {i}"))
            .unwrap();
    }
    core.insert_paragraph_native(0, BODY_PARAS).unwrap();
    let created: Value =
        serde_json::from_str(&core.create_table_native(0, BODY_PARAS, 0, 1, 1).unwrap()).unwrap();
    let table_para = created["paraIdx"].as_u64().unwrap() as usize;
    core.insert_text_in_cell_native(0, table_para, 0, 0, 0, 0, CELL_TEXT)
        .unwrap();

    // 30pt 모양을 하나 만들고 그 id 만 쓴다 — 첫 글자는 원래 모양으로 되돌린다.
    let normal = core.document().sections[0].paragraphs[0]
        .char_shape_id_at(0)
        .unwrap();
    core.apply_char_format_native(0, 0, 0, 1, r#"{"fontSize":3000}"#)
        .unwrap();
    let big = core.document().sections[0].paragraphs[0]
        .char_shape_id_at(0)
        .unwrap();
    assert_ne!(big, normal);
    core.set_char_shape_id_native(0, 0, 0, 1, normal).unwrap();
    (core, table_para, big)
}

fn body_len(core: &DocumentCore, para: usize) -> usize {
    core.document().sections[0].paragraphs[para]
        .text
        .chars()
        .count()
}

fn snapshot(core: &DocumentCore) -> (u32, Vec<String>, Vec<String>, Vec<u8>) {
    let pages = core.page_count();
    let infos = (0..pages)
        .map(|page| core.get_page_info_native(page).unwrap())
        .collect();
    let svgs = (0..pages)
        .map(|page| core.render_page_svg_native(page).unwrap())
        .collect();
    (pages, infos, svgs, core.export_hwp_native().unwrap())
}

/// 같은 문서에 `edit` 을 배치 없이 한 결과와 배치로 묶은 결과를 비교한다.
///
/// 배치 중에는 쪽을 다시 나누지 않는다 — 호출마다 rebuild 하면 `end_batch` 전에 이미
/// 쪽 수가 늘어 있다.
fn assert_batch_matches_eager(edit: fn(&mut DocumentCore, usize, u32)) {
    let (mut eager, table_para, big) = document();
    let before_pages = eager.page_count();
    edit(&mut eager, table_para, big);
    let eager = snapshot(&eager);
    assert!(
        eager.0 > before_pages,
        "30pt 로 바꾸면 쪽이 늘어야 비교가 의미 있다 ({before_pages} → {})",
        eager.0
    );

    let (mut batched, table_para, big) = document();
    batched.begin_batch_native().unwrap();
    edit(&mut batched, table_para, big);
    assert_eq!(
        batched.page_count(),
        before_pages,
        "배치 중 호출이 매번 재페이지네이션했다"
    );
    batched.end_batch_native().unwrap();
    let batched = snapshot(&batched);

    assert_eq!(batched.0, eager.0, "쪽 수가 같아야 한다");
    assert_eq!(batched.1, eager.1, "모든 쪽의 지오메트리가 같아야 한다");
    assert_eq!(batched.2, eager.2, "모든 쪽의 SVG 가 같아야 한다");
    assert_eq!(batched.3, eager.3, "저장 바이트가 같아야 한다");
}

/// 본문 문단마다 `setCharShapeId`, 셀 문단은 다섯 글자씩 `setCharShapeIdInCell`.
#[test]
fn batched_set_char_shape_id_matches_eager_rebuild() {
    assert_batch_matches_eager(|core, table_para, big| {
        for para in 0..BODY_PARAS {
            let len = body_len(core, para);
            core.set_char_shape_id_native(0, para, 0, len, big).unwrap();
        }
        let len = CELL_TEXT.chars().count();
        for start in (0..len).step_by(5) {
            let end = len.min(start + 5);
            core.set_char_shape_id_in_cell_native(0, table_para, 0, 0, 0, start, end, big)
                .unwrap();
        }
    });
}

/// 같은 지연 꼬리를 쓰는 `setCharShapeRuns`(본문)도 배치 뒤 새 모양으로 그린다.
#[test]
fn batched_set_char_shape_runs_matches_eager_rebuild() {
    assert_batch_matches_eager(|core, _, big| {
        for para in 0..BODY_PARAS {
            let len = body_len(core, para);
            let runs = format!(r#"[{{"startOffset":0,"endOffset":{len},"charShapeId":{big}}}]"#);
            core.set_char_shape_runs_native(0, para, 0, len, &runs)
                .unwrap();
        }
    });
}
