//! 관계형·고정형: 바탕쪽 프레임 이동·내용 보존과 독립 Print의 3쪽 소속 계약.
//! 주석 갱신: 2026-11-14T23:50:16+09:00; 직전 파일 변경: eeaecff59e834f28437c98b509089bbb36a7682d 2026-11-14T23:14:44+09:00.
//! 측정: 2026-11-14, 96dpi, compat 2022, 색상 경계 보정 전 2px 내용 실루엣 raw.
//! Both HWPX/HWP, Even HWPX, symmetric/reverse HWPX: 각 3쪽, Native/fresh WASM 전쪽 최저 각각 100.00%.
//! rectangle/table 한컴 재저장 HWPX: 각 3쪽, Native/fresh WASM 전쪽 최저 각각 100.00%.
//! 서식2-간단 원 HWP/실제 HWPX 저장본: 각 3쪽, Native/fresh WASM 전쪽 최저 각각 100.00%.
//! #8307의 실제 2018 Print: 원 HWP/정상 HWPX 각 3쪽, Native/fresh WASM 모두 최저 100.00%.
//! 전체 측정 범위 양쪽 각각 33쪽·최저 100.00%; 독립 PDF 확대 판독 완료. 절대 px 핀은 두지 않는다.
//! 근거: mydocs/pr/assets/pr9071/README.md, native/·wasm/의 전쪽 TSV와 frame-contact.png·color-contact.png.
//! before: Both 짝수 쪽 47.76%, 서식2-간단 3쪽 Native 16.74%·WASM 16.64%; 최종 제품 source eeaecff59e에서 재검증.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn load(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn text(node: &RenderNode, result: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        result.push_str(&run.text);
    }
    for child in &node.children {
        text(child, result);
    }
}

fn compact_text(node: &RenderNode) -> String {
    let mut value = String::new();
    text(node, &mut value);
    value.chars().filter(|c| !c.is_whitespace()).collect()
}

fn object(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(
        node.node_type,
        RenderNodeType::Image(_) | RenderNodeType::Rectangle(_) | RenderNodeType::Table(_)
    ) {
        return Some(node);
    }
    node.children.iter().find_map(object)
}

fn frame(core: &DocumentCore, page: u32) -> (BoundingBox, BoundingBox) {
    let tree = core.build_page_render_tree(page).unwrap();
    let master = tree
        .root
        .children
        .iter()
        // 개체의 paint layer에 따라 바탕쪽 부모가 여러 개일 수 있다.
        // 텍스트 전용 부모의 순서 대신 실제 검증 개체를 따라간다.
        .find(|node| matches!(node.node_type, RenderNodeType::MasterPage) && object(node).is_some())
        .unwrap_or_else(|| panic!("바탕쪽 개체 없음: page={}", page + 1));
    let body = tree
        .root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("본문 프레임");
    let content = compact_text(body);
    assert_eq!(
        content.matches(&format!("page{}", page + 1)).count(),
        1,
        "{content}"
    );
    (object(master).expect("바탕쪽 개체").bbox, body.bbox)
}

fn assert_shared_frame(core: &DocumentCore) {
    assert_eq!(
        core.page_count(),
        3,
        "독립 Print 전쪽·내용 소속을 함께 보호한다"
    );
    let (odd, odd_body) = frame(core, 0);
    let (even, even_body) = frame(core, 1);
    let (next_odd, next_body) = frame(core, 2);
    let body_delta = even_body.x - odd_body.x;
    assert!(
        ((even.x - odd.x) - body_delta).abs() < 1.0,
        "같은 양쪽 바탕쪽의 용지 개체와 본문 기준 프레임이 같은 이동을 소비한다"
    );
    assert!(
        (even.width - odd.width).abs() < 1.0,
        "절단을 너비 축소로 가리지 않는다"
    );
    assert!((even.height - odd.height).abs() < 1.0);
    assert!(
        (even.y - odd.y).abs() < 1.0,
        "가로 맞쪽 이동이 세로 원점을 바꾸지 않는다"
    );
    assert!((next_odd.x - odd.x).abs() < 1.0);
    assert!((next_body.x - odd_body.x).abs() < 1.0);
}

