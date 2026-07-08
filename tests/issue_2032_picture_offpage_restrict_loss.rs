//! Issue #2032: floating 그림 off-page 좌표 완전 소실 — restrictInPage(flowWithText) 미반영.
//!
//! 배경 (issue #2027 조사에서 확인된 미해결 결함):
//! compute_object_position → layout_body_picture 에 페이지 경계 클램프가 전혀 없어,
//! vert_rel_to=Para + 큰 vertOffset 조합으로 좌표가 페이지 캔버스 밖이 되면 그림이
//! 어느 페이지에서도 보이지 않는다 ("완전 소실"). 다이얼로그의 restrictInPage
//! (쪽 영역 안으로 제한, HWP5 attr bit 13 = HWPX hp:pos@flowWithText, 기본 on) 도
//! 배치에 반영되지 않는다.
//!
//! 한컴 실제 동작 (samples/ta-pic-001-r-쪽영역안제한{,no}.hwp + pdf/ 2024 PDF 대조):
//! - restrictInPage=on: 개체가 쪽 영역을 벗어나지 않도록 배치된다 (절대 소실 없음).
//! - restrictInPage=off: 개체가 앵커 기준 좌표 그대로 컨테이너를 벗어나 배치된다.
//!
//! 표(floating table)에는 동일 시멘틱이 이미 구현되어 있다
//! (table_layout.rs compute_table_y: Para 기준 body 영역 클램프).

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

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

/// A4 세로 용지 높이 (HWPUNIT). english.hwp 용지 기준.
const A4_HEIGHT_HU: i32 = 84188;

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

fn picture_exists(core: &DocumentCore, para_idx: usize, ci: usize) -> bool {
    matches!(
        core.document().sections[0].paragraphs[para_idx]
            .controls
            .get(ci),
        Some(Control::Picture(_))
    )
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

fn set_props(core: &mut DocumentCore, para_idx: usize, ci: usize, json: &str, step: &str) {
    core.set_picture_properties_native(0, para_idx, ci, json)
        .unwrap_or_else(|e| panic!("[{step}] set_picture_properties_native({json}): {e}"));
}

/// restrictInPage=on + 페이지 높이를 넘는 vertOffset:
/// 한컴은 개체를 쪽 영역 안으로 끌어올린다 — 그림은 앵커 페이지 안에 완전히 보여야 한다.
///
/// 수정 전 현상: 그림 ImageNode 가 페이지 캔버스 아래(y > page_h)로 emit 되어
/// 어느 페이지에서도 보이지 않음 (완전 소실).
#[test]
fn restrict_on_offpage_offset_keeps_picture_in_page() {
    let mut core = load_core();
    let (para_idx, ci) = insert_test_picture(&mut core);

    // vert=Para + offset 90000 HU (A4 높이 84188 HU 초과) → 클램프 없이는 무조건 캔버스 밖
    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"vertRelTo":"Para","vertAlign":"Top","vertOffset":90000,"restrictInPage":true}"#,
        "restrict on + off-page offset",
    );
    assert!(
        picture_exists(&core, para_idx, ci),
        "그림 컨트롤이 모델에서 사라짐"
    );

    let (page, (page_w, page_h), img) = find_picture_render(&mut core, ci)
        .unwrap_or_else(|| panic!("restrictInPage=on 그림 ImageNode 가 어느 페이지에도 없음"));
    let visible =
        img.x < page_w && img.x + img.width > 0.0 && img.y < page_h && img.y + img.height > 0.0;
    assert!(
        visible,
        "restrictInPage=on 인데 그림이 페이지 {page} 캔버스({page_w:.0}x{page_h:.0}) 밖으로 소실: {img:?}"
    );
    assert!(
        img.y + img.height <= page_h + 0.5,
        "restrictInPage=on 그림 하단이 페이지 하단을 초과 (클램프 미적용): bottom={:.1} page_h={page_h:.1}",
        img.y + img.height
    );
}

/// restrictInPage=on + 페이지 하단에 걸치는 vertOffset:
/// 하단 경계에 걸치는 경우에도 개체 전체가 쪽 영역 안으로 들어와야 한다.
#[test]
fn restrict_on_partial_overflow_clamped_to_body_bottom() {
    let mut core = load_core();
    let (para_idx, ci) = insert_test_picture(&mut core);

    // 앵커(페이지 상단 부근) + 80000 HU → 그림 하단(80000+6000=86000)이 A4 높이 초과
    let offset = A4_HEIGHT_HU - PIC_HEIGHT_HU as i32 + 3000; // 81188: 하단 87188 > 84188
    set_props(
        &mut core,
        para_idx,
        ci,
        &format!(
            r#"{{"vertRelTo":"Para","vertAlign":"Top","vertOffset":{offset},"restrictInPage":true}}"#
        ),
        "restrict on + partial overflow",
    );

    let (_, (_, page_h), img) = find_picture_render(&mut core, ci)
        .unwrap_or_else(|| panic!("restrictInPage=on 그림 ImageNode 가 어느 페이지에도 없음"));
    assert!(
        img.y + img.height <= page_h + 0.5,
        "restrictInPage=on 그림 하단이 페이지 하단을 초과 (클램프 미적용): bottom={:.1} page_h={page_h:.1}",
        img.y + img.height
    );
}

/// restrictInPage=off + 페이지 높이를 넘는 vertOffset:
/// 한컴 정합 — 개체는 쪽 영역을 벗어날 수 있다 (클램프가 적용되면 안 된다).
/// 컨트롤은 모델에 남아 있어야 하고, 렌더가 emit 되는 경우 좌표는 앵커 기준
/// 원값(페이지 하단 아래)이어야 한다.
#[test]
fn restrict_off_offpage_offset_is_not_clamped() {
    let mut core = load_core();
    let (para_idx, ci) = insert_test_picture(&mut core);

    set_props(
        &mut core,
        para_idx,
        ci,
        r#"{"vertRelTo":"Para","vertAlign":"Top","vertOffset":90000,"restrictInPage":false}"#,
        "restrict off + off-page offset",
    );
    assert!(
        picture_exists(&core, para_idx, ci),
        "그림 컨트롤이 모델에서 사라짐"
    );

    // off 는 소실이 허용되는 케이스 (한컴 PDF 에서도 쪽 밖 영역은 보이지 않음).
    // 단, emit 된다면 클램프 없이 원 좌표(페이지 하단 아래)여야 한다.
    if let Some((page, (_, page_h), img)) = find_picture_render(&mut core, ci) {
        assert!(
            img.y + img.height > page_h,
            "restrictInPage=off 인데 그림이 페이지 {page} 안으로 클램프됨 (off 는 쪽 영역 제한이 없어야 함): {img:?}"
        );
    }
}
