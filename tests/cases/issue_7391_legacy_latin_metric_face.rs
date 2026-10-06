//! #7391: legacy-latin 폴백이 **선언 face 자신의 메트릭 표**를 버리면 안 된다.
//!
//! `AmeriGarmnd BT`(라틴 세리프)는 표시할 글꼴이 없는 환경을 위해 `HY견명조`(한글 명조)로
//! 치환된다. 표시로는 뜻이 있지만 그 이름이 **폭 결정에도** 쓰이면서, rhwp 가 이미 가진
//! `AmeriGarmnd BT` 자신의 폭 표가 버려지고 한글 명조의 라틴 폭(`HYMyeongJo-Extra`)으로
//! 영문이 전진했다.
//!
//! 기대값은 한컴 정본 `pdf/1341000_research_report_footnotes-2020.pdf` 에서 온다.
//! 그 정본은 이 글꼴을 **대체하지 않고 그대로 임베드**해 그렸다
//! (`INPILL+AmeriGarmnd-BT` / `-BTBold` / `-BTItalic`). 정본이 그 글꼴로 그린 ASCII
//! 13,919자의 전진폭을 후보 표와 대조하면 값이 갈린다.
//!
//! ```text
//!   AmeriGarmnd BT 자기 표                      중앙 오차 0.0398 em
//!   HY견명조 → HYMyeongJo-Extra (종전 치환 대상)    중앙 오차 0.2134 em   (5.4배)
//!   HYGothic-Medium                            중앙 오차 0.1324 em
//! ```
//!
//! 72쪽 참고문헌 줄에서 최종 좌표로 드러난다. `mutool draw -F stext` 로 정본과 rhwp 의
//! **같은 앞 60글자**를 재면:
//!
//! ```text
//!   정본      23.342 em
//!   수정 전   35.110 em   (오차 11.769)
//!   수정 후   23.540 em   (오차  0.198)
//! ```
//!
//! 그 결과 수정 전에는 줄이 **용지 오른쪽 밖으로 106.5px** 나갔다
//! (`rhwp layout-anomaly` 의 off-canvas 2건 → 0건).
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue2559/1341000_research_report_footnotes.hwp";
/// 정본이 이 줄을 그린 잉크 폭(em). 같은 줄의 글꼴 크기 12.428px 로 나눈 값이다.
const ORACLE_LINE_EM: f64 = 40.757;

fn collect(node: &RenderNode, out: &mut Vec<(String, f64, f64, f64, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((
            run.text.clone(),
            node.bbox.x,
            node.bbox.width,
            run.style.font_size,
            node.bbox.y,
        ));
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn page72() -> (Vec<(String, f64, f64, f64, f64)>, f64) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("공개 회귀 문서")).expect("문서 파싱");
    let tree = core.build_page_render_tree(71).expect("72쪽 렌더 트리");
    let page_width = tree.root.bbox.width;
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    (runs, page_width)
}

/// 영문 참고문헌 줄이 용지 밖으로 나가지 않는다.
///
/// 용지 오른쪽 끝은 구현과 무관한 절대 기준이다. 종전에는 `Mulford, B. (2003).` 줄이
/// 이 경계를 106.5px 넘었다.
#[test]
fn legacy_latin_reference_lines_stay_inside_the_paper() {
    let (runs, page_width) = page72();
    assert!(page_width > 0.0, "72쪽 용지 폭");
    let mut worst: Option<(&str, f64)> = None;
    for (text, x, width, ..) in &runs {
        let over = x + width - page_width;
        if over > 0.5 && worst.map(|(_, w)| over > w).unwrap_or(true) {
            worst = Some((text.as_str(), over));
        }
    }
    assert!(
        worst.is_none(),
        "72쪽 run 이 용지({page_width:.1}px) 밖으로 {:.1}px 나갔다: {:?}. \
         legacy-latin 치환 대상(한글 명조)의 라틴 폭으로 영문을 전진시키면 이렇게 된다.",
        worst.unwrap().1,
        worst.unwrap().0.chars().take(40).collect::<String>(),
    );
}

/// 그 줄의 폭이 한컴 정본과 맞는다.
///
/// 종전에는 같은 앞 60글자가 35.110 em 으로 정본 23.342 em 보다 11.8 em 넓었고,
/// 줄 전체로는 이 구간의 1.5배까지 벌어졌다.
#[test]
fn legacy_latin_reference_line_width_matches_hancom() {
    let (runs, _) = page72();
    // 줄이 여러 run(정자체·이탤릭)으로 쪼개지므로, 같은 baseline 의 run 을 모아
    // 줄 전체 범위를 잰다.
    let head = runs
        .iter()
        .find(|(text, ..)| text.replace(' ', "").starts_with("Mulford,B.(2003)"))
        .expect("72쪽의 `Mulford, B. (2003).` run");
    let baseline = head.4;
    let font_size = head.3;
    assert!(font_size > 0.0, "run 글꼴 크기");
    let line: Vec<&(String, f64, f64, f64, f64)> = runs
        .iter()
        .filter(|(_, _, _, _, y)| (y - baseline).abs() < 0.5)
        .collect();
    let left = line.iter().fold(f64::MAX, |a, r| a.min(r.1));
    let right = line.iter().fold(f64::MIN, |a, r| a.max(r.1 + r.2));
    let em = (right - left) / font_size;
    assert!(
        (ORACLE_LINE_EM - 3.0..ORACLE_LINE_EM + 3.0).contains(&em),
        "`Mulford, B. (2003).` 줄 폭 {em:.3} em 이 정본 {ORACLE_LINE_EM:.3} em 기준 \
         구간 [{:.3}, {:.3}] 밖이다. 선언 face 의 폭 표를 버리고 한글 명조 표로 재면 \
         50% 넘게 넓어진다.",
        ORACLE_LINE_EM - 3.0,
        ORACLE_LINE_EM + 3.0,
    );
}
