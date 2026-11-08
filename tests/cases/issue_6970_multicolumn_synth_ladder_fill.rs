//! [#6970] 다단 문서의 합성 사다리 문단이 단 채움을 문단마다 짧게 센다.
//!
//! ## 증상
//!
//! 저장 `LINE_SEG` 가 없는(합성 줄로 조판되는) 2단 문서에서 **단 0 이 가득 찼는데 단 1 로
//! 넘어가지 않고** 본문이 단 하단을 지나 계속 흐른다. 넘친 줄이 개체 위에 겹쳐 그려져
//! 문서가 읽을 수 없는 상태가 된다. 쪽수는 정답과 맞아서 쪽수 기준 검사로는 안 걸린다.
//!
//! ## 근인
//!
//! `flow_advance_height` 가 **다단이면 무조건 `height_for_fit`** 을 반환했다. 그 값은 문단의
//! trailing 줄간격을 뺀 값이다. 바로 아래 `#2279 ①` 가 단단 경로에 걸어 둔 가드가 그 트림의
//! 전제를 적는다 — *"저장 ladder 가 spacing 을 이미 반영하고 vpos-snap 이 좌표를 복원하는
//! 전제의 트림이므로, 합성(reflow) lineseg 문단은 ladder 가 없어 트림하면 sb·ls 가 흐름에서
//! 그냥 소실된다"*. 다단 경로만 그 가드 **앞에서** 빠져나갔다.
//!
//! 그래서 합성 사다리 문단이 많은 다단 문서에서는 문단마다 몇 px 씩 덜 세고, 그 합이 단
//! 하나를 통째로 넘긴다.
//!
//! ## 실측 (보고된 픽스처 `synth_no_ls_square_wrap.hwp`, 243KB)
//!
//! ```text
//!   내장 진단 RHWP_DIAG_ADV — 문단 151개
//!     Σadv 2195.1 · Σtotal 2936.7 · 차이 -741.6px  (문단당 6~10px)
//!   단 0: usedHeight 708.9 <= 가용 718.1  ->  "아직 남았다"
//!         실제 담은 항목 합 1000.4px (282px 초과)
//!
//!   layout-anomaly   수정 전  offCanvas 20 · overflow 16 · overlap 3 · 쪽수 3
//!                    수정 후  offCanvas  0 · overflow  0 · overlap 0 · 쪽수 3
//! ```
//!
//! 그 픽스처는 저장소에 넣지 않는다(`.hwp` 는 `samples/` 전체를 훑는 `ir_field_sweep`
//! 래칫에 걸린다). 대신 같은 형상 — **2단 + 저장 사다리 없는 문단 다수** — 을 합성해
//! 회계 불변식을 잠근다.
//!
//! ## 범위
//!
//! 저장 사다리의 다음 앵커가 간격을 복원할 수 있는 다단 문단은 종전대로 트림한다.
//! 복원 불가 흐름·독립적으로 인증한 단 영점의 추가 계약은 아래 KTX 셀 검사로 보호한다. 저장소 표본
//! 370개의 쪽수·off-canvas·overflow·overlap 이 수정 전후로 **전건 동일**하다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::wasm_api::HwpDocument;

/// 본문 문단 수 — 한 단을 넘기기에 충분하고, 문단당 트림 손실이 합산되어 드러난다.
const PARA_COUNT: usize = 90;

