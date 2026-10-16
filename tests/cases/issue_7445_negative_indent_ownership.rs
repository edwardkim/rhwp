//! 저장 줄의 음수 정렬 여분은 원본 들여쓰기 접두 원점을 압축하지 않는다.
//! PII 11·21쪽과 규제분석 6쪽은 독립 Print의 96 CSSpx 좌표(±2)를 함께 보호한다.
//! PII 38쪽의 마지막 글리프는 저장 줄 프레임 안의 점유·내용 관계를 검사한다.
//! 자산관리 7쪽은 따옴표 윤곽의 추가 판독이 필요하여 원점 정렬·내용 관계만 검사한다.
//! 원본/Print SHA, 전쪽 Native/fresh WASM 및 페이지 선정 근거는
//! mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/negative-indent-20261016/README.md.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    model::control::Control,
    renderer::{
        font_environment::FontEnvironment,
        hwpunit_to_px,
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};
fn walk<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        walk(c, out);
    }
}
#[test]
fn compressed_body_keeps_the_authored_indent_aligned_with_its_continuation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(false);
    core.set_font_environment(Some(FontEnvironment::from_json(&std::fs::read_to_string(root.join("mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json")).unwrap()).unwrap())).unwrap();
    assert_eq!(core.page_count(), 66);
    let Control::Table(source) = &core.document().sections[0].paragraphs[219].controls[0] else {
        panic!("source table")
    };
    let cell = source
        .cells
        .iter()
        .find(|c| c.row == 1 && c.col == 0)
        .unwrap();
    let para = &cell.paragraphs[45];
    assert!(para.text.starts_with("  ③ "));
    let next_start = para.line_segs[1].text_start as usize;
    let tree = core.build_page_render_tree(20).unwrap();
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let tables=nodes.iter().filter(|n|matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(219)&&t.control_index==Some(0))).collect::<Vec<_>>();
    assert_eq!(tables.len(), 1);
    let painted_cell = tables[0]
        .children
        .iter()
        .find(|n| matches!(&n.node_type,RenderNodeType::TableCell(c)if c.row==1&&c.col==0))
        .unwrap();
    let mut content = Vec::new();
    walk(painted_cell, &mut content);
    let marker=content.iter().filter(|n|matches!(&n.node_type,RenderNodeType::TextRun(r)if r.para_index==Some(45)&&r.char_start==Some(2)&&r.text.starts_with("③ "))).collect::<Vec<_>>();
    assert_eq!(marker.len(), 1);
    let next=content.iter().filter(|n|matches!(&n.node_type,RenderNodeType::TextRun(r)if r.para_index==Some(45)&&r.char_start==Some(next_start))).collect::<Vec<_>>();
    assert_eq!(next.len(), 1);
    let lines = content
        .iter()
        .filter(|n| matches!(&n.node_type, RenderNodeType::TextLine(l) if l.para_index == Some(45)))
        .collect::<Vec<_>>();
    assert!(lines.len() >= 2 && para.line_segs.len() >= 3);
    for (line, index) in lines.iter().take(2).zip(0..2) {
        let start = para.line_segs[index].text_start as usize;
        let end = para.line_segs[index + 1].text_start as usize;
        let expected = para
            .text
            .chars()
            .skip(start)
            .take(end - start)
            .collect::<String>();
        assert_eq!(
            compact(&text(line, 45)),
            compact(&expected),
            "each first/continuation source line keeps its content, order and unique ownership"
        );
    }
    assert!(
        marker[0].bbox.y + marker[0].bbox.height <= next[0].bbox.y,
        "first line precedes continuation"
    );
    assert!(
        (marker[0].bbox.x - 97.9199930826823).abs() <= 2.0,
        "independent Print page21 circled marker x (96 CSSpx)"
    );
    assert!(
        (marker[0].bbox.x - next[0].bbox.x).abs() <= hwpunit_to_px(1, 96.0),
        "independent Print aligns first-line marker and continuation; marker={},next={}",
        marker[0].bbox.x,
        next[0].bbox.x
    );
}

