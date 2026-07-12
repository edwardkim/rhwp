//! Issue #2027: 그림 "본문과의 배치" 반복 변경 시 line_segs 미복원 + undo 미기록 (HOP 보고 버그).
//!
//! 시나리오 (studio 그림 속성 다이얼로그가 보내는 diff JSON 을 그대로 모사):
//! 1. 본문에 그림 삽입 (insert_picture_native 기본 = floating/Square/Paper)
//! 2. 글자처럼 취급 on  → {"treatAsChar":true}
//! 3. 글자처럼 취급 off (어울림 복귀) → {"treatAsChar":false}
//! 4. 자리차지 → {"textWrap":"TopAndBottom"}
//! 5. 어울림 → {"textWrap":"Square"}
//! 6. 2~3 한 번 더 왕복
//!
//! 각 단계 후: 컨트롤 존재 + render tree ImageNode 존재 + bbox 페이지 교차를 단언한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 줄 크기 메트릭 비교 키: (text_start, line_height, text_height, baseline_distance, line_spacing).
///
/// vertical_pos / column_start / segment_width / tag 는 비교에서 제외한다 —
/// 텍스트 편집 표준 경로(reflow_line_segs + recalculate_section_vpos)가 원본 파일
/// 값 대신 자체 재계산 값을 쓰는 필드들이라, 편집 후 비트 동일성이 보장되지 않는다.
/// 본 테스트의 관심사는 "그림 높이로 부풀려진 줄 크기가 텍스트 기준으로 복원되는가"다.
type LineSegKey = (u32, i32, i32, i32, i32);

fn seg_keys(segs: &[LineSeg]) -> Vec<LineSegKey> {
    segs.iter()
        .map(|s| {
            (
                s.text_start,
                s.line_height,
                s.text_height,
                s.baseline_distance,
                s.line_spacing,
            )
        })
        .collect()
}

const SAMPLE: &str = "samples/basic/english.hwp";

/// 1x1 투명 PNG (67 bytes).
const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x62, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

const PIC_WIDTH_HU: u32 = 9000; // 3.17cm
const PIC_HEIGHT_HU: u32 = 6000; // 2.12cm

#[derive(Debug, Clone, Copy)]
struct ImageRender {
    control_index: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn collect_images(node: &RenderNode, out: &mut Vec<ImageRender>) {
    if let RenderNodeType::Image(img) = &node.node_type {
        if let Some(control_index) = img.control_index {
            out.push(ImageRender {
                control_index,
                x: node.bbox.x,
                y: node.bbox.y,
                width: node.bbox.width,
                height: node.bbox.height,
            });
        }
    }
    for child in &node.children {
        collect_images(child, out);
    }
}

fn load_core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("load {SAMPLE}: {e}"))
}

/// 본문 첫 텍스트 문단에 그림을 삽입하고 (para_idx, control_idx) 반환.
fn insert_test_picture(core: &mut DocumentCore) -> (usize, usize) {
    let para_idx = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| !p.text.is_empty())
        .unwrap_or(0);
    let result = core
        .insert_picture_native(
            0,
            para_idx,
            0,
            &[],
            TINY_PNG,
            PIC_WIDTH_HU,
            PIC_HEIGHT_HU,
            1,
            1,
            "png",
            "repro picture",
            Some(20000),
            Some(20000),
        )
        .expect("insert_picture_native");
    let json: serde_json::Value = serde_json::from_str(&result)
        .unwrap_or_else(|e| panic!("insert 결과 JSON 파싱 실패 `{result}`: {e}"));
    let ci = json["controlIdx"]
        .as_u64()
        .unwrap_or_else(|| panic!("insert 결과에 controlIdx 없음: {result}")) as usize;
    (para_idx, ci)
}

fn picture_control(
    core: &DocumentCore,
    para_idx: usize,
    ci: usize,
) -> Option<&rhwp::model::image::Picture> {
    match core.document().sections[0].paragraphs[para_idx]
        .controls
        .get(ci)
    {
        Some(Control::Picture(pic)) => Some(pic.as_ref()),
        _ => None,
    }
}

