//! 공식 대조 자료: `pdf/pr-review-all-open-20261116/`의 동일 입력별 한컴 2020 Hancom PDF Print.
//! 아래 원 PR의 웹한글기안기 설명은 이력이며 이번 검토의 공식 PDF가 아니다.
//! 새 저장 줄 없는 검사 3건은 함수별로 관계형/고정형을 구분하고 Native/fresh WASM 100.00%를 확인했다.
//! 주석 갱신: 2026-11-17T03:47:12+09:00
//!
//! 저장 줄이 있는 세로 가운데 정렬 글상자는 끝 문단의 비글자처럼 표(글 앞으로·자리차지)까지
//! 바깥 여백을 넣어 내용 높이를 잡고, 자리차지 표도 자기 세로 오프셋 자리에 놓는다.
//!
//! 재현물 `samples/textbox_stored_table/textbox_stored_{top,center}_{none,front_v*,tb_v*}.{hwpx,hwp}`:
//! 공개 샘플 `samples/issue8513/textbox_floating_table_offset.hwpx` 의 글상자(높이 30000 HU) 안을
//! `T1`~`T4` + 표 문단(빈 문단 + 표 5669 HU, 바깥 여백 283 HU, 세로 오프셋 0·2000 HU)으로 바꾸고 한컴
//! 저장 형식의 줄 정보(flags 0x60000, 줄 피치 1600)를 채운 합성본이다. HWP 는 같은 HWPX 를 rhwp 로
//! 변환한 것으로 저장 줄이 한컴 저장 표지 그대로 남는다.
//!
//! 기준: `pdf/textbox_stored_table/<이름>-webhwp-print.pdf`, `<이름>-hwp5-webhwp-print.pdf`(웹한글기안기
//! 인쇄). 96dpi px 실측:
//! - 가운데 정렬 `T1` 이 표 없는 대조군보다 위로: 오프셋 0 에서 34.9, 2000 에서 48.0(글 앞으로·자리차지 같음)
//!   = (표 바닥 + 바깥 아래 여백 − 끝 줄 바닥) / 2
//! - 위 정렬 표 칸 글자는 `T1` 아래 120.3(오프셋 0)·146.9(2000) — 자리차지 표도 오프셋을 따른다
//!
//! - 띠가 앵커 줄을 민 판(`_pushed`: 앵커 줄을 앞 문단 끝 + 띠 높이에 저장, 한컴 저장본과 같은 관계)도
//!   표는 앞 문단 끝 + 오프셋에 놓인다: 위 정렬 `T1`→표 칸 글자 77.8(오프셋 0)·104.3(2000)
//!   (웹한글기안기는 뒤 줄을 다시 짜므로 `A1` 자리는 비교하지 않는다)
//!
//! 수정 전 rhwp 는 가운데 정렬 높이에서 표를 빼(그림·도형만 넣음) `T1` 을 대조군 자리에 두었고,
//! 저장 줄 있는 글상자의 자리차지 표는 세로 오프셋과 바깥 위 여백을 버렸다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    model::control::Control,
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};

fn first_y(node: &RenderNode, text: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.trim() == text {
            return Some(node.bbox.y);
        }
    }
    node.children.iter().find_map(|child| first_y(child, text))
}

fn ys(name: &str) -> (f64, Option<f64>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/textbox_stored_table")
        .join(name);
    let bytes = std::fs::read(&path).expect("재현물");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 1, "{name}: 한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    (
        first_y(&tree.root, "T1").expect("T1 줄"),
        first_y(&tree.root, "BOX"),
    )
}

fn assert_near(what: &str, actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.5,
        "{what}: {actual:.1}px, 웹한글기안기 인쇄 {expected:.1}px"
    );
}

const EXTS: [&str; 2] = ["hwpx", "hwp"];

#[test]
fn centered_stored_textbox_counts_trailing_floating_table() {
    for ext in EXTS {
        let (none, _) = ys(&format!("textbox_stored_center_none.{ext}"));
        for kind in ["front", "tb"] {
            for (voff, rise) in [(0, 34.9), (2000, 48.0)] {
                let name = format!("textbox_stored_center_{kind}_v{voff}.{ext}");
                let (t1, _) = ys(&name);
                assert_near(&format!("{name} T1 이 대조군보다 위로"), none - t1, rise);
            }
        }
    }
}

#[test]
fn stored_textbox_topbottom_table_follows_vertical_offset() {
    for ext in EXTS {
        for kind in ["front", "tb"] {
            for (voff, below) in [(0, 120.3), (2000, 146.9)] {
                let name = format!("textbox_stored_top_{kind}_v{voff}.{ext}");
                let (t1, cell) = ys(&name);
                let cell = cell.expect("표 칸 글자");
                assert_near(&format!("{name} T1→칸 글자"), cell - t1, below);
            }
        }
    }
}

