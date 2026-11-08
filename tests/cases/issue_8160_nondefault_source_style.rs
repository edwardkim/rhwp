//! #8160: sample16 전용 회귀. 이전 등록: tests/cases/issue_4680_hwp3_paragraph_style_reference.rs
use rhwp::document_core::DocumentCore;
use std::{collections::BTreeSet, path::Path};

fn parse(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(path).expect("표본 읽기")).expect("표본 파싱")
}

/// 본문 **최상위** 문단의 style_id 서열.
///
/// 표 칸 안쪽은 세지 않는다 — 우리 HWP3 파스는 글자 없는 도형에 빈 글상자를 붙여
/// 칸 문단 수가 한/글 변환본과 다르다(같은 이슈의 별개 결함). 그 축이 이 대조를
/// 오염시키지 않도록 범위를 본문으로 맞춘다.
fn style_ids(rel: &str) -> Vec<u8> {
    let core = parse(rel);
    core.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| p.style_id)
        .collect()
}

#[test]
fn issue_8160_hwp3_paragraphs_reference_more_than_the_default_style() {
    // 결함을 직접 드러낸다 — 수정 전에는 어떤 문서든 `{0}` 하나뿐이었다.
    // `hwp3-sample16` 은 이름 붙은 스타일을 15종 쓰는 문서다.
    let used: BTreeSet<u8> = style_ids("samples/hwp3-sample16.hwp").into_iter().collect();
    assert!(
        used.len() > 1,
        "쓰이는 style_id 가 {used:?} 뿐이다 — 문단이 스타일을 참조하지 않는다"
    );
}
