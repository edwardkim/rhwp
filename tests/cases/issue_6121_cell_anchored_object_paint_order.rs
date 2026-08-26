//! [Issue #6121] 칸 안 자리차지 container 위에 같은 칸의 흰색 잔재 텍스트가
//! 나중에 그려져 "경 찰 청" 글자가 파먹힌다 (경찰청 보도자료 156531618).
//!
//! 근인: 셀 콘텐츠 조립이 문단 순서대로 anchored 개체를 즉시 밀어 넣어, 뒤
//! 문단의 본문 텍스트(서식 잔재 "문화체육관광부", 10pt 흰색)가 앞 문단의
//! container(경찰 마크 + rect drawText "경 찰 청", zOrder 9)를 덮었다.
//! 한글은 칸 문단에 앵커된 자리차지/어울림 개체(글 뒤로 제외)를 칸 본문
//! 텍스트 **위**에 그린다 — 본문 흐름에서 개체가 문단 텍스트 뒤에 일괄
//! 페인트되는 것과 같은 계약이다.
//!
//! 수정: `layout_cell_shape` 가 비-TAC 개체에 layer(text_wrap·z_order)를
//! 마킹하고, 페이지 조립 후처리(`lift_cell_anchored_objects_above_text`)가
//! TableCell children 에서 그 개체들만 z_order 안정 정렬로 끝으로 옮긴다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6121/156531618_police_press_header.hwpx";

/// 머리 칸에서 container 의 회색 "경 찰 청"(#595959)이 흰색 잔재
/// "문화체육관광부"(#ffffff)보다 **나중에**(위에) 방출되어야 한다.
#[test]
fn issue_6121_cell_container_paints_above_leftover_text() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    let white_last = last_offset_of_fill_text(&svg, "#ffffff", '부')
        .expect("흰색 잔재 run(문화체육관광부)이 방출되어야 한다");
    let grey_first = first_offset_of_fill_text(&svg, "#595959", '경')
        .expect("container drawText(경 찰 청)가 방출되어야 한다");
    assert!(
        grey_first > white_last,
        "container 텍스트가 잔재 텍스트보다 먼저 방출됐다 — 흰 획이 위에 그려져 \
         글자를 파먹는다 (grey@{grey_first} vs white-last@{white_last})"
    );
}

fn first_offset_of_fill_text(svg: &str, fill: &str, ch: char) -> Option<usize> {
    offsets_of_fill_text(svg, fill, ch).into_iter().min()
}

fn last_offset_of_fill_text(svg: &str, fill: &str, ch: char) -> Option<usize> {
    offsets_of_fill_text(svg, fill, ch).into_iter().max()
}

/// `fill` 색이면서 본문이 `ch` 로 시작하는 `<text>` 요소들의 파일 내 오프셋.
fn offsets_of_fill_text(svg: &str, fill: &str, ch: char) -> Vec<usize> {
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(rel) = svg[pos..].find("<text ") {
        let start = pos + rel;
        let Some(head_end) = svg[start..].find('>') else {
            break;
        };
        let head = &svg[start..start + head_end];
        let body = &svg[start + head_end + 1..];
        if head.contains(&format!("fill=\"{fill}\"")) && body.starts_with(ch) {
            out.push(start);
        }
        pos = start + head_end + 1;
    }
    out
}