/// 2단 구역에 글자 문단만 담은 문서. 저장 `LINE_SEG` 는 쓰지 않는다(= 합성 사다리).
fn two_column_doc() -> Vec<u8> {
    let mut section = Section::default();

    // 2단 구역 정의.
    let mut column_def = rhwp::model::page::ColumnDef::default();
    column_def.column_count = 2;
    column_def.same_width = true;

    let mut para_shape = rhwp::model::style::ParaShape::default();
    para_shape.line_spacing_type = rhwp::model::style::LineSpacingType::Percent;
    para_shape.line_spacing = 160;

    let mut doc = Document::default();
    doc.doc_info.para_shapes.push(para_shape);
    let mut char_shape = rhwp::model::style::CharShape::default();
    char_shape.base_size = 1000;
    char_shape.ratios = [100; 7];
    doc.doc_info.char_shapes.push(char_shape);

    for i in 0..PARA_COUNT {
        let mut para = Paragraph::default();
        para.text = format!("column fill accounting line {i}");
        para.char_offsets = (0..para.text.chars().count() as u32).collect();
        para.para_shape_id = 0;
        para.char_shapes = vec![rhwp::model::paragraph::CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }];
        // line_segs 는 비운다 — 이 문서의 모든 문단이 합성 사다리다.
        if i == 0 {
            para.controls
                .push(rhwp::model::control::Control::SectionDef(Box::default()));
            para.controls
                .push(rhwp::model::control::Control::ColumnDef(column_def.clone()));
        }
        section.paragraphs.push(para);
    }
    doc.sections.push(section);
    rhwp::serializer::cfb_writer::serialize_hwp(&doc).expect("HWP5 직렬화")
}

/// 단의 (항목 높이 합, usedHeight).
fn column_accounting(bytes: &[u8]) -> Vec<(f64, f64)> {
    let doc = HwpDocument::from_bytes(bytes).expect("합성 문서가 열려야 한다");
    let pages = doc.dump_page_items_json(None);
    let pages = pages.as_array().expect("pages array");
    let mut out = Vec::new();
    for page in pages {
        for col in page["columns"].as_array().into_iter().flatten() {
            let mut sum = 0.0;
            for item in col["items"].as_array().into_iter().flatten() {
                if let Some(h) = item["height"]["total"].as_f64() {
                    sum += h;
                }
            }
            let used = col["usedHeight"].as_f64().unwrap_or(0.0);
            out.push((sum, used));
        }
    }
    out
}

/// 합성 사다리 문단만 있는 다단 단은 자기가 담은 항목 높이만큼 센다.
///
/// 수정 전에는 문단마다 trailing 줄간격을 빼고 세어, 단이 실제로는 넘쳤는데도 "아직
/// 남았다"고 판단해 다음 단으로 넘기지 않았다.
#[test]
fn multicolumn_synth_ladder_column_counts_what_it_holds() {
    let bytes = two_column_doc();
    let cols = column_accounting(&bytes);
    assert!(
        !cols.is_empty(),
        "단이 하나도 없다 — 합성 문서 전제가 깨졌다"
    );

    let short: Vec<_> = cols
        .iter()
        .enumerate()
        .filter(|(_, (sum, used))| *sum - *used > 1.0)
        .map(|(i, (sum, used))| (i, *sum, *used, *sum - *used))
        .collect();
    assert!(
        short.is_empty(),
        "단 채움을 담은 항목보다 짧게 셌다 (단 인덱스, 항목합, usedHeight, 부족분): {short:?}"
    );
}

/// 어느 단도 가용 높이를 넘겨 담지 않는다 — 넘겼다면 다음 단으로 갔어야 한다.
#[test]
fn no_column_holds_more_than_it_can() {
    let bytes = two_column_doc();
    let doc = HwpDocument::from_bytes(&bytes).expect("합성 문서가 열려야 한다");
    let pages = doc.dump_page_items_json(None);
    let pages = pages.as_array().expect("pages array");
    let mut over = Vec::new();
    for (pi, page) in pages.iter().enumerate() {
        let body_h = page["bodyArea"]["height"]
            .as_f64()
            .expect("bodyArea.height — 키 이름이 바뀌면 이 검사가 조용히 0건이 된다");
        for (ci, col) in page["columns"].as_array().into_iter().flatten().enumerate() {
            let mut sum = 0.0;
            for item in col["items"].as_array().into_iter().flatten() {
                if let Some(h) = item["height"]["total"].as_f64() {
                    sum += h;
                }
            }
            if sum > body_h + 1.0 {
                over.push((pi, ci, sum, body_h));
            }
        }
    }
    assert!(
        over.is_empty(),
        "단이 가용 높이보다 많이 담았다 (쪽, 단, 항목합, 본문높이): {over:?}"
    );
}

