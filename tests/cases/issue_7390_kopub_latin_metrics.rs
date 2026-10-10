//! #7390: KoPub 라틴 줄의 내용·소속과 비례 전진폭을 보호한다.
//!
//! 같은 원본의 한컴 2024 Print PDF 전체 140쪽을 기존 96dpi 조건으로
//! Native/fresh WASM 비교했다(최저 91.93033%, 미달 0). 94·108쪽을 직접
//! 판독한 뒤 잉크폭과 run 점유폭을 섞던 절대 px 핀을 교정했다.
//! 라틴 비례 글꼴의 i < w 계약과 실제 줄 내부 포함, 단어·공백 보존을 검사한다.
//! 공백은 독립 한컴 출력의 연속 공백 실측 0.4767em 부근이며, 글꼴 hmtx의
//! 0.290em을 쓰는 출력과 구분한다. 한컴 실측 출처는 기존 #7390 검토 기록이다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue2006/1790387_prep_final_report.hwpx";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("PrEP 정식 원본")).expect("문서 로드")
}

fn text(node: &RenderNode) -> String {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    own + &node.children.iter().map(text).collect::<String>()
}

fn find_line<'a>(node: &'a RenderNode, prefix: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && text(node).starts_with(prefix) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_line(child, prefix))
}

fn cursor(
    core: &DocumentCore,
    run: &rhwp::renderer::render_tree::TextRunNode,
    offset: usize,
) -> serde_json::Value {
    serde_json::from_str(
        &core
            .get_cursor_rect_native(
                run.section_index.expect("구역 소유"),
                run.para_index.expect("문단 소유"),
                run.char_start.expect("문자 시작") + offset,
            )
            .expect("소유 문단의 커서"),
    )
    .expect("커서 JSON")
}

fn advance(
    core: &DocumentCore,
    run: &rhwp::renderer::render_tree::TextRunNode,
    offset: usize,
    page: u32,
) -> f64 {
    let before = cursor(core, run, offset);
    let after = cursor(core, run, offset + 1);
    assert_eq!(before["pageIndex"].as_u64(), Some(u64::from(page)));
    assert_eq!(after["pageIndex"].as_u64(), Some(u64::from(page)));
    assert_eq!(before["y"], after["y"], "같은 줄 안 문자 경계");
    after["x"].as_f64().unwrap() - before["x"].as_f64().unwrap()
}

#[test]
fn kopub_latin_line_preserves_content_and_proportional_advances() {
    let core = core();
    for (page, expected) in [
        (
            93u32,
            "MSM, men who have sex with men; TGW/TGM, transwomen/transgender",
        ),
        (
            107,
            "with men through the network scale-up method in Japan. PLoS One. 2012;7(1):e31184.",
        ),
    ] {
        let tree = core.build_page_render_tree(page).expect("Print 대응 쪽");
        let line = find_line(&tree.root, expected).expect("완전한 라틴 줄의 쪽 소속");
        assert_eq!(text(line), expected, "단어·공백·마지막 어절 보존");
        for child in &line.children {
            if matches!(child.node_type, RenderNodeType::TextRun(_)) {
                let epsilon = line.bbox.width.max(1.0) * 1e-9;
                assert!(child.bbox.x >= line.bbox.x - epsilon);
                assert!(
                    child.bbox.x + child.bbox.width <= line.bbox.x + line.bbox.width + epsilon,
                    "라틴 run이 자기 줄의 가용 영역을 벗어났다"
                );
            }
        }
    }
    let tree = core.build_page_render_tree(107).unwrap();
    let line = find_line(&tree.root, "with men through").unwrap();
    let run = line
        .children
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if run.text.starts_with("with men through") => Some(run),
            _ => None,
        })
        .expect("라틴 run");
    assert!(
        run.style.font_family.contains("KoPub"),
        "원본 라틴 글꼴 소유"
    );
    // 줄 시작 커서는 앞 줄 끝과 같은 논리 경계이므로 내부 network의 w를 사용한다.
    let w = run.text.find("network").expect("network 단어") + 3;
    assert!(
        advance(&core, run, 1, 107) < advance(&core, run, w, 107),
        "KoPub 비례 글꼴의 i는 w보다 좁다. ASCII 일률 반각 전진을 허용하지 않는다"
    );
}

#[test]
fn kopub_space_stays_half_width() {
    let core = core();
    let tree = core.build_page_render_tree(107).expect("Print 108쪽");
    let line = find_line(&tree.root, "with men through").expect("참고문헌 이어지는 줄");
    let run = line
        .children
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if run.text.starts_with("with men through") => Some(run),
            _ => None,
        })
        .expect("소유 run");
    assert!(
        run.style.font_family.contains("KoPub"),
        "원본 공백 글꼴 소유"
    );
    assert_eq!(run.text.chars().nth(4), Some(' '));
    let space_em = advance(&core, run, 4, 107) / (run.style.font_size * run.style.ratio);
    // 한컴의 연속 공백 실측(0.4767em)과 글자 모양의 장평 관계를 대조한다.
    // 글꼴 hmtx 0.290em을 쓰는 회귀는 이 구간에 들어오지 않는다.
    assert!(
        (space_em - 0.4767).abs() <= 0.03,
        "소유 글자모양 대비 공백 전진 {space_em:.4}em은 한컴의 반각 공백과 다르다"
    );
}
