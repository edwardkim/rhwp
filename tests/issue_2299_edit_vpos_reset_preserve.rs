//! Issue #2299 — 편집발 vpos 재계산이 저장 단/쪽-상대 vpos 리셋(단 경계 인코딩)을
//! 파괴해 다단 단-밴드가 소멸하던 회귀 핀.
//!
//! shortcut.hwp(1단 제목 + 2단 배분 zone 이 카테고리마다 반복, 저장 리셋 76곳 전부
//! vpos=0)의 앞 문단을 편집하면 `recalculate_section_vpos` 의 선형 누적이 하류 리셋
//! 전부를 덮어써 typeset(#321/#470/#702)·pagination 의 단/쪽 진행 신호가 소멸 —
//! 7쪽(한글 2022 정합)이 9쪽으로, 0쪽 Column 이 col=[0]만으로 붕괴하던 결함.
//! 편집 문단의 높이가 변하지 않는 편집(1자 삽입)에서도 발생한다.
//!
//! 수정: 직전 문단의 "이동 전(저장)" end 대비 현재 문단의 저장 first 가 감소하면
//! 단/쪽 경계 인코딩으로 보고 next_vpos 를 저장 first 로 되돌려 보존.
//! 근거·임계 불요 사유는 `recalculate_section_vpos` doc comment 참조.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn load_shortcut() -> HwpDocument {
    let bytes = std::fs::read("samples/basic/shortcut.hwp").expect("read shortcut.hwp");
    HwpDocument::from_bytes(&bytes).expect("parse shortcut.hwp")
}

fn collect_cols(node: &RenderNode, out: &mut Vec<u16>) {
    if let RenderNodeType::Column(c) = node.node_type {
        out.push(c);
    }
    for child in &node.children {
        collect_cols(child, out);
    }
}

/// 0쪽 렌더트리의 Column 노드 col 인덱스 나열 (문서 순서).
fn page0_cols(doc: &HwpDocument) -> Vec<u16> {
    let tree = doc.build_page_render_tree(0).expect("page 0 render tree");
    let mut cols = Vec::new();
    collect_cols(&tree.root, &mut cols);
    cols
}

#[test]
fn front_paragraph_edit_preserves_column_bands_and_page_count() {
    let mut doc = load_shortcut();
    assert_eq!(doc.page_count(), 7, "원본 쪽수 전제 (한글 2022 = 7쪽)");
    let cols_before = page0_cols(&doc);
    assert!(
        cols_before.contains(&1),
        "원본 0쪽에 우측 단(col=1) 밴드 존재 전제: {cols_before:?}"
    );

    doc.insert_text_native(0, 0, 0, "X").expect("insert");

    assert_eq!(
        doc.page_count(),
        7,
        "앞 문단 1자 삽입 후에도 7쪽 유지 — 9쪽이면 저장 vpos 리셋 파괴(#2299) 회귀"
    );
    assert_eq!(
        page0_cols(&doc),
        cols_before,
        "편집 후 0쪽 단-밴드 배치 보존 — col=1 소멸이면 #2299 회귀"
    );
}

#[test]
fn delete_edit_preserves_layout() {
    let mut doc = load_shortcut();
    doc.delete_text_native(0, 2, 0, 1).expect("delete");
    assert_eq!(doc.page_count(), 7, "삭제 편집 후에도 7쪽 유지");
    assert!(
        page0_cols(&doc).contains(&1),
        "삭제 편집 후에도 우측 단(col=1) 밴드 보존"
    );
}

#[test]
fn editing_reset_paragraph_itself_preserves_layout() {
    // pi=16 은 0쪽 2단 zone 의 col1 첫 문단(저장 vpos=0 리셋 문단).
    // 리셋 문단 자체를 편집해도 reflow 가 첫 LineSeg vpos 를 보존하고
    // 재계산이 그 리셋을 유지해야 한다.
    let mut doc = load_shortcut();
    // 전제 고정: 픽스처/파서 드리프트로 pi=16 이 리셋 문단이 아니게 되면 이
    // 테스트는 front-edit 케이스의 중복으로 퇴화한다 — 명시적으로 단언한다.
    assert_eq!(
        doc.document().sections[0].paragraphs[16]
            .line_segs
            .first()
            .map(|ls| ls.vertical_pos),
        Some(0),
        "전제: pi=16 은 저장 vpos=0 리셋 문단이어야 함 (픽스처 드리프트 감지)"
    );
    doc.insert_text_native(0, 16, 0, "X")
        .expect("insert at reset para");
    assert_eq!(doc.page_count(), 7, "리셋 문단 자체 편집 후에도 7쪽 유지");
    assert!(
        page0_cols(&doc).contains(&1),
        "리셋 문단 자체 편집 후에도 col=1 밴드 보존"
    );
}

