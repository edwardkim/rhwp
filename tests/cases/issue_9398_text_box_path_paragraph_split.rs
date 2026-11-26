//! #9398.
//! 표 칸 안 글상자의 문단도 경로 호출(`split_paragraph_in_cell_by_path`,
//! `merge_paragraph_in_cell_by_path`)로 나누고 합친다.
//!
//! 시험 문서: `cell_group_text_box_path.rs`와 같은 1×1 표 문서다. 본문 1문단의 표(컨트롤 0)
//! 셀 0, 셀 문단 1에 글상자 둘을 묶은 개체(컨트롤 0)가 있고, 글상자마다 "첫 상자"·"둘째 상자"를 쓴다.
//! 경로 커서·누르기는 이 글상자를 `[표 셀, 글상자]` 경로로 가리키지만(#1138), 경로 나누기·합치기는
//! 경로를 따라 표만 받아 Enter·Backspace가 "경로: 표가 아닙니다"로 거절됐다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;

type Path = [(usize, usize, usize); 2];

fn fixture_bytes() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/group_textbox_cell/cell-group-textbox.hwp"
    ))
    .unwrap()
}

/// 표(컨트롤 0) 셀 0 · 셀 문단 1 · 묶음(컨트롤 0)의 글상자 `child` · 그 글상자 문단 `para`.
fn text_box(child: usize, para: usize) -> Path {
    [(0, 0, 1), (0, child, para)]
}

/// 글상자 `child`의 문단 글을 차례로 읽는다(범위를 넘으면 멈춘다).
fn text_box_paragraphs(doc: &HwpDocument, child: usize) -> Vec<String> {
    (0..)
        .map_while(|para| {
            doc.get_text_in_cell_by_path(0, 1, &text_box(child, para), 0, 100)
                .ok()
        })
        .collect()
}

/// 바깥 표 셀 0의 문단 글.
fn outer_cell_paragraphs(doc: &HwpDocument) -> Vec<String> {
    (0..)
        .map_while(|para| {
            doc.get_text_in_cell_by_path(0, 1, &[(0, 0, para)], 0, 100)
                .ok()
        })
        .collect()
}

fn reopen(label: &str, bytes: &[u8]) -> HwpDocument {
    HwpDocument::new(bytes).unwrap_or_else(|e| panic!("{label} 다시 열기 실패: {e:?}"))
}

#[test]
fn split_and_merge_paragraph_in_text_box_inside_table_cell() {
    let mut doc = HwpDocument::from_bytes(&fixture_bytes()).unwrap();
    let outer_before = outer_cell_paragraphs(&doc);
    assert_eq!(text_box_paragraphs(&doc, 0), ["첫 상자"]);
    assert_eq!(text_box_paragraphs(&doc, 1), ["둘째 상자"]);

    // Enter: "첫 상자"의 "첫" 뒤에서 나눈다.
    let split: serde_json::Value = serde_json::from_str(
        &doc.split_paragraph_in_cell_by_path(0, 1, &text_box(0, 0), 1, None)
            .expect("표 칸 안 글상자 문단 나누기"),
    )
    .unwrap();
    assert_eq!(split["cellParaIndex"], 1, "새 문단 자리: {split}");
    assert_eq!(text_box_paragraphs(&doc, 0), ["첫", " 상자"]);
    assert_eq!(
        text_box_paragraphs(&doc, 1),
        ["둘째 상자"],
        "옆 글상자는 그대로"
    );
    assert_eq!(
        outer_cell_paragraphs(&doc),
        outer_before,
        "바깥 셀은 그대로"
    );
    // 나눈 문단에 커서를 둘 수 있다.
    doc.get_cursor_rect_by_path(
        0,
        1,
        r#"[{"controlIndex":0,"cellIndex":0,"cellParaIndex":1},{"controlIndex":0,"cellIndex":0,"cellParaIndex":1}]"#,
        0,
    )
    .expect("나눈 문단의 커서");

    // 편집 → 저장 → 다시 열기 → 재저장 → 다시 열기 (HWP, HWPX).
    for (label, first) in [
        ("hwp", doc.export_hwp_native().unwrap()),
        ("hwpx", doc.export_hwpx_native().unwrap()),
    ] {
        let reopened = reopen(label, &first);
        assert_eq!(
            text_box_paragraphs(&reopened, 0),
            ["첫", " 상자"],
            "{label}"
        );
        assert_eq!(text_box_paragraphs(&reopened, 1), ["둘째 상자"], "{label}");
        assert_eq!(outer_cell_paragraphs(&reopened), outer_before, "{label}");
        let second = if label == "hwp" {
            reopened.export_hwp_native().unwrap()
        } else {
            reopened.export_hwpx_native().unwrap()
        };
        let again = reopen(label, &second);
        assert_eq!(
            text_box_paragraphs(&again, 0),
            ["첫", " 상자"],
            "{label} 재저장"
        );
        assert_eq!(
            text_box_paragraphs(&again, 1),
            ["둘째 상자"],
            "{label} 재저장"
        );
        assert_eq!(
            outer_cell_paragraphs(&again),
            outer_before,
            "{label} 재저장"
        );
    }

    // Backspace: 둘째 문단을 첫 문단에 합친다.
    let merged: serde_json::Value = serde_json::from_str(
        &doc.merge_paragraph_in_cell_by_path(0, 1, &text_box(0, 1))
            .expect("표 칸 안 글상자 문단 합치기"),
    )
    .unwrap();
    assert_eq!(merged["cellParaIndex"], 0, "합친 문단 자리: {merged}");
    assert_eq!(merged["charOffset"], 1, "합친 자리: {merged}");
    assert_eq!(text_box_paragraphs(&doc, 0), ["첫 상자"]);
    assert_eq!(text_box_paragraphs(&doc, 1), ["둘째 상자"]);
    assert_eq!(outer_cell_paragraphs(&doc), outer_before);

    for (label, first) in [
        ("hwp", doc.export_hwp_native().unwrap()),
        ("hwpx", doc.export_hwpx_native().unwrap()),
    ] {
        let reopened = reopen(label, &first);
        assert_eq!(text_box_paragraphs(&reopened, 0), ["첫 상자"], "{label}");
        let second = if label == "hwp" {
            reopened.export_hwp_native().unwrap()
        } else {
            reopened.export_hwpx_native().unwrap()
        };
        let again = reopen(label, &second);
        assert_eq!(
            text_box_paragraphs(&again, 0),
            ["첫 상자"],
            "{label} 재저장"
        );
        assert_eq!(
            text_box_paragraphs(&again, 1),
            ["둘째 상자"],
            "{label} 재저장"
        );
        assert_eq!(
            outer_cell_paragraphs(&again),
            outer_before,
            "{label} 재저장"
        );
    }
}

