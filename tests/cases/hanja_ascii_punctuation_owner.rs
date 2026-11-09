//! 관계형: 한자 뒤 ASCII 괄호도 원본 영문 슬롯을 소유하며 내용 순서와 전진을 보존한다.
//! R33 근거 갱신 2026-11-07T13:58:21+09:00; 직전 파일 변경 f00d9a4895ada1212bfd23c0ea4e941024c847d0 (2026-11-07T11:03:07+09:00).
//! 제품 ef77d740e1, 전94쪽 96 DPI·Print raw 최저 Native 99.98% / fresh WASM 99.99%.
//! 검사5쪽은 각각100.00%/99.99%, 독립 PDF와94쪽 일치·직접 확대 판독 완료.
//! 최신 증거: mydocs/pr/assets/planet6897_20261106/pr8611/r33-source-symbol/.
//! 주석 작성: 2026-11-07 09:43:33 +0900. 새 파일이므로 직전 변경 커밋은 없다.
//! 원본 samples/basic/sungeo.hwp, 독립 pdf/basic/sungeo-hwp-2020.pdf 전94쪽.
//! 2026-11-07 R11, 96DPI·Print·보정 전2px raw 전쪽 최저 Native99.24%/fresh WASM99.32%.
//! 양쪽 모두94쪽 일치. 직접 판독·TSV·입력 해시: mydocs/pr/assets/planet6897_20261106/
//! pr8611/r11/sungeo/provenance.json 및 native.tsv·wasm.tsv. 실제 원본의5쪽을 검사한다.
//! 독립 Print 전94쪽의 한양해서 한자 뒤 여는 괄호947건은 영문 슬롯의 Gungsuh다.
//! 같은 크기의 궁서/굴림/바탕 등3개 슬롯 대조군도 한컴2020 11.0.0.9136 Print로 확인했다.
//! 글꼴 설치·절대 px·픽셀 이미지와 무관하게 원본 주소·슬롯 소유·내용·앞뒤 관계를 검사한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode},
    DocumentCore,
};

fn collect<'a>(node: &'a RenderNode, runs: &mut Vec<(&'a RenderNode, &'a TextRunNode)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.section_index == Some(1) && run.para_index == Some(63) && run.char_start.is_some() {
            runs.push((node, run));
        }
    }
    for child in &node.children {
        collect(child, runs);
    }
}

#[test]
fn original_parenthesis_uses_the_latin_owner_instead_of_the_preceding_hanja_program() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/basic/sungeo.hwp")).expect("정식 고사성어 원본"),
    )
    .unwrap();
    let source = &core.document().sections[1].paragraphs[63];
    assert!(source.text.contains("江湖煙波(강호연파)"));
    let tree = core.build_page_render_tree(4).expect("독립 Print 대응5쪽");
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    assert_eq!(
        runs.iter()
            .map(|(_, run)| run.text.as_str())
            .collect::<String>(),
        source.text,
        "언어 슬롯 분할이 원문의 글자·순서·공백을 보존해야 한다"
    );
    let (open_node, open) = runs
        .iter()
        .find(|(_, run)| run.text.contains('('))
        .expect("원문의 여는 괄호");
    let (body_node, body) = runs
        .iter()
        .find(|(_, run)| run.text.contains("강호"))
        .expect("괄호 뒤 한글");
    let (_, hanja) = runs
        .iter()
        .find(|(_, run)| run.text.contains('江'))
        .expect("앞 한자");
    assert_eq!(open.text, "(", "원본의 괄호는 한자 문자열에서 분리된다");
    assert_eq!(
        open.style.font_family, body.style.font_family,
        "이 원본의 영문·한글은 같은 궁서 프로그램을 소유한다"
    );
    assert_ne!(
        open.style.font_family, hanja.style.font_family,
        "앞 한자의 한양해서를 괄호에 상속하지 않는다"
    );
    assert!(
        body_node.bbox.x >= open_node.bbox.x + open_node.bbox.width,
        "괄호의 실제 전진 뒤에 다음 한글을 놓는다"
    );
}