#[test]
fn last_paragraph_edit_stays_stable() {
    // 마지막 문단은 아래 재계산 대상이 없어 수정 전에도 7쪽이 유지되던 케이스 —
    // 보존 로직 추가 후에도 불변임을 고정한다.
    let mut doc = load_shortcut();
    let last = doc.document().sections[0].paragraphs.len() - 1;
    doc.insert_text_native(0, last, 0, "X")
        .expect("insert at last para");
    assert_eq!(doc.page_count(), 7, "끝 문단 편집은 7쪽 불변");
}

/// 본문 문단 vpos 의 무겹침 불변식: 각 문단은 직전 문단 흐름 end 이후에서
/// 시작하거나(연속), 직전 first 이하로 내려간 깨끗한 리셋(단/쪽 경계)이어야 한다.
/// 가짜-핀 회귀(성장 편집·삽입·붙여넣기 고착)는 prev_first < current < prev_end
/// 의 "중간 동결"을 만들므로 이 단언이 잡는다.
fn assert_no_mid_freeze(doc: &HwpDocument, label: &str) {
    let paras = &doc.document().sections[0].paragraphs;
    let mut prev: Option<(usize, i32, i32)> = None; // (pi, first, end)
    for (pi, p) in paras.iter().enumerate() {
        let Some(first_seg) = p.line_segs.first() else {
            continue;
        };
        let first = first_seg.vertical_pos;
        let last = p.line_segs.last().unwrap();
        let advance = if last.line_height > last.text_height && last.text_height > 0 {
            last.text_height + last.line_spacing
        } else {
            last.line_height + last.line_spacing
        };
        let end = last.vertical_pos + advance;
        if let Some((ppi, prev_first, prev_end)) = prev {
            assert!(
                first >= prev_end || first <= prev_first,
                "{label}: pi={pi} first={first} 가 직전 pi={ppi} 구간({prev_first}..{prev_end}) \
                 안에 동결됨 — 가짜 리셋 핀(#2314 리뷰 확증 회귀)"
            );
        }
        prev = Some((pi, first, end));
    }
}

#[test]
fn growth_edit_pushes_followers_down() {
    // [리뷰 확증 회귀 1] 문단이 저장 gap 보다 크게 자라는 편집(줄바꿈 유발) 후에도
    // 다음 문단이 가짜 리셋으로 동결되지 않고 아래로 밀려야 한다.
    let bytes = std::fs::read("samples/basic/english.hwp").expect("read english.hwp");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse english.hwp");
    let long = "가나다라마바사아자차카타파하 ".repeat(20);
    doc.insert_text_native(0, 2, 0, &long).expect("grow para 2");

    let paras = &doc.document().sections[0].paragraphs;
    let p2_last = paras[2].line_segs.last().unwrap();
    let p2_end = p2_last.vertical_pos + p2_last.line_height + p2_last.line_spacing;
    let p3_first = paras[3].line_segs.first().unwrap().vertical_pos;
    assert!(
        p3_first >= p2_end,
        "성장 편집 후 다음 문단이 밀려야 함: p3_first={p3_first} < p2_end={p2_end} (동결 회귀)"
    );
    assert_no_mid_freeze(&doc, "growth_edit");
}

