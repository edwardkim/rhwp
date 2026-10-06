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
//! 저장 사다리가 **있는** 다단 문단은 종전대로 트림한다(`#391` 의 본래 의도). 저장소 표본
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