#[test]
fn both_master_picture_rectangle_and_table_share_the_facing_frame() {
    for sample in [
        "samples/pr9071/issue9046/hancom/02-both.hwpx",
        "samples/pr9071/master-frame-controls/symmetric.hwpx",
        "samples/pr9071/master-frame-controls/reverse.hwpx",
        "samples/pr9071/master-frame-controls/rectangle-normalized.hwpx",
        "samples/pr9071/master-frame-controls/table-normalized.hwpx",
    ] {
        eprintln!("master frame input: {sample}");
        assert_shared_frame(&load(sample));
    }
    let mut original = load("samples/pr9071/issue9046/hancom/02-both.hwpx");
    eprintln!("master frame input: actual HWP roundtrip");
    // 독립 Print와 비교한 실제 CLI의 HWPX → HWP 저장 경로를 사용한다.
    // native serializer에 원 HWPX IR을 직접 넘기면 형식 lowering을 생략한다.
    let saved = original.export_hwp_with_adapter().unwrap();
    assert_shared_frame(&DocumentCore::from_bytes(&saved).unwrap());
}

#[test]
fn even_only_master_uses_its_own_frame_and_keeps_other_pages_clear() {
    let both = load("samples/pr9071/issue9046/hancom/02-both.hwpx");
    let even = load("samples/pr9071/issue9046/hancom/03-even.hwpx");
    assert_eq!(even.page_count(), 3);
    let (both_odd_object, _) = frame(&both, 0);
    let (even_object, _) = frame(&even, 1);
    assert!(
        (even_object.x - both_odd_object.x).abs() < 1.0,
        "짝수 전용은 자기 쪽에서 정의한 동일 용지 위치를 유지한다"
    );
    for page in [0, 2] {
        let tree = even.build_page_render_tree(page).unwrap();
        assert!(!tree
            .root
            .children
            .iter()
            .any(|node| matches!(node.node_type, RenderNodeType::MasterPage)));
        assert!(compact_text(&tree.root).contains(&format!("page{}", page + 1)));
    }
}

fn assert_successors_on_third_page(core: &DocumentCore) {
    assert_eq!(core.page_count(), 3);
    let pages: Vec<_> = (0..core.page_count())
        .map(|page| compact_text(&core.build_page_render_tree(page).unwrap().root))
        .collect();
    let first = "[서식2]이줄은3페이지에위치해야함";
    for page in &pages[..2] {
        assert!(
            !page.contains(first),
            "표 뒤 본문이 표와 같은 쪽에 끼어들면 안 된다"
        );
        assert!(!page.contains("다음줄"));
    }
    assert_eq!(pages[0].matches("서식1").count(), 1);
    assert_eq!(pages[1].matches("서식2").count(), 1);
    assert_eq!(pages[2].matches(first).count(), 1);
    assert_eq!(pages[2].matches("다음줄").count(), 1);
    assert!(pages[2].find(first).unwrap() < pages[2].find("다음줄").unwrap());
}

#[test]
fn owned_tac_gap_keeps_successor_content_after_the_page_boundary_on_save() {
    let mut original = load("samples/pr9071/issue8318/서식2-간단.hwp");
    let original_text: Vec<_> = original.document().sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.clone())
        .collect();
    assert_successors_on_third_page(&original);
    // 같은 원 신고의 순수 HWPX는 기존의 올바른 새 쪽 배치를 유지한다.
    assert_successors_on_third_page(&load("samples/pr9071/issue8307/서식2-간단.hwpx"));
    let saved = original.export_hwpx_native().unwrap();
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    assert_successors_on_third_page(&reopened);
    let reopened_text: Vec<_> = reopened.document().sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.clone())
        .collect();
    assert_eq!(
        original_text, reopened_text,
        "의도적인 빈 문단을 포함해 문단 순서를 보존한다"
    );
}
