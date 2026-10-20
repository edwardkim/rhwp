//! [Issue #7017] 검증된 한컴 PUA 점선 `U+F081C` 를 **측정만** 0폭으로 봐서 가운데 정렬
//! 절취선 줄이 208.8px 우측으로 밀려 용지 밖으로 나간다
//! (`samples/issue7017/2528375-science-invention-contest-receipt.hwp` 1쪽).
//!
//! 기전: 같은 코드 포인트에 정책이 셋이었다.
//!
//! ```text
//!   text_measurement.rs   폭 0 (`U+FFFC` 오브젝트 자리표시자와 같은 갈래)
//!   expand_pua_display_text   `continue` — 안 그린다
//!   hancom_pua.rs         (0xF081C, "┈") — 반각 점선으로 그린다
//! ```
//!
//! 실제 렌더는 셋째를 타 글자마다 `0.5em × 장평` 만큼 전진하는데, 정렬에 쓰인 줄 폭은
//! PUA 60자를 전부 0으로 센 84.0px 이었다. 가운데 정렬이 그 폭으로 잡은 시작 x 는
//! `75.6 + (642.5 − 84.0)/2 = 354.85` 이고, 밀림량 `(501.6 − 84.0)/2 = 208.8px` 가
//! 관측값과 소수점까지 맞는다. `hancom_pua` 모듈 계약("paint **및 폭 측정**에만
//! 투영한다")이 지켜지지 않은 상태였다.
//!
//! 수정: `U+F081C` 를 0폭 분기에서 뺀다. 다른 `hancom_pua` 괘선 조각(`F0806`
//! `F0807` `F0810` …)과 같이 폴백 0.5em 으로 떨어지고, 그 값이 `┈` 의 렌더 전진폭과
//! 같다.
//!
//! 정답지 — 한컴 2020 (`lastSavedWith` = hancom-office-2010 8.5.8.1677 → §3.5.1 2020
//! 버킷, 새로 생성한 PDF):
//!
//! ```text
//!            줄 시작 x   마지막 글리프 우단 x
//!   한컴 2020   145.4px        647.7px
//!   결함        354.85px       856.5px      ← 용지 우단 793.3px 를 63.2px 넘는다
//! ```
//!
//! 같은 수정이 `#677` 복학원서의 한/글 정본 정합도 함께 올린다 —
//! `tests/svg_snapshot.rs` 의 `issue_677_bokhakwonseo_page1` 주석 참조.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::{
    hwpunit_to_px,
    render_tree::{RenderNode, RenderNodeType},
};

const SAMPLE: &str = "samples/issue7017/2528375-science-invention-contest-receipt.hwp";