#[test]
fn insert_empty_paragraph_pushes_displaced_paragraph_down() {
    // [리뷰 확증 회귀 2] 빈 문단 삽입 시 밀려난 후속 문단(저장 vpos 0)이 신규
    // placeholder 와 겹치지 않고 아래로 이동해야 한다.
    let bytes = std::fs::read("samples/basic/english.hwp").expect("read english.hwp");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse english.hwp");
    let pages_before = doc.page_count();
    doc.insert_paragraph_native(0, 0)
        .expect("insert empty at 0");

    let paras = &doc.document().sections[0].paragraphs;
    let p0_last = paras[0].line_segs.last().unwrap();
    let p0_end = p0_last.vertical_pos + p0_last.line_height + p0_last.line_spacing;
    let p1_first = paras[1].line_segs.first().unwrap().vertical_pos;
    assert!(
        p1_first >= p0_end,
        "삽입 후 기존 첫 문단이 밀려야 함: p1_first={p1_first} < p0_end={p0_end} (겹침 회귀)"
    );
    // 빈 줄 1개가 정당하게 만들 수 있는 증가는 최대 1쪽 (넘침).
    let pages_after_insert = doc.page_count();
    assert!(
        pages_after_insert <= pages_before + 1,
        "빈 문단 1개 삽입에 쪽수 {pages_before}→{pages_after_insert} — 팬텀 쪽나눔 회귀"
    );
    assert_no_mid_freeze(&doc, "insert_empty_paragraph");

    // 고착류 회귀는 후속 편집마다 팬텀이 증식한다 — 편집 후 쪽수 안정성을 고정.
    doc.insert_text_native(0, 2, 0, "X").expect("edit below");
    assert_eq!(
        doc.page_count(),
        pages_after_insert,
        "후속 1자 편집으로 쪽수 변동 — placeholder 고착 회귀"
    );
    assert_no_mid_freeze(&doc, "insert_then_edit");
}

#[test]
fn paste_then_edit_does_not_entrench_placeholders() {
    // [리뷰 확증 회귀 3] 다중 문단 붙여넣기가 남긴 신규 문단 좌표가 이후 편집에서
    // 가짜 리셋으로 고착되지 않아야 한다 (붙여넣기 직후 흐름 재연결 계약).
    let mut doc = load_shortcut();
    doc.copy_selection_native(0, 2, 0, 4, 1).expect("copy");
    doc.paste_internal_native(0, 98, 0).expect("paste");
    assert_no_mid_freeze(&doc, "after_paste");
    // 붙여넣기 직후 신규 문단이 흐름 좌표에 연결돼야 한다 — placeholder 0 이
    // 남으면 "깨끗한 리셋"으로 위장해 무겹침 단언을 통과하면서 이후 편집에서
    // 가짜 단/쪽 경계로 굳는다 (리뷰 실측: pi=98/99 가 0 에 고착).
    for pi in [98usize, 99] {
        let first = doc.document().sections[0].paragraphs[pi]
            .line_segs
            .first()
            .map(|ls| ls.vertical_pos)
            .unwrap_or(-1);
        assert!(
            first > 0,
            "붙여넣은 pi={pi} 가 placeholder vpos {first} 에 방치됨 — 흐름 재연결 회귀"
        );
    }
    let pages_after_paste = doc.page_count();

    // 고착류 회귀는 이후 편집이 placeholder 를 가짜 리셋으로 굳히며 쪽수를 바꾼다
    // (리뷰 실측: 수정 전 paste 후 편집에서 팬텀 쪽 고착). 콘텐츠가 실제로 늘어난
    // 붙여넣기 자체의 쪽수 증가는 정당하므로, "후속 1자 편집의 쪽수 불변"을 계약으로.
    doc.insert_text_native(0, 0, 0, "X").expect("edit above");
    assert_no_mid_freeze(&doc, "after_paste_then_edit");
    assert_eq!(
        doc.page_count(),
        pages_after_paste,
        "붙여넣기 후 1자 편집으로 쪽수 변동 — placeholder 고착 회귀"
    );
}

#[test]
fn edit_above_tac_host_does_not_pin_follower() {
    // [리뷰 확증 회귀 4] TAC(글자처럼) 개체 호스트 줄은 저장 vpos 체인이 th 기준
    // 전진(lh>th)이므로, lh 기준 end 로 리셋을 감지하면 호스트 다음 문단이 가짜
    // 핀된다. 로드 경로와 동일한 th-관례 전진을 고정한다.
    for sample in [
        "samples/tac-verify/scenario-a-before.hwp",
        "samples/test-image.hwp",
    ] {
        let Ok(bytes) = std::fs::read(sample) else {
            continue;
        };
        let Ok(mut doc) = HwpDocument::from_bytes(&bytes) else {
            continue;
        };
        let pages_before = doc.page_count();
        if doc.insert_text_native(0, 0, 0, "X").is_err() {
            continue;
        }
        assert_no_mid_freeze(&doc, sample);
        assert_eq!(
            doc.page_count(),
            pages_before,
            "{sample}: 1자 삽입으로 쪽수 변동 — TAC 호스트 가짜 리셋 회귀"
        );
    }
}
