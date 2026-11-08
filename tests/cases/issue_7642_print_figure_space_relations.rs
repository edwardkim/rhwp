//! 한컴 2024의 정상 저장·Print와 Native/fresh WASM 전체 TSV를 확인한 공백 계약.
//! 증적: mydocs/pr/assets/issue7642-figure-space-contract-20261012/README.md
//! 0pt 기본값의 과거 합성 입력을 정상 출력 계약으로 사용하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-figure-space-diagnostic/figure-space-2024.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("한컴 정상 저장 입력"))
        .expect("고정폭 공백 입력 읽기");
    core.set_hangul2024_compat(true);
    core
}

fn runs<'a>(node: &'a RenderNode, owner: (usize, usize), out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if (run.section_index, run.para_index) == (Some(owner.0), Some(owner.1)) {
            out.push(node);
        }
    }
    for child in &node.children {
        runs(child, owner, out);
    }
}

#[test]
fn print_saved_figure_space_preserves_its_owner_and_scales_against_normal_space() {
    let core = core();
    assert_eq!(core.page_count(), 1, "독립 Print 전체 쪽수");
    let tree = core.build_page_render_tree(0).expect("정상 크기의 출력");
    let mut groups: BTreeMap<i32, BTreeMap<String, (f64, f64)>> = BTreeMap::new();
    for (section, data) in core.document().sections.iter().enumerate() {
        for (paragraph, para) in data.paragraphs.iter().enumerate() {
            if !matches!(para.text.as_str(), "가가" | "가 가" | "가\u{2007}가") {
                continue;
            }
            let shape = &core.document().doc_info.char_shapes[para
                .char_shapes
                .first()
                .expect("독립 저장 글자 모양")
                .char_shape_id
                as usize];
            assert!(
                shape.base_size > 0,
                "유효한 저장 크기에서만 전진을 검증한다"
            );
            let mut found = Vec::new();
            runs(&tree.root, (section, paragraph), &mut found);
            assert_eq!(found.len(), 1, "소유 문단의 누락·중복 없는 단일 런");
            let RenderNodeType::TextRun(run) = &found[0].node_type else {
                unreachable!()
            };
            assert_eq!(run.text, para.text, "공백 종류와 앞뒤 글자의 내용 보존");
            assert!(run.style.font_size > 0.0 && found[0].bbox.width > 0.0);
            assert!(
                groups
                    .entry(shape.base_size)
                    .or_default()
                    .insert(
                        para.text.clone(),
                        (found[0].bbox.width, run.style.font_size)
                    )
                    .is_none(),
                "동일 원본의 중복 그룹을 허용하지 않는다"
            );
        }
    }
    assert_eq!(
        groups.keys().copied().collect::<Vec<_>>(),
        [1000, 1500, 2000, 3000]
    );
    for values in groups.values() {
        assert_eq!(values.len(), 3, "같은 글자 크기의 독립 세 문단");
        let (plain, fs) = values["가가"];
        let (normal, normal_fs) = values["가 가"];
        let (fixed, fixed_fs) = values["가\u{2007}가"];
        assert_eq!(fs, normal_fs);
        assert_eq!(fs, fixed_fs);
        let normal_advance = normal - plain;
        let fixed_advance = fixed - plain;
        assert!(normal_advance > fixed_advance && fixed_advance > 0.0);
        // 독립 Print 네 크기에서 각각 약 0.25em, 일반 공백의 절반이다.
        // 글꼴/장치 격자의 관측 범위를 비율로 검사하며 화면 px를 고정하지 않는다.
        assert!((0.24..=0.26).contains(&(fixed_advance / fs)));
        assert!((0.45..=0.55).contains(&(fixed_advance / normal_advance)));
    }
}