/// 전쪽 Native/fresh WASM이 각각 99.36494/99.38975%이며 셀 배경을 직접 판독했다.
/// 독립 PDF의 벡터 사각형(pt×4/3)에서 얻은 96DPI 경계다. 후보 좌표를 답안으로 복사하지 않는다.
/// 출처: task_m100_7445_rebase_repair_stage87.md의 KTX 1쪽·세 셀·입력/PDF SHA.
#[test]
fn stored_multicolumn_float_and_tac_cells_match_independent_print_frames() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    use rhwp::DocumentCore;
    use sha2::{Digest, Sha256};

    fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            collect(child, out);
        }
    }
    fn text(node: &RenderNode) -> String {
        let mut nodes = Vec::new();
        collect(node, &mut nodes);
        nodes
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::TextRun(run) => Some(run.display_or_text()),
                _ => None,
            })
            .collect::<String>()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    }
    let input = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/KTX-003.hwp"),
    )
    .expect("독립 Print 대응 원본");
    assert_eq!(
        Sha256::digest(&input)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "619bc919de3f2b8113fb9af0fcc808c754dc601ed9e66e49226280fd3d1ae032",
        "다른 원본을 같은 PDF 좌표로 고정하지 않는다"
    );
    let mut core = DocumentCore::from_bytes(&input).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 1);
    let page = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&page.root, &mut nodes);
    let mut mismatches = Vec::new();
    for (paragraph, row, col, expected, expected_text) in [
        (
            30,
            1,
            0,
            [
                530.7799886067709,
                101.225341796875,
                581.4440104166666,
                118.33601888020833,
            ],
            "서울",
        ),
        (
            32,
            2,
            1,
            [
                554.2733154296875,
                319.0279947916667,
                587.1973063151041,
                332.4613444010417,
            ],
            "용산",
        ),
        (
            33,
            2,
            1,
            [
                577.2880045572916,
                570.7333170572916,
                641.0586751302084,
                587.6840006510416,
            ],
            "서울",
        ),
    ] {
        let tables: Vec<_> = nodes
            .iter()
            .copied()
            .filter(|node| {
                matches!(&node.node_type,
            RenderNodeType::Table(table) if table.section_index == Some(0)
                && table.para_index == Some(paragraph) && table.control_index == Some(0))
            })
            .collect();
        assert_eq!(tables.len(), 1, "표 원문 소유·누락·중복: {paragraph}");
        let cells: Vec<_> = tables[0]
            .children
            .iter()
            .filter(|node| {
                matches!(&node.node_type,
            RenderNodeType::TableCell(cell) if cell.row == row && cell.col == col)
            })
            .collect();
        assert_eq!(cells.len(), 1, "원본 셀 주소의 단일 소유");
        let cell = cells[0];
        assert_eq!(text(cell), expected_text, "PDF 배경과 원문 셀의 내용 소속");
        let paints: Vec<_> = cell
            .children
            .iter()
            .filter(|node| matches!(&node.node_type, RenderNodeType::Rectangle(_)))
            .collect();
        assert_eq!(paints.len(), 1, "셀의 실제 칠 사각형을 검사한다");
        let b = paints[0].bbox;
        let actual = [b.x, b.y, b.x + b.width, b.y + b.height];
        for (axis, (actual, expected)) in actual.into_iter().zip(expected).enumerate() {
            if (actual - expected).abs() > 1.0 {
                mismatches.push((paragraph, row, col, axis, actual, expected));
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "독립 Print 셀 배경의 1px 경계 회귀: {mismatches:?}"
    );
}