/// 전체 페이지 render tree 에서 control_index 로 그림 ImageNode 를 찾는다.
/// 반환: (페이지 번호, 페이지 bbox(w,h), ImageRender)
fn find_picture_render(
    core: &mut DocumentCore,
    ci: usize,
) -> Option<(u32, (f64, f64), ImageRender)> {
    let pages = core.page_count();
    for page in 0..pages {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render tree page {page}: {e}"));
        let mut images = Vec::new();
        collect_images(&tree.root, &mut images);
        if let Some(img) = images.iter().find(|img| img.control_index == ci) {
            return Some((page, (tree.root.bbox.width, tree.root.bbox.height), *img));
        }
    }
    None
}

/// 단계별 불변식: 컨트롤 존재 + ImageNode 존재 + bbox 가 페이지 캔버스와 교차.
fn assert_picture_visible(core: &mut DocumentCore, para_idx: usize, ci: usize, step: &str) {
    let pic = picture_control(core, para_idx, ci)
        .unwrap_or_else(|| panic!("[{step}] 그림 컨트롤이 모델에서 사라짐"));
    let (tac, wrap, vo, ho) = (
        pic.common.treat_as_char,
        pic.common.text_wrap,
        pic.common.vertical_offset,
        pic.common.horizontal_offset,
    );
    let (page, (page_w, page_h), img) = find_picture_render(core, ci).unwrap_or_else(|| {
        panic!(
            "[{step}] 그림 ImageNode 가 어느 페이지에도 없음 (tac={tac}, wrap={wrap:?}, v_off={vo}, h_off={ho})"
        )
    });
    let intersects =
        img.x < page_w && img.x + img.width > 0.0 && img.y < page_h && img.y + img.height > 0.0;
    assert!(
        intersects,
        "[{step}] 그림 bbox 가 페이지 {page} 캔버스({page_w:.0}x{page_h:.0}) 밖: {img:?} (tac={tac}, wrap={wrap:?}, v_off={vo}, h_off={ho})"
    );
}

fn set_props(core: &mut DocumentCore, para_idx: usize, ci: usize, json: &str, step: &str) {
    core.set_picture_properties_native(0, para_idx, ci, json)
        .unwrap_or_else(|e| panic!("[{step}] set_picture_properties_native({json}): {e}"));
}

/// 시나리오 재현: 삽입 → 글자처럼 on/off → 자리차지 → 어울림 → 재왕복.
/// 각 단계에서 그림이 페이지에 보여야 한다.
#[test]
fn picture_survives_wrap_and_tac_toggle_sequence() {
    let mut core = load_core();
    let (para_idx, ci) = insert_test_picture(&mut core);
    assert_picture_visible(&mut core, para_idx, ci, "1.삽입(floating/Square/Paper)");

    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"treatAsChar":true}"#,
        "2.글자처럼 on",
    );
    assert_picture_visible(&mut core, para_idx, ci, "2.글자처럼 on");

    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"treatAsChar":false}"#,
        "3.글자처럼 off(어울림)",
    );
    assert_picture_visible(&mut core, para_idx, ci, "3.글자처럼 off(어울림)");

    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"textWrap":"TopAndBottom"}"#,
        "4.자리차지",
    );
    assert_picture_visible(&mut core, para_idx, ci, "4.자리차지");

    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"textWrap":"Square"}"#,
        "5.어울림",
    );
    assert_picture_visible(&mut core, para_idx, ci, "5.어울림");

    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"treatAsChar":true}"#,
        "6.글자처럼 on(2회차)",
    );
    assert_picture_visible(&mut core, para_idx, ci, "6.글자처럼 on(2회차)");

    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"treatAsChar":false}"#,
        "7.글자처럼 off(2회차)",
    );
    assert_picture_visible(&mut core, para_idx, ci, "7.글자처럼 off(2회차)");
}

