//! 관계형: 같은 host의 Square 앵커 줄과 TAC 줄의 소유·순서·뒤 본문 비겹침.
//! 측정 2026-10-30, 96 DPI, 색상 경계 보정 전 2px silhouette_raw_match_percent.
//! paper-square-table-tac-host.hwpx: 전체 1쪽, Native/최저 98.84484%, fresh WASM/최저 98.80408%.
//! same-host-full-row.hwp: 전체 1쪽, Native/최저 98.84715%, fresh WASM/최저 98.80599%.
//! same-host-with-spare-width.hwp: 전체 1쪽, Native/최저 98.84554%, fresh WASM/최저 98.80318%.
//! source 21ba44e3ea; 한컴 2024 Print, 전체 TSV·PNG·입력/글꼴 해시·실행 명령:
//! mydocs/pr/assets/pr_8144/README.md.
//!
//! 쪽·종이 기준 어울림 표와 같은 문단의 글자처럼 취급 표는 옆에 들어갈 폭이 없으면
//! 어울림 표의 바깥 아래 여백 아래 줄에 놓이고, 뒤 문단도 그 줄 아래에서 이어진다.
//!
//! 저장 host 는 뒤 문단을 위해 어울림 표 상자를 등록하지 않는다(뒤 문단 위치는 저장
//! vpos 가 증언한다, #7548). 그런데 host 자신의 글자처럼 취급 표 줄은 그 상자를 보지 않고
//! 문단 시작에 계상되어, 뒤 문단의 저장 TAC 줄 원점이 상자 높이만큼 위로 당겨졌다.
//! 표본은 한글 2024 가 다시 저장한 HWPX 다(내용은 무의미한 글자로 바꿨다).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/paper-square-table-tac-host.hwpx";

fn hu_to_px(hu: i16) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 쪽 render tree 의 (종류, 문단, 컨트롤, 위끝, 아래끝). 표 안 글줄은 뺀다.
fn boxes(doc: &HwpDocument, page: u32) -> Vec<(String, u64, Option<u64>, f64, f64)> {
    fn walk(node: &Value, out: &mut Vec<(String, u64, Option<u64>, f64, f64)>) {
        let kind = node["type"].as_str().unwrap_or_default();
        if matches!(kind, "Table" | "TextLine") {
            if let Some(pi) = node["pi"].as_u64() {
                let y = node["bbox"]["y"].as_f64().unwrap();
                let bottom = y + node["bbox"]["h"].as_f64().unwrap();
                out.push((kind.to_string(), pi, node["ci"].as_u64(), y, bottom));
            }
        }
        if kind != "Table" {
            for child in node["children"].as_array().into_iter().flatten() {
                walk(child, out);
            }
        }
    }
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
    let mut out = Vec::new();
    walk(&tree, &mut out);
    out
}

#[test]
fn host_inline_table_clears_paper_square_table_and_successors_follow() {
    assert_host_rows(SAMPLE);
}

#[test]
fn normal_hancom_full_width_table_owns_a_separate_row() {
    assert_host_rows("tests/fixtures/pr8144/same-host-full-row.hwp");
}

#[test]
fn normal_hancom_spare_width_table_shares_the_anchor_row() {
    assert_host_rows("tests/fixtures/pr8144/same-host-with-spare-width.hwp");
}

fn assert_host_rows(sample: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let mut doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    doc.set_hangul2024_compat(true);
    assert_eq!(doc.page_count(), 1, "독립 한글 출력과 같은 쪽 수");

    let host = &doc.document().sections[0].paragraphs[0];
    let mut square = None;
    let mut inline = None;
    for (index, control) in host.controls.iter().enumerate() {
        if let Control::Table(table) = control {
            if table.common.treat_as_char {
                inline = Some(index as u64);
            } else {
                square = Some((index as u64, table.outer_margin_bottom));
            }
        }
    }
    let (square, square_outer_bottom) = square.expect("host 의 쪽·종이 기준 어울림 표");
    let inline = inline.expect("같은 host 의 글자처럼 취급 표");

    let boxes = boxes(&doc, 0);
    let table = |pi: u64, ci: u64| {
        boxes
            .iter()
            .find(|(kind, p, c, _, _)| kind == "Table" && *p == pi && *c == Some(ci))
            .map(|(_, _, _, top, bottom)| (*top, *bottom))
            .unwrap_or_else(|| panic!("문단 {pi} 컨트롤 {ci} 표가 1쪽에 없다"))
    };
    let (_, square_bottom) = table(0, square);
    let (inline_top, inline_bottom) = table(0, inline);
    let floor = square_bottom + hu_to_px(square_outer_bottom);
    assert!(
        inline_top >= floor - 0.5,
        "글자처럼 취급 표({inline_top:.1})가 어울림 표 바깥 상자({floor:.1})와 겹친다"
    );

    let Control::Table(inline_source) = &host.controls[inline as usize] else {
        unreachable!()
    };
    let source_width = i64::from(inline_source.common.width)
        + i64::from(inline_source.outer_margin_left)
        + i64::from(inline_source.outer_margin_right);
    let frame_width = i64::from(host.line_segs[0].segment_width);
    if source_width > frame_width {
        let shape = host.char_shape_id_at(0).expect("앵커 줄의 원본 글자 모양");
        let em = f64::from(doc.document().doc_info.char_shapes[shape as usize].base_size) / 75.0;
        assert!(
            inline_top - floor >= em - 0.5,
            "폭을 넘는 TAC 앞에는 실제 앵커 줄이 있어야 한다: {sample}"
        );
    } else {
        assert!(
            (inline_top - floor).abs() <= 0.5,
            "같은 줄에 들어가는 TAC는 제목 바깥 상자 아래의 앵커 줄을 공유한다: {sample}"
        );
    }

    // 뒤 문단의 표와 글줄은 모두 host 표 아래에서 시작하고 서로 겹치지 않는다.
    let mut successors: Vec<_> = boxes
        .iter()
        .filter(|(_, pi, _, _, _)| *pi > 0)
        .map(|(kind, pi, _, top, bottom)| (kind.as_str(), *pi, *top, *bottom))
        .collect();
    assert!(
        successors.iter().any(|(kind, _, _, _)| *kind == "Table"),
        "뒤 문단의 글자처럼 취급 표가 있다"
    );
    for (kind, pi, top, _) in &successors {
        assert!(
            *top >= inline_bottom - 0.5,
            "문단 {pi} {kind}({top:.1})가 host 표 아래끝({inline_bottom:.1}) 위로 올라왔다"
        );
    }
    successors.sort_by(|a, b| a.2.total_cmp(&b.2));
    for pair in successors.windows(2) {
        let (kind_a, pi_a, _, bottom_a) = pair[0];
        let (kind_b, pi_b, top_b, _) = pair[1];
        assert!(
            top_b >= bottom_a - 0.5,
            "문단 {pi_a} {kind_a} 와 문단 {pi_b} {kind_b} 가 겹친다"
        );
    }
}