/// 관계형: 98% 미만인 밀린 앵커 문서는 측정한 77.8/104.3px 거리를 고정하지 않는다.
/// Native/fresh WASM: 오프셋 0은 97.48/97.37%, 2000HU는 97.40/97.30%다.
/// 원본의 이전 줄 끝·표 오프셋·바깥 여백과 실제 표/글의 소속을 비교한다.
#[test]
fn pushed_anchor_line_places_topbottom_table_at_previous_paragraph_end() {
    fn find_box(node: &RenderNode, predicate: fn(&RenderNode) -> bool) -> Option<BoundingBox> {
        if predicate(node) {
            return Some(node.bbox);
        }
        node.children
            .iter()
            .find_map(|child| find_box(child, predicate))
    }
    for ext in EXTS {
        for voff in [0, 2000] {
            let name = format!("textbox_stored_top_tb_v{voff}_pushed.{ext}");
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("samples/textbox_stored_table")
                .join(&name);
            let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
            let textbox = core.document().sections[0]
                .paragraphs
                .iter()
                .flat_map(|para| &para.controls)
                .find_map(|control| match control {
                    Control::Shape(shape) => shape.drawing()?.text_box.as_ref(),
                    _ => None,
                })
                .expect("원본 글상자");
            let (anchor, table) = textbox
                .paragraphs
                .iter()
                .enumerate()
                .find_map(|(index, para)| {
                    para.controls.iter().find_map(|control| match control {
                        Control::Table(table) => Some((index, table)),
                        _ => None,
                    })
                })
                .expect("원본의 자리차지 표");
            assert!(anchor > 0);
            let previous_line = textbox.paragraphs[anchor - 1]
                .line_segs
                .last()
                .expect("앵커 앞 문단의 저장 줄");
            let previous_end =
                previous_line.vertical_pos + previous_line.line_height + previous_line.line_spacing;
            assert_eq!(table.common.vertical_offset as i32, voff);
            let expected_offset = rhwp::renderer::hwpunit_to_px(
                previous_end + voff + i32::from(table.outer_margin_top),
                96.0,
            );
            let tree = core.build_page_render_tree(0).unwrap();
            let content = find_box(&tree.root, |node| {
                matches!(node.node_type, RenderNodeType::TextBox)
            })
            .unwrap();
            let painted = find_box(&tree.root, |node| {
                matches!(node.node_type, RenderNodeType::Table(_))
            })
            .unwrap();
            assert!(
                (painted.y - content.y - expected_offset).abs() < 1e-6,
                "{name}: 표는 밀린 빈 앵커 줄이 아닌 이전 줄 끝과 원본 오프셋·여백을 소비한다"
            );
            let text_y = |text| first_y(&tree.root, text).expect("원본 내용 보존");
            assert!(text_y("T1") < text_y("T2"));
            assert!(text_y("T2") < painted.y);
            assert!(painted.y <= text_y("BOX"));
            assert!(text_y("BOX") < painted.y + painted.height);
            assert!(painted.y + painted.height <= text_y("A1"));
            assert!(text_y("A1") < text_y("A2"));
        }
    }
}

// 아래 저장 줄 없는 5개 원본은 한컴 2020 Hancom PDF Print와 Native/fresh WASM 모두
// 100.00%에서 검토했다. 웹한글기안기 추정으로 TITLE이 표 없는 대조군과 같다고 하던
// 기존 가드는 독립 Print와 반대였으므로 원본 프레임·소속 관계로 교체한다.

#[derive(Debug)]
struct UnstoredFrame {
    outer: BoundingBox,
    content: BoundingBox,
    table: Option<BoundingBox>,
    title_y: f64,
    body_y: f64,
}

fn unstored_frame(name: &str) -> UnstoredFrame {
    unstored_frame_with_table_offset(name, None)
}

