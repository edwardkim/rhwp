//! 관계형·고정형: #8660 제한 개체의 독립 쪽 소유, 본문 순서·포함·예약과 독립 Print 쪽수.
//! 주석 갱신: 2026-11-14 13:26 KST. 직전 파일 변경: 2026-11-14 03:24:28 +09:00,
//! commit 351674bfa464d4a09385f7e5e0bc2755843eff1e(Git 확인; 측정 시각과 구분).
//! 측정: 2026-11-14 KST, source e2d7fb08b5, Native Skia/fresh 최적화 WASM Canvas,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, 색상 경계 보정 전 2px raw.
//! 원 8문서 전 13쪽: Native/fresh WASM 각각 100.00%, 독립 Desktop Print 쪽수 일치.
//! 실제 편집 12종 전 20쪽: Native/fresh WASM 모두 raw 최저 100.00%.
//! 첫 문단 X, 기준 문단 X, 실제 Enter, HWPX 재조판과 HWP/HWPX 반복 저장을 구분했다.
//! 독립 기준: `pdf/issue8660/*-hwp-2020.pdf`와 `pdf/pr9063/`의 한컴 2020 11.0.0.9136 Print(method 0, one-up).
//! 원본은 RHWP 합성 공개 HWP이며 실제 편집·저장 바이트도 Print했다. 웹 Print를 판정에 쓰지 않았다.
//! 증거: `mydocs/pr/assets/pr9063/evidence.md`와 최종 `correction/`의 전쪽 TSV·대표 PNG.
//! 위치는 다음 쪽 본문 상단·셀/본문 포함·앞뒤 내용 관계로 보호한다. 절대 페이지 px 핀은 추가하지 않는다.
//!
//! #8660: 쪽 영역 안으로 제한(`flowWithText`)한 문단 기준 개체가 본문 아래 끝을 넘으면
//! 한컴은 개체만 다음 쪽 본문 맨 위에 그리고, 기준 문단과 뒤 글은 원래 쪽에 남긴다.
//!
//! 기준: `pdf/issue8660/*-hwp-2020.pdf` (데스크톱 한컴 2020 Print). 보통 줄 30개 뒤 기준 문단
//! `ANCHOR` 에 그림(1908×2067 HU) 또는 사각형을 문단 기준 세로 오프셋으로 붙였다.
//! - 아래 끝이 본문 끝에 딱 맞으면 1쪽(`picture_in_front_fits`)
//! - 1px 넘으면 2쪽, 그림은 2쪽 본문 위 끝(`picture_in_front_over_1px`)
//! - 사각형·자리차지도 같다(`rect_top_and_bottom_over_12px`)
//! - 2쪽에 뒤 글이 이어지면 자리차지 그림 아래에서 글이 시작한다
//!   (`picture_top_and_bottom_over_12px_text_follows`, 2쪽 첫 줄이 그림 높이 20.67pt 만큼 내려감)
//! - 제한을 끄면 같은 쪽에 그대로 둔다(`picture_unrestricted_over_12px`, 1쪽)
//!
//! 수정 전 rhwp 는 그리기 단계에서 개체를 같은 쪽 아래 끝으로 끌어올려 쪽이 하나 모자랐다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8660")
        .join(format!("{name}.hwp"));
    let bytes = std::fs::read(&path).expect("#8660 재현물");
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn collect<'a>(
    node: &'a RenderNode,
    pick: &dyn Fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if pick(node) {
        out.push(node);
    }
    for child in &node.children {
        collect(child, pick, out);
    }
}

fn boxes(core: &DocumentCore, page: u32, pick: &dyn Fn(&RenderNode) -> bool) -> Vec<BoundingBox> {
    let tree = core.build_page_render_tree(page).unwrap();
    let mut out = Vec::new();
    collect(&tree.root, pick, &mut out);
    out.into_iter().map(|node| node.bbox.clone()).collect()
}

fn body(core: &DocumentCore, page: u32) -> BoundingBox {
    boxes(core, page, &|node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .into_iter()
    .next()
    .expect("본문 영역")
}

fn is_image(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Image(_))
}

fn is_rect(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Rectangle(_))
}

fn text_lines(core: &DocumentCore, page: u32) -> Vec<BoundingBox> {
    boxes(core, page, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
    })
}

fn assert_object_moved_to_next_page_top(name: &str, pick: &dyn Fn(&RenderNode) -> bool) {
    let core = load(name);
    assert_eq!(core.page_count(), 2, "{name}: 데스크톱 한컴 Print는 2쪽");
    assert!(
        boxes(&core, 0, pick).is_empty(),
        "{name}: 1쪽에는 개체가 없어야 한다"
    );
    let objects = boxes(&core, 1, pick);
    let object = objects.first().expect("2쪽 개체");
    let top = body(&core, 1).y;
    assert!(
        (object.y - top).abs() < 1.0,
        "{name}: 개체는 2쪽 본문 위 끝({top:.2}px)에 놓인다: {:.2}px",
        object.y
    );
    assert!(
        !text_lines(&core, 0).is_empty(),
        "{name}: 기준 문단 글은 1쪽에 남는다"
    );
}

