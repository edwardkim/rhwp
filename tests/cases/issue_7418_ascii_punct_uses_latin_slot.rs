//! [#7418] 영문 슬롯이 옛 한컴 영문 글꼴이면 ASCII 구두점도 한/글이 **영문 슬롯** 글꼴로 재고
//! 그린다.
//!
//! # 무엇이 깨져 있었나
//!
//! rhwp 는 ASCII 구두점을 언어 중립으로 보고 앞 글자 언어를 물려받았다. 한글 뒤 `-` 는 한글
//! 슬롯의 휴먼명조로 재여 9.63px(722 HWPUNIT)가 됐다. 한/글 PDF 에서 같은 `-` 는 영문 슬롯의
//! Palatino Linotype(영문 슬롯 `HCI Poppy` 의 치환)으로 그려지고 전진폭은 ≈6.4px 다.
//!
//! `78494-virtual-convergence-industry-decree` 8쪽 `pi=73` 첫 줄 `  - 해당 사업자는 … 수반되나,
//! 사업` 이 그 3.2px 때문에 `업` 을 213 HWPUNIT 넘겨 `사`/`업` 으로 갈렸다. 문단이 정본 4줄 대신
//! 5줄이 되어 +29px, 그 뒤 빈 문단 `pi=86` 이 혼자 9쪽을 열어 75쪽(정본 74)이 됐다.
//!
//! # 근거
//!
//! 말뭉치 `pdf/` 1,359개에서 한글 뒤 ASCII 구두점의 글꼴을 쟀다. 한글 글꼴이 휴먼명조인 줄에서
//! `(` 2278:17 · `,` 1733:6 · `)` 819:8 · `.` 525:10 · `:` 252:15 · `-` 77:0 (영문 글꼴:한글 글꼴)
//! — 99% 가 영문 슬롯이다. 구두점 뒤 공백은 88% 가 한글 글꼴이라 앞 글자 언어를 그대로 잇는다.
//! 한글 글꼴이 바탕·맑은 고딕이면 대부분 한글 글꼴 그대로다(2404:329 · 622:70) — 그래서 규칙은
//! 영문 슬롯이 옛 한컴 영문 글꼴(`LegacyLatin` 치환 — 이 문서는 휴먼명조 + `HCI Poppy`)일 때만
//! 건다. 반례는 마지막 검사다.
//!
//! # 이 검사가 말하지 않는 것
//!
//! 저장 줄(`LINE_SEG`)을 쓰는 문단의 줄 나눔은 바뀌지 않는다. `samples/` 1,176개 중 쪽 경계가
//! 바뀐 문서는 이 문서 하나다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const DOC: &str = "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx";
/// 0-based. 한/글 출력 PDF 의 8쪽.
const PAGE: u32 = 7;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DOC);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{DOC} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn collect_runs(node: &RenderNode, out: &mut Vec<(f64, f64, String, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((
            node.bbox.y,
            node.bbox.x,
            run.text.clone(),
            run.style.font_family.clone(),
        ));
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 8쪽 줄들을 공백 없이 이어 붙인 문자열과, 그 줄을 이룬 run 목록.
fn page_lines(core: &DocumentCore) -> Vec<(String, Vec<(String, String)>)> {
    let page = core.build_page_render_tree(PAGE).expect("8쪽 render tree");
    let mut runs = Vec::new();
    collect_runs(&page.root, &mut runs);
    runs.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).unwrap());

    let mut lines: Vec<(f64, String, Vec<(String, String)>)> = Vec::new();
    for (y, _x, text, family) in runs {
        match lines.last_mut() {
            Some((prev_y, buf, parts)) if (*prev_y - y).abs() < 0.5 => {
                buf.push_str(&text);
                parts.push((text, family));
            }
            _ => lines.push((y, text.clone(), vec![(text, family)])),
        }
    }
    lines
        .into_iter()
        .map(|(_, text, parts)| (text.chars().filter(|c| !c.is_whitespace()).collect(), parts))
        .collect()
}

/// 첫 줄이 한/글처럼 `사업` 까지 담고, 문서가 정본과 같은 74쪽이다.
#[test]
fn a_hyphen_measured_in_the_latin_slot_keeps_hancoms_line_break() {
    let core = load();
    let lines = page_lines(&core);
    let joined: Vec<&String> = lines.iter().map(|(text, _)| text).collect();

    let first = lines
        .iter()
        .position(|(text, _)| text.starts_with("-해당사업자는"))
        .unwrap_or_else(|| {
            panic!("정답지 전제가 깨졌다 — 8쪽에서 대상 문단을 찾지 못했다. 줄={joined:?}")
        });

    assert_eq!(
        lines[first].0, "-해당사업자는신고업무처리를위한행정적부담이수반되나,사업",
        "한/글 출력 8쪽의 이 줄은 `사업` 으로 끝난다 — `-` 를 한글 슬롯(휴먼명조 반각 9.63px)으로 \
         재면 `업` 이 213 HWPUNIT 넘쳐 `사`/`업` 으로 갈린다. 줄={joined:?}"
    );
    assert!(
        lines
            .get(first + 1)
            .is_some_and(|(text, _)| text.starts_with("일반현황,")),
        "다음 줄은 `일반현황,` 으로 시작해야 한다. 줄={joined:?}"
    );

    assert_eq!(
        core.page_count(),
        74,
        "정본은 74쪽이다 — 이 문단이 5줄이면 빈 문단 pi=86 이 혼자 9쪽을 열어 75쪽이 된다"
    );
}

/// 한글 run 안의 `-` 는 영문 슬롯 메트릭으로 전진한다 — run 은 쪼개지 않는다.
///
/// 공개 text-layout 의 `charX` 로 잰다. 한글 슬롯(휴먼명조 반각)으로 재면 9.63px, 영문 슬롯
/// (Palatino Linotype 0.336em × 20px − 자간 3%)이면 ≈6.1px 다. 정본 PDF 의 이 `-` 는 6.4px 전진.
#[test]
fn the_hyphen_in_a_hangul_run_advances_with_latin_slot_metrics() {
    let core = load();
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(PAGE)
            .expect("공개 text-layout"),
    )
    .expect("text-layout JSON");
    let run = layout["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .find(|run| {
            run["text"]
                .as_str()
                .is_some_and(|text| text.contains("- 해당 사업자는"))
        })
        .expect("`- 해당 사업자는` 이 든 run — 구두점은 한글 run 안에 남는다");
    let text: Vec<char> = run["text"].as_str().unwrap().chars().collect();
    let hyphen = text.iter().position(|c| *c == '-').expect("`-`");
    let char_x: Vec<f64> = run["charX"]
        .as_array()
        .expect("charX")
        .iter()
        .map(|x| x.as_f64().expect("문자 경계"))
        .collect();
    let advance = char_x[hyphen + 1] - char_x[hyphen];
    assert!(
        (5.5..7.0).contains(&advance),
        "`-` 전진폭 {advance:.2}px — 영문 슬롯이면 ≈6.1px, 한글 슬롯(휴먼명조 반각)이면 9.63px"
    );
}
