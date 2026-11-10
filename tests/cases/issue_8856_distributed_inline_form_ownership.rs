//! 관계형: 배분 정렬 줄의 마지막 인라인 양식도 자기 표 셀과 용지 안에 남는다.
//! 주석 갱신: 2026-11-10T00:14:44+09:00 (Asia/Seoul), 신규 검사.
//! 독립 입력 request.hwp / PDF basic/request-hwp-2020.pdf 전 1쪽.
//! 최소 글꼴·동일 웹폰트 실측: 보정 전 Native 96.20377%, 보정 후 Native 99.22562%,
//! fresh WASM 99.20457%. 절대 px를 고정하지 않고 선택 항목 수·셀 포함·순서를 검사한다.
//! 근거: mydocs/pr/assets/issue8856/stage1-distributed-forms/validation.json.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    diagnostics::layout_anomaly::{scan_page, AnomalyOptions},
    document_core::DocumentCore,
    model::style::Alignment,
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
};

fn check_forms(node: &RenderNode, cell: Option<BoundingBox>, captions: &mut Vec<String>) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node.bbox)
    } else {
        cell
    };
    if let RenderNodeType::FormObject(form) = &node.node_type {
        let owner = cell.expect("이 입력의 모든 선택 항목은 표 셀이 소유한다");
        assert!(
            node.bbox.x >= owner.x - 1e-6
                && node.bbox.x + node.bbox.width <= owner.x + owner.width + 1e-6,
            "선택 항목 {}가 셀 밖으로 이탈: form={:?}, cell={owner:?}",
            form.caption,
            node.bbox
        );
        captions.push(form.caption.clone());
    }
    for child in &node.children {
        check_forms(child, cell, captions);
    }
}

fn assert_form_ownership(core: &DocumentCore) {
    assert_eq!(core.page_count(), 1, "독립 Print와 같은 전체 쪽수");
    let tree = core
        .build_page_render_tree(0)
        .expect("선택 항목의 실제 출력 트리");
    let mut captions = Vec::new();
    check_forms(&tree.root, None, &mut captions);
    assert_eq!(
        captions.len(),
        13,
        "수신 여부·광고접촉수단·직업 항목을 모두 보존"
    );
    assert!(captions
        .windows(3)
        .any(|row| row == ["신문", "인터넷홈쇼핑", "홈쇼핑"]));
    assert!(captions
        .windows(4)
        .any(|row| row == ["전단", "TV", "우편물", "기타"]));
    assert!(captions
        .windows(4)
        .any(|row| row == ["주부", "회사원", "자영업", "기타"]));
    assert!(
        scan_page(0, &tree.root, core.page_count(), &AnomalyOptions::default())
            .off_canvas
            .is_empty(),
        "모든 선택 항목을 복구한 원본에는 용지 밖으로 나간 내용이 없다"
    );
}

#[test]
fn distributed_form_rows_keep_the_last_option_inside_its_cell() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/basic/request.hwp"))
        .expect("독립 Print와 같은 원본");
    assert_form_ownership(&core);
}

#[test]
fn left_aligned_control_keeps_all_form_options_and_their_owners() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/basic/request.hwp"))
        .expect("같은 원본의 정렬 대조군");
    let mut doc = core.document().clone();
    for style in &mut doc.doc_info.para_shapes {
        if style.alignment == Alignment::Distribute {
            style.alignment = Alignment::Left;
        }
    }
    core.set_document(doc);
    assert_form_ownership(&core);
}