#[test]
fn restricted_picture_that_fits_stays_on_the_anchor_page() {
    let core = load("picture_in_front_fits");
    assert_eq!(core.page_count(), 1, "본문 끝에 딱 맞으면 1쪽");
    assert_eq!(boxes(&core, 0, &is_image).len(), 1);
}

#[test]
fn restricted_picture_over_body_bottom_moves_to_next_page_top() {
    assert_object_moved_to_next_page_top("picture_in_front_over_1px", &is_image);
}

#[test]
fn restricted_top_and_bottom_rect_moves_to_next_page_top() {
    assert_object_moved_to_next_page_top("rect_top_and_bottom_over_12px", &is_rect);
}

#[test]
fn unrestricted_picture_stays_on_the_anchor_page() {
    let core = load("picture_unrestricted_over_12px");
    assert_eq!(core.page_count(), 1, "제한을 끈 그림은 같은 쪽에 남는다");
    assert_eq!(boxes(&core, 0, &is_image).len(), 1);
}

#[test]
fn following_text_starts_below_moved_top_and_bottom_picture() {
    let core = load("picture_top_and_bottom_over_12px_text_follows");
    assert_eq!(core.page_count(), 2);
    let images = boxes(&core, 1, &is_image);
    let image = images.first().expect("2쪽 그림");
    let top = body(&core, 1).y;
    assert!((image.y - top).abs() < 1.0, "그림은 2쪽 본문 위 끝");
    let first_line = text_lines(&core, 1)
        .into_iter()
        .map(|line| line.y)
        .fold(f64::INFINITY, f64::min);
    assert!(
        first_line >= image.y + image.height - 0.5,
        "자리차지 그림 아래에서 2쪽 글이 시작한다: 그림 아래 {:.2}px, 첫 줄 {first_line:.2}px",
        image.y + image.height
    );
}

#[test]
fn restricted_picture_of_table_host_paragraph_moves_to_next_page_top() {
    // 기준 문단이 글자처럼 표를 품어도 같다. 표는 1쪽에 남고 그림만 2쪽 맨 위로 간다.
    assert_object_moved_to_next_page_top("picture_in_front_with_table_host_over_12px", &is_image);
    let core = load("picture_in_front_with_table_host_over_12px");
    assert!(
        !boxes(&core, 0, &|node| matches!(
            node.node_type,
            RenderNodeType::Table(_)
        ))
        .is_empty(),
        "기준 문단의 표는 1쪽에 남는다"
    );
}

// ── 넘기지 않아야 하는 경우와 앞에 표가 있는 경우(가드) ──
// 보통 줄 20개 뒤 글자처럼 표(높이 12000 HU) 문단, 그 뒤 기준 문단에 같은 그림을 붙였다.
// 데스크톱 한컴 Print: 그림 아래 끝이 본문 끝보다 6px 위면 1쪽에 그대로, 6px 넘으면 2쪽 맨 위.
// 앞에 표가 있어도 판정 기준은 같다(표 높이까지 더한 기준 문단 위치).

#[test]
fn restricted_picture_after_table_that_fits_stays_on_the_anchor_page() {
    let core = load("picture_in_front_after_table_fits_6px");
    assert_eq!(core.page_count(), 1, "본문 끝 6px 위에서 끝나면 1쪽");
    let images = boxes(&core, 0, &is_image);
    let image = images.first().expect("1쪽 그림");
    let body = body(&core, 0);
    assert!(
        image.y + image.height <= body.y + body.height + 0.5,
        "그림은 1쪽 본문 안에 그대로 있다: 그림 아래 {:.2}px, 본문 아래 {:.2}px",
        image.y + image.height,
        body.y + body.height
    );
}

#[test]
fn restricted_picture_after_table_over_body_bottom_moves_to_next_page_top() {
    assert_object_moved_to_next_page_top("picture_in_front_after_table_over_6px", &is_image);
}

fn rendered_text(core: &DocumentCore) -> String {
    fn append(node: &RenderNode, text: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            text.push_str(&run.text);
        }
        for child in &node.children {
            append(child, text);
        }
    }
    let mut text = String::new();
    for page in 0..core.page_count() {
        append(&core.build_page_render_tree(page).unwrap().root, &mut text);
    }
    text
}