#[test]
fn first_paragraph_of_text_box_inside_table_cell_does_not_merge() {
    let mut doc = HwpDocument::from_bytes(&fixture_bytes()).unwrap();
    assert!(doc
        .merge_paragraph_in_cell_by_path(0, 1, &text_box(0, 0))
        .is_err());
    assert_eq!(text_box_paragraphs(&doc, 0), ["첫 상자"]);
}

#[test]
fn path_through_a_non_text_box_step_is_still_rejected() {
    let mut doc = HwpDocument::from_bytes(&fixture_bytes()).unwrap();
    // 묶음에 셋째 글상자는 없다.
    assert!(doc
        .split_paragraph_in_cell_by_path(0, 1, &text_box(2, 0), 1, None)
        .is_err());
    // 셀 문단 0에는 컨트롤이 없다.
    assert!(doc
        .split_paragraph_in_cell_by_path(0, 1, &[(0, 0, 0), (0, 0, 0)], 1, None)
        .is_err());
    assert_eq!(text_box_paragraphs(&doc, 0), ["첫 상자"]);
    assert_eq!(text_box_paragraphs(&doc, 1), ["둘째 상자"]);
}

/// 글상자 안 표 칸(`[글상자, 표 셀]`)도 같은 경로 호출로 나누고 합친다.
/// 시험 문서: #8513의 글상자 문서. 본문 0문단의 글상자(컨트롤 2) 첫 문단에 1칸 표가 있고 칸 글은 "BOX"다.
#[test]
fn split_and_merge_paragraph_in_table_cell_inside_text_box() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue8513/textbox_floating_table_offset.hwpx"
    ))
    .unwrap();
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
    let cell = |para: usize| [(2, 0, 0), (0, 0, para)];
    let cell_paragraphs = |doc: &HwpDocument| -> Vec<String> {
        (0..)
            .map_while(|para| doc.get_text_in_cell_by_path(0, 0, &cell(para), 0, 100).ok())
            .collect()
    };
    assert_eq!(cell_paragraphs(&doc), ["BOX"]);

    doc.split_paragraph_in_cell_by_path(0, 0, &cell(0), 1, None)
        .expect("글상자 안 표 칸 문단 나누기");
    assert_eq!(cell_paragraphs(&doc), ["B", "OX"]);
    for (label, first) in [
        ("hwp", doc.export_hwp_native().unwrap()),
        ("hwpx", doc.export_hwpx_native().unwrap()),
    ] {
        let reopened = reopen(label, &first);
        assert_eq!(cell_paragraphs(&reopened), ["B", "OX"], "{label}");
        let second = if label == "hwp" {
            reopened.export_hwp_native().unwrap()
        } else {
            reopened.export_hwpx_native().unwrap()
        };
        assert_eq!(
            cell_paragraphs(&reopen(label, &second)),
            ["B", "OX"],
            "{label} 재저장"
        );
    }

    doc.merge_paragraph_in_cell_by_path(0, 0, &cell(1))
        .expect("글상자 안 표 칸 문단 합치기");
    assert_eq!(cell_paragraphs(&doc), ["BOX"]);
    for (label, first) in [
        ("hwp", doc.export_hwp_native().unwrap()),
        ("hwpx", doc.export_hwpx_native().unwrap()),
    ] {
        let reopened = reopen(label, &first);
        let second = if label == "hwp" {
            reopened.export_hwp_native().unwrap()
        } else {
            reopened.export_hwpx_native().unwrap()
        };
        assert_eq!(
            cell_paragraphs(&reopen(label, &second)),
            ["BOX"],
            "{label} 재저장"
        );
    }
}