// C447: 같은 원본 전1쪽 Native99.35164% / fresh WASM99.37179%, 독립2020 Print와
// review/overlay 직접 판독 완료. 원본 여백·내용 소속·점선60개 관계를 유지하며,
// 복구한 절·취·선의 독립 PDF 원점/기준선을 ±2CSSpx로 보호한다.
// 증적: stage8 C447 및 pua-flow-paint의 전쪽 TSV·대표 쌍.
#[test]
fn issue_7017_centered_pua_dotted_line_stays_on_the_paper() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("read sample");
    let source = rhwp::parser::parse_document(&bytes).expect("parse source");
    let cut_paras: Vec<_> = source.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, p)| p.text.contains('\u{F081C}'))
        .collect();
    assert_eq!(cut_paras.len(), 1, "원본 절취선 문단은 하나");
    let (para_index, source_cut) = cut_paras[0];
    let source_dots = source_cut
        .text
        .chars()
        .filter(|&c| c == '\u{F081C}')
        .count();
    assert_eq!(source_dots, 60, "독립 Print의 절취선 원문 보존");
    let page = &source.sections[0].section_def.page_def;
    let left = hwpunit_to_px(
        page.margin_left.try_into().expect("source left margin"),
        96.0,
    );
    let right = hwpunit_to_px(
        (page.width - page.margin_right)
            .try_into()
            .expect("source body right"),
        96.0,
    );
    let core = DocumentCore::from_bytes(&bytes).expect("open");
    assert_eq!(core.page_count(), 1, "독립2020 Print 전1쪽");
    let tree = core.build_page_render_tree(0).expect("page 1 tree");
    let mut cuts = Vec::new();
    cut_nodes(&tree.root, &mut cuts);
    assert_eq!(cuts.len(), 1, "절취선 문단의 누락·중복 금지");
    let (node, run) = cuts[0];
    assert_eq!(run.section_index, Some(0));
    assert_eq!(run.para_index, Some(para_index));
    assert_eq!(run.text, source_cut.text, "원문 소속·글자 순서 보존");
    assert!(
        node.bbox.x >= left && node.bbox.x + node.bbox.width <= right,
        "절취선의 실제 점유는 원본 본문 폭 안에 포함"
    );
    let before = node.bbox.x - left;
    let after = right - node.bbox.x - node.bbox.width;
    // 음수 자간은 마지막 글자 뒤에 그리지 않는 전진을 포함한다. 가운데 정렬한
    // 점유와 보이는 run의 여유 차이는 원본 자간 한 번+HU 반올림 안이어야 한다.
    // 현재 좌표를 기대값으로 옮기지 않고 원본 글자모양의 계약을 사용한다.
    let shape = &source.doc_info.char_shapes[source_cut.char_shapes[0].char_shape_id as usize];
    let trailing_tracking =
        hwpunit_to_px(shape.base_size, 96.0) * f64::from(shape.relative_sizes[0]) / 100.0
            * f64::from(shape.spacings[0]).abs()
            / 100.0;
    let hu_rounding = hwpunit_to_px(1, 96.0);
    assert!((before - after).abs() <= trailing_tracking + hu_rounding,
        "가운데 정렬의 좌우 차이는 원본 끝 자간 이내: {before} vs {after}, tracking {trailing_tracking}");

    let svg = core.render_page_svg_native(0).expect("page 1 svg");
    let dotted = glyph_xs(&svg, '┈');
    assert_eq!(
        dotted.len(),
        source_dots,
        "원본 PUA마다 실제 점선 하나: 소실·중복 금지"
    );
    assert!(
        dotted.windows(2).all(|pair| pair[0] < pair[1]),
        "점선의 원문 진행 순서"
    );
    assert!(
        dotted
            .iter()
            .all(|&x| x >= node.bbox.x && x < node.bbox.x + node.bbox.width),
        "paint의 점선도 측정한 절취선 점유 안에 포함"
    );
    let labels: Vec<_> = ['절', '취', '선']
        .into_iter()
        .map(|ch| {
            let xs = glyph_xs(&svg, ch);
            assert_eq!(xs.len(), 1, "절취선 라벨 {ch} 누락·중복 금지");
            xs[0]
        })
        .collect();
    assert!(
        dotted[0] < labels[0]
            && labels[0] < labels[1]
            && labels[1] < labels[2]
            && labels[2] < dotted[dotted.len() - 1],
        "점선 안의 절·취·선 순서 보존"
    );

    // 같은 독립 Print의 Batang Tm x=2087/2435/2784, y=2763와
    // device CTM (0.119935, -0.119869)를 원래 pt→96CSSpx로 변환한다.
    // 관측된 현재 좌표를 기대값으로 쓰지 않는다.
    for (actual, source_x) in labels.iter().zip([2087.0, 2435.0, 2784.0]) {
        let independent = source_x * 0.119_935 * 96.0 / 72.0;
        assert!(
            (actual - independent).abs() <= 2.0,
            "독립 Print 절취선 라벨 x ±2CSSpx: {actual} vs {independent}"
        );
    }
    let independent_baseline = 2763.0 * 0.119_869 * 96.0 / 72.0;
    assert!(
        (node.bbox.y + run.baseline - independent_baseline).abs() <= 2.0,
        "독립 Print 절취선 라벨 기준선 ±2CSSpx"
    );
}

fn cut_nodes<'a>(
    node: &'a RenderNode,
    out: &mut Vec<(&'a RenderNode, &'a rhwp::renderer::render_tree::TextRunNode)>,
) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains('\u{F081C}') {
            out.push((node, run));
        }
    }
    for child in &node.children {
        cut_nodes(child, out);
    }
}

/// SVG 에서 주어진 글자를 담은 `<text>` 들의 x 좌표(오름차순).
fn glyph_xs(svg: &str, needle: char) -> Vec<f64> {
    let mut xs: Vec<f64> = Vec::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let Some(close) = chunk[tag_end + 1..].find("</text>") else {
            continue;
        };
        if !chunk[tag_end + 1..tag_end + 1 + close].contains(needle) {
            continue;
        }
        if let Some(x) = attr(&chunk[..tag_end], "x") {
            xs.push(x);
        }
    }
    xs.sort_by(|a, b| a.partial_cmp(b).expect("finite x"));
    xs
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