fn unstored_frame_with_table_offset(name: &str, table_offset: Option<i32>) -> UnstoredFrame {
    fn find_box(node: &RenderNode, predicate: fn(&RenderNode) -> bool) -> Option<BoundingBox> {
        if predicate(node) {
            return Some(node.bbox);
        }
        node.children
            .iter()
            .find_map(|child| find_box(child, predicate))
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/textbox_stored_table")
        .join(format!("textbox_nostored_header_{name}.hwpx"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("원본")).unwrap();
    if let Some(offset) = table_offset {
        let mut changed = 0;
        for para in &mut core.document_mut().sections[0].paragraphs {
            for control in &mut para.controls {
                let Control::Shape(shape) = control else {
                    continue;
                };
                let Some(textbox) = shape
                    .drawing_mut()
                    .and_then(|drawing| drawing.text_box.as_mut())
                else {
                    continue;
                };
                for inner in &mut textbox.paragraphs {
                    for control in &mut inner.controls {
                        if let Control::Table(table) = control {
                            assert_eq!(
                                table.common.vertical_offset as i32, -388,
                                "원본의 부호 있는 오프셋"
                            );
                            table.common.vertical_offset = offset as u32;
                            changed += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(changed, 1, "다른 속성을 바꾸지 않는 단일 표 대조군");
        core = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    }
    assert_eq!(core.page_count(), 1, "독립 Print와 같은 한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    fn occurrences(node: &RenderNode, text: &str) -> usize {
        usize::from(
            matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.trim() == text),
        ) + node
            .children
            .iter()
            .map(|child| occurrences(child, text))
            .sum::<usize>()
    }
    for text in ["TITLE", "BODY"] {
        assert_eq!(
            occurrences(&tree.root, text),
            1,
            "{name}: {text} 누락·중복 금지"
        );
    }
    let outer = find_box(&tree.root, |node| {
        matches!(node.node_type, RenderNodeType::Rectangle(_))
            && node
                .children
                .iter()
                .any(|child| matches!(child.node_type, RenderNodeType::TextBox))
    })
    .expect("글상자 바깥 프레임");
    let content = find_box(&tree.root, |node| {
        matches!(node.node_type, RenderNodeType::TextBox)
    })
    .expect("글상자 내용 프레임");
    UnstoredFrame {
        outer,
        content,
        table: find_box(&tree.root, |node| {
            matches!(node.node_type, RenderNodeType::Table(_))
        }),
        title_y: first_y(&tree.root, "TITLE").expect("제목 보존"),
        body_y: first_y(&tree.root, "BODY").expect("뒤 본문 보존"),
    }
}

/// 관계형: 제한 표의 바깥 여백을 포함한 프레임·제목 정렬·뒤 본문의 소속을 함께 보호한다.
/// Native/fresh WASM: 원본과 세 대조군·표 없는 대조군 모두 100.00%.
#[test]
fn unstored_restricted_table_shares_its_expanded_frame_with_following_body() {
    let none = unstored_frame("none");
    let zero = unstored_frame("front_zero_offset");
    let canonical_table = zero.table.expect("오프셋 0의 표");
    for name in ["front_tall", "front_zero_offset", "front_resolved_height"] {
        let frame = unstored_frame(name);
        let table = frame.table.expect("제한 표");
        assert!(
            frame.outer.height > none.outer.height,
            "{name}: 표와 두 여백을 담는 프레임"
        );
        assert!(
            frame.title_y < none.title_y,
            "{name}: 제한 표를 세로 정렬 높이에 포함"
        );
        assert!(
            table.y >= frame.content.y,
            "{name}: 음수 오프셋은 내용 위끝에서 제한"
        );
        assert!(
            table.y + table.height <= frame.content.y + frame.content.height,
            "{name}: 표는 확장한 내용 프레임 안에 있어야 한다"
        );
        assert!(
            frame.body_y + 1e-6 >= frame.outer.y + frame.outer.height,
            "{name}: 뒤 본문이 늘어난 흰 프레임에 가려지면 안 된다: {frame:?}"
        );
        assert!(
            (table.y - canonical_table.y).abs() < 1e-6,
            "{name}: 제한된 음수 오프셋·정리 전 공통 높이도 같은 물리 표 원점"
        );
    }
}

/// 관계형: 자유 배치 표는 글자의 가운데 정렬 전 원점을 유지하며 본문 프레임을 늘리지 않는다.
/// Native/fresh WASM: 자유 배치·표 없는 대조군 각각 100.00%.
#[test]
fn unstored_unrestricted_table_keeps_its_signed_offset_before_text_alignment() {
    let none = unstored_frame("none");
    let free = unstored_frame("front_unrestricted");
    let zero = unstored_frame_with_table_offset("front_unrestricted", Some(0));
    let table = free.table.expect("자유 배치 표");
    assert!(
        (free.title_y - none.title_y).abs() < 1e-6,
        "글자만 가운데 정렬한다"
    );
    assert!(
        (free.outer.height - none.outer.height).abs() < 1e-6,
        "자유 표는 본문 높이를 예약하지 않는다"
    );
    assert!(
        (table.y - zero.table.expect("오프셋 0 대조군").y - (-388.0 * 96.0 / 7200.0)).abs() < 1e-6,
        "자유 표의 음수 오프셋을 임의로 자르지 않는다: {free:?}, {zero:?}"
    );
    assert!(
        (free.title_y - zero.title_y).abs() < 1e-6,
        "자유 표만 옮겨도 글자 정렬은 그대로다"
    );
    assert!(
        free.body_y + 1e-6 >= free.outer.y + free.outer.height,
        "뒤 본문은 자기 프레임 뒤에 남는다"
    );
}

/// 고정형: 완료 원본 쪽의 독립 Hancom PDF 경계·본문 위치를 ±2px로 고정한다.
/// Native/fresh WASM 각각 100.00%, 같은 쪽 review·overlay 직접 판독 완료.
#[test]
fn completed_unstored_header_preserves_independent_print_frame_and_body_positions() {
    let frame = unstored_frame("front_tall");
    // 독립 PDF의 path 상자 세로 68.205pt와 BODY bbox 위끝 167.35031pt를 96dpi로 환산.
    // 경계 stroke와 논리 상자의 차이는 허용 오차 안에 있으며 기준 PDF 자체는 바꾸지 않는다.
    let expected_height = 68.205 * 96.0 / 72.0;
    let expected_body_y = 167.35031 * 96.0 / 72.0;
    assert!(
        (frame.outer.height - expected_height).abs() <= 2.0,
        "{frame:?}"
    );
    assert!((frame.body_y - expected_body_y).abs() <= 2.0, "{frame:?}");
}
