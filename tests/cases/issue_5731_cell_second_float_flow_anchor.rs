//! #5731: 둘째 자리차지 그림은 앞 캡션 뒤에 있고 셀 프레임 안에 있어야 한다.
//!
//! 원본 그림을 복원한 동일 7쪽 픽스처를 한컴2020 PDF와 전쪽 비교한다.
//! 첫 그림의 저장 자르기 범위는 높이가 비어 한컴이 그리지 않는다.
//! 이전 검사의 "한컴도 그림7개" 주장과 절대 픽셀 위치 고정은 독립 출력과 달랐다.
//! 빈 그림도 저장 글줄의 공간은 점유하므로 그 프레임·캡션 뒤에 둘째 그림이 온다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;
use std::path::Path;

const SAMPLE: &str = "samples/issue5731/cell_second_float_flow_anchor.hwpx";

fn collect<'a>(node: &'a Value, kind: &str, output: &mut Vec<&'a Value>) {
    if node["type"] == kind {
        output.push(node);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect(child, kind, output);
        }
    }
}

#[test]
fn issue_5731_second_cell_float_uses_stored_flow_anchor() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("실물 문서 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("실물 문서 열기");
    let Control::Table(source) = &core.document().sections[0].paragraphs[34].controls[0] else {
        panic!("기술자료 개요 표");
    };
    let pictures: Vec<_> = source.cells[0]
        .paragraphs
        .iter()
        .flat_map(|para| &para.controls)
        .filter_map(|control| match control {
            Control::Picture(pic) => Some(pic),
            _ => None,
        })
        .collect();
    assert_eq!(pictures.len(), 2, "원문 그림 개체 두 개를 보존한다");
    assert!(pictures[0].crop.right > pictures[0].crop.left);
    assert!(
        pictures[0].crop.bottom <= pictures[0].crop.top,
        "첫 그림의 빈 자르기 선택"
    );
    assert!(
        !pictures[1].common.treat_as_char,
        "둘째 그림은 자리차지 앵커다"
    );

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("실물 문서 렌더링");
    assert_eq!(doc.page_count(), 7, "한컴 정본의 쪽수");
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(2).expect("3쪽 트리"))
        .expect("렌더 트리 JSON");
    let mut tables = Vec::new();
    collect(&tree, "Table", &mut tables);
    let owners: Vec<_> = tables
        .into_iter()
        .filter(|table| table["pi"] == 34)
        .collect();
    assert_eq!(owners.len(), 1, "기술자료 개요 표는 3쪽에 한 번만 배치된다");
    let mut cells = Vec::new();
    collect(owners[0], "Cell", &mut cells);
    assert_eq!(cells.len(), 1, "두 그림과 캡션은 같은 칸 소유다");
    let cell = cells[0];
    let mut images = Vec::new();
    collect(cell, "Image", &mut images);
    assert_eq!(
        images.len(),
        2,
        "그리지 않는 선택도 저장 그림의 자리와 소유는 보존한다"
    );
    let mut runs = Vec::new();
    collect(cell, "TextRun", &mut runs);
    let captions: Vec<_> = runs
        .iter()
        .copied()
        .filter(|run| !run["text"].as_str().expect("캡션 글자").trim().is_empty())
        .collect();
    let content: String = captions
        .iter()
        .map(|run| run["text"].as_str().unwrap())
        .collect();
    assert_eq!(content.trim(), "[ 피에이치에이와 A 협력사, B 업체, C 협력사와의 관계 ][ 이 사건 기술자료 유용행위 관련 행위사실 요약 ]");
    let second_caption = captions.last().expect("둘째 캡션");
    assert_eq!(
        second_caption["text"],
        "[ 이 사건 기술자료 유용행위 관련 행위사실 요약 ]"
    );
    let metric = |node: &Value, key: &str| node["bbox"][key].as_f64().expect("상자 좌표");
    let bottom = |node: &Value| metric(node, "y") + metric(node, "h");
    assert!(
        bottom(captions[0]) <= metric(images[0], "y"),
        "첫 캡션 뒤에 첫 그림 글줄"
    );
    assert!(
        bottom(images[0]) <= metric(second_caption, "y"),
        "첫 그림 글줄 뒤에 둘째 캡션"
    );
    assert!(
        bottom(second_caption) <= metric(images[1], "y"),
        "둘째 그림이 앞 캡션을 덮지 않는다"
    );
    for image in images {
        assert_eq!(image["pi"], 34, "그림의 본문 표 소유");
        assert!(
            metric(image, "y") >= metric(cell, "y") && bottom(image) <= bottom(cell),
            "그림이 소유 칸 안에 배치된다: {image}"
        );
        assert!(
            metric(image, "x") >= metric(cell, "x")
                && metric(image, "x") + metric(image, "w") <= metric(cell, "x") + metric(cell, "w"),
            "그림이 셀 가로 경계 안에 있다"
        );
    }
    let svg = doc.render_page_svg_native(2).expect("3쪽 SVG");
    assert_eq!(
        svg.matches("<image ").count(),
        1,
        "빈 자르기를 원본 전체 그림으로 되살리지 않는다: 한컴 3쪽 그림1개"
    );
}
