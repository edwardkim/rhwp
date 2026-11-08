//! 관계형: 줄 끝 첫 구분 공백과 뒤에 작성한 공백의 소유·전진 비율을 구분한다.
//! 주석 갱신: 2026-11-07T03:42:00+09:00.
//! 실제 21쪽 Native 99.97728% / fresh WASM 99.97745%; 같은 쪽의 독립 좌표는 별도 고정형으로 보호한다.
//! 용지 테두리 보정 후 합성 대조군 raw(Native/WASM): wide·edge·tight 모두 100/100%(각 독립 Print 1쪽).
//! 용지 테두리의 고정형 화소 검사는 native_fractional_glyph_origin에 추가했다.
//! 문단 소속·첫 낱말·작성 공백 수에 따른 font_size 비율을 검사하며 후보 좌표를 기대값으로 복제하지 않는다.
//! 근거: mydocs/pr/assets/issue4138/interword-space-20261107/README.md.
//! 추가 근거: mydocs/pr/assets/issue4138/page-frame-coverage-20261107/README.md.

use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};
fn walk<'a>(n: &'a RenderNode, word: &str, out: &mut Vec<&'a RenderNode>) {
    if matches!(&n.node_type,RenderNodeType::TextRun(t)if t.text.contains(word)) {
        out.push(n)
    }
    for c in &n.children {
        walk(c, word, out)
    }
}
fn open(path: &str) -> DocumentCore {
    let r = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut c = DocumentCore::from_bytes(&std::fs::read(r.join(path)).unwrap()).unwrap();
    c.set_hangul2024_compat(true);
    c
}
#[test]
fn production_first_separator_stays_with_the_previous_word() {
    let c=open("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let t = c.build_page_render_tree(20).unwrap();
    let mut nodes = vec![];
    walk(&t.root, "다음에 따라 계산하여야 한다.", &mut nodes);
    assert_eq!(nodes.len(), 1);
    let RenderNodeType::TextRun(r) = &nodes[0].node_type else {
        unreachable!()
    };
    assert_eq!(r.para_index, Some(271));
    assert!(
        r.text.starts_with("다음에"),
        "독립 Print·한컴 재저장 textpos61은 첫 낱말 앞 공백이 없는 줄: {:?}",
        r.text
    );
}
fn controls(path: &str, counts: &[usize]) {
    let c = open(path);
    let mut seen = 0;
    for p in 0..c.page_count() {
        let t = c.build_page_render_tree(p).unwrap();
        let mut ns = vec![];
        walk(&t.root, "따라간다", &mut ns);
        for n in ns {
            let RenderNodeType::TextRun(r) = &n.node_type else {
                unreachable!()
            };
            let i = r.text.chars().position(|x| x == '따').unwrap();
            let xs = r.layout_positions.clone().unwrap_or_else(|| {
                EmbeddedTextMeasurer.compute_char_positions(r.display_or_text(), &r.style)
            });
            let pi = r.para_index.unwrap() as usize;
            let expected = counts[pi - 1] as f64 * r.style.font_size * 0.5;
            assert!(
                (xs[i] - expected).abs() < 1e-6,
                "독립 Print의 이월 공백 소유 pi={pi}, actual={}, expected={expected}",
                xs[i]
            );
            seen += 1;
        }
    }
    assert_eq!(seen, counts.len());
}
#[test]
fn first_interword_separator_is_not_a_literal_carried_blank() {
    controls("pdf/issue4138/interword-space-edge.hwpx", &[0; 16]);
}
#[test]
fn additional_authored_spaces_keep_their_continuation_advance() {
    controls(
        "pdf/issue4138/interword-space-wide.hwpx",
        &[0, 0, 1, 2, 4, 6, 0, 0, 1, 2, 4, 6],
    );
}
#[test]
fn tight_frames_preserve_only_the_additional_carried_spaces() {
    controls(
        "pdf/issue4138/interword-space-tight.hwpx",
        &[1, 4, 0, 1, 1, 4, 0, 1],
    );
}