/// tac 토글 왕복 후 앵커 문단 line_segs 가 원본으로 복원되는지 (마이그레이션 비가역성 검증).
#[test]
fn tac_roundtrip_preserves_anchor_line_segs() {
    let mut core = load_core();
    let (para_idx, ci) = insert_test_picture(&mut core);

    let before = seg_keys(&core.document().sections[0].paragraphs[para_idx].line_segs);

    set_props(&mut core, para_idx, ci, r#"{"treatAsChar":true}"#, "tac on");
    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"treatAsChar":false}"#,
        "tac off",
    );

    let after = seg_keys(&core.document().sections[0].paragraphs[para_idx].line_segs);
    assert_eq!(
        before, after,
        "tac on→off 왕복 후 앵커 문단 line_segs 가 원본과 달라짐 (비가역 마이그레이션)"
    );
}

/// 대조 실험: core 스냅샷 undo 는 배치 변경을 완전히 복원해야 한다.
/// (이 테스트가 통과하면 'undo 미복원'은 studio 의 기록 누락이 원인)
#[test]
fn snapshot_restore_recovers_picture_placement() {
    let mut core = load_core();
    let (para_idx, ci) = insert_test_picture(&mut core);

    let before_common = picture_control(&core, para_idx, ci)
        .expect("삽입 직후 그림")
        .common
        .clone();
    let before_segs = seg_keys(&core.document().sections[0].paragraphs[para_idx].line_segs);

    let snapshot_id = core.save_snapshot_native();

    set_props(&mut core, para_idx, ci, r#"{"treatAsChar":true}"#, "tac on");
    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"treatAsChar":false}"#,
        "tac off",
    );
    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"textWrap":"TopAndBottom"}"#,
        "자리차지",
    );

    core.restore_snapshot_native(snapshot_id)
        .expect("restore_snapshot_native");

    let after_common = &picture_control(&core, para_idx, ci)
        .unwrap_or_else(|| panic!("스냅샷 복원 후 그림 컨트롤이 사라짐"))
        .common;
    assert_eq!(
        before_common.treat_as_char, after_common.treat_as_char,
        "스냅샷 복원 후 treat_as_char 불일치"
    );
    assert_eq!(
        before_common.text_wrap, after_common.text_wrap,
        "스냅샷 복원 후 text_wrap 불일치"
    );
    assert_eq!(
        before_common.vertical_offset, after_common.vertical_offset,
        "스냅샷 복원 후 vertical_offset 불일치"
    );
    assert_eq!(
        before_common.horizontal_offset, after_common.horizontal_offset,
        "스냅샷 복원 후 horizontal_offset 불일치"
    );
    let after_segs = seg_keys(&core.document().sections[0].paragraphs[para_idx].line_segs);
    assert_eq!(
        before_segs, after_segs,
        "스냅샷 복원 후 앵커 문단 line_segs 불일치"
    );
    assert_picture_visible(&mut core, para_idx, ci, "스냅샷 복원 후");
}

/// floating(어울림) 그림이 HWP 저장 → 재로드에서 보존되는지.
/// (망가진파일1.hwp 에서 그림 컨트롤이 통째로 사라진 경로 검증)
#[test]
fn floating_picture_survives_hwp_save_roundtrip() {
    let mut core = load_core();
    let (para_idx, ci) = insert_test_picture(&mut core);

    // 어울림 floating 상태로 만든 뒤 (삽입 기본값이 이미 그 상태지만 tac 왕복도 거침)
    set_props(&mut core, para_idx, ci, r#"{"treatAsChar":true}"#, "tac on");
    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"treatAsChar":false}"#,
        "tac off",
    );

    let saved = core.export_hwp_with_adapter().expect("export_hwp");
    let reloaded = DocumentCore::from_bytes(&saved).expect("재로드");

    let picture_count: usize = reloaded.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| {
            p.controls
                .iter()
                .filter(|c| matches!(c, Control::Picture(_)))
                .count()
        })
        .sum();
    assert_eq!(
        picture_count, 1,
        "HWP 저장 → 재로드 후 그림 컨트롤이 보존되어야 함 (저장 단계 소실 여부 검증)"
    );
}