fn assert_restricted_edit_state(
    core: &DocumentCore,
    name: &str,
    pages: u32,
    object_page: u32,
    expected_text: &str,
) {
    assert_eq!(core.page_count(), pages, "{name}: 독립 Desktop Print 쪽수");
    assert_eq!(
        rendered_text(core),
        expected_text,
        "{name}: 본문 순서·누락·중복"
    );
    let pick: &dyn Fn(&RenderNode) -> bool = if name.starts_with("rect_") {
        &is_rect
    } else {
        &is_image
    };
    for page in 0..pages {
        let objects = boxes(core, page, pick);
        assert_eq!(
            objects.len(),
            usize::from(page == object_page),
            "{name}: 개체 단일 쪽 소유"
        );
        if object_page > 0 && page == object_page {
            let frame = body(core, page);
            let object = &objects[0];
            assert!(
                (object.y - frame.y).abs() < 1.0,
                "{name}: 다음 쪽 본문 상단"
            );
            assert!(
                object.y + object.height <= frame.y + frame.height,
                "{name}: 본문 내부"
            );
            if name.contains("text_follows") {
                let first = text_lines(core, page)
                    .iter()
                    .map(|line| line.y)
                    .fold(f64::INFINITY, f64::min);
                assert!(
                    first >= object.y + object.height - 0.5,
                    "자리차지 예약과 뒤 첫 본문 줄"
                );
            }
        }
    }
}

fn assert_repeated_saves(
    core: &DocumentCore,
    name: &str,
    pages: u32,
    object_page: u32,
    expected_text: &str,
) {
    for hwpx in [false, true] {
        let bytes = if hwpx {
            core.export_hwpx_native().unwrap()
        } else {
            core.export_hwp_native().unwrap()
        };
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_restricted_edit_state(&reopened, name, pages, object_page, expected_text);
        let bytes2 = if hwpx {
            reopened.export_hwpx_native().unwrap()
        } else {
            reopened.export_hwp_native().unwrap()
        };
        assert_restricted_edit_state(
            &DocumentCore::from_bytes(&bytes2).unwrap(),
            name,
            pages,
            object_page,
            expected_text,
        );
    }
}

#[test]
fn editing_before_restricted_objects_preserves_ownership_and_text() {
    for (name, pages, object_page) in [
        ("picture_in_front_fits", 1, 0),
        ("picture_unrestricted_over_12px", 1, 0),
        ("picture_in_front_over_1px", 2, 1),
        ("rect_top_and_bottom_over_12px", 2, 1),
        ("picture_in_front_with_table_host_over_12px", 2, 1),
        ("picture_top_and_bottom_over_12px_text_follows", 2, 1),
        ("picture_in_front_after_table_fits_6px", 1, 0),
        ("picture_in_front_after_table_over_6px", 2, 1),
    ] {
        let mut core = load(name);
        let text = format!("X{}", rendered_text(&core));
        core.insert_text_native(0, 0, 0, "X").unwrap();
        assert_restricted_edit_state(&core, name, pages, object_page, &text);
        assert_repeated_saves(&core, name, pages, object_page, &text);
    }
}

#[test]
fn editing_the_anchor_preserves_deferral_through_both_save_formats() {
    let name = "picture_in_front_over_1px";
    let mut core = load(name);
    let text = rendered_text(&core).replacen("ANCHOR", "XANCHOR", 1);
    let host = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|para| {
            para.controls.iter().any(|ctrl| {
                matches!(
                    ctrl,
                    rhwp::model::control::Control::Picture(_)
                        | rhwp::model::control::Control::Shape(_)
                )
            })
        })
        .unwrap();
    core.insert_text_native(0, host, 0, "X").unwrap();
    assert_restricted_edit_state(&core, name, 2, 1, &text);
    assert_repeated_saves(&core, name, 2, 1, &text);
}

#[test]
fn inserting_a_real_line_before_a_fitted_object_uses_the_new_flow_origin() {
    let name = "picture_in_front_fits";
    let mut core = load(name);
    let text = rendered_text(&core);
    assert_eq!(core.page_count(), 1);
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    assert_restricted_edit_state(&core, name, 2, 1, &text);
    assert_repeated_saves(&core, name, 2, 1, &text);
}

#[test]
fn reopened_hwpx_uses_the_same_restricted_object_edit_contract() {
    let name = "picture_top_and_bottom_over_12px_text_follows";
    let original = load(name);
    let mut core = DocumentCore::from_bytes(&original.export_hwpx_native().unwrap()).unwrap();
    let text = format!("X{}", rendered_text(&core));
    core.insert_text_native(0, 0, 0, "X").unwrap();
    assert_restricted_edit_state(&core, name, 2, 1, &text);
    assert_repeated_saves(&core, name, 2, 1, &text);
}