fn text(node: &RenderNode, para: usize) -> String {
    let mut n = Vec::new();
    walk(node, &mut n);
    n.into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.para_index == Some(para) => Some(r.text.as_str()),
            _ => None,
        })
        .collect::<String>()
}
fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn marker(node: &RenderNode, para: usize, label: &str) -> (f64, f64) {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    let positions = nodes
        .into_iter()
        .filter_map(|n| {
            let RenderNodeType::TextRun(r) = &n.node_type else {
                return None;
            };
            if r.para_index != Some(para) {
                return None;
            }
            let spaces = r.text.chars().take_while(|c| *c == ' ').count();
            if !r.text.trim_start_matches(' ').starts_with(label) {
                return None;
            }
            let positions = EmbeddedTextMeasurer.compute_char_positions(&r.text, &r.style);
            Some((n.bbox.x + positions[spaces], n.bbox.y))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        positions.len(),
        1,
        "marker glyph has exactly one page owner, independent of text-run segmentation"
    );
    positions[0]
}
#[test]
fn asset_negative_fit_keeps_the_second_item_aligned_with_the_first() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let d = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/issue6031/3249937_asset_management_rules.hwpx")).unwrap(),
    )
    .unwrap();
    assert_eq!(d.page_count(), 60);
    let source = &d.document().sections[0].paragraphs;
    assert!(source[102].text.starts_with("  1) "));
    assert!(source[103].text.starts_with("  2) "));
    let page = d.build_page_render_tree(6).unwrap();
    let first = marker(&page.root, 102, "1) ");
    let second = marker(&page.root, 103, "2) ");
    assert!(first.1 < second.1, "authored list order");
    assert!(
        (first.0 - second.0).abs() <= hwpunit_to_px(1, 96.0),
        "independent Print has the same authored list origin: first={},second={}",
        first.0,
        second.0
    );
    for pi in [102, 103] {
        assert_eq!(
            compact(&text(&page.root, pi)),
            compact(&source[pi].text),
            "page7 paragraph content/order must survive without omissions or repeats"
        );
    }
}
#[test]
fn regulatory_negative_fit_keeps_list_origins_and_complete_cell_content() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut d = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/issue7445/80250_regulatory_analysis.hwp")).unwrap(),
    )
    .unwrap();
    d.set_hangul2024_compat(true);
    assert_eq!(d.page_count(), 17);
    let Control::Table(source) = &d.document().sections[0].paragraphs[15].controls[0] else {
        panic!("source table")
    };
    let cell = source
        .cells
        .iter()
        .find(|c| c.row == 1 && c.col == 1)
        .unwrap();
    assert!(cell.paragraphs[17].text.starts_with("  1. "));
    assert!(cell.paragraphs[18].text.starts_with("  2. "));
    let page = d.build_page_render_tree(5).unwrap();
    let mut all = Vec::new();
    walk(&page.root, &mut all);
    let table=all.iter().find(|n|matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(15)&&t.control_index==Some(0))).unwrap();
    let owner = table
        .children
        .iter()
        .find(|n| matches!(&n.node_type,RenderNodeType::TableCell(c)if c.row==1&&c.col==1))
        .unwrap();
    let first = marker(owner, 17, "1. ");
    let second = marker(owner, 18, "2. ");
    assert!(first.1 < second.1, "source list order");
    assert!(
        (first.0 - second.0).abs() <= hwpunit_to_px(1, 96.0),
        "both authored two-space prefixes share their original origin"
    );
    assert!(
        (first.0 - 416.8000081380208).abs() <= 2.0,
        "independent Print page6 list x (96 CSSpx)"
    );
    for pi in [17, 18] {
        assert_eq!(
            compact(&text(owner, pi)),
            compact(&cell.paragraphs[pi].text),
            "complete source cell paragraph on page6"
        );
    }
    assert!(
        first.0 >= owner.bbox.x && second.0 < owner.bbox.x + owner.bbox.width,
        "marker belongs inside the right cell"
    );
}

