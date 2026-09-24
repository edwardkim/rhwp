//! #7387: `CharShape.useFontSpace` 가 켜진 run 의 공백은 반각이 아니라
//! **영문 슬롯 글꼴이 선언한 공백 전진폭**이다.
//!
//! 종전에는 `useFontSpace` 를 렌더러가 아예 읽지 않아 공백이 언제나 `em/2` 였다.
//! `samples/exam_eng.hwp` 는 영문 슬롯이 `Times New Roman`(대체 없음, 표 공백
//! 512/2048 = 0.25 em)이고 본문 charPr 가 `useFontSpace="1"` 이라, 반각 0.5 em 을
//! 쓰면 공백마다 0.25 em 씩 줄이 넓어진다.
//!
//! 기대값은 구현이 아니라 한컴 정본 `pdf/exam_eng-2022.pdf` 에서 온다.
//! **문단 마지막 줄**만 고른다 — 양쪽정렬 슬랙이 없어 자연폭이 그대로 보이는 줄이다.
//! `mutool draw -F stext` 로 같은 줄의 첫 글자 `x0` 부터 마지막 글자 `x1` 까지를 재고
//! 그 줄의 글꼴 크기(16.32px)로 나눈 값이 아래 `oracle_ink_em` 이다.
//!
//! ```text
//!   쪽  정본 잉크폭   수정 전   수정 후     (글자 원점 기준 폭, em)
//!    2    20.064     22.201    19.951   we promise to hold another race in the near future.
//!    3    15.701     17.316    15.566   the first visual signature of the new era.
//!    6    17.789     19.148    17.648   can be devalued when contracts are violated.
//! ```
//!
//! 검사는 run 의 **점유폭**(`bbox.w`)을 본다. 점유폭은 마지막 글자의 전진폭까지
//! 포함하므로 정본의 잉크폭보다 한 글자 몫만큼 넓다 — 그래서 등식이 아니라
//! `[정본 잉크폭 - 0.2, 정본 잉크폭 + 0.8] em` 구간으로 판정한다. 수정 전 값은
//! 이 구간보다 1.4~1.6 em 위에 있어 구간 밖이다.
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

/// 공백을 지운 글자열이 `needle` 과 같은 TextRun 의 (점유폭 / 글꼴크기)를 모은다.
fn run_width_em(node: &RenderNode, needle: &str, out: &mut Vec<f64>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let key: String = run.text.chars().filter(|c| !c.is_whitespace()).collect();
        if key == needle && run.style.font_size > 0.0 {
            out.push(node.bbox.width / run.style.font_size);
        }
    }
    for child in &node.children {
        run_width_em(child, needle, out);
    }
}

fn core(name: &str) -> DocumentCore {
    let sample = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("samples/{name}"));
    DocumentCore::from_bytes(&std::fs::read(sample).expect("공개 회귀 문서")).expect("문서 파싱")
}

#[test]
fn use_font_space_run_matches_hancom_natural_width() {
    let doc = core("exam_eng.hwp");
    for (page, needle, oracle_ink_em) in [
        (
            2u32,
            "wepromisetoholdanotherraceinthenearfuture.",
            20.064_f64,
        ),
        (3, "thefirstvisualsignatureofthenewera.", 15.701),
        (6, "canbedevaluedwhencontractsareviolated.", 17.789),
    ] {
        let tree = doc
            .build_page_render_tree(page - 1)
            .expect("대조 쪽 렌더 트리");
        let mut widths = Vec::new();
        run_width_em(&tree.root, needle, &mut widths);
        assert_eq!(
            widths.len(),
            1,
            "{page}쪽에서 `{needle}` run 을 유일하게 찾는다 (찾은 수 {})",
            widths.len()
        );
        let actual = widths[0];
        assert!(
            actual >= oracle_ink_em - 0.2 && actual <= oracle_ink_em + 0.8,
            "{page}쪽 `{needle}`: 점유폭 {actual:.3} em 이 정본 잉크폭 {oracle_ink_em:.3} em \
             기준 구간 [{:.3}, {:.3}] 밖이다. useFontSpace 를 무시하고 공백을 반각으로 \
             전진시키면 공백마다 0.25 em 씩 넓어진다.",
            oracle_ink_em - 0.2,
            oracle_ink_em + 0.8,
        );
    }
}

/// 반례 가드 — 규칙은 상수가 아니라 **영문 슬롯 글꼴의 값**이다.
///
/// `1382000_domestic_violence_survey.hwp` 의 charPr 26 은 `useFontSpace="1"` 인데
/// 영문 슬롯이 `휴먼명조`(대체 없이 그대로 쓰인다)이고 그 글꼴의 공백은 256/512 =
/// 0.5 em 이다. 정본 p19 에서 이 charPr 의 공백은 n=38 관측 최빈 **0.489 em** 으로,
/// 같은 쪽 다른 `useFontSpace="1"` charPr 들의 0.337 em 과 뚜렷이 갈린다.
/// 즉 `useFontSpace` 를 켰다고 공백이 일률적으로 좁아지면 안 된다.
///
/// 이 검사는 수정 전에도 통과한다 — 결함 검출 증거가 아니라, 뒤에 이 갈래를
/// 상수로 눌러 버리는 변경을 막는 경계다.
#[test]
fn latin_slot_with_half_em_space_keeps_half_width() {
    let doc = core("task2430/1382000_domestic_violence_survey.hwp");
    let tree = doc.build_page_render_tree(16).expect("17쪽 렌더 트리");
    let needle = "만일이연구에참여하지않는다면불이익이있습니까?";
    let mut widths = Vec::new();
    run_width_em(&tree.root, needle, &mut widths);
    assert_eq!(widths.len(), 1, "17쪽에서 7번 문항 run 을 유일하게 찾는다");
    // 공백 6개. 영문 슬롯 휴먼명조가 0.5 em 공백을 선언하므로 반각과 같은 폭이 나온다
    // (실측 25.461 em). 이 갈래까지 같은 쪽의 다른 `useFontSpace` run 처럼 0.25 em 으로
    // 눌러 버리면 6 x 0.25 = 1.5 em 이 빠져 23.96 em 이 된다 — 구간은 그 둘을 가른다.
    let actual = widths[0];
    assert!(
        (24.7..26.2).contains(&actual),
        "17쪽 7번 문항 점유폭 {actual:.3} em 이 24.7~26.2 em 밖이다. \
         영문 슬롯이 0.5 em 공백을 선언한 run 까지 좁히면 안 된다.",
    );
}
