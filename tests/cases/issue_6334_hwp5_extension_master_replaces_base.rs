//! [#6334] HWP5 확장 바탕쪽(마지막 쪽·임의 쪽)이 기본 홀/짝 바탕쪽을 **대체**하는지 고정한다.
//!
//! 종전에는 `parser/body_text.rs` 가 `replace_base: false` 로 고정해, 확장 바탕쪽이
//! `rendering.rs` 의 `replace_exts` 필터(`!overlap || replace_base`)에 절대 들어가지 못하고
//! 기본 바탕쪽 **위에 덧그려졌다.** 두 바탕쪽의 쪽번호·머리말이 같은 자리에 포개진다.
//!
//! # 판정 근거 — 저장소 안의 한컴 정답지
//!
//! `pdf/exam_science-2022.pdf` 4쪽의 바탕쪽 글자는 `32 32`·`* 확인 사항`·
//! `4 (화학 I) 과학탐구 영역` 이고 **기본 짝수 바탕쪽의 `31` 이 없다.** 종전 rhwp 는 두 겹을
//! 그려 `['31','32']` 와 `['32','32', …]` 가 18.0 x 15.3px 겹쳤다.
//!
//! 정답지는 sparse-checkout 대상이 아니라 작업 트리엔 안 보이지만 오브젝트엔 있다.
//!
//! ```bash
//! git show "HEAD:pdf/exam_science-2022.pdf" > oracle.pdf
//! ```
//!
//! # HWPX 경로와의 관계
//!
//! 같은 형상의 HWPX 경로는 #6323 / PR #6329 가 다뤘다. 거기서는 `pageDuplicate="0"` 이라는
//! 문서의 명시적 선언이 있었지만, HWP5 에는 그 속성이 없고 overlap 비트만 있다. 그 비트는
//! 의도를 구분하지 못한다 — 한컴의 HWPX -> HWP5 저장본은 `pageDuplicate="0"` 인 바탕쪽도
//! overlap 비트를 함께 세운다(`parser/hwpx/section.rs` 의 같은 지점 주석). 그래서 판정
//! 근거가 파일 안이 아니라 **한컴이 실제로 어떻게 그리는가**(정답지)다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn scan(sample: &str, page: u32) -> Option<Vec<Vec<String>>> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(path).ok()?;
    let doc = DocumentCore::from_bytes(&bytes).ok()?;
    let tree = doc.build_page_render_tree(page).ok()?;
    Some(
        tree.root
            .children
            .iter()
            .filter(|c| matches!(c.node_type, RenderNodeType::MasterPage))
            .map(|m| {
                let mut acc = Vec::new();
                collect_text(m, &mut acc);
                acc
            })
            .collect(),
    )
}

fn collect_text(node: &RenderNode, out: &mut Vec<String>) {
    if let RenderNodeType::TextRun(tr) = &node.node_type {
        let t = tr.display_or_text().trim();
        if !t.is_empty() {
            out.push(t.to_string());
        }
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

/// 확장 바탕쪽이 있는 쪽에는 바탕쪽이 하나만 그려진다.
#[test]
fn hwp5_extension_master_does_not_stack_on_base() {
    // (샘플, 쪽 0기준) — 둘 다 구역 마지막 쪽에 "확인 사항" 안내문 바탕쪽이 붙는다.
    for (sample, page) in [("samples/exam_science.hwp", 3), ("samples/exam-kor-3p.hwp", 2)] {
        let Some(masters) = scan(sample, page) else {
            continue;
        };
        assert_eq!(
            masters.len(),
            1,
            "{sample} {page}쪽: 바탕쪽이 겹쳐 그려지면 쪽번호·머리말이 같은 좌표에 포개진다. \
             그려진 바탕쪽 {}겹, 각 글자: {masters:?}",
            masters.len()
        );
    }
}

/// 그려진 바탕쪽이 한컴 정답지와 같은 것이다.
///
/// `pdf/exam_science-2022.pdf` 4쪽에는 `32`·`확인 사항` 이 있고 `31` 이 없다.
/// 대체가 아니라 덧그리기로 되돌아가면 기본 짝수 바탕쪽의 `31` 이 함께 나타난다.
#[test]
fn hwp5_extension_master_matches_hancom_oracle_text() {
    let Some(masters) = scan("samples/exam_science.hwp", 3) else {
        return;
    };
    let all: Vec<&String> = masters.iter().flatten().collect();
    assert!(
        all.iter().any(|t| t.contains("확인 사항")),
        "확장 바탕쪽의 안내문이 그려져야 한다: {all:?}"
    );
    assert!(
        !all.iter().any(|t| t.as_str() == "31"),
        "기본 짝수 바탕쪽의 쪽번호 '31' 은 정답지에 없다 — 덧그리기로 되돌아갔다: {all:?}"
    );
}