// 같은 끝점 동작의 완료 쪽 중 Native/fresh WASM 모두 100%인 11쪽을 px 검사에
// 우선 선정한다. 실제 결함을 발견한 38쪽(양쪽99.96%대)은 소스 줄 포함 관계로
// 함께 보존한다. 독립 Print의 좌표는 72pt → 96CSSpx로 환산했다.
fn terminal_glyph_geometry(line: &RenderNode, para: usize) -> (f64, f64, f64) {
    let mut nodes = Vec::new();
    walk(line, &mut nodes);
    let (node, run, visible) = nodes
        .into_iter()
        .rev()
        .find_map(|node| {
            let RenderNodeType::TextRun(run) = &node.node_type else {
                return None;
            };
            let visible = run.text.trim_end_matches(' ');
            (run.para_index == Some(para) && !visible.is_empty()).then_some((node, run, visible))
        })
        .unwrap();
    assert!(run.style.letter_spacing < 0.0, "authored negative tracking");
    let index = visible.chars().count() - 1;
    let fallback;
    let positions = if let Some(positions) = &run.layout_positions {
        positions
    } else {
        fallback = EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style);
        &fallback
    };
    let x = node.bbox.x + positions[index];
    let mut natural = run.style.clone();
    natural.letter_spacing = 0.0;
    natural.extra_word_spacing = 0.0;
    natural.extra_char_spacing = 0.0;
    let glyph = visible.chars().next_back().unwrap().to_string();
    let glyph_width = EmbeddedTextMeasurer.compute_char_positions(&glyph, &natural)[1];
    let end = x + glyph_width;
    let frame_end = line.bbox.x + line.bbox.width;
    assert!(
        end <= frame_end + hwpunit_to_px(2, 96.0),
        "last glyph belongs inside the source line frame, allowing only source-unit rounding: end={end}, frame={frame_end}"
    );
    (x, node.bbox.y + run.baseline, end)
}

#[test]
fn justified_terminal_glyph_preserves_the_source_frame_and_content() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(false);
    core.set_font_environment(Some(FontEnvironment::from_json(&std::fs::read_to_string(root.join("mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json")).unwrap()).unwrap())).unwrap();
    assert_eq!(core.page_count(), 66);

    let page = core.build_page_render_tree(10).unwrap();
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let tables = nodes.iter().filter(|n|matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(110)&&t.control_index==Some(0))).collect::<Vec<_>>();
    assert_eq!(tables.len(), 1);
    let cell = tables[0]
        .children
        .iter()
        .find(|n| matches!(&n.node_type,RenderNodeType::TableCell(c)if c.row==1&&c.col==2))
        .unwrap();
    let mut content = Vec::new();
    walk(cell, &mut content);
    let selected = content
        .iter()
        .filter(|n| {
            matches!(&n.node_type,RenderNodeType::TextLine(l)if l.para_index==Some(0))
                && text(n, 0).starts_with("이전하는 경우 개인정보 처리방침에")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        selected.len(),
        1,
        "source cell and line text identify the owner"
    );
    let (x, baseline, _) = terminal_glyph_geometry(selected[0], 0);
    assert!(
        (x - 685.7596028645833).abs() <= 2.0,
        "independent Print page11 final 개 x: {x}"
    );
    assert!(
        (baseline - 268.79998779296875).abs() <= 2.0,
        "independent Print page11 baseline: {baseline}"
    );
    let Control::Table(source) = &core.document().sections[0].paragraphs[110].controls[0] else {
        panic!("source table")
    };
    let source_cell = source
        .cells
        .iter()
        .find(|c| c.row == 1 && c.col == 2)
        .unwrap();
    assert_eq!(
        compact(&text(cell, 0)),
        compact(&source_cell.paragraphs[0].text),
        "complete cell paragraph and order"
    );

    let page = core.build_page_render_tree(37).unwrap();
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let selected = nodes.iter().filter(|n|matches!(&n.node_type,RenderNodeType::TextLine(l)if l.para_index==Some(363)&&l.line_index==Some(1))).collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert!(text(selected[0], 363).starts_with("제시하여 개인정보처리자의"));
    terminal_glyph_geometry(selected[0], 363);
    assert_eq!(
        compact(&text(&page.root, 363)),
        compact(&core.document().sections[0].paragraphs[363].text),
        "page38 paragraph has no missing or repeated text"
    );
}
