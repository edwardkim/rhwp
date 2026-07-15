//! 메모 렌더링(--show-memos) 통합 테스트.
//!
//! - 기본(플래그 off): 메모가 렌더 트리/SVG에 나타나지 않는다 (기존 출력 불변 —
//!   한컴 PDF 권위 자료에 메모가 출력되지 않으므로 기본 출력은 메모 없음).
//! - 플래그 on: 메모 앵커가 있는 페이지에 MemoArea 노드가 생기고, SVG 캔버스가
//!   우측으로 확장되며, 메모 본문 텍스트가 출력에 포함된다.

use rhwp::wasm_api::HwpDocument;

/// SVG 내 모든 `<text>` 내용을 이어붙인다 (SVG 는 글자 단위 `<text>` 를 방출하므로
/// 다글자 부분 문자열 검색에는 이 결합 문자열을 쓴다).
fn svg_text(svg: &str) -> String {
    let mut out = String::new();
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        if let Some(gt) = rest[open..].find('>') {
            let after = &rest[open + gt + 1..];
            if let Some(close) = after.find("</text>") {
                out.push_str(&after[..close]);
                rest = &after[close + 7..];
                continue;
            }
        }
        break;
    }
    out
}

/// aift.hwpx: 메모 2건 (같은 페이지, memoPr width=15591 fillColor=#CBFF99).
/// 플래그를 잠시 켜서 MemoArea 노드가 생기는 페이지(메모 앵커 페이지)를 찾는다.
fn find_memo_page(doc: &mut HwpDocument) -> Option<u32> {
    let was_on = doc.get_show_memos();
    doc.set_show_memos(true);
    let page = (0..doc.page_count()).find(|&p| {
        doc.get_page_render_tree(p)
            .map(|json| json.contains("\"MemoArea\""))
            .unwrap_or(false)
    });
    doc.set_show_memos(was_on);
    page
}

#[test]
fn memo_hidden_by_default() {
    let bytes = std::fs::read("samples/hwpx/aift.hwpx").expect("샘플 읽기");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("파싱");
    let page = find_memo_page(&mut doc).expect("메모 앵커 페이지");

    let svg = doc.render_page_svg_native(page).expect("렌더");
    // 기본 출력에는 메모 본문/박스가 없어야 한다 (한컴 PDF 정합).
    // 앵커 문단 텍스트("공동기관1 : 두아즈")와 메모 본문은 다른 문자열이다.
    assert!(
        !memo_body_present(&svg),
        "플래그 off 인데 메모 본문이 출력됨"
    );

    let tree_json = doc.get_page_render_tree(page).expect("트리");
    assert!(
        !tree_json.contains("MemoArea"),
        "플래그 off 인데 MemoArea 노드가 생성됨"
    );
}

#[test]
fn memo_rendered_with_show_memos() {
    let bytes = std::fs::read("samples/hwpx/aift.hwpx").expect("샘플 읽기");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("파싱");
    let page = find_memo_page(&mut doc).expect("메모 앵커 페이지");

    // 플래그 off 기준 캔버스 폭
    let plain_svg = doc.render_page_svg_native(page).expect("렌더 (off)");
    let plain_width = svg_width(&plain_svg);

    doc.set_show_memos(true);
    assert!(doc.get_show_memos());

    let svg = doc.render_page_svg_native(page).expect("렌더 (on)");

    // 메모 본문 텍스트가 페이지 SVG에 포함된다.
    assert!(memo_body_present(&svg), "메모 본문 미출력");
    // 작성자 라벨 (aift 메모 command 의 author = "user")
    assert!(svg_text(&svg).contains("user"), "메모 작성자 라벨 미출력");
    // 캔버스가 우측으로 확장된다 (박스 폭 15591HU ≈ 208px + 거터).
    let width = svg_width(&svg);
    assert!(
        width > plain_width + 100.0,
        "캔버스 미확장: off={plain_width} on={width}"
    );

    // 렌더 트리에 MemoArea 노드 2개 (메모 2건).
    let tree_json = doc.get_page_render_tree(page).expect("트리");
    let memo_nodes = tree_json.matches("\"MemoArea\"").count();
    assert_eq!(memo_nodes, 2, "MemoArea 노드 수");

    // 플래그를 다시 끄면 원상 복구된다 (캐시 무효화 확인).
    doc.set_show_memos(false);
    let svg_off = doc.render_page_svg_native(page).expect("렌더 (재off)");
    assert!(
        !memo_body_present(&svg_off),
        "플래그 재off 후에도 메모가 남음"
    );
    assert!(
        (svg_width(&svg_off) - plain_width).abs() < 0.01,
        "재off 폭 복구"
    );
}

/// 메모가 없는 페이지는 플래그가 켜져 있어도 캔버스가 확장되지 않는다.
#[test]
fn memo_free_page_unchanged_with_flag() {
    let bytes = std::fs::read("samples/hwpx/aift.hwpx").expect("샘플 읽기");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("파싱");

    let plain = doc.render_page_svg_native(0).expect("렌더 (off)");
    doc.set_show_memos(true);
    let with_flag = doc.render_page_svg_native(0).expect("렌더 (on)");
    assert_eq!(
        plain, with_flag,
        "메모 없는 페이지 출력이 플래그에 영향받음"
    );
}

/// 메모 본문("기업 소개 및 본 과제 관련 기술력 소개") 포함 여부.
/// SVG 는 글자 단위 `<text>` 로 방출되고 공백은 요소로 나오지 않으므로
/// 공백을 제거한 결합 문자열로 비교한다.
fn memo_body_present(svg: &str) -> bool {
    svg_text(svg)
        .replace(' ', "")
        .contains("기업소개및본과제관련기술력소개")
}

fn svg_width(svg: &str) -> f64 {
    let start = svg.find("width=\"").expect("width 속성") + 7;
    let end = svg[start..].find('"').expect("width 닫힘") + start;
    svg[start..end].parse().expect("width 숫자")
}
